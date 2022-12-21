use super::DSPRegister;
use super::brr::{BRRInfo, BRREnd};
use super::envelope::{Envelope, ADSRMode};
use super::gaussian_table;
use super::SAMPLE_BUFFER_SIZE;

use crate::emulator::ram::Ram;

#[derive(Clone)]
pub struct DSPBlock {
    pub reg: DSPRegister,
    
    pub buffer: [i16; SAMPLE_BUFFER_SIZE],
    pub base_idx: usize,

    pub start_addr: u16,
    pub loop_addr: u16,
    pub src_addr: u16,
    pub brr_info: BRRInfo,    
    pub envelope: Envelope,    

    pub pitch_counter: u16,    
    pub require_next: bool,
    pub is_loop: bool,

    pub sample_out: i16,
    pub sample_left: i16,
    pub sample_right: i16,
    pub echo_left: i16,
    pub echo_right: i16,

    pub key_on_delay: u8,
}

impl DSPBlock {
    pub const fn new() -> DSPBlock {
        DSPBlock {
            reg: DSPRegister::new(),

            buffer: [0; SAMPLE_BUFFER_SIZE],
            base_idx: 0,

            start_addr: 0,
            loop_addr: 0,
            src_addr: 0,
            brr_info: BRRInfo::empty(),            
            envelope: Envelope::empty(),

            pitch_counter: 0,            
            require_next: false,
            is_loop: false,

            sample_out: 0,
            sample_left: 0,
            sample_right: 0,
            echo_left: 0,
            echo_right: 0,

            key_on_delay: 0,
        }
    }
    
    pub fn init(&mut self, idx: usize, regs: &[u8; 128]) {
        self.reg = DSPRegister::new_with_init(idx, regs);    
    }

    pub fn flush(&mut self, before_out: Option<i16>, soft_reset: bool, cycle_counter: u16) -> () {                
        // fetch brr nibbles 
        let brr_info = &self.brr_info;
        
        // calculate related pitch
        let step = generate_additional_pitch(&self.reg, before_out);
        let (next_pitch, require_next_block) = self.pitch_counter.overflowing_add(step);        
        
        // filter sample
        let nibble_idx = ((self.pitch_counter >> 12) & 0x0F) as i8;
        let gaussian_idx = (self.pitch_counter >> 4) & 0xFF;
        let sample = gaussian_interpolation(gaussian_idx as usize, &self.buffer, self.base_idx as i8 + nibble_idx);        

        // envelope        
        let is_brr_end = brr_info.end == BRREnd::Mute;        
        let envelope_level = 
            if is_brr_end || soft_reset || self.key_on_delay > 0 {
                0
            } else {
                self.envelope.level
            };
        let envelope_mode =
            if is_brr_end || self.reg.key_off || soft_reset {
                ADSRMode::Release
            } else {
                self.envelope.adsr_mode
            };
        let envelope = self.envelope.copy(envelope_level, envelope_mode);
        let env = 
            if self.key_on_delay > 0 { Envelope::new(0, 0, envelope_mode) }
            else { envelope.envelope(self, cycle_counter) };
        let out = ((sample as i32) * (env.level as i32)) >> 11; // envelope bit width is 11, so dividing 2^11.

        //
        // POST PROCESS
        //    
                
        // renew dsp registers                
        let envx = (env.level >> 4) as u8;
        let outx = (out >> 7) as u8;        
        self.reg.env = envx;
        self.reg.out = outx;        
        self.reg.voice_end = brr_info.end == BRREnd::Loop || brr_info.end == BRREnd::Mute;
        self.reg.key_on_is_modified = false;
        self.require_next = require_next_block;
        self.is_loop = self.brr_info.end == BRREnd::Loop;                              
        self.envelope = env;
        self.reg.key_off = false;
        
        // renew buffer 
        if self.key_on_delay == 0 {
            self.pitch_counter = next_pitch;
        }          
        
        if soft_reset {
            self.reg.key_off = soft_reset;            
        }

        // output sample of left and right
        if self.key_on_delay <= 0 {
            let left_vol = (self.reg.vol_left as i8) as i32;
            let right_vol = (self.reg.vol_right as i8) as i32;
            
            self.sample_out = out as i16;
            self.sample_left = ((out * left_vol) >> 6) as i16;
            self.sample_right = ((out * right_vol) >> 6) as i16;
            
            self.echo_left = if self.reg.echo_enable { self.sample_left } else { 0 };
            self.echo_right = if self.reg.echo_enable { self.sample_right } else { 0 };
        } else {
            self.sample_out = 0;
            self.sample_left = 0;
            self.sample_right = 0;
            self.echo_left = 0;
            self.echo_right = 0;
            self.key_on_delay -= 1;
        }            
    }

    pub fn keyon(&mut self, table_addr: u16) {
        self.reg.key_on = true;
        self.reg.key_on_is_modified = true;
        self.envelope.adsr_mode = ADSRMode::Attack;
        self.envelope.level = 0;

        let table_addr = table_addr * 256 + (self.reg.srcn as u16 * 4);
        let start0 = Ram::global().read_ram(table_addr) as u16;
        let start1 = Ram::global().read_ram(table_addr + 1) as u16;
        let loop0 = Ram::global().read_ram(table_addr + 2) as u16;
        let loop1 = Ram::global().read_ram(table_addr + 3) as u16;

        self.pitch_counter = 0x0000;
                
        self.buffer.fill(0);
        self.base_idx = 0;

        self.start_addr = start0 | (start1 << 8);                
        self.loop_addr = loop0 | (loop1 << 8);
        self.src_addr = self.start_addr;
        self.key_on_delay = 5;
    }
}

fn generate_additional_pitch(reg: &DSPRegister, before_out: Option<i16>) -> u16 {
    let base_step = reg.pitch & 0x3FFF;
    
    if !reg.pmon_enable || before_out.is_none() {
        base_step
    } else {        
        let factor = before_out.unwrap();
        let factor = (factor >> 4) + 0x400;
        let ret = ((base_step as i32) * (factor as i32)) >> 10;

        // (ret & 0x7FFF) as u16
        if ret > 0x3FFF { 0x3FFF }
        else if ret < 0 { 0 }
        else            { ret as u16 }
    }
}

fn gaussian_interpolation(base_idx: usize, buffer: &[i16; SAMPLE_BUFFER_SIZE], sample_idx: i8) -> i16 {       
    let table_idxs = [
        0x0FF - base_idx,
        0x1FF - base_idx,
        0x100 + base_idx,
        0x000 + base_idx,
    ];

    let buffer_idx_from = (sample_idx - 3).rem_euclid(SAMPLE_BUFFER_SIZE as i8) as usize;
    let buffer_idxs = [
        (buffer_idx_from + 0) % SAMPLE_BUFFER_SIZE,
        (buffer_idx_from + 1) % SAMPLE_BUFFER_SIZE,
        (buffer_idx_from + 2) % SAMPLE_BUFFER_SIZE,
        (buffer_idx_from + 3) % SAMPLE_BUFFER_SIZE,
    ];

    let out = table_idxs.into_iter().zip(buffer_idxs.into_iter())
        .map(|(table_idx, buffer_idx)| { 
            (gaussian_table::GAUSSIAN_TABLE[table_idx] as i32 * buffer[buffer_idx] as i32) >> 10
        })
        .sum::<i32>()
        .min(0x7FFF)
        .max(-0x8000);
    
    (out as i16) & !1
}