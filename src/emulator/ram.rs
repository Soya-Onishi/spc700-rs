use crate::dsp::DSP;
use crate::emulator::timer::Timer;
use std::fs;

const BOOT_ROM_DATA: [u8; 64] = [
    0xCD, 0xEF,       // mov  x, EF    
    0xBD,             // mov  sp, x
    0xE8, 0x00,       // mov  a, 00

    0xC6,             // mov  [x], a
    0x1D,             // dec  x
    0xD0, 0xFC,       // jnz  @@zerofill_lop
    0x8F, 0xAA, 0xF4, // mov  [0xF4], AA
    0x8F, 0xBB, 0xF5, // mov  [0xF5], BB

    0x78, 0xCC, 0xF4, // cmp  [0xF4], CC
    0xD0, 0xFB,       // jnz  @@wait_for_cc
    0x2F, 0x19,       // jr   main

    0xEB, 0xF4,       // mov  y, [0xF4]
    0xD0, 0xFC,       // jnz  @@wait_for_00

    0x7E, 0xF4,       // cmp  y, [0xF4]
    0xD0, 0x0B,       // jnz  0xFFE9
    0xE4, 0xF5,       // mov  a, [0xF5]
    0xCB, 0xF4,       // mov  [0xF4], y
    0xD7, 0x00,       // mov  [[0x00] + y], a
    0xFC,             // inc  y
    0xD0, 0xF3,       // jnz  @@transfer_lop
    0xAB, 0x01,       // inc  [0x01]

    0x10, 0xEF,       // jns  @@transfer_lop
    0x7E, 0xF4,       // cmp  y, [0xF4]
    0x10, 0xEB,       // jns  @@transfer_lop

    0xBA, 0xF6,       // movw ya, [0xF6]
    0xDA, 0x00,       // movw [0x00], ya
    0xBA, 0xF4,       // movw ya, [0xF4]
    0xC4, 0xF4,       // mov  [0xF4], a
    0xDD,             // mov  a, y
    0x5D,             // mov  x, a
    0xD0, 0xDB,       // jnz  @@transfer_data
    0x1F, 0x00, 0x00, // jmp  [0x0000 + x]
    0xC0, 0xFF,       // dw   0xFFC0
];

pub struct Ram {
    pub ram: [u8; 0x10000],
    rom: [u8; 64],
    pub read_log: Vec<(u16, u8)>,
    pub write_log: Vec<(u16, u8)>,

    ram_writable: bool,
    rom_writable: bool,

    dsp_addr: u8,
}

impl Ram {
    pub fn new() -> Ram {
        Ram {
            ram: [0; 0x10000],
            rom: BOOT_ROM_DATA,
            read_log: Vec::new(),
            write_log: Vec::new(),

            ram_writable: true,
            rom_writable: false,

            dsp_addr: 0,
        }        
    }

    pub fn new_with_init(ram: &[u8; 0x10000], rom: &[u8; 64]) -> Ram {
        let test = ram[0x00F0];
        let control = ram[0x00F1];
        let dsp_addr = ram[0x00F2];
        let ram_writable = (test & 2) > 0;
        let rom_writable = (control & 0x80) == 0;

        Ram {
            ram: ram.clone(),
            rom: rom.clone(),
            read_log: Vec::new(),
            write_log: Vec::new(),

            ram_writable: ram_writable,
            rom_writable: rom_writable,

            dsp_addr: dsp_addr,
        }
    }

    pub fn load(&mut self, filename: String, start_pos: u16, set_pos: u16) {
        let binaries = fs::read(filename).expect("not found");
        let start_pos = start_pos as usize;
        let set_pos = set_pos as usize;

        for (offset, bin) in binaries[start_pos..].iter().enumerate() {
            if bin.clone() != 0 {
                // println!("Loading...[{:#06x}] <= {:#04x}", set_pos + offset, bin);
            }

            self.ram[set_pos + offset] = bin.clone();
        }
    }

