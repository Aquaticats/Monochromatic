//! A measurement for the record,
//! not a control:
//! what the rotation of a log costs on the path that appends a line,
//! and what the audit log's
//!  pin check costs on the thread the rotation hands it to.
//! `schedule_trim` and the entry in
//!  `doc/planning/cli-git-shipped-limits-audit.md`,
//! "The audit and timing logs' history",
//! cite the result.
//!  Run:
//! `mise run //package/git-policy/cli:native:test:host -- --run-ignored only --no-capture rotation_cost_measurement`.
//!
//! Measured on 2026-11-10 on this host,
//! 32 MiB of audit generations in a fixture on the repository's own disk:
//! - a rotation of a log under its bound:
//!   median 267 us,
//!   99th percentile 322 us;
//! - a rotation of the audit log over its bound,
//!   the pass scheduled and not waited for:
//!   median 385 us,
//!   99th percentile 580 us;
//! - the pass on its own thread,
//!   dropping one generation that names no saved copy:
//!   median 13 ms,
//!   99th percentile 254 ms;
//! - the pass on its own thread where every generation names a saved copy on its last line and none may go:
//!   median 369 ms,
//!   99th percentile 432 ms.
//! The same pass run inside the appending call,
//! measured before it moved to a thread,
//! cost a median of 23.5 ms and 542 ms in those
//!  two shapes,
//! and 4.4 s at the 99th percentile in the second,
//! which is what put it on a thread of its own.
#![cfg(unix)]

use crate::command_test_support::{fixture, os_arguments, remove};
use crate::discard_audit::{AUDIT_RETENTION, AuditOutcome};
use crate::discard_audit_record::{AuditRecord, DecidedForm, audit_line};
use crate::log_generations_kept_to_a_bound::{LOG_HISTORY_BYTES, LogRetention, UNPINNED_LOG, a_pass_is_running, trim_generations};
use crate::log_lines_appended_without_a_lock::{ROTATE_AT_BYTES, append_line};
use crate::rule_linked_worktree::GuardedCommand;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// The line an audit record writes,
/// near enough to the shipped length that the read of a generation costs what a real one costs.
const AUDIT_LIKE_LINE: &str = concat!(
    r#"{"schemaVersion":1,"timeMillis":"1794301687351","time":"2026-11-10T09:08:07.759Z","level":"info","pid":3498659,"parentPid":1,"#,
    r#""worktree":"linked","form":"Commit","outcome":"allowed","dirty":false,"cwd":"/var/mnt/encrypted/repository/Monochromatic","#,
    r#""arguments":["commit","--message","a subject line"],"release":"0.6.1-alpha.6","gitDir":"/r/.git/worktrees/w"}"#
);

/// How many rotations one measurement times.
const ROTATIONS: usize = 20;

/// How many whole generations the bound holds.
fn kept_generations() -> u64 {
    return LOG_HISTORY_BYTES / ROTATE_AT_BYTES;
}

/// Prints the spread of `micros`:
/// how many,
/// the middle,
/// the 99th percentile and the largest,
/// in microseconds.
fn print_spread(label: &str, mut micros: Vec<u128>) {
    micros.sort_unstable();
    let at = |per_thousand: usize| -> u128 { return micros[(micros.len() * per_thousand / 1000).min(micros.len() - 1)]; };
    println!("{label}: n={} median={}us p99={}us max={}us", micros.len(), at(500), at(990), micros.last().copied().unwrap_or(0));
}

/// What one rotated append cost on the command's path,
/// timed around the whole call:
/// the open,
/// the check of the size,
/// the rotation,
/// the scheduling of the pass and the line's own write.
fn time_rotations(target: &Path, live: &[u8], retention: &LogRetention) -> Vec<u128> {
    let mut micros: Vec<u128> = Vec::with_capacity(ROTATIONS);
    for _ in 0..ROTATIONS {
        std::fs::write(target, live).expect("a full live file");
        let started: Instant = Instant::now();
        append_line(target, b"{\"live\":1}\n", retention).expect("the append that rotates");
        micros.push(started.elapsed().as_micros());
        wait_for_the_pass(target);
    }
    return micros;
}

