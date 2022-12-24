use super::DSPRegister;
use super::brr::{BRRInfo, BRREnd};
use super::envelope::{Envelope, ADSRMode};
use super::gaussian_table;
use super::SAMPLE_BUFFER_SIZE;
use super::FilterType;

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
        let sample = gaussian_interpolation(gaussian_idx as usize, &self.buffer, nibble_idx + 16);        

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

        if require_next_block {
            if self.is_loop {
                self.src_addr = self.loop_addr;
            } else {
                self.src_addr += 9;
            }

            let addr = self.src_addr as usize;                
            let brr_block = &Ram::global().ram[addr..addr + 9];                

            self.base_idx = 16;
            self.brr_info = BRRInfo::new(brr_block[0]);
            self.buffer.rotate_left(16);
            generate_new_sample(&brr_block[1..], &mut self.buffer, &self.brr_info);
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

        let addr = self.src_addr as usize;                
        let brr_block = &Ram::global().ram[addr..addr + 9];                

        self.base_idx = 16;
        self.brr_info = BRRInfo::new(brr_block[0]);                
        self.buffer.rotate_left(16);
        generate_new_sample(&brr_block[1..], &mut self.buffer, &self.brr_info);
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

    let buffer_idx_from = sample_idx as usize - 3;
    let buffer = &buffer[buffer_idx_from..];

    let out = table_idxs.into_iter().zip(buffer.into_iter())
        .map(|(table_idx, &sample)| { 
            (gaussian_table::GAUSSIAN_TABLE[table_idx] as i32 * sample as i32) >> 10
        })
        .sum::<i32>()
        .min(0x7FFF)
        .max(-0x8000);
    
    (out as i16) & !1
}

fn generate_new_sample(brrs: &[u8], buffer: &mut [i16; SAMPLE_BUFFER_SIZE], brr_info: &BRRInfo) -> () {    
    fn no_filter(sample: i32, _old: i32, _older: i32) -> i32 {
        sample
    }

    fn use_old(sample: i32, old: i32, _older: i32) -> i32 {
        let old_filter = old + ((-old) >> 4);
        sample + old_filter
    }

    fn use_all0(sample: i32, old: i32, older: i32) -> i32 {
        let old_filter = (old * 2) + ((old * -3) >> 5);
        let older_filter = -older + (older >> 4);

        sample + old_filter + older_filter
    }

    fn use_all1(sample: i32, old: i32, older: i32) -> i32 {
        let old_filter = (old * 2) + ((old * -13) >> 6);
        let older_filter = -older + ((older * 3) >> 4);

        sample + old_filter + older_filter
    }

    let nibbles = brrs.iter().map(|&brr| brr as i8).map(|brr| [brr >> 4, (brr << 4) >> 4]).flatten();
    let filter = match brr_info.filter {
        FilterType::NoFilter => no_filter,
        FilterType::UseOld => use_old,
        FilterType::UseAll0 => use_all0,
        FilterType::UseAll1 => use_all1,
    };

    fn shift_more_than_12(nibble: i8, _shamt: i32) -> i32 {
            // FullSNESではshamt > 12の場合は
            // nibble = nibble >> 3との記載がある。
            // 11の左シフトが必要か確認
            ((nibble as i8) >> 3) as i32
    }

    fn normal_shift(nibble: i8, shamt: i32) -> i32 {
        ((nibble as i32) << shamt) >> 1
    }

    let shift = if brr_info.shift_amount > 12 { shift_more_than_12 } else { normal_shift };

    let mut older = buffer[14] as i32;
    let mut old = buffer[15] as i32;

    nibbles.zip(16usize..).for_each(|(nibble, idx)| {
        let shamt = brr_info.shift_amount as i32;
        let sample = shift(nibble, shamt);
            
        let sample = filter(sample, old, older);
        let sample = sample.min(0x7FFF).max(-0x8000); 
        let sample = ((sample as i16) << 1) >> 1;       
        
        buffer[idx] = sample;
        older = old;
        old = sample as i32;
    }); 
}