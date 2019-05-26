use super::ram::*;
use super::instruction::Instruction;
use super::instruction::Addressing;
use super::instruction::Opcode;
use super::register::*;
use super::execution::*;
use super::subject::Subject;

trait BinOp<T> {
    fn binop(&mut self, inst: &Instruction, op: impl Fn(T, T) -> (T, Flag)) -> Flag;
}

impl BinOp<u8> for Spc700 {
    fn binop(&mut self, inst: &Instruction, op: impl Fn(u8, u8) -> (u8, Flag)) -> Flag {
        let (op1_sub, incl) = Subject::new(self, inst.op1, false);
        self.reg.inc_pc(incl);
        let (op0_sub, incl) = Subject::new(self, inst.op0, false);
        self.reg.inc_pc(incl);

        let op0 = self.read(op0_sub);
        let op1 = self.read(op1_sub);

        let (res, pwd) = op(op0 as u8, op1 as u8);

        if inst.opcode != Opcode::CMP {
            self.write(&op0_sub, res as u16);
        }

        pwd
    }
}

impl BinOp<u16> for Spc700 {
    fn binop(&mut self, inst: &Instruction, op: impl Fn(u16, u16) -> (u16, Flag)) -> Flag {
        let (op1_sub, incl) = Subject::new(self, inst.op1, true);
        self.reg.inc_pc(incl);
        let (op0_sub, incl) = Subject::new(self, inst.op0, true);
        self.reg.inc_pc(incl);

        let op0 = self.read(op0_sub);
        let op1 = self.read(op1_sub);

        let (res, psw) = op(op0, op1);
        self.write(&op0_sub, res);

        psw
    }
}

trait UnaryOp<T> {
    fn unaryop(&mut self, inst: &Instruction, op: impl Fn(T) -> (u8, Flag)) -> Flag;
}

impl UnaryOp<u8> for Spc700 {
    fn unaryop(&mut self, inst: &Instruction, op: impl Fn(u8) -> (u8, Flag)) -> Flag {
        let (op0_sub, incl) = Subject::new(self, inst.op0, false);
        self.reg.inc_pc(incl);

        let op0 = self.read(op0_sub);

        let (res, psw) = op(op0 as u8);
        self.write(&op0_sub, res as u16);

        psw
    }
}


impl UnaryOp<(u8, bool)> for Spc700 {
    fn unaryop(&mut self, inst: &Instruction, op: impl Fn((u8, bool)) -> (u8, Flag)) -> Flag {
        let (op0_sub, incl) = Subject::new(self, inst.op0, false);
        self.reg.inc_pc(incl);

        let op0 = self.read(op0_sub);

        let (res, psw) = op((op0 as u8, self.reg.psw.carry()));
        self.write(&op0_sub, res as u16);

        psw
    }
}

trait PullOperation<T> {
    fn pull(&mut self) -> T;
}

impl PullOperation<u8> for Spc700 {
    fn pull(&mut self) -> u8 {
        let addr = 0x0100 | (self.reg.sp.wrapping_add(1) as u16);
        let byte = self.ram.read(addr);

        self.reg.sp = self.reg.sp.wrapping_add(1);

        byte
    }
}

impl PullOperation<u16> for Spc700 {
    fn pull(&mut self) -> u16 {
        let addr_for_msb = 0x0100 | (self.reg.sp.wrapping_add(1) as u16);
        let addr_for_lsb = 0x0100 | (self.reg.sp.wrapping_add(2) as u16);
        let word_msb = self.ram.read(addr_for_msb) as u16;
        let word_lsb = self.ram.read(addr_for_lsb) as u16;

        self.reg.sp =self.reg.sp.wrapping_add(2);

        (word_msb << 8) | word_lsb
    }
}

trait PushOperation<T> {
    fn push(&mut self, data: T);
}

impl PushOperation<u8> for Spc700 {
    fn push(&mut self, data: u8) {
        let addr = 0x0100 | (self.reg.sp as u16);
        self.ram.write(addr, data);

        self.reg.sp = self.reg.sp.wrapping_sub(1);
    }
}

impl PushOperation<u16> for Spc700 {
    fn push(&mut self, data: u16) {
        for i in 0..1 {
            let addr = 0x0100 | (self.reg.sp.wrapping_sub(i) as u16);
            let byte = ((data >> (i * 8)) & 0xff) as u8;
            self.ram.write(addr, byte);
        }

        self.reg.sp = self.reg.sp.wrapping_sub(2);
    }
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
        let pc = self.reg.inc_pc(1);
        let opcode = self.ram.read(pc);
        let inst = Instruction::decode(opcode);