    pub fn read(&mut self, addr: u16, dsp: &mut DSP, timer: &mut [Timer; 3]) -> u8 {
        log::debug!("ram[r] addr: {:06x}", addr);

        match addr {
            0x0000..=0x00EF => self.ram[addr as usize],         // RAM (typically used for CPU pointers/variables)
            0x00F0..=0x00FF => self.read_from_io(addr as usize, dsp, timer),  // I/O Ports (writes are also passed to RAM)
            0x0100..=0x01FF => self.ram[addr as usize],         // RAM (typically used for CPU stack)            
            0x0200..=0xFFBF => self.ram[addr as usize],         // RAM (code ,data, dir-table, brr-samples, echo-buffer, etc..)
            0xFFC0..=0xFFFF => 
                if self.rom_writable { 
                    self.ram[addr as usize]
                } else {
                    self.rom[(addr - 0xFFC0) as usize]
                }
        }
    }

    fn read_from_io(&mut self, addr: usize, dsp: &mut DSP, timer: &mut [Timer; 3]) -> u8 {        
        match addr {
            0x00F0 => 0, // self.ram[addr], // test is write only
            0x00F1 => 0, // self.ram[addr], // control is write only
            0x00F2 => self.dsp_addr,
            0x00F3 => dsp.read_from_register(self.dsp_addr as usize, self),
            0x00F4..=0x00F7 => 0,     // return 0 (write to CPUIO for S-CPU(nor main CPU), but this is not functional for this emulator)
            0x00F8 => self.ram[addr],
            0x00F9 => self.ram[addr],
            0x00FA..=0x00FC => self.ram[addr], // each timer dividers are write only            
            0x00FD => timer[0].read_out(),
            0x00FE => timer[1].read_out(),
            0x00FF => timer[2].read_out(),
            _ => panic!("{:#06x} should not be io address", addr),
        }
    }

    pub fn write(&mut self, addr: u16, data: u8, dsp: &mut DSP, timer: &mut [Timer; 3]) -> () {
        log::debug!("ram[w] addr: {:06x}, data: {:04x}", addr, data);

        match addr {
            0x0000..=0x00EF => self.ram[addr as usize] = data,         // RAM (typically used for CPU pointers/variables)
            0x00F0..=0x00FF => self.write_to_io(addr as usize, data, dsp, timer),  // I/O Ports (writes are also passed to RAM)
            0x0100..=0x01FF => self.ram[addr as usize] = data,         // RAM (typically used for CPU stack)
            0x0200..=0xFFBF => self.ram[addr as usize] = data,         // RAM (code ,data, dir-table, brr-samples, echo-buffer, etc..)
            0xFFC0..=0xFFFF => self.ram[addr as usize] = data,                
        };     
    }

    fn write_to_io(&mut self, addr: usize, data: u8, dsp: &mut DSP, timer: &mut [Timer; 3]) -> () {    
        match addr {
            0x00F0 => self.write_to_test(data),
            0x00F1 => self.write_to_control(data, timer), 
            0x00F2 => self.dsp_addr = data,
            0x00F3 => dsp.write_to_register(self.dsp_addr as usize, data, self),            
            0x00F4..=0x00F7 => (), // nothing to do (write to CPUIO for S-CPU(nor main CPU), but this is not functional for this emulator)
            0x00F8 => self.ram[addr] = data, // each AUXIO has no functionality
            0x00F9 => self.ram[addr] = data,
            0x00FA => timer[0].write_divider(data), // timer 0 divider settings
            0x00FB => timer[1].write_divider(data), // timer 1 divider settings
            0x00FC => timer[2].write_divider(data), // timer 2 divider settings
            0x00FD..=0x00FF => (), // writing to TxOUT is not available (T0OUT, T1OUT, T2OUT is read only).
            _ => panic!("{:#06x} should not be io address", addr),
        };

        // data is also written to ram
        self.ram[addr] = data;
    }

    fn write_to_test(&mut self, data: u8) -> () {
        let ram_writable = (data & 2) > 0;

        self.ram_writable = ram_writable;
    }

    fn write_to_control(&mut self, data: u8, timer: &mut [Timer; 3]) -> () {
        let timer0_enable = (data & 0x01) > 0;
        let timer1_enable = (data & 0x02) > 0;
        let timer2_enable = (data & 0x04) > 0;

        let rom_writable = !((data & 0x80) > 0);

        if timer0_enable { timer[0].enable() } else { timer[0].disable() };
        if timer1_enable { timer[1].enable() } else { timer[1].disable() };
        if timer2_enable { timer[2].enable() } else { timer[2].disable() };

        self.rom_writable = rom_writable;
    }
}