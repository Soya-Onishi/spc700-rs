use super::*;

pub type RetType = (u8, Flag);

pub fn asl((op0, carry_flag): (u8, bool)) -> RetType {
    let shifter = |op, carry| -> u8 {
        op << 1
    };
    let is_carry = |op| -> bool {
        op & 0x80 > 0
    };

    shift(op0, carry_flag, shifter, is_carry)
}

pub fn rol((op0, carry_flag): (u8, bool)) -> RetType {
    let shifter = |op, carry| -> u8 {
        let c: u8 = if carry { 1 } else { 0 };
        op << 1  | c
    };
    let is_carry = |op| -> bool {
        op & 0x80 > 0
    };

    shift(op0, carry_flag, shifter, is_carry)
}

pub fn lsr((op0, carry_flag): (u8, bool)) -> RetType {
    let shifter = |op, carry| {
        (op >> 1) & (0x7f as u8)
    };
    let is_carry = |op| -> bool {
        op & 1 > 0
    };

    shift(op0, carry_flag, shifter, is_carry)
}

pub fn ror((op0, carry_flag): (u8, bool)) -> RetType {
    let shifter = |op, carry| -> u8 {
        let c = if carry { 0x80 } else { 0 };
        c | ((op0 >> 1) & 0x7f)
    };
    let is_carry = |op| -> bool {
        op & 1 > 0
    };

    shift(op0, carry_flag, shifter, is_carry)
}

fn shift(op0: u8, carry_flag: bool, shifter: impl Fn(u8, bool) -> u8, is_carry: impl Fn(u8) -> bool) -> RetType {
    let res = shifter(op0, carry_flag);

    let mask = 0b1000_0011;
    let carry = is_carry(op0) as u8;
    let sign = is_sign(res);
    let zero = is_zero(res);
    let flag = mask & (carry | sign | zero);

    (res, (flag, mask))
}