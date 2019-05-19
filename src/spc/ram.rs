use super::core::Spc700;

pub struct Ram {
    ram: u8,
}

pub trait RamManipulate<T> {
    fn read(&self, addr: T) -> u8;
    fn write(&mut self, addr: T, data: u8);
}

impl RamManipulate<u8> for Ram {
    fn read(&self, addr: u8) -> u8 {

    }

    fn write(&mut self, addr: u8, data: u8) {

    }
}

impl RamManipulate<u16> for Ram {
    fn read(&self, addr: u16) -> u8 {

    }

    fn write(&mut self, addr: u16, data: u8) {

    }
}

impl Ram {
    pub fn new() -> Ram {
        Ram {
            ram: 0,
        }
    }
}