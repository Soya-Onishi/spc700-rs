use crate::spc::register::Flags;

const MASK: u8 = 0b0000_0000;
const FLAG: u8 = 0b0000_0000;
const PSW_FLAG: super::Flag = (FLAG, MASK);

pub type RetType = (u8, super::Flag);

pub fn clr1(_dummy0: u8, _dummy1: u8) -> RetType {
    (0, PSW_FLAG)
}

pub fn set1(_dummy0: u8, _dummy1: u8) -> RetType {
    (1, PSW_FLAG)
}

pub fn not1(bit: u8, _dummy: u8) -> RetType {
    (!bit & 1, PSW_FLAG)
}

pub fn mov1(bit: u8, _dummy: u8) -> RetType {
    (bit, PSW_FLAG)
}

pub fn or1(c: u8, bit: u8) -> RetType {
    ((bit | c) & 1, PSW_FLAG)
}

pub fn and1(c: u8, bit: u8) -> RetType {
    ((bit & c) & 1, PSW_FLAG)
}

pub fn eor1(c: u8, bit: u8) -> RetType {
    ((bit ^ c) & 1, PSW_FLAG)
}

pub fn clrc(_dummy0: u8, _dummy1: u8) -> RetType {
    (0, PSW_FLAG)
}

pub fn setc(_dummy0: u8, _dummy1: u8) -> RetType {
    (1, PSW_FLAG)
}

pub fn notc(c: u8, _dummy: u8) -> RetType {
    (c ^ 1, PSW_FLAG)
}

pub fn clrv(psw: u8, _dummy1: u8) -> RetType {
    (psw & 0b1011_0111 , PSW_FLAG)
}