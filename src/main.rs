mod spc;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut core = spc::core::Spc700::new(0x0430);

    if args.len() == 4 {
        let start_pos = u16::from_str_radix(&args[2], 16).unwrap();
        let set_pos = u16::from_str_radix(&args[3].clone(), 16).unwrap();
        core.ram.load(args[1].clone(), start_pos, set_pos);
    }

    while core.ram.read(0x8000) == 0 {
        core.execute();
    }

    while core.ram.read(0x8000) == 0x80 {
        core.execute();
    }

    println!("0x8000: {:#06x}", core.ram.read(0x8000));
}
