// What:     Controller tests for D113: the library and its true peaks follow the disk without any user action.
//           Each test builds a controller over a disposable folder and a throwaway cache file, feeds the controller its own
//           sweep reports the way the engine does, and reads the updates the window would receive.
// Why:      D113 removed Rescan and Re-analyze from the interface, so this is the only way a replaced, edited, new or broken
//           file gets measured, marked or reported. The file watcher sends `Command::Rescan`; the tests send it directly.

use super::*;
use crate::command::{Command, Update};
use crate::peakcache::CacheHandle;
use crate::sweep_report::SweepReport;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// What: `Rig` is a disposable library folder, a controller over it, and what the controller and its sweeps reported.
/// Why: every test needs the three together, and the folder and cache file go away when the value drops.
struct Rig {
    controller: Controller,
    root: PathBuf,
    cache_file: PathBuf,
    reports: Arc<Mutex<Vec<SweepReport>>>,
    updates: Arc<Mutex<Vec<Update>>>,
    pumped: usize,
}

/// Methods of `Rig`.
impl Rig {
    /// What: `new` makes the folder with the given `(relative name, fixture)` files and opens it in a new controller.
    /// Why: tests choose which files exist at the start; a fixture name that is not a file under `fixture/` is written as
    /// bytes that cannot be decoded.
    fn new(tag: &str, files: &[(&str, &str)]) -> Rig {
        let nanos: u128 = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |elapsed| return elapsed.as_nanos());
        let root = crate::test_scratch::scratch_root().join(format!("music-player-refresh-{}-{nanos}-{tag}", std::process::id()));
        let cache_file = crate::test_scratch::scratch_root().join(format!("music-player-refresh-{}-{nanos}-{tag}.db", std::process::id()));
        std::fs::create_dir_all(&root).expect("the folder is created");
        let reports: Arc<Mutex<Vec<SweepReport>>> = Arc::new(Mutex::new(Vec::new()));
        let updates: Arc<Mutex<Vec<Update>>> = Arc::new(Mutex::new(Vec::new()));
        let update_sink = Arc::clone(&updates);
        let mut controller = Controller::new(
            Box::new(move |update| {
                update_sink.lock().expect("the sink is not poisoned").push(update);
            }),
            None,
            CacheHandle::open_at(cache_file.clone()),
        );
        let report_sink = Arc::clone(&reports);
        controller.sweep_sink = Arc::new(move |report| {
            report_sink.lock().expect("the sink is not poisoned").push(report);
        });
        let rig = Rig { controller, root, cache_file, reports, updates, pumped: 0 };
        for (name, fixture) in files {
            rig.put(name, fixture);
        }
        return rig;
    }

    /// What: `put` writes one file of the library: a copy of the fixture, or bytes that cannot be decoded.
    /// Why: replacing a file is how the tests change the disk under a running controller.
    fn put(&self, name: &str, fixture: &str) {
        let path = self.root.join(name);
        std::fs::create_dir_all(path.parent().expect("a file has a folder")).expect("the folder is created");
        let source = PathBuf::from("fixture").join(fixture);
        if source.is_file() {
            std::fs::copy(&source, &path).expect("the fixture is copied");
        } else {
            std::fs::write(&path, format!("this is not audio: {fixture}").as_bytes()).expect("the broken file is written");
        }
    }

    /// What: `open` opens the folder as a launch or the folder dialog does, selecting and playing nothing.
    /// Why: the sweep of the load starts here.
    fn open(&mut self) {
        self.controller.handle_command(Command::OpenRoot { root: self.root.clone(), select: None, play: false });
    }

    /// What: `pump` hands the controller every sweep report that arrived since the last call, as the engine's loop does.
    /// Why: the controller turns reports into updates and drops stale ones.
    fn pump(&mut self) {
        let waiting: Vec<SweepReport> = self.reports.lock().expect("the sink is not poisoned").clone();
        for report in waiting.into_iter().skip(self.pumped) {
            self.pumped += 1;
            self.controller.handle_command(Command::SweepReport(report));
        }
    }

    /// What: `wait_for` pumps reports until `condition` holds over the recorded updates, or ten seconds pass.
    /// Why: sweeps run on their own threads, so the tests wait for their effects.
    fn wait_for(&mut self, condition: impl Fn(&[Update]) -> bool) -> bool {
        let deadline = Instant::now() + Duration::from_secs(10);
        while Instant::now() < deadline {
            self.pump();
            if condition(&self.updates.lock().expect("the sink is not poisoned")) {
                return true;
            }
            std::thread::sleep(Duration::from_millis(40));
        }
        return false;
    }

    /// What: `settle` pumps reports for a while with nothing expected.
    /// Why: a test that asserts that something did not happen must give it the time to happen.
    fn settle(&mut self) {
        let deadline = Instant::now() + Duration::from_millis(1_200);
        while Instant::now() < deadline {
            self.pump();
            std::thread::sleep(Duration::from_millis(40));
        }
    }

    /// What: `wait_for_peaks` pumps reports until at least `count` measures with a true peak arrived for the file `name`.
    /// Why: one such measure is one decode of that file's current content.
    fn wait_for_peaks(&mut self, name: &str, count: usize) -> bool {
        let deadline = Instant::now() + Duration::from_secs(10);
        while Instant::now() < deadline {
            self.pump();
            if self.measures_with_peak(name) >= count {
                return true;
            }
            std::thread::sleep(Duration::from_millis(40));
        }
        return false;
    }

    /// What: `measures_with_peak` counts the measures that carry a true peak for the file called `name`.
    /// Why: one such measure is one decode of that file's current content.
    fn measures_with_peak(&self, name: &str) -> usize {
        return self
            .updates
            .lock()
            .expect("the sink is not poisoned")
            .iter()
            .map(|update| return peaks_for(update, name))
            .sum();
    }

    /// What: `unplayable_updates` lists the unplayable reports the window received, as the file names in each.
    /// Why: a failure must be told once.
    fn unplayable_updates(&self) -> Vec<Vec<String>> {
        return self
            .updates
            .lock()
            .expect("the sink is not poisoned")
            .iter()
            .filter_map(unplayable_names)
            .collect();
    }
}

