//! What:
//!  `git commit` region facts:
//!  every `commit.unit.test.ts` case,
//!  then the readings
//!       where the incumbent parser and Git 2.56.0 disagree.
//! Why:
//!  The commit-only rule decides from these facts;
//!  a wrong fact is a bypass or a false
//!      rejection.
//!  Real-Git controls for the disagreements are in `command_commit_git_tests.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(parseCommitRegion(args).isDryRun).toBe(true);
//! ```

/// The parser under test,
///  the tokenizer refusal types and the shared argument builders.
use super::{COMMIT_ESCAPE_HATCH, CommitRegion, FixupKind, parse_commit_region};
use crate::command_options::{OptionError, OptionErrorKind, OptionValue, WrapperOccurrence};
use crate::command_options_query::value_bytes;
use crate::command_test_support::os_arguments;
use std::ffi::OsString;

/// Parse a region Git accepts,
///  with no other wrapper flags.
fn region(values: &[&str]) -> CommitRegion {
    return parse_commit_region(os_arguments(values).as_slice(), &[]).expect("valid region");
}

/// Parse a region Git refuses and return the refusal.
fn refusal(values: &[&str]) -> OptionError {
    return parse_commit_region(os_arguments(values).as_slice(), &[]).expect_err("refused region");
}

/// Ported:
///  the eleven "detects dry run for ..." cases.
#[test]
fn detects_dry_run_for_every_accepted_spelling() {
    for values in [
        vec!["--dry-run", "-m", "message", "file.ts"],
        vec!["--dr", "-m", "message", "file.ts"],
        vec!["--short", "-m", "message", "file.ts"],
        vec!["--sh", "file.ts"],
        vec!["--porcelain", "file.ts"],
        vec!["--por", "file.ts"],
        vec!["--long", "file.ts"],
        vec!["--l", "file.ts"],
        vec!["-z", "file.ts"],
        vec!["--null", "file.ts"],
        vec!["--nu", "file.ts"],
    ] {
        assert!(region(values.as_slice()).dry_run, "{values:?}");
    }
}

/// Ported:
///  the three "reports real commit for ..." cases.
#[test]
fn reports_real_commits() {
    for values in [
        vec!["-m", "message", "file.ts"],
        vec!["--amend", "--no-edit", "file.ts"],
        vec!["-m", "message", "--", "--dry-run"],
    ] {
        assert!(!region(values.as_slice()).dry_run, "{values:?}");
    }
}

/// Ported:
///  "detects include flag in short,
///  long,
///  and abbreviated forms" and
/// "reports no include flag for plain commits".
#[test]
fn detects_include_in_every_form() {
    for values in [
        vec!["-i", "file.ts"],
        vec!["--include", "file.ts"],
        vec!["--inc", "file.ts"],
        vec!["-im", "message", "file.ts"],
    ] {
        assert!(region(values.as_slice()).include, "{values:?}");
    }
    assert!(!region(&["-m", "message", "file.ts"]).include);
    // `--in` also prefixes `--interactive` and `--inter-hunk-context`.
    assert_eq!(refusal(&["--in"]).kind, OptionErrorKind::AmbiguousOption);
}

/// Ported:
///  "extracts pathspecs without wrapper flags or option values".
#[test]
fn extracts_pathspecs_without_wrapper_flags_or_option_values() {
    let arguments: Vec<OsString> = os_arguments(&[
        "--no-enforce-fixture/policy",
        "--author",
        "Author <author@example.invalid>",
        "-m",
        "message",
        "first.txt",
        "--",
        "--dash-path",
    ]);
    let parsed: CommitRegion =
        parse_commit_region(arguments.as_slice(), &[b"--no-enforce-fixture/policy"])
            .expect("valid region");
    assert_eq!(parsed.pathspecs, vec![5, 7]);
    assert_eq!(
        parsed.wrapper.other,
        vec![WrapperOccurrence { flag: 0, token: 0 }]
    );
    assert!(parsed.wrapper.escape.is_empty());
    // An unlisted wrapper spelling is an option Git does not know.
    assert_eq!(
        refusal(&["--no-enforce-fixture/policy", "x"]).kind,
        OptionErrorKind::UnknownOption
    );
}

