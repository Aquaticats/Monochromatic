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
    AUDIT_FILE, AuditOutcome, AuditRecord, ROTATE_AT_BYTES, append_line, audit_line, audit_place,
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
        directory: Path::new("/r/sub"),
        arguments: arguments.as_slice(),
        dirty: Some(true),
    });
    assert!(line.ends_with('\n'));
    assert_eq!(line.matches('\n').count(), 1);
    let value: serde_json::Value = serde_json::from_str(line.trim_end()).expect("JSON");
    assert_eq!(value["schemaVersion"], 1);
    assert_eq!(value["timeMillis"], "1700000000123");
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
        directory: Path::new("/r"),
        arguments: &[],
        dirty: None,
    });
    let unasked_value: serde_json::Value = serde_json::from_str(unasked.trim_end()).expect("JSON");
    assert_eq!(unasked_value["dirty"], serde_json::Value::Null);
    assert_eq!(unasked_value["outcome"], "hatch");
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
            directory: Path::new("/r"),
            arguments: &[],
            dirty: None,
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
    std::fs::write(&target, vec![b'x'; ROTATE_AT_BYTES as usize - 1]).expect("nearly full");
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

/// The size limit is one mebibyte: the rotation test measures against the constant, so only its value pins the limit.
#[test]
fn the_size_limit_is_one_mebibyte() {
    assert_eq!(ROTATE_AT_BYTES, 1_048_576);
}

/// The parent is the process that started the test, never the placeholder zero or one.
#[cfg(unix)]
#[test]
fn the_parent_is_the_process_that_started_this_one() {
    let parent: u32 = parent_process();
    assert_eq!(parent, std::os::unix::process::parent_id());
    assert!(parent > 1, "parent {parent}");
}
