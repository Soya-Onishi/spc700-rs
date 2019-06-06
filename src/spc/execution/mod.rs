pub mod eight_alu;
pub mod eight_shift;
pub mod inclement;
pub mod sixteen_alu;
pub mod one_alu;
pub mod special;
pub mod condjump;
pub mod jump;

pub type Flag = (u8, u8); // (Flag from execution, mask of Flag)

fn is_carry(res: u16) -> u8 {
    (res > 0xff) as u8
}

fn is_zero(value: u8) -> u8 {
    let flag = (value == 0) as u8;
    flag << 1
}

fn is_half(op0: u8, op1: u8, res: u8) -> u8 {
    let flag = (((op0 ^ op1 ^ res) & 0x10) > 0) as u8;
    flag << 3
}

fn is_overflow(op0: u8, op1: u8, res: u8) -> u8 {
    let flag = ((!(op0 ^ op1) & (op0 ^ res) & 0x80) > 0) as u8;
    flag << 6
}

fn is_sign(value: u8) -> u8 {
    value & 0x80
}