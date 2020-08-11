use super::*;

type RetType = (u8, Flag);

pub fn daa(acc: u8, half_flag: bool, carry_flag: bool) -> RetType {
    let (tmp, carry) =
        if (acc > 0x99) || carry_flag {
            (acc.wrapping_add(0x60), 0b0000_0001)
        } else {
            (acc, 0b0000_0000)
        };
    let res =
        if ((tmp & 0x0f) > 0x09) || half_flag {
            tmp.wrapping_add(0x06)
        } else {
            tmp
        };

    let mask = 0b1000_0011;
    let sign = is_sign(res);
    let zero = is_zero(res);
    let flag = (sign | zero | carry) & mask;

    (res, (flag, mask))
}

pub fn das(acc: u8, half_flag: bool, carry_flag: bool) -> RetType {
    let (tmp, carry) =
        if (acc > 0x99) || !carry_flag {
            (acc.wrapping_sub(0x60), 0b0000_0000)
        } else {
            (acc, 0b0000_0001)
        };
    let res =
        if ((tmp & 0x0f) > 0x09) || !half_flag {
            tmp.wrapping_sub(0x06)
        } else {
            tmp
        };

    let mask = 0b1000_0011;
    let sign = is_sign(res);
    let zero = is_zero(res);
    let flag = (sign | zero | carry) & mask;

    (res, (flag, mask))
}

pub fn xcn(acc: u8) -> RetType {
    let res = (acc << 4) | (acc >> 4);

    let mask = 0b1000_0010;
    let sign = is_sign(res);
    let zero = is_zero(res);
    let flag = (sign | zero) & mask;

    (res, (flag, mask))
}

pub fn tclr1(byte: u8, acc: u8) -> RetType {
    let res = byte & !acc;

    (res, gen_flag(byte, acc))
}

pub fn tset1(byte: u8, acc: u8) -> RetType {
    let res = byte | acc;


    (res, gen_flag(byte, acc))
}

fn gen_flag(byte: u8, acc: u8) -> Flag {
    let cmp = acc.wrapping_sub(byte);

    let mask = 0b1000_0010;
    let sign = is_sign(cmp);
    let zero = is_zero(cmp);
    let flag = (sign | zero) & mask;

    (flag, mask)
}