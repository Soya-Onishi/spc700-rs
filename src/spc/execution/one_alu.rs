const MASK: u8 = 0b0000_0000;
const FLAG: u8 = 0b0000_0000;

pub fn clr1(pwd: u8) -> (u8, u8, u8) {
    (0, FLAG, MASK)
}

pub fn set1() -> (u8, u8, u8) {
    (1, FLAG, MASK)
}

pub fn not1(bit: u8) -> (u8, u8, u8) {
    (!bit & 1, MASK, FLAG)
}

pub fn mov1(bit: u8) -> (u8, u8, u8) {
    (bit, MASK, FLAG)
}

pub fn or1(bit: u8, c: u8) -> (u8, u8, u8) {
    (bit | c, MASK, FLAG)
}

pub fn and1(bit: u8, c: u8) -> (u8, u8, u8) {
    (bit & c, MASK, FLAG)
}

pub fn eor1(bit: u8, c: u8) -> (u8, u8, u8) {
    (bit ^ c, MASK, FLAG)
}

pub fn clrc(pwd: u8) -> (u8, u8, u8) {
    (pwd & 0b1111_1110, MASK, FLAG)
}

pub fn setc(pwd: u8) -> (u8, u8, u8) {
    (pwd | 0b0000_0001, MASK, FLAG)
}

pub fn notc(pwd: u8) -> (u8, u8, u8) {
    (pwd ^ 0b0000_0001, MASK, FLAG)
}

pub fn clrv(pwd: u8) -> (u8, u8, u8) {
    (pwd & 0b0100_1000, MASK, FLAG)
}