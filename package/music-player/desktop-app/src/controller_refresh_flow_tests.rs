// What:     Controller tests for D113 refreshes driven by the paths the watcher names (`Command::SourceChanged`), pulled in
//           as a child of `controller_refresh_tests.rs` so they share its rig.
// Why:      The refresh must cost what changed, not the size of the library: a replaced file is measured again with no
//           change of the list, an added or deleted file edits the list in place, nothing is walked, overlapping refreshes
//           merge, and a newly opened folder forgets what was asked for the old one.

use super::*;
use crate::change_set::{Change, DirtyPaths};

/// What: `named` is the command the watcher sends for one path.
/// Why: every test drives the controller with it.
fn named(path: PathBuf, structural: bool) -> Command {
    return Command::SourceChanged(Change::Paths(DirtyPaths::single(path, structural)));
}

/// What: `reconciled_names` lists the name lists of every `Reconciled` update so far.
/// Why: a changed list is announced by `Reconciled`; an unchanged one by nothing.
fn reconciled_names(rig: &Rig) -> Vec<Vec<String>> {
    return rig
        .updates
        .lock()
        .expect("the sink is not poisoned")
        .iter()
        .filter_map(|update| return if let Update::Reconciled { names, .. } = update { Some(names.clone()) } else { None })
        .collect();
}

/// What: `done_reports` counts the refresh sweeps that have ended, as the controller has been told.
/// Why: each refresh sweep ends with one report, so the count is the number of sweeps that ran.
fn done_reports(rig: &Rig) -> usize {
    return rig.reports.lock().expect("the sink is not poisoned").iter().filter(|report| return matches!(report, SweepReport::RefreshDone { .. })).count();
}

#[test]
fn a_replaced_file_is_measured_again_and_the_list_is_left_alone() {
    let mut rig = Rig::new("flow-replace", &[("a.flac", "tone.flac"), ("b.flac", "tone.wav")]);
    rig.open();
    assert!(rig.wait_for_peaks("a.flac", 1));
    rig.settle();
    let generation = rig.controller.analysis_generation;
    let path = rig.root.join("a.flac");
    rig.put("a.flac", "tone.mp3");
    rig.controller.handle_command(named(path, false));
    assert!(rig.wait_for_peaks("a.flac", 2), "the replaced file was measured by the sweep of that one path");
    assert_eq!(rig.measures_with_peak("b.flac"), 1, "the other file was not decoded again");
    assert!(reconciled_names(&rig).is_empty(), "no list was sent: nothing was added or removed");
    assert_eq!(rig.controller.analysis_generation, generation, "the load's generation stands");
}

#[test]
fn nothing_is_walked_a_file_nobody_named_is_not_listed() {
    let mut rig = Rig::new("flow-no-walk", &[("a.flac", "tone.flac")]);
    rig.open();
    assert!(rig.wait_for_peaks("a.flac", 1));
    rig.put("unnamed.flac", "tone.wav");
    rig.put("a.flac", "tone.mp3");
    rig.controller.handle_command(named(rig.root.join("a.flac"), false));
    rig.settle();
    assert_eq!(rig.controller.queue.len(), 1, "a walk would have found the unnamed file; the refresh looked only at what was named");
}

#[test]
fn an_added_file_is_listed_in_place_announced_and_measured_without_a_new_load() {
    let mut rig = Rig::new("flow-add", &[("a.flac", "tone.flac"), ("c.flac", "tone.wav")]);
    rig.open();
    assert!(rig.wait_for_peaks("c.flac", 1));
    rig.settle();
    let generation = rig.controller.analysis_generation;
    rig.put("b.flac", "tone.mp3");
    rig.controller.handle_command(named(rig.root.join("b.flac"), true));
    let names = reconciled_names(&rig);
    assert_eq!(names.last().map(Vec::len), Some(3), "the list has the new file at once");
    assert_eq!(rig.controller.queue.tracks().iter().map(|path| return path.file_name().map(|name| return name.to_string_lossy().into_owned()).unwrap_or_default()).collect::<Vec<_>>(), vec!["a.flac", "b.flac", "c.flac"]);
    assert!(rig.wait_for_peaks("b.flac", 1), "and measured");
    assert_eq!(rig.measures_with_peak("a.flac"), 1, "the others were not decoded again");
    assert_eq!(rig.controller.analysis_generation, generation, "no new load");
}

