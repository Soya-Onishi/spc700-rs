const RATE_RANGE = 30720;
const INFINITE_RATE = 32;

const ADSR_GAIN_RATES: [u16; 33] = [
  RATE_RANGE + 1,
  2048, 1536, 1280, 1024,
  768,  640,  512,  384,
  320,  256,  192,  160,
  128,  96,   80,   64,
  48,   40,   32,   24,
  20,   16,   12,   10,
  8,    6,    5,    4,
  3,    2,    1,    1,
];

enum Mode {
  Attack,
  Decay,
  Sustain,
  Release,
}

struct Envelope {
  adsr1: mut u8,
  adsr2: mut u8,
  gain: mut u8,
  level: mut u16,
  out: mut u8,
  mode: mut Mode
}

impl Envelope {  
  fn update_by_adsr(&self) -> (u8, i16) {
    match self.mode {
      Mode::Attack => {
        let attack_rate = self.adsr1 & 0b1111;
        let rate = (attack_rate << 1) + 1;
        let step = if rate == 31 { 1024 } else { 32 };
        
        (rate, step)
      }
      Mode::Decay => {
        let decay_rate = (self.adsr1 >> 4) & 0b0111;
        let rate = (decay_rate << 1) + 16;
        let step = -(((self.env - 1) >> 8) + 1);

        (rate, step)
      }
      Mode::Sustain => {
        let rate = self.adsr2 & 0b1_1111;
        let step = -(((self.env - 1) >> 8) + 1);
        
        (rate, step)
      }
      Mode::Release => {
        (31, -8)
      }
    }
  }

  fn update_by_gain(&self) -> (u8, i16){
    let is_direct = ((self.gain >> 7) & 1)  == 0;
      let mut next_env: u16;
      let mut rate: u8;
      
      if is_direct {
        let new_level = (self.gain & 0b0111_1111) * 16;
        
        (INFINITE_RATE, self.level - new_level)
      } else {
        let rate: u8 = self.gain & 0b1_1111;
        let mode = (self.gain >> 5) & 0b11;

        let step: i8 = match mode {
          0 => { -32 }                                     // Linear Decrease
          1 => { -(((self.level - 1) >> 8) + 1) }          // Exp Decrease
          2 => {  32 }                                     // Linear Increase
          3 => { if self.level < 0x600 { 32 } else { 8 } } // Bent Increase
        }

        (rate, step)
      }
  }

  fn clip_level(&self, step: i16) -> i16 {
    let new_level: i16 = self.level + step;

    if(new_level < 0) {
      if(step < 0) {
        0
      } else {
        0x7ff
      }      
    } else {
      new_level
    }
  }

  fn is_required_to_renew(&self, rate: u8) -> bool {
    self.dsp.counter % ADSR_GAIN_RATES[rate] == 0
  }
  
  pub fn tick(&self) {
    let is_adsr_mode = ((adsr1 >> 7) & 1) == 1;

    let (rate, step) = 
      if is_adsr_mode {
        self.update_by_adsr()      
      } else {
        self.update_by_gain()
      }

    let new_level = clip_level(step);
    
    if is_required_to_renew(rate) {
      self.level = new_level;

      self.update_phase();

      
    }
    // TODO: Renew level only if cycle counter match rate
    // if dsp counter match rate {
    //   self.level += ...
    //   
    // }            

  }
}
