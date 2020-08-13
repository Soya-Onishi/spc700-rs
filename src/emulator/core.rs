extern crate spc;

use super::ram::*;
use super::instruction::Instruction;
use super::instruction::Opcode;
use super::register::*;
use super::execution::*;
use super::subject::Subject;
use crate::dsp::DSP;
use crate::emulator::timer::Timer;

use std::io::Result;
use std::path::Path;
use spc::spc::Spc;

trait BinOp<T> {
    fn binop(&mut self, inst: &Instruction, op: impl Fn(T, T) -> (T, Flag)) -> Flag;
}

impl BinOp<u8> for Spc700 {
    fn binop(&mut self, inst: &Instruction, op: impl Fn(u8, u8) -> (u8, Flag)) -> Flag {
        let (op1_sub, incl) = Subject::new(self, inst.op1, inst.raw_op,false);
        self.reg.inc_pc(incl);
        let (op0_sub, incl) = Subject::new(self, inst.op0, inst.raw_op, false);
        self.reg.inc_pc(incl);
        self.cycles(1);

        let op0 = op0_sub.read(self);
        let mut op1 = op1_sub.read(self);

        if inst.opcode == Opcode::AND1 || inst.opcode == Opcode::OR1 {
            if inst.raw_op & 0x20 > 0 {
               op1 = !op1;
            }
        }

        let (res, pwd) = op(op0 as u8, op1 as u8);

        // additional cycles in special case 
        match inst.opcode {
            Opcode::CMP => self.cycles(1),
            Opcode::TCLR1 => self.cycles(1),
            Opcode::TSET1 => self.cycles(1),
            _ => (),
        }

        if inst.opcode != Opcode::CMP {
            op0_sub.write(self, res as u16);
        } else {
            self.cycles(1);
        }
        
        pwd
    }
}

impl BinOp<u16> for Spc700 {
    fn binop(&mut self, inst: &Instruction, op: impl Fn(u16, u16) -> (u16, Flag)) -> Flag {
        let (op1_sub, incl) = Subject::new(self, inst.op1, inst.raw_op, true);
        self.reg.inc_pc(incl);
        let (op0_sub, incl) = Subject::new(self, inst.op0, inst.raw_op, true);
        self.reg.inc_pc(incl);
        self.cycles(1);

        let op0 = op0_sub.read(self);
        let op1 = op1_sub.read(self);

        let (res, psw) = op(op0, op1);
        op0_sub.write(self, res);

        psw
    }
}

trait UnaryOp<T> {
    fn unaryop(&mut self, inst: &Instruction, op: impl Fn(T) -> (u8, Flag)) -> Flag;
}

impl UnaryOp<u8> for Spc700 {
    fn unaryop(&mut self, inst: &Instruction, op: impl Fn(u8) -> (u8, Flag)) -> Flag {
        let (op0_sub, incl) = Subject::new(self, inst.op0, inst.raw_op, false);
        self.reg.inc_pc(incl);

        let op0 = op0_sub.read(self);

        // additional cycle in special case
        match inst.opcode {
            Opcode::XCN => self.cycles(4),
            _ => (),
        }

        let (res, psw) = op(op0 as u8);
        op0_sub.write(self, res as u16);

        psw
    }
}


impl UnaryOp<(u8, bool)> for Spc700 {
    fn unaryop(&mut self, inst: &Instruction, op: impl Fn((u8, bool)) -> (u8, Flag)) -> Flag {
        let (op0_sub, incl) = Subject::new(self, inst.op0, inst.raw_op, false);
        self.reg.inc_pc(incl);

        let op0 = op0_sub.read(self);

        let (res, psw) = op((op0 as u8, self.reg.psw.carry()));
        op0_sub.write(self, res as u16);

        psw
    }
}

trait StackManipulation<T> {
    fn push(&mut self, data: T);
    fn pop(&mut self) -> T;
}

impl StackManipulation<u8> for Spc700 {
    fn pop(&mut self) -> u8 {
        let addr = 0x0100 | (self.reg.sp.wrapping_add(1) as u16);
        let byte = self.read_ram(addr);

        self.reg.sp = self.reg.sp.wrapping_add(1);

        byte
    }

    fn push(&mut self, data: u8) {
        let addr = 0x0100 | (self.reg.sp as u16);
        self.write_ram(addr, data);

        self.reg.sp = self.reg.sp.wrapping_sub(1);
    }
}

impl StackManipulation<u16> for Spc700 {
    fn pop(&mut self) -> u16 {
        let addr_for_lsb = 0x0100 | (self.reg.sp.wrapping_add(1) as u16);
        let addr_for_msb = 0x0100 | (self.reg.sp.wrapping_add(2) as u16);
        let word_lsb = self.read_ram(addr_for_lsb) as u16;
        let word_msb = self.read_ram(addr_for_msb) as u16;

        self.reg.sp =self.reg.sp.wrapping_add(2);

        (word_msb << 8) | word_lsb
    }

