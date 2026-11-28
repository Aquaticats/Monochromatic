//! What:
//!  Plan step 22's sweep cost past 64 clones in the throwaway guest:
//!  the run-to-run spread over the same unmounted clones first,
//!  then the sweep at 16,
//!  64,
//!  128,
//!  256 and 512 clones,
//!  each count beside the record's own largest count in the same round.
//! Why:
//!  The record of 2026-11-25 measured the sweep at 1,
//!  16 and 64 clones
//!  (`doc/planning/cli-git-zfs-step-22-guest-half-measurements.md`)
//!  and its proposal for the handler's heartbeat period says
//!  "A parent that holds more clones than 64 would need the period raised with
//!  the sweep's measured curve",
//!  with nothing measuring a larger parent.
//!  So this arm walks the whole line past 64,
//!  alternating the direction of the counts between rounds,
//!  and prints every round's own time so a ratio is read against the 64-clone
//!  sweep of that round rather than against another round's
//!  (rule `TVC`),
//!  after the spread of the same count on unchanged input says which
//!  differences are noise (rule `QNB`).
//!  Every count asserts that the sweep attached every clone it was given,
//!  so a run that timed a sweep that did nothing fails instead of reporting a
//!  fast line.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const spread = measureTheSpread(fixture);           // five sweeps, 64 clones, same state
//! for (const round of [0, 1, 2]) measureEveryCount(fixture, round); // 64 is each round's control
//! ```

#![cfg(target_os = "linux")]

use std::collections::BTreeMap;

use super::zfs_step_22_measurement::{
    Fixture,
    middle,
    mounted_at_path,
    unmount_quiet,
};
use super::zfs_sweep_guest::{
    Run,
    sweep,
};

/// The clone counts one round visits,
/// smallest first.
/// Why:
///  Sixteen and sixty-four are the record's own counts,
///  so each round's ratios have the same control the record used,
///  and 128,
///  256 and 512 are two doublings and three past it.
const PARENT_COUNTS: &[usize] = &[16, 64, 128, 256, 512];

/// The count the spread probe and each round's control are read at,
/// which is the record's own largest count.
const CONTROL_COUNT: usize = 64;

/// How many sweeps the spread probe runs over the same unmounted clones.
const SPREAD_RUNS: usize = 5;

/// How many rounds visit every count.
const PARENT_ROUNDS: usize = 3;

/// What:
///  The spread first,
///  then every round's counts,
///  and the clones taken away again.
/// Why:
///  The spread is measured before any claim is made,
///  and each round's 64-clone sweep is the control its larger counts are read
///  against.
pub(super) fn large_parent_sweep_arm(fixture: &Fixture) {
    let (spread, standing): (Vec<u128>, usize) = measure_the_spread(fixture);
    let mut present: usize = standing;
    let mut times: BTreeMap<usize, Vec<u128>> = BTreeMap::new();
    for round in 0..PARENT_ROUNDS {
        let order: Vec<usize> = round_order(round);
        for count in order {
            grow_to(fixture, &mut present, count);
            let elapsed: u128 = one_sweep(fixture, count);
            println!("STEP22 parent-sweep round={round} clones={count} ms={elapsed}");
            times.entry(count).or_default().push(elapsed);
        }
    }
    grow_to(fixture, &mut present, 0);
    for (count, list) in &times {
        println!(
            "STEP22 parent-sweep times clones={count} list={list:?} mid={}",
            middle(list.as_slice())
        );
    }
    println!(
        "STEP22 parent-sweep spread runs={SPREAD_RUNS} clones={CONTROL_COUNT} list={spread:?} mid={}",
        middle(spread.as_slice())
    );
    println!(
        "STEP22 parent-sweep done clones={} rounds={PARENT_ROUNDS} control={CONTROL_COUNT}",
        PARENT_COUNTS
            .last()
            .expect("the arm sweeps at least one count")
    );
}

/// The counts one round visits,
/// in the order it visits them.
/// Why:
///  Ascending rounds build the line and descending rounds walk it back,
///  so a cost that drifts with the arm's own age shows as a difference between
///  the rounds at the same count rather than as the shape of the curve.
fn round_order(round: usize) -> Vec<usize> {
    if round % 2 == 0 {
        return PARENT_COUNTS.to_vec();
    }
    return PARENT_COUNTS.iter().rev().copied().collect();
}

/// What:
///  Create the control count's clones,
///  sweep them `SPREAD_RUNS` times with the mounts taken down before each run,
///  and answer the times and the clones left standing.
/// Why:
///  Rule `QNB` asks for the run-to-run spread on unchanged input before any
///  difference is read as a finding,
///  and taking every mount down again makes each run's input the same one.
fn measure_the_spread(fixture: &Fixture) -> (Vec<u128>, usize) {
    let mut present: usize = 0;
    grow_to(fixture, &mut present, CONTROL_COUNT);
    let mut times: Vec<u128> = Vec::new();
    for run in 0..SPREAD_RUNS {
        let elapsed: u128 = one_sweep(fixture, CONTROL_COUNT);
        println!("STEP22 parent-sweep spread run={run} clones={CONTROL_COUNT} ms={elapsed}");
        times.push(elapsed);
    }
    return (times, present);
}

/// What:
///  Take every clone's mount down,
///  run the sweep once,
///  and answer how long it took,
///  refusing a sweep that attached fewer clones than it was given.
/// Why:
///  The sweep's cost is what it pays to walk the parent and mount each clone,
///  so a clone already mounted would make the reading a cheaper one than the
///  line claims.
fn one_sweep(
    fixture: &Fixture,
    count: usize,
) -> u128 {
    for index in 0..count {
        unmount_quiet(fixture.slot(parent_name(index).as_str()).as_path());
    }
    let run: Run = sweep();
    assert_eq!(
        run.status,
        Some(0),
        "the {count}-clone sweep reached its end: {}",
        run.said
    );
    let mut attached: usize = 0;
    for index in 0..count {
        if mounted_at_path(fixture.slot(parent_name(index).as_str()).as_path()) {
            attached += 1;
        }
    }
    assert_eq!(
        attached, count,
        "the {count}-clone sweep attached every clone it was given"
    );
    return run.elapsed.as_millis();
}

/// What:
///  Create or take away the arm's own clones until exactly `count` of them
///  stand.
/// Why:
///  The sweep walks every clone of the checkout's snapshot asking for its
///  mount,
///  so a count is measured by holding exactly that many clones rather than by
///  turning others' requests off.
fn grow_to(
    fixture: &Fixture,
    present: &mut usize,
    count: usize,
) {
    while *present < count {
        fixture.make_a_clone_asking(
            parent_name(*present).as_str(),
            parent_seed(*present).as_str(),
        );
        *present += 1;
    }
    while *present > count {
        *present -= 1;
        fixture.forget(
            parent_name(*present).as_str(),
            parent_seed(*present).as_str(),
        );
    }
}

/// The slot and clone name of one of the arm's clones.
fn parent_name(index: usize) -> String {
    return format!("parent-{index}");
}

/// What:
///  The seed of one of the arm's clones,
///  which keeps its origin snapshot's name its own and keeps the mount intent inside the grammar the sweep reads.
/// Why:
///  The intent is `v1:<32 lowercase hex digits>:mount`,
///  so a seed that is not hex makes every clone's intent a refusal and the sweep attaches nothing:
///  a first run of this arm seeded its clones with `p000` and the 64-clone sweep attached none of them.
fn parent_seed(index: usize) -> String {
    return format!("{index:03x}");
}
