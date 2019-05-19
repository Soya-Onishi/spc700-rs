use super::ram::*;
use super::flags::Flags;
use super::instruction::Instruction;

#[derive(Copy, Clone)]
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

pub struct Spc700 {
    a: u8,
    x: u8,
    y: u8,
    sp: u8,
    psw: Flags,
    pc: u16,
    ram: Ram,
}

impl Spc700 {
    pub fn new() -> Spc700 {
        Spc700 {
            a: 0,
            x: 0,
            y: 0,
            sp: 0xef,
            psw: Flags::new(),
            pc: 0,
            ram: Ram::new(),
        }
    }

    pub fn execute(&mut self) {
        let opcode = self.ram.read(self.read_pc());
        let inst = Instruction::decode(opcode);


    }

    fn read_pc(&mut self) -> u16{
        let pc = self.pc;
        self.pc = pc.wrapping_add(1);

        pc
    }
}