    fn push(&mut self, data: u16) {
        for i in (0..2).rev() {
            let addr = 0x0100 | (self.reg.sp as u16);
            let byte = ((data >> (i * 8)) & 0xff) as u8;
            self.write_ram(addr, byte);
            self.reg.sp = self.reg.sp.wrapping_sub(1);
        }
    }
}

pub struct Spc700 {
    pub reg: Register,
    pub ram: Ram,
    pub dsp: DSP,
    pub timer: [Timer; 3],
    pub cycle_counter: u64,
}

impl Spc700 {
    pub fn new_with_init<P: AsRef<Path>>(path: P) -> Result<Spc700> {
        let spc = Spc::load(path)?;
        let ram = Ram::new_with_init(&spc.ram, &spc.ipl_rom);
        let dsp = DSP::new_with_init(&spc.regs);
        let mut timer = [8000, 8000, 64000].iter().map(|&hz| Timer::new(hz)).collect::<Vec<Timer>>();
        let control = spc.ram[0x00F1]; 
        timer.iter_mut().zip(0..3).for_each(|(timer, idx)| {
            let enable = (control & (1 << idx)) > 0;
            if enable {
                timer.enable();
            }
        });
        let register = Register::new_with_init(&spc);
        
        Ok(Spc700 {
            reg: register,
            ram: ram,
            dsp: dsp,
            timer: [timer[0], timer[1], timer[2]],            
            cycle_counter: 0,
        })
    }

    pub fn new(init_pc: u16) -> Spc700 {
        Spc700 {
            reg: Register::new(init_pc),
            ram: Ram::new(),
            dsp: DSP::new(),
            timer: [Timer::new(8000), Timer::new(8000), Timer::new(64000)],            
            cycle_counter: 0,
        }
    }

    pub fn next_sample(&mut self) -> (u16, u16) {
        let mut before_cycle = self.cycle_counter;

        while (self.cycle_counter - before_cycle) < 64 {
            self.clock();
        }

        (self.dsp.sample_left_out(), self.dsp.sample_right_out())
    }

    fn clock(&mut self) -> () {
        static mut ALL_CYCLE: u64 = 0;
        let before_cycle = self.cycle_counter;

        let pc = self.reg.inc_pc(1);
        let opcode = self.read_ram(pc);
        let mut inst = Instruction::decode(opcode);

        unsafe {
            // println!("[{:#08x}] opcode: {:#04x}, pc: {:#06x}, a: {:#04x}, x: {:#04x}, y: {:#04x}, sp: {:#04x}, psw: {:#010b}", ALL_CYCLE, opcode, pc, self.reg.a, self.reg.x, self.reg.y, self.reg.sp, self.reg.psw.get());            
            let timer = &self.timer[0];
            // println!("enable: {}, cycle: {:#06x}, out: {:#03x}, divided: {:#06x}", timer.enable, timer.cycle_counter, timer.out, timer.divided)
        }            
        
        let flag = match inst.opcode {
            Opcode::MOV => { self.exec_mov(&inst) }
            Opcode::MOVW => { self.exec_movw(&inst) }
            Opcode::PUSH => { self.exec_push(&inst) }
            Opcode::POP => { self.exec_pop(&inst) }
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
            Opcode::BPL => { self.branch(&mut inst) }
            Opcode::BMI => { self.branch(&mut inst) }
            Opcode::BVC => { self.branch(&mut inst) }
            Opcode::BVS => { self.branch(&mut inst) }
            Opcode::BCC => { self.branch(&mut inst) }
            Opcode::BCS => { self.branch(&mut inst) }
            Opcode::BNE => { self.branch(&mut inst) }
            Opcode::BEQ => { self.branch(&mut inst) }
            Opcode::BBS => { self.branch(&mut inst) }
            Opcode::BBC => { self.branch(&mut inst) }
            Opcode::CBNE => { self.branch(&mut inst) }
            Opcode::DBNZ => { self.branch(&mut inst) }
            Opcode::BRA => { self.relative_jump(&inst) }
            Opcode::JMP => { self.absolute_jump(&inst) }
            Opcode::CALL => { self.call(&inst) }
            Opcode::TCALL => { self.tcall(&inst) }
            Opcode::PCALL => { self.pcall(&inst) }
            Opcode::RET => { self.ret() }
            Opcode::RETI => { self.ret1() }
            Opcode::BRK => { self.brk() }
            Opcode::NOP => { self.cycles(1); (0x00, 0x00) /* nothing to do */ }
            Opcode::SLEEP => { panic!("SPC700 is suspended by SLEEP") }
            Opcode::STOP => { panic!("SPC700 is suspended by STOP") }
            Opcode::CLRP => { self.clrp() }
            Opcode::SETP => { self.setp() }
            Opcode::EI => { self.ei() }
            Opcode::DI => { self.di() }
        };

        self.renew_psw(flag); 
        self.dsp.flush(&mut self.ram);  // flush in force        
        assert_eq!(inst.cycles, (self.cycle_counter - before_cycle) as u16, "{:?}[{:#04x}], {:?}, {:?}", inst.opcode, inst.raw_op, inst.op0, inst.op1);
        unsafe {
            ALL_CYCLE += self.cycle_counter - before_cycle;
        }    
    }

