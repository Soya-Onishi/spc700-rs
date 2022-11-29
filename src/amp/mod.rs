extern crate cpal;
extern crate hound;

use crate::emulator::core::Spc700;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::f32;
use std::sync::mpsc;
use std::thread;

const INPUT_SAMPLING_RATE: usize = 32000;
const BUFFER_SIZE: usize = INPUT_SAMPLING_RATE * 8;

pub struct Amplifier;
impl Amplifier {
  pub fn play(core: Spc700) -> ! {
    let (device, config) = build_config();
    let format = config.sample_format();
    let config = cpal::StreamConfig {
      channels: config.channels(),
      buffer_size: cpal::BufferSize::Default,
      sample_rate: cpal::SampleRate(32000),
    };
    let (tx, rx) = mpsc::sync_channel::<(i16, i16)>(BUFFER_SIZE);
    
    let stream = match format {
      cpal::SampleFormat::F32 => {
        build_stream::<f32>(&device, &config, rx)
      }
      cpal::SampleFormat::I16 => {
        build_stream::<i16>(&device, &config, rx)
      }
      cpal::SampleFormat::U16 => {
        build_stream::<u16>(&device, &config, rx)
      }
    };

    // thread for creating samples
    let mut core = core;
    {
      thread::spawn(move || {
        loop {
          let output = core.next_sample();
          tx.send(output).unwrap();
        }
      });
    }

    stream.play().unwrap();

    loop {}
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
  rx: mpsc::Receiver<(i16, i16)>,
) -> cpal::Stream {  
  let channels = config.channels as usize;
  let error_callback = |err| eprintln!("an error occurred on stream: {}", err);  

  let data_callback = move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
    for (dsts, output) in data.chunks_mut(channels).zip(rx.iter()) {
      let (left, right) = output;
      let sample = ((left as i32 + right as i32) / 2) as i16;
      let sample = cpal::Sample::from(&sample);
      dsts.fill(sample);
    }
  };

  device.build_output_stream(config, data_callback, error_callback).unwrap()
}