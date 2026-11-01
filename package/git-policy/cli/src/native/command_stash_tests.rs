//! What:
//!  `git stash` dispatch and escape-hatch positions per subcommand,
//!  with real-Git
//!       controls for every subcommand table and for the top-level readings.
//! Why:
//!  A hatch spelled as a stash message or a path must be forwarded as that message or
//!      path;
//!  a hatch in option position must be found wherever the subcommand allows one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(parseStashRegion(['push', '-m', '--no-enforce-worktree']).hasEscapeHatch).toBe(false);
//! ```

/// The parser,
///  its tables,
///  the oracles and the real-Git fixture helpers.
use super::{StashRegion, StashSubcommand, parse_stash_region};
use crate::command_options::{OptionError, OptionErrorKind, OptionSpec, WrapperOccurrence};
use crate::command_stash_table::{
    STASH_APPLY_TABLE, STASH_DROP_TABLE, STASH_EMPTY_TABLE, STASH_EXPORT_TABLE, STASH_POP_TABLE,
    STASH_PUSH_TABLE, STASH_SAVE_TABLE, STASH_SHOW_TABLE, STASH_STORE_TABLE,
};
use crate::command_test_completion::{git_completion, render_completion};
use crate::command_test_support::{
    assert_table_invariants, git, git_status, os_arguments, output_text, remove,
    repository_with_tracked_file,
};
use crate::escape_hatch::WORKTREE_ENFORCEMENT_ESCAPE_HATCH;
use std::path::PathBuf;
use std::process::Output;

/// The shared hatch spelling.
const HATCH: &str = WORKTREE_ENFORCEMENT_ESCAPE_HATCH;

/// Parse a region Git accepts,
///  with no other wrapper flags.
fn region(values: &[&str]) -> StashRegion {
    return parse_stash_region(os_arguments(values).as_slice(), &[]).expect("valid region");
}

/// Parse a region Git refuses and return the refusal.
fn refusal(values: &[&str]) -> OptionError {
    return parse_stash_region(os_arguments(values).as_slice(), &[]).expect_err("refused region");
}

/// The first word selects the subcommand;
///  anything else is an assumed `push`.
#[test]
fn dispatches_like_cmd_stash() {
    for (word, subcommand) in [
        ("apply", StashSubcommand::Apply),
        ("clear", StashSubcommand::Clear),
        ("drop", StashSubcommand::Drop),
        ("pop", StashSubcommand::Pop),
        ("branch", StashSubcommand::Branch),
        ("list", StashSubcommand::List),
        ("show", StashSubcommand::Show),
        ("store", StashSubcommand::Store),
        ("create", StashSubcommand::Create),
        ("push", StashSubcommand::Push),
        ("export", StashSubcommand::Export),
        ("import", StashSubcommand::Import),
        ("save", StashSubcommand::Save),
    ] {
        assert_eq!(region(&[word]).subcommand, subcommand, "{word}");
        // A leading wrapper flag does not hide the subcommand word.
        let escaped: StashRegion = region(&[HATCH, word]);
        assert_eq!(escaped.subcommand, subcommand, "{word}");
        assert_eq!(escaped.wrapper.escape, vec![0], "{word}");
    }
    for values in [
        vec![],
        vec!["-m", "message"],
        vec!["-u"],
        vec!["--", "file.ts"],
        vec!["-m", "push"],
        vec![HATCH],
        // A word without a dash is never the single-dash typo, whatever its tail spells.
        vec!["xpush"],
        vec!["apus", "file"],
        vec!["xlist"],
    ] {
        assert_eq!(
            region(values.as_slice()).subcommand,
            StashSubcommand::AssumedPush,
            "{values:?}"
        );
    }
}

