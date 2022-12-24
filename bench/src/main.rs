use spc700_core::SPC700;
use std::path::Path;
use std::time::SystemTime;

const COUNT_UPPER: usize = 1000;
const COUNT_PITCH: usize = 10000;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let filename = &args[1];
    
    let mut spc = SPC700::new();
    spc.load(&Path::new(filename)).unwrap();

    let mut times = Vec::new();
    let mut recv: (i16, i16) = (0, 0);
    for _ in 0..COUNT_UPPER {
        let before = SystemTime::now();
        for _ in 0..COUNT_PITCH {
            unsafe { std::ptr::write_volatile(&mut recv, spc.next_sample()); }
        }
        let duration = before.elapsed().unwrap();
        times.push(duration);

    } 
    

    let average = times.iter().map(|d| d.as_micros()).sum::<u128>() / COUNT_UPPER as u128;
    println!("average: {} us", average);
}
