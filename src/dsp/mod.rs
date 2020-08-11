mod gaussian_table;
mod envelope;

use std::u16;
use std::u32;

use envelope::*;
use crate::emulator::ram::Ram;

const NUMBER_OF_DSP: usize = 8;
const SAMPLE_BUFFER_SIZE: usize = 3;
pub const CYCLE_RANGE: u16 = 30720;

pub struct DSP {
    blocks: Vec<DSPBlock>,
    master_vol_left: u8,
    master_vol_right: u8,
    echo_vol_left: u8,
    echo_vol_right: u8,
    table_addr: u8, // DIR register

    // modification flag
    flag_is_modified: bool,
    key_on_is_modified: bool,

    // FLG register    
    noise_frequency: u8,
    echo_buffer_disable: bool,
    is_mute: bool,
    soft_reset: bool,

    echo_feedback_volume: u8,
    echo_ring_buffer_addr: u8,
    echo_buffer_size: u8,

    sample_left_out: u16,
    sample_right_out: u16,

    // global dsp counter
    counter: u16,    
    
    // These registers are unused in DSP.    
    unused_a: [u8; 8], 
    unused_b: [u8; 8],
    unused_1d: u8,
    unused_e: [u8; 8],    
}

pub struct DSPBlock {
    pub idx: usize, // block id [0 - 7]
    pub reg: DSPRegister,
        
    buffer: [u16; SAMPLE_BUFFER_SIZE],

    start_addr: u16,
    loop_addr: u16,
    src_addr: u16,
    brr_info: BRRInfo,
    brr_nibbles: Vec<u8>,
    envelope: Envelope,    

    pitch_counter: u16,    
    require_next: bool,
    is_loop: bool,

    sample_out: u16,
    sample_left: u16,
    sample_right: u16,

    key_on_cooling: u8,    
}

pub struct DSPRegister {
    pub vol_left: u8,
    pub vol_right: u8,
    pub pitch: u16,
    pub srcn: u8,
    pub adsr: u16,
    pub gain: u8,
    pub env: u8,
    pub out: u8,
    pub echo_filter: u8,    
    
    pub key_on: bool,    
    pub key_off: bool,    

    pub voice_end: bool,
    pub noise_enable: bool,
    pub echo_enable: bool,
    pub pmon_enable: bool,        
}

impl DSPRegister {
    pub fn new() -> DSPRegister {
        DSPRegister {
            vol_left: 0,
            vol_right: 0,
            pitch: 0,
            srcn: 0,
            adsr: 0,
            gain: 0,
            env: 0,
            out: 0,
            echo_filter: 0,

            key_on: false,            
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
        let bit  = |idx: u8, data: u8| -> bool { (data & (1 << idx)) > 0 };

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
            echo_filter: regs[addr(0xF)],
            
            key_on: bit(idx as u8, regs[0x4C]),
            key_off: bit(idx as u8, regs[0x5C]),

            voice_end: bit(idx as u8, regs[0x7C]),
            noise_enable: bit(idx as u8, regs[0x3D]),
            echo_enable: bit(idx as u8, regs[0x4D]),
            pmon_enable: bit(idx as u8, regs[0x2D]),
        }
    }
}

struct BRRInfo {
    shift_amount: u8,
    filter: FilterType,
    end: BRREnd,
}

#[derive(Copy, Clone)]
enum FilterType {
    NoFilter,
    UseOld,
    UseAll0,
    UseAll1,
}

#[derive(Copy, Clone, PartialEq)]
enum BRREnd {
    Normal,
    Mute,
    Loop,
}

impl DSP {
    pub fn new() -> DSP {
        let blocks = (0..NUMBER_OF_DSP).map(|idx| DSPBlock::new(idx)).collect::<Vec<DSPBlock>>();

        let mut dsp = DSP {
            blocks: blocks,
            master_vol_left: 0,
            master_vol_right: 0,
            echo_vol_left: 0,
            echo_vol_right: 0,
            table_addr: 0,

            flag_is_modified: false,
            key_on_is_modified: false,

            noise_frequency: 0,
            echo_buffer_disable: true,
            is_mute: true,
            soft_reset: true,

            echo_feedback_volume: 0,
            echo_ring_buffer_addr: 0,
            echo_buffer_size: 0,

            sample_left_out: 0,
            sample_right_out: 0,

            counter: 0,

            unused_a: [0; 8],
            unused_b: [0; 8],
            unused_1d: 0,
            unused_e: [0; 8],
        };

        dsp.reset();
        dsp
    }

