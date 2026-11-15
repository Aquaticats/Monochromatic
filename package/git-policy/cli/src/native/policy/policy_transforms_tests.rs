//! What:
//!  The fixed transforms from scripted facts:
//!  what each inserts,
//!  what commit-only
//!       rejects,
//!  and which repository fact it asked for.
//! Why:
//!  A transform that silently skips its insertion weakens every push or commit;
//!  one
//!      that asks Git for a fact it does not need slows every command.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect((await applyFixedTransforms({ args: ['push', 'origin'] })).args).toEqual(['push', '--atomic', 'origin']);
//! ```

/// The stage under test,
///  its result and the scripted facts.
use super::{COMMIT_ONLY_CORE, TransformResult, apply_fixed_transforms};
use crate::command_test_support::os_arguments;
use crate::diagnostics::EngineFailureCode;
use crate::policy_events::PolicyEvent;
use crate::policy_test_support::{ScriptedFacts, scripted_facts};
use crate::rule_commit_index::IndexVsHead;
use crate::rule_commit_only_message::{ALL_FLAG_MESSAGE, NO_PATHSPEC_MESSAGE};
use crate::rule_commit_sequencer::SequencerState;
/// The handover of a pathspec file's bytes, the facts it asks, and the copy it looks for.
use crate::command_test_support::{git, remove, repository_with_tracked_file};
use crate::repo_read::repository_facts::{GitFacts, git_facts};
use crate::repository_facts::RepositoryFacts;
use crate::test_support::{REAL_GIT, isolated_overlay};
use std::path::{Path, PathBuf};

/// Apply the transforms without the commit hatch and return the result with the facts asked for.
fn transform(values: &[&str], facts: ScriptedFacts) -> (TransformResult, Vec<String>) {
    let mut scripted: ScriptedFacts = facts;
    let result: TransformResult =
        apply_fixed_transforms(os_arguments(values).as_slice(), os_arguments(values).as_slice(), None, false, false, &mut scripted);
    return (result, scripted.asked);
}

/// A complete result that forwards the given arguments with no event.
fn forwards(values: &[&str]) -> TransformResult {
    return TransformResult {
        arguments: os_arguments(values),
        events: Vec::<PolicyEvent>::new(),
        complete: true,
        notices: Vec::<String>::new(),
    };
}

/// A complete result that keeps the input and carries one commit-only rejection.
fn rejects(values: &[&str], code: &'static str, message: &str) -> TransformResult {
    return TransformResult {
        arguments: os_arguments(values),
        events: vec![PolicyEvent::CoreFinding {
            core_id: "commit-only",
            code,
            message: String::from(message),
        }],
        complete: true,
        notices: Vec::<String>::new(),
    };
}

/// What:
///  The bytes of a pathspec file the guard read are what the words Git receives name.
/// Why:
///  Git would otherwise open the file a second time,
///  and a pipe has one writer:
///  a wrapped `restore --pathspec-from-file=<fifo>` waited for a second writer until the run killed
///  it, while direct Git answered (the seventh review, reproduced here).
///  The copy exists only when this invocation read that file,
///  so a command run with the guard's opt-out, and a file the guard never read, are forwarded
///  exactly as the person typed them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// handThePathspecFileToGit(['restore', '--pathspec-from-file=list.txt'], facts) === ['restore', `--pathspec-from-file=${copy}`];
/// ```
#[test]
fn the_bytes_of_a_pathspec_file_the_guard_read_are_what_git_is_given() {
    let (directory, repository): (PathBuf, PathBuf) = repository_with_tracked_file("transform-pathspec-handover");
    let mut facts: GitFacts = git_facts(
        Path::new(REAL_GIT),
        &[std::ffi::OsString::from("-C"), repository.clone().into_os_string()],
        isolated_overlay().as_slice(),
    );
    facts.location().expect("the repository's location");
    let scratch: PathBuf = repository.join(".git").join(crate::scratch_location::REPOSITORY_SCRATCH_NAME);
    std::fs::create_dir_all(scratch.as_path()).expect("the private scratch directory");
    let copy: PathBuf = crate::repository_facts_written_paths::a_pathspec_copy_the_guard_hands_to_git(scratch.as_path(), b"list.txt");
    std::fs::write(copy.as_path(), b"a.txt\n").expect("the copy the guard makes");
    // The stage itself does it: the attached spelling is rewritten in its own token.
    let attached: Vec<std::ffi::OsString> = os_arguments(&["restore", "--source=HEAD", "--pathspec-from-file=list.txt"]);
    let handed: TransformResult = apply_fixed_transforms(
        attached.as_slice(),
        attached.as_slice(),
        None,
        false,
        false,
        &mut facts,
    );
    let mut wanted: Vec<u8> = Vec::from(b"--pathspec-from-file=".as_slice());
    wanted.extend_from_slice(copy.as_os_str().as_encoded_bytes());
    assert_eq!(
        handed.arguments,
        os_arguments(&["restore", "--source=HEAD", &String::from_utf8_lossy(wanted.as_slice())]),
        "the words Git receives name the bytes the guard read"
    );
    // The separated spelling has its own token replaced, and nothing else changes.
    let detached: Vec<std::ffi::OsString> = os_arguments(&["restore", "--pathspec-from-file", "list.txt", "--", "a.txt"]);
    let handed_detached: Vec<std::ffi::OsString> = super::hand_the_pathspec_file_to_git(detached.as_slice(), &mut facts);
    let mut expected: Vec<std::ffi::OsString> = os_arguments(&["restore", "--pathspec-from-file"]);
    expected.push(copy.clone().into_os_string());
    expected.extend(os_arguments(&["--", "a.txt"]));
    assert_eq!(handed_detached, expected);
    // A file this invocation did not read is left to Git, which is what an opt-out run needs.
    let other: Vec<std::ffi::OsString> = os_arguments(&["restore", "--pathspec-from-file=missing.txt"]);
    assert_eq!(super::hand_the_pathspec_file_to_git(other.as_slice(), &mut facts), other);
    // A command with no pathspec file at all is untouched.
    let plain: Vec<std::ffi::OsString> = os_arguments(&["restore", "--", "a.txt"]);
    assert_eq!(super::hand_the_pathspec_file_to_git(plain.as_slice(), &mut facts), plain);
    git(repository.as_path(), &["status", "--porcelain"]);
    remove(directory.as_path());
}