#[test]
fn a_deleted_file_leaves_the_list_and_nothing_is_decoded() {
    let mut rig = Rig::new("flow-delete", &[("a.flac", "tone.flac"), ("b.flac", "tone.wav"), ("c.flac", "tone.mp3")]);
    rig.open();
    assert!(rig.wait_for_peaks("c.flac", 1));
    rig.settle();
    let before: usize = ["a.flac", "b.flac", "c.flac"].iter().map(|name| return rig.measures_with_peak(name)).sum();
    std::fs::remove_file(rig.root.join("b.flac")).expect("removed");
    rig.controller.handle_command(named(rig.root.join("b.flac"), true));
    assert_eq!(reconciled_names(&rig).last().map(Vec::len), Some(2));
    rig.settle();
    assert_eq!(["a.flac", "b.flac", "c.flac"].iter().map(|name| return rig.measures_with_peak(name)).sum::<usize>(), before);
}

#[test]
fn the_selected_track_follows_its_path_when_files_are_added_before_it_and_a_removed_one_says_so() {
    let mut rig = Rig::new("flow-selected", &[("m.flac", "tone.flac"), ("n.flac", "tone.wav")]);
    rig.controller.handle_command(Command::OpenRoot { root: rig.root.clone(), select: Some(rig.root.join("n.flac")), play: false });
    assert_eq!(rig.controller.queue.current_index(), Some(1));
    rig.put("a.flac", "tone.mp3");
    rig.controller.handle_command(named(rig.root.join("a.flac"), true));
    assert_eq!(rig.controller.queue.current_index(), Some(2), "the highlight moved with the track");
    assert_eq!(rig.controller.queue.current_path(), Some(&rig.root.join("n.flac")));
    std::fs::remove_file(rig.root.join("n.flac")).expect("removed");
    rig.controller.handle_command(named(rig.root.join("n.flac"), true));
    assert!(rig.controller.queue.current_path().is_none());
    assert!(rig.updates.lock().expect("the sink is not poisoned").iter().any(|update| return matches!(update, Update::FileGone { .. })), "the person is told why the track left");
}

#[test]
fn a_folder_deleted_and_recreated_with_other_files_is_listed_as_it_is_now() {
    let mut rig = Rig::new("flow-folder", &[("album/x.flac", "tone.flac"), ("album/y.flac", "tone.wav"), ("keep.flac", "tone.mp3")]);
    rig.open();
    assert!(rig.wait_for_peaks("y.flac", 1));
    std::fs::remove_dir_all(rig.root.join("album")).expect("removed");
    rig.put("album/z.flac", "tone.opus");
    rig.controller.handle_command(named(rig.root.join("album"), true));
    let names: Vec<String> = rig.controller.queue.tracks().iter().map(|path| return path.strip_prefix(&rig.root).expect("below the root").components().map(|part| return part.as_os_str().to_string_lossy().into_owned()).collect::<Vec<String>>().join("/")).collect();
    assert_eq!(names, vec!["keep.flac", "album/z.flac"]);
    assert!(rig.wait_for_peaks("z.flac", 1));
}

#[test]
fn overlapping_events_run_one_sweep_and_one_follow_up_that_covers_them_all() {
    let mut rig = Rig::new("flow-coalesce", &[("a.flac", "tone.flac"), ("b.flac", "tone.wav"), ("c.flac", "tone.mp3")]);
    rig.open();
    assert!(rig.wait_for_peaks("c.flac", 1));
    rig.settle();
    let sweeps_before = done_reports(&rig);
    rig.put("a.flac", "tone.opus");
    rig.controller.handle_command(named(rig.root.join("a.flac"), false));
    // The first sweep's end is not handled yet (no pump), so the next events join the pending follow-up.
    rig.put("b.flac", "tone.mp3");
    rig.controller.handle_command(named(rig.root.join("b.flac"), false));
    rig.put("c.flac", "tone.opus");
    rig.controller.handle_command(named(rig.root.join("c.flac"), false));
    rig.put("b.flac", "tone.ogg");
    rig.controller.handle_command(named(rig.root.join("b.flac"), false));
    assert!(rig.wait_for_peaks("c.flac", 2), "the follow-up covered the files named while the first sweep ran");
    assert!(rig.wait_for_peaks("b.flac", 2));
    rig.settle();
    assert_eq!(done_reports(&rig) - sweeps_before, 2, "one sweep, and one follow-up for all the events that came meanwhile");
}

