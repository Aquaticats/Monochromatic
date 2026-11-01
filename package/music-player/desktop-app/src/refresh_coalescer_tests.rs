// What:     Tests of `refresh_coalescer.rs`, pulled in by `refresh_coalescer.rs`.
// Why:      The coalescer decides which refresh sweeps run and which requests merge, so the cases that lose work or run
//           sweeps twice are the ones to prove: a request between the end of a sweep and the handling of its end, a
//           file replaced twice while a sweep runs, a storm, a reset, and an end that is not the running sweep's.

use super::*;

/// What: `files` builds a file-set work from names.
/// Why: tests name files by short strings.
fn files(names: &[&str]) -> Work {
    let check: HashSet<PathBuf> = names.iter().map(|name| return PathBuf::from(name)).collect();
    return Work::Files(Files { check, announce: HashSet::new() });
}

#[test]
fn the_first_request_starts_a_sweep_at_once() {
    let mut coalescer = RefreshCoalescer::default();
    let launch = coalescer.submit(files(&["a"])).expect("idle: a sweep starts");
    assert_eq!(launch.work, files(&["a"]));
}

#[test]
fn requests_during_a_sweep_become_one_follow_up_with_the_union() {
    let mut coalescer = RefreshCoalescer::default();
    let first = coalescer.submit(files(&["a"])).expect("starts");
    assert_eq!(coalescer.submit(files(&["b"])), None);
    assert_eq!(coalescer.submit(files(&["c", "b"])), None);
    let follow_up = coalescer.finished(first.id).expect("the pending work starts when the sweep ends");
    assert_eq!(follow_up.work, files(&["b", "c"]));
    assert_ne!(follow_up.id, first.id);
    assert_eq!(coalescer.finished(follow_up.id), None, "and then idle");
    assert!(coalescer.submit(files(&["d"])).is_some(), "idle again: the next request starts at once");
}

#[test]
fn a_file_replaced_twice_while_a_sweep_runs_is_looked_at_once_more_not_twice() {
    let mut coalescer = RefreshCoalescer::default();
    let first = coalescer.submit(files(&["song.flac"])).expect("starts");
    assert_eq!(coalescer.submit(files(&["song.flac"])), None, "the first replacement while it runs");
    assert_eq!(coalescer.submit(files(&["song.flac"])), None, "the second replacement while it runs");
    let follow_up = coalescer.finished(first.id).expect("one follow-up");
    assert_eq!(follow_up.work, files(&["song.flac"]));
    assert_eq!(coalescer.finished(follow_up.id), None, "and no more");
}

#[test]
fn a_request_that_lands_between_the_end_of_a_sweep_and_the_handling_of_its_end_is_not_lost() {
    let mut coalescer = RefreshCoalescer::default();
    let first = coalescer.submit(files(&["a"])).expect("starts");
    // The sweep thread has finished and queued its end; the engine handles a watcher request first.
    assert_eq!(coalescer.submit(files(&["late"])), None);
    let follow_up = coalescer.finished(first.id).expect("the end starts the late request");
    assert_eq!(follow_up.work, files(&["late"]));
}

#[test]
fn everything_absorbs_files_and_files_do_not_shrink_everything() {
    let mut coalescer = RefreshCoalescer::default();
    let first = coalescer.submit(files(&["a"])).expect("starts");
    assert_eq!(coalescer.submit(files(&["b"])), None);
    assert_eq!(coalescer.submit(Work::Everything), None);
    assert_eq!(coalescer.submit(files(&["c"])), None);
    assert_eq!(coalescer.pending(), Some(&Work::Everything));
    let follow_up = coalescer.finished(first.id).expect("starts");
    assert_eq!(follow_up.work, Work::Everything);
}

#[test]
fn a_storm_of_requests_during_a_sweep_stays_bounded_and_becomes_one_follow_up() {
    let mut coalescer = RefreshCoalescer::default();
    let first = coalescer.submit(files(&["a"])).expect("starts");
    for number in 0..100_000_usize {
        assert_eq!(coalescer.submit(files(&[&format!("/lib/{}.flac", number % 50)])), None);
    }
    let Some(Work::Files(waiting)) = coalescer.pending().cloned() else {
        panic!("files are pending");
    };
    assert_eq!(waiting.check.len(), 50);
    let follow_up = coalescer.finished(first.id).expect("one follow-up");
    assert!(matches!(follow_up.work, Work::Files(ref set) if set.check.len() == 50));
    assert_eq!(coalescer.finished(follow_up.id), None);
}

