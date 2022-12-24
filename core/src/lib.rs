mod processor;
mod dsp;

pub type SPC700 = processor::Spc700;

pub const BOOT_ROM_DATA: [u8; 64] = processor::ram::BOOT_ROM_DATA;