pub mod dsp;
pub mod emulator;

use emulator::core::Spc700;
use std::env;
use std::io::{Error, ErrorKind};
use std::path::Path;
use std::result::Result;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use hound;

fn main() {
    let args: Vec<String> = env::args().collect();
    let result = match args.get(1) {
        None => Result::Err(Error::new(ErrorKind::Other, "filename must be specified")),
        Some(name) => Spc700::new_with_init(Path::new(name)),
    };

    match result {
        Err(err) => println!("{}", err),
        Ok(emu) => play_by_hound(emu),
    }
}

fn play_by_cpal(emu: Spc700) -> ! {
    let (device, config) = build_config();
    let stream = match config.sample_format() {
        cpal::SampleFormat::F32 => build_stream::<f32>(emu, &device, &config.into()),
        cpal::SampleFormat::I16 => build_stream::<i16>(emu, &device, &config.into()),
        cpal::SampleFormat::U16 => build_stream::<u16>(emu, &device, &config.into()),
    };

    if let Err(err) = stream.play() {
        println!("{}", err);
        std::process::exit(1);
    }

    loop {}
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
    emu: Spc700,
    device: &cpal::Device,
    config: &cpal::StreamConfig,
) -> cpal::Stream {
    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;
    let mut emu = emu;

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let stream = device
        .build_output_stream(
            config,
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                play_spc(data, channels, &mut emu, sample_rate)
            },
            err_fn,
        )
        .expect("unexpected error when building stream");

    stream
}

fn play_spc<T: cpal::Sample>(
    output: &mut [T],
    channels: usize,
    emu: &mut Spc700,
    _sample_rate: f32,
) -> () {
    for frame in output.chunks_mut(channels) {
        let (left, right) = emu.next_sample();
        let left_value: T = cpal::Sample::from::<i16>(&(left as i16));
        let right_value: T = cpal::Sample::from::<i16>(&(right as i16));
        if channels == 1 {
            let composed = ((left as u32 + (right as u32)) >> 1) as i16;
            let value: T = cpal::Sample::from::<i16>(&composed);
            for sample in frame.iter_mut() {
                *sample = value;
            }
        } else {
            for (sample, idx) in frame.iter_mut().zip(0..) {
                if (channels / 2) > idx {
                    *sample = left_value
                } else {
                    *sample = right_value
                }
            }
        }
    }
}

fn play_by_hound(emu: Spc700) -> ! {
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: 32000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = hound::WavWriter::create("test.wav", spec).expect("fatal error occurred when wav writer is instantiated");
    let seconds = 5;
    let sample_rate = 32000;
    let mut emu = emu;
    for _ in 0 .. sample_rate * seconds {
        let (left, right) = emu.next_sample();
        writer.write_sample(left as i16).unwrap();
        writer.write_sample(right as i16).unwrap();
    }

    writer.finalize().unwrap();
    std::process::exit(0);
}

/*
fn print_log(core: &mut Spc700) {
    core.ram.read_log.sort_by_key(|k|  k.0);
    core.ram.write_log.sort_by_key(|k| k.0);

    print!("read[{}]: ", core.ram.read_log.len());
    for (addr, data) in core.ram.read_log.iter() {
        print!("({:#06x}, {:#04x}), ", addr, data);
    }
    println!("");

    print!("write[{}]: ", core.ram.write_log.len());
    for (addr, data) in core.ram.write_log.iter() {
        print!("({:#06x}, {:#04x}), ", addr, data);
    }
    println!("");

    core.ram.read_log = Vec::new();
    core.ram.write_log = Vec::new();
}
*/