    pub fn new_with_init(regs: &[u8; 128]) -> DSP {
        let blocks = (0..NUMBER_OF_DSP).map(|idx| DSPBlock::new_with_init(idx, regs)).collect::<Vec<DSPBlock>>();        
        let mut dsp = DSP::new();

        // initialized by regs
        dsp.blocks = blocks;
        dsp.master_vol_left = regs[0x0C];
        dsp.master_vol_right = regs[0x1C];
        dsp.echo_vol_left = regs[0x2C];
        dsp.echo_vol_right = regs[0x3C];
        dsp.table_addr = regs[0x5D];

        dsp.flag_is_modified = true;
        dsp.key_on_is_modified = true;

        let flag = regs[0x6C];
        dsp.noise_frequency = flag & 0x1F;
        dsp.echo_buffer_disable = (flag & 0x20) > 0;
        dsp.is_mute = (flag & 0x40) > 0;
        dsp.soft_reset = (flag & 0x80) > 0;

        dsp.echo_feedback_volume = regs[0x0D];
        dsp.echo_ring_buffer_addr = regs[0x6D];
        dsp.echo_buffer_size = regs[0x7D];
        
        dsp
    }

    pub fn flush(&mut self, ram: &mut Ram) -> () {
        let table_addr = self.table_addr as u16;
        let soft_reset = self.soft_reset && self.flag_is_modified;
        let cycle_counter = self.counter;
        
        let key_on_is_modified = self.key_on_is_modified;

        self.blocks.iter_mut().fold(Option::<u16>::None, |before_out, blk| {
            // ready for next brr block by key on            
            if blk.reg.key_on {
                let tab_addr = (table_addr * 256 + (blk.reg.srcn as u16 * 4)) as usize;
                let start0 = ram.ram[tab_addr] as u16;
                let start1 = ram.ram[tab_addr + 1] as u16;
                let loop0 = ram.ram[tab_addr + 2] as u16;
                let loop1 = ram.ram[tab_addr + 3] as u16;

                blk.start_addr = start0 + (start1 << 8);                
                blk.loop_addr = loop0 + (loop1 << 8);
                blk.src_addr = blk.start_addr;
                blk.key_on_cooling = 5;
            }

            // ready for next brr block by normal or loop
            if blk.require_next && !blk.reg.key_on {
                if blk.is_loop {
                    blk.src_addr = blk.loop_addr;
                } else {
                    blk.src_addr += 9;
                }                
            }

            // fetch brr block
            if blk.reg.key_on || blk.require_next {
                let addr = blk.src_addr as usize;
                let brr_block = &ram.ram[addr..addr + 9];

                blk.brr_info = BRRInfo::new(brr_block[0]);
                blk.brr_nibbles = Vec::from(&brr_block[1..]);              
            }
                                    
            if blk.key_on_cooling == 0 {
                blk.flush(before_out, soft_reset, key_on_is_modified, cycle_counter);
            } else {
                blk.key_on_cooling -= 1;

                blk.sample_out = 0;
                blk.sample_left = 0;
                blk.sample_right = 0;
            }
            
            Some(blk.sample_out)
        });

        let (left, right) = combine_all_sample(&self.blocks, self);
        self.flag_is_modified = false;
        self.key_on_is_modified = false;
        self.counter = (self.counter + 1) % CYCLE_RANGE;
        self.sample_left_out = left;
        self.sample_right_out = right;
    }

    pub fn read_from_register(&self, addr: usize) -> u8 {
        let upper_base = (addr >> 4) & 0xF;
        let upper = if upper_base >= 0x8 { upper_base - 0x8 } else { upper_base}; // to address mirror
        let lower = addr & 0xF;

        match (upper as usize, lower as usize) {
            (upper, 0x1) => self.blocks[upper].reg.vol_left,
            (upper, 0x2) => self.blocks[upper].reg.vol_right,
            (upper, 0x3) => (self.blocks[upper].reg.pitch & 0xFF) as u8,
            (upper, 0x4) => ((self.blocks[upper].reg.pitch >> 8) & 0xFF) as u8,
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
            (  0x5, 0xC) => vec_to_u8(self.blocks.iter().map(|blk| blk.reg.key_off).collect()),
            (  0x6, 0xC) => self.read_FLG(),
            (  0x7, 0xC) => vec_to_u8(self.blocks.iter().map(|blk| blk.reg.voice_end).collect()),
            (  0x0, 0xD) => self.echo_feedback_volume,
            (  0x1, 0xD) => self.unused_1d,
            (  0x2, 0xD) => vec_to_u8(self.blocks.iter().map(|blk| blk.reg.pmon_enable).collect()),
            (  0x3, 0xD) => vec_to_u8(self.blocks.iter().map(|blk| blk.reg.noise_enable).collect()),
            (  0x4, 0xD) => vec_to_u8(self.blocks.iter().map(|blk| blk.reg.echo_enable).collect()),
            (  0x5, 0xD) => self.table_addr,
            (  0x6, 0xD) => self.echo_ring_buffer_addr,
            (  0x7, 0xD) => self.echo_buffer_size,
            (upper, 0xE) => self.unused_e[upper],
            (upper, 0xF) => self.blocks[upper].reg.echo_filter,
            _ => panic!(format!("{:#06x} is not unexpected address", addr)),
        }
    }

