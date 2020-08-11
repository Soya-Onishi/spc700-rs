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
    pub fn new(spc: &mut Spc700, addressing: Addressing, raw_op: u8, word_access: bool) -> (Subject, u16) {
        fn set_msb(lsb: u8, spc: &Spc700) -> u16 {
            let lsb = lsb as u16;
            let msb = if spc.reg.psw.page() { 0x0100 } else { 0 };

            msb | lsb
        }

        fn word_address(spc: &mut Spc700) -> u16 {
            let lsb_addr = spc.reg.pc;
            let msb_addr = spc.reg.pc.wrapping_add(1);
            let lsb = spc.read_ram(lsb_addr) as u16;
            let msb = spc.read_ram(msb_addr) as u16;

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
                let abs = spc.read_ram(spc.reg.pc);
                let addr = set_msb(abs, spc);
                (Subject::Addr(addr, word_access), 1)
            }
            Addressing::AbsX => {
                let abs = spc.read_ram(spc.reg.pc);
                let addr = set_msb(abs.wrapping_add(spc.reg.x), spc);

                (Subject::Addr(addr, word_access), 1)
            }
            Addressing::AbsY => {
                let abs = spc.read_ram(spc.reg.pc);
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
                let ind = spc.read_ram(spc.reg.pc);
                let ind_x = ind.wrapping_add(spc.reg.x);
                let abs_lsb = spc.read_ram(set_msb(ind_x, spc)) as u16;
                let abs_msb = spc.read_ram(set_msb(ind_x.wrapping_add(1), spc)) as u16;
                let addr = (abs_msb << 8) | abs_lsb;

                (Subject::Addr(addr, word_access), 1)
            }
            Addressing::IndAbsY => {
                let ind_addr = spc.read_ram(spc.reg.pc);
                let abs_lsb = spc.read_ram(set_msb(ind_addr, spc)) as u16;
                let abs_msb = spc.read_ram(set_msb(ind_addr.wrapping_add(1), spc)) as u16;
                let abs = (abs_msb << 8) | abs_lsb;
                let addr = abs.wrapping_add(spc.reg.y as u16);

                (Subject::Addr(addr, word_access), 1)
            }
            Addressing::AbsB => {
                let abs = spc.read_ram(spc.reg.pc) as u16;            
                let bit = raw_op >> 5;

                (Subject::Bit(abs, bit), 1)
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

    pub fn read(self, spc: &mut Spc700) -> u16 {
        match self {
            Subject::Addr(addr, is_word) => {
                let lsb = spc.read_ram(addr) as u16;
                let msb =
                    if is_word {
                        spc.read_ram(addr.wrapping_add(1)) as u16
                    } else {
                        0
                    };

                msb << 8 | lsb
            }
            Subject::Bit(addr, bit) => {
                let byte = spc.read_ram(addr);

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
                spc.ram.write(addr, lsb, &mut spc.dsp, &mut spc.timer);

                if is_word {
                    let msb = (data >> 8) as u8;
                    spc.write_ram(addr.wrapping_add(1), msb);
                }
            }
            Subject::Bit(addr, bit_pos) => {
                let origin = spc.ram.ram[addr as usize];
                let origin = origin & !(1 << bit_pos);
                let data = (data as u8) << bit_pos;

                spc.write_ram(addr, data | origin);
            }
            Subject::A => {
                if data == 0xef {
                    // println!("0xef written");
                }

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
            Subject::PSW(bit) => {
                match bit {
                    PSWBit::ALL => { spc.reg.psw.set(data as u8) }
                    PSWBit::B => { spc.reg.psw.set_brk(data & 1 > 0)}
                    PSWBit::C => { spc.reg.psw.set_carry(data & 1 > 0) }
                    PSWBit::I => { spc.reg.psw.set_interrupt(data & 1 > 0)}
                    PSWBit::N => { spc.reg.psw.set_sign(data & 1 > 0) }
                    PSWBit::P => { spc.reg.psw.set_page(data & 1 > 0) }
                    PSWBit::V => { spc.reg.psw.set_overflow(data & 1 > 0) }
                    PSWBit::Z => { spc.reg.psw.set_zero(data & 1 > 0) }
                }
            }
            Subject::None => {
                // nothing to do
            }
        }
    }
}
