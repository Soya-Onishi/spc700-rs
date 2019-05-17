pub fn inc(op: u8) -> (u8, u8) {
    let res = op.wrapping_add(1);
    let flag = gen_flag(res);

    (res, flag)
}

pub fn dec(op: u8) -> (u8, u8) {
    let res = op.wrapping_sub(1);
    let flag = gen_flag(res);

    (res, flag)
}

fn gen_flag(res: u8) -> u8 {
    let mask = 0b1000_0010;
    let zero = ((res == 0) as u8) << 1;
    let sign = res & 0x80;
    let flag = (zero | sign) & mask;
}