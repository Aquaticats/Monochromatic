//! What:
//!  Controls for the direct fix over real Git:
//!  the loop's three operations,
//!  what a
//!       settled,
//!  blocked,
//!  cycling or unbounded fix reports,
//!  and when files are installed.
//! Why:
//!  Only the last pass may be reported,
//!  nothing may be installed unless it exits 0,
//!      and the index must never change;
//!  each ending is driven for real or,
//!  where the
//!      shipped policies cannot reach it,
//!  through the ending function itself.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(await runDirectFix(request, checks, root)).toEqual({ events: [fixSummary], end: 'completed' });
//! ```
#![cfg(unix)]

/// Import the module under test.
use super::{
    DIRECT_FIX_CYCLE_MESSAGE, DirectFixPasses, NO_WORKTREE_MESSAGE, convergence_pass, finish_fix,
    fix_failure, run_direct_fix, settled_fix,
};
use crate::candidate_object::CandidateMode;
use crate::candidate_prediction::CandidateRequest;
use crate::command_test_support::os_arguments;
use crate::config_schema::PolicyConfig;
use crate::diagnostics::EngineFailureCode;
/// The event form of a pathname.
use crate::event_path::EventPath;
use crate::policy_checks::{ShippedChecks, shipped_checks};
use crate::policy_content::{Correction, LifecycleContent};
use crate::policy_convergence::{
    Convergence, FIX_PASS_LIMIT_MESSAGE, FixPasses, PassResult as ConvergencePass,
};
use crate::policy_engine::{StageEnd, StageRequest, Unavailable};
use crate::policy_events::{FindingEvent, PolicyEvent};
use crate::policy_pass::PassResult;
use crate::policy_registry::{PolicyId, Severity};
use crate::policy_test_support::{ScriptedFacts, scripted_facts, with_severity};
use crate::policy_trigger::Trigger;
use crate::test_support::{executable, fixture, git, remove, repository};
use crate::wrapper_controls::no_controls;
use std::ffi::OsString;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// A direct-fix request with these settings.
fn fix_request(config: PolicyConfig) -> StageRequest {
    return StageRequest {
        trigger: Trigger::DirectFix,
        config,
        controls: no_controls(),
        selected: Vec::<PolicyId>::new(),
    };
}

/// Shipped checks over the whole worktree of `repo`,
///  preparing with real Git.
fn fix_checks(repo: &Path) -> ShippedChecks<ScriptedFacts> {
    let mut facts: ScriptedFacts = scripted_facts();
    facts.candidates_repository = Some(repo.to_path_buf());
    return shipped_checks(
        facts,
        os_arguments(&[]),
        LifecycleContent::Requested(CandidateRequest::Direct(vec![OsString::from(":/")])),
        Vec::<PathBuf>::new(),
    );
}

/// The fix summary of `passes` changed passes over these paths.
fn summary(passes: u64, paths: &[&str]) -> PolicyEvent {
    let mut changed_paths: Vec<EventPath> = Vec::new();
    for path in paths {
        changed_paths.push(EventPath::from_git_bytes(path.as_bytes()));
    }
    return PolicyEvent::FixSummary {
        trigger: Trigger::DirectFix,
        passes,
        changed_paths,
    };
}

/// A pass result with these events and ending,
///  and no arguments.
fn pass_with(events: Vec<PolicyEvent>, end: StageEnd) -> PassResult {
    return PassResult {
        arguments: Vec::new(),
        events,
        end,
    };
}