/// Ported from `argv.unit.test.ts`:
///  "keeps an ordinary undeclared joined git option working".
/// With Git's whole table declared,
///  the option is known and consumes nothing.
#[test]
fn reads_every_git_option_by_its_own_arity() {
    assert_eq!(
        region(&["-m", "msg", "--untracked-files=no", "a.txt"]).pathspecs,
        vec![3]
    );
    // Divergence: a separated value after an optional-value option is a pathspec.
    assert_eq!(
        region(&["--untracked-files", "no", "-S", "key", "-u", "x"]).pathspecs,
        vec![1, 3, 5]
    );
    // Every required-value option takes the next token, including `-C` after the subcommand.
    assert_eq!(
        region(&[
            "-C",
            "HEAD~",
            "--cleanup",
            "strip",
            "-U",
            "3",
            "--trailer",
            "a: b",
            "-t",
            "tpl",
            "p"
        ])
        .pathspecs,
        vec![10]
    );
    assert!(
        region(&[
            "-q",
            "-v",
            "-n",
            "-s",
            "-e",
            "--no-verify",
            "--reset-author"
        ])
        .pathspecs
        .is_empty()
    );
}

/// Divergence:
///  Git reads every letter of a cluster,
///  so `-qa` and `-va` contain `-a`.
#[test]
fn finds_all_inside_any_cluster() {
    for values in [
        vec!["-qa"],
        vec!["-aq"],
        vec!["-vam", "msg"],
        vec!["-am", "msg"],
    ] {
        assert!(region(values.as_slice()).all, "{values:?}");
    }
    // The letter after a value-taking letter is value text, not a flag.
    for values in [vec!["-ma"], vec!["-ua"], vec!["-Sa"], vec!["-m", "-a"]] {
        assert!(!region(values.as_slice()).all, "{values:?}");
    }
    assert_eq!(region(&["-ua", "file.ts"]).pathspecs, vec![1]);
}

/// Git's last option wins;
///  the facts are final states,
///  not occurrence counts.
#[test]
fn reports_final_states() {
    assert!(!region(&["--all", "--no-all"]).all);
    assert!(region(&["--no-all", "-a"]).all);
    assert_eq!(region(&["x"]).only, None);
    assert_eq!(region(&["-o", "x"]).only, Some(true));
    assert_eq!(region(&["-om", "m", "x"]).only, Some(true));
    assert_eq!(region(&["--only", "--no-only"]).only, Some(false));
    assert_eq!(region(&["--no-only", "--on"]).only, Some(true));
    assert!(!region(&["--short", "--no-short"]).dry_run);
    assert!(!region(&["--porcelain", "--no-porcelain", "--dry-run", "--no-dry-run"]).dry_run);
    assert!(region(&["--no-short", "--long"]).dry_run);
    assert!(!region(&["-z", "--no-null"]).dry_run);
    assert!(region(&["--amend", "--allow-empty", "--interactive", "-p"]).amend);
    let mixed: CommitRegion = region(&["--am", "--no-amend", "--allow-empty", "--patch"]);
    assert!(!mixed.amend && mixed.allow_empty && mixed.patch && !mixed.interactive);
    // `--allow-empty-message` is a different option.
    assert!(!region(&["--allow-empty-message"]).allow_empty);
    assert!(region(&["--interactive"]).interactive);
    assert!(region(&["--pathspec-file-nul"]).pathspec_file_nul);
}

/// The final `--pathspec-from-file` value is reported by position,
///  whichever form it takes.
#[test]
fn reports_the_pathspec_file_value() {
    let separated: Vec<OsString> = os_arguments(&["--pathspec-from-file", "paths.txt", "x"]);
    let first: CommitRegion = parse_commit_region(separated.as_slice(), &[]).expect("valid");
    assert_eq!(
        first.pathspec_from_file,
        Some(OptionValue::Detached { token: 1 })
    );
    assert_eq!(first.pathspecs, vec![2]);
    let joined: Vec<OsString> = os_arguments(&["--pathspec-from-file=a", "--pathspec-from-file=-"]);
    let second: CommitRegion = parse_commit_region(joined.as_slice(), &[]).expect("valid");
    assert_eq!(
        value_bytes(
            joined.as_slice(),
            second.pathspec_from_file.expect("final value")
        ),
        b"-"
    );
    assert_eq!(
        region(&["--pathspec-from-file=a", "--no-pathspec-from-file"]).pathspec_from_file,
        None
    );
}