    pub fn write_to_register(&mut self, addr: usize, data: u8) -> () {
        let upper = (addr >> 4) & 0xF;
        let lower = addr & 0xF;
        match (upper, lower) {
            (0x8..=0xF, _) => (), // 0x80..0xFF are read only mirrors of 0x00..0x7F
            (upper, 0x0) => self.blocks[upper].reg.vol_left = data,
            (upper, 0x1) => self.blocks[upper].reg.vol_right = data,
            (upper, 0x2) => {
                let old_pitch = self.blocks[upper].reg.pitch;
                let assigned = data as u16;
                let new_pitch = (old_pitch & !0xFF) | assigned;

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
                self.key_on_is_modified = true;
                self.blocks.iter_mut().zip(bools.iter()).for_each(|(blk, &is_on)| {                    
                    blk.reg.key_on = is_on;
                });
            }
            (  0x5, 0xC) => {
                let bools = u8_to_vec(data);
                self.blocks.iter_mut().zip(bools.iter()).for_each(|(blk, &is_off)| {
                    blk.reg.key_off = is_off;
                });
            }
            (  0x6, 0xC) => {
                let noise_frequency = data & 0x1F;
                let echo_buffer_disable = (data & 0x20) > 0;
                let is_mute = (data & 0x40) > 0;
                let soft_reset = (data & 0x80) > 0;

                self.flag_is_modified = true;
                self.noise_frequency = noise_frequency;
                self.echo_buffer_disable = echo_buffer_disable;
                self.is_mute = is_mute;
                self.soft_reset = soft_reset;
            }
            (  0x7, 0xC) => self.blocks.iter_mut().for_each(|blk| blk.reg.voice_end = false), // writings ENDX register means sending ack command, and clear all bits.
            (  0x0, 0xD) => self.echo_feedback_volume = data,
            (  0x1, 0xD) => self.unused_1d = data,
            (  0x2, 0xD) => {
                let bools = u8_to_vec(data);
                self.blocks.iter_mut().zip(bools.iter()).for_each(|(blk, &is_enable)| {
                    blk.reg.pmon_enable = is_enable;
                });
            }
            (  0x3, 0xD) => {
                let bools = u8_to_vec(data);
                self.blocks.iter_mut().zip(bools.iter()).for_each(|(blk, &is_enable)| {
                    blk.reg.noise_enable = is_enable;
                });
            }
            (  0x4, 0xD) => {
                let bools = u8_to_vec(data);
                self.blocks.iter_mut().zip(bools.iter()).for_each(|(blk, &is_enable)| {
                    blk.reg.echo_enable = is_enable;
                });
            }
            (  0x5, 0xD) => self.table_addr = data,
            (  0x6, 0xD) => self.echo_ring_buffer_addr = data,
            (  0x7, 0xD) => self.echo_buffer_size = data,      
            (upper, 0xE) => self.unused_e[upper] = data,
            (upper, 0xF) => self.blocks[upper].reg.echo_filter = data,
            _ => panic!(format!("{:#06x} is not expected address", addr)),
        }
    }

    pub fn reset(&mut self) -> () {
        self.blocks.iter_mut().for_each(|blk| {            
            blk.reg.voice_end = true;
            blk.reg.env = 0;
            blk.reg.out = 0;            
        });

        self.echo_buffer_disable = true;
        self.is_mute = true;        
        self.soft_reset = true;
    }

    #[allow(non_snake_case)]
    fn read_FLG(&self) -> u8 {
        let noise_freq = self.noise_frequency;
        let echo_buffer_disable = (self.echo_buffer_disable as u8) << 5;
        let is_mute = (self.is_mute as u8) << 6;
        let soft_reset = (self.soft_reset as u8) << 7;

        noise_freq | echo_buffer_disable | is_mute | soft_reset
    }
}

