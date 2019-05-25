use super::ram::*;
use super::instruction::Instruction;
use super::instruction::Addressing;
use super::instruction::Opcode;
use super::register::*;
use super::execution::*;

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

    fn write(&mut self, dst: &Subject, data: u16) {

    }

    fn alu_bit_op(&mut self, inst: &Instruction, op: impl Fn(u8, u8) -> eight_alu::RetType) -> Flag {
        let op1_sub = self.gen_subject(inst.op1, false);
        let op0_sub = self.gen_subject(inst.op0, false);
        let op0 = self.read(op0_sub);
        let op1 = self.read(op1_sub);

        let (res, pwd) = op(op0 as u8, op1 as u8);
        self.write(&op0_sub, res as u16);

        pwd
    }

    fn alu_cmp(&mut self, inst: &Instruction) -> Flag {
        let op1_sub = self.gen_subject(inst.op1, false);
        let op0_sub = self.gen_subject(inst.op0, false);
        let op0 = self.read(op0_sub);
        let op1 = self.read(op1_sub);

        let (_, pwd) = eight_alu::cmp(op0 as u8, op1 as u8);

        pwd
    }

    fn alu_op(&mut self, inst: &Instruction, op: impl Fn(u8, u8, bool) -> eight_alu::RetType) -> Flag {
        let op1_sub = self.gen_subject(inst.op1, false);
        let op0_sub = self.gen_subject(inst.op0, false);
        let op0 = self.read(op0_sub);
        let op1 = self.read(op1_sub);

        let (res, psw) = op(op0 as u8, op1 as u8, self.reg.psw.carry());
        self.write(&op0_sub, res as u16);

        psw
    }

    fn alu_shift(&mut self, inst: &Instruction, op: impl Fn(u8, bool) -> eight_shift::RetType) -> Flag {
        let op0_sub = self.gen_subject(inst.op0, false);
        let op0 = self.read(op0_sub);

        let (res, psw) = op(op0 as u8, self.reg.psw.carry());
        self.write(&op0_sub, res as u16);

        psw
    }

    fn alu_inclement(&mut self, inst: &Instruction, op: impl Fn(u8) -> inclement::RetType) -> Flag {
        let op0_sub = self.gen_subject(inst.op0, false);
        let op0 = self.read(op0_sub);

        let (res, psw) = op(op0 as u8);
        self.write(&op0_sub, res as u16);

        psw
    }

    fn alu_word_op(&mut self, inst: &Instruction, op: impl Fn(u16, u16) -> sixteen_alu::RetType) -> Flag {
        let op1_sub = self.gen_subject(inst.op1, true);
        let op0_sub = self.gen_subject(inst.op0, true);
        let op0 = self.read(op0_sub);
        let op1 = self.read(op1_sub);

        let (res, psw) = op(op0, op1);
        self.write(&op0_sub, res);

        psw
    }

    fn one_bit_op(&mut self, inst: &Instruction, op: impl Fn(u8, u8) -> one_alu::RetType) -> Flag {
        let op1_sub = self.gen_subject(inst.op1, false);
        let op0_sub = self.gen_subject(inst.op0, false);
        let op0 = self.read(op0_sub);
        let op1 = self.read(op1_sub);

        let (res, psw) = op(op0 as u8, op1 as u8);
        self.write(&op0_sub, res as u16);

        psw
    }

    fn branch(&mut self, inst: &Instruction) -> (Flag, bool) {
        let op0_sub = self.gen_subject(inst.op0, false); // either psw, [aa], [aa+X] or y
        let op0 = self.read(op0_sub);

        let rr_sub =self.gen_subject(inst.op1, false);
        let rr = self.read(rr_sub);

        let (bias, is_branch) = match inst.opcode {
            Opcode::CBNE => {
                condjump::cbne(op0 as u8, self.reg.a, rr as u8)
            }
            Opcode::DBNZ => {
                let byte = op0.wrapping_sub(1);
                let (bias, is_branch) = condjump::dbnz(byte as u8, rr as u8);

                self.write(&op0_sub, byte);

                (bias, is_branch)
            }
            _ => {
                condjump::branch(op0 as u8, rr as u8, inst.raw_op & 0x20 > 0)
            }
        };

        self.reg.pc = self.reg.pc.wrapping_add(bias);

        ((0x00, 0x00), is_branch)
    }

    fn relative_jump(&mut self, inst: &Instruction) -> Flag {
        let rr_sub = self.gen_subject(inst.op0, false);
        let rr = self.read(rr_sub);

        self.reg.pc = self.reg.pc.wrapping_add(rr);

        (0x00, 0x00)
    }

    fn absolute_jump(&mut self, inst: &Instruction) -> Flag {
        let addr_sub = self.gen_subject(inst.op0, true);

        let dst = match inst.raw_op {
            0x5f => {
                match addr_sub {
                    Subject::Addr(addr, _ ) => { addr }
                    _ => { panic!("This code is unreachable.") }
                }
            }
            0x1f => {
                self.read(addr_sub)
            }
            _ => {
                panic!("This code is unreacheable")
            }
        };

        self.reg.pc = dst;

        (0x00, 0x00)
    }

    fn push_word(&mut self, word: u16) {
        for i in 0..1 {
            let addr = 0x0100 | (self.reg.sp.wrapping_sub(i) as u16);
            let byte = ((word >> (i * 8)) & 0xff) as u8;
            self.ram.write(addr, byte);
        }

        self.reg.sp = self.reg.sp.wrapping_sub(2);
    }

    fn push_byte(&mut self, byte: u8) {
        let addr = 0x0100 | (self.reg.sp as u16);
        self.ram.write(addr, byte);

        self.reg.sp = self.reg.sp.wrapping_sub(1);
    }

    fn pull_byte(&mut self) -> u8 {
        let addr = 0x0100 | (self.reg.sp.wrapping_add(1) as u16);
        let byte = self.ram.read(addr);

        self.reg.sp = self.reg.sp.wrapping_add(1);

        byte
    }

    fn pull_word(&mut self) -> u16 {
        let addr_for_msb = 0x0100 | (self.reg.sp.wrapping_add(1) as u16);
        let addr_for_lsb = 0x0100 | (self.reg.sp.wrapping_add(2) as u16);
        let word_msb = self.ram.read(addr_for_msb) as u16;
        let word_lsb = self.ram.read(addr_for_lsb) as u16;

        self.reg.sp =self.reg.sp.wrapping_add(2);

        (word_msb << 8) | word_lsb
    }

    fn call(&mut self, inst: &Instruction) -> Flag {
        let dst = match self.gen_subject(inst.op0, false) {
            Subject::Addr(addr, _) => { addr }
            _ => { panic!("This code is unreachable") }
        };

        self.push_word(self.reg.pc);
        self.reg.pc = dst;

        (0x00, 0x00)
    }

    fn tcall(&mut self, inst: &Instruction) -> Flag {
        let n = (((inst.raw_op >> 4) & 0x0f) << 1) as u16;
        self.push_word(self.reg.pc);

        let lsb =self.ram.read(0xffde - n) as u16;
        let msb = self.ram.read(0xffde - n + 1) as u16;
        self.reg.pc = msb << 8 | lsb;

        (0x00, 0x00)
    }

    fn pcall(&mut self, inst: &Instruction) -> Flag {
        let nn_sub = self.gen_subject(inst.op0, false);
        let nn = self.read(nn_sub);
        let dst = 0xff00 | nn;

        self.push_word(self.reg.pc);
        self.reg.pc = dst;

        (0x00, 0x00)
    }

    fn ret(&mut self, inst: &Instruction) -> Flag {
        let dst = self.pull_word();
        self.reg.pc = dst;

        (0x00, 0x00)
    }

    fn ret1(&mut self, inst: &Instruction) -> Flag {
        let psw = self.pull_byte();
        let pc = self.pull_word();

        self.reg.pc = pc;
        self.reg.psw.set(psw);

        (0x00, 0x00)
    }

    fn brk(&mut self, inst: &Instruction) -> Flag {
        self.push_word(self.reg.pc);
        self.push_byte(self.reg.psw.get());

        let pc_lsb = self.ram.read(0xffde) as u16;
        let pc_msb = self.ram.read(0xffdf) as u16;
        self.reg.pc = (pc_msb << 8) | pc_lsb;

        (0b0001_0000, 0b0001_0100)
    }


}