/// `--fixup` suboptions follow builtin/commit.c:1378-1411.
#[test]
fn classifies_fixup_values() {
    for (value, kind) in [
        ("HEAD", FixupKind::Plain),
        ("amend:HEAD", FixupKind::Amend),
        ("reword:HEAD~2", FixupKind::Reword),
        ("squash:HEAD", FixupKind::UnknownSuboption),
        (":HEAD", FixupKind::Plain),
        ("amend", FixupKind::Plain),
        ("am3nd:HEAD", FixupKind::Plain),
        ("", FixupKind::Plain),
    ] {
        assert_eq!(region(&["--fixup", value]).fixup, Some(kind), "{value}");
        let joined: String = format!("--fixup={value}");
        assert_eq!(region(&[joined.as_str()]).fixup, Some(kind), "{value}");
    }
    assert_eq!(region(&["--fixup=amend:HEAD", "--no-fixup"]).fixup, None);
    assert_eq!(region(&["-m", "m"]).fixup, None);
}

/// The escape hatch counts in option position only.
#[test]
fn finds_the_escape_hatch_in_option_position_only() {
    assert_eq!(
        region(&[COMMIT_ESCAPE_HATCH, "-m", "m", COMMIT_ESCAPE_HATCH])
            .wrapper
            .escape,
        vec![0, 3]
    );
    for values in [
        vec!["-m", COMMIT_ESCAPE_HATCH, "x"],
        vec!["--message", COMMIT_ESCAPE_HATCH],
        vec!["-F", COMMIT_ESCAPE_HATCH],
        vec!["x", "--", COMMIT_ESCAPE_HATCH],
    ] {
        assert!(
            region(values.as_slice()).wrapper.escape.is_empty(),
            "{values:?}"
        );
    }
    // Wrapper flags are exact: no abbreviation and no joined value.
    assert_eq!(
        refusal(&["--no-enforce-onl"]).kind,
        OptionErrorKind::UnknownOption
    );
    assert_eq!(
        refusal(&["--no-enforce-only=1"]).kind,
        OptionErrorKind::UnknownOption
    );
}

/// A region Git refuses yields the refusal,
///  never guessed facts.
#[test]
fn refuses_regions_git_refuses() {
    for (values, kind, token) in [
        (vec!["-m"], OptionErrorKind::MissingValue, 0),
        (vec!["x", "--author"], OptionErrorKind::MissingValue, 1),
        (vec!["--porcelain=v2"], OptionErrorKind::UnexpectedValue, 0),
        (vec!["--unknown"], OptionErrorKind::UnknownOption, 0),
        (vec!["-x"], OptionErrorKind::UnknownOption, 0),
        (vec!["-all"], OptionErrorKind::SingleDashLongOption, 0),
        (vec!["-amend"], OptionErrorKind::SingleDashLongOption, 0),
        (vec!["--s"], OptionErrorKind::AmbiguousOption, 0),
        (vec!["--no-unified"], OptionErrorKind::UnknownOption, 0),
        (vec!["x", "-h"], OptionErrorKind::HelpRequested, 1),
    ] {
        assert_eq!(
            refusal(values.as_slice()),
            OptionError { kind, token },
            "{values:?}"
        );
    }
}

/// Empty values,
///  a leading terminator and bytes that are not UTF-8.
#[cfg(unix)]
#[test]
fn keeps_empty_values_separators_and_non_utf8_bytes() {
    use crate::command_test_support::byte_argument;
    assert_eq!(region(&["-m", "", "--message=", "x"]).pathspecs, vec![3]);
    assert_eq!(region(&["--", "-a", "--amend"]).pathspecs, vec![1, 2]);
    assert!(!region(&["--", "-a"]).all);
    assert_eq!(region(&["--end-of-options", "-a"]).pathspecs, vec![1]);
    assert_eq!(
        region(&["-", ":/", ":(top)x", "*"]).pathspecs,
        vec![0, 1, 2, 3]
    );
    let arguments: Vec<OsString> = vec![
        byte_argument(b"-m\xff"),
        byte_argument(b"--pathspec-from-file=\xfe\xff"),
        byte_argument(b"dir/\xff.txt"),
    ];
    let parsed: CommitRegion = parse_commit_region(arguments.as_slice(), &[]).expect("valid");
    assert_eq!(parsed.pathspecs, vec![2]);
    assert_eq!(
        value_bytes(
            arguments.as_slice(),
            parsed.pathspec_from_file.expect("value")
        ),
        b"\xfe\xff"
    );
}
