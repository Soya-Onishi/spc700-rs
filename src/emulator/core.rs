extern crate spc;

use super::ram::*;
use super::register::*;
use crate::dsp::DSP;
use crate::emulator::timer::Timer;

use std::io::Result;
use std::path::Path;
use spc::spc::Spc;

pub struct Spc700 {
    pub reg: Register,
    pub ram: Ram,
    pub dsp: DSP,
    pub timer: [Timer; 3],
    pub cycle_counter: u64,
    total_cycles: u64,
    is_stopped: bool
}

const DECODE_TABLE: [fn(&mut Spc700, u8) -> (); 256] = [
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
    Spc700::mov_load_dp,
    Spc700::mov_load_addr,
    Spc700::mov_load_x_indirect,
    Spc700::mov_load_x_ind_ind,
    // upper opcode: 0xE
    // lower opcode: 0x8
    Spc700::mov_reg_imm,
    Spc700::mov_load_addr,
    Spc700::not1,
    Spc700::mov_load_dp,
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
    Spc700::mov_load_dp,
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
            self.cycles(2);
            self.dsp.flush(&mut self.ram);
            return;
        }

        let pc = self.reg.inc_pc(1);
        let opcode = self.read_ram(pc);                
        let instruction = DECODE_TABLE[opcode as usize];
        instruction(self, opcode);
        
        log::debug!("{}", &self.reg);

        self.dsp.flush(&mut self.ram);  // flush in force                                        
    }

    fn mov_reg_imm(&mut self, opcode: u8) -> () {
        let reg_type = match opcode {
            0x8D => 2,
            0xCD => 1, 
            0xE8 => 0,
            _ => panic!("unexpected opcode: {}", opcode),
        };

        let imm = self.read_from_pc();
        match reg_type {
            0 => self.reg.a = imm,
            1 => self.reg.x = imm,
            2 => self.reg.y = imm,
            _ => panic!("register type must be between 0 to 2"),
        }

        self.set_mov_flag(imm);
    }

    fn mov_reg_reg(&mut self, opcode: u8) -> () {
        self.cycles(1);
       
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
    }

    fn mov_load_dp(&mut self, opcode: u8) -> () {
        let reg_type = match opcode {
            0xE4 => 0,
            0xEB => 2,
            0xF8 => 1,
            _ => panic!("unexpected opcode: {}", opcode),
        };

        let addr = self.read_from_pc();
        let data = self.read_from_page(addr);

        match reg_type {
            0 => self.reg.a = data,
            1 => self.reg.x = data,
            2 => self.reg.y = data,
            _ => panic!("register type must be between 0 to 2"),
        };

        self.set_mov_flag(data);
    }

    fn mov_load_x_idx_indirect(&mut self, opcode: u8) -> () {
        let reg_type = match opcode {
            0xF4 => 0,
            0xFB => 2,
            _ => panic!("unexpected opcode: {}", opcode),
        };

        let addr = self.read_from_pc().wrapping_add(self.reg.x);
        let data = self.read_from_page(addr);
        self.cycles(1);

        match reg_type {
            0 => self.reg.a = data,
            2 => self.reg.y = data,
            _ => panic!("register type must be between 0 to 2"),
        }

        self.set_mov_flag(data);
    }

    fn mov_load_y_idx_indirect(&mut self, _opcode: u8) -> () {
        let addr = self.read_from_pc().wrapping_add(self.reg.y);
        let data = self.read_from_page(addr);
        self.cycles(1);
        self.reg.x = data;

        self.set_mov_flag(data);
    }

    fn mov_load_addr(&mut self, opcode: u8) -> () {
        let reg_type = match opcode {
            0xE5 => 0,
            0xE9 => 1,
            0xEC => 2,
            _ => panic!("unexpected opcode: {}", opcode),
        };

        let lower = self.read_from_pc() as u16;
        let upper = self.read_from_pc() as u16;
        let addr = (upper << 8) | lower;
        let data = self.read_ram(addr);

        match reg_type {
            0 => self.reg.a = data,
            1 => self.reg.x = data,
            2 => self.reg.y = data,
            _ => panic!("register type must be between 0 to 2"),
        }

        self.set_mov_flag(data);
    }

    fn mov_load_x_idx_addr(&mut self, _opcode: u8) -> () {
        let lower = self.read_from_pc() as u16;
        let upper = self.read_from_pc() as u16;
        let addr = (upper << 8) | lower;    
        let addr = addr.wrapping_add(self.reg.x as u16);
        self.cycles(1);

        let data = self.read_ram(addr);

        self.reg.a = data;
        self.set_mov_flag(data);
    }

    fn mov_load_y_idx_addr(&mut self, _opcode: u8) -> () {
        let lower = self.read_from_pc() as u16;
        let upper = self.read_from_pc() as u16;
        let addr = (upper << 8) | lower;    
        let addr = addr.wrapping_add(self.reg.y as u16);
        self.cycles(1);

        let data = self.read_ram(addr);

        self.reg.a = data;
        self.set_mov_flag(data);
    }

    fn mov_load_x_indirect(&mut self, _opcode: u8) -> () {
        let data = self.read_from_page(self.reg.x);
        self.cycles(1);

        self.reg.a = data;
        self.set_mov_flag(data);
    }

    fn mov_load_x_indirect_inc(&mut self, _opcode: u8) -> () {
        self.cycles(1);
        let data = self.read_from_page(self.reg.x);
        self.reg.x = self.reg.x.wrapping_add(1);
        self.cycles(1);

        self.reg.a = data;
        self.set_mov_flag(data);
    }

    fn mov_load_y_ind_ind(&mut self, _opcode: u8) -> () {
        let base_addr = self.read_from_pc();
        let lower = self.read_from_page(base_addr) as u16;
        let upper = self.read_from_page(base_addr.wrapping_add(1)) as u16;        
        let addr = (upper << 8) | lower;
        let addr = addr.wrapping_add(self.reg.y as u16);
        self.cycles(1);
        let data = self.read_ram(addr);

        self.reg.a = data;
        self.set_mov_flag(data);
    }

    fn mov_load_x_ind_ind(&mut self, _opcode: u8) -> () {
        let base_addr = self.read_from_pc().wrapping_add(self.reg.x);        
        self.cycles(1);
        let lower = self.read_from_page(base_addr) as u16;
        let upper = self.read_from_page(base_addr.wrapping_add(1)) as u16;
        let addr = (upper << 8) | lower;
        let data = self.read_ram(addr);

        self.reg.a = data;
        self.set_mov_flag(data);
    }

    fn mov_load_word(&mut self, _opcode: u8) -> () {
        let addr = self.read_from_pc();
        let word_lower = self.read_from_page(addr) as u16;
        let word_upper = self.read_from_page(addr.wrapping_add(1)) as u16;
        let word = (word_upper << 8) | word_lower;

        self.reg.set_ya(word);
        self.cycles(1);

        let is_negative = (word & 0x8000) != 0;
        let is_zero = word == 0;
        self.reg.psw.set_sign(is_negative);
        self.reg.psw.set_zero(is_zero);
    }

    fn set_mov_flag(&mut self, data: u8) -> () {
        let is_negative = (data & 0x80) != 0;
        let is_zero = data == 0;
        self.reg.psw.set_zero(is_zero);
        self.reg.psw.set_sign(is_negative);
    }

    fn mov_store_dp_imm(&mut self, _opcode: u8) -> () {
        let imm = self.read_from_pc();
        let addr = self.read_from_pc();
        let _ = self.read_from_page(addr);        

        self.write_to_page(addr, imm);        
    }

    fn mov_store_dp_dp(&mut self, _opcode: u8) -> () {
        let bb = self.read_from_pc();
        let b = self.read_from_page(bb);
        let aa = self.read_from_pc();        
        
        self.write_to_page(aa, b);
    }

    fn mov_store_dp_reg(&mut self, opcode: u8) -> () {
        let reg_type = match opcode {
            0xC4 => 0,
            0xCB => 2,
            0xD8 => 1,
            _ => panic!("unexpected opcode: {}", opcode),
        };

        let addr = self.read_from_pc();
        let _ = self.read_from_page(addr);
        let data = match reg_type {
            0 => self.reg.a,
            1 => self.reg.x,
            2 => self.reg.y,
            _ => panic!("register type must be between 0 to 2"),
        };

        self.write_to_page(addr, data);
    }

    fn mov_store_x_idx_indirect(&mut self, opcode: u8) -> () {
        let reg_type = match opcode {
            0xD4 => 0,
            0xDB => 2,
            _ => panic!("unexpected opcode: {}", opcode),
        };

        let addr = self.read_from_pc().wrapping_add(self.reg.x);
        self.cycles(1);
        let _ = self.read_from_page(addr);
        let data = match reg_type {
            0 => self.reg.a,
            2 => self.reg.y,    
            _ => panic!("register type must be 0 or 2"),
        };
        
        self.write_to_page(addr, data);
    }

    fn mov_store_y_idx_indirect(&mut self, _opcode: u8) -> () {
        let addr = self.read_from_pc().wrapping_add(self.reg.y);
        self.cycles(1);
        let _ = self.read_from_page(addr);
        let data = self.reg.x;
        
        self.write_to_page(addr, data);
    }

    fn mov_store_addr(&mut self, opcode: u8) -> () {
        let reg_type = match opcode {
            0xC5 => 0,
            0xC9 => 1,
            0xCC => 2,
            _ => panic!("unexpected opcode: {}", opcode),
        };

        let lower = self.read_from_pc() as u16;
        let upper = self.read_from_pc() as u16;
        let addr = (upper << 8) | lower;
        let _ = self.read_ram(addr);

        let data = match reg_type { 
            0 => self.reg.a,
            1 => self.reg.x,
            2 => self.reg.y,    
            _ => panic!("register type must be between 0 to 2"),
        };

        self.write_ram(addr, data);
    }

    fn mov_store_x_idx_addr(&mut self, _opcode: u8) -> () {
        let lower = self.read_from_pc() as u16;
        let upper = self.read_from_pc() as u16;
        let addr = (upper << 8) | lower;
        let addr = addr.wrapping_add(self.reg.x as u16);
        self.cycles(1);
        let _ = self.read_ram(addr);
        
        self.write_ram(addr, self.reg.a);
    }

    fn mov_store_y_idx_addr(&mut self, _opcode: u8) -> () {
        let lower = self.read_from_pc() as u16;
        let upper = self.read_from_pc() as u16;
        let addr = (upper << 8) | lower;
        let addr = addr.wrapping_add(self.reg.y as u16);
        self.cycles(1);
        let _ = self.read_ram(addr);

        self.write_ram(addr, self.reg.a);
    }

    fn mov_store_x_indirect_inc(&mut self, _opcode: u8) -> () {
        self.cycles(1);
        self.write_to_page(self.reg.x, self.reg.a);
        self.reg.x = self.reg.x.wrapping_add(1);
        self.cycles(1);
    }

    fn mov_store_x_indirect(&mut self, _opcode: u8) -> () {
        self.cycles(1);
        self.read_from_page(self.reg.x);
        self.write_to_page(self.reg.x, self.reg.a);        
    }

    fn mov_store_x_ind_ind(&mut self, _opcode: u8) -> () {
        let base_addr = self.read_from_pc().wrapping_add(self.reg.x);
        self.cycles(1);
        let lower = self.read_from_page(base_addr) as u16;
        let upper = self.read_from_page(base_addr.wrapping_add(1)) as u16;
        let addr = (upper << 8) | lower;
        let _ = self.read_ram(addr);

        self.write_ram(addr, self.reg.a);
    }

    fn mov_store_y_ind_ind(&mut self, _opcode: u8) -> () {
        let base_addr = self.read_from_pc();
        let lower = self.read_from_page(base_addr) as u16;
        let upper = self.read_from_page(base_addr) as u16;
        let addr = (upper << 8) | lower;        
        let addr = addr.wrapping_add(self.reg.y as u16);
        self.cycles(1);
        let _ = self.read_ram(addr);
        
        self.write_ram(addr, self.reg.a);
    }

    fn mov_store_word(&mut self, _opcode: u8) -> () {
        let addr = self.read_from_pc();
        let _ = self.read_from_page(addr);
        self.write_to_page(addr, self.reg.a);
        self.write_to_page(addr.wrapping_add(1), self.reg.y);
    }

    fn push(&mut self, opcode: u8) -> () {
        let reg_type = match opcode {
            0x0D => 3,
            0x2D => 0,
            0x4D => 1,
            0x6D => 2,
            _ => panic!("unexpected opcode: {}", opcode),
        };

        self.cycles(1);

        let data = match reg_type {
            0 => self.reg.a,
            1 => self.reg.x,
            2 => self.reg.y,
            3 => self.reg.psw.get(),
            _ => panic!("register type must be between 0 to 3"),
        };
        self.write_to_stack(data);
        self.reg.sp = self.reg.sp.wrapping_sub(1);
        self.cycles(1);
    }

    fn pop(&mut self, opcode: u8) -> () {
        let reg_type = match opcode { 
            0x8E => 3,
            0xAE => 0,
            0xCE => 1,
            0xEE => 2,
            _ => panic!("unexpected opcode: {}", opcode),
        };

        self.reg.sp = self.reg.sp.wrapping_add(1);
        self.cycles(1);
        let data = self.read_from_stack();

        match reg_type {
            0 => self.reg.a = data,
            1 => self.reg.x = data,
            2 => self.reg.y = data,
            3 => self.reg.psw.set(data),
            _ => panic!("register type must be between 0 to 3"),
        };
        self.cycles(1);
    }

    fn nop(&mut self, _opcode: u8) -> () {
        self.cycles(1);
    }

    fn sleep_or_stop(&mut self, _opcode: u8) -> () {        
        self.is_stopped = true;
        self.cycles(2);
    }

    fn clrp(&mut self, _opcode: u8) -> () {
        self.reg.psw.negate_page();
        self.cycles(1);
    }

    fn setp(&mut self, _opcode: u8) -> () {
        self.reg.psw.assert_page();
        self.cycles(1);
    }

    fn ei(&mut self, _opcode: u8) -> () {
        self.reg.psw.assert_interrupt();
        self.cycles(2);
    }

    fn di(&mut self, _opcode: u8) -> () {
        self.reg.psw.negate_overflow();
        self.cycles(2);
    }

    fn set1(&mut self, opcode: u8) -> () {
        let addr = self.read_from_pc();
        let shamt = (opcode >> 5) & 1;
        let x = self.read_from_page(addr) | (1 << shamt);

        self.write_to_page(addr, x);        
    }

    fn clr1(&mut self, opcode: u8) -> () {
        let addr = self.read_from_pc();
        let shamt = (opcode >> 5) & 1;
        let x = self.read_from_page(addr) & !(1 << shamt);

        self.write_to_page(addr, x);
    }

    fn not1(&mut self, _opcode: u8) -> () {
        let (addr, bit_idx) = self.addr_and_idx();
        let data = self.read_ram(addr);
        let ret = data ^ (1 << bit_idx);
        
        self.write_ram(addr, ret);
    }

    fn mov1_to_mem(&mut self, _opcode: u8) -> () {
        let (addr, bit_idx) = self.addr_and_idx();        
        let data = self.read_ram(addr);
        self.cycles(1);
        let ret = 
            if self.reg.psw.carry() {
                data | (1 << bit_idx)                
            } else {
                data & !(1 << bit_idx)
            };
        
        self.write_ram(addr, ret);        
    }

    fn mov1_to_psw(&mut self, _opcode: u8) -> () {
        let (addr, bit_idx) = self.addr_and_idx();
        let data = self.read_ram(addr);
        let carry = (data >> bit_idx) & 1;
        self.reg.psw.set_carry(carry == 1);
    }

    fn or1(&mut self, opcode: u8) -> () {
        let rev = (opcode & 0x20)  != 0;
        let (addr, bit_idx) = self.addr_and_idx();
        let data = self.read_ram(addr);
        let bit = ((data >> bit_idx) & 1) == 1;        
        let ret = self.reg.psw.carry () | (rev ^ bit);
        self.cycles(1);

        self.reg.psw.set_carry(ret);
    }

    fn and1(&mut self, opcode: u8) -> () {
        let rev = (opcode & 0x20) != 0;
        let (addr, bit_idx) = self.addr_and_idx();
        let data = self.read_ram(addr);
        let bit = ((data >> bit_idx) & 1) == 1;        
        let ret = self.reg.psw.carry () & (rev ^ bit);
        
        self.reg.psw.set_carry(ret);
    }

    fn eor1(&mut self, _opcode: u8) -> () {
        let (addr, bit_idx) = self.addr_and_idx();
        let data = self.read_ram(addr);
        let bit = ((data >> bit_idx) & 1) == 1;        
        let ret = self.reg.psw.carry () ^ bit;
        self.cycles(1);

        self.reg.psw.set_carry(ret);
    }

    fn clrc(&mut self, _opcode: u8) -> () {
        self.cycles(1);
        self.reg.psw.set_carry(false);
    }

    fn setc(&mut self, _opcode: u8) -> () {
        self.cycles(1);
        self.reg.psw.set_carry(true);
    }

    fn notc(&mut self, _opcode: u8) -> () {
        self.cycles(2);
        self.reg.psw.set_carry(!self.reg.psw.carry());
    }

    fn clrv(&mut self, _opcode: u8) -> () {
        self.cycles(1);
        self.reg.psw.set_overflow(false);
        self.reg.psw.set_half(false);
    }

    fn addr_and_idx(&mut self) -> (u16, u8) {
        let lower = self.read_from_pc() as u16;
        let upper = self.read_from_pc() as u16;
        let addr = (upper << 8) | lower;
        let idx = (addr >> 13) as u8;
        let addr = addr & 0x1FFF;

        (addr, idx)
    }

    fn branch_by_psw(&mut self, opcode: u8) -> () {
        let rr = self.read_from_pc() as u16;        
        let flag_type = opcode & !(0x20);
        let require_true = (opcode & 0x20) != 0;
        let flag = match flag_type {
            0x10 => self.reg.psw.sign(),
            0x50 => self.reg.psw.overflow(),
            0x90 => self.reg.psw.carry(),
            0xD0 => self.reg.psw.zero(),
            _ => panic!("flag type must be between 0x10, 0x50, 0x90, 0xD0. actual: {:#04x}", flag_type),
        };

        let branch = flag == require_true;        
        if branch { 
            self.cycles(2);
            let offset = if (rr & 0x80) != 0 { rr | 0xFF00 } else { rr };
            self.reg.pc = self.reg.pc.wrapping_add(offset);
         }
    }

    fn branch_by_mem_bit(&mut self, opcode: u8) -> () {
        let addr = self.read_from_pc();
        let data = self.read_from_page(addr);        
        let offset = self.read_from_pc() as u16;
        let offset = if (offset & 0x80) != 0 { 0xFF00 | offset } else { offset };

        let require_true = (opcode & 0x10) == 0;
        let bit_idx = (opcode >> 5) & 0x7;
        self.cycles(1);

        let bit = ((data >> bit_idx) & 1) == 1;
        let is_branch = bit == require_true;

        if is_branch {
            self.cycles(2);
            self.reg.pc = self.reg.pc.wrapping_add(offset);
        }
    }

    fn cbne(&mut self, opcode: u8) -> () {
        let aa = self.read_from_pc();
        let require_x = match opcode {
            0x2E => false,
            0xDE => true,
            _ => panic!("expected opcodes are 0x2E and 0xDE. actual: {:#04x}", opcode),
        };
        let addr = 
            if require_x { self.cycles(1); aa.wrapping_add(self.reg.x) }
            else { aa };
        let data = self.read_from_page(addr);
        let rr = self.read_from_pc() as u16;
        self.cycles(1);

        if self.reg.a != data {
            let offset = if (rr & 0x80) != 0 { 0xFF00 | rr } else { rr };
            self.reg.pc = self.reg.pc.wrapping_add(offset);
            self.cycles(2);
        }
    }

    fn dbnz_y(&mut self, _opcode: u8) -> () {
        let rr = self.read_from_pc() as u16;
        self.reg.y = self.reg.y.wrapping_sub(1);
        self.cycles(2);
        
        if self.reg.y != 0 {
            self.cycles(2);
            let offset = if (rr & 0x80) != 0 { 0xFF00 | rr } else { rr };
            self.reg.pc = self.reg.pc.wrapping_add(offset);
        }
    }

    fn dbnz_data(&mut self, _opcode: u8) -> () {
        let addr = self.read_from_pc();
        let rr = self.read_from_pc() as u16;
        let data = self.read_from_page(addr).wrapping_sub(1);        
        self.write_to_page(addr, data);        
        self.cycles(1);

        if data != 0 {
            self.cycles(2);
            let offset = if (rr & 0x80) != 0 { 0xFF00 | rr } else { rr };
            self.reg.pc = self.reg.pc.wrapping_add(offset);
        }
    }

    fn bra(&mut self, _opcode: u8) -> () {
        let rr = self.read_from_pc() as u16;
        self.cycles(2);
        let offset = if (rr & 0x80) != 0 { 0xFF00 | rr } else { rr };
        self.reg.pc = self.reg.pc.wrapping_add(offset);
    }

    fn jmp_abs(&mut self, _opcode: u8) -> () {
        let lower = self.read_from_pc() as u16;
        let upper = self.read_from_pc() as u16;
        let addr = (upper << 8) | lower;

        self.reg.pc = addr;
    }

    fn jmp_abs_x(&mut self, _opcode: u8) -> () {
        let lower = self.read_from_pc() as u16;
        let upper = self.read_from_pc() as u16;
        let addr = (upper << 8) | lower;
        let addr = addr.wrapping_add(self.reg.x as u16);
        self.cycles(1);

        let dst_lower = self.read_ram(addr) as u16;
        let dst_upper = self.read_ram(addr.wrapping_add(1)) as u16;

        self.reg.pc = (dst_upper << 8) | dst_lower;
    }
    
    fn call(&mut self, _opcode: u8) -> () {
        let dst_lower = self.read_from_pc() as u16;
        let dst_upper = self.read_from_pc() as u16;
        let dst = (dst_upper << 8) | dst_lower;

        self.cycles(1);
        self.write_to_stack((self.reg.pc >> 8) as u8);
        self.reg.sp = self.reg.sp.wrapping_sub(1);
        self.cycles(1);
        self.write_to_stack(self.reg.pc as u8);
        self.reg.sp = self.reg.sp.wrapping_sub(1);
        self.cycles(1);
    
        self.reg.pc = dst;
    }

    fn tcall(&mut self, opcode: u8) -> () {
        let pc_lower = self.reg.pc as u8;
        let pc_upper = (self.reg.pc >> 8) as u8;
        self.write_to_stack(pc_upper);
        self.reg.sp = self.reg.sp.wrapping_sub(1);
        self.cycles(1);

        self.write_to_stack(pc_lower);
        self.reg.sp = self.reg.sp.wrapping_sub(1);
        self.cycles(1);

        let offset = ((opcode >> 4) << 1) as u16;        
        let addr = 0xFFDE - offset;        
        self.cycles(1);

        let next_pc_lower = self.read_ram(addr) as u16;
        let next_pc_upper = self.read_ram(addr.wrapping_add(1)) as u16;
        let next_pc = (next_pc_upper << 8) | next_pc_lower;

        self.reg.pc = next_pc;
    }

    fn pcall(&mut self, _opcode: u8) -> () {        
        let lower = self.read_from_pc() as u16;
        let next_pc = 0xFF00 | lower;

        let pc_lower = self.reg.pc as u8;
        let pc_upper = (self.reg.pc >> 8) as u8;
        self.write_to_stack(pc_upper);
        self.reg.sp = self.reg.sp.wrapping_sub(1);
        self.cycles(1);

        self.write_to_stack(pc_lower);
        self.reg.sp = self.reg.sp.wrapping_sub(1);
        self.cycles(1);

        self.reg.pc = next_pc;
    }

    fn ret(&mut self, _opcode: u8) -> () {
        let partial_pcs: Vec<u8> = (0..2).map(|_| {
            self.reg.sp = self.reg.sp.wrapping_add(1);
            self.cycles(1);
            let partial_pc = self.read_from_stack();            

            partial_pc
        }).collect();
        
        let lower = partial_pcs[0] as u16;
        let upper = partial_pcs[1] as u16;
        let next_pc = (upper << 8) | lower;

        self.reg.pc = next_pc;
    }

    fn ret1(&mut self, _opcode: u8) -> () {
        let psw = self.read_from_stack();
        self.reg.sp = self.reg.sp.wrapping_add(1);
        self.reg.psw.set(psw);
        
        let lower_pc = self.read_from_stack() as u16;
        self.reg.sp = self.reg.sp.wrapping_add(1);
        self.cycles(1);

        let upper_pc = self.read_from_stack() as u16;
        self.reg.sp = self.reg.sp.wrapping_add(1);
        self.cycles(1);

        self.reg.pc = (upper_pc << 8) | (lower_pc)
    }

    fn brk(&mut self, _opcode: u8) -> () {
        let lower = self.read_ram(0xFFDE) as u16;
        let upper = self.read_ram(0xFFDF) as u16;
        let next_pc = (upper << 8) | lower;

        let pc_lower = self.reg.pc as u8;
        let pc_upper = (self.reg.pc >> 8) as u8;
        self.write_to_stack(pc_upper);
        self.reg.sp = self.reg.sp.wrapping_sub(1);
        self.cycles(1);
        self.write_to_stack(pc_lower);
        self.reg.sp = self.reg.sp.wrapping_sub(1);
        self.cycles(1);

        self.write_to_stack(self.reg.psw.get());
        self.reg.sp = self.reg.sp.wrapping_sub(1);
        
        self.reg.pc = next_pc;
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
            _ if opcode == 0xC8 => cmp,
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

    fn alu_dp(&mut self, opcode: u8) -> () {
        let op = Spc700::fetch_alu_op(opcode);
        let upper = (opcode >> 4) & 0x0F;
        let lower = opcode & 0x0F;
        let reg_type = match (upper, lower) {
            (upper, 0x4) if upper % 2 == 0 => 0,
            (0x3, 0xE) => 1,
            (0x7, 0xE) => 2,
            _ => panic!("unexpected opcode: {}", opcode),
        };

        let addr = self.read_from_pc();        
        let b = self.read_from_page(addr);
        let a = match reg_type {
            0 => self.reg.a,
            1 => self.reg.x,
            2 => self.reg.y,
            _ => panic!("from register types must be between 0 to 2"),
        };         

        self.reg.a = op(&mut self.reg, a, b);        
    }

    fn alu_addr(&mut self, opcode: u8) -> () {        
        let op = Spc700::fetch_alu_op(opcode);
        let op_upper = (opcode >> 4) & 0x0F;
        let op_lower = opcode & 0x0F;
        let reg_type = match (op_upper, op_lower) {
            (upper, 0x5) if upper % 2 == 0 => 0,
            (0x1, 0xE) => 1,
            (0x5, 0xE) => 2,
            _ => panic!("unexpected opcode: {}", opcode),
        };
        
        let lower = self.read_from_pc() as u16;
        let upper = self.read_from_pc() as u16;
        let addr = (upper << 8) | lower;

        let b = self.read_ram(addr);
        let a = match reg_type {
            0 => self.reg.a,
            1 => self.reg.x,
            2 => self.reg.y,
            _ => panic!("from register types must be between 0 to 2"),
        };        
        
        self.reg.a = op(&mut self.reg, a, b);        
    }

    fn alu_indirect_x(&mut self, opcode: u8) -> () {
        let op = Spc700::fetch_alu_op(opcode);
        let x = self.reg.x;
        self.cycles(1);

        let a = self.reg.a;
        let b = self.read_from_page(x);
        let data = op(&mut self.reg, a, b);

        self.reg.a = data;        
    }

    fn alu_x_idx_indirect(&mut self, opcode: u8) -> () {
        let op = Spc700::fetch_alu_op(opcode);
        let addr = self.read_from_pc().wrapping_add(self.reg.x);
        self.cycles(1);
        
        let a = self.reg.a;
        let b = self.read_from_page(addr);        
        let ret = op(&mut self.reg, a, b);

        self.reg.a = ret;        
    }

    fn alu_x_idx_addr(&mut self, opcode: u8) -> () {
        let op = Spc700::fetch_alu_op(opcode);
        let lower = self.read_from_pc() as u16;
        let upper = self.read_from_pc() as u16;
        let addr = (upper << 8) | lower;
        let addr = addr.wrapping_add(self.reg.x as u16);
        self.cycles(1);
        let data = self.read_ram(addr);

        let a = self.reg.a;
        let ret = op(&mut self.reg, a, data);

        self.reg.a = ret;
    }

    fn alu_x_ind_ind(&mut self, opcode: u8) -> () {
        let op = Spc700::fetch_alu_op(opcode);
        let base_addr = self.read_from_pc().wrapping_add(self.reg.x);        
        self.cycles(1);
        let lower = self.read_from_page(base_addr) as u16;
        let upper = self.read_from_page(base_addr.wrapping_add(1)) as u16;
        let addr = (upper << 8) | lower;
        let data = self.read_ram(addr);

        let a = self.reg.a;
        let ret = op(&mut self.reg, a, data);
        
        self.reg.a = ret;
    }

    fn alu_y_ind_ind(&mut self, opcode: u8) -> () {
        let op = Spc700::fetch_alu_op(opcode);
        let base_addr = self.read_from_pc();
        let lower = self.read_from_page(base_addr) as u16;
        let upper = self.read_from_page(base_addr.wrapping_add(1)) as u16;        
        let addr = (upper << 8) | lower;
        let addr = addr.wrapping_add(self.reg.y as u16);
        self.cycles(1);
        let data = self.read_ram(addr);
        let a = self.reg.a;
        
        let ret = op(&mut self.reg, a, data);
        
        self.reg.a = ret;
    }

    fn alu_y_idx_addr(&mut self, opcode: u8) -> () {
        let op = Spc700::fetch_alu_op(opcode);
        let lower = self.read_from_pc() as u16;
        let upper = self.read_from_pc() as u16;
        let addr = (upper << 8) | lower;
        let addr = addr.wrapping_add(self.reg.y as u16);
        self.cycles(1);
        let data = self.read_ram(addr);
        let a = self.reg.a;

        let ret = op(&mut self.reg, a, data);

        self.reg.a = ret;
    }

    fn alu_x_y(&mut self, opcode: u8) -> () {
        let op = Spc700::fetch_alu_op(opcode);
        let op_upper = (opcode >> 4) & 0x0F;
        let is_cmp_op = op_upper == 0x6 || op_upper == 0x7;

        self.cycles(1);
        let x_data = self.read_from_page(self.reg.x);        
        let y_data = self.read_from_page(self.reg.y);
        let ret = op(&mut self.reg, x_data, y_data);        

        if !is_cmp_op {
            self.write_to_page(self.reg.x, ret);
        } else {
            self.cycles(1)
        }
    }

    // TODO 取得及び書き込むレジスタの種類を数字ではなくenum値で表現するように変更する
    fn alu_imm(&mut self, opcode: u8) -> () {
        let op = Spc700::fetch_alu_op(opcode);
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

        let b = self.read_from_pc();
        let ret = op(&mut self.reg, a, b);
        match reg_type {
            0 => self.reg.a = ret,
            1 => self.reg.x = ret,
            2 => self.reg.y = ret,
            _ => panic!("from register types must be between 0 to 2"),
        };
    }

    fn alu_dp_imm(&mut self, opcode: u8) -> () {
        let op = Spc700::fetch_alu_op(opcode);
        let op_upper = (opcode >> 4) & 0x0F;
        let is_cmp_op = op_upper == 0x6 || op_upper == 0x7; 

        let imm = self.read_from_pc();
        let addr = self.read_from_pc();
        let data = self.read_from_page(addr);
        let ret = op(&mut self.reg, data, imm);

        if !is_cmp_op {
            self.write_to_page(addr, ret);
        } else {
            self.cycles(1);
        }
    }

    fn alu_dp_dp(&mut self, opcode: u8) -> () {
        let op = Spc700::fetch_alu_op(opcode);
        let op_upper = (opcode >> 4) & 0x0F;
        let is_cmp_op = op_upper == 0x6 || op_upper == 0x7; 

        let bb = self.read_from_pc();
        let b = self.read_from_page(bb);
        let aa = self.read_from_pc();        
        let a = self.read_from_page(aa);

        let ret = op(&mut self.reg, a, b);

        if !is_cmp_op {
            self.write_to_page(aa, ret);
        } else {
            self.cycles(1);
        } 
    }

    fn shift_acc(&mut self, opcode: u8) -> () {
        let op = Spc700::fetch_shift_op(opcode);

        self.cycles(1);
        let ret = self.shift(opcode, self.reg.a, op);
        
        self.reg.a = ret;        
    }

    fn shift_dp(&mut self, opcode: u8) -> () {
        let op = Spc700::fetch_shift_op(opcode);

        let addr = self.read_from_pc();
        let data = self.read_from_page(addr);        
        let ret = self.shift(opcode, data, op);

        self.write_to_page(addr, ret);
    }

    fn shift_x_idx_indirect(&mut self, opcode: u8) -> () {
        let op = Spc700::fetch_shift_op(opcode);

        let addr = self.read_from_pc().wrapping_add(self.reg.x);
        self.cycles(1);

        let data = self.read_from_page(addr);
        let ret = self.shift(opcode, data, op);

        self.write_to_page(addr, ret);
    }

    fn shift_addr(&mut self, opcode: u8) -> () {
        let op = Spc700::fetch_shift_op(opcode);

        let lower = self.read_from_pc() as u16;
        let upper = self.read_from_pc() as u16;
        let addr = (upper << 8) | lower;
        let data = self.read_ram(addr);

        let ret = self.shift(opcode, data, op);

        self.write_ram(addr, ret);
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

    fn inc_dec_reg(&mut self, opcode: u8) -> () {
        self.cycles(1);
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
    }

    fn inc_dec_dp(&mut self, opcode: u8) -> () {
        let is_inc = (opcode & 0x20) != 0;
        let addr = self.read_from_pc();
        let data = self.read_from_page(addr);
        let ret =
            if is_inc { data.wrapping_add(1) }
            else      { data.wrapping_sub(1) };

        self.write_to_page(addr, ret);
        self.set_inc_dec_flag(ret);
    }

    fn inc_dec_x_idx_indirect(&mut self, opcode: u8) -> () {
        let is_inc = (opcode & 0x20) != 0;
        let addr = self.read_from_pc().wrapping_add(self.reg.x);
        self.cycles(1);

        let data = self.read_from_page(addr);
        let ret = 
            if is_inc { data.wrapping_add(1) }
            else      { data.wrapping_sub(1) };

        self.write_to_page(addr, ret);
        self.set_inc_dec_flag(ret);
    }

    fn inc_dec_addr(&mut self, opcode: u8) -> () {
        let is_inc = (opcode & 0x20) != 0;
        let lower = self.read_from_pc() as u16;
        let upper = self.read_from_pc() as u16;
        let addr = (upper << 8) | lower;
        let data = self.read_ram(addr);

        let ret =
            if is_inc { data.wrapping_add(1) }
            else      { data.wrapping_sub(1) };

        self.write_ram(addr, ret);
        self.set_inc_dec_flag(ret);
    }
    
    fn set_inc_dec_flag(&mut self, data: u8) -> () {
        let is_neg = (data & 0x80) != 0;
        let is_zero = data == 0;

        self.reg.psw.set_sign(is_neg);
        self.reg.psw.set_zero(is_zero);
    }

    fn addw(&mut self, _opcode: u8) -> () {
        self.reg.psw.negate_carry();

        let (ya, word) = self.get_word_operands();        
        let ret_lower = adc(&mut self.reg, ya as u8, word as u8) as u16;
        let ret_upper = adc(&mut self.reg, (ya >> 8) as u8, (word >> 8) as u8) as u16;
        let ret = (ret_upper << 8) | ret_lower;
                
        self.cycles(1);
        self.reg.set_ya(ret);

        self.reg.psw.set_zero(ret == 0);
    }

    fn subw(&mut self, _opcode: u8) -> () {
        self.reg.psw.assert_carry();

        let (ya, word) = self.get_word_operands();
        let ret_lower = sbc(&mut self.reg, ya as u8, word as u8) as u16;
        let ret_upper = sbc(&mut self.reg, (ya >> 8) as u8, (word >> 8) as u8) as u16;
        let ret = (ret_upper << 8) | ret_lower;

        self.cycles(1);
        self.reg.set_ya(ret);

        self.reg.psw.set_zero(ret == 0);
    }

    fn cmpw(&mut self, _opcode: u8) -> () {
        let (ya, word) = self.get_word_operands();
        let ret = (ya as i32) - (word as i32);

        self.reg.psw.set_sign((ret & 0x8000) != 0);
        self.reg.psw.set_zero(ret as u16 == 0);
        self.reg.psw.set_carry(ret >= 0);
    }

    fn inc_dec_word(&mut self, opcode: u8) -> () {
        let addr = self.read_from_pc();
        let upper = (opcode >> 4) & 0x0F;
        let operand = match upper {
            0x1 => 0xFFFF,
            0x3 => 0x0001,
            _   => panic!("unexpected opcode: {}", opcode),
        };

        let word_lower = self.read_from_page(addr) as u16;                
        let lower_result = word_lower.wrapping_add(operand); 
        let lower_carry = lower_result >> 8;
        self.write_to_page(addr, lower_result as u8);

        let word_upper = self.read_from_page(addr.wrapping_add(1)) as u16;
        let upper_result = word_upper.wrapping_add(lower_carry);
        
        self.write_to_page(addr.wrapping_add(1), upper_result as u8);

        let result = (upper_result << 8) | lower_result;
        self.reg.psw.set_zero(result == 0);
        self.reg.psw.set_sign((result & 0x8000) != 0);
    }

    fn div(&mut self, _opcode: u8) -> () {
        self.cycles(11);
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
    }

    fn mul(&mut self, _opcode: u8) -> () {
        self.cycles(8);
        let ya = (self.reg.y as u16) * (self.reg.a as u16);
        self.reg.set_ya(ya);

        self.reg.psw.set_zero(self.reg.y == 0);
        self.reg.psw.set_sign((self.reg.y & 0x80) != 0);
    }

    fn get_word_operands(&mut self) -> (u16, u16) {
        let addr = self.read_from_pc();
        let word_lower = self.read_from_page(addr) as u16;
        let word_upper = self.read_from_page(addr.wrapping_add(1)) as u16;
        let word = (word_upper << 8) | word_lower;
        let ya = self.reg.ya();

        (ya, word)
    }

    fn daa(&mut self, _opcode: u8) -> () {
        self.cycles(2);
        if self.reg.psw.carry() || self.reg.a > 0x99 {
            self.reg.a = self.reg.a.wrapping_add(0x60);
            self.reg.psw.assert_carry();
        }
        if self.reg.psw.half() || (self.reg.a & 0x0F) > 0x09 {
            self.reg.a = self.reg.a.wrapping_add(0x06);            
        }

        self.reg.psw.set_zero(self.reg.a == 0);
        self.reg.psw.set_sign((self.reg.a & 0x80) != 0);
    }

    fn das(&mut self, _opcode: u8) -> () {
        self.cycles(2);
        if !self.reg.psw.carry() || self.reg.a > 0x99 {
            self.reg.a = self.reg.a.wrapping_sub(0x60);
            self.reg.psw.set_carry(false);
        }
        if !self.reg.psw.half() || (self.reg.a & 0x0F) > 0x09 {
            self.reg.a = self.reg.a.wrapping_sub(0x06);
        }

        self.reg.psw.set_zero(self.reg.a == 0);
        self.reg.psw.set_sign((self.reg.a & 0x80) != 0);
    }

    fn xcn(&mut self, _opcode: u8) -> () {
        self.cycles(4);
        self.reg.a = (self.reg.a >> 4) | ((self.reg.a & 0x0F) << 4);

        self.reg.psw.set_zero(self.reg.a == 0);
        self.reg.psw.set_sign((self.reg.a & 0x80) != 0); 
    }

    fn tclr1(&mut self, _opcode: u8) -> () {
        let lower = self.read_from_pc() as u16;
        let upper = self.read_from_pc() as u16;
        let addr = (upper << 8) | lower;
        let data = self.read_ram(addr);
        let ret = data & !self.reg.a;
        
        self.read_ram(addr);
        let cmp = self.reg.a.wrapping_sub(data);
        self.reg.psw.set_zero(cmp == 0);
        self.reg.psw.set_sign((cmp & 0x80) != 0);        

        self.write_ram(addr, ret);

    }

    fn tset1(&mut self, _opcode: u8) -> () {
        let lower = self.read_from_pc() as u16;
        let upper = self.read_from_pc() as u16;
        let addr = (upper << 8) | lower;
        let data = self.read_ram(addr);
        let ret = data | self.reg.a;

        self.read_ram(addr);
        let cmp = self.reg.a.wrapping_sub(data);
        self.reg.psw.set_zero(cmp == 0);
        self.reg.psw.set_sign((cmp & 0x80) != 0);        

        self.write_ram(addr, ret);
    }

    fn read_from_pc(&mut self) -> u8 {
        let addr = self.reg.pc;
        self.reg.inc_pc(1);
    
        self.read_ram(addr)
    }

    fn read_from_stack(&mut self) -> u8 {
        let addr = (self.reg.sp as u16) | 0x0100;    
        self.read_ram(addr)
    }

    fn read_from_page(&mut self, addr: u8) -> u8 {
        let addr = (addr as u16) | (if self.reg.psw.page() { 0x0100 } else { 0x0000 });
        self.read_ram(addr)
    }    

    fn read_ram(&mut self, addr: u16) -> u8 {
        self.cycles(1);
        self.ram.read(addr, &mut self.dsp, &mut self.timer)
    }    

    fn write_to_page(&mut self, addr: u8, data: u8) -> () {
        let addr = (addr as u16) | (if self.reg.psw.page() { 0x0100 } else { 0x0000 });
        self.write_ram(addr, data)
    }

    fn write_to_stack(&mut self, data: u8) -> () {
        let addr = (self.reg.sp as u16) | 0x0100;        
        self.write_ram(addr, data);        
    }

    pub fn write_ram(&mut self, addr: u16, data: u8) -> () {
        self.cycles(1);
        self.ram.write(addr, data, &mut self.dsp, &mut self.timer);
    }

    pub fn cycles(&mut self, cycle_count: u16) -> () {        
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