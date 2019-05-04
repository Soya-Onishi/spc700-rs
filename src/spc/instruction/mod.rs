mod opcode;
mod addressing;

pub use self::opcode::Opcode;
pub use self::addressing::Addressing;

pub struct Instruction {
  op: Opcode,
  addressing: Addressing,
  cycle: u8,
}

impl Instruction {
  pub const fn new(op: Opcode, addressing: Addressing, cycle: u8) -> Instruction {
    Instruction {
      op, addressing, cycle,
    }
  }
}

/*
const INSTRUCTIONS: [Instruction; ] = [
  // 0x0X
  Instruction::new(Opcode::NOP,   Addressing::NONE, 2),
  Instruction::new(Opcode::TCALL, Addressing::NONE, 8),
  Instruction::new(Opcode::SET1, Addressing::NONE, 4),
  Instruction::new(Opcode::BBS, Addressing::NONE, 5),
  Insturction::new(Opcode::OR, Addressing::, 3),
  Instruction::new(Opcode::OR, Addressing::, 4),
  Instruction::new(Opcode::OR, Addressing::, 3),
  Instruction::new(Opcode::OR, Addressing::, 6),
  Instruction::new(Opcode::OR, Addressing::, 2),
  Instruction::new()
]
*/
