use super::*;

type RetType = (u8, Flag);

macro_rules! adjust {
    ($acc: ident, $wrapper: ident, $half: ident, $carry: ident) => {{
        let (tmp, carry) =
            if($acc > 0x99) || $carry {
                 ($acc.$wrapper(0x60), 0b0000_0001)
            } else {
                 ($acc, 0)
            };
        let res =
            if (($acc & 0x0f) > 0x09) || $half {
                $acc.$wrapper(0x06)
            } else {
                $acc
            };

        let mask = 0b1000_0011;
        let sign = is_sign(res);
        let zero = is_zero(res);
        let flag = (sign | zero | carry) & mask;

        (res, (flag, mask))
    }}
}

pub fn daa(a: u8, half_flag: bool, carry_flag: bool) -> RetType {
    adjust!(a, wrapping_add, half_flag, carry_flag)
}

pub fn das(a: u8, half_flag: bool, carry_flag: bool) -> RetType {
    let half_flag = !half_flag;
    let carry_flag = !carry_flag;

    adjust!(a, wrapping_sub, half_flag, carry_flag)
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