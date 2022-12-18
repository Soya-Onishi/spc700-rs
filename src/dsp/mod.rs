mod gaussian_table;
mod envelope;
mod block;
mod brr;

use std::u8;
use std::i16;
use std::u16;

use array_macro::array;

use crate::emulator::ram::Ram;
use block::DSPBlock;
use brr::{BRRInfo, FilterType};

const NUMBER_OF_DSP: usize = 8;
const SAMPLE_BUFFER_SIZE: usize = 32;
pub const CYCLE_RANGE: u16 = 30720;

static mut GLOBAL_DSP: DSP = DSP::new();

pub struct DSP {
    blocks: [DSPBlock; 8],
    master_vol_left: u8,
    master_vol_right: u8,
    echo_vol_left: u8,
    echo_vol_right: u8,
    table_addr: u8, // DIR register

    // modification flag
    flag_is_modified: bool,    

    // FLG register    
    noise_frequency: u8,
    echo_buffer_enable: bool,
    is_mute: bool,
    soft_reset: bool,
    
    echo_feedback_volume: u8,
    echo_ring_buffer_addr: u16,
    echo_buffer_size: u8,
    echo_pos: u16,
    echo_buf_length: u16,
    fir_left: FIR,
    fir_right: FIR,

    sample_left_out: i16,
    sample_right_out: i16,

    // global dsp counter
    counter: u16,    
    pub sync_counter: u16,
    
    // These registers are unused in DSP.    
    unused_a: [u8; 8], 
    unused_b: [u8; 8],
    unused_1d: u8,
    unused_e: [u8; 8],    
}

#[derive(Clone)]
pub struct DSPRegister {
    pub vol_left: u8,
    pub vol_right: u8,
    pub pitch: u16,
    pub srcn: u8,
    pub adsr: u16,
    pub gain: u8,
    pub env: u8,
    pub out: u8,  
    
    pub key_on: bool,
    pub key_on_is_modified: bool, 
    pub key_off: bool,    

    pub voice_end: bool,
    pub noise_enable: bool,
    pub echo_enable: bool,
    pub pmon_enable: bool,        
}

impl DSPRegister {
    pub const fn new() -> DSPRegister {
        DSPRegister {
            vol_left: 0,
            vol_right: 0,
            pitch: 0,
            srcn: 0,
            adsr: 0,
            gain: 0,
            env: 0,
            out: 0,

            key_on: false,
            key_on_is_modified: false,        
            key_off: false,

            voice_end: false,
            noise_enable: false,
            echo_enable: false,
            pmon_enable: false,
        }
    }

    pub fn new_with_init(idx: usize, regs: &[u8; 128]) -> DSPRegister {
        let upper = (idx as u8) << 4;
        let addr = |idx: u8| -> usize { (upper | idx) as usize };
        let bit  = |idx: u8, data: u8| -> bool { (data & (1 << idx)) != 0 };

        let pitch = ((regs[addr(3)] as u16) << 8) | (regs[addr(2)] as u16);
        let adsr  = ((regs[addr(6)] as u16) << 8) | (regs[addr(5)] as u16);   
        
        DSPRegister {
            vol_left: regs[addr(0)],
            vol_right: regs[addr(1)],
            pitch: pitch,
            srcn: regs[addr(4)],
            adsr: adsr,
            gain: regs[addr(7)],
            env:  regs[addr(8)],
            out:  regs[addr(9)],
            
            key_on: bit(idx as u8, regs[0x4C]),
            key_on_is_modified: bit(idx as u8, regs[0x4C]),
            key_off: bit(idx as u8, regs[0x5C]),

            voice_end: bit(idx as u8, regs[0x7C]),
            noise_enable: bit(idx as u8, regs[0x3D]),
            echo_enable: bit(idx as u8, regs[0x4D]),
            pmon_enable: bit(idx as u8, regs[0x2D]),
        }
    }
}

struct FIR {
    regs: [i16; 8],    
    filter: [i16; 8],
}

impl FIR {
    pub const fn new() -> FIR {
        FIR {
            regs: [0; 8],
            filter: [0; 8],
        }
    }

