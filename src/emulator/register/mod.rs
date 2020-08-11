mod flags;

extern crate spc;

pub use self::flags::Flags;
use spc::spc::Spc;

pub struct Register {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub sp: u8,
    pub psw: Flags,
    pub pc: u16,
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
            sp: spc.y,
            psw: Flags::new_with_init(spc.psw),
            pc: spc.pc,
        }
    }

    pub fn inc_pc(&mut self, count: u16) -> u16 {
        let pc = self.pc;
        self.pc = self.pc.wrapping_add(count);

        pc
    }
}