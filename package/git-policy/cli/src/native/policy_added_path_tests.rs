//! What: Controls for the added-path precondition of a direct fix and its message.
//! Why: Admitting a file that differs from `HEAD` anywhere would discard someone's change;
//!      refusing a clean one would make a narrow fix fail for no reason.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(() => assertAddablePath({ ... })).toThrow('its worktree copy has unstaged changes');
//! ```
#![cfg(unix)]

/// Import the precondition, its reasons and its message.
use super::{
    ADDED_PATH_HEAD_REASON, ADDED_PATH_MISSING_REASON, ADDED_PATH_STAGED_REASON,
    ADDED_PATH_UNSTAGED_REASON, Entry, added_path_message, head_mode, worktree_refusal,
};
/// The modes and object names entries carry.
use crate::candidate_object::{CandidateMode, ObjectId, parse_object_id};
/// Shared fixture directories.
use crate::test_support::{fixture, remove};
/// Unix permission bits.
use std::os::unix::fs::PermissionsExt;
/// Owned filesystem paths.
use std::path::PathBuf;

/// The object named by forty copies of one hexadecimal digit.
fn object(digit: u8) -> ObjectId {
    return parse_object_id(&[digit; 40]).expect("forty hexadecimal digits");
}

/// An entry of `mode` naming the object of one digit.
fn entry(mode: CandidateMode, digit: u8) -> Entry {
    return (mode, object(digit));
}

/// The message is the installed wrapper's direct-fix wording, with both remedies.
#[test]
fn the_message_names_the_file_the_reason_and_both_remedies() {
    assert_eq!(
        added_path_message("package/module/b/package.json", ADDED_PATH_UNSTAGED_REASON),
        "A policy fix needs to change package/module/b/package.json, which this fix did not select, \
         but its worktree copy has unstaged changes. Include package/module/b/package.json in the fix \
         pathspecs so the fix applies to its worktree copy, or restore it to match HEAD \
         (git restore --staged --worktree -- package/module/b/package.json), then run the fix again."
    );
}

/// `HEAD` must hold the computed-against object as an ordinary file; the index must hold it
/// with `HEAD`'s mode.
#[test]
fn head_and_index_must_hold_the_same_ordinary_file() {
    let regular: Entry = entry(CandidateMode::Regular, b'a');
    let executable: Entry = entry(CandidateMode::Executable, b'a');
    assert_eq!(
        head_mode(Some(&regular), &regular),
        Ok(CandidateMode::Regular)
    );
    assert_eq!(
        head_mode(Some(&executable), &executable),
        Ok(CandidateMode::Executable)
    );
    assert_eq!(head_mode(None, &regular), Err(ADDED_PATH_HEAD_REASON));
    assert_eq!(
        head_mode(Some(&entry(CandidateMode::Regular, b'b')), &regular),
        Err(ADDED_PATH_HEAD_REASON)
    );
    for mode in [CandidateMode::Symlink, CandidateMode::Gitlink] {
        assert_eq!(
            head_mode(Some(&entry(mode, b'a')), &entry(mode, b'a')),
            Err(ADDED_PATH_HEAD_REASON),
            "{mode:?}"
        );
    }
    assert_eq!(
        head_mode(Some(&regular), &executable),
        Err(ADDED_PATH_STAGED_REASON)
    );
    assert_eq!(
        head_mode(Some(&executable), &regular),
        Err(ADDED_PATH_STAGED_REASON)
    );
}

/// The worktree copy must be a plain file with `HEAD`'s executable bit and bytes.
#[test]
fn the_worktree_copy_must_match_head() {
    let root: PathBuf = fixture("added-path-worktree");
    let file: PathBuf = root.join("file.json");
    assert_eq!(
        worktree_refusal(file.as_path(), CandidateMode::Regular, b"x\n").expect("missing"),
        Some(ADDED_PATH_MISSING_REASON)
    );
    std::fs::write(&file, b"x\n").expect("file");
    assert_eq!(
        worktree_refusal(file.as_path(), CandidateMode::Regular, b"x\n").expect("clean"),
        None
    );
    assert_eq!(
        worktree_refusal(file.as_path(), CandidateMode::Regular, b"y\n").expect("changed"),
        Some(ADDED_PATH_UNSTAGED_REASON)
    );
    assert_eq!(
        worktree_refusal(file.as_path(), CandidateMode::Executable, b"x\n").expect("bit"),
        Some(ADDED_PATH_UNSTAGED_REASON)
    );
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o755)).expect("executable");
    assert_eq!(
        worktree_refusal(file.as_path(), CandidateMode::Executable, b"x\n").expect("clean"),
        None
    );
    assert_eq!(
        worktree_refusal(file.as_path(), CandidateMode::Regular, b"x\n").expect("bit"),
        Some(ADDED_PATH_UNSTAGED_REASON)
    );
    let link: PathBuf = root.join("link.json");
    std::os::unix::fs::symlink("file.json", &link).expect("link");
    assert_eq!(
        worktree_refusal(link.as_path(), CandidateMode::Executable, b"x\n").expect("link"),
        Some(ADDED_PATH_UNSTAGED_REASON)
    );
    let directory: PathBuf = root.join("directory.json");
    std::fs::create_dir(&directory).expect("directory");
    assert_eq!(
        worktree_refusal(directory.as_path(), CandidateMode::Regular, b"x\n").expect("dir"),
        Some(ADDED_PATH_UNSTAGED_REASON)
    );
    // A file the system refuses to read is an error, not a verdict.
    let unreadable: PathBuf = root.join("unreadable.json");
    std::fs::write(&unreadable, b"x\n").expect("unreadable file");
    std::fs::set_permissions(&unreadable, std::fs::Permissions::from_mode(0o200))
        .expect("write-only");
    if std::fs::read(&unreadable).is_err() {
        assert!(worktree_refusal(unreadable.as_path(), CandidateMode::Regular, b"x\n").is_err());
    }
    // A path below a file cannot be inspected at all.
    let beneath: PathBuf = file.join("child");
    assert!(worktree_refusal(beneath.as_path(), CandidateMode::Regular, b"x\n").is_err());
    remove(root.as_path());
}
