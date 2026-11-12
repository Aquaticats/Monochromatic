//! What:
//!  The audit file's generations:
//! a generation that names a saved copy still on disk is never removed,
//! and the rest are kept up
//!  to the bound every log shares.
//! Why:
//!  The audit file attributes a loss of uncommitted work,
//! and each `saved-conflict` line names the saved copies it made.
//! A generation holding such a line is the only pointer to a place that still exists,
//! so it outlives the bound (the owner's answer of
//!  2026-11-10 in the question tool:
//! "an audit line naming a saved copy that still exists is never dropped").
//! The rotation's races and the numbering of generations are controlled beside the appender
//!  (`log_generations_kept_to_a_bound_tests.rs`).
//! The tests put files in a disposable directory and never touch a real Git directory.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // await rotate(auditFile); expect(await generations(dir)).toEqual([1, 2]);
//! ```

use crate::command_test_support::{fixture, os_arguments, remove};
use crate::discard_audit::{AUDIT_FILE, AUDIT_RETENTION, AuditOutcome};
use crate::discard_audit_record::{AuditRecord, DecidedForm, audit_line};
use crate::log_generations_kept_to_a_bound::{LOG_HISTORY_BYTES, a_pass_is_running};
use crate::log_lines_appended_without_a_lock::{ROTATE_AT_BYTES, append_line};
use crate::rule_linked_worktree::GuardedCommand;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// How many whole generations the bound holds.
fn kept_generations() -> u64 {
    return LOG_HISTORY_BYTES / ROTATE_AT_BYTES;
}

/// A file of exactly `ROTATE_AT_BYTES` bytes whose lines are `lines` and then filler,
/// every line ended,
/// so that the next
/// append sees a full file.
fn full_file(lines: &[String]) -> Vec<u8> {
    let mut bytes: Vec<u8> = Vec::new();
    for line in lines {
        bytes.extend_from_slice(line.as_bytes());
        bytes.push(b'\n');
    }
    let room: usize = ROTATE_AT_BYTES as usize - bytes.len();
    bytes.extend(std::iter::repeat_n(b'x', room - 1));
    bytes.push(b'\n');
    return bytes;
}

/// A line that names its generation and nothing else.
fn marker(name: &str) -> String {
    return format!("{{\"marker\":\"{name}\"}}");
}

/// Writes the file `target.<number>` as a generation made of the lines and filler.
fn write_generation(target: &Path, number: u64, lines: &[String]) {
    let mut name: std::ffi::OsString = target.as_os_str().to_os_string();
    name.push(format!(".{number}"));
    std::fs::write(PathBuf::from(name), full_file(lines)).expect("a generation");
}

