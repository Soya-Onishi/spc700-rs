extern crate spc;

use super::ram::*;
use super::register::*;
use crate::dsp::DSP;
use crate::emulator::timer::Timer;

use std::io::Result;
use std::path::Path;
use spc::spc::Spc;

use typenum::marker_traits::Unsigned;

pub struct Spc700 {
    pub reg: Register,
    pub ram: Ram,
    pub dsp: DSP,
    pub timer: [Timer; 3],
    pub cycle_counter: u64,
    total_cycles: u64,
    is_stopped: bool
}


struct OperationResult<T> {
    cycles: usize,
    ret: T
}

impl<T> OperationResult<T> {
    fn new(ret: T, cycles: usize) -> OperationResult<T> {
        OperationResult { cycles, ret }
    }

    fn cycles(&self) -> usize {
        self.cycles
    }
}

impl OperationResult<()> {
    fn new_unit(cycles: usize) -> OperationResult<()> {
        OperationResult::new((), cycles)
    }
}

const DECODE_TABLE: [fn(&mut Spc700, u8) -> OperationResult<()>; 256] = [
    // upper opcode: 0x0
    // lower opcode: 0x0
    Spc700::nop,
    Spc700::tcall,
    Spc700::set1,
    Spc700::branch_by_mem_bit,
    Spc700::alu_dp,
    Spc700::alu_addr,
    Spc700::alu_indirect_x,
    Spc700::alu_x_ind_ind,
    // upper opcode: 0x0
    // lower opcode: 0x8
    Spc700::alu_imm,
    Spc700::alu_dp_dp,
    Spc700::or1,
    Spc700::shift_dp,
    Spc700::shift_addr,
    Spc700::push,
    Spc700::tset1,
    Spc700::brk,
    // upper opcode: 0x1
    // lower opcode: 0x0
    Spc700::branch_by_psw,
    Spc700::tcall,
    Spc700::clr1,
    Spc700::branch_by_mem_bit,
    Spc700::alu_x_idx_indirect,
    Spc700::alu_x_idx_addr,
    Spc700::alu_y_idx_addr,
    Spc700::alu_y_ind_ind,
    // upper opcode: 0x1
    // lower opcode: 0x8
    Spc700::alu_dp_imm,
    Spc700::alu_x_y,
    Spc700::inc_dec_word,
    Spc700::shift_x_idx_indirect,
    Spc700::shift_acc,
    Spc700::inc_dec_reg,
    Spc700::alu_addr,
    Spc700::jmp_abs_x,
    // upper opcode: 0x2
    // lower opcode: 0x0
    Spc700::clrp,
    Spc700::tcall,
    Spc700::set1,
    Spc700::branch_by_mem_bit,
    Spc700::alu_dp,
    Spc700::alu_addr,
    Spc700::alu_indirect_x,
    Spc700::alu_x_ind_ind,
    // upper opcode: 0x2
    // lower opcode: 0x8
    Spc700::alu_imm,
    Spc700::alu_dp_dp,
    Spc700::or1,
    Spc700::shift_dp,
    Spc700::shift_addr,
    Spc700::push,
    Spc700::cbne,
    Spc700::bra,
    // upper opcode: 0x3
    // lower opcode: 0x0
    Spc700::branch_by_psw,
    Spc700::tcall,
    Spc700::clr1,
    Spc700::branch_by_mem_bit,
    Spc700::alu_x_idx_indirect,
    Spc700::alu_x_idx_addr,
    Spc700::alu_y_idx_addr,
    Spc700::alu_y_ind_ind,
    // upper opcode: 0x3
    // lower opcode: 0x8
    Spc700::alu_dp_imm,
    Spc700::alu_x_y,
    Spc700::inc_dec_word,
    Spc700::shift_x_idx_indirect,
    Spc700::shift_acc,
    Spc700::inc_dec_reg,
    Spc700::alu_dp,
    Spc700::call,
    // upper opcode: 0x4
    // lower opcode: 0x0,
    Spc700::setp,
    Spc700::tcall,
    Spc700::set1,
    Spc700::branch_by_mem_bit,
    Spc700::alu_dp,
    Spc700::alu_addr,
    Spc700::alu_indirect_x,
    Spc700::alu_x_ind_ind,
    // upper opcode: 0x4
    // lower opcode: 0x8
    Spc700::alu_imm,
    Spc700::alu_dp_dp,
    Spc700::and1,
    Spc700::shift_dp,
    Spc700::shift_addr,
    Spc700::push,
    Spc700::tclr1,
    Spc700::pcall,
    // upper opcode: 0x5
    // lower opcode: 0x0
    Spc700::branch_by_psw,
    Spc700::tcall,
    Spc700::clr1,
    Spc700::branch_by_mem_bit,
    Spc700::alu_x_idx_indirect,
    Spc700::alu_x_idx_addr,
    Spc700::alu_y_idx_addr,
    Spc700::alu_y_ind_ind,
    // upper opcode: 0x5
    // lower opcode: 0x8
    Spc700::alu_dp_imm,
    Spc700::alu_x_y,
    Spc700::cmpw,
    Spc700::shift_x_idx_indirect,
    Spc700::shift_acc,
    Spc700::mov_reg_reg,
    Spc700::alu_addr,
    Spc700::jmp_abs,
    // upper opcode: 0x6
    // lower opcode: 0x0
    Spc700::clrc,
    Spc700::tcall,
    Spc700::set1,
    Spc700::branch_by_mem_bit,
    Spc700::alu_dp,
    Spc700::alu_addr,
    Spc700::alu_indirect_x,
    Spc700::alu_x_ind_ind,
    // upper opcode: 0x6
    // lower opcode: 0x8
    Spc700::alu_imm,
    Spc700::alu_dp_dp,
    Spc700::and1,
    Spc700::shift_dp,
    Spc700::shift_addr,
    Spc700::push,
    Spc700::dbnz_data,
    Spc700::ret,
    // upper opcode: 0x7
    // lower opcode: 0x0
    Spc700::branch_by_psw,
    Spc700::tcall,
    Spc700::clr1,
    Spc700::branch_by_mem_bit,
    Spc700::alu_x_idx_indirect,
    Spc700::alu_x_idx_addr,
    Spc700::alu_y_idx_addr,
    Spc700::alu_y_ind_ind,
    // upper opcode: 0x7
    // lower opcode: 0x8 
    Spc700::alu_dp_imm,
    Spc700::alu_x_y,
    Spc700::addw,
    Spc700::shift_x_idx_indirect,
    Spc700::shift_acc,
    Spc700::mov_reg_reg,
    Spc700::alu_dp,
    Spc700::ret1,
    // upper opcode: 0x8
    // lower opcode: 0x0
    Spc700::setc,
    Spc700::tcall,
    Spc700::set1,
    Spc700::branch_by_mem_bit,
    Spc700::alu_dp,
    Spc700::alu_addr,
    Spc700::alu_indirect_x,
    Spc700::alu_x_ind_ind,
    // upper opcode: 0x8
    // lower opcode: 0x8
    Spc700::alu_imm,
    Spc700::alu_dp_dp,
    Spc700::eor1,
    Spc700::inc_dec_dp,
    Spc700::inc_dec_addr,
    Spc700::mov_reg_imm,
    Spc700::pop,
    Spc700::mov_store_dp_imm,
    // upper opcode: 0x9
    // lower opcode: 0x0
    Spc700::branch_by_psw,
    Spc700::tcall,
    Spc700::clr1,
    Spc700::branch_by_mem_bit,
    Spc700::alu_x_idx_indirect,
    Spc700::alu_x_idx_addr,
    Spc700::alu_y_idx_addr,
    Spc700::alu_y_ind_ind,
    // upper opcode: 0x9
    // lower opcode: 0x8
    Spc700::alu_dp_imm,
    Spc700::alu_x_y,
    Spc700::subw,
    Spc700::inc_dec_x_idx_indirect,
    Spc700::inc_dec_reg,
    Spc700::mov_reg_reg,
    Spc700::div,
    Spc700::xcn,
    // upper opcode: 0xA
    // lower opcode: 0x0
    Spc700::ei,
    Spc700::tcall,
    Spc700::set1,
    Spc700::branch_by_mem_bit,
    Spc700::alu_dp,
    Spc700::alu_addr,
    Spc700::alu_indirect_x,
    Spc700::alu_x_ind_ind,
    // upper opcode: 0xA
    // lower opcode: 0x8
    Spc700::alu_imm,
    Spc700::alu_dp_dp,
    Spc700::mov1_to_psw,
    Spc700::inc_dec_dp,
    Spc700::inc_dec_addr,
    Spc700::alu_imm,
    Spc700::pop,
    Spc700::mov_store_x_indirect_inc,
    // upper opcode: 0xB
    // lower opcode: 0x0
    Spc700::branch_by_psw,
    Spc700::tcall,
    Spc700::clr1,
    Spc700::branch_by_mem_bit,
    Spc700::alu_x_idx_indirect,
    Spc700::alu_x_idx_addr,
    Spc700::alu_y_idx_addr,
    Spc700::alu_y_ind_ind,
    // upper opcode: 0xB
    // lower opcode: 0x8
    Spc700::alu_dp_imm,
    Spc700::alu_x_y,
    Spc700::mov_load_word,
    Spc700::inc_dec_x_idx_indirect,
    Spc700::inc_dec_reg,
    Spc700::mov_reg_reg,
    Spc700::das,
    Spc700::mov_load_x_indirect_inc,
    // upper opcode: 0xC
    // lower opcode: 0x0
    Spc700::di,
    Spc700::tcall,
    Spc700::set1,
    Spc700::branch_by_mem_bit,
    Spc700::mov_store_dp_reg,
    Spc700::mov_store_addr,
    Spc700::mov_store_x_indirect,
    Spc700::mov_store_x_ind_ind,
    // upper opcode: 0xC
    // lower opcode: 0x8
    Spc700::alu_imm,
    Spc700::mov_store_addr,
    Spc700::mov1_to_mem,
    Spc700::mov_store_dp_reg,
    Spc700::mov_store_addr,
    Spc700::mov_reg_imm,
    Spc700::pop,
    Spc700::mul,
    // upper opcode: 0xD
    // lower opcode: 0x0
    Spc700::branch_by_psw,
    Spc700::tcall,
    Spc700::clr1,
    Spc700::branch_by_mem_bit,
    Spc700::mov_store_x_idx_indirect,
    Spc700::mov_store_x_idx_addr,
    Spc700::mov_store_y_idx_addr,
    Spc700::mov_store_y_ind_ind,
    // upper opcode: 0xD
    // lower opcode: 0x8
    Spc700::mov_store_dp_reg,
    Spc700::mov_store_y_idx_indirect,
    Spc700::mov_store_word,
    Spc700::mov_store_x_idx_indirect,
    Spc700::inc_dec_reg,
    Spc700::mov_reg_reg,
    Spc700::cbne,
    Spc700::daa,
    // upper opcode: 0xE
    // lower opcode: 0x0
    Spc700::clrv,
    Spc700::tcall,
    Spc700::set1,
    Spc700::branch_by_mem_bit,
    Spc700::mov_load_dp::<typenum::U228>, // 0xE4 = 228
    Spc700::mov_load_addr,
    Spc700::mov_load_x_indirect,
    Spc700::mov_load_x_ind_ind,
    // upper opcode: 0xE
    // lower opcode: 0x8
    Spc700::mov_reg_imm,
    Spc700::mov_load_addr,
    Spc700::not1,
    Spc700::mov_load_dp::<typenum::U235>, // 0xEB = 235
    Spc700::mov_load_addr,
    Spc700::notc,
    Spc700::pop,
    Spc700::sleep_or_stop,
    // upper opcode: 0xF
    // lower opcode: 0x0
    Spc700::branch_by_psw,
    Spc700::tcall,
    Spc700::clr1,
    Spc700::branch_by_mem_bit,
    Spc700::mov_load_x_idx_indirect,
    Spc700::mov_load_x_idx_addr,
    Spc700::mov_load_y_idx_addr,
    Spc700::mov_load_y_ind_ind,
    // upper opcode: 0xF
    // lower opcode: 0x8
    Spc700::mov_load_dp::<typenum::U248>, // 0xF8 = 248
    Spc700::mov_load_y_idx_indirect,
    Spc700::mov_store_dp_dp,
    Spc700::mov_load_x_idx_indirect,
    Spc700::inc_dec_reg,
    Spc700::mov_reg_reg,
    Spc700::dbnz_y,
    Spc700::sleep_or_stop,
];

