pub mod dsp;
pub mod emulator;
pub mod amp;

use std::result::Result;
use emulator::core::Spc700;
use std::io::Error;
use std::path::Path;

use clap::Parser;
use std::io::Write;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, default_value_t = 100)]
    duration: u64,

    file: String,
}

fn main() -> Result<(), Error> {
    let args = Args::parse(); 
    env_logger::builder()
        .format(|buf, record| {
            writeln!(buf, "{}: {}", record.level(), record.args())
        })
        .init();
    log::info!("env logger initialized");

    let emulator = Spc700::new_with_init(Path::new(&args.file))?;
    amp::Amplifier::play(emulator, args.duration);

    Ok(())
}