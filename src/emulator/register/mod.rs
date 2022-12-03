mod flags;

extern crate spc;

use std::fmt;
use std::fmt::Display;

pub use self::flags::Flags;
use spc::spc::Spc;

#[derive(Debug)]
pub struct Register {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub sp: u8,
    pub psw: Flags,
    pub pc: u16,
}

impl Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f, 
            "a: {:04x}, x: {:04x}, y: {:04x}, sp: {:04x}, psw: {:010b}, pc: {:06x}",
            self.a, self.x, self.y, self.sp, self.psw.get(), self.pc
        )
    }
}

impl Register {
    pub fn new(init_pc: u16) -> Register {
        Register {
            a: 0,
            x: 0,
            y: 0,
            sp: 0xef,
            psw: Flags::new(),
            pc: init_pc,
        }
    }

    pub fn new_with_init(spc: &Spc) -> Register {
        Register {
            a: spc.a,
            x: spc.x,
            y: spc.y,
            sp: spc.sp,
            psw: Flags::new_with_init(spc.psw),
            pc: spc.pc,
        }
    }

    pub fn inc_pc(&mut self, count: u16) -> u16 {
        let pc = self.pc;
        self.pc = self.pc.wrapping_add(count);

        pc
    }

    pub fn ya(&self) -> u16 {
        let y = self.y as u16;
        let a = self.a as u16;

        (y << 8) | a
    }

    pub fn set_ya(&mut self, ya: u16) -> () {
        let y = (ya >> 8) as u8;
        let a = (ya & 0xFF) as u8;

        self.y = y;
        self.a = a;
    }
}