/// Facts that fail every question,
///  to prove a command asked for nothing.
fn unanswerable() -> ScriptedFacts {
    let mut facts: ScriptedFacts = scripted_facts();
    facts.location = Err(String::from("unused"));
    facts.index = Err(String::from("no index answer"));
    facts.sequencer = Err(String::from("no sequencer answer"));
    facts.remote_guess = Err(String::from("unused"));
    return facts;
}

/// Each transform inserts its tokens at its own place and asks Git for nothing.
#[test]
fn insertions_need_no_repository_fact() {
    assert_eq!(COMMIT_ONLY_CORE, "commit-only");
    for (values, forwarded) in [
        (
            vec!["push", "origin", "main"],
            vec!["push", "--atomic", "origin", "main"],
        ),
        (
            vec!["-C", "dir", "push"],
            vec!["-C", "dir", "push", "--atomic"],
        ),
        (
            vec!["push", "--no-atomic", "origin"],
            vec!["push", "--no-atomic", "origin"],
        ),
        (
            vec!["status"],
            vec!["-c", "advice.statusHints=false", "status"],
        ),
        (
            vec!["-C", "dir", "status", "--short"],
            vec![
                "-C",
                "dir",
                "-c",
                "advice.statusHints=false",
                "status",
                "--short",
            ],
        ),
        (
            vec!["-c", "advice.statusHints=true", "status"],
            vec!["-c", "advice.statusHints=true", "status"],
        ),
        (
            vec!["commit", "-m", "x", "file"],
            vec!["commit", "-o", "-m", "x", "file"],
        ),
        (
            vec!["commit", "--no-only", "-m", "x"],
            vec!["commit", "--no-only", "-m", "x"],
        ),
        (
            vec!["commit", "-o", "-m", "x", "file"],
            vec!["commit", "-o", "-m", "x", "file"],
        ),
        (
            vec!["checkout", "topic"],
            vec!["checkout", "--no-overwrite-ignore", "topic"],
        ),
        (
            vec!["-C", "dir", "switch", "--detach", "HEAD~1"],
            vec![
                "-C",
                "dir",
                "switch",
                "--no-overwrite-ignore",
                "--detach",
                "HEAD~1",
            ],
        ),
        (
            vec!["checkout", "--overwrite-ignore", "topic"],
            vec!["checkout", "--overwrite-ignore", "topic"],
        ),
        (vec!["add", "file"], vec!["add", "file"]),
        (vec![], vec![]),
        (vec!["--version"], vec!["--version"]),
        // Regions Git itself refuses are left exactly as they are.
        (
            vec!["push", "--no-such-option"],
            vec!["push", "--no-such-option"],
        ),
        (
            vec!["commit", "--no-such-option"],
            vec!["commit", "--no-such-option"],
        ),
    ] {
        assert_eq!(
            transform(values.as_slice(), unanswerable()),
            (forwards(forwarded.as_slice()), Vec::<String>::new()),
            "{values:?}"
        );
    }
}

