use chrono::{Local, Timelike};
use std::{fs::File, io::Write};

fn main() {
    let start = Local::now();
    let hour = start.hour() as u8;
    let min = start.minute() as u8;
    let sec = start.second() as u8;
    let mut output = File::create("assets/time.bin").unwrap();
    println!("编译时间 {start}");
    output.write_all(&[hour, min, sec]).unwrap();
}
