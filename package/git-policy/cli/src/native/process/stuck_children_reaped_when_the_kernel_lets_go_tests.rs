//! Controls for the children a command stopped waiting for.

use super::*;
use std::process::{Command, Stdio};

/// A child of a shell command line whose streams go nowhere.
fn quiet(line: &str) -> Child {
    let mut command = Command::new("/bin/sh");
    command.arg("-c").arg(line).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
    return command.spawn().expect("the stand-in starts");
}

/// What:
///  A child handed to a reaper is collected when it ends,
/// and the command does not wait for it.
/// Why:
///  A child in an uninterruptible call cannot be ended,
/// so the command stops waiting and a thread of its own collects the
///  end whenever the kernel releases it.
#[test]
fn a_child_kept_for_a_reaper_is_collected_when_it_ends() {
    let child: Child = quiet("sleep 0.2");
    let pid: u32 = child.id();
    let began = std::time::Instant::now();
    reap_soon(child);
    // The call returns at once: the command never waits for the child here.
    assert!(began.elapsed() < std::time::Duration::from_millis(100), "reap_soon waited for the child: {:?}", began.elapsed());
    // The record is gone once the reaper has collected the child, and no zombie of that number is left.
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while std::fs::read_to_string(format!("/proc/{pid}/stat")).is_ok() && std::time::Instant::now() < deadline {
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert!(!std::fs::read_to_string(format!("/proc/{pid}/stat")).is_ok(), "the child handed to a reaper was never collected");
}

/// What:
///  A child whose reaper could not start is kept, not lost,
/// and the next reaper that runs collects it.
/// Why:
///  The standard library's spawn consumes the closure,
/// so a refused start would drop the child and leave it a zombie for
///  as long as this process lives;
/// the list of children waiting for a reaper is what keeps it.
#[test]
fn a_child_whose_reaper_could_not_start_is_kept_and_collected_later() {
    let child: Child = quiet("exit 0");
    let pid: u32 = child.id();
    keep(child);
    assert_eq!(waiting_count(), 1, "the child a refused reaper would have lost is not kept");
    drain_the_registry();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while std::fs::read_to_string(format!("/proc/{pid}/stat")).is_ok() && std::time::Instant::now() < deadline {
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert_eq!(waiting_count(), 0, "the kept child was not taken by the reaper");
    assert!(!std::fs::read_to_string(format!("/proc/{pid}/stat")).is_ok(), "the kept child was never collected");
}
