//! What:
//! A start prober:
//! it starts one program as fast as it may,
//! for a fixed span,
//! and reports how many starts the operating system refused with `Text file busy`.
//! A second mode writes that program from one thread while another thread forks,
//! and reports how many starts a descriptor copied into those children refused.
//! Why:
//! It makes the same call the shadow makes (a `Command` spawn,
//! which is `posix_spawn`),
//! so a refusal it counts is a refusal the shadow would have reported.
//! The first mode starts only,
//! so a refusal it counts comes from a holder that outlives its own work;
//! it cannot see a holder that lives from a `fork` to that child's own start,
//! because nothing in it forks.
//! The second mode is that holder:
//! a `fork` copies every descriptor of the process that forks,
//! the child keeps a descriptor opened for writing until its own start closes it,
//! and the device that holds it is measured in
//! `doc/troubleshooting/shadow-tests-that-write-a-program-and-start-it-are-refused-with-etxtbsy.md`.
//!
//! Usage:
//! `start_probe <program> <milliseconds> [pause-microseconds]`
//! `start_probe forked-holder <program> <attempts> [hold-microseconds] [forker-pause-microseconds]`
//! `start_probe writer-only <program> <attempts> [hold-microseconds]`
//!
//! The two writer modes differ in one thing:
//! `forked-holder` runs a thread that starts `/bin/true` in a loop while the program is held open for writing,
//! `writer-only` runs no such thread,
//! so a refusal in the first mode names the copied descriptor
//! and the second mode is its control.

/// Imports the file operations the writer modes need.
use std::fs::OpenOptions;
/// Imports the trait the writer modes write through.
use std::io::Write;
/// Imports the process types.
use std::process::{
    Command,
    Stdio,
};
/// Imports the atomics that stop the forking thread.
use std::sync::atomic::{
    AtomicBool,
    AtomicU64,
    Ordering,
};
/// Imports the clock.
use std::time::{
    Duration,
    Instant,
};

/// Linux's `ETXTBSY`,
/// raised by `execve` for a file any process holds open for writing.
const TEXT_FILE_BUSY: i32 = 26;

/// What one start did,
/// so both modes count the same four outcomes.
#[derive(Debug, Default)]
struct Starts {
    /// Starts the system ran.
    ran: u32,
    /// Starts refused because a file was open for writing.
    refused: u32,
    /// Starts that found no program at the path.
    absent: u32,
    /// Every other errno,
    /// kept as read.
    other: Vec<i32>,
    /// When the first refusal happened,
    /// in microseconds from the start of the mode.
    first_refusal_us: Option<u128>,
    /// When the last refusal happened.
    last_refusal_us: Option<u128>,
}

impl Starts {
    /// Counts one start's outcome.
    fn count(
        &mut self,
        outcome: std::io::Result<std::process::ExitStatus>,
        at: u128,
    ) {
        match outcome {
            Ok(_) => self.ran += 1,
            Err(error) if error.raw_os_error() == Some(TEXT_FILE_BUSY) => {
                self.refused += 1;
                if self.first_refusal_us.is_none() {
                    self.first_refusal_us = Some(at);
                }
                self.last_refusal_us = Some(at);
            }
            Err(error) if error.raw_os_error() == Some(2) => self.absent += 1,
            Err(error) => self.other.push(error.raw_os_error().unwrap_or(-1)),
        }
    }

    /// Renders the counters the loop's own line prints.
    fn render(&self) -> String {
        return format!(
            "ran={} refused={} absent={} other={:?} first_refusal_us={} last_refusal_us={}",
            self.ran,
            self.refused,
            self.absent,
            self.other,
            self.first_refusal_us
                .map_or("-".to_string(), |value| return value.to_string()),
            self.last_refusal_us
                .map_or("-".to_string(), |value| return value.to_string())
        );
    }
}

/// Starts `program` once and counts what happened.
fn start_once(
    program: &str,
    starts: &mut Starts,
    started_at: Instant,
) {
    let outcome = Command::new(program)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    starts.count(outcome, started_at.elapsed().as_micros());
}

/// The first mode:
/// starts one program as fast as it may for a fixed span.
fn start_forever(
    program: &str,
    millis: u64,
    pause: u64,
) {
    let started_at: Instant = Instant::now();
    let deadline: Instant = started_at + Duration::from_millis(millis);
    let mut starts: Starts = Starts::default();
    loop {
        start_once(program, &mut starts, started_at);
        if Instant::now() >= deadline {
            break;
        }
        if pause > 0 {
            std::thread::sleep(Duration::from_micros(pause));
        }
    }
    println!(
        "START_PROBE path={program} span_ms={millis} pause_us={pause} {}",
        starts.render()
    );
}

