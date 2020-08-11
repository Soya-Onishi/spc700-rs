use super::eight_alu::{adc, sbc};

pub type RetType = (u16, super::Flag);

pub fn movw(src: u16) -> RetType {
    let mask = 0b1000_0010;
    let sign = (src & 0x8000 > 0) as u8;
    let zero = (src == 0) as u8;
    let flag = (sign << 7) | (zero << 1);

    (src, (mask, flag))
}

pub fn addw(op0: u16, op1: u16) -> RetType {
    arithmetic(op0, op1, false, adc)
}

pub fn subw(op0: u16, op1: u16) -> RetType {
    arithmetic(op0, op1, true, sbc)
}

fn arithmetic(op0: u16, op1: u16, is_sbc: bool, op: impl Fn(u8, u8, bool) -> (u8, super::Flag)) -> RetType {
    let op0_lsb = op0 as u8;
    let op1_lsb = op1 as u8;
    let op0_msb = (op0 >> 8) as u8;
    let op1_msb = (op1 >> 8) as u8;

    let (lsb, (flag, _)) = op(op0_lsb, op1_lsb, is_sbc);
    let carry = (flag & 0x1) > 0;
    let (msb, (flag, mask)) = op(op0_msb, op1_msb, carry);

    let res = ((msb as u16) << 8) | (lsb as u16);
    let zero = is_zero(res);

    let flag = (flag & (0b1111_1101)) | zero;

    (res, (flag, mask))
}

pub fn cmpw(op0: u16, op1: u16) -> RetType {
    let op0 = op0 as u32;
    let op1 = op1 as u32;

    let res = op0.wrapping_sub(op1);

    let mask = 0b1000_0011;
    let sign = is_sign(res as u16);
    let zero = is_zero(res as u16);
    let carry = is_carry(res);
    let flag = (sign | zero | carry) & mask;

    (res as u16, (flag, mask))
}

pub fn incw(op: u16, _dummy: u16) -> RetType {
    let res = op.wrapping_add(1);
    let flag = inclemenal_flag(res);

    (res, flag)
}

pub fn decw(op: u16, _dummy: u16) -> RetType {
    let res = op.wrapping_sub(1);
    let flag = inclemenal_flag(res);

    (res, flag)
}

pub fn div(ya: u16, x: u16) -> RetType {
    fn is_sign(a: u16) -> u8 {
        (a & 0x80) as u8
    }
    fn is_zero(a: u16) -> u8 {
        let flag = (a & 0xff) == 0;
        (flag as u8) << 1
    }
    fn is_overflow(a: u16) -> u8 {
        let flag = a > 0xff;
        (flag as u8) << 6
    }
    fn is_half(y: u16, x: u16) -> u8 {
        let flag = (x & 0xf) <= (y & 0xf);
        (flag as u8) << 3
    }

    let (a, y, ya) =
        if x == 0 {
            (ya, 0, ya & 0xff)
        } else {
            let a = ya / x;
            let y = ya % x;
            let ya = y << 8 | a;

            (a, y, ya)
        };


    let mask = 0b1100_1010;
    let sign = is_sign(a);
    let zero = is_zero(a);
    let overflow = is_overflow(a);
    let half = is_half(y, x);
    let flag = (sign | zero | overflow | half) & mask;

    (ya, (flag, mask))
}

pub fn mul(ya: u16, _dummy: u16) -> RetType {
    fn is_zero(ya: u16) -> u8 {
        let flag = (ya & 0xff00) == 0;
        (flag as u8) << 1
    }
    fn is_sign(ya: u16) -> u8 {
        let flag = (ya & 0x8000) > 0;
        (flag as u8) << 7
    }

    let y = (ya >> 8) & 0xff;
    let a = ya & 0xff;
    let ya = y.wrapping_mul(a);

    let mask = 0b1000_0010;
    let sign = is_sign(ya);
    let zero = is_zero(ya);
    let flag = (sign | zero) & mask;

    (ya, (flag, mask))
}

fn inclemenal_flag(res: u16) -> super::Flag {
    let mask = 0b1000_0010;
    let sign = is_sign(res);
    let zero = is_zero(res);

    ((sign | zero) & mask, mask)
}

fn is_sign(res: u16) -> u8 {
    ((res & 0x8000) >> 8) as u8
}

fn is_zero(res: u16) -> u8 {
    let flag = res == 0;
    (flag as u8) << 1
}

fn is_carry(res: u32) -> u8 {
    let res = (res >> 8) as u16;

    (res > 0x00ff) as u8
}
