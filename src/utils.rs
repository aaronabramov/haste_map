use num_cpus;
use std;

pub fn duration_to_ms(d: std::time::Duration) -> f64 {
    (d.as_secs() as f64) * 1000.0 + (f64::from(d.subsec_nanos()) / 1000_000.0)
}

pub fn log_time(now: std::time::Instant, msg: &str) {
    println!("{}ms : {}", duration_to_ms(now.elapsed()), msg);
}
// bypass to disable logging without compile warnings
#[allow(unused_variables)]
pub fn skip_log_time(now: std::time::Instant, msg: &str) {}

pub fn get_chunk_size<T>(v: &Vec<T>) -> usize {
    let threads = num_cpus::get();
    // let threads = 1;
    let min: usize = 1;
    let chunk_size: usize = v.len() / threads;
    std::cmp::max(chunk_size, min)
}
