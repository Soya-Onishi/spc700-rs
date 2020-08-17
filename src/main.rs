pub mod dsp;
pub mod emulator;
pub mod amp;

use emulator::core::Spc700;
use std::env;
use std::io::{Error, ErrorKind};
use std::path::Path;
use std::result::Result;

fn main() {
    let args: Vec<String> = env::args().collect();
    let result = match args.get(1) {
        None => Result::Err(Error::new(ErrorKind::Other, "filename must be specified")),
        Some(name) => Spc700::new_with_init(Path::new(name)),
    };

    match result {
        Err(err) => println!("{}", err),
        Ok(emu) => amp::Amplifier::play(emu),
    }
}