//! What:
//!  The audit record's line, its place, its appending and its size limit.
//! Why:
//!  A record that names the wrong directory or loses a field cannot attribute a loss; one that fails the command it
//!  describes would turn a safety log into a new way to break work.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(JSON.parse(auditLine(record)).outcome).toBe('rejected');
//! ```

/// The record, its line and the writer.
use super::{
    AUDIT_FILE, AuditOutcome, AuditRecord, ROTATE_AT_BYTES, append_line, audit_line, audit_place, move_aside_if_still_named,
    parent_process, record_audit,
};
use crate::command_test_support::{fixture, os_arguments, remove};
use crate::rule_linked_worktree::GuardedCommand;
use crate::worktree_identity::WorktreeIdentity;
use std::path::{Path, PathBuf};

/// A main worktree identity under the given root.
fn main_identity(root: &Path) -> WorktreeIdentity {
    return WorktreeIdentity::MainWorktree {
        common_dir: root.join(".git"),
        git_dir: root.join(".git"),
        worktree_root: root.to_path_buf(),
    };
}

/// Every field of the line, as JSON.
#[test]
fn the_line_states_who_ran_what_where_and_what_was_decided() {
    let place: (PathBuf, &'static str) = (PathBuf::from("/r/.git"), "main");
    let arguments = os_arguments(&["-C", "/r", "restore", "--", "a.txt"]);
    let line: String = audit_line(&AuditRecord {
        time_millis: 1_700_000_000_123,
        process: 42,
        parent: 7,
        form: GuardedCommand::RestoreWorktree,
        outcome: AuditOutcome::Rejected,
        place: &place,
        git_dir: Path::new("/r/.git/worktrees/w"),
        directory: Path::new("/r/sub"),
        arguments: arguments.as_slice(),
        dirty: Some(true),
        copies: &[],
    });
    assert!(line.ends_with('\n'));
    assert_eq!(line.matches('\n').count(), 1);
    let value: serde_json::Value = serde_json::from_str(line.trim_end()).expect("JSON");
    assert_eq!(value["schemaVersion"], 1);
    assert_eq!(value["timeMillis"], "1700000000123");
    assert_eq!(value["time"], "2023-11-14T22:13:20.123Z", "the time in the form every other record uses");
    assert_eq!(value["release"], crate::service_protocol::RELEASE);
    assert_eq!(value["gitDir"], "/r/.git/worktrees/w", "the worktree's own Git directory, which the kind word does not name");
    assert_eq!(value["level"], "warn", "a refusal stands out");
    assert_eq!(value["pid"], 42);
    assert_eq!(value["parentPid"], 7);
    assert_eq!(value["worktree"], "main");
    assert_eq!(value["form"], "RestoreWorktree");
    assert_eq!(value["outcome"], "rejected");
    assert_eq!(value["dirty"], true);
    assert_eq!(value["cwd"], "/r/sub");
    assert_eq!(value["arguments"], serde_json::Value::from(vec!["-C", "/r", "restore", "--", "a.txt"]));
    let unasked: String = audit_line(&AuditRecord {
        time_millis: 1,
        process: 1,
        parent: 1,
        form: GuardedCommand::Stash,
        outcome: AuditOutcome::Hatch,
        place: &place,
        git_dir: Path::new("/r/.git/worktrees/w"),
        directory: Path::new("/r"),
        arguments: &[],
        dirty: None,
        copies: &[],
    });
    let unasked_value: serde_json::Value = serde_json::from_str(unasked.trim_end()).expect("JSON");
    assert_eq!(unasked_value["dirty"], serde_json::Value::Null);
    assert_eq!(unasked_value["outcome"], "hatch");
    assert_eq!(unasked_value["level"], "info");
}

/// The four outcomes have four distinct words.
#[test]
fn each_outcome_has_its_own_word() {
    let place: (PathBuf, &'static str) = (PathBuf::from("/r/.git"), "linked");
    let mut words: Vec<String> = Vec::<String>::new();
    for outcome in [
        AuditOutcome::Rejected,
        AuditOutcome::Allowed,
        AuditOutcome::Exempt,
        AuditOutcome::Hatch,
    ] {
        let line: String = audit_line(&AuditRecord {
            time_millis: 1,
            process: 1,
            parent: 1,
            form: GuardedCommand::Clean,
            outcome,
            place: &place,
        git_dir: Path::new("/r/.git/worktrees/w"),
            directory: Path::new("/r"),
            arguments: &[],
            dirty: None,
            copies: &[],
        });
        let value: serde_json::Value = serde_json::from_str(line.trim_end()).expect("JSON");
        words.push(value["outcome"].as_str().expect("word").to_owned());
    }
    assert_eq!(words, vec!["rejected", "allowed", "exempt", "hatch"]);
}

/// The record lives in the common Git directory of every kind of repository, and nowhere outside one.
#[test]
fn the_place_is_the_common_directory_with_the_worktree_kind() {
    let root = Path::new("/r");
    assert_eq!(audit_place(&WorktreeIdentity::OutsideWorktree), None);
    assert_eq!(
        audit_place(&main_identity(root)),
        Some((PathBuf::from("/r/.git"), "main"))
    );
    assert_eq!(
        audit_place(&WorktreeIdentity::LinkedWorktree {
            common_dir: PathBuf::from("/r/.git"),
            git_dir: PathBuf::from("/r/.git/worktrees/w"),
            worktree_root: PathBuf::from("/w"),
        }),
        Some((PathBuf::from("/r/.git"), "linked"))
    );
    assert_eq!(
        audit_place(&WorktreeIdentity::BareRepository {
            common_dir: PathBuf::from("/r.git"),
            git_dir: PathBuf::from("/r.git"),
        }),
        Some((PathBuf::from("/r.git"), "bare"))
    );
}

/// Records append in order to one file in the common directory; the directory is never created.
#[test]
fn records_append_in_order_and_the_directory_is_never_created() {
    let root: PathBuf = fixture("audit-append");
    std::fs::create_dir(root.join(".git")).expect("git directory");
    let identity: WorktreeIdentity = main_identity(root.as_path());
    let arguments = os_arguments(&["restore", "a.txt"]);
    record_audit(&identity, GuardedCommand::RestoreWorktree, AuditOutcome::Rejected, arguments.as_slice(), Some(false));
    record_audit(&identity, GuardedCommand::Stash, AuditOutcome::Hatch, arguments.as_slice(), None);
    let text: String = std::fs::read_to_string(root.join(".git").join(AUDIT_FILE)).expect("audit file");
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].contains("\"outcome\":\"rejected\""));
    assert!(lines[1].contains("\"outcome\":\"hatch\""));
    let missing: PathBuf = root.join("absent");
    record_audit(
        &main_identity(missing.as_path()),
        GuardedCommand::Clean,
        AuditOutcome::Allowed,
        arguments.as_slice(),
        Some(false),
    );
    assert!(!missing.exists());
    remove(root.as_path());
}

