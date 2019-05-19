mod eight_alu;
mod eight_shift;
mod inclement;
mod sixteen_alu;
mod one_alu;
mod special;
mod condjump;
mod jump;

pub use self::eight_alu::*;
pub use self::eight_shift::*;
pub use self::inclement::*;
pub use self::sixteen_alu::*;
pub use self::one_alu::*;
pub use self::special::*;
pub use self::condjump::*;
pub use self::jump::*;

type Flag = (u8, u8); // (Flag from execution, mask of Flag)

fn is_carry(op0: u8, res: u8) -> u8 {
    (op0 > res) as u8
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