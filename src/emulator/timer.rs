#[derive(Copy, Clone)]
pub struct Timer {
  pub enable: bool,
  pub cycle_counter: u16,
  max_cycle: u16,
  pub divided: u16,
  // next_divider: u8,
  divider: u8,
  pub out: u8,
}

impl Timer {
  pub fn new(hz: u32) -> Timer {
    let max_cycle = match hz {
      8000  => 256,
      64000 => 32,
      _ => panic!(format!("{} is invalid, require 8000 or 64000", hz)),
    };

    Timer {
      enable: false,
      cycle_counter: 0,
      max_cycle: max_cycle,
      divided: 0,
      // next_divider: 0,
      divider: 0,
      out: 0,
    }
  }  

  pub fn cycles(&mut self, cycle: u16) -> () {
    if self.enable {
      self.cycle_counter += cycle;

      if self.cycle_counter >= self.max_cycle {
        self.cycle_counter -= self.max_cycle;
        self.divided += 1;

        let divider = if self.divider == 0 { 256 } else { self.divider as u16 };

        if self.divided >= divider {
          self.divided = 0;
          self.out = (self.out + 1) & 0xF;
        }
      }    
    }    
  }

  pub fn enable(&mut self) -> () {
    self.enable = true;    
    self.divided = 0;  
    self.cycle_counter = 0;  
    // self.divider = self.next_divider;
  }

  pub fn disable(&mut self) -> () {
    self.enable = false;    
    self.out = 0;    
  }

  pub fn read_out(&mut self) -> u8 {
    let out = self.out;
    self.out = 0;
    out
  }

  pub fn write_divider(&mut self, data: u8) -> () {    
    self.divider = data;    
  }
}