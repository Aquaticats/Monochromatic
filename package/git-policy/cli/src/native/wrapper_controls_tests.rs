//! What:
//!  Control spellings,
//!  their meanings,
//!  and removal from the arguments before the subcommand.
//! Why:
//!  A control left before the subcommand makes Git's global-option reader report an
//!      unknown option,
//!  after which every policy leaves the command alone;
//!  a control
//!      removed from a value position would change what Git is asked to do.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(stripGlobalControls(['--cli-git-keep-going', 'status'], controls)).toEqual(['status']);
//! ```

/// The table,
///  the scan and the argument builder.
use super::{
    CONTROL_SPELLINGS, ControlMeaning, Controls, KEEP_GOING_FLAG, control_flags, control_meaning,
    is_escaped, no_controls, record_control, strip_global_controls,
};
use crate::command_test_support::os_arguments;
use crate::escape_hatch::WORKTREE_COPY_ESCAPE_HATCH;
use crate::policy_registry::{POLICY_REGISTRY, PolicyId};
use std::ffi::OsString;

/// Strip one argument list and return the kept arguments with the recorded controls.
fn stripped(values: &[&str]) -> (Vec<OsString>, Controls) {
    let mut controls: Controls = no_controls();
    let kept: Vec<OsString> = strip_global_controls(os_arguments(values).as_slice(), &mut controls);
    return (kept, controls);
}

/// The record of an invocation that only escaped the given policies.
fn escaping(policies: &[PolicyId]) -> Controls {
    let mut controls: Controls = no_controls();
    controls.escaped = policies.to_vec();
    return controls;
}

/// Every shipped policy is escaped by `--no-enforce-` followed by its registry name,
///  and by nothing shorter or longer.
#[test]
fn every_shipped_policy_has_its_enforce_spelling() {
    for descriptor in POLICY_REGISTRY {
        let flag: String = format!("--no-enforce-{}", descriptor.name);
        assert_eq!(
            control_meaning(flag.as_bytes()),
            Some(ControlMeaning::Escape(descriptor.id)),
            "{flag}"
        );
        assert_eq!(control_meaning(&flag.as_bytes()[1..]), None, "{flag}");
        assert_eq!(
            control_meaning(format!("{flag}=1").as_bytes()),
            None,
            "{flag}"
        );
    }
    // One spelling per policy, three older spellings, and the two flags that escape no policy.
    assert_eq!(CONTROL_SPELLINGS.len(), POLICY_REGISTRY.len() + 5);
}

/// The older spellings,
///  the keep-going flag and the worktree-copy opt-out keep their exact meanings.
#[test]
fn fixed_spellings_have_their_meaning() {
    for (flag, meaning) in [
        (KEEP_GOING_FLAG, ControlMeaning::KeepGoing),
        (WORKTREE_COPY_ESCAPE_HATCH, ControlMeaning::SkipWorktreeCopy),
        (
            "--no-enforce-worktree",
            ControlMeaning::Escape(PolicyId::LinkedWorktreeOnly),
        ),
        (
            "--no-enforce-worktree-branch",
            ControlMeaning::Escape(PolicyId::BranchWorktreeOnly),
        ),
        (
            "--no-enforce-bulk-add",
            ControlMeaning::Escape(PolicyId::AddExplicit),
        ),
    ] {
        assert_eq!(control_meaning(flag.as_bytes()), Some(meaning), "{flag}");
    }
    assert_eq!(KEEP_GOING_FLAG, "--cli-git-keep-going");
    assert_eq!(WORKTREE_COPY_ESCAPE_HATCH, "--no-worktree-copy");
    for other in [
        "",
        "--",
        "--no-enforce-",
        "--no-enforce-only",
        "--cli-git-keep",
        "--cli-git-keep-going-on",
        "-cli-git-keep-going",
        "--no-pager",
    ] {
        assert_eq!(control_meaning(other.as_bytes()), None, "{other}");
    }
}

/// The byte list handed to option-table readers follows the table row for row.
#[test]
fn control_flags_follow_the_table() {
    let flags: Vec<&'static [u8]> = control_flags();
    assert_eq!(flags.len(), CONTROL_SPELLINGS.len());
    for (index, spelling) in CONTROL_SPELLINGS.iter().enumerate() {
        assert_eq!(flags[index], spelling.flag.as_bytes(), "{}", spelling.flag);
    }
}

