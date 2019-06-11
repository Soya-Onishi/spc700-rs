use super::core::Spc700;
use std::fs;
use std::collections::HashMap;
use std::cmp::Ordering;

pub struct Ram {
    pub ram: [u8; 0x10000],
    pub read_log: Vec<(u16, u8)>,
    pub write_log: Vec<(u16, u8)>,
}

impl Ram {
    pub fn new() -> Ram {
        let r = Ram {
            ram: [0; 0x10000],
            read_log: Vec::new(),
            write_log: Vec::new(),
        };

        r
    }

    pub fn load(&mut self, filename: String, start_pos: u16, set_pos: u16) {
        let binaries = fs::read(filename).expect("not found");
        let start_pos = start_pos as usize;
        let set_pos = set_pos as usize;

        for (offset, bin) in binaries[start_pos..].iter().enumerate() {
            if bin.clone() != 0 {
                // println!("Loading...[{:#06x}] <= {:#04x}", set_pos + offset, bin);
            }

            self.ram[set_pos + offset] = bin.clone();
        }
    }

    pub fn read(&mut self, addr: u16) -> u8 {
        let data = self.ram[addr as usize];
        self.read_log.push((addr, data));
        data
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        if addr == 0x0813 {
            let mut sum: u32 = 0;
            for i in 0..self.ram.len() {
                sum = sum.wrapping_add(self.ram[i] as u32);
            }
            // println!("sum({:#04x}): 0x{:#010x}", self.ram[0x813], sum);
        }

        self.ram[addr as usize] = data;

        self.write_log.push((addr, data));
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
