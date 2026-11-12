//! The order a lean run takes its mechanisms in:
//! the cheap ones first and the units pass last,
//! so a run that reaches its deadline cuts the pass that can afford to wait
//! instead of the passes behind it.
//! Why:
//! Measured in the first run (2026-11-07T23:52:28-05:00 to 2026-11-08T01:24:39-05:00,
//! `doc/handover/cargo-shadow-lean-build-folders.md`):
//! records and toolchains took 2.5 minutes together,
//! the Podman pass 20.5 minutes,
//! the idle-trees pass 29 minutes,
//! and the units pass was stopped 40 minutes in after about 700 of several thousand profile directories,
//! which in the plan's order starved the Podman pass of the whole three-hour deadline.

use std::{collections::HashSet, fs, path::PathBuf, time::Duration};

use clap::ValueEnum;

use crate::cli::MechanismName;
use crate::lean_command::mechanism_of;
use crate::lean_guard::GuardEnvironment;
use crate::lean_identity::{Deadline, now_seconds};
use crate::lean_progress::{LongPass, Progress};
use crate::lean_run::{Mechanism, MechanismReport, RUN_ORDER, RunContext, RunOptions, long_pass, run_mechanism, run_order};
use crate::lean_run_trees::TreeTally;
use crate::lean_test_support::{DAY, age_everything, make_tree};
use crate::test_scratch::scratch;

#[test]
fn a_run_starts_with_the_cheap_mechanisms_and_ends_with_the_units_pass() {
    assert_eq!(run_order(&[]), vec![Mechanism::Records, Mechanism::Toolchains, Mechanism::Podman, Mechanism::Trees, Mechanism::Units]);
}

#[test]
fn a_run_of_a_subset_keeps_that_order() {
    assert_eq!(run_order(&[Mechanism::Units, Mechanism::Records]), vec![Mechanism::Records, Mechanism::Units]);
}

#[test]
fn every_mechanism_the_command_line_names_is_in_the_order() {
    let named: Vec<Mechanism> = MechanismName::value_variants().iter().map(|name| { return mechanism_of(*name); }).collect();
    assert_eq!(named.len(), RUN_ORDER.len(), "the command line names {named:?}");
    for mechanism in named {
        assert!(RUN_ORDER.contains(&mechanism), "{mechanism:?} is missing from the run order");
    }
}

#[test]
fn a_partly_removed_tree_is_counted_apart_from_a_removed_one_and_a_skipped_one() {
    let tally: TreeTally = TreeTally { judged: 3, given: 10, removed: 1, bytes: 1 << 30, partly: 1, partly_bytes: 1 << 29, kept: 1, kept_in_use: 1, skipped: 0, ..TreeTally::default() };
    let line: String = tally.line("removed");
    assert!(line.contains("3 of 10 trees judged"), "{line}");
    assert!(line.contains("removed 1 (1.00 GiB by disk blocks)"), "{line}");
    assert!(line.contains("partly removed 1 (0.50 GiB by disk blocks freed, what stayed is named)"), "{line}");
    assert!(line.contains("kept 1; skipped 0"), "{line}");
    assert!(!line.contains("3 trees read"), "the count is what the pass reached, not what the base holds: {line}");
}

#[test]
fn the_two_long_passes_keep_a_position_and_the_short_mechanisms_do_not() {
    assert!(long_pass(Mechanism::Trees).is_some());
    assert!(long_pass(Mechanism::Units).is_some());
    for short in [Mechanism::Records, Mechanism::Toolchains, Mechanism::Podman] {
        assert!(long_pass(short).is_none(), "{short:?}");
    }
}

