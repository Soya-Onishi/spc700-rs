extern crate cpal;
extern crate hound;

use crate::emulator::core::Spc700;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::f32;
use std::sync::mpsc;
use std::thread;

const SAMPLE_RATE: u32 = 32000;
const INPUT_SAMPLING_RATE: usize = 32000;
const BUFFER_SIZE: usize = INPUT_SAMPLING_RATE * 8;

pub struct Amplifier;
impl Amplifier {
  pub fn play(core: Spc700, duration: u64) {
    let device = cpal::default_host().default_output_device().expect("no output device available");

    // 32000Hzの再生に対応しているconfigを探す。
    // 32000HzはSPC700が再生時に使用するサンプリングレート
    let config = device.supported_output_configs()
      .unwrap()
      .find(|config| {
        let cpal::SampleRate(max) = config.max_sample_rate();
        let cpal::SampleRate(min) = config.min_sample_rate();
        min <= SAMPLE_RATE && SAMPLE_RATE <= max
      })
      .expect("there are no current device configs to play on 32000Hz.")
      .with_sample_rate(cpal::SampleRate(SAMPLE_RATE));

    let format = config.sample_format();
    let config = config.config();
    let stream = match format {
      cpal::SampleFormat::F32 => {
        build_stream::<f32>(&device, &config, core)
      }
      cpal::SampleFormat::I16 => {
        build_stream::<i16>(&device, &config, core)
      }
      cpal::SampleFormat::U16 => {
        build_stream::<u16>(&device, &config, core)
      }
    };

    stream.play().unwrap();

    thread::sleep(std::time::Duration::from_millis(duration));
  }
}

fn build_stream<T: cpal::Sample + std::marker::Send + 'static>(
  device: &cpal::Device,
  config: &cpal::StreamConfig,
  mut core: Spc700
) -> cpal::Stream {  
  let channels = config.channels as usize;

  let (tx, rx) = mpsc::sync_channel(BUFFER_SIZE);

  thread::spawn(move || {
    loop {
      let (left, right) = core.next_sample();
      let output = ((left as i32 + right as i32) / 2) as i16;
      match tx.send(T::from(&output)) {
        Ok(_) => {},
        Err(err) => { 
          println!("{:?}", err);
          std::process::exit(0);
        },
      }
    } 
  });

  let error_callback = |err| eprintln!("an error occurred on stream: {}", err);  
  let data_callback = move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
    for dsts in data.chunks_mut(channels) {
      dsts.fill(rx.recv().unwrap());
    }
  };

  device.build_output_stream(config, data_callback, error_callback).unwrap()
}