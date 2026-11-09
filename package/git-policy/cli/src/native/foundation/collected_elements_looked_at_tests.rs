//! Controls for the counting seam itself.

use super::*;
use std::collections::HashSet;

/// Items named `item-{number}`.
fn items(count: usize) -> Vec<String> {
    let mut built: Vec<String> = Vec::new();
    for number in 0..count {
        built.push(format!("item-{number}"));
    }
    return built;
}

/// What:
///  The count tells one pass from the scan it replaces over the same items.
/// Why:
///   The controls of every pass that uses this seam assert a count,
///  so the seam itself needs the positive control `AGENTS.md` `QPC` asks for:
///  an input that must move.
///  A set answers each question by looking at one collected element;
///  `Vec::contains` looks at every element collected before the candidate,
///  which is the count the same size of input reaches when a scan runs.
#[test]
fn the_count_separates_one_pass_from_a_scan() {
    let size: usize = 64;
    let given: Vec<String> = items(size);
    let before_pass: u64 = looked_at();
    let mut seen: HashSet<&String> = HashSet::new();
    let mut once: Vec<&String> = Vec::new();
    for item in &given {
        // The set answers by naming the one element the candidate is compared with.
        note_looked_at(1);
        if seen.insert(item) {
            once.push(item);
        }
    }
    let pass: u64 = looked_at() - before_pass;
    let before_scan: u64 = looked_at();
    let mut scanned: Vec<&String> = Vec::new();
    for item in &given {
        // A scan answers by comparing the candidate with every element collected before it.
        note_looked_at(scanned.len() as u64);
        if !scanned.contains(&item) {
            scanned.push(item);
        }
    }
    let scan: u64 = looked_at() - before_scan;
    assert_eq!(once, scanned, "both answers collect every item once, in order");
    assert_eq!(pass, size as u64, "a set looks at one collected element per question");
    assert_eq!(
        scan,
        (size * (size - 1) / 2) as u64,
        "a scan looks at every collected element before the candidate"
    );
    assert!(scan > pass, "{scan} elements looked at by a scan against {pass} by one pass");
}

/// Each thread counts its own work,
/// so two controls running beside each other cannot read each other's numbers.
#[test]
fn another_thread_counts_its_own_work() {
    let here: u64 = looked_at();
    note_looked_at(3);
    let there: Vec<std::thread::Result<u64>> = crate::side_by_side_threads::run_all_side_by_side(vec![()], count_and_report);
    assert_eq!(there.len(), 1, "one job ran");
    let mut results = there.into_iter();
    let other: u64 = results.next().expect("one result").expect("no panic");
    assert_eq!(other, 5, "the other thread counted only what it looked at");
    assert_eq!(looked_at(), here + 3, "this thread counted only what it looked at");
}

/// Look at five collected elements on the calling thread and report this thread's count.
fn count_and_report(_state: ()) -> u64 {
    note_looked_at(5);
    return looked_at();
}
