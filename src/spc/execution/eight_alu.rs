pub fn or(op0: u8, op1: u8) -> (u8, u8) {
    bitwise(op0, op1, |x, y| -> u8 { x | y })
}

pub fn and(op0: u8, op1: u8) -> (u8, u8) {
    bitwise(op0, op1, |x, y| -> u8 { x & y })
}

pub fn eor(op0: u8, op1: u8) -> (u8, u8) {
    bitwise(op0, op1, |x, y| -> u8 { x ^ y })
}

fn bitwise(op0: u8, op1: u8, f: impl Fn(u8, u8) -> u8) -> (u8, u8) {
    let res = f(op0, op1);

    let mask: u8 = 0b1000_0010;
    let sign = is_zero(res);
    let zero = is_zero(res);
    let flag = sign | zero;

    (res, flag)
}

pub fn adc(op0: u8, op1: u8, carry_flag: bool) -> (u8, u8) {
    let c: u16 = if carry_flag { 1 } else { 0 };

    let res = op0.wrapping_add(op1).wrapping_add(c);

    let zero = is_zero(res);
    let sign = is_sign(res);
    let half = is_half(op0, op1, res);
    let carry = is_carry(op0, res);
    let overflow = is_overflow(op0, op1, res);
    let flag = zero | sign | half | carry | overflow;

    (res, flag)
}

pub fn sbc(op0: u8, op1: u8, carry_flag: bool) -> (u8, u8) {
    adc(op0, !op1, carry_flag)
}

fn is_carry(op0: u8, res: u8) -> u8 {
    (op0 > res) as u8
}

fn is_zero(value: u8) -> u8 {
    let flag = (value == 0) as u8;
    flag << 1
}

fn is_half(op0: u8, op1: u8, res: u8) -> u8 {
    let flag = (op0 ^ op1 ^ res) & 0x10 == 1 as u8;
    flag << 3
}

fn is_overflow(op0: u8, op1: u8, res: u8) -> u8 {
    let flag = !(op0 ^ op1) & (op0 ^ res) & 0x80 == 1 as u8;
    flag << 6
}

fn is_sign(value: u8) -> u8 {
    value & 0x80
}