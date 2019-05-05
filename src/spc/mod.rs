pub mod instruction;

use self::instruction::Addressing;
use self::instruction::Opcode;
use self::instruction::Instruction;

struct Spc700 {
  a: u8,
  x: u8,
  y: u8,
  sp: u8,
  psw: u8,
  pc: u16,
}

impl Spc700 {
  pub fn exec() {
    let opcode = ram[self.pc];
    let inst = Instruction::decode(opcode);

    match inst.opcode {
      MOV => 
    }
  }

  fn read_ram_byte(&self, addr: u8) -> u8 {
    // TODO: This 0 is dummy
    0
  }

  fn read_ram_word(&self, addr: u16) -> u8 {
    // TODO: This 0 is dummy
    0
  }

  fn mov(&mut self, inst: &Instruction) {
    let op0 = self.read_byte(inst.op0);
      
  }

  fn read_byte(&mut self, addressing: Addressing) ->  u8 {
    fn gen_word_addr(spc: &Spc700) -> u16 {
      let msb = spc.read_ram_word(spc.pc) as u16;
      let lsb = spc.read_ram_word(spc.pc + 1) as u16;

      (msb << 8) | lsb
    }

    fn read_by_abs(spc: &Spc700, bias: u8) -> u8 {
      // TODO: must check P flag      
      let addr = spc.read_ram_word(spc.pc) + bias;
      spc.read_ram_byte(addr)
    }
    
    match addressing {
      Addressing::None => { 0 }
      Addressing::Imm => { self.read_ram_word(self.pc) }
      Addressing::A => { self.a }
      Addressing::X => { self.x }
      Addressing::Y => { self.y }
      Addressing::SP => { self.sp }
      Addressing::PSW(_) => { self.psw }
      Addressing::Abs => { read_by_abs(self, 0) }
      Addressing::AbsX => { read_by_abs(self, self.x) }
      Addressing::AbsY => { read_by_abs(self, self.y) }
      Addressing::IndX => {
        let addr = self.read_ram_byte(self.x);
        self.read_ram_byte(addr)
      }
      Addressing::IndY => {
        let addr = self.read_ram_byte(self.y);
        self.read_ram_byte(addr)
      }
      Addressing::Abs16 => {
        self.read_ram_word(gen_word_addr(self))
      }
      Addressing::Abs16X => {
        let abs_addr = gen_word_addr(self);
        self.read_ram_word(abs_addr + (self.x as u16))
      }
      Addressing::Abs16Y => {
        let abs_addr = gen_word_addr(self);
        self.read_ram_word(abs_addr + (self.y as u16))
      }
      Addressing::IndAbsX => {
        let abs = self.read_ram_word(self.pc);
        let addr = self.read_ram_byte(abs + self.x);

        self.read_ram_byte(addr)
      }
      Addressing::IndAbsY => {
        let abs = self.read_ram_word(self.pc);
        let ind = self.read_ram_byte(abs);
        
        self.read_ram_byte(ind + self.y)        
      }
      _ => { 0 }
    }
  }
}