/// The real audit line of a `saved-conflict` decision that saved `copies` (each `path => place`).
fn saved_conflict_line(root: &Path, copies: &[String]) -> String {
    let place: (PathBuf, &'static str) = (root.join(".git"), "main");
    let arguments = os_arguments(&["checkout", "--theirs", "--", "a.rs"]);
    let line: String = audit_line(&AuditRecord {
        time_millis: 1_700_000_000_000,
        process: 1,
        parent: 1,
        form: DecidedForm::Guarded(GuardedCommand::CheckoutPaths),
        outcome: AuditOutcome::SavedConflict,
        place: &place,
        git_dir: place.0.as_path(),
        directory: root,
        arguments: arguments.as_slice(),
        dirty: None,
        copies,
        facts: &[],
    });
    return line.trim_end().to_string();
}

/// A saved copy that exists on disk,
/// and the `path => place` text an audit line names it by.
fn existing_copy(root: &Path, name: &str) -> String {
    let place: PathBuf = root.join(name);
    std::fs::write(&place, b"the only content of a replaced file\n").expect("a saved copy");
    return format!("a.rs => {}", place.display());
}

/// Fills the live file so that the next append rotates it,
/// then appends one line,
/// and waits for the pass the rotation schedules,
/// so what the control reads afterwards is the state the shipped code settles at.
fn rotate_once(target: &Path, name: &str) {
    std::fs::write(target, full_file(&[marker(name)])).expect("a full live file");
    append_line(target, b"{\"live\":1}\n", &AUDIT_RETENTION).expect("the append that rotates");
    let deadline: Instant = Instant::now() + Duration::from_secs(120);
    while a_pass_is_running(target) {
        assert!(Instant::now() < deadline, "the pass for {} did not end within its deadline", target.display());
        std::thread::sleep(Duration::from_millis(5));
    }
}

/// The numbers of the generations `target.<number>` that exist,
/// ascending.
fn generations(target: &Path) -> Vec<u64> {
    let directory: &Path = target.parent().expect("a directory");
    let prefix: String = format!("{}.", target.file_name().expect("a name").to_string_lossy());
    let mut numbers: Vec<u64> = std::fs::read_dir(directory)
        .expect("directory")
        .filter_map(|entry| {
            let name: String = entry.expect("entry").file_name().to_string_lossy().into_owned();
            return name.strip_prefix(prefix.as_str()).and_then(|rest| { return rest.parse::<u64>().ok(); });
        })
        .collect();
    numbers.sort_unstable();
    return numbers;
}

/// The first line of generation `number`.
fn first_line_of(target: &Path, number: u64) -> String {
    let mut name: std::ffi::OsString = target.as_os_str().to_os_string();
    name.push(format!(".{number}"));
    let text: String = String::from_utf8_lossy(&std::fs::read(PathBuf::from(name)).expect("a generation")).into_owned();
    return text.lines().next().unwrap_or_default().to_string();
}

/// The audit file's generations are bounded by the bound every log shares,
/// not by a number of its own.
#[test]
fn the_audit_bound_is_the_one_every_log_shares() {
    assert_eq!(AUDIT_RETENTION.budget, LOG_HISTORY_BYTES);
}

/// A generation that names a saved copy still on disk stays when the bound is hit,
/// and the next oldest goes in its place.
#[test]
fn a_generation_that_names_a_copy_on_disk_is_never_removed() {
    let root: PathBuf = fixture("generations-pinned");
    let target: PathBuf = root.join(AUDIT_FILE);
    let extra: u64 = 3;
    let copy: String = existing_copy(root.as_path(), "copy-a");
    // The copy is named on the last line of an otherwise full generation, so only a read to its end finds it.
    write_generation(target.as_path(), 1, &[marker("g1"), saved_conflict_line(root.as_path(), &[copy])]);
    for number in 2..=kept_generations() + extra - 1 {
        write_generation(target.as_path(), number, &[marker(&format!("g{number}"))]);
    }
    rotate_once(target.as_path(), "newest");
    let newest: u64 = kept_generations() + extra;
    // Generation 1 stays, so the three the bound asks for are the three after it.
    let mut expected: Vec<u64> = vec![1];
    expected.extend(extra + 2..=newest);
    assert_eq!(generations(target.as_path()), expected, "generation 1 stays; the bound is spent on the others");
    remove(root.as_path());
}

/// A generation whose copy is gone no longer explains anything on disk,
/// so the bound removes it like any other.
#[test]
fn a_generation_that_names_a_deleted_copy_goes_like_any_other() {
    let root: PathBuf = fixture("generations-deleted");
    let target: PathBuf = root.join(AUDIT_FILE);
    let extra: u64 = 3;
    let copy: String = existing_copy(root.as_path(), "copy-b");
    write_generation(target.as_path(), 1, &[marker("g1"), saved_conflict_line(root.as_path(), &[copy])]);
    std::fs::remove_file(root.join("copy-b")).expect("the copy is deleted");
    for number in 2..=kept_generations() + extra - 1 {
        write_generation(target.as_path(), number, &[marker(&format!("g{number}"))]);
    }
    rotate_once(target.as_path(), "newest");
    let newest: u64 = kept_generations() + extra;
    assert_eq!(generations(target.as_path()), (extra + 1..=newest).collect::<Vec<u64>>());
    remove(root.as_path());
}

/// When every older generation names a copy on disk,
/// the bound removes none of them,
/// and the newest history that names
/// nothing is not squeezed out by them.
#[test]
fn when_only_pinned_generations_exceed_the_bound_none_is_removed() {
    let root: PathBuf = fixture("generations-all-pinned");
    let target: PathBuf = root.join(AUDIT_FILE);
    let extra: u64 = 3;
    for number in 1..=kept_generations() + extra - 1 {
        let copy: String = existing_copy(root.as_path(), &format!("copy-{number}"));
        write_generation(target.as_path(), number, &[marker(&format!("g{number}")), saved_conflict_line(root.as_path(), &[copy])]);
    }
    rotate_once(target.as_path(), "newest");
    let newest: u64 = kept_generations() + extra;
    assert_eq!(generations(target.as_path()), (1..=newest).collect::<Vec<u64>>(), "all stay, over the bound");
    remove(root.as_path());
}

/// Only a `saved-conflict` line that names a place on disk pins a generation:
/// a copy list with nothing to copy,
/// and a pruned
/// directory's line (which records a removal),
/// explain nothing that still exists.
#[test]
fn lines_that_name_nothing_on_disk_do_not_pin_a_generation() {
    let root: PathBuf = fixture("generations-unpinned-lines");
    let target: PathBuf = root.join(AUDIT_FILE);
    let extra: u64 = 3;
    let pruned: String = format!(
        "{{\"schemaVersion\":1,\"form\":\"PruneDiscarded\",\"outcome\":\"pruned\",\"path\":{},\"copies\":[\"a.rs\"]}}",
        crate::json_record::quote(&root.to_string_lossy())
    );
    write_generation(
        target.as_path(),
        1,
        &[marker("g1"), saved_conflict_line(root.as_path(), &[String::from("a.rs => none")]), pruned],
    );
    for number in 2..=kept_generations() + extra - 1 {
        write_generation(target.as_path(), number, &[marker(&format!("g{number}"))]);
    }
    rotate_once(target.as_path(), "newest");
    let newest: u64 = kept_generations() + extra;
    assert_eq!(generations(target.as_path()), (extra + 1..=newest).collect::<Vec<u64>>());
    remove(root.as_path());
}

/// A `saved-conflict` line that cannot be parsed (a write cut short) or that names a place the writer could not show as text
/// might name a copy that exists,
/// so its generation stays.
#[test]
fn a_line_that_may_name_a_copy_keeps_its_generation() {
    let root: PathBuf = fixture("generations-doubtful");
    let target: PathBuf = root.join(AUDIT_FILE);
    let extra: u64 = 3;
    let torn: String = String::from("{\"schemaVersion\":1,\"outcome\":\"saved-conflict\",\"copies\":[\"a.rs => /where/it/we");
    let unshowable: String = saved_conflict_line(root.as_path(), &[String::from("a.rs => /bytes/that/are/\u{FFFD}/in/the/path")]);
    write_generation(target.as_path(), 1, &[marker("g1"), torn]);
    write_generation(target.as_path(), 2, &[marker("g2"), unshowable]);
    for number in 3..=kept_generations() + extra - 1 {
        write_generation(target.as_path(), number, &[marker(&format!("g{number}"))]);
    }
    rotate_once(target.as_path(), "newest");
    let newest: u64 = kept_generations() + extra;
    let numbered: Vec<u64> = generations(target.as_path());
    assert!(numbered.starts_with(&[1, 2]), "both doubtful generations stay: {numbered:?}");
    assert_eq!(numbered.last(), Some(&newest));
    assert_eq!(first_line_of(target.as_path(), 1), marker("g1"), "the doubtful line is still there to read");
    remove(root.as_path());
}