impl Spc700 {
    pub fn new_with_init<P: AsRef<Path>>(path: P) -> Result<Spc700> {
        let spc = Spc::load(path)?;
        let ram = Ram::new_with_init(&spc.ram, &spc.ipl_rom);
        let dsp = DSP::new_with_init(&spc.regs);

        let divider0 = spc.ram[0x00FA];
        let divider1 = spc.ram[0x00FB];
        let divider2 = spc.ram[0x00FC];        

        let mut timer: Vec<Timer> = [8000, 8000, 64000].iter()
            .zip([divider0, divider1, divider2].iter())
            .map(|(&hz, &divider)| Timer::new_with_init(hz, divider, 0))
            .collect();

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
            total_cycles: 0,
            is_stopped: false,
        })
    }

    pub fn new(init_pc: u16) -> Spc700 {
        Spc700 {
            reg: Register::new(init_pc),
            ram: Ram::new(),
            dsp: DSP::new(),
            timer: [Timer::new(8000), Timer::new(8000), Timer::new(64000)],            
            cycle_counter: 0,
            total_cycles: 0,
            is_stopped: false,
        }
    }

    pub fn next_sample(&mut self) -> (i16, i16) {        
        loop {
            let before_cycle_count = self.dsp.sync_counter;
            self.clock();
            let after_cycle_count = self.dsp.sync_counter;

            if before_cycle_count > after_cycle_count {
                break;
            }
        }        

        (self.dsp.sample_left_out(), self.dsp.sample_right_out())
    }
    
    fn clock(&mut self) -> () {
        if self.is_stopped {
            self.count_cycles(2);
            self.dsp.flush(&mut self.ram);
            return;
        }

        let pc = self.reg.inc_pc(1);
        let OperationResult { ret: opcode, cycles: fetch_cycles } = self.read_ram(pc);                
        let instruction = DECODE_TABLE[opcode as usize];
        let OperationResult { ret: _, cycles: op_cycles } = instruction(self, opcode);
        
        let cycles = fetch_cycles + op_cycles;
        
        log::debug!("op: {:04x}, {}", opcode, &self.reg);

        self.count_cycles(cycles as u16);
        self.dsp.flush(&mut self.ram);
    }

    fn mov_reg_imm(&mut self, opcode: u8) -> OperationResult<()> {
        let reg_type = match opcode {
            0x8D => 2,
            0xCD => 1, 
            0xE8 => 0,
            _ => panic!("unexpected opcode: {}", opcode),
        };

        let OperationResult{ ret: imm, cycles: read_cycles } = self.read_from_pc();
        match reg_type {
            0 => self.reg.a = imm,
            1 => self.reg.x = imm,
            2 => self.reg.y = imm,
            _ => panic!("register type must be between 0 to 2"),
        }

        self.set_mov_flag(imm);

        OperationResult::new((), read_cycles)
    }

    fn mov_reg_reg(&mut self, opcode: u8) -> OperationResult<()> { 
        let upper = (opcode >> 4) & 0x0F;
        let lower = opcode & 0x0F;
        
        let from = match (upper, lower) {
            (0x5, 0xD) => 0,
            (0x7, 0xD) => 1,
            (0x9, 0xD) => 3,
            (0xB, 0xD) => 1, 
            (0xD, 0xD) => 2,
            (0xF, 0xD) => 0,
            _ => panic!("unexpected opcode: {}", opcode),
        };

        let to = match (upper, lower) {
            (0x5, 0xD) => 1,
            (0x7, 0xD) => 0,
            (0x9, 0xD) => 1,
            (0xB, 0xD) => 3,
            (0xD, 0xD) => 0,
            (0xF, 0xD) => 2,
            _ => panic!("unexpected opcode: {}", opcode),
        };

        let data = match from {
            0 => self.reg.a,
            1 => self.reg.x,
            2 => self.reg.y,
            3 => self.reg.sp,
            _ => panic!("register type must be between 0 to 3"),
        };

        match to {
            0 => self.reg.a = data,
            1 => self.reg.x = data,
            2 => self.reg.y = data,
            3 => self.reg.sp = data,
            _ => panic!("register type must be between 0 to 3"),
        };

        self.set_mov_flag(data);

        OperationResult::new((), 1)
    }

    fn mov_load_dp<Opcode: Unsigned>(&mut self, opcode: u8) -> OperationResult<()> {
        let OperationResult { ret: addr, cycles: read_pc_cycles } = self.read_from_pc();
        let OperationResult { ret: data, cycles: read_cycles } = self.read_from_page(addr);

        match Opcode::to_u8() {
            0xE4 => self.reg.a = data,
            0xF8 => self.reg.x = data,
            0xEB => self.reg.y = data,
            _ => panic!("unexpected opcode: {}", opcode),
        }

        self.set_mov_flag(data);

        OperationResult::new(
            (),
            read_pc_cycles + read_cycles
        )
    }

    fn mov_load_x_idx_indirect(&mut self, opcode: u8) -> OperationResult<()> {
        let reg_type = match opcode {
            0xF4 => 0,
            0xFB => 2,
            _ => panic!("unexpected opcode: {}", opcode),
        };

        let OperationResult { ret: addr, cycles: read_cycles } = self.read_from_pc();
        let addr = addr.wrapping_add(self.reg.x);
        let OperationResult { ret: data, cycles: read_page_cycles } = self.read_from_page(addr);

        match reg_type {
            0 => self.reg.a = data,
            2 => self.reg.y = data,
            _ => panic!("register type must be between 0 to 2"),
        }

        self.set_mov_flag(data);

        OperationResult::new((), read_cycles + read_page_cycles + 1)
    }

    fn mov_load_y_idx_indirect(&mut self, _opcode: u8) -> OperationResult<()> {
        let OperationResult { ret: addr, cycles: read_cycles } = self.read_from_pc();
        let addr = addr.wrapping_add(self.reg.y);
        let OperationResult{ ret: data, cycles: read_page_cycles } = self.read_from_page(addr);
        self.reg.x = data;

        self.set_mov_flag(data);

        OperationResult::new((), read_cycles + read_page_cycles + 1)
    }

    fn mov_load_addr(&mut self, opcode: u8) -> OperationResult<()> {
        let reg_type = match opcode {
            0xE5 => 0,
            0xE9 => 1,
            0xEC => 2,
            _ => panic!("unexpected opcode: {}", opcode),
        };

        let OperationResult{ ret: lower, cycles: read_lower_cycles } = self.read_from_pc();
        let OperationResult{ ret: upper, cycles: read_upper_cycles } = self.read_from_pc();
        let lower = lower as u16;
        let upper = upper as u16; 
        let addr = (upper << 8) | lower;
        let OperationResult{ ret: data, cycles: read_cycles } = self.read_ram(addr);

        match reg_type {
            0 => self.reg.a = data,
            1 => self.reg.x = data,
            2 => self.reg.y = data,
            _ => panic!("register type must be between 0 to 2"),
        }

        self.set_mov_flag(data);

        OperationResult::new(
            (),
            read_lower_cycles + read_upper_cycles + read_cycles
        )
    }

    fn mov_load_x_idx_addr(&mut self, _opcode: u8) -> OperationResult<()> {
        let OperationResult{ ret: lower, cycles: lower_cycles } = self.read_from_pc();
        let OperationResult{ ret: upper, cycles: upper_cycles } = self.read_from_pc();
        let lower = lower as u16;
        let upper = upper as u16;
        let addr = (upper << 8) | lower;    
        let addr = addr.wrapping_add(self.reg.x as u16);

        let OperationResult{ ret: data, cycles: read_cycles } = self.read_ram(addr);

        self.reg.a = data;
        self.set_mov_flag(data);

        OperationResult::new(
            (),
            lower_cycles + upper_cycles + read_cycles + 1
        )
    }

    fn mov_load_y_idx_addr(&mut self, _opcode: u8) -> OperationResult<()> {
        let OperationResult{ ret: lower, cycles: lower_cycles } = self.read_from_pc();
        let OperationResult{ ret: upper, cycles: upper_cycles } = self.read_from_pc();
        let lower = lower as u16;
        let upper = upper as u16;
        let addr = (upper << 8) | lower;    
        let addr = addr.wrapping_add(self.reg.y as u16);

        let OperationResult{ ret: data, cycles: read_cycles } = self.read_ram(addr);

        self.reg.a = data;
        self.set_mov_flag(data);

        OperationResult::new(
            (),
            lower_cycles + upper_cycles + read_cycles + 1
        )
    }

    fn mov_load_x_indirect(&mut self, _opcode: u8) -> OperationResult<()> {
        let OperationResult{ ret: data, cycles: read_cycles } = self.read_from_page(self.reg.x);

        self.reg.a = data;
        self.set_mov_flag(data);

        OperationResult::new_unit(read_cycles + 1)
    }

    fn mov_load_x_indirect_inc(&mut self, _opcode: u8) -> OperationResult<()> {
        let OperationResult{ ret: data, cycles: read_cycles } = self.read_from_page(self.reg.x);
        self.reg.x = self.reg.x.wrapping_add(1);

        self.reg.a = data;
        self.set_mov_flag(data);

        OperationResult::new_unit(read_cycles + 2)
    }

    fn mov_load_y_ind_ind(&mut self, _opcode: u8) -> OperationResult<()> {
        let OperationResult{ ret: base_addr, cycles: read_cycles } = self.read_from_pc();
        let OperationResult{ ret: lower, cycles: lower_cycles } = self.read_from_page(base_addr);
        let OperationResult{ ret: upper, cycles: upper_cycles } = self.read_from_page(base_addr.wrapping_add(1));        
        let lower = lower as u16;
        let upper = upper as u16;
        let addr = (upper << 8) | lower;
        let addr = addr.wrapping_add(self.reg.y as u16);
        let OperationResult{ ret: data, cycles: data_cycles } = self.read_ram(addr);

        self.reg.a = data;
        self.set_mov_flag(data);

        OperationResult::new_unit(read_cycles + lower_cycles + upper_cycles + data_cycles + 1)
    }

    fn mov_load_x_ind_ind(&mut self, _opcode: u8) -> OperationResult<()> {
        let OperationResult{ ret: base_addr, cycles: base_addr_cycles } = self.read_from_pc();
        let base_addr = base_addr.wrapping_add(self.reg.x);
        let OperationResult{ ret: lower, cycles: lower_cycles } = self.read_from_page(base_addr);
        let OperationResult{ ret: upper, cycles: upper_cycles } = self.read_from_page(base_addr.wrapping_add(1));
        let lower = lower as u16;
        let upper = upper as u16;
        let addr = (upper << 8) | lower;
        let OperationResult { ret: data, cycles: read_cycles } = self.read_ram(addr);

        self.reg.a = data;
        self.set_mov_flag(data);

        OperationResult::new_unit(base_addr_cycles + lower_cycles + upper_cycles + read_cycles + 1)
    }

    fn mov_load_word(&mut self, _opcode: u8) -> OperationResult<()> {
        let OperationResult{ ret: addr, cycles: read_cycles } = self.read_from_pc();
        let OperationResult{ ret: lower, cycles: lower_cycles } = self.read_from_page(addr);
        let OperationResult{ ret: upper, cycles: upper_cycles } = self.read_from_page(addr.wrapping_add(1));
        let lower = lower as u16;
        let upper = upper as u16;
        let word = (upper << 8) | lower;

        self.reg.set_ya(word);

        let is_negative = (word & 0x8000) != 0;
        let is_zero = word == 0;
        self.reg.psw.set_sign(is_negative);
        self.reg.psw.set_zero(is_zero);

        OperationResult::new_unit(read_cycles + lower_cycles + upper_cycles + 1)
    }

    fn set_mov_flag(&mut self, data: u8) -> () {
        let is_negative = (data & 0x80) != 0;
        let is_zero = data == 0;
        self.reg.psw.set_zero(is_zero);
        self.reg.psw.set_sign(is_negative); 
    }

    fn mov_store_dp_imm(&mut self, _opcode: u8) -> OperationResult<()> {
        let OperationResult{ ret: imm, cycles: imm_cycles } = self.read_from_pc();
        let OperationResult{ ret: addr, cycles: addr_cycles } = self.read_from_pc();
        let OperationResult{ ret: _, cycles: read_cycles } = self.read_from_page(addr);        

        let OperationResult{ ret: _, cycles: write_cycles } = self.write_to_page(addr, imm);

        OperationResult::new_unit(imm_cycles + addr_cycles + read_cycles + write_cycles)
    }

    fn mov_store_dp_dp(&mut self, _opcode: u8) -> OperationResult<()> {
        let OperationResult{ ret: bb, cycles: bb_cycles } = self.read_from_pc();
        let OperationResult{ ret: b, cycles: b_cycles } = self.read_from_page(bb);
        let OperationResult{ ret: aa, cycles: aa_cycles } = self.read_from_pc();        
        
        let OperationResult { ret: _, cycles: write_cycles } = self.write_to_page(aa, b);

        OperationResult::new_unit(bb_cycles + b_cycles + aa_cycles + write_cycles)
    }

    fn mov_store_dp_reg(&mut self, opcode: u8) -> OperationResult<()> {
        let reg_type = match opcode {
            0xC4 => 0,
            0xCB => 2,
            0xD8 => 1,
            _ => panic!("unexpected opcode: {}", opcode),
        };

        let OperationResult{ ret: addr, cycles: addr_cycles } = self.read_from_pc();
        let OperationResult{ ret: _, cycles: read_cycles } = self.read_from_page(addr);
        let data = match reg_type {
            0 => self.reg.a,
            1 => self.reg.x,
            2 => self.reg.y,
            _ => panic!("register type must be between 0 to 2"),
        };

        let OperationResult{ ret: _, cycles: write_cycles } = self.write_to_page(addr, data);

        OperationResult::new_unit(addr_cycles + read_cycles + write_cycles)
    }

    fn mov_store_x_idx_indirect(&mut self, opcode: u8) -> OperationResult<()> {
        let reg_type = match opcode {
            0xD4 => 0,
            0xDB => 2,
            _ => panic!("unexpected opcode: {}", opcode),
        };

        let OperationResult{ ret: addr, cycles: read_cycles } = self.read_from_pc();
        let addr = addr.wrapping_add(self.reg.x);
        let OperationResult{ ret: _, cycles: page_cycles }= self.read_from_page(addr);
        let data = match reg_type {
            0 => self.reg.a,
            2 => self.reg.y,    
            _ => panic!("register type must be 0 or 2"),
        };
        
        let OperationResult{ ret: _, cycles: write_cycles } = self.write_to_page(addr, data);

        OperationResult::new_unit(read_cycles + page_cycles + write_cycles + 1)
    }

    fn mov_store_y_idx_indirect(&mut self, _opcode: u8) -> OperationResult<()> {
        let OperationResult{ ret: addr, cycles: read_cycles } = self.read_from_pc();
        let addr = addr.wrapping_add(self.reg.y);
        let OperationResult{ ret: _, cycles: read_page_cycles } = self.read_from_page(addr);
        let data = self.reg.x;
        
        let OperationResult{ ret: _, cycles: write_cycles } = self.write_to_page(addr, data);

        OperationResult::new_unit(read_cycles + read_page_cycles + write_cycles + 1)
    }

    fn mov_store_addr(&mut self, opcode: u8) -> OperationResult<()> {
        let reg_type = match opcode {
            0xC5 => 0,
            0xC9 => 1,
            0xCC => 2,
            _ => panic!("unexpected opcode: {}", opcode),
        };

        let OperationResult{ ret: lower, cycles: lower_cycles } = self.read_from_pc();
        let OperationResult{ ret: upper, cycles: upper_cycles } = self.read_from_pc();
        let lower = lower as u16;
        let upper = upper as u16;
        let addr = (upper << 8) | lower;
        let OperationResult{ ret: _, cycles: read_cycles } = self.read_ram(addr);

        let data = match reg_type { 
            0 => self.reg.a,
            1 => self.reg.x,
            2 => self.reg.y,    
            _ => panic!("register type must be between 0 to 2"),
        };

        let OperationResult{ ret: _, cycles: write_cycles } = self.write_ram(addr, data);

        OperationResult::new_unit(lower_cycles + upper_cycles + read_cycles + write_cycles)
    }

    fn mov_store_x_idx_addr(&mut self, _opcode: u8) -> OperationResult<()> {
        let OperationResult{ ret: lower, cycles: lower_cycles } = self.read_from_pc();
        let OperationResult{ ret: upper, cycles: upper_cycles } = self.read_from_pc();
        let lower = lower as u16;
        let upper = upper as u16;
        let addr = (upper << 8) | lower;
        let addr = addr.wrapping_add(self.reg.x as u16);
        let read_cycles = self.read_ram(addr).cycles();
        
        let write_cycles = self.write_ram(addr, self.reg.a).cycles();

        OperationResult::new_unit(lower_cycles + upper_cycles + read_cycles + write_cycles + 1)
    }

    fn mov_store_y_idx_addr(&mut self, _opcode: u8) -> OperationResult<()> {
        let OperationResult{ ret: lower, cycles: lower_cycles } = self.read_from_pc();
        let OperationResult{ ret: upper, cycles: upper_cycles } = self.read_from_pc();
        let lower = lower as u16;
        let upper = upper as u16;
        let addr = (upper << 8) | lower;
        let addr = addr.wrapping_add(self.reg.y as u16);
        let read_cycles = self.read_ram(addr).cycles();

        let write_cycles = self.write_ram(addr, self.reg.a).cycles();

        OperationResult::new_unit(lower_cycles + upper_cycles + read_cycles + write_cycles + 1)
    }

    fn mov_store_x_indirect_inc(&mut self, _opcode: u8) -> OperationResult<()> {
        let write_cycles = self.write_to_page(self.reg.x, self.reg.a).cycles();
        self.reg.x = self.reg.x.wrapping_add(1);

        OperationResult::new_unit(write_cycles + 2)
    }

    fn mov_store_x_indirect(&mut self, _opcode: u8) -> OperationResult<()> {
        let read_cycles = self.read_from_page(self.reg.x).cycles();
        let write_cycles = self.write_to_page(self.reg.x, self.reg.a).cycles();

        OperationResult::new_unit(read_cycles + write_cycles + 1)
    }

    fn mov_store_x_ind_ind(&mut self, _opcode: u8) -> OperationResult<()> {
        let OperationResult{ ret: base_addr, cycles: base_addr_cycles } = self.read_from_pc();
        let base_addr = base_addr.wrapping_add(self.reg.x);
        let OperationResult{ ret: lower, cycles: lower_cycles } = self.read_from_page(base_addr);
        let OperationResult{ ret: upper, cycles: upper_cycles } = self.read_from_page(base_addr.wrapping_add(1));
        let lower = lower as u16;
        let upper = upper as u16;
        let addr = (upper << 8) | lower;
        let read_cycles = self.read_ram(addr).cycles();

        let write_cycles = self.write_ram(addr, self.reg.a).cycles();

        let cycles = read_cycles + base_addr_cycles + lower_cycles + upper_cycles + read_cycles + write_cycles + 1;
        OperationResult::new_unit(cycles)
    }

    fn mov_store_y_ind_ind(&mut self, _opcode: u8) -> OperationResult<()> {
        let OperationResult{ ret: base_addr, cycles: read_pc_cycles } = self.read_from_pc();
        let OperationResult{ ret: lower, cycles: lower_cycles } = self.read_from_page(base_addr);
        let OperationResult{ ret: upper, cycles: upper_cycles } = self.read_from_page(base_addr);
        let lower = lower as u16;
        let upper = upper as u16;
        let addr = (upper << 8) | lower;        
        let addr = addr.wrapping_add(self.reg.y as u16);
        let read_cycles = self.read_ram(addr).cycles();
        
        let write_cycles = self.write_ram(addr, self.reg.a).cycles();

        OperationResult::new_unit(read_pc_cycles + lower_cycles + upper_cycles + read_cycles + write_cycles + 1)
    }

    fn mov_store_word(&mut self, _opcode: u8) -> OperationResult<()> {
        let OperationResult{ ret: addr, cycles: read_cycles } = self.read_from_pc();
        let read_page_cycles = self.read_from_page(addr).cycles();
        let write_cycles_a = self.write_to_page(addr, self.reg.a).cycles();
        let write_cycles_y = self.write_to_page(addr.wrapping_add(1), self.reg.y).cycles();

        OperationResult::new_unit(read_cycles + read_page_cycles + write_cycles_a + write_cycles_y)
    }

    fn push(&mut self, opcode: u8) -> OperationResult<()> {
        let reg_type = match opcode {
            0x0D => 3,
            0x2D => 0,
            0x4D => 1,
            0x6D => 2,
            _ => panic!("unexpected opcode: {}", opcode),
        };
        
        let data = match reg_type {
            0 => self.reg.a,
            1 => self.reg.x,
            2 => self.reg.y,
            3 => self.reg.psw.get(),
            _ => panic!("register type must be between 0 to 3"),
        };

        let write_cycles = self.write_to_stack(data).cycles();
        self.reg.sp = self.reg.sp.wrapping_sub(1);

        OperationResult::new_unit(write_cycles + 2)
    }

    fn pop(&mut self, opcode: u8) -> OperationResult<()> {
        let reg_type = match opcode { 
            0x8E => 3,
            0xAE => 0,
            0xCE => 1,
            0xEE => 2,
            _ => panic!("unexpected opcode: {}", opcode),
        };

        self.reg.sp = self.reg.sp.wrapping_add(1);
        let OperationResult{ ret: data, cycles: read_cycles } = self.read_from_stack();

        match reg_type {
            0 => self.reg.a = data,
            1 => self.reg.x = data,
            2 => self.reg.y = data,
            3 => self.reg.psw.set(data),
            _ => panic!("register type must be between 0 to 3"),
        };

        OperationResult::new_unit(read_cycles + 2)
    }

    fn nop(&mut self, _opcode: u8) -> OperationResult<()> {
        OperationResult::new_unit(1)
    }

    fn sleep_or_stop(&mut self, _opcode: u8) -> OperationResult<()> {        
        self.is_stopped = true;
        OperationResult::new_unit(2)
    }

    fn clrp(&mut self, _opcode: u8) -> OperationResult<()> {
        self.reg.psw.negate_page();
        OperationResult::new_unit(1)
    }

    fn setp(&mut self, _opcode: u8) -> OperationResult<()> {
        self.reg.psw.assert_page();
        OperationResult::new_unit(1)
    }

    fn ei(&mut self, _opcode: u8) -> OperationResult<()> {
        self.reg.psw.assert_interrupt();
        OperationResult::new_unit(2)
    }

    fn di(&mut self, _opcode: u8) -> OperationResult<()> {
        self.reg.psw.negate_overflow();
        OperationResult::new_unit(2)
    }

    fn set1(&mut self, opcode: u8) -> OperationResult<()> {
        let OperationResult{ ret: addr, cycles: read_cycles } = self.read_from_pc();
        let shamt = (opcode >> 5) & 1;
        let OperationResult{ ret: data, cycles: read_page_cycles } = self.read_from_page(addr);
        let x = data | (1 << shamt);

        let write_cycles = self.write_to_page(addr, x).cycles();

        OperationResult::new_unit(read_cycles + read_page_cycles + write_cycles)
    }

    fn clr1(&mut self, opcode: u8) -> OperationResult<()> {
        let OperationResult { ret: addr, cycles: read_cycles } = self.read_from_pc();
        let shamt = (opcode >> 5) & 1;
        let OperationResult { ret: data, cycles: read_page_cycles } = self.read_from_page(addr);
        let x = data & !(1 << shamt);

        let write_cycles = self.write_to_page(addr, x).cycles();

        OperationResult::new_unit(read_cycles + read_page_cycles + write_cycles)
    }

    fn not1(&mut self, _opcode: u8) -> OperationResult<()> {
        let OperationResult { ret: (addr, bit_idx), cycles: fetch_cycles } = self.addr_and_idx();
        let OperationResult { ret: data, cycles: read_cycles } = self.read_ram(addr);
        let ret = data ^ (1 << bit_idx);
        
        let write_cycles = self.write_ram(addr, ret).cycles();

        OperationResult::new_unit(fetch_cycles + read_cycles + write_cycles)
    }

    fn mov1_to_mem(&mut self, _opcode: u8) -> OperationResult<()> {
        let OperationResult{ ret: (addr, bit_idx), cycles: fetch_cycle } = self.addr_and_idx();        
        let OperationResult{ ret: data, cycles: read_cycles } = self.read_ram(addr);
        let ret = 
            if self.reg.psw.carry() {
                data | (1 << bit_idx)                
            } else {
                data & !(1 << bit_idx)
            };
        
        let write_cycles = self.write_ram(addr, ret).cycles(); 

        OperationResult::new_unit(fetch_cycle + read_cycles + write_cycles + 1)
    }

    fn mov1_to_psw(&mut self, _opcode: u8) -> OperationResult<()> {
        let OperationResult{ ret: (addr, bit_idx), cycles: fetch_cycles } = self.addr_and_idx();
        let OperationResult{ ret: data, cycles: read_cycles } = self.read_ram(addr);
        let carry = (data >> bit_idx) & 1;
        self.reg.psw.set_carry(carry == 1);

        OperationResult::new_unit(fetch_cycles + read_cycles)
    }

    fn or1(&mut self, opcode: u8) -> OperationResult<()> {
        let rev = (opcode & 0x20)  != 0;
        let OperationResult{ ret: (addr, bit_idx), cycles: fetch_cycles } = self.addr_and_idx();
        let OperationResult{ ret: data, cycles: read_cycles } = self.read_ram(addr);
        let bit = ((data >> bit_idx) & 1) == 1;        
        let ret = self.reg.psw.carry () | (rev ^ bit);

        self.reg.psw.set_carry(ret);

        OperationResult::new_unit(fetch_cycles + read_cycles + 1)
    }

    fn and1(&mut self, opcode: u8) -> OperationResult<()> {
        let rev = (opcode & 0x20) != 0;
        let OperationResult{ ret: (addr, bit_idx), cycles: fetch_cycles } = self.addr_and_idx();
        let OperationResult{ ret: data, cycles: read_cycles } = self.read_ram(addr);
        let bit = ((data >> bit_idx) & 1) == 1;        
        let ret = self.reg.psw.carry () & (rev ^ bit);
        
        self.reg.psw.set_carry(ret);

        OperationResult::new_unit(fetch_cycles + read_cycles)
    }

    fn eor1(&mut self, _opcode: u8) -> OperationResult<()> {
        let OperationResult{ ret: (addr, bit_idx), cycles: fetch_cycles } = self.addr_and_idx();
        let OperationResult{ ret: data, cycles: read_cycles } = self.read_ram(addr);
        let bit = ((data >> bit_idx) & 1) == 1;        
        let ret = self.reg.psw.carry () ^ bit;

        self.reg.psw.set_carry(ret);

        OperationResult::new_unit(fetch_cycles + read_cycles + 1)
    }

    fn clrc(&mut self, _opcode: u8) -> OperationResult<()> {
        self.reg.psw.set_carry(false);
        OperationResult::new_unit(1)
    }

    fn setc(&mut self, _opcode: u8) -> OperationResult<()> {
        self.reg.psw.set_carry(true);
        OperationResult::new_unit(1)
    }

    fn notc(&mut self, _opcode: u8) -> OperationResult<()> {
        self.reg.psw.set_carry(!self.reg.psw.carry());
        OperationResult::new_unit(2)
    }

    fn clrv(&mut self, _opcode: u8) -> OperationResult<()> {
        self.reg.psw.set_overflow(false);
        self.reg.psw.set_half(false);
        OperationResult::new_unit(1)
    }

    fn addr_and_idx(&mut self) -> OperationResult<(u16, u8)> {
        let OperationResult{ ret: lower, cycles: lower_cycles } = self.read_from_pc();
        let OperationResult{ ret: upper, cycles: upper_cycles } = self.read_from_pc();
        let lower = lower as u16;
        let upper = upper as u16;
        let addr = (upper << 8) | lower;
        let idx = (addr >> 13) as u8;
        let addr = addr & 0x1FFF;

        OperationResult::new((addr, idx), lower_cycles + upper_cycles)
    }

    fn branch_by_psw(&mut self, opcode: u8) -> OperationResult<()> {
        let OperationResult{ ret: rr, cycles: fetch_cycles } = self.read_from_pc();        
        let flag_type = (opcode >> 6) & 0x03;

        // 各フラグ（要素順にsign, overflow, carry, zero）のみを抽出するためのマスク
        let masks = [0x80, 0x40, 0x01, 0x02];
        let flag = masks[flag_type as usize] & self.reg.psw.get();

        // フラグに対してtrueとfalseどちらなら分岐が発生するのか
        let branch_trigger = (opcode & 0x20) != 0;

        let branch = (flag != 0) == branch_trigger;        
        if branch { 
            // 一旦 u8 -> i8のキャストを挟まないと、期待通りの結果が得られない
            // 例) 0xFF as i16 => 255(0xFF), (0xFF as i8) as i16 => -1
            let offset = (rr as i8) as i16;
            let next_pc = (self.reg.pc as i16).wrapping_add(offset);
            self.reg.pc = next_pc as u16;

            OperationResult::new_unit(fetch_cycles + 2)
         } else {
            OperationResult::new_unit(fetch_cycles)
         }

    }

    fn branch_by_mem_bit(&mut self, opcode: u8) -> OperationResult<()> {
        let OperationResult{ ret: addr, cycles: addr_cycles } = self.read_from_pc();
        let OperationResult{ ret: data, cycles: data_cycles } = self.read_from_page(addr);        
        let OperationResult{ ret: offset, cycles: offset_cycles } = self.read_from_pc();
        let offset = offset as u16;
        let offset = if (offset & 0x80) != 0 { 0xFF00 | offset } else { offset };

        let require_true = (opcode & 0x10) == 0;
        let bit_idx = (opcode >> 5) & 0x7;

        let bit = ((data >> bit_idx) & 1) == 1;
        let is_branch = bit == require_true;

        if is_branch {
            self.reg.pc = self.reg.pc.wrapping_add(offset);
            OperationResult::new_unit(addr_cycles + data_cycles + offset_cycles + 3)
        } else {
            OperationResult::new_unit(addr_cycles + data_cycles + offset_cycles + 1)
        }
    }

    fn cbne(&mut self, opcode: u8) -> OperationResult<()> {
        let OperationResult{ ret: aa, cycles: aa_cycles } = self.read_from_pc();
        let require_x = match opcode {
            0x2E => false,
            0xDE => true,
            _ => panic!("expected opcodes are 0x2E and 0xDE. actual: {:#04x}", opcode),
        };
        let OperationResult{ ret: addr, cycles: addr_cycles } = 
            if require_x { 
                let aa = aa.wrapping_add(self.reg.x);
                OperationResult::new(aa, 1)
            } else { 
                OperationResult::new(aa, 0)
            };
        let OperationResult{ ret: data, cycles: data_cycles } = self.read_from_page(addr);
        let OperationResult{ ret: rr, cycles: rr_cycles } = self.read_from_pc();
        let rr = rr as u16;

        if self.reg.a != data {
            let offset = if (rr & 0x80) != 0 { 0xFF00 | rr } else { rr };
            self.reg.pc = self.reg.pc.wrapping_add(offset);
            OperationResult::new_unit(aa_cycles + addr_cycles + data_cycles + rr_cycles + 3)
        } else {
            OperationResult::new_unit(aa_cycles + addr_cycles + data_cycles + rr_cycles + 1)
        }
    }

    fn dbnz_y(&mut self, _opcode: u8) -> OperationResult<()> {
        let OperationResult{ ret: rr, cycles: rr_cycles } = self.read_from_pc();
        let rr = rr as u16;
        self.reg.y = self.reg.y.wrapping_sub(1);
        
        if self.reg.y != 0 {
            let offset = if (rr & 0x80) != 0 { 0xFF00 | rr } else { rr };
            self.reg.pc = self.reg.pc.wrapping_add(offset);
            OperationResult::new_unit(rr_cycles + 4)
        } else {
            OperationResult::new_unit(rr_cycles + 2)
        }
    }

    fn dbnz_data(&mut self, _opcode: u8) -> OperationResult<()> {
        let OperationResult{ ret: addr, cycles: addr_cycles } = self.read_from_pc();
        let OperationResult{ ret: rr, cycles: rr_cycles } = self.read_from_pc();
        let rr = rr as u16;
        let OperationResult{ ret: data, cycles: data_cycles } = self.read_from_page(addr);
        let data = data.wrapping_sub(1);        
        let write_cycles = self.write_to_page(addr, data).cycles();        

        if data != 0 {
            let offset = if (rr & 0x80) != 0 { 0xFF00 | rr } else { rr };
            self.reg.pc = self.reg.pc.wrapping_add(offset);
            OperationResult::new_unit(addr_cycles + rr_cycles + data_cycles + write_cycles + 3)
        } else {
            OperationResult::new_unit(addr_cycles + rr_cycles + data_cycles + write_cycles + 1)
        }
    }

    fn bra(&mut self, _opcode: u8) -> OperationResult<()> {
        let OperationResult{ ret: rr, cycles: rr_cycles } = self.read_from_pc();
        let rr = rr as u16;
        let offset = if (rr & 0x80) != 0 { 0xFF00 | rr } else { rr };
        self.reg.pc = self.reg.pc.wrapping_add(offset);

        OperationResult::new_unit(rr_cycles + 2)
    }

    fn jmp_abs(&mut self, _opcode: u8) -> OperationResult<()> {
        let OperationResult{ ret: lower, cycles: lower_cycles } = self.read_from_pc();
        let OperationResult{ ret: upper, cycles: upper_cycles } = self.read_from_pc();
        let lower = lower as u16;
        let upper = upper as u16;
        let addr = (upper << 8) | lower;

        self.reg.pc = addr;

        OperationResult::new_unit(lower_cycles + upper_cycles)
    }

    fn jmp_abs_x(&mut self, _opcode: u8) -> OperationResult<()> { 
        let OperationResult{ ret: lower, cycles: lower_cycles } = self.read_from_pc();
        let OperationResult{ ret: upper, cycles: upper_cycles } = self.read_from_pc();
        let lower = lower as u16;
        let upper = upper as u16;
        let addr = (upper << 8) | lower;
        let addr = addr.wrapping_add(self.reg.x as u16);

        let OperationResult{ ret: dst_lower, cycles: dst_lower_cycles } = self.read_ram(addr);
        let OperationResult{ ret: dst_upper, cycles: dst_upper_cycles } = self.read_ram(addr.wrapping_add(1));
        let dst_lower = dst_lower as u16;
        let dst_upper = dst_upper as u16; 

        self.reg.pc = (dst_upper << 8) | dst_lower;

        OperationResult::new_unit(lower_cycles + upper_cycles + dst_lower_cycles + dst_upper_cycles + 1)
    }
    
    fn call(&mut self, _opcode: u8) -> OperationResult<()> {
        let OperationResult{ ret: dst_lower, cycles: dst_lower_cycles } = self.read_from_pc();
        let OperationResult{ ret: dst_upper, cycles: dst_upper_cycles } = self.read_from_pc();
        let dst_lower = dst_lower as u16;
        let dst_upper = dst_upper as u16;
        let dst = (dst_upper << 8) | dst_lower;

        let upper_write_cycles = self.write_to_stack((self.reg.pc >> 8) as u8).cycles();
        self.reg.sp = self.reg.sp.wrapping_sub(1);
        let lower_write_cycles = self.write_to_stack(self.reg.pc as u8).cycles();
        self.reg.sp = self.reg.sp.wrapping_sub(1);

        let dst_cycles = dst_upper_cycles + dst_lower_cycles;
        let write_cycles = upper_write_cycles + lower_write_cycles;
    
        self.reg.pc = dst;

        OperationResult::new_unit(dst_cycles + write_cycles + 3)
    }

    fn tcall(&mut self, opcode: u8) -> OperationResult<()> {
        let pc_lower = self.reg.pc as u8;
        let pc_upper = (self.reg.pc >> 8) as u8;
        let upper_write_cycles = self.write_to_stack(pc_upper).cycles();
        self.reg.sp = self.reg.sp.wrapping_sub(1);

        let lower_write_cycles = self.write_to_stack(pc_lower).cycles();
        self.reg.sp = self.reg.sp.wrapping_sub(1);

        let offset = ((opcode >> 4) << 1) as u16;        
        let addr = 0xFFDE - offset;        

        let OperationResult{ ret: next_pc_lower, cycles: lower_cycles } = self.read_ram(addr);
        let OperationResult{ ret: next_pc_upper, cycles: upper_cycles } = self.read_ram(addr.wrapping_add(1));
        let next_pc_lower = next_pc_lower as u16;
        let next_pc_upper = next_pc_upper as u16;
        let next_pc = (next_pc_upper << 8) | next_pc_lower;

        let write_cycles = upper_write_cycles + lower_write_cycles;
        let read_cycles = lower_cycles + upper_cycles;

        self.reg.pc = next_pc;

        OperationResult::new_unit(write_cycles + read_cycles + 3)
    }

    fn pcall(&mut self, _opcode: u8) -> OperationResult<()> {        
        let OperationResult{ ret: lower, cycles: lower_cycles } = self.read_from_pc();
        let next_pc = 0xFF00 | lower as u16;

        let pc_lower = self.reg.pc as u8;
        let pc_upper = (self.reg.pc >> 8) as u8;
        let write_upper_cycles = self.write_to_stack(pc_upper).cycles();
        self.reg.sp = self.reg.sp.wrapping_sub(1);

        let write_lower_cycles = self.write_to_stack(pc_lower).cycles();
        self.reg.sp = self.reg.sp.wrapping_sub(1);

        let write_cycles = write_upper_cycles + write_lower_cycles;
        self.reg.pc = next_pc;

        OperationResult::new_unit(lower_cycles + write_cycles + 2)
    }

    fn ret(&mut self, _opcode: u8) -> OperationResult<()> {
        let partial_pcs: Vec<OperationResult<u8>> = (0..2).map(|_| {
            self.reg.sp = self.reg.sp.wrapping_add(1);
            let OperationResult{ ret: partial_pc, cycles: pc_cycles } = self.read_from_stack();            

            OperationResult::new(partial_pc, pc_cycles + 1)
        }).collect();
        
        let OperationResult{ ret: lower, cycles: lower_cycles } = partial_pcs[0];
        let OperationResult{ ret: upper, cycles: upper_cycles } = partial_pcs[1];
        let lower = lower as u16;
        let upper = upper as u16;
        let next_pc = (upper << 8) | lower;

        self.reg.pc = next_pc;

        OperationResult::new_unit(lower_cycles + upper_cycles)
    }

    fn ret1(&mut self, _opcode: u8) -> OperationResult<()> {
        let OperationResult{ ret: psw, cycles: psw_cycles } = self.read_from_stack();
        self.reg.sp = self.reg.sp.wrapping_add(1);
        self.reg.psw.set(psw);
        
        let OperationResult{ ret: lower_pc, cycles: lower_cycles } = self.read_from_stack();
        self.reg.sp = self.reg.sp.wrapping_add(1);

        let OperationResult{ ret: upper_pc, cycles: upper_cycles } = self.read_from_stack();
        self.reg.sp = self.reg.sp.wrapping_add(1);

        let lower_pc = lower_pc as u16;
        let upper_pc = upper_pc as u16;

        self.reg.pc = (upper_pc << 8) | (lower_pc);

        OperationResult::new_unit(psw_cycles + lower_cycles + upper_cycles + 2)
    }

    fn brk(&mut self, _opcode: u8) -> OperationResult<()> {
        let OperationResult{ ret: lower, cycles: lower_cycles } = self.read_ram(0xFFDE);
        let OperationResult{ ret: upper, cycles: upper_cycles } = self.read_ram(0xFFDF);
        let lower = lower as u16;
        let upper = upper as u16;
        let next_pc = (upper << 8) | lower;

        let pc_lower = self.reg.pc as u8;
        let pc_upper = (self.reg.pc >> 8) as u8;
        let write_upper_cycles = self.write_to_stack(pc_upper).cycles();
        self.reg.sp = self.reg.sp.wrapping_sub(1);
        let write_lower_cycles = self.write_to_stack(pc_lower).cycles();
        self.reg.sp = self.reg.sp.wrapping_sub(1);

        let write_psw_cycles = self.write_to_stack(self.reg.psw.get()).cycles();
        self.reg.sp = self.reg.sp.wrapping_sub(1);
        
        self.reg.pc = next_pc;

        let read_cycles = lower_cycles + upper_cycles;
        let write_cycles = write_upper_cycles + write_lower_cycles + write_psw_cycles;

        OperationResult::new_unit(read_cycles + write_cycles + 2)
    }    

    fn fetch_alu_op(opcode: u8) -> fn(&mut Register, u8, u8) -> u8 {
        let upper = (opcode >> 4) & 0x0F;
        match upper {
            0x0 | 0x1 => or,
            0x2 | 0x3 => and,
            0x4 | 0x5 => eor,
            0x6 | 0x7 => cmp,
            0x8 | 0x9 => adc,
            0xA | 0xB => sbc,
            _ => panic!("upper must be between 0x0 to 0xB. actual {:#04x}", upper),
        } 
    }

    fn fetch_shift_op(opcode: u8) -> fn(u8, bool) -> (u8, bool) {
        let upper = (opcode >> 4) & 0x0F;
        match upper {
            0x0 | 0x1 => asl,
            0x2 | 0x3 => rol,
            0x4 | 0x5 => lsr,
            0x6 | 0x7 => ror,
            _ => panic!("upper expects to be between 0 to 7. actual: {:#04x}", upper),
        }
    }

    fn alu_dp(&mut self, opcode: u8) -> OperationResult<()> {
        let op = match opcode {
            0x3E => cmp,
            0x7E => cmp,
               _ => Spc700::fetch_alu_op(opcode),
        };
        let upper = (opcode >> 4) & 0x0F;
        let lower = opcode & 0x0F;
        let reg_type = match (upper, lower) {
            (upper, 0x4) if upper % 2 == 0 => 0,
            (0x3, 0xE) => 1,
            (0x7, 0xE) => 2,
            _ => panic!("unexpected opcode: {}", opcode),
        };

        let OperationResult{ ret: addr, cycles: addr_cycles } = self.read_from_pc();        
        let OperationResult{ ret: b, cycles: b_cycles } = self.read_from_page(addr);
        let a = match reg_type {
            0 => self.reg.a,
            1 => self.reg.x,
            2 => self.reg.y,
            _ => panic!("from register types must be between 0 to 2"),
        };         

        self.reg.a = op(&mut self.reg, a, b);        

        OperationResult::new_unit(addr_cycles + b_cycles)
    }

    fn alu_addr(&mut self, opcode: u8) -> OperationResult<()> {        
        let op = match opcode {
            0x1E => cmp,
            0x5E => cmp, 
               _ => Spc700::fetch_alu_op(opcode),
        };
        let op_upper = (opcode >> 4) & 0x0F;
        let op_lower = opcode & 0x0F;
        let reg_type = match (op_upper, op_lower) {
            (upper, 0x5) if upper % 2 == 0 => 0,
            (0x1, 0xE) => 1,
            (0x5, 0xE) => 2,
            _ => panic!("unexpected opcode: {}", opcode),
        };
        
        let OperationResult{ ret: lower, cycles: lower_cycles } = self.read_from_pc();
        let OperationResult{ ret: upper, cycles: upper_cycles } = self.read_from_pc();
        let lower = lower as u16;
        let upper = upper as u16;
        let addr = (upper << 8) | lower;

        let OperationResult{ ret: b, cycles: b_cycles } = self.read_ram(addr);
        let a = match reg_type {
            0 => self.reg.a,
            1 => self.reg.x,
            2 => self.reg.y,
            _ => panic!("from register types must be between 0 to 2"),
        };        
        
        self.reg.a = op(&mut self.reg, a, b);        

        OperationResult::new_unit(lower_cycles + upper_cycles + b_cycles)
    }

    fn alu_indirect_x(&mut self, opcode: u8) -> OperationResult<()> {
        let op = Spc700::fetch_alu_op(opcode);
        let x = self.reg.x;

        let a = self.reg.a;
        let OperationResult{ ret: b, cycles: b_cycles } = self.read_from_page(x);
        let data = op(&mut self.reg, a, b);

        self.reg.a = data;       
        
        OperationResult::new_unit(b_cycles + 1)
    }

    fn alu_x_idx_indirect(&mut self, opcode: u8) -> OperationResult<()> {
        let op = Spc700::fetch_alu_op(opcode);
        let OperationResult{ ret: addr, cycles: addr_cycles } = self.read_from_pc();
        let addr = addr.wrapping_add(self.reg.x);
        
        let a = self.reg.a;
        let OperationResult{ ret: b, cycles: b_cycles } = self.read_from_page(addr);        
        let ret = op(&mut self.reg, a, b);

        self.reg.a = ret;       
        
        OperationResult::new_unit(addr_cycles + b_cycles + 1)
    }

    fn alu_x_idx_addr(&mut self, opcode: u8) -> OperationResult<()> {
        let op = Spc700::fetch_alu_op(opcode);
        let OperationResult{ ret: lower, cycles: lower_cycles } = self.read_from_pc();
        let OperationResult{ ret: upper, cycles: upper_cycles } = self.read_from_pc();
        let lower = lower as u16;
        let upper = upper as u16;
        let addr = (upper << 8) | lower;
        let addr = addr.wrapping_add(self.reg.x as u16);
        let OperationResult{ ret: data, cycles: data_cycles } = self.read_ram(addr);

        let a = self.reg.a;
        let ret = op(&mut self.reg, a, data);

        self.reg.a = ret;

        OperationResult::new_unit(lower_cycles + upper_cycles + data_cycles + 1)
    }

    fn alu_x_ind_ind(&mut self, opcode: u8) -> OperationResult<()> {
        let op = Spc700::fetch_alu_op(opcode);
        let OperationResult{ ret: base_addr, cycles: base_addr_cycles } = self.read_from_pc();
        let base_addr = base_addr.wrapping_add(self.reg.x);        
        let OperationResult{ ret: lower, cycles: lower_cycles } = self.read_from_page(base_addr);
        let OperationResult{ ret: upper, cycles: upper_cycles } = self.read_from_page(base_addr.wrapping_add(1));
        let lower = lower as u16;
        let upper = upper as u16;
        let addr = (upper << 8) | lower;
        let OperationResult{ ret: data, cycles: data_cycles } = self.read_ram(addr);

        let a = self.reg.a;
        let ret = op(&mut self.reg, a, data);
        
        self.reg.a = ret;

        let addr_cycles = lower_cycles + upper_cycles;
        OperationResult::new_unit(addr_cycles + base_addr_cycles + data_cycles + 1)
    }

    fn alu_y_ind_ind(&mut self, opcode: u8) -> OperationResult<()> {
        let op = Spc700::fetch_alu_op(opcode);
        let OperationResult{ ret: base_addr, cycles: base_addr_cycles } = self.read_from_pc();
        let OperationResult{ ret: lower, cycles: lower_cycles } = self.read_from_page(base_addr);
        let OperationResult{ ret: upper, cycles: upper_cycles } = self.read_from_page(base_addr.wrapping_add(1));
        let lower = lower as u16;
        let upper = upper as u16;
        let addr = (upper << 8) | lower;
        let addr = addr.wrapping_add(self.reg.y as u16);
        let OperationResult{ ret: data, cycles: data_cycles } = self.read_ram(addr);
        let a = self.reg.a;
        
        let ret = op(&mut self.reg, a, data);
        
        self.reg.a = ret;

        let addr_cycles = lower_cycles + upper_cycles;
        OperationResult::new_unit(base_addr_cycles + addr_cycles + data_cycles + 1)
    }

    fn alu_y_idx_addr(&mut self, opcode: u8) -> OperationResult<()> {
        let op = Spc700::fetch_alu_op(opcode);
        let OperationResult{ ret: lower, cycles: lower_cycles } = self.read_from_pc();
        let OperationResult{ ret: upper, cycles: upper_cycles } = self.read_from_pc();
        let lower = lower as u16;
        let upper = upper as u16;
        let addr = (upper << 8) | lower;
        let addr = addr.wrapping_add(self.reg.y as u16);
        let OperationResult{ ret: data, cycles: data_cycles } = self.read_ram(addr);
        let a = self.reg.a;

        let ret = op(&mut self.reg, a, data);

        self.reg.a = ret;

        let addr_cycles = lower_cycles + upper_cycles;

        OperationResult::new_unit(addr_cycles + data_cycles + 1)
    }

    fn alu_x_y(&mut self, opcode: u8) -> OperationResult<()> {
        let op = Spc700::fetch_alu_op(opcode);
        let op_upper = (opcode >> 4) & 0x0F;
        let is_cmp_op = op_upper == 0x6 || op_upper == 0x7;

        let OperationResult{ ret: x_data, cycles: x_cycles } = self.read_from_page(self.reg.x);        
        let OperationResult{ ret: y_data, cycles: y_cycles } = self.read_from_page(self.reg.y);
        let ret = op(&mut self.reg, x_data, y_data);        

        let additional_cycles = if !is_cmp_op {
            self.write_to_page(self.reg.x, ret).cycles()
        } else {
            1
        };

        OperationResult::new_unit(x_cycles + y_cycles + additional_cycles + 1)
    }

    // TODO 取得及び書き込むレジスタの種類を数字ではなくenum値で表現するように変更する
    fn alu_imm(&mut self, opcode: u8) -> OperationResult<()> {
        let op = match opcode {
            0xC8 => cmp,
            0xAD => cmp,
            _ => Spc700::fetch_alu_op(opcode)
        };
        let upper = (opcode >> 4) & 0x0F;
        let lower = opcode & 0x0F;
        let reg_type = match (upper, lower) {
            (upper, 0x8) if upper <= 0xB => 0,
            (0xC, 0x8) => 1,
            (0xA, 0xD) => 2, 
            _ => panic!("unknown opcode combination: {:x}", opcode),
        };

        let a = match reg_type {
            0 => self.reg.a,
            1 => self.reg.x,
            2 => self.reg.y,
            _ => panic!("from register types must be between 0 to 2"),
        };

        let OperationResult{ ret: b, cycles: b_cycles } = self.read_from_pc();
        let ret = op(&mut self.reg, a, b);
        match reg_type {
            0 => self.reg.a = ret,
            1 => self.reg.x = ret,
            2 => self.reg.y = ret,
            _ => panic!("from register types must be between 0 to 2"),
        };

        OperationResult::new_unit(b_cycles)
    }

    fn alu_dp_imm(&mut self, opcode: u8) -> OperationResult<()> {
        let op = Spc700::fetch_alu_op(opcode);
        let op_upper = (opcode >> 4) & 0x0F;
        let is_cmp_op = op_upper == 0x6 || op_upper == 0x7; 

        let OperationResult{ ret: imm, cycles: imm_cycles } = self.read_from_pc();
        let OperationResult{ ret: addr, cycles: addr_cycles } = self.read_from_pc();
        let OperationResult{ ret: data, cycles: data_cycles } = self.read_from_page(addr);
        let ret = op(&mut self.reg, data, imm);

        let additional_cycles = if !is_cmp_op {
            self.write_to_page(addr, ret).cycles()
        } else {
            1
        };

        let op_cycles = imm_cycles + addr_cycles + data_cycles;

        OperationResult::new_unit(op_cycles + additional_cycles)
    }

    fn alu_dp_dp(&mut self, opcode: u8) -> OperationResult<()> {
        let op = Spc700::fetch_alu_op(opcode);
        let op_upper = (opcode >> 4) & 0x0F;
        let is_cmp_op = op_upper == 0x6 || op_upper == 0x7; 

        let OperationResult{ ret: bb, cycles: bb_cycles } = self.read_from_pc();
        let OperationResult{ ret: b, cycles: b_cycles }  = self.read_from_page(bb);
        let OperationResult{ ret: aa, cycles: aa_cycles } = self.read_from_pc();        
        let OperationResult{ ret: a, cycles: a_cycles } = self.read_from_page(aa);

        let ret = op(&mut self.reg, a, b);

        let additional_cycles = if !is_cmp_op {
            self.write_to_page(aa, ret).cycles()
        } else {
            1
        }; 

        let op_cycles = bb_cycles + b_cycles + aa_cycles + a_cycles;
        OperationResult::new_unit(op_cycles + additional_cycles)
    }

    fn shift_acc(&mut self, opcode: u8) -> OperationResult<()> {
        let op = Spc700::fetch_shift_op(opcode);

        let ret = self.shift(opcode, self.reg.a, op);
        
        self.reg.a = ret;       
    
        OperationResult::new_unit(1)
    }

    fn shift_dp(&mut self, opcode: u8) -> OperationResult<()> {
        let op = Spc700::fetch_shift_op(opcode);

        let OperationResult{ ret: addr, cycles: addr_cycles } = self.read_from_pc();
        let OperationResult{ ret: data, cycles: data_cycles } = self.read_from_page(addr);        
        let ret = self.shift(opcode, data, op);

        let write_cycles = self.write_to_page(addr, ret).cycles();

        OperationResult::new_unit(addr_cycles + data_cycles + write_cycles)
    }

    fn shift_x_idx_indirect(&mut self, opcode: u8) -> OperationResult<()> {
        let op = Spc700::fetch_shift_op(opcode);

        let OperationResult{ ret: addr, cycles: addr_cycles } = self.read_from_pc();
        let addr = addr.wrapping_add(self.reg.x);

        let OperationResult{ ret: data, cycles: data_cycles } = self.read_from_page(addr);
        let ret = self.shift(opcode, data, op);

        let write_cycles = self.write_to_page(addr, ret).cycles();

        OperationResult::new_unit(addr_cycles + data_cycles + write_cycles + 1)
    }

    fn shift_addr(&mut self, opcode: u8) -> OperationResult<()> {
        let op = Spc700::fetch_shift_op(opcode);

        let OperationResult{ ret: lower, cycles: lower_cycles } = self.read_from_pc();
        let OperationResult{ ret: upper, cycles: upper_cycles } = self.read_from_pc();
        let lower = lower as u16;
        let upper = upper as u16;
        let addr = (upper << 8) | lower;
        let OperationResult{ ret: data, cycles: data_cycles } = self.read_ram(addr);

        let ret = self.shift(opcode, data, op);

        let write_cycles = self.write_ram(addr, ret).cycles();

        let cycles = lower_cycles + upper_cycles + data_cycles + write_cycles;

        OperationResult::new_unit(cycles)
    }

    fn shift(&mut self, opcode: u8, data: u8, op: impl Fn(u8, bool) -> (u8, bool)) -> u8 {
        let use_carry = (opcode & 0x20) != 0;
        let bit = if use_carry { self.reg.psw.carry() } else { false };
        let (data, is_carry) = op(data, bit);

        let is_zero = data == 0;
        let is_neg = (data & 0x80) != 0;
        self.reg.psw.set_carry(is_carry);
        self.reg.psw.set_zero(is_zero);
        self.reg.psw.set_sign(is_neg);

        data
    }

    fn inc_dec_reg(&mut self, opcode: u8) -> OperationResult<()> {
        let is_inc = (opcode & 0x20) != 0;
        let upper = (opcode >> 4) & 0x0F;
        let lower = opcode & 0x0F;
        let reg_type = match (upper, lower) {
            (0x9, 0xC) => 0,
            (0xB, 0xC) => 0,
            (0xD, 0xC) => 2,
            (0xF, 0xC) => 2,
            (0x1, 0xD) => 1,
            (0x3, 0xD) => 1,
            _ => panic!("unexpected opcode: {}", opcode),
        };

        let data = match reg_type {
            0 => &mut self.reg.a,
            1 => &mut self.reg.x,
            2 => &mut self.reg.y,
            _ => panic!("expect 0 to 2 as register type"),
        };

        let ret =
            if is_inc { data.wrapping_add(1) }
            else      { data.wrapping_sub(1) };

        *data = ret;

        self.set_inc_dec_flag(ret);

        OperationResult::new_unit(1)
    }

    fn inc_dec_dp(&mut self, opcode: u8) -> OperationResult<()> {
        let is_inc = (opcode & 0x20) != 0;
        let OperationResult{ ret: addr, cycles: addr_cycles } = self.read_from_pc();
        let OperationResult{ ret: data, cycles: data_cycles }  = self.read_from_page(addr);
        let ret =
            if is_inc { data.wrapping_add(1) }
            else      { data.wrapping_sub(1) };

        let write_cycles = self.write_to_page(addr, ret).cycles();
        self.set_inc_dec_flag(ret);

        OperationResult::new_unit(addr_cycles + data_cycles + write_cycles)
    }

    fn inc_dec_x_idx_indirect(&mut self, opcode: u8) -> OperationResult<()> {
        let is_inc = (opcode & 0x20) != 0;
        let OperationResult{ ret: addr, cycles: addr_cycles } = self.read_from_pc();
        let addr = addr.wrapping_add(self.reg.x);

        let OperationResult{ ret: data, cycles: data_cycles } = self.read_from_page(addr);
        let ret = 
            if is_inc { data.wrapping_add(1) }
            else      { data.wrapping_sub(1) };

        let write_cycles = self.write_to_page(addr, ret).cycles();
        self.set_inc_dec_flag(ret);

        OperationResult::new_unit(addr_cycles + data_cycles + write_cycles + 1)
    }

    fn inc_dec_addr(&mut self, opcode: u8) -> OperationResult<()> {
        let is_inc = (opcode & 0x20) != 0;
        let OperationResult{ ret: lower, cycles: lower_cycles } = self.read_from_pc();
        let OperationResult{ ret: upper, cycles: upper_cycles } = self.read_from_pc();
        let lower = lower as u16;
        let upper = upper as u16;
        let addr = (upper << 8) | lower;
        let OperationResult{ ret: data, cycles: data_cycles } = self.read_ram(addr);

        let ret =
            if is_inc { data.wrapping_add(1) }
            else      { data.wrapping_sub(1) };

        let write_cycles = self.write_ram(addr, ret).cycles();
        self.set_inc_dec_flag(ret);

        OperationResult::new_unit(lower_cycles + upper_cycles + data_cycles + write_cycles)
    }
    
    fn set_inc_dec_flag(&mut self, data: u8) -> () {
        let is_neg = (data & 0x80) != 0;
        let is_zero = data == 0;

        self.reg.psw.set_sign(is_neg);
        self.reg.psw.set_zero(is_zero);
    }

    fn addw(&mut self, _opcode: u8) -> OperationResult<()> {
        self.reg.psw.negate_carry();

        let OperationResult{ ret: (ya, word), cycles: fetch_cycles } = self.get_word_operands();        
        let ret_lower = adc(&mut self.reg, ya as u8, word as u8) as u16;
        let ret_upper = adc(&mut self.reg, (ya >> 8) as u8, (word >> 8) as u8) as u16;
        let ret = (ret_upper << 8) | ret_lower;
                
        self.reg.set_ya(ret);
        self.reg.psw.set_zero(ret == 0);

        OperationResult::new_unit(fetch_cycles + 1)
    }

    fn subw(&mut self, _opcode: u8) -> OperationResult<()> {
        self.reg.psw.assert_carry();

        let OperationResult{ ret: (ya, word), cycles: fetch_cycles } = self.get_word_operands();
        let ret_lower = sbc(&mut self.reg, ya as u8, word as u8) as u16;
        let ret_upper = sbc(&mut self.reg, (ya >> 8) as u8, (word >> 8) as u8) as u16;
        let ret = (ret_upper << 8) | ret_lower;

        self.reg.set_ya(ret);
        self.reg.psw.set_zero(ret == 0);

        OperationResult::new_unit(fetch_cycles + 1)
    }

    fn cmpw(&mut self, _opcode: u8) -> OperationResult<()> {
        let OperationResult{ ret: (ya, word), cycles: fetch_cycles } = self.get_word_operands();
        let ret = (ya as i32) - (word as i32);

        self.reg.psw.set_sign((ret & 0x8000) != 0);
        self.reg.psw.set_zero(ret as u16 == 0);
        self.reg.psw.set_carry(ret >= 0);

        OperationResult::new_unit(fetch_cycles)
    }

    fn inc_dec_word(&mut self, opcode: u8) -> OperationResult<()> {
        let OperationResult{ ret: addr, cycles: addr_cycles } = self.read_from_pc();
        let upper = (opcode >> 4) & 0x0F;
        let operand = match upper {
            0x1 => 0xFFFF,
            0x3 => 0x0001,
            _   => panic!("unexpected opcode: {}", opcode),
        };

        let OperationResult{ ret: word_lower, cycles: lower_cycles } = self.read_from_page(addr);
        let word_lower = word_lower as u16;
        let lower_result = word_lower.wrapping_add(operand); 
        let lower_carry = lower_result >> 8;
        let write_lower_cycles = self.write_to_page(addr, lower_result as u8).cycles();

        let OperationResult{ ret: word_upper, cycles: upper_cycles } = self.read_from_page(addr.wrapping_add(1));
        let word_upper = word_upper as u16;
        let upper_result = word_upper.wrapping_add(lower_carry); 
        let write_upper_cycles = self.write_to_page(addr.wrapping_add(1), upper_result as u8).cycles();

        let result = (upper_result << 8) | lower_result;
        self.reg.psw.set_zero(result == 0);
        self.reg.psw.set_sign((result & 0x8000) != 0);

        let write_cycles = write_lower_cycles + write_upper_cycles;
        let cycles = addr_cycles + lower_cycles + upper_cycles + write_cycles;

        OperationResult::new_unit(cycles)
    }

    fn div(&mut self, _opcode: u8) -> OperationResult<()> {
        let ya = self.reg.ya();
        let x = self.reg.x as u16;

        self.reg.psw.set_overflow(self.reg.y >= self.reg.x);
        self.reg.psw.set_half((self.reg.y & 0x0F) >= (self.reg.x & 0x0F));

        if (self.reg.y as u16) < (x << 1) {
            self.reg.a = (ya / x) as u8;
            self.reg.y = (ya % x) as u8;
        } else {
            self.reg.a = (255 - (ya - (x << 9)) / (256 - x)) as u8;
            self.reg.y = (x + (ya - (x << 9)) % (256 - x)) as u8;
        }
        
        self.reg.psw.set_zero(self.reg.a == 0);
        self.reg.psw.set_sign((self.reg.a & 0x80) != 0);

        OperationResult::new_unit(11)
    }

    fn mul(&mut self, _opcode: u8) -> OperationResult<()> {
        let ya = (self.reg.y as u16) * (self.reg.a as u16);
        self.reg.set_ya(ya);

        self.reg.psw.set_zero(self.reg.y == 0);
        self.reg.psw.set_sign((self.reg.y & 0x80) != 0);

        OperationResult::new_unit(8)
    }

    fn get_word_operands(&mut self) -> OperationResult<(u16, u16)> {
        let OperationResult{ ret: addr, cycles: addr_cycles } = self.read_from_pc();
        let OperationResult{ ret: word_lower, cycles: lower_cycles }  = self.read_from_page(addr);
        let OperationResult{ ret: word_upper, cycles: upper_cycles }  = self.read_from_page(addr.wrapping_add(1));
        let word_lower = word_lower as u16;
        let word_upper = word_upper as u16;
        let word = (word_upper << 8) | word_lower;
        let ya = self.reg.ya();

        OperationResult::new((ya, word), addr_cycles + lower_cycles + upper_cycles)
    }

    fn daa(&mut self, _opcode: u8) -> OperationResult<()> {
        if self.reg.psw.carry() || self.reg.a > 0x99 {
            self.reg.a = self.reg.a.wrapping_add(0x60);
            self.reg.psw.assert_carry();
        }
        if self.reg.psw.half() || (self.reg.a & 0x0F) > 0x09 {
            self.reg.a = self.reg.a.wrapping_add(0x06);            
        }

        self.reg.psw.set_zero(self.reg.a == 0);
        self.reg.psw.set_sign((self.reg.a & 0x80) != 0);

        OperationResult::new_unit(2)
    }

    fn das(&mut self, _opcode: u8) -> OperationResult<()> {
        if !self.reg.psw.carry() || self.reg.a > 0x99 {
            self.reg.a = self.reg.a.wrapping_sub(0x60);
            self.reg.psw.set_carry(false);
        }
        if !self.reg.psw.half() || (self.reg.a & 0x0F) > 0x09 {
            self.reg.a = self.reg.a.wrapping_sub(0x06);
        }

        self.reg.psw.set_zero(self.reg.a == 0);
        self.reg.psw.set_sign((self.reg.a & 0x80) != 0);

        OperationResult::new_unit(2)
    }

    fn xcn(&mut self, _opcode: u8) -> OperationResult<()> {
        self.reg.a = (self.reg.a >> 4) | ((self.reg.a & 0x0F) << 4);

        self.reg.psw.set_zero(self.reg.a == 0);
        self.reg.psw.set_sign((self.reg.a & 0x80) != 0); 
        OperationResult::new_unit(4)
    }

    fn tclr1(&mut self, _opcode: u8) -> OperationResult<()> {
        let OperationResult{ ret: lower, cycles: lower_cycles } = self.read_from_pc();
        let OperationResult{ ret: upper, cycles: upper_cycles } = self.read_from_pc();
        let lower = lower as u16;
        let upper = upper as u16;
        let addr = (upper << 8) | lower;
        let OperationResult{ ret: data, cycles: data_cycles } = self.read_ram(addr);
        let ret = data & !self.reg.a;
        
        let read_cycles = self.read_ram(addr).cycles();
        let cmp = self.reg.a.wrapping_sub(data);
        self.reg.psw.set_zero(cmp == 0);
        self.reg.psw.set_sign((cmp & 0x80) != 0);       

        let write_cycles = self.write_ram(addr, ret).cycles();

        let cycles = lower_cycles + upper_cycles + data_cycles + read_cycles + write_cycles;

        OperationResult::new_unit(cycles)
    }

    fn tset1(&mut self, _opcode: u8) -> OperationResult<()> {
        let OperationResult{ ret: lower, cycles: lower_cycles } = self.read_from_pc();
        let OperationResult{ ret: upper, cycles: upper_cycles } = self.read_from_pc();
        let lower = lower as u16;
        let upper = upper as u16;
        let addr = (upper << 8) | lower;
        let OperationResult{ ret: data, cycles: data_cycles } = self.read_ram(addr);
        let ret = data | self.reg.a;

        let read_cycles = self.read_ram(addr).cycles();
        let cmp = self.reg.a.wrapping_sub(data);
        self.reg.psw.set_zero(cmp == 0);
        self.reg.psw.set_sign((cmp & 0x80) != 0);        

        let write_cycles = self.write_ram(addr, ret).cycles();

        let cycles = lower_cycles + upper_cycles + data_cycles + read_cycles + write_cycles;

        OperationResult::new_unit(cycles)
    }

    fn read_from_pc(&mut self) -> OperationResult<u8> {
        let addr = self.reg.inc_pc(1);    
        self.read_ram(addr)
    }

    fn read_from_stack(&mut self) -> OperationResult<u8> {
        let addr = (self.reg.sp as u16) | 0x0100;    
        self.read_ram(addr)
    }

    fn read_from_page(&mut self, addr: u8) -> OperationResult<u8> {
        let addr = (addr as u16) | (if self.reg.psw.page() { 0x0100 } else { 0x0000 });
        self.read_ram(addr)
    }    

    fn read_ram(&mut self, addr: u16) -> OperationResult<u8> {
        let ret = self.ram.read(addr, &mut self.dsp, &mut self.timer);
        OperationResult { cycles: 1, ret }
    }    

    fn write_to_page(&mut self, addr: u8, data: u8) -> OperationResult<()> {
        let addr = (addr as u16) | (if self.reg.psw.page() { 0x0100 } else { 0x0000 });
        self.write_ram(addr, data)
    }

    fn write_to_stack(&mut self, data: u8) -> OperationResult<()> {
        let addr = (self.reg.sp as u16) | 0x0100;        
        self.write_ram(addr, data) 
    }

    fn write_ram(&mut self, addr: u16, data: u8) -> OperationResult<()> {
        self.ram.write(addr, data, &mut self.dsp, &mut self.timer);
        OperationResult::new((), 1)
    }

    pub fn count_cycles(&mut self, cycle_count: u16) -> () {        
        self.dsp.cycles(cycle_count);
        self.timer.iter_mut().for_each(|timer| timer.cycles(cycle_count));
        self.cycle_counter += cycle_count as u64;
        self.total_cycles += cycle_count as u64;
    }
}