    fn renew_psw(&mut self, (flag, mask): Flag) {
        // negate flags
        let psw = self.reg.psw.get();
        self.reg.psw.set(!(flag ^ mask) & psw);

        // assert flags
        let psw = self.reg.psw.get();
        self.reg.psw.set((flag & mask) | psw);
    }

    fn trans_into_decimal(&mut self, inst: &Instruction, op: impl Fn(u8, bool, bool) -> (u8, Flag)) -> Flag {
        let (op0_sub, inc) = Subject::new(self, inst.op0, inst.raw_op, false);
        let op0 = op0_sub.read(self);
        self.reg.inc_pc(inc);
               
        self.cycles(1);
        let (res, psw) = op(op0 as u8, self.reg.psw.half(), self.reg.psw.carry());
        op0_sub.write(self, res as u16);

        psw
    }

    fn calc_with_carry(&mut self, inst: &Instruction, op: impl Fn(u8, u8, bool) -> (u8, Flag)) -> Flag {
        let (op1_sub, incl) = Subject::new(self, inst.op1, inst.raw_op, false);
        self.reg.inc_pc(incl);
        let (op0_sub, incl) = Subject::new(self, inst.op0, inst.raw_op, false);
        self.reg.inc_pc(incl);

        let op0 = op0_sub.read(self) as u8;
        let op1 = op1_sub.read(self) as u8;

        let (res, pwd) = op(op0, op1, self.reg.psw.carry());

        if inst.opcode != Opcode::CMP {
            op0_sub.write(self, res as u16);
        }

        pwd
    }

    fn exec_mov(&mut self, inst: &Instruction) -> Flag {
        let (op1_sub, incl) = Subject::new(self, inst.op1, inst.raw_op, false);
        self.reg.inc_pc(incl);        
        let (op0_sub, incl) = Subject::new(self, inst.op0, inst.raw_op, false);
        self.reg.inc_pc(incl);      
        self.cycles(1)  

        let op1 = op1_sub.read(self) as u8;
        
        let (res, pwd) = eight_alu::mov(op1);

        let pwd = match op0_sub {
            Subject::Addr(addr, _) if (inst.raw_op != 0xFA) && (inst.raw_op != 0xAF) => {
                if (inst.raw_op != 0xFA) && (inst.raw_op != 0xAF) {
                    self.read_ram(addr);
                }

                (0x00, 0x00)
                
            }
            _ => { pwd }
        };

        
        op0_sub.write(self, res as u16);

        pwd
    }

    fn exec_movw(&mut self, inst: &Instruction) -> Flag {
        let (op1_sub, incl) = Subject::new(self, inst.op1, inst.raw_op, true);
        self.reg.inc_pc(incl);
        let (op0_sub, incl) = Subject::new(self, inst.op0, inst.raw_op, true);
        self.reg.inc_pc(incl);

        let op1 = op1_sub.read(self);
        
        let (res, pwd) = sixteen_alu::movw(op1);

        let pwd = match op0_sub {
            Subject::Addr(addr, _) => {
                self.read_ram(addr);
                (0x00, 0x00)
            }
            _ => { pwd }
        };
        
        op0_sub.write(self, res);

        pwd
    }
    
    fn exec_push(&mut self, inst: &Instruction) -> Flag {
        let (subject, inc) = Subject::new(self, inst.op0, inst.raw_op, false);
        let data = subject.read(self) as u8;
        self.reg.inc_pc(inc);

        self.push(data);

        (0x00, 0x00)
    }

    fn exec_pop(&mut self, inst: &Instruction) -> Flag {
        let (subject, inc) = Subject::new(self, inst.op0, inst.raw_op, false);
        let _ = subject.read(self);
        self.reg.inc_pc(inc);

        let data:u8 = self.pop();

        subject.write(self, data as u16);

        (0x00, 0x00)
    }