/// A file at the size limit moves aside to `.1` and a new one starts; below the limit nothing moves.
#[test]
fn a_full_file_moves_aside() {
    let root: PathBuf = fixture("audit-rotate");
    let target: PathBuf = root.join(AUDIT_FILE);
    let mut filler: Vec<u8> = vec![b'x'; ROTATE_AT_BYTES as usize - 1];
    filler[ROTATE_AT_BYTES as usize - 2] = b'\n';
    std::fs::write(&target, filler).expect("nearly full");
    append_line(target.as_path(), b"next\n").expect("append");
    assert!(!root.join(format!("{AUDIT_FILE}.1")).exists());
    assert_eq!(std::fs::metadata(&target).expect("metadata").len(), ROTATE_AT_BYTES + 4);
    append_line(target.as_path(), b"after\n").expect("append past the limit");
    assert_eq!(std::fs::read(&target).expect("new file"), b"after\n");
    assert_eq!(
        std::fs::metadata(root.join(format!("{AUDIT_FILE}.1"))).expect("aside").len(),
        ROTATE_AT_BYTES + 4
    );
    remove(root.as_path());
}

/// A zero size limit would move the audit file aside before every write, so the limit has to be positive.
#[test]
fn the_size_limit_is_positive() {
    const { assert!(ROTATE_AT_BYTES > 0) };
}