fn or(reg: &mut Register, a: u8, b: u8) -> u8 {
    let ret = a | b;
    reg.psw.set_sign((ret & 0x80) != 0);
    reg.psw.set_zero(ret == 0);

    ret
}

fn and(reg: &mut Register, a: u8, b: u8) -> u8 {
    let ret = a & b;
    reg.psw.set_sign((ret & 0x80) != 0);
    reg.psw.set_zero(ret == 0);

    ret
}

fn eor(reg: &mut Register, a: u8, b: u8) -> u8 {
    let ret = a ^ b;
    reg.psw.set_sign((ret & 0x80) != 0);
    reg.psw.set_zero(ret == 0);

    ret
}

fn cmp(reg: &mut Register, a: u8, b: u8) -> u8 {
    let ret = (a as i16) - (b as i16);
    reg.psw.set_sign((ret & 0x80) != 0);
    reg.psw.set_zero((ret as u8) == 0);
    reg.psw.set_carry(ret >= 0);

    a
}

fn adc(reg: &mut Register, a: u8, b: u8) -> u8 {
    let a = a as i32;
    let b = b as i32;
    let r = a + b + reg.psw.carry() as i32;

    reg.psw.set_sign((r as u8 & 0x80) != 0);
    reg.psw.set_overflow((!(a ^ b) & (a ^ r) & 0x80) != 0);
    reg.psw.set_half(((a ^ b ^ r) & 0x10) != 0);
    reg.psw.set_zero(r as u8 == 0);
    reg.psw.set_carry(r > 0xFF);

    r as u8
}