        match inst.opcode {
            Opcode::MOV => { self.binop(&inst, eight_alu::mov) }
            Opcode::MOVW => { self.binop(&inst, sixteen_alu::movw) }
            Opcode::OR => { self.binop(&inst, eight_alu::or) }
            Opcode::AND => { self.binop(&inst, eight_alu::and) }
            Opcode::EOR => { self.binop(&inst, eight_alu::eor) }
            Opcode::CMP => { self.binop(&inst, eight_alu::cmp) }
            Opcode::ADC => { self.calc_with_carry(&inst, eight_alu::adc) }
            Opcode::SBC => { self.calc_with_carry(&inst, eight_alu::sbc) }
            Opcode::ASL => { self.unaryop(&inst, eight_shift::asl) }
            Opcode::ROL => { self.unaryop(&inst, eight_shift::rol) }
            Opcode::LSR => { self.unaryop(&inst, eight_shift::lsr) }
            Opcode::ROR => { self.unaryop(&inst, eight_shift::ror) }
            Opcode::INC => { self.unaryop(&inst, inclement::inc) }
            Opcode::DEC => { self.unaryop(&inst, inclement::dec) }
            Opcode::ADDW => { self.binop(&inst, sixteen_alu::addw) }
            Opcode::SUBW => { self.binop(&inst, sixteen_alu::subw) }
            Opcode::CMPW => { self.binop(&inst, sixteen_alu::cmpw) }
            Opcode::INCW => { self.binop(&inst, sixteen_alu::incw) }
            Opcode::DECW => { self.binop(&inst, sixteen_alu::decw) }
            Opcode::DIV => { self.binop(&inst, sixteen_alu::div) }
            Opcode::MUL => { self.binop(&inst, sixteen_alu::mul) }
            Opcode::CLR1 => { self.binop(&inst, one_alu::clr1) }
            Opcode::SET1 => { self.binop(&inst, one_alu::set1) }
            Opcode::NOT1 => { self.binop(&inst, one_alu::not1) }
            Opcode::MOV1 => { self.binop(&inst, one_alu::mov1) }
            Opcode::OR1 => { self.binop(&inst, one_alu::or1) }
            Opcode::AND1 => { self.binop(&inst, one_alu::and1) }
            Opcode::EOR1 => { self.binop(&inst, one_alu::eor1) }
            Opcode::CLRC => { self.binop(&inst, one_alu::clrc) }
            Opcode::SETC => { self.binop(&inst, one_alu::setc) }
            Opcode::NOTC => { self.binop(&inst, one_alu::notc) }
            Opcode::CLRV => { self.binop(&inst, one_alu::clrv) }
            Opcode::DAA => { self.trans_into_decimal(&inst, special::daa) }
            Opcode::DAS => { self.trans_into_decimal(&inst, special::das) }
            Opcode::XCN => { self.unaryop(&inst, special::xcn) }
            Opcode::TCLR1 => { self.binop(&inst, special::tclr1) }
            Opcode::TSET1 => { self.binop(&inst, special::tset1) }
            Opcode::BPL => {
                let (psw, is_branch) = self.branch(&inst);
                if is_branch { self.cycle_up(2) }
                psw
            }
            Opcode::BMI => {
                let (psw, is_branch) = self.branch(&inst);
                if is_branch { self.cycle_up(2) }
                psw
            }
            Opcode::BVC => {
                let (psw, is_branch) = self.branch(&inst);
                if is_branch { self.cycle_up(2) }
                psw
            }
            Opcode::BVS => {
                let (psw, is_branch) = self.branch(&inst);
                if is_branch { self.cycle_up(2) }
                psw
            }
            Opcode::BCC => {
                let (psw, is_branch) = self.branch(&inst);
                if is_branch { self.cycle_up(2) }
                psw
            }
            Opcode::BCS => {
                let (psw, is_branch) = self.branch(&inst);
                if is_branch { self.cycle_up(2) }
                psw
            }
            Opcode::BNE => {
                let (psw, is_branch) = self.branch(&inst);
                if is_branch { self.cycle_up(2) }
                psw
            }
            Opcode::BEQ => {
                let (psw, is_branch) = self.branch(&inst);
                if is_branch { self.cycle_up(2) }
                psw
            }
            Opcode::BBS => {
                let (psw, is_branch) = self.branch(&inst);
                if is_branch { self.cycle_up(2) }
                psw
            }
            Opcode::BBC => {
                let (psw, is_branch) = self.branch(&inst);
                if is_branch { self.cycle_up(2) }
                psw
            }
            Opcode::CBNE => {
                let (psw, is_branch) = self.branch(&inst);
                if is_branch { self.cycle_up(2) }
                psw
            }
            Opcode::DBNZ => {
                let (psw, is_branch) = self.branch(&inst);
                if is_branch { self.cycle_up(2) }
                psw
            }
            Opcode::BRA => { self.relative_jump(&inst) }
            Opcode::JMP => { self.absolute_jump(&inst) }
            Opcode::CALL => { self.call(&inst) }
            Opcode::TCALL => { self.tcall(&inst) }
            Opcode::PCALL => { self.pcall(&inst) }
            Opcode::RET => { self.ret() }
            Opcode::RETI => { self.ret1() }
            Opcode::BRK => { self.brk() }
            Opcode::NOP => { (0x00, 0x00) /* nothing to do */ }
            Opcode::SLEEP => { panic!("SPC700 is suspended by SLEEP") }
            Opcode::STOP => { panic!("SPC700 is suspended by STOP") }
            Opcode::CLRP => { self.clrp() }
            Opcode::SETP => { self.setp() }
            Opcode::EI => { self.ei() }
            Opcode::DI => { self.di() }
        };
    }

    fn cycle_up(&mut self, count: u64) {

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

    fn trans_into_decimal(&mut self, inst: &Instruction, op: impl Fn(u8, bool, bool) -> (u8, Flag)) -> Flag {
        let (op0_sub, inc) = Subject::new(self, inst.op0, false);
        let op0 = self.read(op0_sub);
        self.reg.inc_pc(inc);

        let (res, psw) = op(op0 as u8, self.reg.psw.half(), self.reg.psw.carry());
        self.write(&op0_sub, res as u16);

        psw
    }

    fn calc_with_carry(&mut self, inst: &Instruction, op: impl Fn(u8, u8, bool) -> (u8, Flag)) -> Flag {
        let (op1_sub, incl) = Subject::new(self, inst.op1, false);
        self.reg.inc_pc(incl);
        let (op0_sub, incl) = Subject::new(self, inst.op0, false);
        self.reg.inc_pc(incl);

        let op0 = self.read(op0_sub) as u8;
        let op1 = self.read(op1_sub) as u8;

        let (res, pwd) = op(op0, op1, self.reg.psw.carry());

        if inst.opcode != Opcode::CMP {
            self.write(&op0_sub, res as u16);
        }

        pwd
    }

    fn branch(&mut self, inst: &Instruction) -> (Flag, bool) {
        let (op0_sub, incl) = Subject::new(self, inst.op0, false); // either psw, [aa], [aa+X] or y
        self.reg.inc_pc(incl);
        let op0 = self.read(op0_sub);

        let (rr_sub, incl) =Subject::new(self, inst.op1, false);
        self.reg.inc_pc(incl);
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
        let (rr_sub, inc) = Subject::new(self,inst.op0, false);
        let rr = self.read(rr_sub);
        self.reg.inc_pc(inc);


        self.reg.pc = self.reg.pc.wrapping_add(rr);

        (0x00, 0x00)
    }

    fn absolute_jump(&mut self, inst: &Instruction) -> Flag {
        let (addr_sub, inc) = Subject::new(self, inst.op0, true);
        self.reg.inc_pc(inc);

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

    fn call(&mut self, inst: &Instruction) -> Flag {
        let (subject, inc) = Subject::new(self, inst.op0, false);
        self.reg.inc_pc(inc);

        let dst = match subject {
            Subject::Addr(addr, _) => { addr }
            _ => { panic!("This code is unreachable") }
        };

        self.push(self.reg.pc);
        self.reg.pc = dst;

        (0x00, 0x00)
    }

    fn tcall(&mut self, inst: &Instruction) -> Flag {
        let n = (((inst.raw_op >> 4) & 0x0f) << 1) as u16;
        self.push(self.reg.pc);

        let lsb =self.ram.read(0xffde - n) as u16;
        let msb = self.ram.read(0xffde - n + 1) as u16;
        self.reg.pc = msb << 8 | lsb;

        (0x00, 0x00)
    }

    fn pcall(&mut self, inst: &Instruction) -> Flag {
        let (nn_sub, inc) = Subject::new(self, inst.op0, false);
        self.reg.inc_pc(inc);
        let nn = self.read(nn_sub);
        let dst = 0xff00 | nn;

        self.push(self.reg.pc);
        self.reg.pc = dst;

        (0x00, 0x00)
    }

    fn ret(&mut self) -> Flag {
        let dst: u16 = self.pull();
        self.reg.pc = dst;

        (0x00, 0x00)
    }

    fn ret1(&mut self) -> Flag {
        let psw: u8 = self.pull();
        let pc: u16 = self.pull();

        self.reg.pc = pc;
        self.reg.psw.set(psw);

        (0x00, 0x00)
    }

    fn brk(&mut self) -> Flag {
        self.push(self.reg.pc);
        self.push(self.reg.psw.get());

        let pc_lsb = self.ram.read(0xffde) as u16;
        let pc_msb = self.ram.read(0xffdf) as u16;
        self.reg.pc = (pc_msb << 8) | pc_lsb;

        (0b0001_0000, 0b0001_0100)
    }

    fn clrp(&mut self) -> Flag {
        self.reg.psw.negate_page();
        (0x00, 0x00)
    }

    fn setp(&mut self) -> Flag {
        self.reg.psw.assert_page();
        (0x00, 0x00)
    }

    fn ei(&mut self) -> Flag {
        self.reg.psw.assert_interrupt();
        (0x00, 0x00)
    }

    fn di(&mut self) -> Flag {
        self.reg.psw.negate_interrupt();
        (0x00, 0x00)
    }
}