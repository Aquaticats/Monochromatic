//! What:
//!  Plans of invocations whose real Git comes from the commit service's record because `PATH` offers none.
//! Why:
//!  The coordinator picked this fallback under the owner's delegation,
//! open to the owner's veto
//!  (`doc/planning/cli-git-no-self-invocation.md`).
//! Its notice must lead the standard error of every plan that starts
//!  Git,
//! once per command,
//! and a stop must stay a stop when the record is unusable.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const plan = planInvocation(['--version'], env, { ...inputs, path: '', serviceUnit });
//! // expect(plan.stderr.startsWith('monochromatic-git-policy-cli: no real Git follows this wrapper on PATH')).toBe(true);
//! ```
#![cfg(unix)]

/// The decision under test,
///  the fixtures shared with the other plan controls,
///  and the notice helper.
use super::plan_invocation;
use super::tests::{inputs, stopped, text};
use crate::action::{Action, with_leading_notice};
use crate::push_stacking::STACKED_PUSH_MARKER;
use crate::real_git::ResolutionInputs;
use crate::service_provision::{ProvisionHost, SERVICE_UNIT_NAME, unit_texts};
use crate::test_support::{REAL_GIT, fixture, remove};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// The words every notice of this fallback starts with.
const NOTICE_LEAD: &str = "monochromatic-git-policy-cli: no real Git follows this wrapper on PATH";

/// What:
///  Inputs whose `PATH` holds no Git and whose unit file `root/unit` names `git`.
/// Why:
///  The image's real Git is outside `PATH`,
/// as in the reported case,
/// and only the record leads to it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const withRecord = (root, git) => ({ ...inputs(root), path: 'empty-directory', serviceUnit: writeUnit(root, git) });
/// ```
fn with_record(root: &Path, git: &Path) -> ResolutionInputs {
    let mut resolution: ResolutionInputs = inputs(root);
    resolution.path = OsString::from("empty-directory");
    resolution.service_unit = Some(root.join("unit"));
    let host: ProvisionHost = ProvisionHost {
        runtime_dir: root.to_path_buf(),
        user_unit_dir: None,
        bus: PathBuf::from("/nonexistent/bus"),
        service_binary: PathBuf::from("/opt/tool/monochromatic-git-policy-cli"),
        git: git.to_path_buf(),
        instance: String::from("x"),
        git_dir: std::path::PathBuf::new(),
        retirement_wait: std::time::Duration::ZERO,
    };
    for (name, unit) in unit_texts(&host) {
        if name == SERVICE_UNIT_NAME {
            std::fs::write(root.join("unit"), unit).expect("unit file");
        }
    }
    return resolution;
}

/// The standard-error text of a plan that carries one.
fn stderr_of(action: &Action) -> &str {
    return match action {
        Action::Forward { stderr, .. }
        | Action::ForwardAnnounced { stderr, .. }
        | Action::ForwardCoordinated { stderr, .. }
        | Action::ForwardSettled { stderr, .. }
        | Action::ForwardPush { stderr, .. }
        | Action::Exit { stderr, .. }
        | Action::ExitBytes { stderr, .. } => stderr.as_str(),
    };
}

/// A version query,
/// a plain forward and an announced help each start the recorded Git and lead with the notice.
#[test]
fn a_recorded_real_git_is_planned_with_the_notice_leading_standard_error() {
    let root: PathBuf = fixture("entry-fallback");
    let resolution: ResolutionInputs = with_record(root.as_path(), Path::new(REAL_GIT));
    for arguments in [vec!["--version"], vec!["--exec-path"], vec!["--help"]] {
        let plan: Action = plan_invocation(text(&arguments).as_slice(), &[], &resolution);
        match &plan {
            Action::Forward { real_git, .. } | Action::ForwardAnnounced { real_git, .. } => {
                assert_eq!(real_git, Path::new(REAL_GIT), "{arguments:?}");
            }
            other => panic!("{arguments:?} plans a start of the recorded Git, not {other:?}"),
        }
        let said: &str = stderr_of(&plan);
        assert!(said.starts_with(NOTICE_LEAD), "{arguments:?}: {said}");
        assert_eq!(said.matches(NOTICE_LEAD).count(), 1, "{arguments:?}: once: {said}");
    }
    remove(root.as_path());
}

/// A plan made for the merge push of an integrated race,
///  which carries the stacking marker,
///  gives no second notice:
///  the plan of the push it came from has given it.
#[test]
fn the_planned_merge_push_of_a_stacked_push_does_not_repeat_the_notice() {
    let root: PathBuf = fixture("entry-fallback-stacked");
    let resolution: ResolutionInputs = with_record(root.as_path(), Path::new(REAL_GIT));
    let environment: Vec<(OsString, OsString)> = vec![(OsString::from(STACKED_PUSH_MARKER), OsString::from("1"))];
    let plan: Action = plan_invocation(text(&["--version"]).as_slice(), environment.as_slice(), &resolution);
    assert!(!stderr_of(&plan).contains(NOTICE_LEAD), "{}", stderr_of(&plan));
    remove(root.as_path());
}

/// A management command that needs Git gives the same notice,
///  and one that needs none gives none.
#[test]
fn management_commands_give_the_notice_only_when_they_resolve_git() {
    let root: PathBuf = fixture("entry-fallback-management");
    let resolution: ResolutionInputs = with_record(root.as_path(), Path::new(REAL_GIT));
    let in_root: Vec<&str> = vec!["-C", root.to_str().expect("a text path")];
    let mut status: Vec<&str> = in_root.clone();
    status.extend(["monochromatic-git-policy-cli", "service", "status"]);
    let status_plan: Action = plan_invocation(text(&status).as_slice(), &[], &resolution);
    assert!(stderr_of(&status_plan).starts_with(NOTICE_LEAD), "{}", stderr_of(&status_plan));
    let help_plan: Action = plan_invocation(text(&["monochromatic-git-policy-cli", "--help"]).as_slice(), &[], &resolution);
    assert!(!stderr_of(&help_plan).contains(NOTICE_LEAD), "{}", stderr_of(&help_plan));
    remove(root.as_path());
}

/// A record naming a file that is not there leaves the stop,
///  with the record named and no notice.
#[test]
fn an_unusable_record_leaves_the_stop() {
    let root: PathBuf = fixture("entry-fallback-refused");
    let resolution: ResolutionInputs = with_record(root.as_path(), root.join("gone/git").as_path());
    let plan: Action = plan_invocation(text(&["--version"]).as_slice(), &[], &resolution);
    let said: String = stopped(plan);
    assert!(said.contains("recorded by the commit service unit"), "{said}");
    assert!(!said.contains(NOTICE_LEAD), "{said}");
    remove(root.as_path());
}

/// The notice goes in front of what an action already says,
///  and no notice leaves the action as it was.
#[test]
fn a_leading_notice_precedes_the_action_text_and_none_changes_nothing() {
    let exit: Action = Action::Exit { code: 3, stdout: String::from("out\n"), stderr: String::from("err\n") };
    let unchanged: Action = with_leading_notice(exit, None);
    assert_eq!(stderr_of(&unchanged), "err\n");
    let noticed: Action = with_leading_notice(unchanged, Some("notice\n"));
    assert_eq!(stderr_of(&noticed), "notice\nerr\n");
    match noticed {
        Action::Exit { code, stdout, .. } => {
            assert_eq!(code, 3);
            assert_eq!(stdout, "out\n");
        }
        other => panic!("an exit stays an exit: {other:?}"),
    }
}
