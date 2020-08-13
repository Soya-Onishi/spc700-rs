#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Opcode {
    // Move/Load/Store Commands
    MOV,
    MOVW,

    PUSH,
    POP,

    // ALU Commands
    OR,
    AND,
    EOR,
    CMP,
    ADC,
    SBC,

    ASL,
    ROL,
    LSR,
    ROR,
    DEC,
    INC,

    ADDW,
    SUBW,
    CMPW,
    INCW,
    DECW,
    DIV,
    MUL,

    CLR1,
    SET1,
    NOT1,
    MOV1,
    OR1,
    AND1,
    EOR1,
    CLRC,
    SETC,
    NOTC,
    CLRV,

    DAA,
    DAS,
    XCN,
    TCLR1,
    TSET1,

    // Jump/Control(Conditional Jump) Commands
    BPL,
    BMI,
    BVC,
    BVS,
    BCC,
    BCS,
    BNE,
    BEQ,
    BBS,
    BBC,
    CBNE,
    DBNZ,

    BRA,
    JMP,
    CALL,
    TCALL,
    PCALL,
    RET,
    RETI,
    BRK,

    NOP,
    SLEEP,
    STOP,
    CLRP,
    SETP,
    EI,
    DI,
}

use self::Opcode::*;

pub const OPCODE_TABLE: [Opcode; 256] = [
    //0        1     2    3    4    5    6    7    8    9           A     B     C      D       E      F
    NOP, TCALL, SET1, BBS, OR, OR, OR, OR, OR, OR, OR1, ASL, ASL, PUSH, TSET1, BRK, // 0x0X
    BPL, TCALL, CLR1, BBC, OR, OR, OR, OR, OR, OR, DECW, ASL, ASL, DEC, CMP, JMP, // 0x1X
    CLRP, TCALL, SET1, BBS, AND, AND, AND, AND, AND, AND, OR1, ROL, ROL, PUSH, CBNE, BRA, // 0x2X
    BMI, TCALL, CLR1, BBC, AND, AND, AND, AND, AND, AND, INCW, ROL, ROL, INC, CMP, CALL, // 0x3X
    SETP, TCALL, SET1, BBS, EOR, EOR, EOR, EOR, EOR, EOR, AND1, LSR, LSR, PUSH, TCLR1, PCALL, // 0x4X
    BVC, TCALL, CLR1, BBC, EOR, EOR, EOR, EOR, EOR, EOR, CMPW, LSR, LSR, MOV, CMP, JMP, // 0x5X
    CLRC, TCALL, SET1, BBS, CMP, CMP, CMP, CMP, CMP, CMP, AND1, ROR, ROR, PUSH, DBNZ, RET, // 0x6X
    BVS, TCALL, CLR1, BBC, CMP, CMP, CMP, CMP, CMP, CMP, ADDW, ROR, ROR, MOV, CMP, RETI, // 0x7X
    SETC, TCALL, SET1, BBS, ADC, ADC, ADC, ADC, ADC, ADC, EOR1, DEC, DEC, MOV, POP, MOV, // 0x8X
    BCC, TCALL, CLR1, BBC, ADC, ADC, ADC, ADC, ADC, ADC, SUBW, DEC, DEC, MOV, DIV, XCN, // 0x9X
    EI, TCALL, SET1, BBS, SBC, SBC, SBC, SBC, SBC, SBC, MOV1, INC, INC, CMP, POP, MOV, // 0xAX
    BCS, TCALL, CLR1, BBC, SBC, SBC, SBC, SBC, SBC, SBC, MOVW, INC, INC, MOV, DAS, MOV, // 0xBX
    DI, TCALL, SET1, BBS, MOV, MOV, MOV, MOV, CMP, MOV, MOV1, MOV, MOV, MOV, POP, MUL, // 0xCX
    BNE, TCALL, CLR1, BBC, MOV, MOV, MOV, MOV, MOV, MOV, MOVW, MOV, DEC, MOV, CBNE, DAA, // 0xDX
    CLRV, TCALL, SET1, BBS, MOV, MOV, MOV, MOV, MOV, MOV, NOT1, MOV, MOV, NOT1, POP, SLEEP, // 0xEX
    BEQ, TCALL, CLR1, BBC, MOV, MOV, MOV, MOV, MOV, MOV, MOV, MOV, INC, MOV, DBNZ, STOP, // 0xFX
];

pub const INST_CYCLE_TABLE: [u16; 256] = [
    //0  1  2  3  4  5  6  7  8  9  A  B  C  D  E  F
    2, 8, 4, 5, 3, 4, 3, 6, 2, 6, 5, 4, 5, 4, 6, 8, // 0x0X
    2, 8, 4, 5, 4, 5, 5, 6, 5, 5, 6, 5, 2, 2, 4, 6, // 0x1X
    2, 8, 4, 5, 3, 4, 3, 6, 2, 6, 5, 4, 5, 4, 5, 4, // 0x2X
    2, 8, 4, 5, 4, 5, 5, 6, 5, 5, 6, 5, 2, 2, 3, 8, // 0x3X
    2, 8, 4, 5, 3, 4, 3, 6, 2, 6, 4, 4, 5, 4, 6, 6, // 0x4X
    2, 8, 4, 5, 4, 5, 5, 6, 5, 5, 4, 5, 2, 2, 4, 3, // 0x5X
    2, 8, 4, 5, 3, 4, 3, 6, 2, 6, 4, 4, 5, 4, 5, 5, // 0x6X
    2, 8, 4, 5, 4, 5, 5, 6, 5, 5, 5, 5, 2, 2, 3, 6, // 0x7X
    2, 8, 4, 5, 3, 4, 3, 6, 2, 6, 5, 4, 5, 2, 4, 5, // 0x8X
    2, 8, 4, 5, 4, 5, 5, 6, 5, 5, 5, 5, 2, 2, 12, 5, // 0x9X
    3, 8, 4, 5, 3, 4, 3, 6, 2, 6, 4, 4, 5, 2, 4, 4, // 0xAX
    2, 8, 4, 5, 4, 5, 5, 6, 5, 5, 5, 5, 2, 2, 3, 4, // 0xBX
    3, 8, 4, 5, 4, 5, 4, 7, 2, 5, 6, 4, 5, 2, 4, 9, // 0xCX
    2, 8, 4, 5, 5, 6, 6, 7, 4, 5, 5, 5, 2, 2, 6, 3, // 0xDX
    2, 8, 4, 5, 3, 4, 3, 6, 2, 4, 5, 3, 4, 3, 4, 3, // 0xEX
    2, 8, 4, 5, 4, 5, 5, 6, 3, 4, 5, 4, 2, 2, 5, 3, // 0xFX
];