/// `-a` is rejected from the arguments alone;
///  the input is kept and nothing is asked.
#[test]
fn commit_all_is_rejected_without_a_fact() {
    assert_eq!(
        transform(&["commit", "-a", "-m", "x"], unanswerable()),
        (
            rejects(&["commit", "-a", "-m", "x"], "all-flag", ALL_FLAG_MESSAGE),
            Vec::<String>::new()
        )
    );
}

/// A pathless commit asks whether an operation awaits its conclusion,
///  and nothing else.
#[test]
fn pathless_commit_asks_for_the_sequencer_state() {
    let asked: Vec<String> = vec![String::from("sequencer")];
    assert_eq!(
        transform(&["commit", "-m", "x"], scripted_facts()),
        (
            rejects(
                &["commit", "-m", "x"],
                "pathspec-required",
                NO_PATHSPEC_MESSAGE
            ),
            asked.clone()
        )
    );
    let mut concluding: ScriptedFacts = scripted_facts();
    concluding.sequencer = Ok(SequencerState::InProgress);
    assert_eq!(
        transform(&["commit", "-m", "x"], concluding),
        (forwards(&["commit", "-m", "x"]), asked.clone())
    );
    let (failed, failed_asked) = transform(&["commit", "-m", "x"], unanswerable());
    assert_eq!(failed_asked, asked);
    assert_eq!(
        failed,
        TransformResult {
            arguments: os_arguments(&["commit", "-m", "x"]),
            events: vec![PolicyEvent::EngineFailure {
                code: EngineFailureCode::CoreIncomplete,
                hint: crate::policy_events::EngineFailureHint::OfCode,
                message: String::from("no sequencer answer"),
                trigger: None,
                policy: None,
                path: None,
            }],
            complete: false,
            notices: Vec::<String>::new(),
        }
    );
}

/// A pathless amend asks whether the index differs from `HEAD`,
///  and nothing else.
#[test]
fn pathless_amend_asks_for_the_index_state() {
    let asked: Vec<String> = vec![String::from("index")];
    for state in [IndexVsHead::Matches, IndexVsHead::Unknown] {
        let mut facts: ScriptedFacts = scripted_facts();
        facts.index = Ok(state);
        assert_eq!(
            transform(&["commit", "--amend", "--no-edit"], facts),
            (
                forwards(&["commit", "-o", "--amend", "--no-edit"]),
                asked.clone()
            ),
            "{state:?}"
        );
    }
    let mut dirty: ScriptedFacts = scripted_facts();
    dirty.index = Ok(IndexVsHead::Differs);
    let (rejected, rejected_asked) = transform(&["commit", "--amend", "--no-edit"], dirty);
    assert_eq!(rejected_asked, asked);
    assert_eq!(
        rejected.arguments,
        os_arguments(&["commit", "--amend", "--no-edit"])
    );
    assert!(rejected.complete);
    assert!(matches!(
        rejected.events.as_slice(),
        [PolicyEvent::CoreFinding { core_id: "commit-only", code: "staged-changes-ignored", message }]
            if message.contains("git commit --amend without pathspecs")
    ));
    let (failed, failed_asked) = transform(&["commit", "--amend", "--no-edit"], unanswerable());
    assert_eq!(failed_asked, asked);
    assert!(!failed.complete);
    assert!(matches!(
        failed.events.as_slice(),
        [PolicyEvent::EngineFailure { code: EngineFailureCode::CoreIncomplete, message, .. }]
            if message == "no index answer"
    ));
    assert_eq!(
        failed.arguments,
        os_arguments(&["commit", "--amend", "--no-edit"])
    );
}

/// With the commit hatch the transform is skipped:
///  nothing is inserted,
///  rejected or asked.
#[test]
fn the_commit_hatch_skips_the_transform() {
    for values in [
        vec!["commit", "-a", "-m", "x"],
        vec!["commit", "-m", "x"],
        vec!["commit", "--amend"],
        vec!["commit", "-m", "x", "file"],
    ] {
        let mut facts: ScriptedFacts = unanswerable();
        assert_eq!(
            apply_fixed_transforms(os_arguments(values.as_slice()).as_slice(), os_arguments(values.as_slice()).as_slice(), None, true, false, &mut facts),
            forwards(values.as_slice()),
            "{values:?}"
        );
        assert_eq!(facts.asked, Vec::<String>::new(), "{values:?}");
    }
    // The hatch belongs to commit-only: the other transforms still run.
    let mut facts: ScriptedFacts = unanswerable();
    assert_eq!(
        apply_fixed_transforms(os_arguments(&["push"]).as_slice(), os_arguments(&["push"]).as_slice(), None, true, false, &mut facts),
        forwards(&["push", "--atomic"])
    );
    assert_eq!(
        apply_fixed_transforms(os_arguments(&["status"]).as_slice(), os_arguments(&["status"]).as_slice(), None, true, false, &mut facts),
        forwards(&["-c", "advice.statusHints=false", "status"])
    );
}