/// The hook the writer modes count: one write of the program,
/// held open,
/// then closed and started.
fn write_and_start(
    program: &str,
    template: &[u8],
    hold_us: u64,
    starts: &mut Starts,
    started_at: Instant,
) -> Result<(), String> {
    let mut file = match OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(program)
    {
        Ok(opened) => opened,
        Err(error) => return Err(format!("cannot open {program} for writing: {error}")),
    };
    if let Err(error) = file.write_all(template) {
        return Err(format!("cannot write {program}: {error}"));
    }
    std::thread::sleep(Duration::from_micros(hold_us));
    // The descriptor is closed here, before the start:
    // what a refusal in `forked-holder` names is a copy another thread's `fork` made.
    drop(file);
    start_once(program, starts, started_at);
    return Ok(());
}

/// What:
/// The two writer modes:
/// write the program,
/// hold it open long enough for a `fork` to copy the descriptor,
/// close it,
/// and start it.
/// Why:
/// The hold is the only part `writer-only` and `forked-holder` share;
/// the forking thread is the whole difference,
/// so the two runs' counts attribute the refusal to the copied descriptor.
fn write_then_start(
    program: &str,
    attempts: u32,
    hold_us: u64,
    forker_pause_us: Option<u64>,
) -> Result<(), String> {
    let template: Vec<u8> = match std::fs::read(program) {
        Ok(bytes) => bytes,
        Err(error) => return Err(format!("cannot read {program}: {error}")),
    };
    let stop = AtomicBool::new(false);
    let forks = AtomicU64::new(0);
    let started_at: Instant = Instant::now();
    let mut starts: Starts = Starts::default();
    let mode: &str = if forker_pause_us.is_some() {
        "forked-holder"
    } else {
        "writer-only"
    };
    std::thread::scope(|scope| -> Result<(), String> {
        if let Some(pause) = forker_pause_us {
            let stop = &stop;
            let forks = &forks;
            scope.spawn(move || {
                while !stop.load(Ordering::Relaxed) {
                    let _ = Command::new("/bin/true")
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .status();
                    forks.fetch_add(1, Ordering::Relaxed);
                    std::thread::sleep(Duration::from_micros(pause));
                }
            });
        }
        for _ in 0..attempts {
            write_and_start(program, &template, hold_us, &mut starts, started_at)?;
        }
        stop.store(true, Ordering::Relaxed);
        return Ok(());
    })?;
    println!(
        "START_PROBE mode={mode} path={program} attempts={attempts} hold_us={hold_us} forks={} elapsed_us={} {}",
        forks.load(Ordering::Relaxed),
        started_at.elapsed().as_micros(),
        starts.render()
    );
    return Ok(());
}

fn main() -> Result<(), String> {
    let arguments: Vec<String> = std::env::args().collect();
    if arguments.get(1).map(String::as_str) == Some("forked-holder")
        || arguments.get(1).map(String::as_str) == Some("writer-only")
    {
        let mode: &str = &arguments[1];
        let program: &str = arguments
            .get(2)
            .ok_or_else(|| return format!("usage: start_probe {mode} <program> <attempts> [hold-microseconds] [forker-pause-microseconds]"))?;
        let attempts: u32 = arguments
            .get(3)
            .ok_or_else(|| return "the number of attempts".to_string())?
            .parse()
            .map_err(|_| return "a number of attempts".to_string())?;
        let hold_us: u64 = arguments
            .get(4)
            .map_or(Ok(0), |text| return text.parse())
            .map_err(|_| return "a number of microseconds to hold".to_string())?;
        let forker_pause_us: Option<u64> = if mode == "forked-holder" {
            Some(
                arguments
                    .get(5)
                    .map_or(Ok(200), |text| return text.parse())
                    .map_err(|_| {
                        return "a number of microseconds for the forking thread to pause"
                            .to_string();
                    })?,
            )
        } else {
            None
        };
        return write_then_start(program, attempts, hold_us, forker_pause_us);
    }
    let program: String = arguments
        .get(1)
        .ok_or_else(|| {
            return "usage: start_probe <program> <milliseconds> [pause-microseconds]".to_string();
        })?
        .clone();
    let millis: u64 = arguments
        .get(2)
        .ok_or_else(|| return "the span in milliseconds".to_string())?
        .parse()
        .map_err(|_| return "a number of milliseconds".to_string())?;
    let pause: u64 = arguments
        .get(3)
        .map_or(Ok(0), |text| return text.parse())
        .map_err(|_| return "a number of microseconds to pause".to_string())?;
    start_forever(&program, millis, pause);
    return Ok(());
}