/// Each meaning sets its own field;
///  a repeated escape lists its policy once,
///  in first-use order.
#[test]
fn recorded_controls_are_set_once() {
    let mut controls: Controls = no_controls();
    assert_eq!(
        controls,
        Controls {
            keep_going: false,
            escaped: Vec::<PolicyId>::new(),
            commit_only_escaped: false,
            skip_worktree_copy: false,
        }
    );
    record_control(&mut controls, ControlMeaning::Escape(PolicyId::AddExplicit));
    assert_eq!(controls, escaping(&[PolicyId::AddExplicit]));
    record_control(&mut controls, ControlMeaning::Escape(PolicyId::RequireRoot));
    record_control(&mut controls, ControlMeaning::Escape(PolicyId::AddExplicit));
    assert_eq!(
        controls,
        escaping(&[PolicyId::AddExplicit, PolicyId::RequireRoot])
    );
    assert!(is_escaped(&controls, PolicyId::RequireRoot));
    assert!(!is_escaped(&controls, PolicyId::FinalNewline));
    let mut keep: Controls = no_controls();
    record_control(&mut keep, ControlMeaning::KeepGoing);
    assert!(keep.keep_going);
    assert!(!keep.skip_worktree_copy);
    let mut copy: Controls = no_controls();
    record_control(&mut copy, ControlMeaning::SkipWorktreeCopy);
    assert!(copy.skip_worktree_copy);
    assert!(!copy.keep_going);
}

/// Controls before the subcommand are removed and recorded,
///  between and after Git's own global options.
#[test]
fn controls_before_the_subcommand_are_removed() {
    let mut keep_going: Controls = no_controls();
    keep_going.keep_going = true;
    assert_eq!(
        stripped(&["--cli-git-keep-going", "status"]),
        (os_arguments(&["status"]), keep_going.clone())
    );
    assert_eq!(
        stripped(&["--cli-git-keep-going", "--cli-git-keep-going", "status"]),
        (os_arguments(&["status"]), keep_going.clone())
    );
    let mut both: Controls = escaping(&[PolicyId::RequireRoot]);
    both.keep_going = true;
    assert_eq!(
        stripped(&[
            "-C",
            "dir",
            "--no-enforce-require-root",
            "--no-pager",
            "--cli-git-keep-going",
            "-c",
            "a.b=c",
            "status",
        ]),
        (
            os_arguments(&["-C", "dir", "--no-pager", "-c", "a.b=c", "status"]),
            both
        )
    );
    assert_eq!(
        stripped(&["--git-dir=x", "--no-enforce-worktree", "add", "."]),
        (
            os_arguments(&["--git-dir=x", "add", "."]),
            escaping(&[PolicyId::LinkedWorktreeOnly])
        )
    );
    // Only controls: nothing is left, and Git then prints its usage.
    assert_eq!(
        stripped(&["--cli-git-keep-going"]),
        (Vec::<OsString>::new(), keep_going)
    );
    for spelling in CONTROL_SPELLINGS {
        let (kept, _) = stripped(&[spelling.flag, "status"]);
        assert_eq!(kept, os_arguments(&["status"]), "{}", spelling.flag);
    }
}

/// A control-looking value,
///  a control after the subcommand,
///  and anything behind an option Git refuses stay.
#[test]
fn tokens_outside_global_option_position_are_kept() {
    for values in [
        vec![],
        vec!["status"],
        vec!["-C", "--cli-git-keep-going", "status"],
        vec!["-c", "--no-enforce-require-root", "status"],
        vec!["--git-dir", "--no-enforce-worktree", "status"],
        vec!["--namespace", "--cli-git-keep-going", "status"],
        vec!["status", "--cli-git-keep-going"],
        vec!["--no-such-option", "--cli-git-keep-going", "status"],
        vec!["--version", "--cli-git-keep-going"],
        vec!["--help", "--no-enforce-require-root"],
        vec!["--exec-path", "--cli-git-keep-going"],
        vec!["-C"],
        vec!["--cli-git-keep-going=1", "status"],
    ] {
        assert_eq!(
            stripped(values.as_slice()),
            (os_arguments(values.as_slice()), no_controls()),
            "{values:?}"
        );
    }
    // A control ahead of an option Git refuses is still removed; Git then reports only its own error.
    let (kept, controls) = stripped(&["--cli-git-keep-going", "--no-such-option", "status"]);
    assert_eq!(kept, os_arguments(&["--no-such-option", "status"]));
    assert!(controls.keep_going);
}

/// Arguments that are not UTF-8 pass through byte for byte around a removed control.
#[cfg(unix)]
#[test]
fn undecodable_arguments_are_kept_unchanged() {
    use std::os::unix::ffi::OsStringExt;
    let directory: OsString = OsString::from_vec(vec![b'd', 0xff, b'r']);
    let arguments: Vec<OsString> = vec![
        OsString::from("-C"),
        directory.clone(),
        OsString::from("--cli-git-keep-going"),
        OsString::from("status"),
    ];
    let mut controls: Controls = no_controls();
    assert_eq!(
        strip_global_controls(arguments.as_slice(), &mut controls),
        vec![OsString::from("-C"), directory, OsString::from("status")]
    );
    assert!(controls.keep_going);
}
