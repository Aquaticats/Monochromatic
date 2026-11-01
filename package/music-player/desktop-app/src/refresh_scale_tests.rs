// What:     Scale test of one refresh event at 1,000,000 tracks, pulled in by `lib.rs` for test builds.
//           The queue is a million synthetic tracks in the walk's order under a throwaway folder; the folder holds only the
//           files the test changes. The test times the whole controller handler for one `SourceChanged` (planning, queue
//           edit and the start of the sweep), the way the engine thread runs it, and the time until the new measure arrives.
// Why:      1,000 artists and 1,000,000 tracks are normal operation (D112), so the normal suite holds this test. A refresh
//           after one replaced file used to walk and stat the whole library (seconds of the engine thread and tens of
//           CPU seconds); it must cost milliseconds, and the run-to-run spread is printed so a bar is read against it.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::change_set::{Change, DirtyPaths};
use crate::command::{Command, Update};
use crate::controller::Controller;
use crate::peakcache::CacheHandle;
use crate::refresh_scale_support::{ROOT, million_library};
use crate::sweep_report::SweepReport;

/// The runs per measurement: the host is shared, so the spread of five runs shows how far a median can be trusted. A
/// debug build runs three, because its timings are informational and the suite must stay short.
const RUNS: usize = if cfg!(debug_assertions) { 3 } else { 5 };

/// The runs of the add and delete measurement: each rebuilds the list and announces it, which takes seconds without optimization.
const LIST_RUNS: usize = if cfg!(debug_assertions) { 2 } else { 5 };

/// What: the bar for the engine-thread cost of one replaced file.
/// Why: measured in release at 1,000,000 tracks the handler takes about a millisecond; the bar is tens of times that, so
/// host noise does not fail it and a return of the walk (more than a second) does. The debug bar allows unoptimized code.
const REPLACE_BAR: Duration = if cfg!(debug_assertions) { Duration::from_millis(500) } else { Duration::from_millis(50) };

/// What: the bar for the engine-thread cost of one added or deleted file, which still announces the whole list to the window.
/// Why: measured in release at 1,000,000 tracks the handler takes about half a second, nearly all of it building the list that
/// the announcement carries; before the planner it took 1.9 seconds, walking the folder and rebuilding the scope twice. The
/// bar sits between the two. The debug bar only catches a return of something far slower.
const LIST_BAR: Duration = if cfg!(debug_assertions) { Duration::from_secs(20) } else { Duration::from_millis(1_200) };

/// What: `spread` prints the median with the minimum and maximum of the samples in milliseconds and returns the median.
/// Why: the report quotes the spread of the runs.
fn spread(label: &str, samples: &mut [Duration]) -> Duration {
    samples.sort();
    let ms = |value: Duration| return value.as_secs_f64() * 1000.0;
    let median = samples[samples.len() / 2];
    println!("REFRESH {label}: median {:.3} ms, min {:.3} ms, max {:.3} ms over {} runs", ms(median), ms(samples[0]), ms(samples[samples.len() - 1]), samples.len());
    return median;
}

/// What: `Scale` is a controller over a million-track queue, the folder holding the files the test changes, and what the
/// controller's sweeps reported.
/// Why: each measurement needs the controller, the folder and the reports to hand back to it.
struct Scale {
    controller: Controller,
    folder: PathBuf,
    tracks: Vec<PathBuf>,
    reports: Arc<Mutex<Vec<SweepReport>>>,
    updates: Arc<Mutex<Vec<Update>>>,
    pumped: usize,
}