/// The parent is the process that started the test, never the placeholder zero or one.
#[cfg(unix)]
#[test]
fn the_parent_is_the_process_that_started_this_one() {
    let parent: u32 = parent_process();
    assert_eq!(parent, std::os::unix::process::parent_id());
    assert!(parent > 1, "parent {parent}");
}

/// A write a full disk or a signal cut short leaves a last line without its newline; the next record starts on a line of its
/// own, so the torn line is lost alone and the record after it still parses.
#[test]
fn a_torn_last_line_does_not_swallow_the_next_record() {
    let root: PathBuf = fixture("audit-torn");
    let target: PathBuf = root.join(AUDIT_FILE);
    std::fs::write(&target, b"{\"ok\":1}\n{\"torn\":").expect("a torn tail");
    append_line(target.as_path(), b"{\"next\":2}\n").expect("append");
    append_line(target.as_path(), b"{\"last\":3}\n").expect("append");
    let text: String = std::fs::read_to_string(&target).expect("file");
    let parsed: Vec<bool> = text.lines().map(|line| { return serde_json::from_str::<serde_json::Value>(line).is_ok(); }).collect();
    assert_eq!(parsed, vec![true, false, true, true], "{text}");
    assert_eq!(text.lines().last(), Some("{\"last\":3}"), "a whole file adds nothing extra");
    remove(root.as_path());
}

/// A writer that saw a full file but lost the race to another writer's rotation must not move the new, small file over
/// the history that is already in `.1`.
#[test]
fn a_second_rotation_does_not_replace_the_history_with_the_new_file() {
    let root: PathBuf = fixture("audit-rotate-race");
    let target: PathBuf = root.join(AUDIT_FILE);
    let aside: PathBuf = root.join(format!("{AUDIT_FILE}.1"));
    std::fs::write(&target, b"the full history\n").expect("the full file");
    let seen_full: std::fs::File = std::fs::OpenOptions::new().read(true).append(true).open(&target).expect("open");
    // Another writer rotates first and starts the new file.
    std::fs::rename(&target, &aside).expect("the other writer's rotation");
    std::fs::write(&target, b"{\"new\":1}\n").expect("the new file");
    move_aside_if_still_named(target.as_path(), &seen_full).expect("no error");
    assert_eq!(std::fs::read(&aside).expect("aside"), b"the full history\n", "the history is still there");
    assert_eq!(std::fs::read(&target).expect("target"), b"{\"new\":1}\n", "the new file stays");
    // The same writer, when the path still names its file, does move it.
    let current: std::fs::File = std::fs::OpenOptions::new().read(true).append(true).open(&target).expect("open");
    move_aside_if_still_named(target.as_path(), &current).expect("moved");
    assert!(!target.exists());
    assert_eq!(std::fs::read(&aside).expect("aside"), b"{\"new\":1}\n");
    remove(root.as_path());
}

/// Writers in several threads each append whole lines: none is lost, torn or joined to another.
#[test]
fn concurrent_appenders_lose_no_line_and_tear_none() {
    let root: PathBuf = fixture("audit-concurrent");
    let target: PathBuf = root.join(AUDIT_FILE);
    let writers: usize = 6;
    let each: usize = 60;
    std::thread::scope(|scope| {
        for writer in 0..writers {
            let path: &Path = target.as_path();
            scope.spawn(move || {
                for number in 0..each {
                    let line: String = format!("{{\"writer\":{writer},\"number\":{number},\"pad\":\"{}\"}}\n", "p".repeat(200));
                    append_line(path, line.as_bytes()).expect("append");
                }
            });
        }
    });
    let text: String = std::fs::read_to_string(&target).expect("file");
    let mut seen: std::collections::BTreeSet<(u64, u64)> = std::collections::BTreeSet::new();
    // A writer that reads the last byte while another one is half-way through a page-crossing write sees no newline and
    //  starts its line on a fresh one: the result is an empty line between records, which every reader of the file skips.
    for line in text.lines().filter(|line| { return !line.is_empty(); }) {
        let value: serde_json::Value = serde_json::from_str(line).unwrap_or_else(|error| { panic!("a torn line {line:?}: {error}") });
        seen.insert((value["writer"].as_u64().expect("writer"), value["number"].as_u64().expect("number")));
    }
    assert_eq!(seen.len(), writers * each);
    remove(root.as_path());
}
