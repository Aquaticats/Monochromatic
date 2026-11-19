//! What:
//! Unit tests of the read of the process table in `store_path_holders.rs`:
//! which store path a `/proc/<pid>/maps` line names,
//! that a process of this test which holds a file of a version is found,
//! and that a path outside the store is never a holder.
//! Why:
//! The cleanup keeps a version because this scan names it,
//! so a scan that found nothing would remove a version in use,
//! and a scan that matched too much would remove nothing at all.

/// Imports `PathBuf` from `std::path`.
use std::path::PathBuf;
/// Imports `Duration` and `Instant` from `std::time`.
use std::time::{Duration, Instant};

/// Imports `HoldKind` and the scan under test.
use crate::toolchain::store_path_holders::{HoldKind, OpaqueProcess, held_reason, holders_of, mapped_path, undescribed_reason};
use crate::test_scratch::scratch;

/// A deadline far enough away that the scan completes.
fn later() -> Instant {
    return Instant::now() + Duration::from_secs(120);
}

/// The path a mapping line names is the rest after the five fields it opens with,
/// with the mark of an unlinked file removed.
#[test]
fn reads_the_path_of_a_mapping_line() {
    assert_eq!(
        mapped_path("7f2a0c000000-7f2a0c021000 r--p 00000000 00:2f 1234567 /store/node/26.13.1/bin/node"),
        Some("/store/node/26.13.1/bin/node")
    );
    assert_eq!(
        mapped_path("7f2a0c000000-7f2a0c021000 r--p 00000000 00:2f 1234567 /store/a file with spaces/lib.so (deleted)"),
        Some("/store/a file with spaces/lib.so")
    );
    assert_eq!(mapped_path("7f2a0c000000-7f2a0c021000 rw-p 00000000 00:00 0"), None);
    assert_eq!(mapped_path("not a mapping line"), None);
}

/// The scan finds the file this test holds open,
/// and names the process that holds it.
#[test]
fn finds_the_file_this_test_holds_open() {
    let directory = scratch();
    let store: PathBuf = directory.path().join("toolchain");
    let version: PathBuf = store.join("node/26.13.1");
    std::fs::create_dir_all(&version).expect("make a version directory");
    std::fs::write(version.join("program"), b"#!/bin/true\n").expect("write the program");
    // Control: before the file is open, the scan finds no holder, so it can distinguish the two states.
    let before = holders_of(std::slice::from_ref(&store), later());
    assert!(held_reason(&before, &version).is_none(), "{before:?}");

    let opened = std::fs::File::open(version.join("program")).expect("open the program");
    let after = holders_of(std::slice::from_ref(&store), later());

    let reason: String = held_reason(&after, &version).expect("the scan must find the open file");
    assert!(reason.contains(&format!("pid {}", std::process::id())), "{reason}");
    assert!(reason.contains("has open"), "{reason}");
    // An opaque process does not mean the enumeration itself failed.
    assert!(after.incomplete.is_none(), "{after:?}");
    assert!(after.processes > 0);
    drop(opened);
}

/// A path under the store that no process holds is not a reason to keep a version.
#[test]
fn finds_no_holder_of_a_version_nothing_uses() {
    let directory = scratch();
    let store: PathBuf = directory.path().join("toolchain");
    let version: PathBuf = store.join("node/26.12.0");
    std::fs::create_dir_all(&version).expect("make a version directory");
    std::fs::write(version.join("program"), b"#!/bin/true\n").expect("write the program");

    let holders = holders_of(&[store], later());

    assert!(held_reason(&holders, &version).is_none(), "{holders:?}");
}

/// A store outside every root the scan was given holds nothing for it.
#[test]
fn ignores_a_path_outside_the_store() {
    let directory = scratch();
    let store: PathBuf = directory.path().join("toolchain");
    let elsewhere: PathBuf = directory.path().join("somewhere-else");
    std::fs::create_dir_all(&store).expect("make the store");
    std::fs::create_dir_all(&elsewhere).expect("make the other directory");
    std::fs::write(elsewhere.join("program"), b"#!/bin/true\n").expect("write a program");
    let opened = std::fs::File::open(elsewhere.join("program")).expect("open it");

    let holders = holders_of(&[store], later());

    assert!(holders.held.is_empty(), "{holders:?}");
    drop(opened);
}

/// A scan whose time limit is already gone reports that it could not tell,
/// so the cleanup keeps every version instead of assuming none is in use.
#[test]
fn reports_a_scan_that_ran_out_of_time() {
    let holders = holders_of(&[PathBuf::from("/nonexistent-store")], Instant::now() - Duration::from_secs(1));

    assert!(holders.incomplete.is_some(), "{holders:?}");
}

/// A process the kernel refuses to describe makes the scan incomplete, whatever its name,
/// and the reason names every such process so a person can end it or decide.
#[test]
fn calls_the_scan_incomplete_when_a_process_cannot_be_described() {
    let opaque: Vec<OpaqueProcess> = vec![
        OpaqueProcess { pid: 4242, comm: "fd".to_string(), started_before: None },
        OpaqueProcess { pid: 4243, comm: "keepassxc".to_string(), started_before: None },
    ];

    let reason: String = undescribed_reason(&opaque).expect("two processes cannot be described");

    assert!(reason.contains("2 process(es)"), "{reason}");
    assert!(reason.contains("pid 4242 (fd)"), "a short name is named like any other: {reason}");
    assert!(reason.contains("pid 4243 (keepassxc)"), "{reason}");
    assert!(undescribed_reason(&[]).is_none(), "a scan that described every process is complete");
}

/// Checks the real proc format and a command name containing spaces and a closing parenthesis.
#[cfg(target_os = "linux")]
#[test]
fn reads_process_start_ticks_after_the_parenthesized_command() {
    let own_row = std::fs::read_to_string("/proc/self/stat").expect("read this test's proc row");
    // A real process has a start time in this boot's ticks.
    assert!(super::store_path_holders::process_start_ticks(&own_row).is_some());
    // Use each field's number as its value, so a parser shifted by one would report a different value.
    let numeric_fields = (4_u64..=23).map(|field| return field.to_string()).collect::<Vec<_>>().join(" ");
    let unusual_command = format!("4242 (name) with space) S {numeric_fields}");
    assert_eq!(super::store_path_holders::process_start_ticks(&unusual_command), Some(22));
    assert_eq!(super::store_path_holders::process_start_ticks("4242 (truncated) S 0"), None);
}

/// The words a hold is printed with say what the process does with the path.
#[test]
fn words_every_hold() {
    assert_eq!(HoldKind::Runs.words(), "runs");
    assert_eq!(HoldKind::WorksInside.words(), "works inside");
    assert_eq!(HoldKind::HasOpen.words(), "has open");
}
