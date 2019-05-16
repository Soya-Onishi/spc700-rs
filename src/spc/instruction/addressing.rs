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

#[derive(Copy, Clone)]
pub enum PSWBit {
    N,
    VH,
    P,
    B,
    I,
    Z,
    C,
    ALL,
}

#[derive(Copy, Clone)]
pub enum Addressing {
    None,
    Imm,
    A,
    X,
    Y,
    YA,
    SP,
    PSW(PSWBit),
    Abs,
    AbsX,
    AbsY,
    IndX,
    IndY,
    Abs16,
    Abs16X,
    Abs16Y,
    IndAbsX,
    IndAbsY,
    AbsB,
    Abs12B,
    Special,
}

// This implementation is not good. Probably, there also needs used register table and table of how to use these registers.
use self::Addressing::*;
use self::PSWBit::*;

pub const ADDRESSING_OP0_TABLE: [Addressing; 256] = [
    //   0         1     2     3      4        5        6          7     8      9        A      B      C        D        E        F
    None, Special, AbsB, AbsB, A, A, A, A, A, Abs, PSW(C), Abs, Abs16, PSW(ALL), Abs16, Special, // 0x0X
    PSW(N), Special, AbsB, AbsB, A, A, A, A, Abs, IndX, Abs, AbsX, A, X, X, None, // 0x1X
    PSW(P), Special, AbsB, AbsB, A, A, A, A, A, Abs, PSW(C), Abs, Abs16, A, Abs, None, // 0x2X
    PSW(N), Special, AbsB, AbsB, A, A, A, A, Abs, IndX, Abs, AbsX, A, X, X, Special, // 0x3X
    PSW(P), Special, AbsB, AbsB, A, A, A, A, A, Abs, PSW(C), Abs, Abs16, X, Abs16, Special, // 0x4X
    PSW(VH), Special, AbsB, AbsB, A, A, A, A, Abs, IndX, YA, AbsX, A, X, Y, None, // 0x5X
    PSW(C), Special, AbsB, AbsB, A, A, A, A, A, Abs, PSW(C), Abs, Abs16, Y, Abs, Special, // 0x6X
    PSW(VH), Special, AbsB, AbsB, A, A, A, A, Abs, IndX, YA, AbsX, A, A, Y, Special, // 0x7X
    PSW(C), Special, AbsB, AbsB, A, A, A, A, A, Abs, PSW(C), Abs, Abs16, Y, PSW(ALL), Abs, // 0x8X
    PSW(C), Special, AbsB, AbsB, A, A, A, A, Abs, IndX, YA, AbsX, A, X, YA, A, // 0x9X
    PSW(I), Special, AbsB, AbsB, A, A, A, A, A, Abs, PSW(C), Abs, Abs16, Y, A, Special, // 0xAX
    PSW(C), Special, AbsB, AbsB, A, A, A, A, Abs, IndX, YA, AbsX, A, SP, A, Special, // 0xBX
    PSW(I), Special, AbsB, AbsB, Abs, Abs16, IndX, IndAbsX, X, Abs16, Abs12B, Abs, Abs16, X, X, YA, // 0xCX
    PSW(Z), Special, AbsB, AbsB, AbsX, Abs16X, Abs16Y, IndAbsY, Abs, AbsY, Abs, AbsX, Y, A, AbsX, A, // 0xDX
    PSW(VH), Special, AbsB, AbsB, A, A, A, A, A, X, Abs12B, Y, Y, PSW(C), Y, None, // 0xEX
    PSW(Z), Special, AbsB, AbsB, A, A, A, A, X, X, Abs, Y, Y, Y, Y, None, // 0xFX
];

