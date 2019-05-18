pub fn bra(pc: u16, offset: u8) -> u16 {
    let pc = pc as i16;
    let offset =
        if offset & 0x80 > 0 {
            (offset as i16) | 0xff00
        } else {
            offset as i16
        };

    (pc.wrapping_add(offset)) as u16
}

pub fn jmp(dest: u16) -> u16 {
    dest
}

pub fn call(dest: u16) -> u16 {
    dest
}

pub fn tcall(dest: u16) -> u16 {
    dest
}

pub fn pcall(byte: u8) -> u16 {
    0xff00 | (byte as u16)
}

pub fn ret(dest: u16) -> u16 {
    dest
}