impl DSPBlock {
    pub fn new(idx: usize) -> DSPBlock {
        DSPBlock {
            idx: idx,
            reg: DSPRegister::new(),
            buffer: [0; SAMPLE_BUFFER_SIZE],

            start_addr: 0,
            loop_addr: 0,
            src_addr: 0,
            brr_info: BRRInfo::empty(),
            brr_nibbles: Vec::<u8>::new(),
            envelope: Envelope::empty(),

            pitch_counter: 0,            
            require_next: false,
            is_loop: false,

            sample_out: 0,
            sample_left: 0,
            sample_right: 0,

            key_on_cooling: 0,
        }
    }
    
    pub fn new_with_init(idx: usize, regs: &[u8; 128]) -> DSPBlock {
        let mut init_block = DSPBlock::new(idx);
        init_block.reg = DSPRegister::new_with_init(idx, regs);
        
        init_block
    }

    pub fn flush(&mut self, before_out: Option<u16>, soft_reset: bool, key_on_is_modified: bool, cycle_counter: u16) -> () {        
        let key_on_kicked = self.reg.key_on && key_on_is_modified;        

        // fetch brr nibbles 
        let (brr_info, nibbles) = (&self.brr_info, &self.brr_nibbles);

        // generate sample
        let nibble_idx = (self.pitch_counter >> 12) as usize;
        let nibble = fetch_brr_nibble(&nibbles, nibble_idx);
        let sample = generate_new_sample(nibble, &self.buffer, brr_info.shift_amount, brr_info.filter);        

        // calculate related pitch
        let step = generate_additional_pitch(&self.reg, before_out);
        let (next_pitch, require_next_block) = self.reg.pitch.overflowing_add(step);
        
        // filter sample
        let gaussian_idx = (next_pitch >> 3) & ((2 << 9) - 1);
        let filtered_sample = gaussian_interpolation(sample, gaussian_idx as usize, &self.buffer);        

        // envelope        
        let is_mute = self.require_next && brr_info.end == BRREnd::Mute;        
        let envelope_level = 
            if is_mute || key_on_kicked || soft_reset {
                0
            } else {
                self.envelope.level
            };
        let envelope_mode =
            if is_mute || self.reg.key_off || soft_reset {
                ADSRMode::Release
            } else if key_on_kicked {
                ADSRMode::Attack
            } else {
                self.envelope.adsr_mode
            };
        let envelope = Envelope::new(envelope_level, envelope_mode);
        let env = envelope.envelope(self, cycle_counter);
        let out = ((filtered_sample as i32) * (env.level as i32)) >> 11; // envelope bit width is 11, so dividing 2^11.

        //
        // POST PROCESS
        //    
        
        // renew buffer
        let [old, older, _] = self.buffer;
        self.buffer = [sample, old, older];

        // renew dsp registers
        let is_voice_end = brr_info.end == BRREnd::Loop || brr_info.end == BRREnd::Mute;
        let envx = (env.level >> 4) as u8;
        let outx = (out >> 7) as u8;        
        self.reg.env = envx;
        self.reg.out = outx;        
        self.reg.voice_end = is_voice_end;        
        self.require_next = require_next_block;
        self.is_loop = self.brr_info.end == BRREnd::Loop;                
        if soft_reset {
            self.reg.key_off = soft_reset;            
        }

        // output sample of left and right
        self.sample_out = out as u16;
        self.sample_left = (out * (self.reg.vol_left as i32) >> 6) as u16;
        self.sample_right = (out * (self.reg.vol_right as i32) >> 6) as u16;
    }
}

impl BRRInfo {
    pub fn new(format: u8) -> BRRInfo {
        let shift_amount = format >> 4;
        let filter = match (format >> 2) & 3 {
            0 => FilterType::NoFilter,
            1 => FilterType::UseOld,
            2 => FilterType::UseAll0,
            3 => FilterType::UseAll1,
            _ => panic!("filter value should be between 0 to 3"),
        };

        let end = match format & 3 {
            0 | 2 => BRREnd::Normal,
            1 => BRREnd::Mute,
            3 => BRREnd::Loop,
            _ => panic!("end range should be between 0 to 3"),
        };

        BRRInfo {
            shift_amount,
            filter,
            end,
        }
    }

    pub fn empty() -> BRRInfo {
        BRRInfo::new(0)
    }
}

