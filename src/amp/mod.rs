extern crate cpal;
extern crate hound;

use crate::emulator::core::Spc700;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::collections::VecDeque;
use std::f32;
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

        assign_output(data, read_data, channels);        
      },
      err_fn,
    )
    .expect("unexpected error when building stream");
  stream
}

fn assign_output<T: cpal::Sample>(data: &mut [T], output: Vec<(i16, i16)>, channels: usize) -> () {
  if channels == 1 {
    let outputs = output.iter().map(|(left, right)| (left + right) / 2);
    for (sample, output) in data.iter_mut().zip(outputs) {
      *sample = cpal::Sample::from(&output);
    }    
  } else {
    let each_len = channels / 2;    
    let left: Vec<T> = output.iter().map(|(left, _)| cpal::Sample::from(left)).collect();
    let right: Vec<T> = output.iter().map(|(_, right)| cpal::Sample::from(right)).collect();    

    for (frame, idx) in data.chunks_mut(channels).zip(0..) {      
      for (sample, ch) in frame.iter_mut().zip(0..) {
        if ch < each_len { *sample = left[idx] }
        else             { *sample = right[idx] }        
      }
    }
  }
}