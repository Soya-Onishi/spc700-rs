use super::core::Spc700;
use std::fs;

pub struct Ram {
    ram: [u8; 0x10000],
}

impl Ram {
    pub fn new() -> Ram {
        let r = Ram {
            ram: [0; 0x10000],
        };

        r
    }

    pub fn load(&mut self, filename: String, start_pos: u16, set_pos: u16) {
        let binaries = fs::read(filename).expect("not found");
        let start_pos = start_pos as usize;
        let set_pos = set_pos as usize;

        for (offset, bin) in binaries[start_pos..].iter().enumerate() {
            if bin.clone() != 0 {
                println!("Loading...[{:#06x}] <= {:#04x}", set_pos + offset, bin);
            }

            self.ram[set_pos + offset] = bin.clone();
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        self.ram[addr as usize]
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        self.ram[addr as usize] = data;
        if addr == 0x8000 {
            println!("0x8000: {:#06x}", self.ram[0x8000]);
        }
    }
}

/*
impl<'a> Ram<'a> {
    pub fn new() -> Ram<'a> {
        Ram {
            core: None,
            ram: 0,
        }
    }

    pub fn reg_core(&mut self, spc: &'a Spc700) {
        self.core = Some(spc)
    }
}
*/