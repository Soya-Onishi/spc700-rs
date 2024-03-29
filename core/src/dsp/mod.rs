mod gaussian_table;
mod envelope;
mod block;
mod brr;

use std::u8;
use std::i16;
use std::u16;

use array_macro::array;

use crate::processor::ram::Ram;
use block::DSPBlock;
use brr::FilterType;

const SAMPLE_BUFFER_SIZE: usize = 16 + 3;
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
            
        let dsp = DSP {
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
            blk.init(idx, regs);
        }
        
        // 初期化時にkonフラグが立っている場合、keyon処理を行う
        for (kon, blk) in u8_to_vec(regs[0x4C]).zip(blocks.iter_mut()) {
            if kon {
                blk.keyon(regs[0x5D] as u16);
            }
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
        dsp.echo_buf_length = calc_echo_buffer_size(regs[0x7D]);
        
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
        let soft_reset = self.soft_reset && self.flag_is_modified;
        let cycle_counter = self.counter;            

        self.blocks.iter_mut().fold(Option::<i16>::None, |before_out, blk| {                                    
            blk.flush(before_out, soft_reset, cycle_counter);
            Some(blk.sample_out)
        });

        let (left, right) = combine_all_sample(&self.blocks);         
        let (echo_left, echo_right) = combine_echo(&self.blocks);        
        let (left_echo, right_echo) = echo_process(echo_left, echo_right, self);

        let left_out = (left as i32) + (left_echo as i32);
        let right_out = (right as i32) + (right_echo as i32);
        
        self.flag_is_modified = false;
        self.counter = (self.counter + 1) % CYCLE_RANGE;
        self.sample_left_out = left_out as i16;
        self.sample_right_out = right_out as i16; 
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
                    .for_each(|(blk, _)| { 
                       blk.keyon(self.table_addr as u16);
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
            (  0x7, 0xD) => {
                self.echo_buffer_size = data;
                self.echo_buf_length = calc_echo_buffer_size(data);
            },
            (upper, 0xE) => self.unused_e[upper] = data,
            (upper, 0xF) => {
                self.fir_left.filter[upper] = (data as i8) as i16;
                self.fir_right.filter[upper] = (data as i8) as i16; 
            }
            _ => panic!("{:#06x} is not expected address", addr),
        }
    }

    #[allow(dead_code)]
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

// TODO: need echo accumulate implementation
fn combine_all_sample(blocks: &[DSPBlock]) -> (i16, i16) {
    let dsp = DSP::global();

    if dsp.is_mute {
        (0, 0)
    } else {
        let left = blocks.iter().map(|blk| blk.sample_left as i32).sum::<i32>() * dsp.master_vol_left as i32 >> 7;
        let right = blocks.iter().map(|blk| blk.sample_right as i32).sum::<i32>() * dsp.master_vol_right as i32 >> 7;

        let left = left.min(0x7FFF).max(-0x8000) as i16;
        let right = right.min(0x7FFF).max(-0x8000) as i16;

        (left, right)
    } 
}

fn combine_echo(blocks: &[DSPBlock]) -> (i16, i16) {
    let left = blocks.iter().map(|blk| blk.echo_left as i32).sum::<i32>();
    let right = blocks.iter().map(|blk| blk.echo_right as i32).sum::<i32>();

    let left = left.min(0x7FFF).max(-0x8000) as i16;
    let right = right.min(0x7FFF).max(-0x8000) as i16;

    (left, right)
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

        let ram = &mut Ram::global().ram[buffer_addr..];
        ram[0] = left_lower;
        ram[1] = left_upper;
        ram[2] = right_lower;
        ram[3] = right_upper; 
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

fn calc_echo_buffer_size(data: u8) -> u16 {
    (data as u16 & 0x0F) * 0x800
}