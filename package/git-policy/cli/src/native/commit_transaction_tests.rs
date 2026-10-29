//! Controls for the commit transaction's pure decisions and conversions.

use super::*;
use crate::command_commit::parse_commit_region;
use crate::index_writer_coordination::CoordinationFailure;
use crate::test_support::{fixture, remove};

/// The commit region of these tokens (everything after `commit`).
fn region(values: &[&str]) -> CommitRegion {
    let tokens: Vec<OsString> = values.iter().map(OsString::from).collect();
    return parse_commit_region(tokens.as_slice(), &[]).expect("a region Git's table reads");
}

/// The selection of a region with no sequencer conclusion.
fn selection(values: &[&str]) -> Selection {
    return decide_selection(&region(values), Conclusion::None);
}

/// A pathless commit, `--no-only`, `-a`, a selection UI and a sequencer conclusion use the index;
///  named paths use explicit-path mode.
#[test]
fn the_mode_follows_the_selection() {
    assert_eq!(selection(&["-m", "x"]).mode, TransactionMode::Index);
    assert_eq!(selection(&["-m", "x", "a.txt"]).mode, TransactionMode::ExplicitPath);
    assert_eq!(selection(&["-m", "x", "--", "a.txt"]).mode, TransactionMode::ExplicitPath);
    assert_eq!(selection(&["-m", "x", "--no-only", "a.txt"]).mode, TransactionMode::Index);
    assert_eq!(selection(&["-m", "x", "-o", "a.txt"]).mode, TransactionMode::ExplicitPath);
    assert_eq!(selection(&["-a", "-m", "x"]).mode, TransactionMode::Index);
    assert_eq!(selection(&["-a", "-m", "x", "a.txt"]).mode, TransactionMode::Index);
    assert_eq!(selection(&["-i", "-m", "x", "a.txt"]).mode, TransactionMode::Index);
    assert_eq!(selection(&["--pathspec-from-file=list", "-m", "x"]).mode, TransactionMode::ExplicitPath);
    assert_eq!(decide_selection(&region(&["-m", "x", "a.txt"]), Conclusion::Merge).mode, TransactionMode::Index);
    assert_eq!(decide_selection(&region(&["-m", "x", "a.txt"]), Conclusion::Amend).mode, TransactionMode::ExplicitPath);
}

/// Only a sequencer conclusion other than an amend concludes a sequencer.
#[test]
fn only_a_merge_cherry_pick_or_revert_concludes_a_sequencer() {
    let plain: CommitRegion = region(&["-m", "x"]);
    assert!(!decide_selection(&plain, Conclusion::None).concludes_sequencer);
    assert!(!decide_selection(&plain, Conclusion::Amend).concludes_sequencer);
    assert!(decide_selection(&plain, Conclusion::Merge).concludes_sequencer);
    assert!(decide_selection(&plain, Conclusion::CherryPick).concludes_sequencer);
    assert!(decide_selection(&plain, Conclusion::Revert).concludes_sequencer);
}

/// The selection UI makes the policies read-only; `-a` stages privately only without a path or a UI or an explicit `--only`.
#[test]
fn the_selection_flags_follow_the_options() {
    assert!(selection(&["-i", "a.txt"]).read_only);
    assert!(selection(&["-p"]).read_only);
    assert!(selection(&["--interactive"]).read_only);
    assert!(!selection(&["-m", "x"]).read_only);
    assert!(selection(&["-a", "-m", "x"]).staged_all);
    assert!(!selection(&["-m", "x"]).staged_all);
    assert!(!selection(&["-a", "a.txt"]).staged_all);
    assert!(!selection(&["-a", "-i"]).staged_all);
    assert!(!selection(&["-a", "-o"]).staged_all);
    assert!(selection(&["a.txt"]).selects_paths);
    assert!(selection(&["--pathspec-from-file=list"]).selects_paths);
    assert!(!selection(&["-m", "x"]).selects_paths);
}

/// A candidate request names each path literally and rooted at the top, and carries the base.
#[test]
fn the_candidate_request_names_paths_literally() {
    let request: CandidateRequest = staged_at_base(None, &[b"a[1].txt".to_vec(), b"dir/b.txt".to_vec()]);
    assert_eq!(
        request,
        CandidateRequest::StagedAtBase {
            base: None,
            pathspecs: vec![
                OsString::from(":(top,literal)a[1].txt"),
                OsString::from(":(top,literal)dir/b.txt"),
            ],
        }
    );
}

/// A transaction directory is the registry root joined with the ID.
#[test]
fn the_transaction_directory_is_the_registry_entry() {
    assert_eq!(transaction_directory(Path::new("/r"), "abc"), PathBuf::from("/r/abc"));
}

