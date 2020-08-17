extern crate cpal;
extern crate hound;

use crate::emulator::core::Spc700;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::collections::VecDeque;
use std::f32;
use std::f64;
use std::sync;
use std::thread;

const INPUT_SAMPLING_RATE: usize = 32000;
const BUFFER_SIZE: usize = INPUT_SAMPLING_RATE * 8;
const EDGE_SAMPLES: usize = 129;
const INPUT_SAMPLES: usize = 4096;

pub struct Amplifier;
impl Amplifier {
  pub fn play(core: Spc700) -> ! {
    let buffer = sync::Arc::new(sync::Mutex::new(RingBuffer::<i16>::new(BUFFER_SIZE)));

    let (device, config) = build_config();
    let format = config.sample_format();
    let config = cpal::StreamConfig {
      channels: config.channels(),
      buffer_size: cpal::BufferSize::Default,
      sample_rate: cpal::SampleRate(32000),
    };
    
    let stream = match format {
      cpal::SampleFormat::F32 => {
        build_stream::<f32>(&device, &config, buffer.clone())
      }
      cpal::SampleFormat::I16 => {
        build_stream::<i16>(&device, &config, buffer.clone())
      }
      cpal::SampleFormat::U16 => {
        build_stream::<u16>(&device, &config, buffer.clone())
      }
    };

    // thread for creating samples
    let mut core = core;
    {
      let buffer = buffer.clone();
      thread::spawn(move || {
        let threshold = EDGE_SAMPLES + INPUT_SAMPLES + EDGE_SAMPLES;
        let mut queue = VecDeque::<(i16, i16)>::new();                        
        
        loop {          
          if threshold * 4 > queue.len() {
            let outputs = core.next_sample();
            queue.push_front(outputs);
          }

          if queue.len() >= threshold {
            let is_writable = {
              let buf = buffer.lock().unwrap();
              buf.is_writable()
            };

            if is_writable {
              let start = queue.len() - threshold;
              let end = queue.len();

              let (lefts, rights) = (start..end).rev().map(|idx| queue[idx]).unzip();
              queue.truncate(queue.len() - threshold);             

              {
                let mut buf = buffer.lock().unwrap();
                buf.write(&lefts, &rights);
              }
            }
          }
        }
      });
    }

    stream.play().unwrap();

    loop {}
  }
}

fn only_dec(x: f64) -> f64 {  
  x - (x as i32) as f64
}

struct RingBuffer<T: Clone + Copy> {
  buf_size: usize,
  pub vector: Vec<(T, T)>,
  right: VecDeque<T>,
  left: VecDeque<T>,
}

impl<T: Clone + Copy> RingBuffer<T> {
  pub fn new(buf_size: usize) -> RingBuffer<T> {
    RingBuffer {
      buf_size: buf_size,
      vector: Vec::new(),
      left: VecDeque::new(),
      right: VecDeque::new(),
    }
  }

  pub fn is_writable(&self) -> bool {
    self.buf_size >= self.left.len() && self.buf_size >= self.left.len()
  }

  pub fn is_readable(&self, requirements: usize) -> bool {
    self.left.len() >= requirements && self.right.len() >= requirements
  }

  pub fn write(&mut self, lefts: &Vec<T>, rights: &Vec<T>) -> () {
    lefts.iter().for_each(|&v| self.left.push_front(v));
    rights.iter().for_each(|&v| self.right.push_front(v));
  }

  pub fn read(&mut self, requirements: usize) -> Vec<(T, T)> {
    let start = self.left.len() - requirements;
    let end = self.left.len();

    let lefts: Vec<T> = (start..end).rev().map(|idx| self.left[idx]).collect();
    let rights: Vec<T> = (start..end).rev().map(|idx| self.right[idx]).collect();
    self.left.truncate(start);
    self.right.truncate(start);

    let tuples: Vec<(T, T)> = lefts
      .iter()
      .zip(rights.iter())
      .map(|(&l, &r)| (l, r))
      .collect();
    tuples.iter().for_each(|&tuple| self.vector.push(tuple));

    tuples
  }
}

fn build_config() -> (cpal::Device, cpal::SupportedStreamConfig) {
  let host = cpal::default_host();
  let device = host
    .default_output_device()
    .expect("no output device available");
  let config = device
    .default_output_config()
    .expect("error while querying configs");

  (device, config)
}

fn build_stream<T: cpal::Sample>(
  device: &cpal::Device,
  config: &cpal::StreamConfig,
  buffer: sync::Arc<sync::Mutex<RingBuffer<i16>>>,
) -> cpal::Stream {
  let sampling_rate = config.sample_rate.0 as u32;
  let channels = config.channels as usize;
  let err_fn = |err| eprintln!("an error occurred on stream: {}", err);  

  let stream = device
    .build_output_stream(
      config,
      move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
        let require_length = data.len() / channels;

        loop {
          let is_readable = {
            let buf = buffer.lock().unwrap();
            buf.is_readable(require_length)
          };

          if is_readable {
            break;
          }
        }

        let read_data = {
          let mut buf = buffer.lock().unwrap();
          buf.read(require_length)
        };

        assign_output(data, read_data, channels, sampling_rate);        
      },
      err_fn,
    )
    .expect("unexpected error when building stream");
  stream
}

