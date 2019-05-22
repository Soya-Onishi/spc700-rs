use super::ram::*;
use super::instruction::Instruction;
use super::instruction::Addressing;
use super::register::*;

#[derive(Copy, Clone)]
enum Subject {
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

pub struct Spc700 {
    pub reg: Register,
    pub ram: Ram,
}

impl Spc700 {
    pub fn new() -> Spc700 {
        Spc700 {
            reg: Register::new(),
            ram: Ram::new(),
        }
    }

    pub fn execute(&mut self) {
        let pc = self.incl_pc();
        let opcode = self.ram.read(pc);
        let inst = Instruction::decode(opcode);


    }

    fn incl_pc(&mut self) -> u16{
        let pc = self.reg.pc;
        self.reg.pc = pc.wrapping_add(1);

        pc
    }

    fn set_addr_msb(&self, addr: u8) -> u16 {
        let addr = addr as u16;

        if self.reg.psw.page() {
            0x0100 | addr
        } else {
            0x0000 | addr
        }
    }
    //    fn get_addr(&self, addr: u16) -> u16 {
//        let byte = self.ram.read(addr);
//
//        self.set_addr_msb(byte)
//    }

    fn get_word_addr(&mut self) -> u16 {
        let msb_pc = self.incl_pc();
        let lsb_pc = self.incl_pc();
        let msb = self.ram.read(msb_pc) as u16;
        let lsb = self.ram.read(lsb_pc) as u16;

        msb << 8 | lsb
    }

    fn gen_subject(&mut self, addressing: Addressing, word_access: bool) -> Subject {
        match addressing {
            Addressing::None => {
                Subject::None
            }
            Addressing::Imm => {
                Subject::Addr(self.incl_pc(), word_access)
            }
            Addressing::A => {
                Subject::A
            }
            Addressing::X => {
                Subject::X
            }
            Addressing::Y => {
                Subject::Y
            }
            Addressing::YA => {
                Subject::YA
            }
            Addressing::SP => {
                Subject::SP
            }
            Addressing::PSW(_) => {
                Subject::PSW
            }
            Addressing::Abs => {
                Subject::Addr(self.incl_pc(), word_access)
            }
            Addressing::AbsX => {
                let pc = self.incl_pc();
                let abs = self.ram.read(pc);
                let addr = self.set_addr_msb(abs.wrapping_add(self.reg.x));

                Subject::Addr(addr, word_access)
            }
            Addressing::AbsY => {
                let pc = self.incl_pc();
                let abs = self.ram.read(pc);
                let addr = self.set_addr_msb(abs.wrapping_add(self.reg.y));

                Subject::Addr(addr, word_access)
            }
            Addressing::IndX => {
                Subject::Addr(self.set_addr_msb(self.reg.x), word_access)
            }
            Addressing::IndY => {
                Subject::Addr(self.set_addr_msb(self.reg.y), word_access)
            }
            Addressing::Abs16 => {
                Subject::Addr(self.get_word_addr(), word_access)
            }
            Addressing::Abs16X => {
                let abs = self.get_word_addr();
                let addr = abs.wrapping_add(self.reg.x as u16);

                Subject::Addr(addr, word_access)
            }
            Addressing::Abs16Y => {
                let abs = self.get_word_addr();
                let addr = abs.wrapping_add(self.reg.y as u16);

                Subject::Addr(addr, word_access)
            }
            Addressing::IndAbsX => {
                let pc = self.incl_pc();
                let abs = self.ram.read(pc);
                let abs_x = abs.wrapping_add(self.reg.x);
                let abs_x = self.set_addr_msb(abs_x);
                let addr = self.ram.read(abs_x);

                Subject::Addr(self.set_addr_msb(addr), word_access)
            }
            Addressing::IndAbsY => {
                let pc = self.incl_pc();
                let abs = self.ram.read(pc);
                let abs = self.set_addr_msb(abs);
                let ind = self.ram.read(abs);
                let addr = ind.wrapping_add(self.reg.y);

                Subject::Addr(self.set_addr_msb(addr), word_access)
            }
            Addressing::AbsB => {
                let pc = self.incl_pc();
                let abs = self.ram.read(pc);
                let abs = self.set_addr_msb(abs);

                Subject::Addr(abs, word_access)
            }
            Addressing::Abs13B => {
                let bit_addr13 = self.get_word_addr();

                let addr = bit_addr13 & 0x1fff;
                let bit = (bit_addr13 >> 13) & 0x0007;

                Subject::Bit(addr, bit as u8)
            }
            Addressing::Special => { Subject::None }
        }
    }

    fn read(&self, subject: Subject) -> u16 {
        match subject {
            Subject::Addr(addr, is_word) => {
                let lsb = self.ram.read(addr) as u16;
                let msb =
                    if is_word {
                        self.ram.read(addr.wrapping_add(1)) as u16
                    } else {
                        0
                    };

                msb << 8 | lsb
            }
            Subject::Bit(addr, bit) => {
                let byte = self.ram.read(addr);

                ((byte >> bit) & 1) as u16
            }
            Subject::A => {
                self.reg.a as u16
            }
            Subject::X => {
                self.reg.x as u16
            }
            Subject::Y => {
                self.reg.y as u16
            }
            Subject::PSW => {
                self.reg.psw.get() as u16
            }
            Subject::SP => {
                self.reg.sp as u16
            }
            Subject::YA => {
                let msb = self.reg.y as u16;
                let lsb = self.reg.a as u16;

                (msb << 8) | lsb
            }
            Subject::None => {
                0
            }
        }
    }

}