/// A journal record is written once, durably; a second write of the same name fails.
#[test]
fn a_journal_record_is_written_exclusively() {
    let root: PathBuf = fixture("transaction-journal");
    write_journal_record(root.as_path(), "record.json", "{}\n").expect("first write");
    assert_eq!(std::fs::read_to_string(root.join("record.json")).expect("record"), "{}\n");
    assert!(matches!(
        write_journal_record(root.as_path(), "record.json", "{}\n"),
        Err(TransactionStop::Failed { code: EngineFailureCode::TransactionFailed, .. })
    ));
    assert!(write_journal_record(root.join("missing").as_path(), "record.json", "{}\n").is_err());
    remove(root.as_path());
}

/// Each failure maps to the ending the incumbent's boundary gives it.
#[test]
fn failures_map_to_their_endings() {
    assert!(matches!(
        TransactionStop::from(GitFailure(String::from("boom"))),
        TransactionStop::Message(text) if text == "boom"
    ));
    assert!(matches!(
        TransactionStop::from(LandingError::Git(GitFailure(String::from("git"))),),
        TransactionStop::Message(text) if text == "git"
    ));
    assert!(matches!(
        TransactionStop::from(LandingError::Lock(CoordinationFailure::UnprovenOwner(String::from("who")))),
        TransactionStop::Failed { code: EngineFailureCode::IndexLockUnprovenOwner, message } if message == "who"
    ));
    assert!(matches!(
        TransactionStop::from(LandingError::Journal(String::from("journal"))),
        TransactionStop::Failed { code: EngineFailureCode::TransactionFailed, message } if message == "journal"
    ));
    assert!(matches!(
        TransactionStop::from(CaptureOrderError::Git(GitFailure(String::from("capture")))),
        TransactionStop::Message(text) if text == "capture"
    ));
    assert!(matches!(
        TransactionStop::from(CaptureOrderError::Write(String::from("write"))),
        TransactionStop::Failed { code: EngineFailureCode::TransactionFailed, message } if message == "write"
    ));
    assert!(matches!(
        failed(String::from("x")),
        TransactionStop::Failed { code: EngineFailureCode::TransactionFailed, .. }
    ));
}

/// The base of an unborn branch is no object; a commit base is its object name; garbage is a failure.
#[test]
fn the_base_object_follows_the_capture() {
    use crate::capture_invocation::InvocationLayout;
    use crate::transaction_journal::{RefFormat, SymbolicHead};
    let capture = |base: Base| {
        return InvocationCapture {
            base,
            layout: InvocationLayout {
                symbolic_head: SymbolicHead::Branch(String::from("refs/heads/main")),
                target_ref: String::from("refs/heads/main"),
                conclusion: Conclusion::None,
                repository_root: PathBuf::from("/r"),
                git_dir: PathBuf::from("/r/.git"),
                common_dir: PathBuf::from("/r/.git"),
                real_index_path: PathBuf::from("/r/.git/index"),
                object_directory: PathBuf::from("/r/.git/objects"),
                registry_root: PathBuf::from("/r/.git/cli-git-transactions"),
                ref_format: RefFormat::Files,
                object_format: String::from("sha1"),
                empty_tree_oid: "4".repeat(40),
                invoked_at: String::from("2026-10-11T00:00:00.000Z"),
                config_snapshot: None,
            },
        };
    };
    assert_eq!(base_object(&capture(Base::Unborn)).expect("unborn"), None);
    let oid: String = "a".repeat(40);
    assert_eq!(
        base_object(&capture(Base::Commit(oid.clone()))).expect("commit").map(|found| { return String::from(found.as_str()); }),
        Some(oid)
    );
    assert!(base_object(&capture(Base::Commit(String::from("nope")))).is_err());
}

/// A settled convergence counts its changed passes; every other ending counts none.
#[test]
fn only_a_settled_convergence_counts_passes() {
    assert_eq!(settled_passes(&Convergence::Settled { changed_passes: 3 }), 3);
    assert_eq!(settled_passes(&Convergence::Settled { changed_passes: 0 }), 0);
    assert_eq!(settled_passes(&Convergence::Blocked), 0);
    assert_eq!(settled_passes(&Convergence::FixCycle), 0);
    assert_eq!(settled_passes(&Convergence::FixPassLimit), 0);
}

/// Either `-a` or `--include` makes the capture list the staged worktree changes.
#[test]
fn a_capture_lists_worktree_changes_for_all_or_include() {
    assert!(stages_worktree(true, false));
    assert!(stages_worktree(false, true));
    assert!(stages_worktree(true, true));
    assert!(!stages_worktree(false, false));
}

/// A path that is not UTF-8 reaches Git byte for byte in its literal, rooted pathspec.
#[cfg(unix)]
#[test]
fn the_candidate_request_keeps_the_bytes_of_a_name_that_is_not_utf8() {
    use std::os::unix::ffi::OsStrExt;
    let request: CandidateRequest = staged_at_base(None, &[b"caf\xe9.txt".to_vec()]);
    let CandidateRequest::StagedAtBase { pathspecs, .. } = request else {
        panic!("expected a staged request");
    };
    assert_eq!(pathspecs.len(), 1);
    assert_eq!(pathspecs[0].as_bytes(), b":(top,literal)caf\xe9.txt");
}
