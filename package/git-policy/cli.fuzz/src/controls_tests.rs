//! What:
//!  Controls proving the control generators reach every way of reading a command,
//!       every control effect and every ending of the lifecycle,
//!  and fixed hard cases.
//! Why:
//!  An invariant that is never reached proves nothing;
//!  these controls count what the
//!      generators produce and run the invariants over all short inputs.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(readingsReachedBy(controlArguments)).toContain('refused');
//! ```

/// Import the generators and invariants under control,
///  and the tables they draw from.
use super::{
    FrontierSeen, MAX_CONTROL_ARGUMENTS, SEPARATOR_MARK, check_control_removal, check_frontier,
    check_separator, control_arguments, removed_tokens, separated_arguments,
};
use crate::arguments::arguments_from_bytes;
use crate::control_tables::{CONTROL_TOKENS, WRAPPER_SPELLINGS};
use git_policy_cli::policy_registry::POLICY_REGISTRY;
use git_policy_cli::wrapper_controls::{CONTROL_SPELLINGS, control_meaning, is_escaped};
use git_policy_cli::wrapper_invocation::{
    RegionReading, StrippedInvocation, strip_wrapper_controls,
};
use std::ffi::OsString;

/// Build owned arguments from text.
fn text(values: &[&str]) -> Vec<OsString> {
    let mut result: Vec<OsString> = Vec::<OsString>::new();
    for value in values {
        result.push(OsString::from(value));
    }
    return result;
}

/// The index of a token in the generator table,
///  as the byte that selects it.
fn byte_of(token: &[u8]) -> u8 {
    for (index, candidate) in CONTROL_TOKENS.iter().enumerate() {
        if *candidate == token {
            return u8::try_from(index).expect("the table fits in a byte");
        }
    }
    panic!("{token:?} is not in the generator table");
}

/// The restated spellings are exactly what the subject recognizes,
///  and the generator can produce each.
#[test]
fn restated_spellings_match_the_subject() {
    assert_eq!(WRAPPER_SPELLINGS.len(), CONTROL_SPELLINGS.len() + 1);
    for spelling in CONTROL_SPELLINGS {
        assert!(
            WRAPPER_SPELLINGS.contains(&spelling.flag.as_bytes()),
            "{}",
            spelling.flag
        );
    }
    for spelling in WRAPPER_SPELLINGS {
        assert!(CONTROL_TOKENS.contains(spelling), "{spelling:?}");
        assert_eq!(
            control_meaning(spelling).is_some(),
            *spelling != b"--no-enforce-only".as_slice(),
            "{spelling:?}"
        );
    }
    assert!(
        CONTROL_TOKENS.len() <= 256,
        "one byte must reach every token"
    );
    assert_eq!(control_arguments(&[]), Vec::<OsString>::new());
    assert_eq!(control_arguments(&[0; 200]).len(), MAX_CONTROL_ARGUMENTS);
}

/// The token difference is exact,
///  and a rewritten or reordered list is not a removal.
#[test]
fn removed_tokens_are_an_exact_difference() {
    let before: Vec<OsString> = text(&["a", "x", "b", "x", "c"]);
    assert_eq!(
        removed_tokens(before.as_slice(), text(&["a", "b", "x", "c"]).as_slice()),
        Some(text(&["x"]))
    );
    assert_eq!(
        removed_tokens(before.as_slice(), before.as_slice()),
        Some(Vec::<OsString>::new())
    );
    assert_eq!(removed_tokens(before.as_slice(), &[]), Some(before.clone()));
    for changed in [
        vec!["a", "b", "x", "d"],
        vec!["b", "a"],
        vec!["a", "x", "b", "x", "c", "c"],
        vec!["z"],
    ] {
        assert_eq!(
            removed_tokens(before.as_slice(), text(changed.as_slice()).as_slice()),
            None,
            "{changed:?}"
        );
    }
}

