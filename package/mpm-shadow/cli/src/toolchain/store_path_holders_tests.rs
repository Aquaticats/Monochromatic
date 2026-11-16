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
use std::time::{Duration, Instant, SystemTime};

/// Imports `HoldKind` and the scan under test.
use crate::toolchain::store_path_holders::{HoldKind, held_reason, holders_of, mapped_path, opaque_holder_reason};
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
    assert_eq!(after.incomplete, None, "{after:?}");
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

/// A hidden process that started after a version was installed is a reason to keep that version,
/// whatever it calls itself, and the line names it and says the process may run it.
/// Why:
/// The kernel will not say which file it runs or where it works, so its name is no evidence either way:
/// a miss was never a clearance.
/// The owner's age rule of 2026-11-15 is what decides between a process that may have re-executed itself into the
/// version and one that started before it was installed.
#[test]
fn names_a_hidden_process_that_may_run_a_tool() {
    let installed: SystemTime = SystemTime::now();
    let later: SystemTime = installed + Duration::from_secs(60);
    let mut holders = crate::toolchain::store_path_holders::StoreHolders::default();
    assert!(opaque_holder_reason(&holders, Some(installed)).is_none(), "nothing hidden keeps nothing");
    holders.opaque.push(crate::toolchain::store_path_holders::OpaqueProcess {
        pid: 4242,
        comm: "git_policy_cli-".to_string(),
        started: Some(later),
    });

    let reason: String = opaque_holder_reason(&holders, Some(installed)).expect("a hidden process started after it keeps it");
    assert!(reason.contains("4242"), "the line names the process: {reason}");
    assert!(reason.contains("may run"), "the line says the version may be run: {reason}");
    assert!(reason.contains("after"), "the line says why the version is kept: {reason}");

    holders.opaque.push(crate::toolchain::store_path_holders::OpaqueProcess {
        pid: 4243,
        comm: "x".to_string(),
        started: Some(later),
    });
    let two: String = opaque_holder_reason(&holders, Some(installed)).expect("two hidden processes keep the versions");
    assert!(two.contains("and 1 more"), "the line counts the others: {two}");

    // A hidden process that started before the version was installed keeps nothing.
    let mut earlier = crate::toolchain::store_path_holders::StoreHolders::default();
    earlier.opaque.push(crate::toolchain::store_path_holders::OpaqueProcess {
        pid: 4244,
        comm: "x".to_string(),
        started: Some(installed - Duration::from_secs(60)),
    });
    assert!(
        opaque_holder_reason(&earlier, Some(installed)).is_none(),
        "a hidden process that started before the version kept it"
    );

    // A hidden process whose start cannot be read keeps the version, and the line says so.
    let mut unknown = crate::toolchain::store_path_holders::StoreHolders::default();
    unknown.opaque.push(crate::toolchain::store_path_holders::OpaqueProcess {
        pid: 4245,
        comm: "x".to_string(),
        started: None,
    });
    let reason_unknown: String =
        opaque_holder_reason(&unknown, Some(installed)).expect("an unreadable start keeps the version");
    assert!(reason_unknown.contains("cannot be read"), "the line says the start is unknown: {reason_unknown}");
}

/// The words a hold is printed with say what the process does with the path.
#[test]
fn words_every_hold() {
    assert_eq!(HoldKind::Runs.words(), "runs");
    assert_eq!(HoldKind::WorksInside.words(), "works inside");
    assert_eq!(HoldKind::HasOpen.words(), "has open");
}

/// What:
/// A process table of a PID namespace is a table the scan says it cannot trust, and one that sees the whole machine is
/// not.
/// Why:
/// The review of 82531265a7 ran the same install in the machine's namespace (the version was kept) and inside
/// `unshare --pid` (the version was removed while its holder still ran): `/proc` there lists only that namespace.
#[test]
fn a_nested_pid_namespace_is_a_table_the_scan_cannot_trust() {
    let machine: &str = "Name:\tmpm-shadow\nNSpid:\t4242\nPid:\t4242\n";
    let nested: &str = "Name:\tmpm-shadow\nNSpid:\t4242\t7\nPid:\t7\n";
    let without: &str = "Name:\tmpm-shadow\nPid:\t4242\n";

    assert!(!crate::toolchain::store_path_holders::in_a_pid_namespace(machine), "the machine's own namespace is the whole machine");
    assert!(crate::toolchain::store_path_holders::in_a_pid_namespace(nested), "a nested PID namespace holds only its own processes");
    assert!(!crate::toolchain::store_path_holders::in_a_pid_namespace(without), "a kernel that prints no NSpid line is this namespace");
}
