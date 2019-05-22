mod flags;

pub use self::flags::*;

pub struct Register {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub sp: u8,
    pub psw: Flags,
    pub pc: u16,
}

impl Register {
    pub fn new() -> Register {
        Register {
            a: 0,
            x: 0,
            y: 0,
            sp: 0xef,
            psw: Flags::new(),
            pc: 0,
        }
    }
}