/// All three-token inputs hold the removal invariants and reach every reading and every effect.
#[test]
fn generated_arguments_reach_every_reading_and_effect() {
    let mut no_command: usize = 0;
    let mut table: usize = 0;
    let mut leading_only: usize = 0;
    let mut refused: usize = 0;
    let mut keep_going: usize = 0;
    let mut skip_copy: usize = 0;
    let mut commit_hatch: usize = 0;
    let mut kept_spelling: usize = 0;
    let mut escaped: Vec<usize> = vec![0; POLICY_REGISTRY.len()];
    let count: u8 = u8::try_from(CONTROL_TOKENS.len()).expect("the table fits in a byte");
    for first in 0..count {
        for second in 0..count {
            for third in 0..count {
                let arguments: Vec<OsString> = control_arguments(&[first, second, third]);
                check_control_removal(arguments.as_slice());
                let stripped: StrippedInvocation = strip_wrapper_controls(arguments.as_slice());
                match stripped.reading {
                    RegionReading::NoCommand => no_command += 1,
                    RegionReading::Table => table += 1,
                    RegionReading::LeadingOnly => leading_only += 1,
                    RegionReading::Refused(_) => refused += 1,
                }
                if stripped.controls.keep_going {
                    keep_going += 1;
                }
                if stripped.controls.skip_worktree_copy {
                    skip_copy += 1;
                }
                if stripped.controls.commit_only_escaped {
                    commit_hatch += 1;
                }
                for (index, descriptor) in POLICY_REGISTRY.iter().enumerate() {
                    if is_escaped(&stripped.controls, descriptor.id) {
                        escaped[index] += 1;
                    }
                }
                // A spelling that survives removal: a value, a path or a command word's argument.
                for token in &stripped.arguments {
                    if WRAPPER_SPELLINGS.contains(&token.as_encoded_bytes()) {
                        kept_spelling += 1;
                        break;
                    }
                }
            }
        }
    }
    for (name, reached) in [
        ("no command", no_command),
        ("table", table),
        ("leading only", leading_only),
        ("refused", refused),
        ("keep-going", keep_going),
        ("worktree-copy opt-out", skip_copy),
        ("commit hatch", commit_hatch),
        ("a spelling kept as a value or path", kept_spelling),
    ] {
        assert!(reached > 0, "{name} was never reached");
    }
    for (index, descriptor) in POLICY_REGISTRY.iter().enumerate() {
        assert!(escaped[index] > 0, "{} was never escaped", descriptor.name);
    }
}

/// Fixed hard cases:
///  values,
///  paths and near-misses stay;
///  a control in option position goes.
#[test]
fn fixed_cases_hold_the_removal_invariants() {
    for (values, expected) in [
        (
            vec!["commit", "-m", "--no-enforce-only", "--no-enforce-only"],
            vec!["commit", "-m", "--no-enforce-only"],
        ),
        (
            vec![
                "reset",
                "--no-enforce-worktree",
                "--",
                "--no-enforce-worktree",
            ],
            vec!["reset", "--", "--no-enforce-worktree"],
        ),
        (
            vec!["-C", "--cli-git-keep-going", "status"],
            vec!["-C", "--cli-git-keep-going", "status"],
        ),
        (
            vec![
                "--cli-git-keep-going",
                "-C",
                "dir",
                "--no-worktree-copy",
                "status",
            ],
            vec!["-C", "dir", "status"],
        ),
        (
            vec!["status", "--cli-git-keep-going=1", "--cli-git-keep-goin"],
            vec!["status", "--cli-git-keep-going=1", "--cli-git-keep-goin"],
        ),
        (
            vec![
                "worktree",
                "add",
                "--no-worktree-copy",
                "-b",
                "--no-worktree-copy",
                "x",
            ],
            vec!["worktree", "add", "-b", "--no-worktree-copy", "x"],
        ),
        (
            vec!["fetch", "origin", "--cli-git-keep-going"],
            vec!["fetch", "origin", "--cli-git-keep-going"],
        ),
    ] {
        let arguments: Vec<OsString> = text(values.as_slice());
        check_control_removal(arguments.as_slice());
        assert_eq!(
            strip_wrapper_controls(arguments.as_slice()).arguments,
            text(expected.as_slice()),
            "{values:?}"
        );
    }
    // Raw bytes: an argument that only starts like a control, and one that is not UTF-8.
    for data in [
        b"commit\0--no-enforce-only\xff\0-m\0x".as_slice(),
        b"\xff\xfe\0--cli-git-keep-going".as_slice(),
        b"--cli-git-keep-going\0\0".as_slice(),
        b"".as_slice(),
    ] {
        check_control_removal(arguments_from_bytes(data).as_slice());
    }
}