    fn branch(&mut self, inst: &mut Instruction) -> Flag {
        let (op0_sub, incl) = Subject::new(self, inst.op0, inst.raw_op, false); // either psw, [aa], [aa+X] or y
        self.reg.inc_pc(incl);
        let op0 = op0_sub.read(self);

        let (rr_sub, incl) =Subject::new(self, inst.op1, inst.raw_op, false);
        self.reg.inc_pc(incl);
        let rr = rr_sub.read(self);

        let (bias, is_branch) = match inst.opcode {
            Opcode::CBNE => {
                self.cycles(1); condjump::cbne(op0 as u8, self.reg.a, rr as u8)
            }
            Opcode::DBNZ => {
                self.cycles(1);

                let byte = op0.wrapping_sub(1);
                let (bias, is_branch) = condjump::dbnz(byte as u8, rr as u8);

                op0_sub.write(self, byte);

                (bias, is_branch)
            }
            Opcode::BBS => {
                self.cycles(1); condjump::branch(op0 as u8, rr as u8, true)
            }
            Opcode::BBC => {
                self.cycles(1); condjump::branch(op0 as u8, rr as u8, false)
            }
            _ => {
                condjump::branch(op0 as u8, rr as u8, (inst.raw_op & 0x20) > 0)
            }
        };

        self.reg.pc = self.reg.pc.wrapping_add(bias);

        if is_branch {
            self.cycles(2);
        }

        (0x00, 0x00)
    }

    fn relative_jump(&mut self, inst: &Instruction) -> Flag {
        let (rr_sub, inc) = Subject::new(self,inst.op0, inst.raw_op, false);
        let rr = rr_sub.read(self);
        self.reg.inc_pc(inc);

        let rr = if (rr & 0x80) > 0 {
            rr | 0xff00
        } else {
            rr
        };
            
        self.reg.pc = self.reg.pc.wrapping_add(rr);

        (0x00, 0x00)
    }

    fn absolute_jump(&mut self, inst: &Instruction) -> Flag {
        let (addr_sub, inc) = Subject::new(self, inst.op0, inst.raw_op, true);
        self.reg.inc_pc(inc);

        let dst = match inst.raw_op {
            0x5f => {
                match addr_sub {
                    Subject::Addr(addr, _ ) => { addr }
                    _ => { panic!("This code is unreachable.") }
                }
            }
            0x1f => {
                addr_sub.read(self)
            }
            _ => {
                panic!("This code is unreacheable")
            }
        };

        self.reg.pc = dst;

        (0x00, 0x00)
    }

    fn call(&mut self, inst: &Instruction) -> Flag {
        let (subject, inc) = Subject::new(self, inst.op0, inst.raw_op, false);
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

        let lsb =self.read_ram(0xffde - n) as u16;
        let msb = self.read_ram(0xffde - n + 1) as u16;
        
        self.cycles(3);
        let current_pc = self.reg.pc;
        self.push(current_pc);

        self.reg.pc = msb << 8 | lsb;

        (0x00, 0x00)
    }

    fn pcall(&mut self, inst: &Instruction) -> Flag {
        let (nn_sub, inc) = Subject::new(self, inst.op0, inst.raw_op, false);
        self.reg.inc_pc(inc);
        let nn = nn_sub.read(self);
        let dst = 0xff00 | nn;

        self.push(self.reg.pc);
        self.reg.pc = dst;

        (0x00, 0x00)
    }

    fn ret(&mut self) -> Flag {
        let dst: u16 = self.pop();
        self.reg.pc = dst;

        (0x00, 0x00)
    }

    fn ret1(&mut self) -> Flag {
        let psw: u8 = self.pop();
        let pc: u16 = self.pop();

        self.reg.pc = pc;
        self.reg.psw.set(psw);

        (0x00, 0x00)
    }

    fn brk(&mut self) -> Flag {
        let pc_lsb = self.read_ram(0xffde) as u16;
        let pc_msb = self.read_ram(0xffdf) as u16;
        let next_pc = (pc_msb << 8) | pc_lsb;

        self.cycles(2);
        self.push(self.reg.pc);
        self.push(self.reg.psw.get());        

        self.reg.pc = next_pc;
        self.reg.psw.assert_brk();
        self.reg.psw.negate_interrupt();

        (0x00, 0x00)
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

    pub fn read_ram(&mut self, addr: u16) -> u8 {
        self.cycles(1);
        self.read_ram_without_cycle(addr)
    }

    pub fn read_ram_without_cycle(&mut self, addr: u16) -> u8 {
        let data = self.ram.read(addr, &mut self.dsp, &mut self.timer);
        // println!("[ read ram] addr: {:#06x}, data: {:#04x}", addr, data);
        data
    }

    pub fn write_ram(&mut self, addr: u16, data: u8) -> () {
        self.cycles(1);
        self.ram.write(addr, data, &mut self.dsp, &mut self.timer);
        // println!("[write ram] addr: {:#06x}, data: {:#04x}", addr, data);
    }

    pub fn cycles(&mut self, cycle_count: u16) -> () {
        self.dsp.cycles(cycle_count);
        self.timer.iter_mut().for_each(|timer| timer.cycles(cycle_count));
        self.cycle_counter += cycle_count as u64;
    }
}
