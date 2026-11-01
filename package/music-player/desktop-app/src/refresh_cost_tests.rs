// What:     Refresh-cost measurements over a real, materialized library, pulled in by `controller_measures.rs`.
//           The tests are `#[ignore]` and read `MP_REFRESH_LIBRARY` (the library folder) and `MP_REFRESH_CACHE` (a
//           throwaway cache file that is reused between runs, so the load's own sweep decodes only once).
// Why:      D113 makes every watcher event start a refresh; the report quotes the cost of one replaced, added and
//           deleted file at 100,000 and 1,000,000 real files, with the spread of five runs.

use super::*;
use crate::command::{Command, Update};
use crate::peakcache::CacheHandle;
use crate::sweep_report::SweepReport;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// What: `Bench` is a controller over the real library with the sweep reports it received and the updates it emitted.
/// Why: every measurement needs the controller, the reports to pump and the updates to read.
struct Bench {
    controller: Controller,
    reports: Arc<Mutex<Vec<SweepReport>>>,
    updates: Arc<Mutex<Vec<Update>>>,
    pumped: usize,
}

/// Methods of `Bench`.
impl Bench {
    /// What: `new` builds the controller with the persistent cache file.
    /// Why: the load's sweep must not decode a million files in every run.
    fn new(cache_file: &str) -> Bench {
        let reports: Arc<Mutex<Vec<SweepReport>>> = Arc::new(Mutex::new(Vec::new()));
        let updates: Arc<Mutex<Vec<Update>>> = Arc::new(Mutex::new(Vec::new()));
        let update_sink = Arc::clone(&updates);
        let mut controller = Controller::new(
            Box::new(move |update| {
                update_sink.lock().expect("the sink is not poisoned").push(update);
            }),
            None,
            CacheHandle::open_at(PathBuf::from(cache_file)),
        );
        let report_sink = Arc::clone(&reports);
        controller.sweep_sink = Arc::new(move |report| {
            report_sink.lock().expect("the sink is not poisoned").push(report);
        });
        return Bench { controller, reports, updates, pumped: 0 };
    }

    /// What: `pump` hands the controller the sweep reports that arrived since the last call.
    /// Why: the engine's loop does the same.
    fn pump(&mut self) {
        let waiting: Vec<SweepReport> = {
            let all = self.reports.lock().expect("the sink is not poisoned");
            all[self.pumped..].to_vec()
        };
        for report in waiting {
            self.pumped += 1;
            self.controller.handle_command(Command::SweepReport(report));
        }
    }

    /// What: `wait_for_peak` pumps until a measure with a peak arrived for `path` since `from`, and says how long it took.
    /// Why: the person sees the new peak when that update arrives.
    fn wait_for_peak(&mut self, path: &Path, started: Instant) -> Option<Duration> {
        let deadline = Instant::now() + Duration::from_secs(20);
        while Instant::now() < deadline {
            self.pump();
            let seen = self.updates.lock().expect("the sink is not poisoned").iter().any(|update| {
                return matches!(update, Update::Measures(list) if list.iter().any(|(found, measure)| return found == path && measure.peak_dbtp.is_some()));
            });
            if seen {
                return Some(started.elapsed());
            }
            std::thread::sleep(Duration::from_micros(500));
        }
        return None;
    }

    /// What: `clear_updates` forgets the updates so far.
    /// Why: each run looks only at its own.
    fn clear_updates(&self) {
        self.updates.lock().expect("the sink is not poisoned").clear();
    }
}

/// What: `summary` prints the median with the minimum and maximum of the samples in milliseconds.
/// Why: the report quotes the spread of five runs.
fn summary(label: &str, samples: &mut [Duration]) {
    samples.sort();
    let ms = |value: Duration| return value.as_secs_f64() * 1000.0;
    println!(
        "REFRESH {label}: median {:.2} ms, min {:.2} ms, max {:.2} ms over {} runs",
        ms(samples[samples.len() / 2]),
        ms(samples[0]),
        ms(samples[samples.len() - 1]),
        samples.len()
    );
}

/// What: `cpu_seconds` is the CPU time (user and system) this process has used so far, in seconds.
/// Why: a refresh sweep runs at idle priority on other threads, so its cost on a busy host is its CPU time, not its latency.
#[cfg(not(windows))]
fn cpu_seconds() -> f64 {
    let mut usage: libc::rusage = unsafe { std::mem::zeroed() };
    let status = unsafe { libc::getrusage(libc::RUSAGE_SELF, &mut usage) };
    assert_eq!(status, 0, "getrusage works");
    let seconds = |value: libc::timeval| return value.tv_sec as f64 + value.tv_usec as f64 / 1_000_000.0;
    return seconds(usage.ru_utime) + seconds(usage.ru_stime);
}

/// What: `cpu_seconds` is the CPU time (user and kernel) this process has used so far, in seconds, from `GetProcessTimes`.
/// Why: the same measurement as on Unix. The `libc` crate is not a dependency on Windows, so the test module did not compile
///      there until this version existed.
#[cfg(windows)]
fn cpu_seconds() -> f64 {
    use windows::Win32::Foundation::FILETIME;
    use windows::Win32::System::Threading::{GetCurrentProcess, GetProcessTimes};
    let (mut created, mut exited, mut kernel, mut user) = (FILETIME::default(), FILETIME::default(), FILETIME::default(), FILETIME::default());
    unsafe { GetProcessTimes(GetCurrentProcess(), &mut created, &mut exited, &mut kernel, &mut user) }.expect("GetProcessTimes works");
    // A FILETIME counts 100-nanosecond ticks split over two 32-bit halves.
    let seconds = |value: FILETIME| return ((u64::from(value.dwHighDateTime) << 32) | u64::from(value.dwLowDateTime)) as f64 / 10_000_000.0;
    return seconds(kernel) + seconds(user);
}

