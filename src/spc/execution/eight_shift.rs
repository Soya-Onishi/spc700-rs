use super::Flag;

pub type RetType = (u8, Flag);

pub fn asl(op0: u8, carry_flag: bool) -> RetType {
    let shifter = |op, carry| -> u8 {
        op << 1
    };
    let is_carry = |op| -> bool {
        op & 0x80 > 0
    };

    shift(op0, carry_flag, shifter, is_carry)
}

pub fn rol(op0: u8, carry_flag: bool) -> RetType {
    let shifter = |op, carry| -> u8 {
        let c = if carry_flag { 1 } else { 0 };
        op0 << 1  | c
    };
    let is_carry = |op| -> bool {
        op & 0x80 > 0
    };

    shift(op0, carry_flag, shifter, is_carry)
}

pub fn lsr(op0: u8, carry_flag: bool) -> RetType {
    let shifter = |op, carry| {
        (op >> 1) & (0x7f as u8)
    };
    let is_carry = |op| -> bool {
        op & 1 > 0
    };

    shift(op0, carry_flag, shifter, is_carry)
}

pub fn ror(op0: u8, carry_flag: bool) -> RetType {
    let shifter = |op, carry| -> u8 {
        let c = if carry_flag { 0x80 } else { 0 };
        op0 >> 1 | c
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
    let sign = res & 0x80;
    let zero = ((res == 0) as u8) << 1;
    let flag = mask & (carry | sign | zero);

    (res, (flag, mask))
}