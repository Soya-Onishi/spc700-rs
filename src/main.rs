mod spc;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut core = spc::core::Spc700::new();


    if args.len() == 4 {
        let start_pos = u16::from_str_radix(&args[2], 16).unwrap();
        let set_pos = u16::from_str_radix(&args[3].clone(), 16).unwrap();
        core.ram.load(args[1].clone(), start_pos, set_pos);
    }
}