fn sbc(reg: &mut Register, a: u8, b: u8) -> u8 {
    let a = a as i32;
    let b = b as i32;
    let r = a - b - !reg.psw.carry() as i32;

    reg.psw.set_sign((r as u8 & 0x80) != 0);
    reg.psw.set_overflow(((a ^ b) & (a ^ r) & 0x80) != 0);
    reg.psw.set_half((!(a ^ b ^ r) & 0x10) != 0);
    reg.psw.set_zero(r as u8 == 0);
    reg.psw.set_carry(r >= 0);

    r as u8
}

fn asl(operand: u8, _is_carry: bool) -> (u8, bool) {
    let is_carried = (operand & 0x80) != 0;
    let ret = operand << 1;

    (ret, is_carried)
}

fn rol(operand: u8, is_carry: bool) -> (u8, bool) {
    let is_carried = (operand & 0x80) != 0;
    let shifted = operand << 1;
    let ret = shifted | (is_carry as u8);

    (ret, is_carried)
}

fn lsr(operand: u8, _is_carry: bool) -> (u8, bool) {
    let ret = operand >> 1;
    let carried = (operand & 1) != 0;

    (ret, carried)
}

fn ror(operand: u8, is_carry: bool) -> (u8, bool) {
    let shifted = operand >> 1;
    let ret = shifted | ((is_carry as u8) << 7);
    let carried = (operand & 1) != 0;

    (ret, carried)
}