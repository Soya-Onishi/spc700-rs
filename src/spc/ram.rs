use super::core::Spc700;

/*
pub struct Ram<'a> {
    core: Option<&'a Spc700<'a>>,
    ram: u8,
}
*/
/*
pub trait RamManipulate<T> {
    fn read_ram(&self, addr: T) -> u8;
    fn write_ram(&mut self, addr: T, data: u8);
}

impl RamManipulate<u8> for Spc700 {
    fn read_ram(&self, addr: u8) -> u8 {
        self.ram
    }

    fn write_ram(&mut self, addr: u8, data: u8) {

    }
}

impl RamManipulate<u16> for Spc700 {
    fn read_ram(&self, addr: u16) -> u8 {
        self.ram
    }

    fn write_ram(&mut self, addr: u16, data: u8) {

    }
}
*/

pub struct Ram {
    ram: u8
}

impl Ram {
    pub fn new() -> Ram {
        Ram {
            ram: 0
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        0
    }

    pub fn write(&self, addr: u16, data: u8) {

    }
}

/*
impl<'a> Ram<'a> {
    pub fn new() -> Ram<'a> {
        Ram {
            core: None,
            ram: 0,
        }
    }

    pub fn reg_core(&mut self, spc: &'a Spc700) {
        self.core = Some(spc)
    }
}
*/