/// A run of one mechanism over a fixture base.
/// Why the trees here have no ledger:
/// a tree whose ledger names a workspace that is gone is a candidate,
/// and the candidates are judged from the front on every pass,
/// so the recorded position only orders the trees a pass reaches by walking the whole list.
/// A tree with no ledger and no dependency information is judged on its age alone,
/// which is what the position orders.
#[test]
fn a_trees_pass_continues_after_the_recorded_tree_and_starts_at_the_front_without_one() {
    let root = scratch();
    let base: PathBuf = root.path().join("base");
    let first: PathBuf = make_tree(&base, "ab/00000000000001", None);
    let second: PathBuf = make_tree(&base, "ab/00000000000002", None);
    let third: PathBuf = make_tree(&base, "ab/00000000000003", None);
    for tree in [&first, &second, &third] {
        age_everything(tree, now_seconds() - 30 * DAY);
    }
    let options: RunOptions = RunOptions { dry_run: false, only: vec![Mechanism::Trees], deadline: Duration::from_secs(120), start: root.path().to_path_buf() };
    let environment: GuardEnvironment = GuardEnvironment::real();
    let pinned: HashSet<String> = HashSet::new();
    let checkouts: Vec<PathBuf> = vec![root.path().to_path_buf()];
    let mut progress: Progress = Progress::default();
    progress.set(LongPass::Trees, Some(first.clone()));
    let resumed: MechanismReport = {
        let context = RunContext { options: &options, deadline: Deadline::after(options.deadline), environment: &environment, checkouts: &checkouts, pinned: &pinned, base: Some(&base), rustup_home: root.path(), progress: &progress };
        run_mechanism(Mechanism::Trees, &context)
    };
    assert!(second.exists() == false && third.exists() == false, "the trees after the recorded one went: {resumed:?}");
    assert!(first.exists(), "the tree the last run stopped at is not judged again by this one");
    assert!(resumed.lines.iter().any(|line| { return line.contains("this pass started after") && line.contains(&first.display().to_string()); }), "{:?}", resumed.lines);
    assert_eq!(resumed.resume_after, None, "a pass that finished its list leaves no position behind");
    let front: MechanismReport = {
        let context = RunContext { options: &options, deadline: Deadline::after(options.deadline), environment: &environment, checkouts: &checkouts, pinned: &pinned, base: Some(&base), rustup_home: root.path(), progress: &Progress::default() };
        run_mechanism(Mechanism::Trees, &context)
    };
    assert!(!first.exists(), "without a record the pass starts at the front: {front:?}");
    assert!(!front.lines.iter().any(|line| { return line.contains("this pass started after"); }), "{:?}", front.lines);
}

/// The units pass continues after the profile directory the last run stopped at,
/// keeps its position when the deadline cuts it before its first profile,
/// and leaves no position behind once it finishes the list.
#[test]
fn the_units_pass_continues_after_the_recorded_profile_directory() {
    let root = scratch();
    let base: PathBuf = root.path().join("base");
    let earlier: PathBuf = make_tree(&base, "ab/00000000000004", None);
    let later: PathBuf = make_tree(&base, "ab/00000000000005", None);
    let earlier_profile: PathBuf = earlier.join("debug");
    let later_profile: PathBuf = later.join("debug");
    for tree in [&earlier, &later] {
        age_everything(tree, now_seconds() - 30 * DAY);
    }
    let options: RunOptions = RunOptions { dry_run: false, only: vec![Mechanism::Units], deadline: Duration::from_secs(120), start: root.path().to_path_buf() };
    let environment: GuardEnvironment = GuardEnvironment::real();
    let pinned: HashSet<String> = HashSet::new();
    let checkouts: Vec<PathBuf> = Vec::new();
    let run_with = |after: Option<PathBuf>, deadline: Duration| -> MechanismReport {
        let mut progress: Progress = Progress::default();
        progress.set(LongPass::Units, after);
        let context = RunContext { options: &options, deadline: Deadline::after(deadline), environment: &environment, checkouts: &checkouts, pinned: &pinned, base: Some(&base), rustup_home: root.path(), progress: &progress };
        return run_mechanism(Mechanism::Units, &context);
    };
    // The deadline is already gone, so even the listing does not finish:
    // nothing is judged and the position it started at is kept.
    let cut: MechanismReport = run_with(Some(earlier_profile.clone()), Duration::from_secs(0));
    assert!(cut.lines.iter().any(|line| { return line.contains("the deadline passed while listing the profile directories"); }), "{:?}", cut.lines);
    assert_eq!(cut.resume_after, Some(earlier_profile), "a pass cut before its first profile keeps the position it started at");
    // The record already names the last profile, so there is nothing left and the next run starts at the front.
    let exhausted: MechanismReport = run_with(Some(later_profile), Duration::from_secs(120));
    assert!(exhausted.lines.iter().any(|line| { return line.contains("0 of 0 profile directories read"); }), "{:?}", exhausted.lines);
    assert_eq!(exhausted.resume_after, None);
    // No record: the whole list, and no position left behind.
    let whole: MechanismReport = run_with(None, Duration::from_secs(120));
    assert!(whole.lines.iter().any(|line| { return line.contains("2 of 2 profile directories read"); }), "{:?}", whole.lines);
    assert_eq!(whole.resume_after, None, "the pass finished the list it was given");
}

