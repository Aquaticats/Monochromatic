//! Controls for the names one walk over a state directory reaches:
//! what a walk that stops part-way gives the callers that only work with the queue.

use super::*;

/// What:
///  A walk that failed on its third entry,
///  behind two names it reached.
/// Why:
///  A directory that cannot be listed at all and one that stopped part-way are the two ways a walk ends early,
/// and this is the second as `names_reached` sees it.
fn a_walk_that_stopped(stopped_by: std::io::Error) -> WhatTheWalkReached {
    let entries: Vec<std::io::Result<(String, SystemTime)>> = vec![
        Ok((String::from("push-one-f921bd05e68b0374"), SystemTime::UNIX_EPOCH)),
        Ok((String::from("push-two-f921bd05e68b0374"), SystemTime::UNIX_EPOCH)),
        Err(stopped_by),
        Ok((String::from("push-three-f921bd05e68b0374"), SystemTime::UNIX_EPOCH)),
    ];
    let mut walked = entries.into_iter();
    return names_reached(&mut walked);
}

/// A walk that stops part-way keeps the names it reached,
/// so a caller that only works reads a shorter queue and never an empty one:
/// `main` kept them, and a caller that read an empty queue would take nothing that is owed.
#[test]
fn a_walk_that_stops_part_way_keeps_the_names_it_reached() {
    let reached: WhatTheWalkReached = a_walk_that_stopped(std::io::Error::from_raw_os_error(5));
    assert_eq!(reached.names.len(), 2, "the walk stopped at its third entry: {:?}", reached.names);
    assert!(!reached.reached_the_end(), "a walk that failed did not reach the end");
    let taken: Vec<(String, SystemTime)> = the_names_the_work_takes(reached);
    assert_eq!(taken.len(), 2, "a caller that only works reads the names the walk reached, not an empty queue: {taken:?}");
}

/// A directory that cannot be opened at all is a walk that reached no name,
/// and it says so rather than reading as an empty directory.
#[test]
fn a_walk_that_could_not_be_opened_reached_no_name_and_says_so() {
    let root: PathBuf = crate::test_support::fixture("walk-that-could-not-be-opened");
    let work_dir: PathBuf = root.join("work");
    std::fs::create_dir_all(work_dir.as_path()).expect("the work directory");
    let reached: WhatTheWalkReached = the_names_in(work_dir.as_path(), PENDING);
    crate::test_support::remove(root.as_path());
    assert!(reached.names.is_empty(), "nothing was there to reach: {:?}", reached.names);
    assert!(!reached.reached_the_end(), "a directory that is not there is not an empty one");
}
