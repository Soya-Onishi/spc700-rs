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
        let (upper, lower) = (opcode >> 4, opcode & 0xF);
        let op_select = |upper: u8| {
            match upper {
                0x0 | 0x1 => or,
                0x2 | 0x3 => and,
                0x4 | 0x5 => eor,
                0x6 | 0x7 => cmp,
                0x8 | 0x9 => adc,
                0xA | 0xB => sbc,
                _ => panic!("upper must be between 0x0 to 0xB. actual {:#04x}", upper),
            }
        };
        let shift_select = |upper: u8| {
            match upper {
                0x0 | 0x1 => asl,
                0x2 | 0x3 => rol,
                0x4 | 0x5 => lsr,
                0x6 | 0x7 => ror,
                _ => panic!("upper expects to be between 0 to 7. actual: {:#04x}", upper),
            }
        };
        
        let is_cmp = |upper: u8| { upper == 0x6 || upper == 0x7 };

        match (upper, lower) {            
            (  0x0, 0x0) => self.nop(),
            (upper, 0x0) if upper % 2 == 1 => self.branch_by_psw(opcode),
            (  0x2, 0x0) => self.clrp(),
            (  0x4, 0x0) => self.setp(),
            (  0x6, 0x0) => self.clrc(),
            (  0x8, 0x0) => self.setc(),
            (  0xA, 0x0) => self.ei(),
            (  0xC, 0x0) => self.di(),
            (  0xE, 0x0) => self.clrv(),
            (    _, 0x1) => self.tcall(opcode),
            (upper, 0x2) if upper % 2 == 0 => self.set1(opcode),
            (    _, 0x2) => self.clr1(opcode),
            (    _, 0x3) => self.branch_by_mem_bit(opcode),            
            (upper, 0x4) if upper <= 0xB && upper % 2 == 0 => {
                let op = op_select(upper);
                self.alu_dp(0, op);
            },
            (upper, 0x4) if upper <= 0xB => {
                let op = op_select(upper);
                self.alu_x_idx_indirect(op);
            },
            (  0xC, 0x4) => self.mov_store_dp_reg(0),
            (  0xD, 0x4) => self.mov_store_x_idx_indirect(0),
            (  0xE, 0x4) => self.mov_load_dp(0),
            (  0xF, 0x4) => self.mov_load_x_idx_indirect(0),
            (upper, 0x5) if upper <= 0xB && upper % 2 == 0 => {
                let op = op_select(upper);
                self.alu_addr(0, op);
            },
            (upper, 0x5) if upper <= 0xB => {
                let op = op_select(upper);
                self.alu_x_idx_addr(op);
            },
            (  0xC, 0x5) => self.mov_store_addr(0),
            (  0xD, 0x5) => self.mov_store_x_idx_addr(),
            (  0xE, 0x5) => self.mov_load_addr(0),
            (  0xF, 0x5) => self.mov_load_x_idx_addr(),
            (upper, 0x6) if upper <= 0xB && upper % 2 == 0 => {
                let op = op_select(upper);
                self.alu_indirect_x(op);
            },
            (upper, 0x6) if upper <= 0xB => {
                let op = op_select(upper);
                self.alu_y_idx_addr(op);
            },
            (  0xC, 0x6) => self.mov_store_x_indirect(),
            (  0xD, 0x6) => self.mov_store_y_idx_addr(),
            (  0xE, 0x6) => self.mov_load_x_indirect(),
            (  0xF, 0x6) => self.mov_load_y_idx_addr(),
            (upper, 0x7) if upper <= 0xB && upper % 2 == 0 => {
                let op = op_select(upper);
                self.alu_x_ind_ind(op);
            },
            (upper, 0x7) if upper <= 0xB => {
                let op = op_select(upper);
                self.alu_y_ind_ind(op);
            },
            (  0xC, 0x7) => self.mov_store_x_ind_ind(),
            (  0xD, 0x7) => self.mov_store_y_ind_ind(),
            (  0xE, 0x7) => self.mov_load_x_ind_ind(),
            (  0xF, 0x7) => self.mov_load_y_ind_ind(),
            (upper, 0x8) if upper <= 0xB && upper % 2 == 0 => {
                let op = op_select(upper);
                self.alu_imm(0, op);
            },
            (upper, 0x8) if upper <= 0xB => {
                let op = op_select(upper);
                self.alu_dp_imm(is_cmp(upper), op);
            },
            (  0xC, 0x8) => self.alu_imm(1, cmp),
            (  0xD, 0x8) => self.mov_store_dp_reg(1),
            (  0xE, 0x8) => self.mov_reg_imm(0),
            (  0xF, 0x8) => self.mov_load_dp(1),
            (upper, 0x9) if upper <= 0xB && upper % 2 == 0 => {
                let op = op_select(upper);
                self.alu_dp_dp(is_cmp(upper), op);
            },
            (upper, 0x9) if upper <= 0xB => {
                let op = op_select(upper);
                self.alu_x_y(is_cmp(upper), op);
            },
            (  0xC, 0x9) => self.mov_store_addr(1),
            (  0xD, 0x9) => self.mov_store_y_idx_indirect(),
            (  0xE, 0x9) => self.mov_load_addr(1),
            (  0xF, 0x9) => self.mov_load_y_idx_indirect(),
            (upper, 0xA) if upper % 2 == 0 => {
                match upper / 2 {
                    0 | 1 => self.or1(opcode),
                    2 | 3 => self.and1(opcode),
                    4     => self.eor1(),
                    5     => self.mov1_to_psw(),
                    6     => self.mov1_to_mem(),
                    7     => self.not1(),
                    other => panic!("upper / 2 must be between 0 to 7, actual {:#04x}", other),
                };
            }
            (  0x1, 0xA) => self.inc_dec_word(!0),
            (  0x3, 0xA) => self.inc_dec_word( 1),
            (  0x5, 0xA) => self.cmpw(),
            (  0x7, 0xA) => self.addw(),
            (  0x9, 0xA) => self.subw(),
            (  0xB, 0xA) => self.mov_load_word(),
            (  0xD, 0xA) => self.mov_store_word(),
            (  0xF, 0xA) => self.mov_store_dp_dp(),
            (upper, 0xB) if upper <= 0x7 && upper % 2 == 0 => {
                let op = shift_select(upper);
                self.shift_dp(opcode, op);
            },
            (upper, 0xB) if upper <= 0x7 => {
                let op = shift_select(upper);
                self.shift_x_idx_indirect(opcode, op);
            },
            (  0x8, 0xB) => self.inc_dec_dp(opcode),
            (  0x9, 0xB) => self.inc_dec_x_idx_indirect(opcode),
            (  0xA, 0xB) => self.inc_dec_dp(opcode),
            (  0xB, 0xB) => self.inc_dec_x_idx_indirect(opcode),
            (  0xC, 0xB) => self.mov_store_dp_reg(2),
            (  0xD, 0xB) => self.mov_store_x_idx_indirect(2),
            (  0xE, 0xB) => self.mov_load_dp(2),
            (  0xF, 0xB) => self.mov_load_x_idx_indirect(2),
            (upper, 0xC) if upper <= 0x7 && upper % 2 == 0 => {
                let op = shift_select(upper);
                self.shift_addr(opcode, op);
            },
            (upper, 0xC) if upper <= 0x7 => {
                let op = shift_select(upper);
                self.shift_acc(opcode, op);
            },
            (  0x8, 0xC) => self.inc_dec_addr(opcode),
            (  0x9, 0xC) => self.inc_dec_reg(opcode, 0),
            (  0xA, 0xC) => self.inc_dec_addr(opcode),
            (  0xB, 0xC) => self.inc_dec_reg(opcode, 0),
            (  0xC, 0xC) => self.mov_store_addr(2),
            (  0xD, 0xC) => self.inc_dec_reg(opcode, 2),
            (  0xE, 0xC) => self.mov_load_addr(2),
            (  0xF, 0xC) => self.inc_dec_reg(opcode, 2),
            (  0x0, 0xD) => self.push(3),
            (  0x1, 0xD) => self.inc_dec_reg(opcode, 1),
            (  0x2, 0xD) => self.push(0),
            (  0x3, 0xD) => self.inc_dec_reg(opcode, 1),
            (  0x4, 0xD) => self.push(1),
            (  0x5, 0xD) => self.mov_reg_reg(0, 1),
            (  0x6, 0xD) => self.push(2),
            (  0x7, 0xD) => self.mov_reg_reg(1, 0),
            (  0x8, 0xD) => self.mov_reg_imm(2),
            (  0x9, 0xD) => self.mov_reg_reg(3, 1),
            (  0xA, 0xD) => self.alu_imm(2, cmp),
            (  0xB, 0xD) => self.mov_reg_reg(1, 3),
            (  0xC, 0xD) => self.mov_reg_imm(1),
            (  0xD, 0xD) => self.mov_reg_reg(2, 0),
            (  0xE, 0xD) => self.notc(),
            (  0xF, 0xD) => self.mov_reg_reg(0, 2),
            (  0x0, 0xE) => self.tset1(),
            (  0x1, 0xE) => self.alu_addr(1, cmp),
            (  0x2, 0xE) => self.cbne(opcode),
            (  0x3, 0xE) => self.alu_dp(1, cmp),
            (  0x4, 0xE) => self.tclr1(),
            (  0x5, 0xE) => self.alu_addr(2, cmp),
            (  0x6, 0xE) => self.dbnz_data(),
            (  0x7, 0xE) => self.alu_dp(2, cmp),
            (  0x8, 0xE) => self.pop(3),
            (  0x9, 0xE) => self.div(),
            (  0xA, 0xE) => self.pop(0),
            (  0xB, 0xE) => self.das(),
            (  0xC, 0xE) => self.pop(1),
            (  0xD, 0xE) => self.cbne(opcode),
            (  0xE, 0xE) => self.pop(2),
            (  0xF, 0xE) => self.dbnz_y(),
            (  0x0, 0xF) => self.brk(),
            (  0x1, 0xF) => self.jmp_abs_x(),
            (  0x2, 0xF) => self.bra(),
            (  0x3, 0xF) => self.call(),
            (  0x4, 0xF) => self.pcall(),
            (  0x5, 0xF) => self.jmp_abs(),
            (  0x6, 0xF) => self.ret(),
            (  0x7, 0xF) => self.ret1(),
            (  0x8, 0xF) => self.mov_store_dp_imm(),
            (  0x9, 0xF) => self.xcn(),
            (  0xA, 0xF) => self.mov_store_x_indirect_inc(),
            (  0xB, 0xF) => self.mov_load_x_indirect_inc(),
            (  0xC, 0xF) => self.mul(),
            (  0xD, 0xF) => self.daa(),
            (  0xE, 0xF) => self.sleep_or_stop(),
            (  0xF, 0xF) => self.sleep_or_stop(),
            (upper, lower) => panic!("invalid parsed opcode. upper: {:#04x}, lower: {:#04x}", upper, lower),
        }

        self.dsp.flush(&mut self.ram);  // flush in force                                        
    }

    fn mov_reg_imm(&mut self, to: u8) -> () {
        let imm = self.read_from_pc();
        match to {
            0 => self.reg.a = imm,
            1 => self.reg.x = imm,
            2 => self.reg.y = imm,
            _ => panic!("register type must be between 0 to 2"),
        }

        self.set_mov_flag(imm);
    }

    fn mov_reg_reg(&mut self, from: u8, to: u8) -> () {
        self.cycles(1);
        
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

    fn mov_load_dp(&mut self, to: u8) -> () {
        let addr = self.read_from_pc();
        let data = self.read_from_page(addr);

        match to {
            0 => self.reg.a = data,
            1 => self.reg.x = data,
            2 => self.reg.y = data,
            _ => panic!("register type must be between 0 to 2"),
        };

        self.set_mov_flag(data);
    }

    fn mov_load_x_idx_indirect(&mut self, to: u8) -> () {
        let addr = self.read_from_pc().wrapping_add(self.reg.x);
        let data = self.read_from_page(addr);
        self.cycles(1);

        match to {
            0 => self.reg.a = data,
            2 => self.reg.y = data,
            _ => panic!("register type must be between 0 to 2"),
        }

        self.set_mov_flag(data);
    }

    fn mov_load_y_idx_indirect(&mut self) -> () {
        let addr = self.read_from_pc().wrapping_add(self.reg.y);
        let data = self.read_from_page(addr);
        self.cycles(1);
        self.reg.x = data;

        self.set_mov_flag(data);
    }

    fn mov_load_addr(&mut self, to: u8) -> () {
        let lower = self.read_from_pc() as u16;
        let upper = self.read_from_pc() as u16;
        let addr = (upper << 8) | lower;
        let data = self.read_ram(addr);

        match to {
            0 => self.reg.a = data,
            1 => self.reg.x = data,
            2 => self.reg.y = data,
            _ => panic!("register type must be between 0 to 2"),
        }

        self.set_mov_flag(data);
    }

    fn mov_load_x_idx_addr(&mut self) -> () {
        let lower = self.read_from_pc() as u16;
        let upper = self.read_from_pc() as u16;
        let addr = (upper << 8) | lower;    
        let addr = addr.wrapping_add(self.reg.x as u16);
        self.cycles(1);

        let data = self.read_ram(addr);

        self.reg.a = data;
        self.set_mov_flag(data);
    }

    fn mov_load_y_idx_addr(&mut self) -> () {
        let lower = self.read_from_pc() as u16;
        let upper = self.read_from_pc() as u16;
        let addr = (upper << 8) | lower;    
        let addr = addr.wrapping_add(self.reg.y as u16);
        self.cycles(1);

        let data = self.read_ram(addr);

        self.reg.a = data;
        self.set_mov_flag(data);
    }

    fn mov_load_x_indirect(&mut self) -> () {
        let data = self.read_from_page(self.reg.x);
        self.cycles(1);

        self.reg.a = data;
        self.set_mov_flag(data);
    }

    fn mov_load_x_indirect_inc(&mut self) -> () {
        self.cycles(1);
        let data = self.read_from_page(self.reg.x);
        self.reg.x = self.reg.x.wrapping_add(1);
        self.cycles(1);

        self.reg.a = data;
        self.set_mov_flag(data);
    }

    fn mov_load_y_ind_ind(&mut self) -> () {
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

    fn mov_load_x_ind_ind(&mut self) -> () {
        let base_addr = self.read_from_pc().wrapping_add(self.reg.x);        
        self.cycles(1);
        let lower = self.read_from_page(base_addr) as u16;
        let upper = self.read_from_page(base_addr.wrapping_add(1)) as u16;
        let addr = (upper << 8) | lower;
        let data = self.read_ram(addr);

        self.reg.a = data;
        self.set_mov_flag(data);
    }

    fn mov_load_word(&mut self) -> () {
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

    fn mov_store_dp_imm(&mut self) -> () {
        let imm = self.read_from_pc();
        let addr = self.read_from_pc();
        let _ = self.read_from_page(addr);        

        self.write_to_page(addr, imm);        
    }

    fn mov_store_dp_dp(&mut self) -> () {
        let bb = self.read_from_pc();
        let b = self.read_from_page(bb);
        let aa = self.read_from_pc();        
        
        self.write_to_page(aa, b);
    }

    fn mov_store_dp_reg(&mut self, reg: u8) -> () {
        let addr = self.read_from_pc();
        let _ = self.read_from_page(addr);
        let data = match reg {
            0 => self.reg.a,
            1 => self.reg.x,
            2 => self.reg.y,
            _ => panic!("register type must be between 0 to 2"),
        };

        self.write_to_page(addr, data);
    }

    fn mov_store_x_idx_indirect(&mut self, reg: u8) -> () {
        let addr = self.read_from_pc().wrapping_add(self.reg.x);
        self.cycles(1);
        let _ = self.read_from_page(addr);
        let data = match reg {
            0 => self.reg.a,
            2 => self.reg.y,    
            _ => panic!("register type must be 0 or 2"),
        };
        
        self.write_to_page(addr, data);
    }

    fn mov_store_y_idx_indirect(&mut self) -> () {
        let addr = self.read_from_pc().wrapping_add(self.reg.y);
        self.cycles(1);
        let _ = self.read_from_page(addr);
        let data = self.reg.x;
        
        self.write_to_page(addr, data);
    }

    fn mov_store_addr(&mut self, reg: u8) -> () {
        let lower = self.read_from_pc() as u16;
        let upper = self.read_from_pc() as u16;
        let addr = (upper << 8) | lower;
        let _ = self.read_ram(addr);
        let data = match reg { 
            0 => self.reg.a,
            1 => self.reg.x,
            2 => self.reg.y,    
            _ => panic!("register type must be between 0 to 2"),
        };

        self.write_ram(addr, data);
    }

    fn mov_store_x_idx_addr(&mut self) -> () {
        let lower = self.read_from_pc() as u16;
        let upper = self.read_from_pc() as u16;
        let addr = (upper << 8) | lower;
        let addr = addr.wrapping_add(self.reg.x as u16);
        self.cycles(1);
        let _ = self.read_ram(addr);
        
        self.write_ram(addr, self.reg.a);
    }

    fn mov_store_y_idx_addr(&mut self) -> () {
        let lower = self.read_from_pc() as u16;
        let upper = self.read_from_pc() as u16;
        let addr = (upper << 8) | lower;
        let addr = addr.wrapping_add(self.reg.y as u16);
        self.cycles(1);
        let _ = self.read_ram(addr);

        self.write_ram(addr, self.reg.a);
    }

    fn mov_store_x_indirect_inc(&mut self) -> () {
        self.cycles(1);
        self.write_to_page(self.reg.x, self.reg.a);
        self.reg.x = self.reg.x.wrapping_add(1);
        self.cycles(1);
    }

    fn mov_store_x_indirect(&mut self) -> () {
        self.cycles(1);
        self.read_from_page(self.reg.x);
        self.write_to_page(self.reg.x, self.reg.a);        
    }

    fn mov_store_x_ind_ind(&mut self) -> () {
        let base_addr = self.read_from_pc().wrapping_add(self.reg.x);
        self.cycles(1);
        let lower = self.read_from_page(base_addr) as u16;
        let upper = self.read_from_page(base_addr.wrapping_add(1)) as u16;
        let addr = (upper << 8) | lower;
        let _ = self.read_ram(addr);

        self.write_ram(addr, self.reg.a);
    }

    fn mov_store_y_ind_ind(&mut self) -> () {
        let base_addr = self.read_from_pc();
        let lower = self.read_from_page(base_addr) as u16;
        let upper = self.read_from_page(base_addr) as u16;
        let addr = (upper << 8) | lower;        
        let addr = addr.wrapping_add(self.reg.y as u16);
        self.cycles(1);
        let _ = self.read_ram(addr);
        
        self.write_ram(addr, self.reg.a);
    }

    fn mov_store_word(&mut self) -> () {
        let addr = self.read_from_pc();
        let _ = self.read_from_page(addr);
        self.write_to_page(addr, self.reg.a);
        self.write_to_page(addr.wrapping_add(1), self.reg.y);
    }

    fn push(&mut self, reg: u8) -> () {
        self.cycles(1);
        let data = match reg {
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

    fn pop(&mut self, reg: u8) -> () {
        self.reg.sp = self.reg.sp.wrapping_add(1);
        self.cycles(1);
        let data = self.read_from_stack();
        match reg {
            0 => self.reg.a = data,
            1 => self.reg.x = data,
            2 => self.reg.y = data,
            3 => self.reg.psw.set(data),
            _ => panic!("register type must be between 0 to 3"),
        };
        self.cycles(1);
    }

    fn nop(&mut self) -> () {
        self.cycles(1);
    }

    fn sleep_or_stop(&mut self) -> () {        
        self.is_stopped = true;
        self.cycles(2);
    }

    fn clrp(&mut self) -> () {
        self.reg.psw.negate_page();
        self.cycles(1);
    }

    fn setp(&mut self) -> () {
        self.reg.psw.assert_page();
        self.cycles(1);
    }

    fn ei(&mut self) -> () {
        self.reg.psw.assert_interrupt();
        self.cycles(2);
    }

    fn di(&mut self) -> () {
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

    fn not1(&mut self) -> () {
        let (addr, bit_idx) = self.addr_and_idx();
        let data = self.read_ram(addr);
        let ret = data ^ (1 << bit_idx);
        
        self.write_ram(addr, ret);
    }

    fn mov1_to_mem(&mut self) -> () {
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

    fn mov1_to_psw(&mut self) -> () {
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

    fn eor1(&mut self) -> () {
        let (addr, bit_idx) = self.addr_and_idx();
        let data = self.read_ram(addr);
        let bit = ((data >> bit_idx) & 1) == 1;        
        let ret = self.reg.psw.carry () ^ bit;
        self.cycles(1);

        self.reg.psw.set_carry(ret);
    }

    fn clrc(&mut self) -> () {
        self.cycles(1);
        self.reg.psw.set_carry(false);
    }

    fn setc(&mut self) -> () {
        self.cycles(1);
        self.reg.psw.set_carry(true);
    }

    fn notc(&mut self) -> () {
        self.cycles(2);
        self.reg.psw.set_carry(!self.reg.psw.carry());
    }

    fn clrv(&mut self) -> () {
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

    fn dbnz_y(&mut self) -> () {
        let rr = self.read_from_pc() as u16;
        self.reg.y = self.reg.y.wrapping_sub(1);
        self.cycles(2);
        
        if self.reg.y != 0 {
            self.cycles(2);
            let offset = if (rr & 0x80) != 0 { 0xFF00 | rr } else { rr };
            self.reg.pc = self.reg.pc.wrapping_add(offset);
        }
    }

    fn dbnz_data(&mut self) -> () {
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

    fn bra(&mut self) -> () {
        let rr = self.read_from_pc() as u16;
        self.cycles(2);
        let offset = if (rr & 0x80) != 0 { 0xFF00 | rr } else { rr };
        self.reg.pc = self.reg.pc.wrapping_add(offset);
    }

    fn jmp_abs(&mut self) -> () {
        let lower = self.read_from_pc() as u16;
        let upper = self.read_from_pc() as u16;
        let addr = (upper << 8) | lower;

        self.reg.pc = addr;
    }

    fn jmp_abs_x(&mut self) -> () {
        let lower = self.read_from_pc() as u16;
        let upper = self.read_from_pc() as u16;
        let addr = (upper << 8) | lower;
        let addr = addr.wrapping_add(self.reg.x as u16);
        self.cycles(1);

        let dst_lower = self.read_ram(addr) as u16;
        let dst_upper = self.read_ram(addr.wrapping_add(1)) as u16;

        self.reg.pc = (dst_upper << 8) | dst_lower;
    }
    
    fn call(&mut self) -> () {
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

    fn pcall(&mut self) -> () {        
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

    fn ret(&mut self) -> () {
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

    fn ret1(&mut self) -> () {
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

    fn brk(&mut self) -> () {
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

    fn alu_dp(&mut self, from: u8, op: impl Fn(&mut Register, u8, u8) -> u8) -> () {
        let addr = self.read_from_pc();        
        let b = self.read_from_page(addr);
        let a = match from {
            0 => self.reg.a,
            1 => self.reg.x,
            2 => self.reg.y,
            _ => panic!("from register types must be between 0 to 2"),
        };         

        self.reg.a = op(&mut self.reg, a, b);        
    }

    fn alu_addr(&mut self, from: u8, op: impl Fn(&mut Register, u8, u8) -> u8) -> () {        
        let lower = self.read_from_pc() as u16;
        let upper = self.read_from_pc() as u16;
        let addr = (upper << 8) | lower;

        let b = self.read_ram(addr);
        let a = match from {
            0 => self.reg.a,
            1 => self.reg.x,
            2 => self.reg.y,
            _ => panic!("from register types must be between 0 to 2"),
        };        
        
        self.reg.a = op(&mut self.reg, a, b);        
    }

    fn alu_indirect_x(&mut self, op: impl Fn(&mut Register, u8, u8) -> u8) -> () {
        let x = self.reg.x;
        self.cycles(1);

        let a = self.reg.a;
        let b = self.read_from_page(x);
        let data = op(&mut self.reg, a, b);

        self.reg.a = data;        
    }

    fn alu_x_idx_indirect(&mut self, op: impl Fn(&mut Register, u8, u8) -> u8) -> () {
        let addr = self.read_from_pc().wrapping_add(self.reg.x);
        self.cycles(1);
        
        let a = self.reg.a;
        let b = self.read_from_page(addr);        
        let ret = op(&mut self.reg, a, b);

        self.reg.a = ret;        
    }

    fn alu_x_idx_addr(&mut self, op: impl Fn(&mut Register, u8, u8) -> u8) -> () {
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

    fn alu_x_ind_ind(&mut self, op: impl Fn(&mut Register, u8, u8) -> u8) -> () {
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

    fn alu_y_ind_ind(&mut self, op: impl Fn(&mut Register, u8, u8) -> u8) -> () {
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

    fn alu_y_idx_addr(&mut self, op: impl Fn(&mut Register, u8, u8) -> u8) -> () {
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

    fn alu_x_y(&mut self, is_cmp: bool, op: impl Fn(&mut Register, u8, u8) -> u8) -> () {
        self.cycles(1);
        let x_data = self.read_from_page(self.reg.x);        
        let y_data = self.read_from_page(self.reg.y);
        let ret = op(&mut self.reg, x_data, y_data);        

        if !is_cmp {
            self.write_to_page(self.reg.x, ret);
        } else {
            self.cycles(1)
        }
    }

    fn alu_imm(&mut self, from: u8, op: impl Fn(&mut Register, u8, u8) -> u8) -> () {
        let a = match from {
            0 => self.reg.a,
            1 => self.reg.x,
            2 => self.reg.y,
            _ => panic!("from register types must be between 0 to 2"),
        };

        let b = self.read_from_pc();
        let ret = op(&mut self.reg, a, b);

        match from {
            0 => self.reg.a = ret,
            1 => self.reg.x = ret,
            2 => self.reg.y = ret,
            _ => panic!("from register types must be between 0 to 2"),
        };        
    }

    fn alu_dp_imm(&mut self, is_cmp: bool, op: impl Fn(&mut Register, u8, u8) -> u8) -> () {
        let imm = self.read_from_pc();
        let addr = self.read_from_pc();
        let data = self.read_from_page(addr);
        let ret = op(&mut self.reg, data, imm);

        if !is_cmp {
            self.write_to_page(addr, ret);
        } else {
            self.cycles(1);
        }
    }

    fn alu_dp_dp(&mut self, is_cmp: bool, op: impl Fn(&mut Register, u8, u8) -> u8) -> () {
        let bb = self.read_from_pc();
        let b = self.read_from_page(bb);
        let aa = self.read_from_pc();        
        let a = self.read_from_page(aa);

        let ret = op(&mut self.reg, a, b);

        if !is_cmp {
            self.write_to_page(aa, ret);
        } else {
            self.cycles(1);
        } 
    }

    fn shift_acc(&mut self, opcode: u8, op: impl Fn(u8, bool) -> (u8, bool)) -> () {
        self.cycles(1);
        let ret = self.shift(opcode, self.reg.a, op);
        
        self.reg.a = ret;        
    }

    fn shift_dp(&mut self, opcode: u8, op: impl Fn(u8, bool) -> (u8,bool)) -> () {
        let addr = self.read_from_pc();
        let data = self.read_from_page(addr);        
        let ret = self.shift(opcode, data, op);

        self.write_to_page(addr, ret);
    }

    fn shift_x_idx_indirect(&mut self, opcode: u8, op: impl Fn(u8, bool) -> (u8, bool)) -> () {
        let addr = self.read_from_pc().wrapping_add(self.reg.x);
        self.cycles(1);

        let data = self.read_from_page(addr);
        let ret = self.shift(opcode, data, op);

        self.write_to_page(addr, ret);
    }

    fn shift_addr(&mut self, opcode: u8, op: impl Fn(u8, bool) -> (u8, bool)) -> () {
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

    fn inc_dec_reg(&mut self, opcode: u8, reg_type: u8) -> () {
        self.cycles(1);

        let is_inc = (opcode & 0x20) != 0;
        let data = match reg_type {
            0 => self.reg.a,
            1 => self.reg.x,
            2 => self.reg.y,
            _ => panic!("require 0 to 2 as register type"),
        };

        let ret =
            if is_inc { data.wrapping_add(1) }
            else      { data.wrapping_sub(1) };

        match reg_type {
            0 => self.reg.a = ret,
            1 => self.reg.x = ret,
            2 => self.reg.y = ret,
            _ => panic!("require 0 to 2 as register type"),
        }

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

    fn addw(&mut self) -> () {
        self.reg.psw.negate_carry();

        let (ya, word) = self.get_word_operands();        
        let ret_lower = adc(&mut self.reg, ya as u8, word as u8) as u16;
        let ret_upper = adc(&mut self.reg, (ya >> 8) as u8, (word >> 8) as u8) as u16;
        let ret = (ret_upper << 8) | ret_lower;
                
        self.cycles(1);
        self.reg.set_ya(ret);

        self.reg.psw.set_zero(ret == 0);
    }

    fn subw(&mut self) -> () {
        self.reg.psw.assert_carry();

        let (ya, word) = self.get_word_operands();
        let ret_lower = sbc(&mut self.reg, ya as u8, word as u8) as u16;
        let ret_upper = sbc(&mut self.reg, (ya >> 8) as u8, (word >> 8) as u8) as u16;
        let ret = (ret_upper << 8) | ret_lower;

        self.cycles(1);
        self.reg.set_ya(ret);

        self.reg.psw.set_zero(ret == 0);
    }

    fn cmpw(&mut self) -> () {
        let (ya, word) = self.get_word_operands();
        let ret = (ya as i32) - (word as i32);

        self.reg.psw.set_sign((ret & 0x8000) != 0);
        self.reg.psw.set_zero(ret as u16 == 0);
        self.reg.psw.set_carry(ret >= 0);
    }

    fn inc_dec_word(&mut self, x: u16) -> () {
        let addr = self.read_from_pc();

        let word_lower = self.read_from_page(addr) as u16;                
        let lower_result = word_lower.wrapping_add(x); 
        let lower_carry = lower_result >> 8;
        self.write_to_page(addr, lower_result as u8);

        let word_upper = self.read_from_page(addr.wrapping_add(1)) as u16;
        let upper_result = word_upper.wrapping_add(lower_carry);
        
        self.write_to_page(addr.wrapping_add(1), upper_result as u8);

        let result = (upper_result << 8) | lower_result;
        self.reg.psw.set_zero(result == 0);
        self.reg.psw.set_sign((result & 0x8000) != 0);
    }

    fn div(&mut self) -> () {
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

    fn mul(&mut self) -> () {
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

    fn daa(&mut self) -> () {
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

    fn das(&mut self) -> () {
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

    fn xcn(&mut self) -> () {
        self.cycles(4);
        self.reg.a = (self.reg.a >> 4) | ((self.reg.a & 0x0F) << 4);

        self.reg.psw.set_zero(self.reg.a == 0);
        self.reg.psw.set_sign((self.reg.a & 0x80) != 0); 
    }

    fn tclr1(&mut self) -> () {
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

    fn tset1(&mut self) -> () {
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