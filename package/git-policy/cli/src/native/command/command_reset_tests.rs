//! What:
//!  `git reset` modes in every spelling Git 2.56.0 accepts,
//!  with real-Git controls for
//!       the table and for last-mode-wins.
//! Why:
//!  A missed `--hard` spelling lets a destructive reset run in the main worktree;
//!  a
//!      mode read as destructive after a later `--soft` blocks a harmless one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(resetChangesWorktree(parseResetRegion(['--har', 'HEAD~1']))).toBe(true);
//! ```

/// The parser,
///  its table,
///  the oracles and the real-Git fixture helpers.
use super::{RESET_TABLE, ResetMode, ResetRegion, parse_reset_region, reset_changes_worktree};
use crate::command_options::OptionErrorKind;
use crate::command_test_completion::{git_completion, render_completion};
use crate::command_test_support::{
    assert_table_invariants, git, os_arguments, remove, repository_with_tracked_file,
};
use crate::escape_hatch::WORKTREE_ENFORCEMENT_ESCAPE_HATCH;
use std::path::PathBuf;

/// Parse a region Git accepts,
///  with no other wrapper flags.
fn region(values: &[&str]) -> ResetRegion {
    return parse_reset_region(os_arguments(values).as_slice(), &[]).expect("valid region");
}

/// Every abbreviation of the three worktree-changing modes;
///  `--m` is ambiguous.
#[test]
fn detects_destructive_modes_in_every_abbreviation() {
    for (spellings, mode) in [
        (vec!["--hard", "--har", "--ha", "--h"], ResetMode::Hard),
        (vec!["--merge", "--merg", "--mer", "--me"], ResetMode::Merge),
        (vec!["--keep", "--kee", "--ke", "--k"], ResetMode::Keep),
    ] {
        for spelling in spellings {
            let parsed: ResetRegion = region(&[spelling, "HEAD~1"]);
            assert_eq!(parsed.mode, Some(mode), "{spelling}");
            assert!(reset_changes_worktree(&parsed), "{spelling}");
        }
    }
    assert_eq!(
        parse_reset_region(os_arguments(&["--m"]).as_slice(), &[])
            .expect_err("refused")
            .kind,
        OptionErrorKind::AmbiguousOption
    );
}

/// Index-only forms:
///  no mode,
///  `--mixed`,
///  `--soft`,
///  patch mode and path resets.
#[test]
fn detects_index_only_forms() {
    for values in [
        vec![],
        vec!["HEAD~1"],
        vec!["--mixed", "HEAD~1"],
        vec!["--soft", "HEAD~1"],
        vec!["--mi"],
        vec!["--so"],
        vec!["-p"],
        vec!["-q", "--", "file.ts"],
        vec!["--pathspec-from-file", "--hard"],
        vec!["HEAD", "--", "--hard"],
        vec!["--recurse-submodules", "-N", "x"],
    ] {
        assert!(
            !reset_changes_worktree(&region(values.as_slice())),
            "{values:?}"
        );
    }
    assert_eq!(region(&["--mixed"]).mode, Some(ResetMode::Mixed));
    assert_eq!(region(&["--soft"]).mode, Some(ResetMode::Soft));
    assert_eq!(region(&["HEAD"]).mode, None);
}

/// Divergence:
///  the five modes write one variable,
///  so the last one decides.
#[test]
fn the_last_mode_option_decides() {
    assert_eq!(region(&["--hard", "--soft"]).mode, Some(ResetMode::Soft));
    assert!(!reset_changes_worktree(&region(&["--hard", "--soft"])));
    assert!(!reset_changes_worktree(&region(&[
        "--keep", "--merge", "--mixed"
    ])));
    assert!(reset_changes_worktree(&region(&[
        "--soft", "HEAD", "--hard"
    ])));
    assert!(reset_changes_worktree(&region(&["--mixed", "--keep"])));
}

/// The escape hatch counts in option position only.
#[test]
fn finds_the_escape_hatch_in_option_position_only() {
    let hatch: &str = WORKTREE_ENFORCEMENT_ESCAPE_HATCH;
    assert_eq!(region(&["--hard", hatch]).wrapper.escape, vec![1]);
    assert_eq!(region(&[hatch, "--hard"]).wrapper.escape, vec![0]);
    for values in [
        vec!["--pathspec-from-file", hatch],
        vec!["-U", hatch],
        vec!["--", hatch],
        vec!["--hard"],
    ] {
        assert!(
            region(values.as_slice()).wrapper.escape.is_empty(),
            "{values:?}"
        );
    }
}

/// A region Git refuses is reported as refused;
///  Git then resets nothing.
#[test]
fn reports_what_git_refuses() {
    for (values, kind) in [
        (vec!["--unknown"], OptionErrorKind::UnknownOption),
        (vec!["--no-hard"], OptionErrorKind::UnknownOption),
        (vec!["--hard=1"], OptionErrorKind::UnexpectedValue),
        (vec!["--pathspec-from-file"], OptionErrorKind::MissingValue),
        (vec!["-hard"], OptionErrorKind::SingleDashLongOption),
        (vec!["-x"], OptionErrorKind::UnknownOption),
    ] {
        assert_eq!(
            parse_reset_region(os_arguments(values.as_slice()).as_slice(), &[])
                .expect_err("refused")
                .kind,
            kind,
            "{values:?}"
        );
    }
}

/// The copied table matches the binary;
///  `--har` discards a worktree change and a later
/// `--soft` keeps it.
#[test]
fn table_abbreviation_and_last_mode_match_git() {
    let (directory, root): (PathBuf, PathBuf) = repository_with_tracked_file("reset-table");
    assert_table_invariants(RESET_TABLE);
    assert_eq!(
        render_completion(RESET_TABLE),
        git_completion(root.as_path(), &["reset"])
    );
    let file: PathBuf = root.join("tracked.txt");
    std::fs::write(&file, b"changed\n").expect("modify tracked file");
    git(root.as_path(), &["reset", "--quiet", "--hard", "--soft"]);
    assert_eq!(std::fs::read(&file).expect("read"), b"changed\n");
    git(root.as_path(), &["reset", "--quiet", "--soft", "--har"]);
    assert_eq!(std::fs::read(&file).expect("read"), b"one\n");
    remove(directory.as_path());
}
