//! What:
//!  `git clean` facts in every spelling Git 2.56.0 accepts,
//!  with real-Git controls for
//!       the table,
//!  for clustered dry runs and for an interactive dry run.
//! Why:
//!  A missed dry-run spelling blocks a harmless listing;
//!  a dry run read where Git sees
//!      none lets a deleting clean run in the main worktree.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(cleanChangesWorktree(parseCleanRegion(['-ndX']))).toBe(false);
//! ```

/// The parser,
///  its table,
///  the oracles and the real-Git fixture helpers.
use super::{CLEAN_TABLE, CleanRegion, clean_changes_worktree, parse_clean_region};
use crate::command_options::OptionErrorKind;
use crate::command_test_completion::{git_completion, render_completion};
use crate::command_test_support::{
    assert_table_invariants, git, git_with_input, os_arguments, remove,
    repository_with_tracked_file,
};
use crate::escape_hatch::WORKTREE_ENFORCEMENT_ESCAPE_HATCH;
use std::path::PathBuf;
use std::process::Output;

/// Parse a region Git accepts,
///  with no other wrapper flags.
fn region(values: &[&str]) -> CleanRegion {
    return parse_clean_region(os_arguments(values).as_slice(), &[]).expect("valid region");
}

/// Dry runs in long,
///  abbreviated,
///  short and clustered forms delete nothing.
#[test]
fn detects_dry_runs_in_every_spelling() {
    for values in [
        vec!["-n"],
        vec!["--dry-run"],
        vec!["--dry"],
        vec!["--d"],
        vec!["-ndX"],
        vec!["-dn"],
        vec!["-fdn", "path"],
        vec!["--no-dry-run", "-n"],
        vec!["-n", "-e", "--no-dry-run"],
        vec!["-n", "--exclude=--no-dry-run"],
        vec!["-ne", "--no-dry-run"],
        vec!["-n", "--", "--no-dry-run"],
    ] {
        assert!(
            !clean_changes_worktree(&region(values.as_slice())),
            "{values:?}"
        );
    }
}

/// Without a final dry run the clean can delete,
///  including when `-n` is only a value.
#[test]
fn detects_deleting_forms() {
    for values in [
        vec![],
        vec!["-f"],
        vec!["-fd"],
        vec!["-fdx", "path"],
        vec!["-n", "--no-dry-run"],
        vec!["--dry-run", "--no-dry"],
        vec!["-e", "-n"],
        vec!["--exclude", "--dry-run"],
        vec!["--excl", "-n"],
        vec!["-en"],
        vec!["-fe", "-n"],
        vec!["--", "-n"],
        vec!["-i"],
        vec!["--interactive"],
    ] {
        assert!(
            clean_changes_worktree(&region(values.as_slice())),
            "{values:?}"
        );
    }
}

/// Interactive state is reported with Git's last-wins order;
///  it does not decide deletion.
#[test]
fn reports_the_final_interactive_state() {
    for values in [
        vec!["-i"],
        vec!["--interactive"],
        vec!["--i"],
        vec!["-ni"],
        vec!["--no-interactive", "-i"],
    ] {
        assert!(region(values.as_slice()).interactive, "{values:?}");
    }
    for values in [vec![], vec!["-i", "--no-interactive"], vec!["-e", "-i"]] {
        assert!(!region(values.as_slice()).interactive, "{values:?}");
    }
    // Divergence: an interactive dry run deletes nothing.
    assert!(!clean_changes_worktree(&region(&["-i", "-n"])));
}

/// The escape hatch counts in option position only.
#[test]
fn finds_the_escape_hatch_in_option_position_only() {
    let hatch: &str = WORKTREE_ENFORCEMENT_ESCAPE_HATCH;
    assert_eq!(region(&["-fd", hatch]).wrapper.escape, vec![1]);
    for values in [
        vec!["-e", hatch],
        vec!["--exclude", hatch],
        vec!["--exc", hatch],
        vec!["-fe", hatch],
        vec!["--", hatch],
        vec!["-fd"],
    ] {
        assert!(
            region(values.as_slice()).wrapper.escape.is_empty(),
            "{values:?}"
        );
    }
}

/// A region Git refuses is reported as refused;
///  Git then deletes nothing.
#[test]
fn reports_what_git_refuses() {
    for (values, kind) in [
        (vec!["--unknown"], OptionErrorKind::UnknownOption),
        (vec!["-z"], OptionErrorKind::UnknownOption),
        (vec!["-e"], OptionErrorKind::MissingValue),
        (vec!["--no-exclude"], OptionErrorKind::UnknownOption),
        (vec!["-dry"], OptionErrorKind::SingleDashLongOption),
        (vec!["--dry-run=1"], OptionErrorKind::UnexpectedValue),
    ] {
        assert_eq!(
            parse_clean_region(os_arguments(values.as_slice()).as_slice(), &[])
                .expect_err("refused")
                .kind,
            kind,
            "{values:?}"
        );
    }
}

/// The copied table matches the binary;
///  a clustered or abbreviated dry run keeps the file,
/// `-e -n` deletes it,
///  and an interactive dry run keeps it even after choosing "clean".
#[test]
fn table_and_dry_run_readings_match_git() {
    let (directory, root): (PathBuf, PathBuf) = repository_with_tracked_file("clean-table");
    assert_table_invariants(CLEAN_TABLE);
    assert_eq!(
        render_completion(CLEAN_TABLE),
        git_completion(root.as_path(), &["clean"])
    );
    let stray: PathBuf = root.join("stray.txt");
    std::fs::write(&stray, b"stray\n").expect("write untracked file");
    for arguments in [
        vec!["clean", "-ndX"],
        vec!["clean", "--dry"],
        vec!["clean", "-dn"],
    ] {
        git(root.as_path(), arguments.as_slice());
        assert!(stray.exists(), "{arguments:?}");
    }
    // `1` chooses "clean" in the interactive menu; under `-n` nothing is removed.
    let menu: Output = git_with_input(
        root.as_path(),
        os_arguments(&["clean", "-i", "-n"]).as_slice(),
        b"1\n",
    );
    assert!(menu.status.success());
    assert!(String::from_utf8_lossy(&menu.stdout).contains("Would remove stray.txt"));
    assert!(stray.exists());
    // `-n` as the value of `-e` is an exclude pattern, so this clean deletes.
    git(root.as_path(), &["clean", "-f", "-e", "-n"]);
    assert!(!stray.exists());
    remove(directory.as_path());
}