fn fetch_brr_nibble(nibbles: &[u8], idx: usize) -> u8 {
    let two_nibbles = nibbles[idx / 2];
    let nibble_idx = two_nibbles & 1;

    if nibble_idx == 0 {
        (two_nibbles & 0xF0) >> 4        
    } else {
        two_nibbles & 0x0F
    }
}

fn generate_new_sample(nibble: u8, buffer: &[u16; SAMPLE_BUFFER_SIZE], shamt: u8, filter: FilterType) -> u16 {
    let &[old, older, _] = buffer;
    let signed_old = old as i16;
    let signed_older = older as i16;
    let sample = if shamt > 12 {
        (((nibble as i8) >> 3) as u16) << 12
    } else {
        ((((nibble as u16) << shamt) as i16) >> 1) as u16
    };

    match filter {
        FilterType::NoFilter => sample,
        FilterType::UseOld => {
            let old_filter = signed_old + (-signed_old >> 4);
            sample + (old_filter as u16)
        }
        FilterType::UseAll0 => {
            let old_filter = (signed_old * 2) + ((-signed_old * 3) >> 5);
            let older_filter = signed_older + (signed_older >> 4);

            sample + ((old_filter - older_filter) as u16)
        }
        FilterType::UseAll1 => {
            let old_filter = (signed_old * 2) + ((-signed_old * 13) >> 6);
            let older_filter = signed_older + ((signed_older * 3) >> 4);

            sample + ((old_filter - older_filter) as u16)
        }
    }
}

fn generate_additional_pitch(reg: &DSPRegister, before_out: Option<u16>) -> u16 {
    let base_step = reg.pitch & ((2 << 14) - 1);

    if !reg.pmon_enable || before_out.is_none() {
        base_step
    } else {
        let factor = before_out.unwrap() as i16;
        let factor = (factor >> 4).wrapping_add(0x400);
        let ret = (base_step as u32) * (factor as u32) >> 10;

        if ret > 0x7FEE {
            0x7FEE
        } else {
            ret as u16
        }
    }
}

fn gaussian_interpolation(sample: u16, base_idx: usize, buffer: &[u16; SAMPLE_BUFFER_SIZE]) -> i16 {    
    let factor0 = (gaussian_table::GAUSSIAN_TABLE[0x0FF - base_idx] as i32 * buffer[SAMPLE_BUFFER_SIZE - 1] as i32) >> 10;
    let factor1 = (gaussian_table::GAUSSIAN_TABLE[0x1FF - base_idx] as i32 * buffer[SAMPLE_BUFFER_SIZE - 2] as i32) >> 10;
    let factor2 = (gaussian_table::GAUSSIAN_TABLE[0x100 + base_idx] as i32 * buffer[SAMPLE_BUFFER_SIZE - 3] as i32) >> 10;
    let factor3 = (gaussian_table::GAUSSIAN_TABLE[0x000 + base_idx] as i32 * sample as i32) >> 10;

    let out = factor0;
    let out = out + factor1;
    let out = out + factor2;
    let out = out + factor3;
    let out = 
        if out > 0x7FFF { 0x7FFF }
        else if out < -0x8000 { -0x8000 }
        else { out };
    
    (out as i16) >> 1
}

// TODO: need echo accumulate implementation
fn combine_all_sample(blocks: &Vec<DSPBlock>, dsp: &DSP) -> (u16, u16) {
    let f = |samples: Vec<u16>, master_vol: u8| -> u16 {
        let acc = samples.iter().fold(0_u16, |acc, &sample| {
            acc.saturating_add(sample)
        });

        let acc = (acc as u32) * (master_vol as u32) >> 7;
        if dsp.is_mute { 0 } else { acc as u16 }
    };

    let lefts = blocks.iter().map(|blk| blk.sample_left).collect::<Vec<u16>>();
    let rights = blocks.iter().map(|blk| blk.sample_right).collect::<Vec<u16>>();
    let left = f(lefts, dsp.master_vol_left);
    let right = f(rights, dsp.master_vol_right);

    (left, right)
}

fn u8_to_vec(v: u8) -> Vec<bool> {
    let f = |value: u8, shamt: u8| -> bool {
        ((value >> shamt) & 1) == 1
    };

    (0..8).map(|shamt| f(v, shamt)).collect::<Vec<bool>>()
}

fn vec_to_u8(bools: Vec<bool>) -> u8 {
    (0..8).fold(0, |acc, idx| {
        let bit = (bools[idx] as u8) << (idx as u8);
        acc | bit
    })    
}