    pub fn new_with_init(filter: [i16; 8]) -> FIR {
        FIR {
            regs: [0; 8],
            filter: filter,
        }
    }

    pub fn next(&mut self, value: i16) -> i16 {
        let mut new_regs = [0; 8];
        self.regs[1..].iter().zip(0..).for_each(|(&v, idx)| new_regs[idx] = v);
        new_regs[7] = value;

        let ret = new_regs.iter().zip(self.filter.iter())
            .map(|(&value, &filter)| ((value as i32) * (filter as i32)) >> 7)
            .fold(0, |acc, value| { acc + value });

        self.regs = new_regs;        
        
        if ret > 0x7FFF       {  0x7FFF }
        else if ret < -0x8000 { -0x8000 }
        else                  { ret as i16 }
    }
}

impl DSP {
    pub const fn new() -> DSP {
        let blocks = [
            DSPBlock::new(),
            DSPBlock::new(),
            DSPBlock::new(),
            DSPBlock::new(),
            DSPBlock::new(),
            DSPBlock::new(),
            DSPBlock::new(),
            DSPBlock::new(),
        ];
            
        let mut dsp = DSP {
            blocks: blocks,
            master_vol_left: 0,
            master_vol_right: 0,
            echo_vol_left: 0,
            echo_vol_right: 0,
            table_addr: 0,

            flag_is_modified: false,            

            noise_frequency: 0,
            echo_buffer_enable: false,
            is_mute: true,
            soft_reset: true,

            echo_feedback_volume: 0,
            echo_ring_buffer_addr: 0,
            echo_buffer_size: 0,
            echo_buf_length: 0,
            echo_pos: 0,
            fir_left: FIR::new(),
            fir_right: FIR::new(),

            sample_left_out: 0,
            sample_right_out: 0,

            counter: 0,
            sync_counter: 0,

            unused_a: [0; 8],
            unused_b: [0; 8],
            unused_1d: 0,
            unused_e: [0; 8],
        };

        dsp
    }

    pub fn init(regs: &[u8; 128]) {
        let dsp = DSP::global();
        let mut blocks = array![DSPBlock::new(); 8];
        for (idx, blk) in blocks.iter_mut().enumerate() {
            blk.init(idx, regs)
        } 

        // initialized by regs
        dsp.blocks = blocks;
        dsp.master_vol_left = regs[0x0C];
        dsp.master_vol_right = regs[0x1C];
        dsp.echo_vol_left = regs[0x2C];
        dsp.echo_vol_right = regs[0x3C];
        dsp.table_addr = regs[0x5D];

        dsp.flag_is_modified = true;        

        let flag = regs[0x6C];
        dsp.noise_frequency = flag & 0x1F;
        dsp.echo_buffer_enable = (flag & 0x20) == 0;
        dsp.is_mute = (flag & 0x40) > 0;
        dsp.soft_reset = (flag & 0x80) > 0;

        dsp.echo_feedback_volume = regs[0x0D];
        dsp.echo_ring_buffer_addr = (regs[0x6D] as u16) << 8;
        dsp.echo_buffer_size = regs[0x7D];
        
        let mut fir_coefficients = [0; 8];
        (0..8).map(|upper: usize| regs[(upper << 4) | 0x0F])
            .map(|v| (v as i8) as i16 )
            .zip(0..).for_each(|(v, idx)| fir_coefficients[idx] = v);

        dsp.fir_left = FIR::new_with_init(fir_coefficients.clone());
        dsp.fir_right = FIR::new_with_init(fir_coefficients.clone());
    }

