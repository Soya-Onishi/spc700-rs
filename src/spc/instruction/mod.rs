mod opcode;
mod addressing;

pub use self::opcode::Opcode;
pub use self::addressing::Addressing;

pub struct Instruction {
    pub opcode: Opcode,
    pub raw_op: u8,
    pub op0: Addressing,
    pub op1: Addressing,
    pub cycle: u8,
}

impl Instruction {
    pub const fn new(raw_op: u8, opcode: Opcode, op0: Addressing, op1: Addressing, cycle: u8) -> Instruction {
        Instruction {
            opcode,
            raw_op,
            op0,
            op1,
            cycle,
        }
    }

    pub fn decode(value: u8) -> Instruction {
        Instruction::new(
            value,
            self::opcode::OPCODE_TABLE[value as usize],
            self::addressing::ADDRESSING_OP0_TABLE[value as usize],
            self::addressing::ADDRESSING_OP1_TABLE[value as usize],
            self::opcode::INST_CYCLE_TABLE[value as usize],
        )
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
