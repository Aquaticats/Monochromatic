//! What:
//! Round 13 tests: what a changed process number in procfs does and does not show (12a), and the false greens of part 12b.
//! Why:
//! The review of round 12 (`cargo-shadow-sol-12a.txt`, `-12b.txt`) named schedules inside existing rows of the acceptance list.
//! The owner's answer of 2026-11-04 ("Out of scope, keep cheap guards") puts a program that deliberately changes mounts or `/proc`
//! while a run or a recovery is in progress outside the contract (assumption a), so the remount schedules of 12a are not tested here.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // describe('round thirteen', () => { it('a changed positive number is unknown', ...) })
//! ```

use std::fs;
use std::process::{Command, Stdio};

use rustix::process::{Pid, PidfdFlags, pidfd_open};

use crate::control_group::{StartOf, missing_entry_verdict};

/// Acceptance rows A1, V2 (12a, the missing-entry inference):
/// after a failed lookup of a process's directory, only the value that the kernel prints for an ended, reaped process (`-1`) is gone;
/// the same number is a hidden process, and any other number (another positive id, `0`) is unknown, never gone.
#[test]
fn only_the_ended_process_number_shows_a_gone_process() {
    let pid: i32 = 4242;
    assert_eq!(missing_entry_verdict(-1, pid, "r"), StartOf::Gone, "the sentinel of an ended process was not read as gone");
    for (label, number) in [("the same number", i64::from(pid)), ("another positive id", i64::from(pid) + 1), ("a process this procfs does not show", 0), ("another negative number", -2)] {
        assert!(matches!(missing_entry_verdict(number, pid, "r"), StartOf::Unknown(_)), "{label}: read as gone");
    }
}

/// The kernel's own statement, which the rule above rests on (measured here, not recalled):
/// the `Pid:` line of the fdinfo of a pidfd is this process's number for the target while the target exists (a zombie that is not
/// reaped yet included), and `-1` once the target has ended and been reaped.
#[test]
fn the_fdinfo_of_a_pidfd_says_minus_one_only_for_a_reaped_process() {
    let mut child = Command::new("true").stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn().expect("starts");
    let id: i32 = i32::try_from(child.id()).expect("a process id");
    let pidfd = pidfd_open(Pid::from_raw(id).expect("a pid"), PidfdFlags::empty()).expect("a pidfd");
    let number = |descriptor: &std::os::fd::OwnedFd| -> i64 {
        use std::os::fd::AsRawFd;
        let info: String = fs::read_to_string(format!("/proc/self/fdinfo/{}", descriptor.as_raw_fd())).expect("the fdinfo");
        return info.lines().find_map(|line| { return line.strip_prefix("Pid:"); }).expect("a Pid line").trim().parse::<i64>().expect("a number");
    };
    crate::cgroup_test_support::wait_until("the process to end and wait for its reaping", &|| { return fs::read_to_string(format!("/proc/{id}/stat")).is_ok_and(|text| { return text.rsplit(')').next().is_some_and(|rest| { return rest.trim_start().starts_with('Z'); }); }); });
    let unreaped: i64 = number(&pidfd);
    let _ = child.wait();
    let reaped: i64 = number(&pidfd);

    assert_eq!(unreaped, i64::from(id), "an unreaped zombie is still numbered");
    assert_eq!(reaped, -1, "the sentinel of an ended process is not -1 on this kernel");
}
