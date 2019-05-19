type RetType = (u16, super::Flag);

pub fn addw(op0: u16, op1: u16) -> RetType {
    let res = op0.wrapping_add(op1);

    let mask = 0b1100_1011;
    let sign = is_sign(res);
    let overflow = is_overflow(op0, op1, res);
    let half = is_half(op0, op1, res);
    let zero = is_zero(res);
    let carry = is_carry(op0, res);
    let flag = (sign | overflow | half | zero | carry) & mask;

    (res, (flag, mask))
}

pub fn subw(op0: u16, op1: u16) -> RetType {
    addw(op0, (!op1).wrapping_add(1))
}

fn cmpw(op0: u16, op1: u16) -> RetType {
    let res = op0.wrapping_sub(op1);

    let mask = 0b1000_0011;
    let sign = is_sign(res);
    let zero = is_zero(res);
    let carry = is_carry(op0, res);
    let flag = (sign | zero | carry) & mask;

    (res, (flag, mask))
}

fn incw(op: u16) -> RetType {
    let res = op.wrapping_add(1);
    let flag = inclemenal_flag(res);

    (res, flag)
}

fn decw(op: u16) -> RetType {
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

pub fn mul(ya: u16) -> RetType {
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

fn is_overflow(op0: u16, op1: u16, res: u16) -> u8 {
    let flag = (!(op0 ^ op1) & (op0 ^ res) & 0x8000) > 1;
    (flag as u8) << 6
}

fn is_half(op0: u16, op1: u16, res: u16) -> u8 {
    let flag = ((op0 ^ op1 ^ res) & 0x1000) > 1;
    (flag as u8) << 3
}

fn is_zero(res: u16) -> u8 {
    let flag = res == 0;
    (flag as u8) << 1
}

fn is_carry(op0: u16, res: u16) -> u8 {
    (op0 > res) as u8
}