    pub fn global() -> &'static mut DSP {
        unsafe { &mut GLOBAL_DSP }
    }

    pub fn cycles(&mut self, cycle_count: u16) -> () {
        self.sync_counter += cycle_count
    }

    pub fn flush(&mut self) -> () {       
        let flush_count = self.sync_counter / 64;
        if flush_count != 0 {
            let next_sync_counter = self.sync_counter % 64;
            self.exec_flush();
            self.sync_counter = next_sync_counter;
        } 
    }

    fn exec_flush(&mut self) -> () {        
        let table_addr = self.table_addr as u16;
        let soft_reset = self.soft_reset && self.flag_is_modified;
        let cycle_counter = self.counter;            

        self.blocks.iter_mut().fold(Option::<i16>::None, |before_out, blk| {
            // ready for next brr block by key on            
            if blk.reg.key_on && blk.reg.key_on_is_modified {
                let tab_addr = table_addr * 256 + (blk.reg.srcn as u16 * 4);
                let start0 = Ram::global().read_ram(tab_addr) as u16;
                let start1 = Ram::global().read_ram(tab_addr + 1) as u16;
                let loop0 = Ram::global().read_ram(tab_addr + 2) as u16;
                let loop1 = Ram::global().read_ram(tab_addr + 3) as u16;

                blk.pitch_counter = 0x0000;
                
                blk.buffer = [0; SAMPLE_BUFFER_SIZE];
                blk.base_idx = 0;

                blk.start_addr = start0 | (start1 << 8);                
                blk.loop_addr = loop0 | (loop1 << 8);
                blk.src_addr = blk.start_addr;
                blk.key_on_delay = 5;
            }

            // ready for next brr block by normal or loop
            if blk.require_next && !(blk.reg.key_on && blk.reg.key_on_is_modified) {
                if blk.is_loop {
                    blk.src_addr = blk.loop_addr;
                } else {
                    blk.src_addr += 9;
                }                
            }

            // fetch brr block
            if (blk.reg.key_on && blk.reg.key_on_is_modified) || blk.require_next {
                let addr = blk.src_addr as usize;                
                let brr_block = &Ram::global().ram[addr..addr + 9];                

                blk.base_idx = (blk.base_idx + 16) % SAMPLE_BUFFER_SIZE;
                blk.brr_info = BRRInfo::new(brr_block[0]);                
                generate_new_sample(&brr_block[1..], &mut blk.buffer, &blk.brr_info, blk.base_idx);
            }
                                                
            blk.flush(before_out, soft_reset, cycle_counter);
            Some(blk.sample_out)
        });

        let (left, right) = combine_all_sample(&self.blocks, self);         
        let (echo_left, echo_right) = combine_echo(&self.blocks);        
        let (left_echo, right_echo) = echo_process(echo_left, echo_right, self);

        let left_out = (left as i32) + (left_echo as i32);
        let right_out = (right as i32) + (right_echo as i32);
        let clamp = |v: i32| -> i16 {
            if v > 0x7FFF { 0x7FFF }
            else if v < -0x8000 { -0x8000 }
            else { v as i16 }
        };        
        
        self.flag_is_modified = false;
        self.counter = (self.counter + 1) % CYCLE_RANGE;
        self.sample_left_out = clamp(left_out);
        self.sample_right_out = clamp(right_out);
    }

    pub fn read_from_register(&mut self, addr: usize) -> u8 {
        let upper_base = (addr >> 4) & 0xF;
        let upper = if upper_base >= 0x8 { upper_base - 0x8 } else { upper_base}; // to address mirror
        let lower = addr & 0xF;

        match (upper as usize, lower as usize) {
            (upper, 0x0) => self.blocks[upper].reg.vol_left,
            (upper, 0x1) => self.blocks[upper].reg.vol_right,
            (upper, 0x2) => (self.blocks[upper].reg.pitch & 0xFF) as u8,
            (upper, 0x3) => ((self.blocks[upper].reg.pitch >> 8) & 0xFF) as u8,
            (upper, 0x4) => self.blocks[upper].reg.srcn,
            (upper, 0x5) => (self.blocks[upper].reg.adsr & 0xFF) as u8,
            (upper, 0x6) => ((self.blocks[upper].reg.adsr >> 8) & 0xFF) as u8,
            (upper, 0x7) => self.blocks[upper].reg.gain,
            (upper, 0x8) => self.blocks[upper].reg.env,
            (upper, 0x9) => self.blocks[upper].reg.out,
            (upper, 0xA) => self.unused_a[upper],
            (upper, 0xB) => self.unused_b[upper],
            (  0x0, 0xC) => self.master_vol_left,
            (  0x1, 0xC) => self.master_vol_right,
            (  0x2, 0xC) => self.echo_vol_left,
            (  0x3, 0xC) => self.echo_vol_right,
            (  0x4, 0xC) => 0,
            (  0x5, 0xC) => vec_to_u8(self.blocks.iter().map(|blk| blk.reg.key_off)),
            (  0x6, 0xC) => self.read_FLG(),
            (  0x7, 0xC) => vec_to_u8(self.blocks.iter().map(|blk| blk.reg.voice_end)),
            (  0x0, 0xD) => self.echo_feedback_volume,
            (  0x1, 0xD) => self.unused_1d,
            (  0x2, 0xD) => vec_to_u8(self.blocks.iter().map(|blk| blk.reg.pmon_enable)),
            (  0x3, 0xD) => vec_to_u8(self.blocks.iter().map(|blk| blk.reg.noise_enable)),
            (  0x4, 0xD) => vec_to_u8(self.blocks.iter().map(|blk| blk.reg.echo_enable)),
            (  0x5, 0xD) => self.table_addr,
            (  0x6, 0xD) => (self.echo_ring_buffer_addr >> 8) as u8,
            (  0x7, 0xD) => self.echo_buffer_size,
            (upper, 0xE) => self.unused_e[upper],
            (upper, 0xF) => self.fir_left.filter[upper] as u8,         
            _ => panic!("{:#06x} is not unexpected address", addr),
        }                
    }

    pub fn write_to_register(&mut self, addr: usize, data: u8) -> () {                
        // self.flush(ram);

        let upper = (addr >> 4) & 0x0F;
        let lower = addr & 0x0F;
        match (upper, lower) {
            (0x8..=0xF, _) => (), // 0x80..0xFF are read only mirrors of 0x00..0x7F
            (upper, 0x0) => self.blocks[upper].reg.vol_left = data,
            (upper, 0x1) => self.blocks[upper].reg.vol_right = data,
            (upper, 0x2) => {
                let old_pitch = self.blocks[upper].reg.pitch;
                let assigned = data as u16;
                let new_pitch = (old_pitch & 0xFF00) | assigned;

                self.blocks[upper].reg.pitch = new_pitch;
            }
            (upper, 0x3) => {
                let old_pitch = self.blocks[upper].reg.pitch;
                let assigned = (data as u16 & 0x3F) << 8;
                let new_pitch = (old_pitch & 0x00FF) | assigned;

                self.blocks[upper].reg.pitch = new_pitch;
            }
            (upper, 0x4) => self.blocks[upper].reg.srcn = data,
            (upper, 0x5) => {
                let old_adsr = self.blocks[upper].reg.adsr;                
                let new_adsr = (old_adsr & 0xFF00) | data as u16;

                self.blocks[upper].reg.adsr = new_adsr
            }
            (upper, 0x6) => {
                let old_adsr = self.blocks[upper].reg.adsr;
                let new_adsr = (old_adsr & 0x00FF) | ((data as u16) << 8);

                self.blocks[upper].reg.adsr = new_adsr;
            }
            (upper, 0x7) => self.blocks[upper].reg.gain = data,
            (upper, 0x8) => self.blocks[upper].reg.env = data,
            (upper, 0x9) => self.blocks[upper].reg.out = data,
            (upper, 0xA) => self.unused_a[upper] = data,
            (upper, 0xB) => self.unused_b[upper] = data,
            (  0x0, 0xC) => self.master_vol_left = data,
            (  0x1, 0xC) => self.master_vol_right = data,
            (  0x2, 0xC) => self.echo_vol_left = data,
            (  0x3, 0xC) => self.echo_vol_right = data,
            (  0x4, 0xC) => {                
                let bools = u8_to_vec(data);                
                self.blocks.iter_mut()
                    .zip(bools)
                    .filter(|(_, is_on)| *is_on)
                    .for_each(|(blk, is_on)| { 
                        blk.reg.key_on = is_on;
                        blk.reg.key_on_is_modified = is_on;
                    });
            }
            (  0x5, 0xC) => {
                let bools = u8_to_vec(data);
                self.blocks.iter_mut().zip(bools).for_each(|(blk, is_off)| {
                    blk.reg.key_off = is_off;
                });
            }
            (  0x6, 0xC) => {
                let noise_frequency = data & 0x1F;
                let echo_buffer_enable = (data & 0x20) == 0;
                let is_mute = (data & 0x40) > 0;
                let soft_reset = (data & 0x80) > 0;

                self.flag_is_modified = true;
                self.noise_frequency = noise_frequency;
                self.echo_buffer_enable = echo_buffer_enable;
                self.is_mute = is_mute;
                self.soft_reset = soft_reset;
            }
            (  0x7, 0xC) => self.blocks.iter_mut().for_each(|blk| blk.reg.voice_end = false), // writings ENDX register means sending ack command, and clear all bits.
            (  0x0, 0xD) => self.echo_feedback_volume = data,
            (  0x1, 0xD) => self.unused_1d = data,
            (  0x2, 0xD) => {
                let bools = u8_to_vec(data);
                self.blocks.iter_mut().zip(bools).for_each(|(blk, is_enable)| {
                    blk.reg.pmon_enable = is_enable;
                });
            }
            (  0x3, 0xD) => {
                let bools = u8_to_vec(data);
                self.blocks.iter_mut().zip(bools).for_each(|(blk, is_enable)| {
                    blk.reg.noise_enable = is_enable;
                });
            }
            (  0x4, 0xD) => {
                let bools = u8_to_vec(data);
                self.blocks.iter_mut().zip(bools).for_each(|(blk, is_enable)| {
                    blk.reg.echo_enable = is_enable;
                });
            }
            (  0x5, 0xD) => self.table_addr = data,
            (  0x6, 0xD) => self.echo_ring_buffer_addr = (data as u16) << 8,
            (  0x7, 0xD) => self.echo_buffer_size = data,      
            (upper, 0xE) => self.unused_e[upper] = data,
            (upper, 0xF) => {
                self.fir_left.filter[upper] = (data as i8) as i16;
                self.fir_right.filter[upper] = (data as i8) as i16; 
            }
            _ => panic!("{:#06x} is not expected address", addr),
        }
    }

    pub fn reset(&mut self) -> () {
        for mut blk in self.blocks.iter_mut() {    
            blk.reg.voice_end = true;
            blk.reg.env = 0;
            blk.reg.out = 0;            
        }

        self.echo_buffer_enable = false;
        self.is_mute = true;        
        self.soft_reset = true;
    }

    #[allow(non_snake_case)]
    fn read_FLG(&self) -> u8 {
        let noise_freq = self.noise_frequency;
        let echo_buffer_disable = (!self.echo_buffer_enable as u8) << 5;
        let is_mute = (self.is_mute as u8) << 6;
        let soft_reset = (self.soft_reset as u8) << 7;

        noise_freq | echo_buffer_disable | is_mute | soft_reset
    }

    pub fn sample_left_out(&self) -> i16 { self.sample_left_out }
    pub fn sample_right_out(&self) -> i16 { self.sample_right_out }    
}

