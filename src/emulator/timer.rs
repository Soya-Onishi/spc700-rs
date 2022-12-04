#[derive(Copy, Clone)]
pub struct Timer {
  pub enable: bool,
  pub cycle_counter: u16,
  max_cycle: u16,
  pub divided: u16,
  // next_divider: u8,
  pub divider: u16,
  pub out: u8,
}

impl Timer {
  pub fn new(hz: u32) -> Timer {
    let max_cycle = match hz {
      8000  => 256,
      64000 => 32,
      _ => panic!("{} is invalid, require 8000 or 64000", hz),
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

  pub fn new_with_init(hz: u32, divider: u8, out: u8) -> Timer {
    let mut timer = Timer::new(hz);
    timer.divider = if divider == 0 { 256 } else { divider as u16 };
    timer.out = out % 16;

    timer
  }

  pub fn cycles(&mut self, cycle: u16) -> () {
    // 分岐処理を使わせないためにcycleを加算するか0を加算するかという形にしている。
    // 素直な書き方では 
    // self.enable { 
    //   self.cycle_counter += cycle 
    // }
    // となる。
    // 以下のis_reach_maxやis_divider_maxの部分も同様の理由。
    let cycle = if self.enable { cycle } else { 0 };
    self.cycle_counter += cycle;
    
    let is_reach_max = self.cycle_counter >= self.max_cycle;
    let subtraction_cycles = if is_reach_max { self.max_cycle } else { 0 };
    let divided_count_up = if is_reach_max { 1 } else { 0 };
    self.cycle_counter -= subtraction_cycles;
    self.divided += divided_count_up;

    let is_divider_max = self.divided >= self.divider;
    let next_divided_value = if is_divider_max { 0 } else { self.divided };
    let next_out_operand = if is_divider_max { 1 } else { 0 };
    self.divided = next_divided_value;
    self.out = (self.out + next_out_operand) % 16; 
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
    self.divider = if data == 0 { 256 } else { data as u16 };    
  }
}