/// What: `peaks_for` counts the measures with a true peak that one update carries for the file called `name`.
/// Why: one such measure is one decode of that file's current content.
fn peaks_for(update: &Update, name: &str) -> usize {
    return match update {
        Update::Measures(list) => list
            .iter()
            .filter(|(path, measure)| return path.ends_with(name) && measure.peak_dbtp.is_some())
            .count(),
        _ => 0,
    };
}

/// What: `unplayable_names` gives the file names of an unplayable update, or nothing for any other update.
/// Why: the tests compare what the window was told, in plain names.
fn unplayable_names(update: &Update) -> Option<Vec<String>> {
    return match update {
        Update::Unplayable(paths) => Some(
            paths
                .iter()
                .map(|path| return path.file_name().map_or_else(String::new, |name| return name.to_string_lossy().into_owned()))
                .collect(),
        ),
        _ => None,
    };
}

/// Removes the folder and the cache file when the test ends.
impl Drop for Rig {
    /// What: `drop` removes the folder and the cache file.
    /// Why: tests leave no files behind.
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.root).ok();
        std::fs::remove_file(&self.cache_file).ok();
    }
}

#[test]
fn a_file_replaced_on_disk_gets_a_new_measure_by_itself() {
    let mut rig = Rig::new("replaced", &[("a.flac", "tone.flac")]);
    rig.open();
    assert!(rig.wait_for_peaks("a.flac", 1), "the load's sweep measured the file");
    rig.settle();
    assert_eq!(rig.measures_with_peak("a.flac"), 1, "once");
    rig.put("a.flac", "tone.mp3");
    rig.controller.handle_command(Command::Rescan);
    assert!(rig.wait_for_peaks("a.flac", 2), "the replaced file was measured again without any command but the watcher's");
}

#[test]
fn a_rescan_with_nothing_changed_decodes_nothing_and_shows_no_progress() {
    let mut rig = Rig::new("unchanged", &[("a.flac", "tone.flac"), ("b.flac", "tone.wav")]);
    rig.open();
    assert!(rig.wait_for(|updates| return updates.iter().any(|update| return matches!(update, Update::Analysis { done, total } if done == total && *total == 2))));
    let measured = rig.measures_with_peak("a.flac") + rig.measures_with_peak("b.flac");
    let analysis_updates = rig.updates.lock().expect("the sink is not poisoned").iter().filter(|update| return matches!(update, Update::Analysis { .. })).count();
    rig.controller.handle_command(Command::Rescan);
    rig.controller.handle_command(Command::Rescan);
    rig.settle();
    assert_eq!(rig.measures_with_peak("a.flac") + rig.measures_with_peak("b.flac"), measured, "no file was decoded again");
    let after = rig.updates.lock().expect("the sink is not poisoned").iter().filter(|update| return matches!(update, Update::Analysis { .. })).count();
    assert_eq!(after, analysis_updates, "the refresh sweep draws no progress");
}

