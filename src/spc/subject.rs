use super::core::Spc700;
use super::instruction::Addressing;
use super::instruction::PSWBit;

#[derive(Copy, Clone)]
pub enum Subject {
    Addr(u16, bool),
    Bit(u16, u8),
    A,
    X,
    Y,
    YA,
    SP,
    PSW(PSWBit),
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
            let lsb_addr = spc.reg.pc;
            let msb_addr = spc.reg.pc.wrapping_add(1);
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
            Addressing::PSW(psw_bit) => {
                (Subject::PSW(psw_bit), 0)
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

    pub fn read(self, spc: &Spc700) -> u16 {
        match self {
            Subject::Addr(addr, is_word) => {
                let lsb = spc.ram.read(addr) as u16;
                let msb =
                    if is_word {
                        spc.ram.read(addr.wrapping_add(1)) as u16
                    } else {
                        0
                    };

                msb << 8 | lsb
            }
            Subject::Bit(addr, bit) => {
                let byte = spc.ram.read(addr);

                ((byte >> bit) & 1) as u16
            }
            Subject::A => {
                spc.reg.a as u16
            }
            Subject::X => {
                spc.reg.x as u16
            }
            Subject::Y => {
                spc.reg.y as u16
            }
            Subject::PSW(bit) => {
                match bit {
                    PSWBit::ALL => { spc.reg.psw.get() as u16 }
                    PSWBit::B => { spc.reg.psw.brk() as u16 }
                    PSWBit::C => { spc.reg.psw.carry() as u16 }
                    PSWBit::I => { spc.reg.psw.interrupt() as u16 }
                    PSWBit::N => { spc.reg.psw.sign() as u16 }
                    PSWBit::P => { spc.reg.psw.page() as u16 }
                    PSWBit::V => { spc.reg.psw.overflow() as u16 }
                    PSWBit::Z => { spc.reg.psw.zero() as u16 }
                }
            }
            Subject::SP => {
                spc.reg.sp as u16
            }
            Subject::YA => {
                let msb = spc.reg.y as u16;
                let lsb = spc.reg.a as u16;

                (msb << 8) | lsb
            }
            Subject::None => {
                0
            }
        }
    }

    pub fn write(self, spc: &mut Spc700, data: u16) {
        match self {
            Subject::Addr(addr, is_word) => {
                let lsb = data as u8;
                spc.ram.write(addr, lsb);

                if is_word {
                    let msb = (data >> 8) as u8;
                    spc.ram.write(addr.wrapping_add(1), msb);
                }
            }
            Subject::Bit(addr, bit_pos) => {
                let origin = spc.ram.read(addr);
                let origin = origin & !(1 << bit_pos);
                let data = (data as u8) << bit_pos;

                spc.ram.write(addr, data | origin);
            }
            Subject::A => {
                println!("a = 0x{:02x}", data);
                spc.reg.a = data as u8;
            }
            Subject::X => {
                spc.reg.x = data as u8;
            }
            Subject::Y => {
                spc.reg.y = data as u8;
            }
            Subject::YA => {
                let y = (data >> 8) as u8;
                let a = (data & 0x00ff) as u8;

                spc.reg.y = y;
                spc.reg.a = a;
            }
            Subject::SP => {
                spc.reg.sp = data as u8;
            }
            Subject::PSW(_) => {
                spc.reg.psw.set(data as u8);
            }
            Subject::None => {
                // nothing to do
            }
        }
    }
}