/// The hatch in a value or path position is not a hatch.
#[test]
fn reads_the_hatch_by_position_in_each_subcommand() {
    for values in [
        vec!["push", "-m", HATCH],
        vec!["push", "--message", HATCH],
        vec!["push", "--pathspec-from-file", HATCH],
        vec!["push", "-U", HATCH],
        vec!["push", "--", HATCH],
        vec!["-m", HATCH],
        vec!["-um", HATCH],
        vec!["save", "-m", HATCH],
        vec!["store", "-m", HATCH, "abc123"],
        vec!["apply", "--label-ours", HATCH],
        vec!["export", "--to-ref", HATCH],
        // `create` joins every argument into the message.
        vec!["create", HATCH],
        // An assumed push stops at the first non-option.
        vec!["-q", "file.ts", HATCH],
        vec!["clear", "x", HATCH],
    ] {
        assert!(
            region(values.as_slice()).wrapper.escape.is_empty(),
            "{values:?}"
        );
    }
    for (values, tokens) in [
        (vec!["push", HATCH, "-m", "x"], vec![1]),
        (vec!["push", "-m", HATCH, HATCH], vec![3]),
        (vec!["push", "file.ts", HATCH], vec![2]),
        (vec![HATCH, "-m", "x"], vec![0]),
        (vec!["-q", HATCH, "-m", "x"], vec![1]),
        (vec!["save", HATCH, "message"], vec![1]),
        (vec!["store", HATCH, "-m", "x", "abc123"], vec![1]),
        (vec!["list", "--format=%s", "-3", HATCH], vec![3]),
        (vec!["show", "-p", HATCH, "stash@{0}"], vec![2]),
        (vec!["apply", "--index", HATCH], vec![2]),
        (vec!["pop", HATCH], vec![1]),
        (vec!["drop", "-q", HATCH], vec![2]),
        (vec!["branch", HATCH, "name"], vec![1]),
        (vec!["import", HATCH, "rev"], vec![1]),
        (vec!["export", "--print", HATCH], vec![2]),
        (vec![HATCH, "create", HATCH], vec![0]),
    ] {
        assert_eq!(
            region(values.as_slice()).wrapper.escape,
            tokens,
            "{values:?}"
        );
    }
    let other: StashRegion = parse_stash_region(
        os_arguments(&["--cli-git-keep-going", "push", "--cli-git-keep-going"]).as_slice(),
        &[b"--cli-git-keep-going"],
    )
    .expect("valid region");
    assert_eq!(
        other.wrapper.other,
        vec![
            WrapperOccurrence { flag: 0, token: 0 },
            WrapperOccurrence { flag: 0, token: 2 }
        ]
    );
}

/// Git's refusals,
///  with token indexes relative to the whole region.
#[test]
fn reports_what_git_refuses() {
    for (values, kind, token) in [
        (vec!["-push"], OptionErrorKind::SingleDashLongOption, 0),
        (vec!["-list"], OptionErrorKind::SingleDashLongOption, 0),
        (vec!["-pus"], OptionErrorKind::SingleDashLongOption, 0),
        // Refused by the assumed-push pass, exactly as by Git's top-level pass.
        (vec!["-no-x"], OptionErrorKind::SingleDashLongOption, 0),
        (vec!["--help"], OptionErrorKind::HelpRequested, 0),
        (vec!["--help-all"], OptionErrorKind::HelpRequested, 0),
        (vec!["-h"], OptionErrorKind::HelpRequested, 0),
        (vec!["-hq"], OptionErrorKind::HelpRequested, 0),
        (vec![HATCH, "-h"], OptionErrorKind::HelpRequested, 1),
        (vec!["-x"], OptionErrorKind::UnknownOption, 0),
        (vec!["--unknown"], OptionErrorKind::UnknownOption, 0),
        (vec!["-m"], OptionErrorKind::MissingValue, 0),
        (vec!["push", "--unknown"], OptionErrorKind::UnknownOption, 1),
        (vec!["push", "-q", "-m"], OptionErrorKind::MissingValue, 2),
        (vec!["drop", "-x"], OptionErrorKind::UnknownOption, 1),
        (vec!["clear", "-q"], OptionErrorKind::UnknownOption, 1),
        (
            vec!["pop", "--index=1"],
            OptionErrorKind::UnexpectedValue,
            1,
        ),
        (
            vec!["export", "--no-print"],
            OptionErrorKind::UnknownOption,
            1,
        ),
        (
            vec!["save", "--pathspec-from-file", "x"],
            OptionErrorKind::UnknownOption,
            1,
        ),
    ] {
        assert_eq!(
            refusal(values.as_slice()),
            OptionError { kind, token },
            "{values:?}"
        );
    }
    // Two letters never spell a subcommand word: `-pu` is `--patch --include-untracked`.
    assert_eq!(region(&["-pu"]).subcommand, StashSubcommand::AssumedPush);
    // `list`, `show` and `store` keep options they do not declare.
    for values in [
        vec!["list", "--oneline"],
        vec!["show", "--stat", "-x"],
        vec!["store", "--unknown", "abc123"],
    ] {
        assert!(
            parse_stash_region(os_arguments(values.as_slice()).as_slice(), &[]).is_ok(),
            "{values:?}"
        );
    }
}