fn generate_new_sample(brrs: &[u8], buffer: &mut [i16; SAMPLE_BUFFER_SIZE], brr_info: &BRRInfo, base_idx: usize) -> () {    
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

    let first_older_idx = (base_idx as i32 - 2).rem_euclid(SAMPLE_BUFFER_SIZE as i32) as usize;
    let mut older = buffer[first_older_idx % SAMPLE_BUFFER_SIZE] as i32;
    let mut old = buffer[(first_older_idx + 1) % SAMPLE_BUFFER_SIZE] as i32;

    nibbles.enumerate().for_each(|(idx, nibble)| {
        let shamt = brr_info.shift_amount as i32;
        let sample = shift(nibble, shamt);
            
        let sample = filter(sample, old, older);
        let sample = sample.min(0x7FFF).max(-0x8000); 
        let sample = ((sample as i16) << 1) >> 1;       
        
        buffer[(base_idx + idx) as usize] = sample;
        older = old;
        old = sample as i32;
    }); 
}

// TODO: need echo accumulate implementation
fn combine_all_sample(blocks: &[DSPBlock], dsp: &DSP) -> (i16, i16) {
    fn combine(samples: impl Iterator<Item = i32>, master_vol: i8) -> i16 {
        let acc: i32 = samples.fold(0, |acc, sample| {
            let sum = acc + sample;
            sum.min(0x7FFF).max(-0x8000)
        });

        let out = (acc * (master_vol as i32)) >> 7;
        let out = out.min(0x7FFF).max(-0x8000); 

        out as i16
    }

    if dsp.is_mute {
        (0, 0)
    } else {
        let lefts = blocks.iter().map(|blk| blk.sample_left as i32);
        let rights = blocks.iter().map(|blk| blk.sample_right as i32);
        let left = combine(lefts, dsp.master_vol_left as i8);
        let right = combine(rights, dsp.master_vol_right as i8);

        (left, right)
    } 
}

