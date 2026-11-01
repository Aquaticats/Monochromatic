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

/// Apply the transforms without the commit hatch and return the result with the facts asked for.
fn transform(values: &[&str], facts: ScriptedFacts) -> (TransformResult, Vec<String>) {
    let mut scripted: ScriptedFacts = facts;
    let result: TransformResult =
        apply_fixed_transforms(os_arguments(values).as_slice(), false, &mut scripted);
    return (result, scripted.asked);
}

/// A complete result that forwards the given arguments with no event.
fn forwards(values: &[&str]) -> TransformResult {
    return TransformResult {
        arguments: os_arguments(values),
        events: Vec::<PolicyEvent>::new(),
        complete: true,
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
    };
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
                message: String::from("no sequencer answer"),
                trigger: None,
                policy: None,
                path: None,
            }],
            complete: false,
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
            apply_fixed_transforms(os_arguments(values.as_slice()).as_slice(), true, &mut facts),
            forwards(values.as_slice()),
            "{values:?}"
        );
        assert_eq!(facts.asked, Vec::<String>::new(), "{values:?}");
    }
    // The hatch belongs to commit-only: the other transforms still run.
    let mut facts: ScriptedFacts = unanswerable();
    assert_eq!(
        apply_fixed_transforms(os_arguments(&["push"]).as_slice(), true, &mut facts),
        forwards(&["push", "--atomic"])
    );
    assert_eq!(
        apply_fixed_transforms(os_arguments(&["status"]).as_slice(), true, &mut facts),
        forwards(&["-c", "advice.statusHints=false", "status"])
    );
}
