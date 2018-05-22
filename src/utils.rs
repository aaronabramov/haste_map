use std;

pub fn duration_to_ms(d: std::time::Duration) -> f64 {
    (d.as_secs() as f64) * 1000.0 + (f64::from(d.subsec_nanos()) / 1000_000.0)
}
