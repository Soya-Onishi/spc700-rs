use super::DSPRegister;
use super::DSPBlock;
use super::CYCLE_RANGE;

const ADSR_GAIN_RATES: [u16; 32] = [    
    CYCLE_RANGE + 1, 2048, 1536, 1280, 
    1024, 768, 640, 512, 
    384, 320, 256, 192, 
    160, 128, 96, 80, 
    64, 48, 40, 32, 
    24, 20, 16, 12, 
    10, 8, 6, 5, 
    4, 3, 2, 1,
];

const COUNTER_OFFSETS: [u16; 32] = [
	  1, 0, 1040,
	536, 0, 1040,
	536, 0, 1040,
	536, 0, 1040,
	536, 0, 1040,
	536, 0, 1040,
	536, 0, 1040,
	536, 0, 1040,
	536, 0, 1040,
	536, 0, 1040,
	     0,
         0
];

#[derive(PartialEq, Copy, Clone)]
pub enum ADSRMode {
    Attack,
    Decay,
    Sustain,
    Release,
}

#[derive(Copy, Clone)]
enum GainMode {
    LinearDecrease,
    ExpDecrease,
    LinearIncrease,
    BentIncrease,
}

#[derive(Copy, Clone)]
pub struct Envelope {
    pub level: i16,    
    pub adsr_mode: ADSRMode,    
}

impl Envelope {    
    pub fn new(level: i16, adsr_mode: ADSRMode) -> Envelope {
        Envelope { level, adsr_mode }
    }

    pub fn empty() -> Envelope {
        Envelope::new(0, ADSRMode::Release)
    }

    pub fn envelope(&self, dsp: &DSPBlock, cycle_count: u16) -> Envelope {
        let is_adsr_mode = (dsp.reg.adsr & 0x80) > 0;

        let (rate, step) =
            if is_adsr_mode || self.adsr_mode == ADSRMode::Release {
                update_envelope_with_adsr(self, &dsp.reg)
            } else {
                update_envelope_with_gain(self, &dsp.reg)
            };

        let new_level = clip_level(self.level as i16, step);
        let new_mode = refresh_mode(new_level, &dsp.reg, self.adsr_mode);

        if is_require_renew(cycle_count, rate) {  
            Envelope::new(new_level, new_mode)
        } else {
            Envelope::new(self.level, new_mode)
        }
    }    
}

fn update_envelope_with_adsr(env: &Envelope, reg: &DSPRegister) -> (Option<usize>, i16) {
    let (rate, step) = match env.adsr_mode {
        ADSRMode::Attack => {
            let attack_rate = reg.adsr & 0x0F;
            let rate = (attack_rate << 1) + 1;
            let step = if rate == 31 { 1024 } else { 32 };

            (rate, step)
        }
        ADSRMode::Decay => {
            let decay_rate = (reg.adsr >> 4) & 0b0111;
            let rate = (decay_rate << 1) + 16;
            // fullsnes say -(((env.level as i16 - 1) >> 8) + 1);
            // but snes9x implement like below
            let step = -(((env.level as i16 - 1) >> 8) + 1);

            (rate, step)
        }
        ADSRMode::Sustain => {
            let rate = (reg.adsr >> 8) & 0b11111;
            // like above comment
            let step = -(((env.level as i16 - 1) >> 8) + 1);

            (rate, step)
        }
        ADSRMode::Release => {
            (31, -8)
        }
    };

    (Some(rate as usize), step)
}

fn update_envelope_with_gain(env: &Envelope, reg: &DSPRegister) -> (Option<usize>, i16) {
    let is_direct = ((reg.gain >> 7) & 1) == 0;

    let (rate, step) = if is_direct {        
        (31, (reg.gain as i16 & 0b0111_1111) * 16)
    } else {
        let rate = reg.gain & 0x1F;
        let step = match get_gain_mode(reg.gain) {
            GainMode::LinearDecrease => -32,
            GainMode::ExpDecrease => -(((env.level as i16 - 1) >> 8) + 1), // same as above comment
            GainMode::LinearIncrease => 32,
            GainMode::BentIncrease => if env.level < 0x600 { 32 } else { 8 }
        };
    
        (rate, step)
    };

    (Some(rate as usize), step)
}

fn is_require_renew(counter: u16, rate: Option<usize>) -> bool {
    match rate {
        None => true,
        Some(rate) => ((counter + COUNTER_OFFSETS[rate]) % ADSR_GAIN_RATES[rate]) == 0,
    }    
}

fn clip_level(current: i16, step: i16) -> i16 {
    let new_level = (current as i32) + (step as i32);
    
    let level = 
        if new_level < 0 { 0 }
        else if new_level > 0x7ff { 0x7ff }
        else { new_level };

    level as i16
}

fn get_gain_mode(flag: u8) -> GainMode {
    match (flag >> 5) & 3 {
        0 => GainMode::LinearDecrease,
        1 => GainMode::ExpDecrease,
        2 => GainMode::LinearIncrease,
        3 => GainMode::BentIncrease,
        _ => panic!("gain mode value must be between 0 to 3"),
    }
}

fn refresh_mode(env: i16, reg: &DSPRegister, current: ADSRMode) -> ADSRMode {
    let boundary = (((reg.adsr >> 13) & 7) + 1) * 0x100;

    match current {
        ADSRMode::Attack if env >= 0x7E0 =>  ADSRMode::Decay,
        ADSRMode::Decay  if env <= boundary as i16 => ADSRMode::Sustain,
        others => others,
    }
}