#[test]
fn more_distinct_files_than_the_cap_become_everything() {
    let mut coalescer = RefreshCoalescer::default();
    let _first = coalescer.submit(files(&["a"])).expect("starts");
    let big: HashSet<PathBuf> = (0..=MAX_DIRTY_PATHS).map(|number| return PathBuf::from(format!("/lib/{number}.flac"))).collect();
    assert_eq!(coalescer.submit(Work::Files(Files { check: big, announce: HashSet::new() })), None);
    assert_eq!(coalescer.pending(), Some(&Work::Everything));
}

#[test]
fn an_empty_request_starts_nothing_and_pends_nothing() {
    let mut coalescer = RefreshCoalescer::default();
    assert_eq!(coalescer.submit(files(&[])), None);
    let first = coalescer.submit(files(&["a"])).expect("starts");
    assert_eq!(coalescer.submit(files(&[])), None);
    assert_eq!(coalescer.pending(), None);
    assert_eq!(coalescer.finished(first.id), None);
}

#[test]
fn a_reset_drops_what_waited_and_the_end_of_the_old_sweep_is_ignored() {
    let mut coalescer = RefreshCoalescer::default();
    let old = coalescer.submit(files(&["old"])).expect("starts");
    assert_eq!(coalescer.submit(files(&["queued for the old folder"])), None);
    coalescer.reset();
    assert_eq!(coalescer.pending(), None);
    let fresh = coalescer.submit(files(&["new"])).expect("after a reset the next request starts at once");
    assert_eq!(coalescer.finished(old.id), None, "the old sweep's end does not release the new sweep");
    assert_eq!(coalescer.submit(files(&["during the new sweep"])), None, "the new sweep still runs");
    assert!(coalescer.finished(fresh.id).is_some());
}

#[test]
fn an_end_that_is_not_the_running_sweeps_is_ignored() {
    let mut coalescer = RefreshCoalescer::default();
    assert_eq!(coalescer.finished(99), None, "nothing runs");
    let first = coalescer.submit(files(&["a"])).expect("starts");
    assert_eq!(coalescer.submit(files(&["b"])), None);
    assert_eq!(coalescer.finished(first.id + 1), None, "another number");
    assert!(coalescer.pending().is_some(), "the pending work stays");
}

#[test]
fn what_is_new_to_the_list_stays_announced_through_a_merge() {
    let mut coalescer = RefreshCoalescer::default();
    let _first = coalescer.submit(files(&["running"])).expect("starts");
    let added = Work::Files(Files { check: [PathBuf::from("new.flac"), PathBuf::from("old.flac")].into_iter().collect(), announce: [PathBuf::from("new.flac")].into_iter().collect() });
    assert_eq!(coalescer.submit(added), None);
    assert_eq!(coalescer.submit(files(&["later.flac"])), None);
    let Some(Work::Files(waiting)) = coalescer.pending().cloned() else {
        panic!("files are pending");
    };
    assert_eq!(waiting.check.len(), 3);
    assert_eq!(waiting.announce, [PathBuf::from("new.flac")].into_iter().collect::<HashSet<_>>());
}

#[test]
fn a_new_file_named_after_a_plain_request_is_still_announced_when_they_merge() {
    let mut coalescer = RefreshCoalescer::default();
    let _first = coalescer.submit(files(&["running"])).expect("starts");
    assert_eq!(coalescer.submit(files(&["old.flac"])), None);
    let added = Work::Files(Files { check: [PathBuf::from("new.flac")].into_iter().collect(), announce: [PathBuf::from("new.flac")].into_iter().collect() });
    assert_eq!(coalescer.submit(added), None);
    let Some(Work::Files(waiting)) = coalescer.pending().cloned() else {
        panic!("files are pending");
    };
    assert_eq!(waiting.announce, [PathBuf::from("new.flac")].into_iter().collect::<HashSet<_>>());
}
