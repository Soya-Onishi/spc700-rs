use super::*;

pub type RetType = (u8, super::Flag);

pub fn mov(_dummy: u8, src: u8) -> RetType {
    bitwise(_dummy, src, |_x, y| -> u8 { y } )
}

pub fn or(op0: u8, op1: u8) -> RetType {
    bitwise(op0, op1, |x, y| -> u8 { x | y })
}

pub fn and(op0: u8, op1: u8) -> RetType {
    bitwise(op0, op1, |x, y| -> u8 { x & y })
}

pub fn eor(op0: u8, op1: u8) -> RetType {
    bitwise(op0, op1, |x, y| -> u8 { x ^ y })
}

fn bitwise(op0: u8, op1: u8, f: impl Fn(u8, u8) -> u8) -> RetType {
    let res = f(op0, op1);

    let mask: u8 = 0b1000_0010;
    let sign = is_sign(res);
    let zero = is_zero(res);
    let flag = sign | zero;

    (res, (flag, mask))
}

pub fn cmp(op0: u8, op1: u8) -> RetType {
    let op0 = op0 as u16;
    let op1 = !op1 as u16;
    let res = op0.wrapping_add(op1).wrapping_add(1);

    let mask = 0b1000_0011;
    let sign = is_sign(res as u8);
    let zero = is_zero(res as u8);
    let carry = is_carry(res);
    let flag = (sign | zero | carry) & mask;

    (res as u8, (flag, mask))
}

pub fn adc(op0: u8, op1: u8, carry_flag: bool) -> RetType {
    let op0 = op0 as u16;
    let op1 = op1 as u16;
    let c: u16 = if carry_flag { 1 } else { 0 };

    let res = op0.wrapping_add(op1).wrapping_add(c);

    let mask = 0b1100_1011;
    let zero = is_zero(res as u8);
    let sign = is_sign(res as u8);
    let half = is_half(op0 as u8, op1 as u8, res as u8);
    let carry = is_carry(res);
    let overflow = is_overflow(op0 as u8, op1 as u8, res as u8);
    let flag = (zero | sign | half | carry | overflow) & mask;

    (res as u8, (flag, mask))
}

pub fn sbc(op0: u8, op1: u8, carry_flag: bool) -> RetType {
    adc(op0, !op1, carry_flag)
}