fn assign_output<T: cpal::Sample>(data: &mut [T], output: Vec<(i16, i16)>, channels: usize, sampling_rate: u32) -> () {
  static mut LEFT_IN: [f32; 2] = [0.0; 2];
  static mut RIGHT_IN: [f32; 2] = [0.0; 2];
  static mut LEFT_OUT: [f32; 2] = [0.0; 2];
  static mut RIGHT_OUT: [f32; 2] = [0.0; 2];  

  let left_in = [0.0; 2];
  let right_in = [0.0; 2];
  let left_out = [0.0; 2];
  let right_out = [0.0; 2];

  let q = 1.0 / f32::sqrt(2.0);
  let freq = sampling_rate as f32 / 2.0;
  let omega = std::f32::consts::PI;
  let alpha = 2.0 * q;

  let a0 =  1.0 + alpha;
  let a1 =  0.0; //-2.0 * f32::cos(omega);
  let a2 =  1.0 - alpha;
  let b0 =  0.5; //(1.0 - f32::cos(omega)) / 2.0;
  let b1 =  1.0; // 1.0 - f32::cos(omega);
  let b2 =  0.5; // (1.0 - f32::cos(omega)) / 2.0;

  let a = [a0, a1, a2];
  let b = [b0, b1, b2];

  if channels == 1 {
    let outputs = output.iter().map(|(left, right)| (left + right) / 2);
    for (sample, output) in data.iter_mut().zip(outputs) {
      *sample = cpal::Sample::from(&output);
    }    
  } else {
    let each_len = channels / 2;
    let lefts: Vec<i16> = output.iter().map(|&(left, _)| left).collect();
    let rights: Vec<i16> = output.iter().map(|&(_, right)| right).collect();
    // let (lefts, left_in, left_out) = low_pass_filter(lefts, left_in, left_out, a, b);
    // let (rights, right_in, right_out) = low_pass_filter(rights, right_in, right_out, a, b);
    let left: Vec<T> = lefts.iter().map(|left| cpal::Sample::from(left)).collect();
    let right: Vec<T> = rights.iter().map(|right| cpal::Sample::from(right)).collect();
    unsafe {
      LEFT_IN   = left_in;
      RIGHT_IN  = right_in;
      LEFT_OUT  = left_out;
      RIGHT_OUT = right_out;
    }

    for (frame, idx) in data.chunks_mut(channels).zip(0..) {      
      for (sample, ch) in frame.iter_mut().zip(0..) {
        if ch < each_len { *sample = left[idx] }
        else             { *sample = right[idx] }        
      }
    }
  }
}

fn low_pass_filter(inputs: Vec<f32>, last_in: [f32; 2], last_out: [f32; 2], a: [f32; 3], b: [f32; 3]) -> (Vec<f32>, [f32; 2], [f32; 2]) {  
  let mut last_in = last_in;
  let mut last_out = last_out;

  let outputs = inputs.iter().map(|&input| {
    let i0 = (b[0] / a[0]) * input;
    let i1 = (b[1] / a[0]) * last_in[0];
    let i2 = (b[2] / a[0]) * last_in[1];
    let o0 = (a[1] / a[0]) * last_out[0];
    let o1 = (a[2] / a[0]) * last_out[1];

    let output = i0 + i1 + i2 - o0 - o1;
    last_in = [input, last_in[0]];
    last_out = [output, last_out[0]];

    output
  });

  (outputs.collect(), last_in, last_out)
}

fn up_sampling(input: Vec<i16>, in_hz: u32, out_hz: u32) -> Vec<f32> {    
  let edge = EDGE_SAMPLES - 1;
  let rate = in_hz as f64 / out_hz as f64;
  let out_sample_num = (INPUT_SAMPLES as f64) * ((out_hz as f64) / (in_hz as f64));

  (0..out_sample_num as usize)
    .map(|m| m as f64 * rate)    
    .map(|x| {      
      let num = x as usize;
      let dec = x - num as f64;     

      (num, dec)
    })
    .map(|(num, dec)| (num as usize, dec))
    .map(|(idx, dec)| (idx - edge..idx + edge, dec))
    .map(|(range, dec)| {
      let start = -(edge as i32);      

      range.zip(start..)
        .map(|(idx, k)| input[idx] as f64 * sinc(k as f64 - dec))
        .sum()
    })
    .map(|value: f64| value / i16::MAX as f64)    
    .map(|value| value as f32)
    .collect()    
}

fn sinc(x: f64) -> f64 {    
  f64::sin(std::f64::consts::PI * x) / (std::f64::consts::PI * x)
}

fn clamp(x: i32) -> i16 {
  if x > 0x7FFF {
    0x7FFF
  } else if x < -0x8000 {
    -0x8000
  } else {
    x as i16
  }
}
