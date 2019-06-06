mod spc;

use std::env;
use std::collections::HashMap;
use spc::core::Spc700;
fn main() {
    let args: Vec<String> = env::args().collect();

    let mut core = spc::core::Spc700::new(0x0430);

    if args.len() == 4 {
        let start_pos = u16::from_str_radix(&args[2], 16).unwrap();
        let set_pos = u16::from_str_radix(&args[3].clone(), 16).unwrap();
        core.ram.load(args[1].clone(), start_pos, set_pos);
    }

    while core.ram.ram[0x8000] == 0 {
        core.execute();

        print_log(&mut core)
    }

    while core.ram.ram[0x8000] == 0x80 {
        core.execute();

        print_log(&mut core)
    }

    println!("0x8000: {:#06x}", core.ram.ram[0x8000]);
}

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
