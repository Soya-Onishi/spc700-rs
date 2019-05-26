use super::core::Spc700;
use super::instruction::Addressing;

#[derive(Copy, Clone)]
pub enum Subject {
    Addr(u16, bool),
    Bit(u16, u8),
    A,
    X,
    Y,
    YA,
    SP,
    PSW,
    None,
}

impl Subject {
    // one of return values, u16, means pc incremental value
    pub fn new(spc: &Spc700, addressing: Addressing, word_access: bool) -> (Subject, u16) {
        fn set_msb(lsb: u8, spc: &Spc700) -> u16 {
            let lsb = lsb as u16;
            let msb = if spc.reg.psw.page() { 0x0100 } else { 0 };

            msb | lsb
        }

        fn word_address(spc: &Spc700) -> u16 {
            let msb_addr = spc.reg.pc;
            let lsb_addr = spc.reg.pc.wrapping_add(1);
            let lsb = spc.ram.read(lsb_addr) as u16;
            let msb = spc.ram.read(msb_addr) as u16;

            (msb << 8) | lsb
        }

        match addressing {
            Addressing::None => {
                (Subject::None, 0)
            }
            Addressing::Imm => {
                (Subject::Addr(spc.reg.pc, word_access), 1)
            }
            Addressing::A => {
                (Subject::A, 0)
            }
            Addressing::X => {
                (Subject::X, 0)
            }
            Addressing::Y => {
                (Subject::Y, 0)
            }
            Addressing::YA => {
                (Subject::YA, 0)
            }
            Addressing::SP => {
                (Subject::SP, 0)
            }
            Addressing::PSW(_) => {
                (Subject::PSW, 0)
            }
            Addressing::Abs => {
                (Subject::Addr(spc.reg.pc, word_access), 1)
            }
            Addressing::AbsX => {
                let abs = spc.ram.read(spc.reg.pc);
                let addr = set_msb(abs.wrapping_add(spc.reg.x), spc);

                (Subject::Addr(addr, word_access), 1)
            }
            Addressing::AbsY => {
                let abs = spc.ram.read(spc.reg.pc);
                let addr = set_msb(abs.wrapping_add(spc.reg.y), spc);

                (Subject::Addr(addr, word_access), 1)
            }
            Addressing::IndX => {
                (Subject::Addr(set_msb(spc.reg.x, spc), word_access), 0)
            }
            Addressing::IndY => {
                (Subject::Addr(set_msb(spc.reg.y, spc), word_access), 0)
            }
            Addressing::Abs16 => {
                (Subject::Addr(word_address(spc), word_access), 2)
            }
            Addressing::Abs16X => {
                let abs = word_address(spc);
                let addr = abs.wrapping_add(spc.reg.x as u16);

                (Subject::Addr(addr, word_access), 2)
            }
            Addressing::Abs16Y => {
                let abs = word_address(spc);
                let addr = abs.wrapping_add(spc.reg.y as u16);

                (Subject::Addr(addr, word_access), 2)
            }
            Addressing::IndAbsX => {
                let abs = spc.ram.read(spc.reg.pc);
                let abs_x = abs.wrapping_add(spc.reg.x);
                let abs_x = set_msb(abs_x, spc);
                let addr = spc.ram.read(abs_x);

                (Subject::Addr(set_msb(addr, spc), word_access), 1)
            }
            Addressing::IndAbsY => {
                let abs = spc.ram.read(spc.reg.pc);
                let abs = set_msb(abs, spc);
                let ind = spc.ram.read(abs);
                let addr = ind.wrapping_add(spc.reg.y);

                (Subject::Addr(set_msb(addr, spc), word_access), 1)
            }
            Addressing::AbsB => {
                let abs = spc.ram.read(spc.reg.pc);
                let abs = set_msb(abs, spc);

                (Subject::Addr(abs, word_access), 1)
            }
            Addressing::Abs13B => {
                let bit_addr13 = word_address(spc);

                let addr = bit_addr13 & 0x1fff;
                let bit = (bit_addr13 >> 13) & 0x0007;

                (Subject::Bit(addr, bit as u8), 2)
            }
            Addressing::Special => { (Subject::None, 0) }
        }
    }
}