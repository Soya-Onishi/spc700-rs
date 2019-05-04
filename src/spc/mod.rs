pub mod instruction;

// use self::instruction;

struct SpcRegisters {
  a: u8,
  x: u8,
  y: u8,
  sp: u8,
  psw: u8,
  ya: u16,
  pc: u16,
}

struct Spc700 {
  regs: SpcRegisters,
  
}

impl Spc700 {
 
}