/// What: `dispatch` tells the controller that `paths` changed, the way the watcher does; with `MP_REFRESH_EVERYTHING` set it
/// sends the lost-events command instead, which walks and refreshes the whole library.
/// Why: the same measurements then show the cost of one event and the cost of the fallback.
fn dispatch(controller: &mut Controller, paths: &[PathBuf], structural: bool) {
    if std::env::var("MP_REFRESH_EVERYTHING").is_ok() {
        controller.handle_command(Command::Rescan);
        return;
    }
    let mut dirty = crate::change_set::DirtyPaths::default();
    for path in paths {
        dirty.insert(path.clone(), structural);
    }
    controller.handle_command(Command::SourceChanged(crate::change_set::Change::Paths(dirty)));
}

#[test]
#[ignore = "measures a real materialized library; set MP_REFRESH_LIBRARY and MP_REFRESH_CACHE"]
fn refresh_cost_of_one_replaced_added_and_deleted_file() {
    let (Ok(library), Ok(cache)) = (std::env::var("MP_REFRESH_LIBRARY"), std::env::var("MP_REFRESH_CACHE")) else {
        return;
    };
    let mut bench = Bench::new(&cache);
    let root = PathBuf::from(&library);
    let opened = Instant::now();
    bench.controller.handle_command(Command::OpenRoot { root: root.clone(), select: None, play: false });
    println!("REFRESH open handler: {:.0} ms", opened.elapsed().as_secs_f64() * 1000.0);
    let total = bench.controller.queue.len();
    let waiting = Instant::now();
    loop {
        bench.pump();
        let done = bench.updates.lock().expect("the sink is not poisoned").iter().any(|update| return matches!(update, Update::Analysis { done, total: all } if done == all && *all > 0));
        if done || waiting.elapsed() > Duration::from_secs(1_800) {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    println!("REFRESH tracks {total}; the load's sweep finished after {:.1} s", waiting.elapsed().as_secs_f64());
    let fixtures = ["tone.wav", "tone.mp3", "tone.opus", "tone.ogg", "tone.aac.m4a"];
    let mut handler: Vec<Duration> = Vec::new();
    let mut visible: Vec<Duration> = Vec::new();
    for (run, fixture) in fixtures.iter().enumerate() {
        let path = bench.controller.queue.tracks()[total * (run + 1) / (fixtures.len() + 1)].clone();
        bench.clear_updates();
        std::fs::copy(PathBuf::from("fixture").join(fixture), &path).expect("the fixture replaces the file");
        let started = Instant::now();
        dispatch(&mut bench.controller, std::slice::from_ref(&path), false);
        handler.push(started.elapsed());
        let seen = bench.wait_for_peak(&path, started);
        visible.push(seen.unwrap_or(Duration::from_secs(20)));
    }
    let mut cpu: Vec<Duration> = Vec::new();
    for (run, fixture) in fixtures.iter().enumerate() {
        let path = bench.controller.queue.tracks()[total * (run + 1) / (fixtures.len() + 1) + 3].clone();
        std::fs::copy(PathBuf::from("fixture").join(fixture), &path).expect("the fixture replaces the file");
        std::thread::sleep(Duration::from_millis(300));
        let before = cpu_seconds();
        dispatch(&mut bench.controller, std::slice::from_ref(&path), false);
        let settle = Instant::now();
        while settle.elapsed() < Duration::from_secs(4) {
            bench.pump();
            std::thread::sleep(Duration::from_millis(5));
        }
        cpu.push(Duration::from_secs_f64(cpu_seconds() - before));
    }
    summary("replace: CPU seconds spent in the 4 s after one event (shown as ms)", &mut cpu);
    summary("replace: handler", &mut handler);
    summary("replace: new peak shown", &mut visible);
    let mut add_handler: Vec<Duration> = Vec::new();
    let mut add_visible: Vec<Duration> = Vec::new();
    let mut remove_handler: Vec<Duration> = Vec::new();
    for run in 0..fixtures.len() {
        let sibling = bench.controller.queue.tracks()[total * (run + 1) / (fixtures.len() + 1) + 7].clone();
        let added = sibling.with_file_name(format!("zz added {run}.wav"));
        bench.clear_updates();
        std::fs::copy(PathBuf::from("fixture").join("tone.wav"), &added).expect("the new file is written");
        let started = Instant::now();
        dispatch(&mut bench.controller, std::slice::from_ref(&added), true);
        add_handler.push(started.elapsed());
        add_visible.push(bench.wait_for_peak(&added, started).unwrap_or(Duration::from_secs(20)));
        assert!(bench.controller.queue.tracks().contains(&added), "the new file is listed");
        std::fs::remove_file(&added).expect("the new file is removed");
        let removal_started = Instant::now();
        dispatch(&mut bench.controller, std::slice::from_ref(&added), true);
        remove_handler.push(removal_started.elapsed());
        assert!(!bench.controller.queue.tracks().contains(&added), "the deleted file is gone from the list");
    }
    summary("add: handler", &mut add_handler);
    summary("add: new peak shown", &mut add_visible);
    summary("delete: handler", &mut remove_handler);
}