#[test]
fn a_file_replaced_twice_while_a_sweep_runs_ends_with_the_last_content_measured() {
    let mut rig = Rig::new("flow-twice", &[("a.flac", "tone.flac"), ("b.flac", "tone.wav")]);
    rig.open();
    assert!(rig.wait_for_peaks("b.flac", 1));
    rig.settle();
    let path = rig.root.join("a.flac");
    rig.put("a.flac", "tone.mp3");
    rig.controller.handle_command(named(path.clone(), false));
    rig.put("a.flac", "tone.opus");
    rig.controller.handle_command(named(path.clone(), false));
    rig.settle();
    let key = crate::peakcache::fingerprint(&path).expect("the file has a fingerprint");
    assert!(rig.controller.peaks.get(key).is_some(), "the content that is on disk now has its decision");
    assert!(rig.measures_with_peak("a.flac") <= 3, "the load, the first replacement and the last: no content was decoded twice");
}

#[test]
fn opening_a_folder_drops_what_was_asked_for_the_old_one() {
    let mut rig = Rig::new("flow-reopen", &[("a.flac", "tone.flac")]);
    rig.open();
    assert!(rig.wait_for_peaks("a.flac", 1));
    rig.put("a.flac", "tone.mp3");
    rig.controller.handle_command(named(rig.root.join("a.flac"), false));
    rig.controller.handle_command(named(rig.root.join("a.flac"), false));
    assert!(rig.controller.refresh.pending().is_some(), "the second request waits for the first sweep");
    rig.open();
    assert!(rig.controller.refresh.pending().is_none(), "a new folder forgets it");
}

#[test]
fn lost_events_walk_everything_and_find_a_file_nobody_named() {
    let mut rig = Rig::new("flow-lost", &[("a.flac", "tone.flac")]);
    rig.open();
    assert!(rig.wait_for_peaks("a.flac", 1));
    rig.put("quiet.flac", "tone.wav");
    rig.controller.handle_command(Command::SourceChanged(Change::Everything));
    assert_eq!(rig.controller.queue.len(), 2, "after lost events the folder is walked");
    assert!(rig.wait_for_peaks("quiet.flac", 1));
}

#[test]
fn an_event_for_a_file_that_did_not_change_decodes_nothing() {
    let mut rig = Rig::new("flow-unchanged", &[("a.flac", "tone.flac"), ("b.flac", "tone.wav")]);
    rig.open();
    assert!(rig.wait_for_peaks("b.flac", 1));
    rig.settle();
    let path = rig.root.join("a.flac");
    // A touch with no change of size or time, as an attribute event or a read-write cycle leaves it.
    rig.controller.handle_command(named(path.clone(), false));
    rig.controller.handle_command(named(path, false));
    rig.settle();
    assert_eq!(rig.measures_with_peak("a.flac"), 1, "the known fingerprint was asked of the cache, not decoded");
    assert_eq!(rig.measures_with_peak("b.flac"), 1);
}

#[test]
fn a_new_row_gets_its_length_at_once_before_its_peak() {
    let mut rig = Rig::new("flow-announce", &[("a.flac", "tone.flac")]);
    rig.open();
    assert!(rig.wait_for_peaks("a.flac", 1));
    rig.settle();
    rig.put("b.flac", "tone.wav");
    rig.controller.handle_command(named(rig.root.join("b.flac"), true));
    assert!(rig.wait_for_peaks("b.flac", 1));
    let first: Option<(Option<f64>, Option<f64>)> = rig.updates.lock().expect("the sink is not poisoned").iter().find_map(|update| {
        return match update {
            Update::Measures(list) => list.iter().find(|(path, _)| return path.ends_with("b.flac")).map(|(_, measure)| return (measure.duration_secs, measure.peak_dbtp)),
            _ => None,
        };
    });
    let (duration, peak) = first.expect("the new row was announced");
    assert!(duration.is_some(), "the row has its length at once");
    assert!(peak.is_none(), "from the header, before the decode that finds the peak");
}

