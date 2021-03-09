use once_cell::sync::Lazy;
use std::{collections::HashMap, time::Instant};

struct Measurement {
    min: u128,
    max: u128,
    avg: u128,
    count: u128,
}

static APP_START: Lazy<Instant> = Lazy::new(|| Instant::now());
static mut MEASUREMENTS: Lazy<HashMap<String, Measurement>> = Lazy::new(|| HashMap::default());

unsafe fn log(message: String, start: Instant) {
    let elapsed = start.elapsed().as_micros();
    let m = MEASUREMENTS.get_mut(&message);

    match m {
        Some(mut existing) => {
            if elapsed < existing.min {
                existing.min = elapsed
            };
            if elapsed > existing.max {
                existing.max = elapsed
            };

            existing.avg = ((existing.avg * existing.count) + elapsed) / (existing.count + 1);
            existing.count += 1;
        }
        None => {
            MEASUREMENTS.insert(
                message,
                Measurement {
                    min: elapsed,
                    max: elapsed,
                    avg: elapsed,
                    count: 1,
                },
            );
        }
    }
}

pub fn measure_time<T, F: FnOnce() -> T>(message: &str, f: F) -> T {
    let start = Instant::now();
    let r = f();

    unsafe {
        if APP_START.elapsed().as_millis() > 2000 {
            log(message.to_string(), start);
        }
    }
    r
}

pub fn print() {
    unsafe {
        for (message, m) in MEASUREMENTS.iter() {
            println!(
                "{}: avg: {} - min: {} - max: {} - count: {}",
                message,
                m.avg as f64 / 1000.0,
                m.min as f64 / 1000.0,
                m.max as f64 / 1000.0,
                m.count
            );
        }
    }
}