/// Everything from the separator on survives,
///  for every command and every two tokens around it.
#[test]
fn separated_arguments_keep_everything_after_the_separator() {
    assert_eq!(separated_arguments(&[]), None);
    assert_eq!(
        separated_arguments(&[0]),
        Some((text(&["commit", "--"]), 1))
    );
    assert_eq!(
        separated_arguments(&[1, 0, SEPARATOR_MARK, byte_of(b"--no-enforce-worktree")]),
        Some((
            text(&["add", "--cli-git-keep-going", "--", "--no-enforce-worktree"]),
            2
        ))
    );
    let mut with_spelling_after: usize = 0;
    let mut with_removal_before: usize = 0;
    let count: u8 = u8::try_from(CONTROL_TOKENS.len()).expect("the table fits in a byte");
    for command in 0..10_u8 {
        for before in 0..14_u8 {
            for after in 0..count {
                for data in [
                    vec![command, before, SEPARATOR_MARK, after],
                    vec![command, SEPARATOR_MARK, after, before],
                    vec![command, before, before, SEPARATOR_MARK, after, after],
                ] {
                    let (arguments, separator) =
                        separated_arguments(data.as_slice()).expect("non-empty input");
                    assert_eq!(arguments[separator], "--", "{data:?}");
                    check_control_removal(arguments.as_slice());
                    check_separator(arguments.as_slice(), separator);
                    let stripped: StrippedInvocation = strip_wrapper_controls(arguments.as_slice());
                    if stripped.arguments.len() < arguments.len() {
                        with_removal_before += 1;
                    }
                    for token in &arguments[separator..] {
                        if WRAPPER_SPELLINGS.contains(&token.as_encoded_bytes()) {
                            with_spelling_after += 1;
                            break;
                        }
                    }
                }
            }
        }
    }
    assert!(
        with_spelling_after > 0,
        "no spelling was placed after the separator"
    );
    assert!(
        with_removal_before > 0,
        "no control was removed before the separator"
    );
}

/// All two-token inputs at both locations hold the frontier,
///  and every ending is reached.
#[test]
fn generated_arguments_reach_every_frontier_ending() {
    let mut forwarded: usize = 0;
    let mut rejected: usize = 0;
    let mut refused: usize = 0;
    let count: u8 = u8::try_from(CONTROL_TOKENS.len()).expect("the table fits in a byte");
    for mode in 0..32_u8 {
        for first in 0..count {
            for second in 0..count {
                let arguments: Vec<OsString> = control_arguments(&[first, second]);
                match check_frontier(arguments.as_slice(), mode) {
                    FrontierSeen::Forwarded => forwarded += 1,
                    FrontierSeen::Rejected => rejected += 1,
                    FrontierSeen::Refused => refused += 1,
                }
            }
        }
    }
    assert!(forwarded > 0, "nothing was forwarded");
    assert!(rejected > 0, "nothing was rejected");
    assert!(refused > 0, "nothing was refused");
    // Fixed cases on each side of every frontier rule, at the linked worktree (mode 1) and outside (mode 0).
    for (values, mode, seen) in [
        (vec!["commit", "-m", "x"], 0, FrontierSeen::Refused),
        (vec!["commit", "-n", "-m", "x"], 0, FrontierSeen::Refused),
        (
            vec!["commit", "--dry-run", "file"],
            0,
            FrontierSeen::Forwarded,
        ),
        (vec!["commit", "--dry", "file"], 0, FrontierSeen::Forwarded),
        // A status format makes `git commit` a dry run by itself.
        (
            vec!["commit", "--short", "file"],
            0,
            FrontierSeen::Forwarded,
        ),
        (vec!["commit", "-z", "file"], 0, FrontierSeen::Forwarded),
        // Git accepts `--l` for `--long`, the shortest abbreviation of any status format (probed on Git 2.56.0).
        (vec!["commit", "--l", "file"], 0, FrontierSeen::Forwarded),
        (vec!["commit", "--dry-run", "-a"], 0, FrontierSeen::Rejected),
        (vec!["push", "origin"], 0, FrontierSeen::Refused),
        (vec!["push", "-n", "origin"], 0, FrontierSeen::Forwarded),
        (
            vec!["push", "--no-enforce-final-newline", "origin"],
            0,
            FrontierSeen::Forwarded,
        ),
        (vec!["add", "file"], 1, FrontierSeen::Refused),
        (vec!["add", "file"], 0, FrontierSeen::Forwarded),
        (vec!["add", "."], 1, FrontierSeen::Rejected),
        (
            vec!["add", "--no-enforce-final-newline", "file"],
            1,
            FrontierSeen::Forwarded,
        ),
        (vec!["worktree", "add", "x"], 1, FrontierSeen::Forwarded),
        (vec!["worktree", "add", "x"], 0, FrontierSeen::Forwarded),
        (
            vec!["worktree", "add", "--no-worktree-copy", "x"],
            1,
            FrontierSeen::Forwarded,
        ),
        (vec!["worktree", "list"], 1, FrontierSeen::Forwarded),
        (vec!["st"], 1, FrontierSeen::Forwarded),
        (vec!["st"], 0, FrontierSeen::Forwarded),
        (vec!["status"], 1, FrontierSeen::Forwarded),
        (vec!["--version"], 1, FrontierSeen::Forwarded),
    ] {
        assert_eq!(
            check_frontier(text(values.as_slice()).as_slice(), mode),
            seen,
            "{values:?} mode {mode}"
        );
    }
}