fn combine_echo(blocks: &[DSPBlock]) -> (i16, i16) {
    fn combine(samples: impl Iterator<Item = i32>) -> i32 {
        samples.fold(0, |acc, sample| {
            let sum = acc + sample;
            sum.min(0x7FFF).max(-0x8000)
        })
    }

    let lefts = blocks.iter().map(|blk| blk.echo_left as i32);
    let rights = blocks.iter().map(|blk| blk.echo_right as i32);
    let left = combine(lefts);
    let right = combine(rights);

    (left as i16, right as i16)
}

fn echo_process(left: i16, right: i16, dsp: &mut DSP) -> (i16, i16) {    
    let buffer_addr = ((dsp.echo_ring_buffer_addr + dsp.echo_pos) & 0xFFFF) as usize;

    let (left_out, left_new_echo) = echo_process_inner(left, buffer_addr, dsp.echo_feedback_volume as i8, dsp.echo_vol_left as i8, &mut dsp.fir_left);
    let (right_out, right_new_echo) = echo_process_inner(right, buffer_addr + 2, dsp.echo_feedback_volume as i8, dsp.echo_vol_right as i8, &mut dsp.fir_right);    

    if dsp.echo_buffer_enable {
        let left_lower  = left_new_echo as u8;
        let left_upper  = (left_new_echo >> 8) as u8;
        let right_lower = right_new_echo as u8;
        let right_upper = (right_new_echo >> 8) as u8;

        [left_lower, left_upper, right_lower, right_upper].iter().zip(0..).for_each (|(&sample, idx)| {
            Ram::global().ram[buffer_addr + idx] = sample;
        });
    }

    if dsp.echo_pos == 0 {
        dsp.echo_buf_length = (dsp.echo_buffer_size as u16 & 0x0F) * 0x800;
    }
    dsp.echo_pos += 4;
    if dsp.echo_pos >= dsp.echo_buf_length {
        dsp.echo_pos = 0;
    }

    (left_out as i16, right_out as i16)
}

