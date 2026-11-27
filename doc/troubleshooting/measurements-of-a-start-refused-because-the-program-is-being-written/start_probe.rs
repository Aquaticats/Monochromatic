//! A start prober:
//! it starts one program as fast as it may,
//! for a fixed span,
//! and reports how many starts the operating system refused with `Text file busy`.
//!
//! It makes the same call the shadow makes (a `Command` spawn,
//! which is `posix_spawn`),
//! so a refusal it counts is a refusal the shadow would have reported.
//!
//! Usage:
//! `start_probe <program> <milliseconds> [pause-microseconds]`

/// Imports the process types.
use std::process::{
    Command,
    Stdio,
};
/// Imports the clock.
use std::time::{
    Duration,
    Instant,
};

/// Linux's `ETXTBSY`,
/// raised by `execve` for a file any process holds open for writing.
const TEXT_FILE_BUSY: i32 = 26;

fn main() {
    let arguments: Vec<String> = std::env::args().collect();
    let program: String = arguments
        .get(1)
        .expect("usage: start_probe <program> <milliseconds> [pause-microseconds]")
        .clone();
    let millis: u64 = arguments
        .get(2)
        .expect("the span in milliseconds")
        .parse()
        .expect("a number of milliseconds");
    let pause: u64 = arguments
        .get(3)
        .map_or(0, |text| return text.parse().expect("a number of microseconds"));
    let started_at: Instant = Instant::now();
    let deadline: Instant = started_at + Duration::from_millis(millis);
    let mut refused: u32 = 0;
    let mut ran: u32 = 0;
    let mut absent: u32 = 0;
    let mut other: Vec<i32> = Vec::new();
    let mut first_refusal_us: Option<u128> = None;
    let mut last_refusal_us: Option<u128> = None;
    loop {
        let outcome = Command::new(&program)
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        match outcome {
            Ok(_) => ran += 1,
            Err(error) if error.raw_os_error() == Some(TEXT_FILE_BUSY) => {
                refused += 1;
                let at: u128 = started_at.elapsed().as_micros();
                if first_refusal_us.is_none() {
                    first_refusal_us = Some(at);
                }
                last_refusal_us = Some(at);
            }
            Err(error) if error.raw_os_error() == Some(2) => absent += 1,
            Err(error) => other.push(error.raw_os_error().unwrap_or(-1)),
        }
        if Instant::now() >= deadline {
            break;
        }
        if pause > 0 {
            std::thread::sleep(Duration::from_micros(pause));
        }
    }
    println!(
        "START_PROBE path={program} span_ms={millis} pause_us={pause} ran={ran} refused={refused} absent={absent} other={other:?} first_refusal_us={} last_refusal_us={}",
        first_refusal_us.map_or("-".to_string(), |v| return v.to_string()),
        last_refusal_us.map_or("-".to_string(), |v| return v.to_string())
    );
}