/// A repository whose `a.txt` lacks its final newline,
///  `b.txt` has two,
///  `run.sh` is an
/// executable without one,
///  and `ok.txt` is canonical.
fn untidy_repository(root: &Path) -> PathBuf {
    let repo: PathBuf = repository(root, "repo");
    std::fs::write(repo.join("ok.txt"), b"ok\n").expect("ok");
    std::fs::write(repo.join("b.txt"), b"b\n\n").expect("b");
    git(repo.as_path(), &["add", "ok.txt", "b.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    std::fs::write(repo.join("a.txt"), b"a").expect("a");
    executable(repo.join("run.sh").as_path(), b"#!/bin/sh");
    return repo;
}

/// Each stage ending is read by the loop as stable,
///  proposing or failed.
#[test]
fn stage_endings_drive_the_loop() {
    assert_eq!(
        convergence_pass(StageEnd::Completed),
        ConvergencePass::Stable
    );
    assert_eq!(convergence_pass(StageEnd::Stopped), ConvergencePass::Stable);
    assert_eq!(
        convergence_pass(StageEnd::Proposed),
        ConvergencePass::Proposed
    );
    assert_eq!(convergence_pass(StageEnd::Failed), ConvergencePass::Failed);
    assert_eq!(
        convergence_pass(StageEnd::Unavailable(Unavailable::Lifecycle(
            Trigger::PostCommit
        ))),
        ConvergencePass::Failed
    );
}

/// The loop's operations on real candidates:
///  a proposing pass,
///  its application,
///  the
/// stored states,
///  and a stable pass;
///  stale proposals never reach the next pass.
#[test]
fn passes_propose_apply_and_compare_states() {
    let root: PathBuf = fixture("direct-fix-passes");
    let repo: PathBuf = untidy_repository(root.as_path());
    let request: StageRequest = fix_request(PolicyConfig::defaults());
    let mut checks: ShippedChecks<ScriptedFacts> = fix_checks(repo.as_path());
    let stale: Correction = Correction {
        path: b"stale.txt".to_vec(),
        mode: CandidateMode::Regular,
        before: Rc::from(&b"x"[..]),
        after: Rc::from(&b"y"[..]),
    };
    checks.content.propose(stale.clone());
    let mut passes: DirectFixPasses<'_, ScriptedFacts> =
        DirectFixPasses::new(&request, &mut checks);
    assert_eq!(passes.run_pass(0), ConvergencePass::Proposed);
    assert!(passes.apply(1));
    assert!(!passes.snapshots_equal(0, 1));
    assert!(passes.snapshots_equal(1, 1));
    assert_eq!(passes.run_pass(1), ConvergencePass::Stable);
    assert_eq!(
        passes.last,
        Some(pass_with(Vec::new(), StageEnd::Completed))
    );
    // A second application of no proposals stores a state equal to the previous one.
    assert!(passes.apply(2));
    assert!(passes.snapshots_equal(1, 2));
    let changed: Vec<Vec<u8>> = checks
        .content
        .install_changes()
        .into_iter()
        .map(change_path)
        .collect();
    assert_eq!(
        changed,
        vec![b"a.txt".to_vec(), b"b.txt".to_vec(), b"run.sh".to_vec()],
        "the stale proposal was dropped before the first pass"
    );
    remove(root.as_path());
}

/// The pathname of an install change.
fn change_path(change: crate::direct_fix_install::InstallChange) -> Vec<u8> {
    return change.path;
}

/// A fix that settles cleanly installs every corrected file,
///  keeps modes and the index,
/// and reports only the summary;
///  a second fix finds nothing.
#[test]
fn a_clean_fix_installs_and_reports_only_its_summary() {
    let root: PathBuf = fixture("direct-fix-clean");
    let repo: PathBuf = untidy_repository(root.as_path());
    let index_before: Vec<u8> = std::fs::read(repo.join(".git/index")).expect("index");
    let request: StageRequest = fix_request(PolicyConfig::defaults());
    let mut checks: ShippedChecks<ScriptedFacts> = fix_checks(repo.as_path());
    assert_eq!(
        run_direct_fix(&request, &mut checks, Some(repo.as_path())),
        pass_with(
            vec![summary(1, &["a.txt", "b.txt", "run.sh"])],
            StageEnd::Completed
        )
    );
    assert_eq!(std::fs::read(repo.join("a.txt")).expect("a"), b"a\n");
    assert_eq!(std::fs::read(repo.join("b.txt")).expect("b"), b"b\n");
    assert_eq!(
        std::fs::read(repo.join("run.sh")).expect("run"),
        b"#!/bin/sh\n"
    );
    let mode: u32 = std::fs::metadata(repo.join("run.sh"))
        .expect("run")
        .permissions()
        .mode();
    assert_eq!(mode & 0o100, 0o100, "{mode:o}");
    assert_eq!(
        std::fs::read(repo.join(".git/index")).expect("index"),
        index_before
    );
    let mut again: ShippedChecks<ScriptedFacts> = fix_checks(repo.as_path());
    assert_eq!(
        run_direct_fix(&request, &mut again, Some(repo.as_path())),
        pass_with(Vec::new(), StageEnd::Completed)
    );
    remove(root.as_path());
}

/// A fix with the dependent-version policy listed at warn completes its last pass and installs the
///  final-newline correction,
///  because the wired policy plans nothing for a fix with no manifest to raise.
#[test]
fn a_fix_with_the_wired_policy_listed_completes_and_installs() {
    let root: PathBuf = fixture("direct-fix-listed");
    let repo: PathBuf = untidy_repository(root.as_path());
    let listed: StageRequest = fix_request(with_severity(
        &PolicyConfig::defaults(),
        PolicyId::DependentVersionBump,
        Severity::Warn,
    ));
    let mut checks: ShippedChecks<ScriptedFacts> = fix_checks(repo.as_path());
    let result: PassResult = run_direct_fix(&listed, &mut checks, Some(repo.as_path()));
    assert_eq!(result.end, StageEnd::Completed);
    assert_eq!(std::fs::read(repo.join("a.txt")).expect("a"), b"a\n");
    remove(root.as_path());
}

/// A settled fix without a worktree,
///  or whose installation fails,
///  reports one
/// transaction failure and changes nothing.
#[test]
fn a_settled_fix_that_cannot_install_reports_one_failure() {
    let root: PathBuf = fixture("direct-fix-install-failure");
    let repo: PathBuf = untidy_repository(root.as_path());
    let request: StageRequest = fix_request(PolicyConfig::defaults());
    let mut checks: ShippedChecks<ScriptedFacts> = fix_checks(repo.as_path());
    assert_eq!(
        run_direct_fix(&request, &mut checks, None),
        fix_failure(
            &[],
            EngineFailureCode::TransactionFailed,
            NO_WORKTREE_MESSAGE
        )
    );
    // Prepared once, then the worktree file changes before the fix installs.
    let mut changing: ShippedChecks<ScriptedFacts> = fix_checks(repo.as_path());
    changing
        .content
        .prepare(&changing.candidates, &mut changing.facts)
        .expect("prepared");
    std::fs::write(repo.join("a.txt"), b"edited meanwhile").expect("edit");
    assert_eq!(
        run_direct_fix(&request, &mut changing, Some(repo.as_path())),
        fix_failure(
            &[],
            EngineFailureCode::TransactionFailed,
            "cli-git fix could not install its corrections: a selected file changed while cli-git fix ran, so no file was changed. Run cli-git fix again."
        )
    );
    assert_eq!(
        std::fs::read(repo.join("a.txt")).expect("a"),
        b"edited meanwhile"
    );
    assert_eq!(std::fs::read(repo.join("b.txt")).expect("b"), b"b\n\n");
    remove(root.as_path());
}

/// The endings the shipped policies cannot reach:
///  a cycle and a pass limit report one
/// engine failure each and drop the pass;
///  a blocked fix reports its pass;
///  a settled pass
/// with nothing corrected or a nonzero exit is reported as it is.
#[test]
fn every_ending_reports_what_the_installed_wrapper_reports() {
    let warned: PassResult = pass_with(
        vec![PolicyEvent::WarnUnsafe {
            trigger: Trigger::DirectFix,
            policy: PolicyId::FinalNewline,
        }],
        StageEnd::Completed,
    );
    let checks: ShippedChecks<ScriptedFacts> = shipped_checks(
        scripted_facts(),
        os_arguments(&["kept"]),
        LifecycleContent::None,
        Vec::<PathBuf>::new(),
    );
    let mut arguments_kept: PassResult = warned.clone();
    arguments_kept.arguments = os_arguments(&["kept"]);
    assert_eq!(
        finish_fix(Convergence::FixCycle, arguments_kept.clone(), &checks, None),
        PassResult {
            arguments: os_arguments(&["kept"]),
            events: vec![PolicyEvent::EngineFailure {
                code: EngineFailureCode::FixCycle,
                message: String::from(DIRECT_FIX_CYCLE_MESSAGE),
                trigger: Some(Trigger::DirectFix),
                policy: None,
                path: None,
            }],
            end: StageEnd::Failed,
        }
    );
    assert_eq!(
        finish_fix(Convergence::FixPassLimit, warned.clone(), &checks, None),
        fix_failure(&[], EngineFailureCode::FixPassLimit, FIX_PASS_LIMIT_MESSAGE)
    );
    assert_eq!(
        finish_fix(Convergence::Blocked, warned.clone(), &checks, None),
        warned
    );
    // Nothing corrected: the pass as it is, with no summary and no worktree needed.
    assert_eq!(
        finish_fix(
            Convergence::Settled { changed_passes: 0 },
            warned.clone(),
            &checks,
            None
        ),
        warned
    );
    let failing: PassResult = pass_with(Vec::new(), StageEnd::Failed);
    assert_eq!(settled_fix(failing.clone(), 1, &checks, None), failing);
    // An error finding in the last pass: reported as it is, exit status 1, nothing
    // installed although a file was corrected.
    let mut corrected: ShippedChecks<ScriptedFacts> = shipped_checks(
        scripted_facts(),
        os_arguments(&[]),
        LifecycleContent::None,
        Vec::<PathBuf>::new(),
    );
    corrected.content.apply(vec![Correction {
        path: b"a.txt".to_vec(),
        mode: CandidateMode::Regular,
        before: Rc::from(&b"a"[..]),
        after: Rc::from(&b"a\n"[..]),
    }]);
    let stopped: PassResult = pass_with(
        vec![PolicyEvent::Finding(FindingEvent {
            trigger: Trigger::DirectFix,
            policy: PolicyId::FinalNewline,
            severity: Severity::Error,
            code: "noncanonical-final-newline",
            message: String::from("message"),
            path: Some(EventPath::from_git_bytes("a.txt".as_bytes())),
            location: None,
            fix_available: false,
        })],
        StageEnd::Stopped,
    );
    assert_eq!(
        finish_fix(
            Convergence::Settled { changed_passes: 1 },
            stopped.clone(),
            &corrected,
            None
        ),
        stopped
    );
    // The same correction under a clean last pass needs a worktree to install into.
    assert_eq!(
        settled_fix(warned.clone(), 1, &corrected, None),
        fix_failure(
            &[],
            EngineFailureCode::TransactionFailed,
            NO_WORKTREE_MESSAGE
        )
    );
}