/// Every subcommand table matches what the binary reports for that subcommand.
#[test]
fn tables_match_git_stash_completion_helpers() {
    let (directory, root): (PathBuf, PathBuf) = repository_with_tracked_file("stash-tables");
    let tables: [(&str, &[OptionSpec]); 12] = [
        ("push", STASH_PUSH_TABLE),
        ("save", STASH_SAVE_TABLE),
        ("apply", STASH_APPLY_TABLE),
        ("pop", STASH_POP_TABLE),
        ("drop", STASH_DROP_TABLE),
        ("store", STASH_STORE_TABLE),
        ("show", STASH_SHOW_TABLE),
        ("export", STASH_EXPORT_TABLE),
        ("clear", STASH_EMPTY_TABLE),
        ("branch", STASH_EMPTY_TABLE),
        ("list", STASH_EMPTY_TABLE),
        ("import", STASH_EMPTY_TABLE),
    ];
    for (word, table) in tables {
        assert_table_invariants(table);
        assert_eq!(
            render_completion(table),
            git_completion(root.as_path(), &["stash", word]),
            "{word}"
        );
    }
    remove(directory.as_path());
}

/// Real Git stores the hatch text as the message,
///  refuses `-push`,
///  and refuses an assumed
/// push that meets a non-option.
#[test]
fn top_level_readings_match_git() {
    let (directory, root): (PathBuf, PathBuf) = repository_with_tracked_file("stash-readings");
    std::fs::write(root.join("tracked.txt"), b"two\n").expect("modify tracked file");
    git(root.as_path(), &["stash", "-m", HATCH]);
    assert!(output_text(&git(root.as_path(), &["stash", "list"])).contains(HATCH));
    let typo: Output = git_status(root.as_path(), &["stash", "-push"]);
    assert_eq!(typo.status.code(), Some(129));
    assert!(String::from_utf8_lossy(&typo.stderr).contains("did you mean `--push`"));
    let unexpected: Output = git_status(root.as_path(), &["stash", "-q", "file.ts"]);
    assert!(!unexpected.status.success());
    assert!(
        String::from_utf8_lossy(&unexpected.stderr).contains("'push' can't be assumed"),
        "{}",
        String::from_utf8_lossy(&unexpected.stderr)
    );
    // `create` takes option-looking words as message text and prints a commit name.
    std::fs::write(root.join("tracked.txt"), b"three\n").expect("modify tracked file");
    let created: Output = git(root.as_path(), &["stash", "create", "--unknown", "-x"]);
    assert_eq!(output_text(&created).len(), 40);
    remove(directory.as_path());
}

/// Only the subcommands that write nothing the stash guard protects are read-only.
#[test]
fn only_list_show_and_create_leave_state_alone() {
    for (subcommand, changes) in [
        (StashSubcommand::List, false),
        (StashSubcommand::Show, false),
        (StashSubcommand::Create, false),
        (StashSubcommand::Apply, true),
        (StashSubcommand::Clear, true),
        (StashSubcommand::Drop, true),
        (StashSubcommand::Pop, true),
        (StashSubcommand::Branch, true),
        (StashSubcommand::Store, true),
        (StashSubcommand::Push, true),
        (StashSubcommand::Export, true),
        (StashSubcommand::Import, true),
        (StashSubcommand::Save, true),
        (StashSubcommand::AssumedPush, true),
    ] {
        assert_eq!(super::stash_changes_state(subcommand), changes, "{subcommand:?}");
    }
}
