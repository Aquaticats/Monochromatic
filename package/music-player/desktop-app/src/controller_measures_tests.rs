// What:     Unit tests for `controller_measures.rs`, pulled in by `#[cfg(test)] #[path = "controller_measures_tests.rs"] mod tests;`.
// Why:      Reports from a replaced queue must never reach the window, the folder name must be sent for each state,
//           and nothing offers a manual re-analysis (D113).

// What:     `use super::*;` brings the controller methods and their imports into the test scope.
// Why:      The tests drive the methods on a real controller built with a recording callback.
use super::*;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::controller::Controller;
use crate::peakcache::CacheHandle;

// What:     `fn recording_controller() -> (Controller, Arc<Mutex<Vec<String>>>)` builds a controller whose updates are recorded as short tags.
// Why:      `Update` has no `Debug`, so each update is reduced to a tag the assertions can compare.
fn recording_controller() -> (Controller, Arc<Mutex<Vec<String>>>) {
    let log: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let sink_log = Arc::clone(&log);
    let controller = Controller::new(
        Box::new(move |update| {
            let tag = match update {
                Update::Measures(measures) => format!("measures:{}", measures.len()),
                Update::Analysis { done, total } => format!("analysis:{done}/{total}"),
                Update::SourceRoot(name) => format!("root:{name:?}"),
                _ => "other".to_string(),
            };
            sink_log.lock().expect("update log lock").push(tag);
        }),
        None,
        CacheHandle::open_degraded(),
    );
    return (controller, log);
}

// What:     `fn tags(log) -> Vec<String>` copies the recorded tags out of the log.
// Why:      Each assertion compares the whole list, so the lock is released before the comparison.
fn tags(log: &Arc<Mutex<Vec<String>>>) -> Vec<String> {
    return log.lock().expect("update log lock").clone();
}

#[test]
fn progress_from_the_newest_generation_reaches_the_window() {
    let (mut controller, log) = recording_controller();
    controller.analysis_generation = 2;
    controller.handle_sweep_report(SweepReport::Progress { generation: 2, done: 1, total: 3 });
    assert_eq!(tags(&log), vec!["analysis:1/3".to_string()]);
}

#[test]
fn progress_from_a_replaced_queue_is_dropped() {
    let (mut controller, log) = recording_controller();
    controller.analysis_generation = 2;
    controller.handle_sweep_report(SweepReport::Progress { generation: 1, done: 1, total: 3 });
    assert!(tags(&log).is_empty());
}

#[test]
fn measures_from_the_newest_generation_reach_the_window() {
    let (mut controller, log) = recording_controller();
    controller.analysis_generation = 4;
    let measure = (PathBuf::from("/music/a.flac"), crate::track_measure::TrackMeasure::default());
    controller.handle_sweep_report(SweepReport::Measures { generation: 4, measures: vec![measure] });
    assert_eq!(tags(&log), vec!["measures:1".to_string()]);
}

#[test]
fn measures_from_a_replaced_queue_are_dropped() {
    let (mut controller, log) = recording_controller();
    controller.analysis_generation = 4;
    let measure = (PathBuf::from("/music/a.flac"), crate::track_measure::TrackMeasure::default());
    controller.handle_sweep_report(SweepReport::Measures { generation: 3, measures: vec![measure] });
    assert!(tags(&log).is_empty());
}

#[test]
fn source_root_without_a_folder_says_none() {
    let (controller, log) = recording_controller();
    controller.emit_source_root();
    assert_eq!(tags(&log), vec!["root:None".to_string()]);
}

#[test]
fn source_root_names_the_folder_by_its_last_component() {
    let (mut controller, log) = recording_controller();
    controller.source_root = Some(PathBuf::from("/music/Albums"));
    controller.emit_source_root();
    assert_eq!(tags(&log), vec!["root:Some(\"Albums\")".to_string()]);
}

#[test]
fn source_root_of_a_filesystem_root_uses_the_whole_path() {
    assert_eq!(display_name_of(Path::new("/")), "/");
}

#[test]
fn decision_measure_of_the_playing_track_is_published() {
    let (controller, log) = recording_controller();
    let decision = truepeak_core::Decision {
        gain: 0.5,
        kind: truepeak_core::DecisionKind::FullScanExact,
        measured_peak: 0.5,
        duration_secs: 20.0,
    };
    controller.publish_decision_measure(Path::new("/music/a.flac"), &decision);
    assert_eq!(tags(&log), vec!["measures:1".to_string()]);
}
