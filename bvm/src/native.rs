use std::time::{SystemTime, UNIX_EPOCH};

pub fn clock() -> f64 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards.");
    let in_millis = since_the_epoch.as_millis();
    let in_secs: f64 = (in_millis as f64) / 1e+3;
    in_secs
}