/// A resumed trees pass still holds every workspace the base records as a guard,
/// including the ones recorded by the trees before the position it starts at.
/// Why:
/// A tree is removed only when no other tree's workspace lies inside it,
/// and the trees a resumed pass starts after are still other trees:
/// losing their records with the truncated list let the pass delete a tree that holds live sources.
#[test]
fn a_resumed_trees_pass_keeps_a_tree_that_holds_a_workspace_recorded_before_its_position() {
    let root = scratch();
    let base: PathBuf = root.path().join("base");
    let gone: PathBuf = root.path().join("no-longer-here");
    // The tree that sorts second was built for a workspace that is gone, and another tree's live workspace lies inside it.
    let later: PathBuf = make_tree(&base, "ab/00000000000002", Some(&gone));
    let live: PathBuf = later.join("live-workspace");
    fs::create_dir_all(&live).expect("the live workspace");
    let important: PathBuf = live.join("important.txt");
    fs::write(&important, b"live user source\n").expect("the live workspace's file");
    let earlier: PathBuf = make_tree(&base, "aa/00000000000001", Some(&live));
    for tree in [&earlier, &later] {
        age_everything(tree, now_seconds() - 30 * DAY);
    }
    let options: RunOptions = RunOptions { dry_run: false, only: vec![Mechanism::Trees], deadline: Duration::from_secs(120), start: root.path().to_path_buf() };
    let environment: GuardEnvironment = GuardEnvironment::real();
    let pinned: HashSet<String> = HashSet::new();
    let checkouts: Vec<PathBuf> = vec![root.path().to_path_buf()];
    let run_with = |after: Option<PathBuf>| -> MechanismReport {
        let mut progress: Progress = Progress::default();
        progress.set(LongPass::Trees, after);
        let context = RunContext { options: &options, deadline: Deadline::after(options.deadline), environment: &environment, checkouts: &checkouts, pinned: &pinned, base: Some(&base), rustup_home: root.path(), progress: &progress };
        return run_mechanism(Mechanism::Trees, &context);
    };
    // The positive control: from the front, the base's records keep both trees.
    let whole: MechanismReport = run_with(None);
    assert!(whole.lines.iter().any(|line| { return line.contains("kept 2"); }), "the guard must keep the tree the live workspace lies inside: {:?}", whole.lines);
    assert!(important.exists(), "the live workspace's file survived a pass from the front: {:?}", whole.lines);
    // The same base, resumed after the tree that records the live workspace.
    let resumed: MechanismReport = run_with(Some(earlier.clone()));
    assert!(resumed.lines.iter().any(|line| { return line.contains("this pass started after"); }), "{:?}", resumed.lines);
    assert!(resumed.lines.iter().any(|line| { return line.contains("kept 1"); }), "the tree the earlier record guards is kept, not removed: {:?}", resumed.lines);
    assert!(later.exists(), "a resumed pass keeps a tree another tree's workspace lies inside: {:?}", resumed.lines);
    assert!(important.exists(), "the live workspace's file is user data and must survive a resumed pass: {:?}", resumed.lines);
}

/// A pass that removes nothing says why it removed nothing.
/// Why:
/// "free space falling and the cleanup freed nothing" is the question the journal has to answer without a person
/// reading the proof lines of every kept tree:
/// the two reasons a pass keeps a whole base are that something still uses the trees, or that they are too young.
#[test]
fn a_pass_that_frees_nothing_says_whether_the_trees_are_in_use_or_too_young() {
    let root = scratch();
    let base: PathBuf = root.path().join("base");
    let alive: PathBuf = root.path().join("alive");
    fs::create_dir_all(&alive).expect("the workspace");
    let in_use: PathBuf = make_tree(&base, "aa/00000000000001", Some(&alive));
    let too_young: PathBuf = make_tree(&base, "ab/00000000000002", Some(&root.path().join("gone")));
    age_everything(&in_use, now_seconds() - 90 * DAY);
    age_everything(&too_young, now_seconds() - DAY);
    let options: RunOptions = RunOptions { dry_run: true, only: vec![Mechanism::Trees], deadline: Duration::from_secs(120), start: root.path().to_path_buf() };
    let environment: GuardEnvironment = GuardEnvironment::real();
    let pinned: HashSet<String> = HashSet::new();
    let checkouts: Vec<PathBuf> = vec![root.path().to_path_buf()];
    let report: MechanismReport = {
        let context = RunContext { options: &options, deadline: Deadline::after(options.deadline), environment: &environment, checkouts: &checkouts, pinned: &pinned, base: Some(&base), rustup_home: root.path(), progress: &Progress::default() };
        run_mechanism(Mechanism::Trees, &context)
    };
    let reported: String = report.lines.join("\n");
    assert!(reported.contains("kept 2"), "{reported}");
    assert!(reported.contains("nothing was found to remove"), "{reported}");
    assert!(reported.contains("1 trees were kept because something that uses them is still there and 1 because they have not been left alone long enough"), "{reported}");
}