pub const ADDRESSING_OP1_TABLE: [Addressing; 256] = [
    // 0     1     2    3      4        5        6          7    8      9        A      B      C     D      E        F
    None, None, None, Imm, Abs, Abs16, IndX, IndAbsX, Imm, Abs, Abs12B, None, None, None, A, None, // 0x0X
    Imm, None, None, Imm, AbsX, Abs16X, Abs16Y, IndAbsY, Imm, Abs, None, None, None, None, Abs16, Abs16X, // 0x1X
    None, None, None, Imm, Abs, Abs16, IndX, IndAbsX, Imm, Abs, Abs12B, None, None, None, Imm, Imm, // 0x2X
    Imm, None, None, Imm, AbsX, Abs16X, Abs16Y, IndAbsY, Imm, Abs, None, None, None, None, Abs, None, // 0x3X
    None, None, None, Imm, Abs, Abs16, IndX, IndAbsX, Imm, Abs, Abs12B, None, None, None, A, None, // 0x4X
    Imm, None, None, Imm, AbsX, Abs16X, Abs16Y, IndAbsY, Imm, Abs, None, None, None, A, Abs16, Abs16, // 0x5X
    None, None, None, Imm, Abs, Abs16, IndX, IndAbsX, Imm, Abs, Abs12B, None, None, None, Imm, None, // 0x6X
    Imm, None, None, Imm, AbsX, Abs16X, Abs16Y, IndAbsY, Imm, Abs, None, None, None, X, Abs, None, // 0x7X
    None, None, None, Imm, Abs, Abs16, IndX, IndAbsX, Imm, Abs, Abs12B, None, None, Imm, None, Imm, // 0x8X
    Imm, None, None, Imm, AbsX, Abs16X, Abs16Y, IndAbsY, Imm, Abs, None, None, None, SP, X, None, // 0x9X
    None, None, None, Imm, Abs, Abs16, IndX, IndAbsX, Imm, Abs, Abs12B, None, None, Imm, None, None, // 0xAX
    Imm, None, None, Imm, AbsX, Abs16X, Abs16Y, IndAbsY, Imm, Abs, None, None, None, X, None, None, // 0xBX
    None, None, None, Imm, A, A, A, A, Imm, X, PSW(C), Y, Y, Imm, None, None, // 0xCX
    Imm, None, None, Imm, A, A, A, A, X, X, YA, Y, None, Y, Imm, None, // 0xDX
    None, None, None, Imm, Abs, Abs16, IndX, IndAbsX, Imm, Abs16, None, Abs, Abs16, None, None, None, // 0xEX
    Imm, None, None, Imm, AbsX, Abs16X, Abs16Y, IndAbsY, Abs, AbsY, Abs, AbsX, None, A, Imm, None, // 0xFX
];
/*
pub const ADDRESSING_TABLE: [Addressing; 256] = [
  // 0     1     2     3     4       5       6         7       8      9      A     B     C     D     E     F
  None, None, AA_B, AA_B,   AA,   AAAA,      X, IND_AA_X,    Imm, AA_BB, AAA_B,   AA, AAAA, None, AAAA,   None, // 0x0X
  Imm,  None, AA_B, AA_B, AA_X, AAAA_X, AAAA_Y, IND_AA_Y, AA_Imm,   X_Y,    AA, AA_X,  D_A, None, AAAA,   AAAA, // 0x1X
  None, None, AA_B, AA_B,   AA,   AAAA,      X, IND_AA_X,    Imm, AA_BB, AAA_B,   AA, AAAA, None,  Imm,    Imm, // 0x2X
  Imm,  None, AA_B, AA_B, AA_X, AAAA_X, AAAA_Y, IND_AA_Y, AA_Imm,   X_Y,    AA, AA_X,  D_A, None,   AA,   AAAA, // 0x3X
  None, None, AA_B, AA_B,   AA,   AAAA,      X, IND_AA_X,    Imm, AA_BB, AAA_B,   AA, AAAA, None, AAAA,   None, // 0x4X
  Imm,  None, AA_B, AA_B, AA_X, AAAA_X, AAAA_Y, IND_AA_Y, AA_Imm,   X_Y,    AA, AA_X,  D_A, None, AAAA,   AAAA, // 0x5X
  None, None, AA_B, AA_B,   AA,   AAAA,      X, IND_AA_X,    Imm, AA_BB, AAA_B,   AA, AAAA, None,  Imm,   None, // 0x6X
  Imm,  None, AA_B, AA_B, AA_X, AAAA_X, AAAA_Y, IND_AA_Y, AA_Imm,   X_Y,    AA, AA_X,  D_A, None,   AA,   None, // 0x7X
  None, None, AA_B, AA_B,   AA,   AAAA,      X, IND_AA_X,    Imm, AA_BB, AAA_B,   AA, AAAA,  Imm, None, AA_Imm, // 0x8X // 0x8D is MOV to Y from [#i]
  Imm,  None, AA_B, AA_B, AA_X, AAAA_X, AAAA_Y, IND_AA_Y, AA_Imm,   X_Y,    AA, AA_X,  D_A, None, None,    D_A, // 0x9X
  None, None, AA_B, AA_B,   AA,   AAAA,      X, IND_AA_X,    Imm, AA_BB, AAA_B,   AA, AAAA,  Imm, None, X_WITH_INC, // 0xAX // 0xAD compare with Y, but not A
  Imm,  None, AA_B, AA_B, AA_X, AAAA_X, AAAA_Y, IND_AA_Y, AA_Imm,   X_Y,    AA, AA_X,  D_A, None, None, X_WITH_INC, // 0xBX
  None, None, AA_B, AA_B,   AA,   AAAA,      X, IND_AA_X,    Imm, AAAA_X, AAA_B,  AA, AAAA,  Imm, None, None, // 0xCX  // 0xC8 compare with X, but not A
  Imm,  None, AA_B, AA_B, AA_X, AAAA_X, AAAA_Y, IND_AA_Y, AA_Imm,   AA_Y,    AA, AA_X,  None, None, AA_X, None, // 0xDX // 0xDB stores Y value to [AA+X]
  None, None, AA_B, AA_B,   AA,   AAAA,      X, IND_AA_X,    Imm, AAAA_X, AAA_B,  AA, AAAA, None, None, None, // 0xEX
  Imm,  None, AA_B, AA_B, AA_X, AAAA_X, AAAA_Y, IND_AA_Y, AA_Imm,   AA_Y, AA_BB, AA_X, None, None, Imm, None, // 0xFX 
];
*/

