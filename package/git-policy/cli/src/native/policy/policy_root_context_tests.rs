//! What:
//!  Controls for the root-context check over real candidates.
//! Why:
//!  Only a top-level `CONTEXT.md` that enters the index is forbidden;
//!  a nested one and
//!      a removal must pass,
//!  and unreadable candidates must fail rather than pass.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(await check(['CONTEXT.md'])).toEqual([{ code: 'root-context-forbidden', path: 'CONTEXT.md' }]);
//! ```
#![cfg(unix)]

/// Import the module under test.
use super::{ROOT_CONTEXT_CODE, ROOT_CONTEXT_MESSAGE, check_root_context};
use crate::candidate_prediction::CandidateRequest;
use crate::diagnostics::EngineFailureCode;
/// The event form of a pathname.
use crate::event_path::EventPath;
use crate::policy_content::{ContentState, LifecycleContent};
use crate::policy_engine::{PolicyFinding, PolicyOutcome};
use crate::policy_test_support::{ScriptedFacts, scripted_facts};
use crate::test_support::{fixture, git, remove, repository};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// The check's outcome for `git add` with these arguments in `repo`.
fn add_outcome(repo: &Path, values: &[&str]) -> PolicyOutcome {
    let mut facts: ScriptedFacts = scripted_facts();
    facts.candidates_repository = Some(repo.to_path_buf());
    let mut region: Vec<OsString> = Vec::new();
    for value in values {
        region.push(OsString::from(value));
    }
    return check_root_context(
        &mut ContentState::new(),
        &LifecycleContent::Requested(CandidateRequest::Add(region)),
        &mut facts,
    );
}

/// A top-level `CONTEXT.md` that is added or changed is one finding;
///  a nested one,
///  a
/// removal and other files are not.
#[test]
fn only_a_top_level_context_file_entering_the_index_is_forbidden() {
    let root: PathBuf = fixture("root-context");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let forbidden: PolicyOutcome = PolicyOutcome::Findings(vec![PolicyFinding {
        code: ROOT_CONTEXT_CODE,
        message: String::from(ROOT_CONTEXT_MESSAGE),
        path: Some(EventPath::from_git_bytes("CONTEXT.md".as_bytes())),
        location: None,
        fix_available: false,
    }]);
    let nothing: PolicyOutcome = PolicyOutcome::Findings(Vec::new());
    std::fs::write(repo.join("CONTEXT.md"), b"context\n").expect("context");
    std::fs::create_dir(repo.join("nested")).expect("nested");
    std::fs::write(repo.join("nested/CONTEXT.md"), b"nested\n").expect("nested context");
    std::fs::write(repo.join("other.md"), b"other\n").expect("other");
    assert_eq!(add_outcome(repo.as_path(), &["CONTEXT.md"]), forbidden);
    assert_eq!(
        add_outcome(repo.as_path(), &["other.md", "CONTEXT.md"]),
        forbidden
    );
    assert_eq!(add_outcome(repo.as_path(), &["nested/CONTEXT.md"]), nothing);
    assert_eq!(add_outcome(repo.as_path(), &["other.md"]), nothing);
    // Committed, then changed: still forbidden. Removed: allowed.
    git(repo.as_path(), &["add", "CONTEXT.md"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=context"]);
    std::fs::write(repo.join("CONTEXT.md"), b"changed\n").expect("change");
    assert_eq!(add_outcome(repo.as_path(), &["CONTEXT.md"]), forbidden);
    std::fs::remove_file(repo.join("CONTEXT.md")).expect("remove");
    assert_eq!(add_outcome(repo.as_path(), &["CONTEXT.md"]), nothing);
    // Without candidates nothing is prepared; unpreparable candidates fail the policy.
    let mut idle: ScriptedFacts = scripted_facts();
    assert_eq!(
        check_root_context(&mut ContentState::new(), &LifecycleContent::None, &mut idle),
        nothing
    );
    assert_eq!(idle.asked, Vec::<String>::new());
    assert_eq!(
        check_root_context(
            &mut ContentState::new(),
            &LifecycleContent::Requested(CandidateRequest::Add(Vec::new())),
            &mut scripted_facts()
        ),
        PolicyOutcome::Failed {
            code: EngineFailureCode::ContentUnavailable,
            message: String::from("the scripted facts prepare no candidates"),
            path: None,
        }
    );
    remove(root.as_path());
}