#[test]
fn a_new_file_appears_and_is_measured_without_any_action() {
    let mut rig = Rig::new("new-file", &[("a.flac", "tone.flac")]);
    rig.open();
    assert!(rig.wait_for_peaks("a.flac", 1));
    rig.put("b.flac", "tone.wav");
    rig.controller.handle_command(Command::Rescan);
    assert!(rig.wait_for(|updates| return updates.iter().any(|update| return matches!(update, Update::Reconciled { names, .. } if names.len() == 2))), "the new file is listed");
    assert!(rig.wait_for_peaks("b.flac", 1), "and measured");
}

#[test]
fn a_file_that_cannot_be_decoded_is_reported_once_retried_and_reported_again_only_after_it_decoded() {
    let mut rig = Rig::new("broken", &[("good.flac", "tone.flac"), ("bad.flac", "garbage one")]);
    rig.open();
    assert!(rig.wait_for(|updates| return updates.iter().any(|update| return matches!(update, Update::Unplayable(_)))), "the broken file is reported");
    assert_eq!(rig.unplayable_updates(), vec![vec!["bad.flac".to_string()]]);
    rig.controller.handle_command(Command::Rescan);
    rig.settle();
    assert_eq!(rig.unplayable_updates().len(), 1, "the next sweep tried it again and did not say it again");
    rig.put("bad.flac", "garbage two, longer than the first");
    rig.controller.handle_command(Command::Rescan);
    rig.settle();
    assert_eq!(rig.unplayable_updates().len(), 1, "a file that is still broken after it changed is not told again, so a copy in progress says it once");
    rig.put("bad.flac", "tone.wav");
    rig.controller.handle_command(Command::Rescan);
    assert!(rig.wait_for_peaks("bad.flac", 1), "once it decodes it is measured");
    rig.put("bad.flac", "garbage three");
    rig.controller.handle_command(Command::Rescan);
    assert!(
        rig.wait_for(|updates| return updates.iter().filter(|update| return matches!(update, Update::Unplayable(_))).count() >= 2),
        "a file that decoded and then broke again is told again",
    );
}

#[test]
fn a_new_folder_open_forgets_what_was_reported_so_its_rows_are_marked_again() {
    let mut rig = Rig::new("reopen", &[("bad.flac", "garbage")]);
    rig.open();
    assert!(rig.wait_for(|updates| return updates.iter().any(|update| return matches!(update, Update::Unplayable(_)))));
    rig.open();
    assert!(rig.wait_for(|updates| return updates.iter().filter(|update| return matches!(update, Update::Unplayable(_))).count() >= 2), "opening the folder again reports it again");
}

#[test]
fn a_paused_selected_file_that_is_replaced_is_loaded_again_and_a_playing_one_is_loaded_again_without_stopping() {
    let mut rig = Rig::new("selected", &[("a.flac", "tone.flac"), ("b.flac", "tone.wav")]);
    let selected = rig.root.join("a.flac");
    rig.controller.handle_command(Command::OpenRoot { root: rig.root.clone(), select: Some(selected.clone()), play: false });
    let first = rig.controller.loaded_file.clone().expect("the selected file is loaded");
    assert_eq!(first.0, selected);
    rig.put("a.flac", "tone.mp3");
    rig.controller.handle_command(Command::Rescan);
    let second = rig.controller.loaded_file.clone().expect("the selected file is loaded again");
    assert_ne!(second.1, first.1, "the paused selection follows the new file");
    assert!(rig.controller.source.is_some());
    assert!(!rig.controller.playing, "and it stays paused");
    rig.controller.handle_command(Command::Play);
    assert!(rig.controller.playing);
    rig.put("a.flac", "tone.opus");
    rig.controller.handle_command(Command::Rescan);
    let third = rig.controller.loaded_file.clone().expect("the selected file is loaded again");
    assert_ne!(third.1, second.1, "a playing track follows its new file too (D113)");
    assert!(rig.controller.playing, "and keeps playing");
}

/// Tests of the refresh from the paths the watcher names, with the same rig.
#[path = "controller_refresh_flow_tests.rs"]
mod flow;