/// Waits,
/// within a deadline,
/// for the pass a rotation scheduled to end,
/// so each timed rotation starts from a settled state.
fn wait_for_the_pass(target: &Path) {
    let deadline: Instant = Instant::now() + Duration::from_secs(600);
    while a_pass_is_running(target) {
        assert!(Instant::now() < deadline, "the pass for {} did not end within its deadline", target.display());
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// What one pass cost on its own thread,
/// timed around the call that removes the generations the bound does not keep.
fn time_passes(target: &Path, live: &[u8], retention: &LogRetention) -> Vec<u128> {
    let mut micros: Vec<u128> = Vec::with_capacity(ROTATIONS);
    for _ in 0..ROTATIONS {
        std::fs::write(target, live).expect("a full live file");
        let taken: std::fs::File = std::fs::OpenOptions::new().read(true).append(true).open(target).expect("open");
        rotate_now(target, &taken);
        let started: Instant = Instant::now();
        trim_generations(target, retention).expect("the pass");
        micros.push(started.elapsed().as_micros());
    }
    return micros;
}

/// The rotation alone,
/// on this thread,
/// as the pass would find it.
fn rotate_now(target: &Path, opened: &std::fs::File) {
    crate::log_lines_appended_without_a_lock::move_aside_between(target, opened, &mut |_step: crate::log_lines_appended_without_a_lock::RotationStep| {}).expect("the rotation");
}

/// A file of exactly `ROTATE_AT_BYTES` bytes,
/// filled with `line` up to `ends_with`,
/// which ends it if there is room,
/// every line ended,
/// so that the next append rotates it.
fn full_file(line: &str, ends_with: Option<&str>) -> Vec<u8> {
    let mut bytes: Vec<u8> = Vec::with_capacity(ROTATE_AT_BYTES as usize);
    let tail: usize = ends_with.map_or(0, |ending| { return ending.len() + 1; });
    while bytes.len() + line.len() + 1 + tail <= ROTATE_AT_BYTES as usize {
        bytes.extend_from_slice(line.as_bytes());
        bytes.push(b'\n');
    }
    let room: usize = ROTATE_AT_BYTES as usize - bytes.len();
    bytes.extend(std::iter::repeat_n(b'x', room - 1));
    bytes.push(b'\n');
    if let Some(ending) = ends_with {
        // The line that decides whether the generation pins is the last one, so reading to it costs the whole file.
        bytes.truncate(ROTATE_AT_BYTES as usize - ending.len() - 1);
        bytes.extend_from_slice(ending.as_bytes());
        bytes.push(b'\n');
    }
    return bytes;
}

/// Writes `count` generations of `target`,
/// each of `bytes`.
fn generations(target: &Path, count: u64, bytes: &[u8]) {
    for number in 1..=count {
        let mut name: std::ffi::OsString = target.as_os_str().to_os_string();
        name.push(format!(".{number}"));
        std::fs::write(PathBuf::from(name), bytes).expect("a generation");
    }
}

/// The numbered generations of `target` that exist,
/// so a printed spread says how much history the pass worked over.
fn count_generations(target: &Path) -> usize {
    let directory: &Path = target.parent().expect("a directory");
    let prefix: String = format!("{}.", target.file_name().expect("a name").to_string_lossy());
    return std::fs::read_dir(directory)
        .expect("directory")
        .filter(|entry| {
            let name: String = entry.as_ref().expect("entry").file_name().to_string_lossy().into_owned();
            let Some(rest) = name.strip_prefix(prefix.as_str()) else { return false };
            return !rest.is_empty() && rest.bytes().all(|byte| { return byte.is_ascii_digit(); });
        })
        .count();
}

/// A real audit line of a `saved-conflict` decision that saved the copy at `place`,
/// so a generation holding it pins.
fn saved_conflict_line(root: &Path, place: &Path) -> String {
    let held: (PathBuf, &'static str) = (root.join(".git"), "main");
    let arguments = os_arguments(&["checkout", "--theirs", "--", "a.rs"]);
    let line: String = audit_line(&AuditRecord {
        time_millis: 1_700_000_000_000,
        process: 1,
        parent: 1,
        form: DecidedForm::Guarded(GuardedCommand::CheckoutPaths),
        outcome: AuditOutcome::SavedConflict,
        place: &held,
        git_dir: held.0.as_path(),
        directory: root,
        arguments: arguments.as_slice(),
        dirty: None,
        copies: &[format!("a.rs => {}", place.display())],
        facts: &[],
    });
    return line.trim_end().to_string();
}

/// A rotation of a log under its bound:
/// the directory listing and the move,
/// and no generation is read.
#[test]
#[ignore = "a measurement for the record, not a control"]
fn measure_a_rotation_under_the_bound() {
    let root: PathBuf = fixture("rotation-cost-under");
    let target: PathBuf = root.join("under.jsonl");
    let live: Vec<u8> = full_file(AUDIT_LIKE_LINE, None);
    generations(target.as_path(), 4, live.as_slice());
    let micros: Vec<u128> = time_rotations(target.as_path(), live.as_slice(), &UNPINNED_LOG);
    print_spread("rotation under the bound", micros);
    println!("generations left: {}", count_generations(target.as_path()));
    remove(root.as_path());
}

/// A rotation of the audit log over its bound,
/// on the command's path:
/// the move and the scheduling of the pass,
/// which reads the generation that may or may not pin.
#[test]
#[ignore = "a measurement for the record, not a control"]
fn measure_a_rotation_of_the_audit_log_over_the_bound() {
    let root: PathBuf = fixture("rotation-cost-over");
    let target: PathBuf = root.join("over.jsonl");
    let live: Vec<u8> = full_file(AUDIT_LIKE_LINE, None);
    generations(target.as_path(), kept_generations(), live.as_slice());
    let micros: Vec<u128> = time_rotations(target.as_path(), live.as_slice(), &AUDIT_RETENTION);
    print_spread("rotation of the audit log over the bound, on the command's path", micros);
    println!("generations left: {} of {}", count_generations(target.as_path()), kept_generations());
    remove(root.as_path());
}

/// The pass of an audit log over its bound,
/// on its own thread:
/// the oldest generation is read to the end to see that it names no saved copy,
/// then removed.
#[test]
#[ignore = "a measurement for the record, not a control"]
fn measure_the_pass_of_the_audit_log_over_the_bound() {
    let root: PathBuf = fixture("rotation-cost-pass");
    let target: PathBuf = root.join("pass.jsonl");
    let live: Vec<u8> = full_file(AUDIT_LIKE_LINE, None);
    generations(target.as_path(), kept_generations(), live.as_slice());
    let micros: Vec<u128> = time_passes(target.as_path(), live.as_slice(), &AUDIT_RETENTION);
    print_spread("pass of the audit log over the bound, on its own thread", micros);
    println!("generations left: {} of {}", count_generations(target.as_path()), kept_generations());
    remove(root.as_path());
}

/// The pass of an audit log whose generations each name a saved copy still on disk on their last line:
/// it reads every one of them to the end and keeps them all,
/// which is the most one pass can cost,
/// and the rotation that schedules it costs the command nothing.
#[test]
#[ignore = "a measurement for the record, not a control"]
fn measure_the_pass_of_the_audit_log_whose_generations_all_pin() {
    let root: PathBuf = fixture("rotation-cost-pinned");
    let target: PathBuf = root.join("pinned.jsonl");
    let copy: PathBuf = root.join("copy-a");
    std::fs::write(&copy, b"the only content of a replaced file\n").expect("a saved copy");
    let pinned: String = saved_conflict_line(root.as_path(), copy.as_path());
    let live: Vec<u8> = full_file(AUDIT_LIKE_LINE, Some(pinned.as_str()));
    generations(target.as_path(), kept_generations(), live.as_slice());
    let micros: Vec<u128> = time_passes(target.as_path(), live.as_slice(), &AUDIT_RETENTION);
    let total: u64 = kept_generations() + u64::try_from(ROTATIONS).expect("small");
    print_spread("pass of the audit log whose generations all pin", micros);
    println!("generations left: {} of {total}", count_generations(target.as_path()));
    remove(root.as_path());
}
