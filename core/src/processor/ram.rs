use crate::dsp::DSP;
use crate::processor::timer::Timer;

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

static mut RAM: Ram = Ram::new();

pub struct Ram {
    pub ram: [u8; 0x10000],
    pub read_log: Vec<(u16, u8)>,
    pub write_log: Vec<(u16, u8)>,

    ram_writable: bool,
    rom_writable: bool,

    dsp_addr: u8,
}

impl Ram {
    pub const fn new() -> Ram {
        Ram {
            ram: [0; 0x10000],
            read_log: Vec::new(),
            write_log: Vec::new(),

            ram_writable: true,
            rom_writable: false,

            dsp_addr: 0,
        }        
    }

    pub fn init(ram: &[u8; 0x10000], rom: &[u8; 64]) {
        let test = ram[0x00F0];
        let control = ram[0x00F1];
        let dsp_addr = ram[0x00F2];
        let ram_writable = (test & 2) > 0;
        let rom_writable = (control & 0x80) == 0;

        let mut global = Self::global();
        global.ram.copy_from_slice(ram);
        global.ram[0xFFC0..].copy_from_slice(&BOOT_ROM_DATA[..]);
        global.ram_writable = ram_writable;
        global.rom_writable = rom_writable;
        global.dsp_addr = dsp_addr; 
    }

    #[inline]
    pub fn global() -> &'static mut Ram {
        unsafe { &mut RAM }
    }

    pub fn read(&mut self, addr: u16, timer: &mut [Timer; 3]) -> u8 {
        log::debug!("ram[r] addr: {:06x}", addr);
        if (0x00F0..=0x00FF).contains(&addr) {
            self.read_from_io(addr as usize, timer)
        }  else {
            self.ram[addr as usize]
        }
    }

    #[inline]
    pub fn read_ram(&self, addr: u16) -> u8 {
        self.ram[addr as usize]
    }

    fn read_from_io(&mut self, addr: usize, timer: &mut [Timer; 3]) -> u8 {     
        fn zero(_ram: &mut Ram, _addr: usize, _timer: &mut [Timer; 3]) -> u8 {
            0
        }

        fn dsp_addr(ram: &mut Ram, _addr: usize, _timer: &mut [Timer; 3]) -> u8 {
            ram.dsp_addr
        }

        fn read_from_dsp(ram: &mut Ram, _addr: usize, _timer: &mut [Timer; 3]) -> u8 {
            DSP::global().read_from_register(ram.dsp_addr as usize)
        }

        
        fn read_from_ram(ram: &mut Ram, addr: usize, _timer: &mut [Timer; 3]) -> u8 {
            ram.ram[addr]
        }

        fn read_from_timer(_ram: &mut Ram, addr: usize, timer: &mut [Timer; 3]) -> u8 {
            let idx = (addr & 0xF) - 0xD;
            timer[idx].read_out()
        } 

        let idx = addr & 0x0F;
        let table = [
            // idx = 0
            zero,
            zero,
            dsp_addr,
            read_from_dsp,
            // idx = 4
            zero,
            zero,
            zero,
            zero,
            // idx = 8
            read_from_ram,
            read_from_ram,
            read_from_ram,
            read_from_ram,
            // idx = C
            read_from_ram,
            read_from_timer,
            read_from_timer,
            read_from_timer,
        ];

        table[idx](self, addr, timer)

        // match idx {
            // 0x0 => 0, // self.ram[addr], // test is write only
            // 0x1 => 0, // self.ram[addr], // control is write only
            // 0x2 => self.dsp_addr,
            // 0x3 => dsp.read_from_register(self.dsp_addr as usize, self),
            // 0x4..=0x7 => 0,     // return 0 (write to CPUIO for S-CPU(nor main CPU), but this is not functional for this emulator)
            // 0x8 => self.ram[addr],
            // 0x9 => self.ram[addr],
            // 0xA..=0xC => self.ram[addr], // each timer dividers are write only            
            // 0xD => timer[0].read_out(),
            // 0xE => timer[1].read_out(),
            // 0xF => timer[2].read_out(),
            // _ => panic!("{:#06x} should not be io address", addr),
        // }
    }

    pub fn write(&mut self, addr: u16, data: u8, timer: &mut [Timer; 3]) -> () {
        log::debug!("ram[w] addr: {:06x}, data: {:04x}", addr, data);

        match addr {
            0x0000..=0x00EF => self.ram[addr as usize] = data,         // RAM (typically used for CPU pointers/variables)
            0x00F0..=0x00FF => self.write_to_io(addr as usize, data, timer),  // I/O Ports (writes are also passed to RAM)
            0x0100..=0x01FF => self.ram[addr as usize] = data,         // RAM (typically used for CPU stack)
            0x0200..=0xFFBF => self.ram[addr as usize] = data,         // RAM (code ,data, dir-table, brr-samples, echo-buffer, etc..)
            0xFFC0..=0xFFFF => self.ram[addr as usize] = data,                
        };     
    }

    fn write_to_io(&mut self, addr: usize, data: u8, timer: &mut [Timer; 3]) -> () {    
        match addr {
            0x00F0 => self.write_to_test(data),
            0x00F1 => self.write_to_control(data, timer), 
            0x00F2 => self.dsp_addr = data,
            0x00F3 => DSP::global().write_to_register(self.dsp_addr as usize, data),            
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