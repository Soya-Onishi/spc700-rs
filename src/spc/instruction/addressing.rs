/*
pub enum Addressing {
  AA,    
  AA_X,
  AA_Y,
  X,
  Y,
  AAAA,
  AAAA_X,
  AAAA_Y,
  IND_AA_X,
  IND_AA_Y,
  AA_B,
  AAA_B,
  STACK,
  
  // These addressings are not described in fullsnes.html but These are probably needed.
  IMM,       
  AA_BB,
  AA_IMM,
  X_WITH_INC,
  X_Y,
  D_A,
  D_X,
  D_Y,
  NONE,      // The opcode is not separted by addressing
}
 */

pub enum Addressing {
  NONE,
  IMM,
  A,
  X,
  Y,
  YA,
  SP,
  PSW,
  PC,
  ABS,
  ABS_X,
  ABS_Y,
  IND_X,
  IND_Y,
  ABS16,
  ABS16_X,
  ABS16_Y,
  IND_ABS_X,
  IND_ABS_Y,
  ABS_B,
  ABS12_B,
  SPECIAL,
}

// This implementation is not good. Probably, there also needs used register table and table of how to use these registers.
use self::Addressing::*;

pub const ADDRESSING_DEST_TABLE: [Addressing; 256] = [
  // 0        1      2   3  4  5  6  7    8      9    A      B      C        D      E        F
  NONE, SPECIAL, ABS_B, PC, A, A, A, A,   A,   ABS, PSW,   ABS, ABS16, SPECIAL, ABS16, SPECIAL, // 0x0X
  PC,   SPECIAL, ABS_B, PC, A, A, A, A, ABS, IND_X, ABS, ABS_X,     A,       X,     X       PC, // 0x1X
  PSW,  SPECIAL, ABS_B, PC, A, A, A, A,   A,   ABS, PSW,   ABS, ABS16, SPECIAL,    PC,      PC, // 0x2X
  PC,   SPECIAL, ABS_B, PC, A, A, A, A, ABS, IND_X, ABS, ABS_X,     A,       X,     X,   ABS16, // 0x3X
  PSW,  SPECIAL, ABS_B, PC, A, A, A, A,   A,   ABS, PSW,   ABS, ABS16, SPECIAL, ABS16, SPECIAL, // 0x4X
];

pub const ADDRESSING_OPERAND_TABLE: [Addressing; 256] = [
  
];

pub const ADDRESSING_TABLE: [Addressing; 256] = [
  // 0     1     2     3     4       5       6         7       8      9      A     B     C     D     E     F
  NONE, NONE, AA_B, AA_B,   AA,   AAAA,      X, IND_AA_X,    IMM, AA_BB, AAA_B,   AA, AAAA, NONE, AAAA,   NONE, // 0x0X
  IMM,  NONE, AA_B, AA_B, AA_X, AAAA_X, AAAA_Y, IND_AA_Y, AA_IMM,   X_Y,    AA, AA_X,  D_A, NONE, AAAA,   AAAA, // 0x1X
  NONE, NONE, AA_B, AA_B,   AA,   AAAA,      X, IND_AA_X,    IMM, AA_BB, AAA_B,   AA, AAAA, NONE,  IMM,    IMM, // 0x2X
  IMM,  NONE, AA_B, AA_B, AA_X, AAAA_X, AAAA_Y, IND_AA_Y, AA_IMM,   X_Y,    AA, AA_X,  D_A, NONE,   AA,   AAAA, // 0x3X
  NONE, NONE, AA_B, AA_B,   AA,   AAAA,      X, IND_AA_X,    IMM, AA_BB, AAA_B,   AA, AAAA, NONE, AAAA,   NONE, // 0x4X
  IMM,  NONE, AA_B, AA_B, AA_X, AAAA_X, AAAA_Y, IND_AA_Y, AA_IMM,   X_Y,    AA, AA_X,  D_A, NONE, AAAA,   AAAA, // 0x5X
  NONE, NONE, AA_B, AA_B,   AA,   AAAA,      X, IND_AA_X,    IMM, AA_BB, AAA_B,   AA, AAAA, NONE,  IMM,   NONE, // 0x6X
  IMM,  NONE, AA_B, AA_B, AA_X, AAAA_X, AAAA_Y, IND_AA_Y, AA_IMM,   X_Y,    AA, AA_X,  D_A, NONE,   AA,   NONE, // 0x7X
  NONE, NONE, AA_B, AA_B,   AA,   AAAA,      X, IND_AA_X,    IMM, AA_BB, AAA_B,   AA, AAAA,  IMM, NONE, AA_IMM, // 0x8X // 0x8D is MOV to Y from [#i]
  IMM,  NONE, AA_B, AA_B, AA_X, AAAA_X, AAAA_Y, IND_AA_Y, AA_IMM,   X_Y,    AA, AA_X,  D_A, NONE, NONE,    D_A, // 0x9X
  NONE, NONE, AA_B, AA_B,   AA,   AAAA,      X, IND_AA_X,    IMM, AA_BB, AAA_B,   AA, AAAA,  IMM, NONE, X_WITH_INC, // 0xAX // 0xAD compare with Y, but not A
  IMM,  NONE, AA_B, AA_B, AA_X, AAAA_X, AAAA_Y, IND_AA_Y, AA_IMM,   X_Y,    AA, AA_X,  D_A, NONE, NONE, X_WITH_INC, // 0xBX
  NONE, NONE, AA_B, AA_B,   AA,   AAAA,      X, IND_AA_X,    IMM, AAAA_X, AAA_B,  AA, AAAA,  IMM, NONE, NONE, // 0xCX  // 0xC8 compare with X, but not A
  IMM,  NONE, AA_B, AA_B, AA_X, AAAA_X, AAAA_Y, IND_AA_Y, AA_IMM,   AA_Y,    AA, AA_X,  NONE, NONE, AA_X, NONE, // 0xDX // 0xDB stores Y value to [AA+X]
  NONE, NONE, AA_B, AA_B,   AA,   AAAA,      X, IND_AA_X,    IMM, AAAA_X, AAA_B,  AA, AAAA, NONE, NONE, NONE, // 0xEX
  IMM,  NONE, AA_B, AA_B, AA_X, AAAA_X, AAAA_Y, IND_AA_Y, AA_IMM,   AA_Y, AA_BB, AA_X, NONE, NONE, IMM, NONE, // 0xFX 
];