#[test]
fn a_replaced_selected_file_named_by_the_watcher_is_loaded_again_at_once() {
    let mut rig = Rig::new("flow-selected-reload", &[("a.flac", "tone.flac"), ("b.flac", "tone.wav")]);
    let selected = rig.root.join("a.flac");
    rig.controller.handle_command(Command::OpenRoot { root: rig.root.clone(), select: Some(selected.clone()), play: false });
    let first = rig.controller.loaded_file.clone().expect("the selected file is loaded");
    rig.put("a.flac", "tone.mp3");
    rig.controller.handle_command(named(selected, false));
    let second = rig.controller.loaded_file.clone().expect("the selected file is loaded again");
    assert_ne!(second.1, first.1, "the decoder follows the new file the moment the watcher names it");
}

#[test]
fn a_named_file_whose_decision_is_already_stored_is_not_decoded_even_in_a_new_generation() {
    let mut rig = Rig::new("flow-stored", &[("a.flac", "tone.flac"), ("b.flac", "tone.wav")]);
    rig.open();
    assert!(rig.wait_for_peaks("b.flac", 1));
    rig.settle();
    // A later run of the app starts with an empty set of claims but the stored decisions of the earlier run.
    rig.controller.sweep_claims = Arc::new(Mutex::new(std::collections::HashSet::new()));
    let path = rig.root.join("a.flac");
    rig.controller.handle_command(named(path.clone(), false));
    rig.controller.handle_command(named(path, false));
    rig.settle();
    assert_eq!(rig.measures_with_peak("a.flac"), 1, "the cache already holds that file's exact decision, so it is not decoded again");
}

#[test]
fn an_unrelated_change_retries_a_file_that_failed_to_decode_and_is_not_named() {
    let mut rig = Rig::new("flow-retry-failed", &[("good.flac", "tone.flac"), ("bad.flac", "garbage one")]);
    rig.open();
    assert!(rig.wait_for(|updates| return updates.iter().any(|update| return matches!(update, Update::Unplayable(_)))), "the broken file is reported");
    rig.settle();
    // The broken file is completed on disk, but no event names it: the watcher missed it, or it was written before the watch.
    rig.put("bad.flac", "tone.wav");
    // A change to another file elsewhere in the library is the next library change.
    rig.put("other.flac", "tone.mp3");
    rig.controller.handle_command(named(rig.root.join("other.flac"), true));
    assert!(rig.wait_for_peaks("bad.flac", 1), "the recorded failure was tried again by the unrelated change and now decodes");
    assert_eq!(rig.unplayable_updates().len(), 1, "and it was told only once");
}

#[test]
fn a_recorded_failure_that_is_still_broken_is_retried_by_each_change_and_told_once() {
    let mut rig = Rig::new("flow-retry-still-broken", &[("good.flac", "tone.flac"), ("bad.flac", "garbage one")]);
    rig.open();
    assert!(rig.wait_for(|updates| return updates.iter().any(|update| return matches!(update, Update::Unplayable(_)))));
    rig.settle();
    for round in 0..3 {
        rig.put(&format!("extra{round}.flac"), "tone.wav");
        rig.controller.handle_command(named(rig.root.join(format!("extra{round}.flac")), true));
        rig.settle();
    }
    assert_eq!(rig.unplayable_updates().len(), 1, "told once however often it was retried");
    assert!(rig.controller.reported_unplayable.lock().expect("not poisoned").contains(&rig.root.join("bad.flac")), "and it stays recorded");
}

#[test]
fn a_recorded_failure_whose_file_was_deleted_is_forgotten() {
    let mut rig = Rig::new("flow-forget-failed", &[("good.flac", "tone.flac"), ("bad.flac", "garbage one")]);
    rig.open();
    assert!(rig.wait_for(|updates| return updates.iter().any(|update| return matches!(update, Update::Unplayable(_)))));
    std::fs::remove_file(rig.root.join("bad.flac")).expect("removed");
    rig.controller.handle_command(named(rig.root.join("bad.flac"), true));
    assert!(rig.controller.reported_unplayable.lock().expect("not poisoned").is_empty(), "a deleted file is not kept in the set");
}
