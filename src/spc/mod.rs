pub mod instruction;
mod register;
mod execution;
mod ram;
pub mod core;
mod subject;

use self::instruction::Addressing;
use self::instruction::Opcode;
use self::instruction::Instruction;
use self::execution::*;
/*
#[allow(unused_variables)]
impl Spc700 {
    pub fn exec(&mut self) {
        let opcode = self.read_ram_word(self.pc);
        let inst = Instruction::decode(opcode);

        match inst.opcode {
            Opcode::MOV => { self.mov(&inst); }
            Opcode::MOVW => { self.movw(&inst); }
            Opcode::PUSH => { self.push(&inst); }
            Opcode::POP => { self.pop(&inst); }
            Opcode::OR => { self.alu_command(&inst, or) }
            Opcode::AND => { self.alu_command(&inst, and) }
            Opcode::EOR => { self.alu_command(&inst, eor) }
            Opcode::CMP => { self.alu_command(&inst, cmp) }
            Opcode::ADC => { self.alu_adc(&inst, adc) }
            Opcode::SBC => { self.alu_adc(&inst, sbc) }
        }
    }

    fn gen_word_addr(&mut self) -> u16 {
        let msb = self.read_ram_word(self.incl_pc()) as u16;
        let lsb = self.read_ram_word(self.incl_pc()) as u16;

        (msb << 8) | lsb
    }

    fn incl_pc(&mut self) -> u16 {
        self.add_pc(1)
    }

    fn add_pc(&mut self, incl: u16) -> u16 {
        self.pc = self.pc.wrapping_add(incl);
        self.pc
    }

    fn gen_subject(&mut self, addressing: Addressing) -> Subject {
        match addressing {
            Addressing::None => { Subject::None }
            Addressing::Imm => { Subject::Addr(self.incl_pc()) }
            Addressing::A => { Subject::A }
            Addressing::X => { Subject::X }
            Addressing::Y => { Subject::Y }
            Addressing::YA => { Subject::YA }
            Addressing::SP => { Subject::SP }
            Addressing::PSW(_) => { Subject::PSW }
            Addressing::Abs => { Subject::Addr(self.incl_pc()) }
            Addressing::AbsX => {
                let abs = self.read_ram_word(self.incl_pc());
                let addr = self.add_prefix_addr(abs + self.x);

                Subject::Addr(addr)
            }
            Addressing::AbsY => {
                let abs = self.read_ram_word(self.incl_pc());
                let addr = self.add_prefix_addr(abs + self.y);

                Subject::Addr(addr)
            }
            Addressing::IndX => {
                Subject::Addr(self.add_prefix_addr(self.x))
            }
            Addressing::IndY => {
                Subject::Addr(self.add_prefix_addr(self.y))
            }
            Addressing::Abs16 => {
                Subject::Addr(self.gen_word_addr())
            }
            Addressing::Abs16X => {
                let abs = self.gen_word_addr();
                let addr = abs.wrapping_add(self.x as u16);

                Subject::Addr(addr)
            }
            Addressing::Abs16Y => {
                let abs = self.gen_word_addr();
                let addr = abs + (self.y as u16);

                Subject::Addr(addr)
            }
            Addressing::IndAbsX => {
                let abs = self.read_ram_word(self.incl_pc());
                let abs_x = abs.wrapping_add(self.x);
                let addr = self.read_ram_byte(abs_x);

                Subject::Addr(self.add_prefix_addr(addr))
            }
            Addressing::IndAbsY => {
                let abs = self.read_ram_word(self.incl_pc());
                let ind = self.read_ram_byte(abs);
                let addr = ind.wrapping_add(self.y);

                Subject::Addr(self.add_prefix_addr(addr))
            }
            Addressing::AbsB => {
                let abs = self.read_ram_word(self.incl_pc());

                Subject::Addr(self.add_prefix_addr(abs))
            }
            Addressing::Abs13B => {
                let bit_addr13 = self.gen_word_addr();
                let addr = bit_addr13 & 0x1fff;
                let bit = (bit_addr13 >> 13) & 0x0007;

                Subject::Bit(addr, bit as u8)
            }
            Addressing::Special => { Subject::None }
        }
    }

    fn read_word(&self, subject: Subject) -> u16 {
        match subject {
            Subject::YA => {
                let msb = self.y as u16;
                let lsb = self.a as u16;

                (msb << 8) | lsb
            }
            Subject::Addr(addr) => {
                let lsb = self.read_ram_word(addr) as u16;
                let msb = self.read_ram_word(addr.wrapping_add(1)) as u16;

                (msb << 8) | lsb
            }
            None => {
                0
            }
            _ => {
                panic!("not allowed subject");
            }
        }
    }

    fn read_byte(&self, subject: Subject) -> u8 {
        match subject {
            Subject::SP => { self.sp }
            Subject::Addr(addr) => { self.read_ram_word(addr) }
            Subject::Bit(addr, bit) => {
                let byte = self.read_ram_word(addr);
                // TODO: get bit
            }
            Subject::A => { self.a }
            Subject::X => { self.x }
            Subject::Y => { self.y }
            Subject::PSW => { self.psw.get() }
            Subject::None => { 0 }
            _ => { panic!("not allowed subject"); }
        }
    }

    fn write_word(&mut self, subject: Subject, word: u16) {
        match subject {
            Subject::YA => {
                let y = (word >> 8) as u8;
                let a = (word & 0x00ff) as u8;

                self.y = y;
                self.a = a;
            }
            Subject::Addr(addr) => {
                let next_addr = addr.wrapping_add(1);
                let low = (word & 0xff) as u8;
                let high = ((word >> 8) & 0xff) as u8;

                self.write_ram_word(addr, low);
                self.write_ram_word(addr, high);
            }
            Subject::None => {
                // nothing to do
            }
            _ => {
                panic!("not allowed subject");
            }
        }
    }

    fn write_byte(&mut self, subject: Subject, byte: u8) {
        match subject {
            Subject::SP => { self.sp = byte; }
            Subject::Addr(addr) => { self.write_ram_byte(addr, byte); }
            Subject::Bit(addr, _) => { self.write_ram_byte(addr, byte); }
            Subject::A => { self.a = byte; }
            Subject::X => { self.x = byte; }
            Subject::Y => { self.y = byte; }
            Subject::PSW => { self.psw.set(byte); }
            Subject::None => {
                // nothing to do
            }
            _ => {
                panic!("not allowed subject");
            }
        }
    }

    fn mov(&mut self, inst: &Instruction) {
        fn set_flag(spc: &mut Spc700, value: u8) {
            spc.psw.set_sign((value >> 7) & 0x1 == 1);
            spc.psw.set_zero(value == 0);
        }

        let assigned_value = match inst.op0 {
            Addressing::Special => {
                match inst.raw_op {
                    0xAF => {
                        let addr = self.gen_subject(Addressing::IndX);
                        let a = self.read_byte(Subject::A);

                        self.write_byte(addr, a);
                        self.x = self.x.wrapping_add(1);

                        a
                    }
                    0xBF => {
                        let addr = self.gen_subject(Addressing::IndX);
                        let x = self.read_byte(addr);

                        self.write_byte(Subject::A, x);
                        self.x = self.x.wrapping_add(1);

                        x
                    }
                    _ => {
                        panic!("This is bug");
                    }
                }
            }
            _ => {
                let op0_addr = self.gen_subject(inst.op0);
                let op1 = self.read_byte(self.gen_subject(inst.op1));

                self.write_byte(op0_addr, op1);

                op1
            }
        };

        match inst.op0 {
            Addressing::A => { set_flag(&mut self, assigned_value) }
            Addressing::X => { set_flag(&mut self, assigned_value) }
            Addressing::Y => { set_flag(&mut self, assigned_value) }
            _ => {}
        }
    }

    fn movw(&mut self, inst: &Instruction) {
        let subject = self.gen_subject(inst.op1);
        let src = self.read_word(subject);

        let dst = self.gen_subject(inst.op0);

        self.write_word(dst, src);

        match inst.op0 {
            Addressing::YA => {
                self.psw.set_sign((src >> 15) & 0x1 == 1);
                self.psw.set_zero(src == 0);
            }
            _ => {}
        }
    }

    fn push(&mut self, inst: &Instruction) {
        let src = self.read_byte(self.gen_subject(inst.op0));
        let sp_addr = Subject::Addr(0x0100 | (self.sp as u16));

        self.write_byte(sp_addr, src);

        self.sp.wrapping_sub(1);
    }

    fn pop(&mut self, inst: &Instruction) {
        self.sp.wrapping_add(1);


        let sp_addr = Subject::Addr(0x0100 | (self.sp as u16));
        let src = self.read_byte(sp_addr);

        let dst = self.gen_subject(inst.op0);

        self.write_byte(dst, src);
    }

    fn alu_command(&mut self, inst: &Instruction, operation: impl Fn(u8, u8) -> (u8, (u8, u8))) -> (u8, u8) {
        let op1_sub = self.gen_subject(inst.op1);
        let op0_sub = self.gen_subject(inst.op0);

        let op0 = self.read_byte(op0_sub);
        let op1 = self.read_byte(op1_sub);

        let (res, (flag, mask)) = operation(op0, op1);

        if inst.opcode != Opcode::CMP {
            self.write_byte(op0_sub, res);
        }

        (flag, mask)
    }

    fn alu_adc(&mut self, inst: &Instruction, operation: impl Fn(u8, u8, bool) -> (u8, (u8, u8))) -> (u8, u8) {
        let op1_sub = self.gen_subject(inst.op1);
        let op0_sub = self.gen_subject(inst.op0);

        let op0 = self.read_byte(op0_sub);
        let op1 = self.read_byte(op1_sub);
        let c = self.psw.carry();

        let (res, (flag, mask)) = operation(op0, op1, c);

        self.write_byte(op0_sub, res);

        (flag, mask)
    }

    fn eight_bit_command(&mut self, inst: &Instruction, operation: impl Fn(u8) -> u8) {
        let op0_sub = self.gen_subject(inst.op0);
        let op0 = self.read_byte(op0_sub);

        let res = operation(op0);

        self.write_byte(op0_sub, res);
    }

    fn sixteen_bit_command(&mut self, inst: &Instruction, operation: impl Fn(u16, u16) -> u16) {
        let op1_sub = self.gen_subject(inst.op1);
        let op0_sub = self.gen_subject(inst.op0);

        let op0 = self.read_word(op0_sub);
        let op1 = self.read_word(op1_sub);

        let res = operation(op0, op1);

        self.write_word(op0_sub, res);
    }

    fn one_bit_command(&mut self, inst: &Instruction, operation: impl Fn(u8, u8) -> u8) {
        let op1_sub = self.gen_subject(inst.op1);
        let op0_sub = self.gen_subject(inst.op0);

        let op0 = self.read_byte(op0_sub);
        let op1 = match inst.opcode {
            Opcode::SET1 => { ((inst.raw_op - 0x12) >> 5) }
            Opcode::CLR1 => { ((inst.raw_op - 0x02) >> 5) }
            _ => { self.read_byte(op1_sub) }
        };

        let res = operation(op0, op1);

        self.write_byte(op0_sub, res);
    }

    fn cmp(&mut self, op0: u8, op1: u8) -> u8 {
        let res = op0 - op1;

        self.psw.set_carry(res > 0xff);

        0
    }

    fn adc(&mut self, op0: u8, op1: u8) -> u8 {
        let c: u16 = if self.psw.carry() { 1 } else { 0 };

        let res = op0.wrapping_add(op1).wrapping_add(c);

        self.psw.set_half(((op0 ^ op1 ^ res) & 0x10) == 1);
        self.psw.set_carry(res > 0xff);
        self.psw.set_overflow((!(op0 ^ op1) & (op0 ^ res) & 0x80) == 1);

        res & 0xff
    }

    fn sbc(&mut self, op0: u8, op1: u8) -> u8 {
        let res = self.adc(op0, !op1 & 0xff);

        res & 0xff
    }

    fn asl(&mut self, op0: u8) -> u8 {
        let res = op0 << 1;

        self.psw.set_carry(res & 0x100 == 1);

        res & 0xff
    }

    fn rol(&mut self, op0: u8) -> u8 {
        let c: u16 = if self.psw.carry() { 1 } else { 0 };
        let res = op0 << 1 | c;

        self.psw.set_carry(res & 0x100 == 1);

        res & 0xff
    }

    fn lsr(&mut self, op0: u8) -> u8 {
        let res = op0 >> 1;

        self.psw.set_carry(op0 & 0x1 == 1);

        res & 0xff
    }

    fn ror(&mut self, op0: u8) -> u8 {
        let c: u16 = if self.psw.carry() { 0x80 } else { 0 };
        let res = op0 >> 1 | c;

        self.psw.set_carry(op0 & 0x1 == 0);

        res & 0xff
    }

    fn inc(op0: u8) -> u8 {
        op0.wrapping_add(1)
    }

    fn dec(op0: u8) -> u8 {
        op0.wrapping_sub(1)
    }

    fn addw(&mut self, op0: u16, op1: u16) -> u16 {
        self.psw.negate_carry();

        let op0_lsb = op0 as u8;
        let op0_msb = (op0 >> 8) as u8;
        let op1_lsb = op1 as u8;
        let op1_msb = (op1 >> 8) as u8;

        let lsb = self.adc(op0_lsb, op1_lsb) as u16;
        let msb = self.adc(op0_msb, op1_msb) as u16;

        (msb << 8) | lsb
    }

    fn subw(&mut self, op0: u16, op1: u16) -> u16 {
        self.psw.assert_carry();

        let op0_lsb = op0 as u8;
        let op0_msb = (op0 >> 8) as u8;
        let op1_lsb = op1 as u8;
        let op1_msb = (op1 >> 8) as u8;

        let lsb = self.sbc(op0_lsb, op1_lsb) as u16;
        let msb = self.sbc(op0_msb, op1_msb) as u16;

        (msb << 8) | lsb
    }
}
*/