pub mod instruction;

use self::instruction::Addressing;
use self::instruction::Opcode;
use self::instruction::Instruction;

enum Subject {
  Addr(u16),
  Bit(u16, u8),
  A,
  X,
  Y,
  YA,
  SP,
  PSW,
  None,
}

struct Spc700 {
  a: u8,
  x: u8,
  y: u8,
  sp: u8,
  psw: u8,
  pc: u16,
}

#[allow(unused_variables)]
impl Spc700 {
  pub fn exec(&mut self) {
    let opcode = self.read_ram_word(self.pc);    
    let inst = Instruction::decode(opcode);

    match inst.opcode {
      Opcode::MOV => { self.mov(&inst); }
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

  fn write_ram_word(&mut self, addr: u16, byte: u8) {
    // TODO: Write to ram
  }

  fn add_prefix_addr(&self, addr: u8) -> u16 {
    // TODO: if P flag is set, addr: u16 = 0x0100 | addr;
    0
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
      Addressing::Imm  => { Subject::Addr(self.incl_pc()) }
      Addressing::A    => { Subject::A }
      Addressing::X    => { Subject::X }
      Addressing::Y    => { Subject::Y }
      Addressing::YA   => { Subject::YA }
      Addressing::SP   => { Subject::SP }
      Addressing::PSW(_)  => { Subject::PSW }
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
        let abs   = self.read_ram_word(self.incl_pc());
        let abs_x = abs.wrapping_add(self.x);
        let addr  = self.read_ram_byte(abs_x);
        
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
      Addressing::Abs12B => {
        let bit_addr13 = self.gen_word_addr();
        let addr = bit_addr13 & 0x1fff;
        let bit = (bit_addr13 >> 13) & 0x0007;

        Subject::Bit(addr, bit as u8)
      }
      Addressing::Special => { Subject::None }
    }    
  }

  fn read(&self, subject: Subject) -> u16 {
    match subject {
      Subject::YA => {
        let msb = self.y as u16;
        let lsb = self.a as u16;
        
        (msb << 8) | lsb          
      }    
      _ => {
        let byte = match subject {
          Subject::SP =>  { self.sp }
          Subject::Addr(addr) => { self.read_ram_word(addr) }
          Subject::Bit(addr, _) => { self.read_ram_word(addr) }
          Subject::A => { self.a }
          Subject::X => { self.x }
          Subject::Y => { self.y }
          Subject::PSW => { self.psw }
          _ => { 0 }           
        };

        byte as u16
      }     
    }
  }

  fn write(&mut self, subject: Subject, word: u16) {
    match subject {
      Subject::YA => {
        let y = (word >> 8) as u8;
        let a = (word & 0x00ff) as u8;

        self.y = y;
        self.a = a;          
      }
      _ => {
        let byte = (word & 0x00ff) as u8;

        match subject {
          Subject::SP => { self.sp = byte; }
          Subject::Addr(addr) => { self.write_ram_word(addr, byte); }
          Subject::Bit(addr, _) => { self.write_ram_word(addr, byte); }
          Subject::A => { self.a = byte; }
          Subject::X => { self.x = byte; }
          Subject::Y => { self.y = byte; }
          Subject::PSW => { self.psw = byte; }
          _ => {}
        }
      }
    }
  }
  
  fn mov(&mut self, inst: &Instruction) {
    // TODO: mov operation implement
    match inst.op0 {
      Addressing::Special => {
        match inst.raw_op {
          0xAF => {
            let addr = self.gen_subject(Addressing::IndX);
            let a = self.read(Subject::A);
            
            self.write(addr, a);

            // TODO: probably need one more cycle
            self.x = self.x.wrapping_add(1);
          }
          0xBF => {
            let addr = self.gen_subject(Addressing::IndX);
            let x = self.read(addr);
            
            self.write(Subject::A, x);

            // TODO: probably need one more cycle
            self.x = self.x.wrapping_add(1);
          }
          _ => {
            panic!("This is bug");
          }

        }
      }
      _ => {
        let op0_addr = self.gen_subject(inst.op0);
        let op1 = self.read(self.gen_subject(inst.op1));

        self.write(op0_addr, op1);
      }
    }    
  }
 
  fn push(&mut self, inst: &Instruction) {
    let source = self.read(self.gen_subject(inst.op0));    
    let sp_addr = Subject::Addr(0x0100 | (self.sp as u16));

    self.write(sp_addr, source);

    self.sp.wrapping_sub(1);    
  }
  
  fn pop(&mut self, inst: &Instruction) {
    self.sp.wrapping_add(1);
    
    let sp_addr = Subject::Addr(0x0100 | (self.sp as u16));  
    let source = self.read(sp_addr);

    let dst = self.gen_subject(inst.op0);
    
    self.write(dst, source);
  }

  fn or(&mut self, inst: &Instruction) {
    let op1_sub = self.gen_subject(inst.op1);
    let op0_sub = self.gen_subject(inst.op0);

    let op0 = self.read(op0_sub);
    let op1 = self.read(op1_sub);

    let res = op0 | op1;
    
    self.write(op0_sub, res);
  }
}