/// Methods of `Scale`.
impl Scale {
    /// What: `new` builds the controller with the library moved under a fresh folder.
    /// Why: the planner asks the real disk about named paths, so the changed files must exist there.
    fn new(tag: &str) -> Scale {
        crate::logging::init();
        let nanos: u128 = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |elapsed| return elapsed.as_nanos());
        let folder = crate::test_scratch::scratch_root().join(format!("mp-refresh-scale-{}-{nanos}-{tag}", std::process::id()));
        std::fs::create_dir_all(&folder).expect("the folder is created");
        let tracks: Vec<PathBuf> = million_library().into_iter().map(|path| return folder.join(path.strip_prefix(ROOT).expect("below the root"))).collect();
        let reports: Arc<Mutex<Vec<SweepReport>>> = Arc::new(Mutex::new(Vec::new()));
        let updates: Arc<Mutex<Vec<Update>>> = Arc::new(Mutex::new(Vec::new()));
        let update_sink = Arc::clone(&updates);
        let mut controller = Controller::new(
            Box::new(move |update| {
                if matches!(update, Update::Measures(_)) {
                    update_sink.lock().expect("the sink is not poisoned").push(update);
                }
            }),
            None,
            CacheHandle::open_degraded(),
        );
        let report_sink = Arc::clone(&reports);
        controller.sweep_sink = Arc::new(move |report| {
            report_sink.lock().expect("the sink is not poisoned").push(report);
        });
        controller.source_root = Some(folder.clone());
        controller.queue.set_tracks(tracks.clone());
        controller.queue.clear_selection();
        return Scale { controller, folder, tracks, reports, updates, pumped: 0 };
    }

    /// What: `put` writes a real fixture file at the track's path.
    /// Why: a replaced or added file has to exist for the planner to find it.
    fn put(&self, path: &PathBuf, fixture: &str) {
        std::fs::create_dir_all(path.parent().expect("a track has a folder")).expect("the folder is created");
        std::fs::copy(PathBuf::from("fixture").join(fixture), path).expect("the fixture is copied");
    }

    /// What: `pump` hands the controller the sweep reports that arrived since the last call.
    /// Why: the end of a sweep is how the next one is allowed to start.
    fn pump(&mut self) {
        let waiting: Vec<SweepReport> = self.reports.lock().expect("the sink is not poisoned")[self.pumped..].to_vec();
        for report in waiting {
            self.pumped += 1;
            self.controller.handle_command(Command::SweepReport(report));
        }
    }

    /// What: `wait_for_measure` pumps until a measure with a peak arrived for the path, and says how long it took.
    /// Why: the person sees the new peak when that update arrives.
    fn wait_for_measure(&mut self, path: &PathBuf, started: Instant) -> Duration {
        let deadline = Instant::now() + Duration::from_secs(30);
        while Instant::now() < deadline {
            self.pump();
            let seen = self.updates.lock().expect("the sink is not poisoned").iter().any(|update| {
                return matches!(update, Update::Measures(list) if list.iter().any(|(found, measure)| return found == path && measure.peak_dbtp.is_some()));
            });
            if seen {
                return started.elapsed();
            }
            std::thread::sleep(Duration::from_micros(200));
        }
        panic!("the measure of {} did not arrive", path.display());
    }

    /// What: `wait_idle` pumps until the coalescer has no sweep running.
    /// Why: the next run starts a fresh sweep, so the previous one must have ended.
    fn wait_idle(&mut self) {
        let deadline = Instant::now() + Duration::from_secs(30);
        while Instant::now() < deadline {
            self.pump();
            if self.controller.refresh.is_idle() {
                return;
            }
            std::thread::sleep(Duration::from_micros(200));
        }
        panic!("the refresh sweep did not end");
    }
}

/// Removes the folder when the test ends.
impl Drop for Scale {
    /// What: `drop` removes the folder.
    /// Why: tests leave no files behind.
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.folder).ok();
    }
}

#[test]
fn one_replaced_file_in_a_million_tracks_costs_milliseconds_not_a_walk() {
    let mut scale = Scale::new("replace");
    assert_eq!(scale.controller.queue.len(), 1_000_000);
    let fixtures = ["tone.wav", "tone.mp3", "tone.opus", "tone.ogg", "tone.flac"];
    let mut handler: Vec<Duration> = Vec::new();
    let mut shown: Vec<Duration> = Vec::new();
    for run in 0..RUNS {
        let path: PathBuf = scale.tracks[scale.tracks.len() * (run + 1) / (RUNS + 1)].clone();
        scale.put(&path, fixtures[run % fixtures.len()]);
        scale.updates.lock().expect("the sink is not poisoned").clear();
        let started = Instant::now();
        scale.controller.handle_command(Command::SourceChanged(Change::Paths(DirtyPaths::single(path.clone(), false))));
        handler.push(started.elapsed());
        shown.push(scale.wait_for_measure(&path, started));
        scale.wait_idle();
    }
    let median = spread("one replaced file at 1,000,000 tracks: engine-thread handler", &mut handler);
    spread("one replaced file at 1,000,000 tracks: handler until the new peak is shown", &mut shown);
    assert!(median <= REPLACE_BAR, "the handler took {median:?} for one replaced file; a walk of the library would take over a second");
    assert_eq!(scale.controller.queue.len(), 1_000_000, "the list did not change");
}

#[test]
fn one_added_and_one_deleted_file_in_a_million_tracks_edit_the_list_in_place() {
    let mut scale = Scale::new("add-delete");
    let mut added: Vec<Duration> = Vec::new();
    let mut deleted: Vec<Duration> = Vec::new();
    for run in 0..LIST_RUNS {
        let neighbour: PathBuf = scale.tracks[scale.tracks.len() * (run + 1) / (LIST_RUNS + 1)].clone();
        let path: PathBuf = neighbour.with_file_name(format!("zz added {run}.wav"));
        scale.put(&path, "tone.wav");
        let started = Instant::now();
        scale.controller.handle_command(Command::SourceChanged(Change::Paths(DirtyPaths::single(path.clone(), true))));
        added.push(started.elapsed());
        assert_eq!(scale.controller.queue.len(), 1_000_000 + 1, "the file is listed");
        scale.wait_for_measure(&path, started);
        scale.wait_idle();
        std::fs::remove_file(&path).expect("removed");
        let deletion_started = Instant::now();
        scale.controller.handle_command(Command::SourceChanged(Change::Paths(DirtyPaths::single(path.clone(), true))));
        deleted.push(deletion_started.elapsed());
        assert_eq!(scale.controller.queue.len(), 1_000_000, "and unlisted");
        scale.wait_idle();
    }
    let added_median = spread("one added file at 1,000,000 tracks: engine-thread handler (list edit and announcement)", &mut added);
    let deleted_median = spread("one deleted file at 1,000,000 tracks: engine-thread handler (list edit and announcement)", &mut deleted);
    assert!(added_median <= LIST_BAR, "an added file took {added_median:?}");
    assert!(deleted_median <= LIST_BAR, "a deleted file took {deleted_median:?}");
}