fn echo_process_inner(echo_sample: i16, addr: usize, feedback_volume: i8, out_volume: i8, fir: &mut FIR) -> (i32, i16) {
    let sample0 = (Ram::global().ram[addr + 1] as u16) << 8;
    let sample1 = Ram::global().ram[addr] as u16;
    let buf_echo = sample0 | sample1; 
    let fir_out = fir.next(buf_echo as i16); 

    let out_echo = ((fir_out as i32) * (out_volume as i32)) >> 7;
    let new_echo = (echo_sample as i32) + (((fir_out as i32) * (feedback_volume as i32)) >> 7);
    let new_echo = 
        if new_echo > 0x7FFF { 0x7FFF }
        else if new_echo < -0x8000 { -0x8000 }
        else { new_echo };

    let new_echo = (new_echo as u16) & 0xFFFE;

    (out_echo, new_echo as i16)
}

fn u8_to_vec(v: u8) -> impl Iterator<Item = bool> {
    fn extract_bit(value: u8, shamt: u8) -> bool {
        ((value >> shamt) & 1) == 1
    }

    (0..8).map(move |shamt| extract_bit(v, shamt))
}

fn vec_to_u8(bools: impl Iterator<Item = bool>) -> u8 {
    bools.map(|b| b as u8)
        .zip(0..)
        .map(|(flag, idx)| flag << idx)
        .sum()
}
