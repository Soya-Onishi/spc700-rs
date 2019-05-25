
pub type RetType = (u16, bool); // (destination, is_branched)

/*
pub fn bpl(pwd: u8, pc: u16, offset: u8) -> RetType {
    branch(!pwd, 0x80, pc, offset)
}

pub fn bmi(pwd: u8, pc: u16, offset: u8) -> RetType {
    branch(pwd, 0x80, pc, offset)
}

pub fn bvc(pwd: u8, pc: u16, offset: u8) -> RetType {
    branch(!pwd, 0x40, pc, offset)
}

pub fn bvs(pwd: u8, pc: u16, offset: u8) -> RetType {
    branch(pwd, 0x40, pc, offset)
}

pub fn bcc(pwd: u8, pc: u16, offset: u8) -> RetType {
    branch(!pwd, 0x01, pc, offset)
}

pub fn bcs(pwd: u8, pc: u16, offset: u8) -> RetType {
    branch(pwd, 0x01, pc, offset)
}

pub fn bne(pwd: u8, pc: u16, offset: u8) -> RetType {
    branch(!pwd, 0x02, pc, offset)
}

pub fn beq(pwd: u8, pc: u16, offset: u8) -> RetType {
    branch(pwd, 0x02, pc, offset)
}

pub fn bbs(byte: u8, pc: u16, offset: u8) -> RetType {
    branch(byte, 0x01, pc, offset)
}

pub fn bbc(byte: u8, pc: u16, offset: u8) -> RetType {
    branch(!byte, 0x01, pc, offset)
}
*/

pub fn cbne(byte: u8, acc: u8, offset: u8) -> RetType {
    let is_branch = byte != acc;
    let bias = get_bias(offset, is_branch);

    (bias, is_branch)
}

pub fn dbnz(byte: u8, offset: u8) -> RetType {
    let is_branch = byte != 0;
    let bias = get_bias(offset, is_branch);

    (bias, is_branch)
}

pub fn branch(bit: u8, offset: u8, is_true: bool) -> RetType {
    let is_branch = (bit & 1 > 0) == is_true;
    let bias = get_bias(offset, is_branch);

    (bias, is_branch)
}

fn get_bias(offset: u8, is_branch: bool) -> u16 {
    let offset =
        if offset & 0x80 > 0 {
            (offset as u16) | 0xff00
        } else {
            offset as u16
        };

    if is_branch { offset } else { 0 }
}