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
use super::{
    CLEAN_READ_NEVER_ANSWERED, CLEAN_TABLE, CleanRegion, UNREADABLE_CLEAN_DELETION, clean_changes_worktree,
    clean_deleted_paths_from_answer, clean_dry_run_arguments, clean_dry_run_arguments_without_exclusions,
    parse_clean_region, paths_only_the_exclusions_add,
};
use crate::command_options::OptionErrorKind;
use crate::command_test_completion::{git_completion, render_completion};
use crate::command_test_support::{
    assert_table_invariants, git, git_output, git_with_input, os_arguments, remove,
    repository_with_tracked_file,
};
use crate::git_metadata::MetadataOutput;
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

/// The dry-run query is built from the facts and never from the spelled region,
/// so no interactive menu and
///  no quiet flag can reach it,
///  and every exclusion spelling arrives as its own pattern.
#[test]
fn the_dry_run_query_carries_the_facts_and_no_menu() {
    let prefix: Vec<std::ffi::OsString> = os_arguments(&["-C", "dir"]);
    for spelling in [
        vec!["-f", "-X", "--exclude=notes.txt"],
        vec!["-f", "-X", "--exclude", "notes.txt"],
        vec!["-f", "-X", "-e", "notes.txt"],
        vec!["-f", "-X", "-enotes.txt"],
        vec!["-fX", "--excl", "notes.txt"],
    ] {
        let region: Vec<std::ffi::OsString> = os_arguments(spelling.as_slice());
        let query: Vec<std::ffi::OsString> =
            clean_dry_run_arguments(prefix.as_slice(), region.as_slice()).expect("Git accepts the region");
        assert_eq!(
            query,
            os_arguments(&[
                "-C",
                "dir",
                "--no-optional-locks",
                "clean",
                "-n",
                "-f",
                "-X",
                "-e",
                "notes.txt",
                "--"
            ]),
            "{spelling:?}"
        );
    }
    // Everything that changes what Git deletes is carried: the directory descent, each force, and the paths.
    let region: Vec<std::ffi::OsString> =
        os_arguments(&["-ffdX", "-i", "-q", "-e", "one", "-e", "two", "--", "sub", "other"]);
    assert_eq!(
        clean_dry_run_arguments(&[], region.as_slice()),
        Some(os_arguments(&[
            "--no-optional-locks",
            "clean",
            "-n",
            "-f",
            "-f",
            "-d",
            "-X",
            "-e",
            "one",
            "-e",
            "two",
            "--",
            "sub",
            "other"
        ]))
    );
    // A clean that deletes untracked files as well keeps its own mode, and one that names neither has no mode.
    assert_eq!(
        clean_dry_run_arguments(&[], os_arguments(&["-fx"]).as_slice()),
        Some(os_arguments(&["--no-optional-locks", "clean", "-n", "-f", "-x", "--"]))
    );
    assert_eq!(
        clean_dry_run_arguments(&[], os_arguments(&["-d"]).as_slice()),
        Some(os_arguments(&["--no-optional-locks", "clean", "-n", "-f", "-d", "--"]))
    );
    // Both ignored modes together are Git's own refusal, so the rebuilt reading carries both and meets it too: a reading
    //  that dropped one succeeded where the command the person typed cannot, and the clean then moved a repository for a
    //  command Git never ran.
    assert_eq!(
        clean_dry_run_arguments(&[], os_arguments(&["-f", "-f", "-d", "-x", "-X"]).as_slice()),
        Some(os_arguments(&["--no-optional-locks", "clean", "-n", "-f", "-f", "-d", "-x", "-X", "--"]))
    );
    assert_eq!(
        clean_dry_run_arguments(&[], os_arguments(&["-Xfd", "-x"]).as_slice()),
        Some(os_arguments(&["--no-optional-locks", "clean", "-n", "-f", "-d", "-x", "-X", "--"]))
    );
    // A region Git refuses is nothing to ask about.
    assert_eq!(clean_dry_run_arguments(&[], os_arguments(&["--no-such-option"]).as_slice()), None);
    // The same clean without its own patterns is the second reading: every fact is the same but each `-e` pattern,
    //  and a `-e` in a clustered short option goes with the whole cluster.
    assert_eq!(
        clean_dry_run_arguments_without_exclusions(&[], os_arguments(&["-fdX", "-e", "notes.txt"]).as_slice()),
        Some(os_arguments(&["--no-optional-locks", "clean", "-n", "-f", "-d", "-X", "--"]))
    );
    assert_eq!(
        clean_dry_run_arguments_without_exclusions(&[], os_arguments(&["--exclude=notes.txt"]).as_slice()),
        Some(os_arguments(&["--no-optional-locks", "clean", "-n", "-f", "--"]))
    );
    assert_eq!(clean_dry_run_arguments_without_exclusions(&[], os_arguments(&["--no-such-option"]).as_slice()), None);
}

/// What:
///  A dry run that never answered is told apart from Git's own refusal of the clean.
/// Why:
///  Git's refusal means nothing would be deleted,
/// so the clean can run and Git says why;
///  a read that ended without an exit status says nothing at all,
///  and a clean that ran on that assumption deleted a nested repository no copy held.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // expect(cleanDeletedPathsFromAnswer({ success: false, code: null })).toMatch(/^Git never answered/)
/// ```
#[test]
fn a_dry_run_that_never_answered_is_not_gits_refusal() {
    let killed = MetadataOutput {
        success: false,
        code: None,
        stdout: Vec::<u8>::new(),
        stderr: Vec::<u8>::new(),
    };
    let failure: String = clean_deleted_paths_from_answer(&killed).expect_err("a read that ended without an answer");
    assert_eq!(CLEAN_READ_NEVER_ANSWERED, "Git never answered what the clean would delete:");
    assert!(failure.starts_with(CLEAN_READ_NEVER_ANSWERED), "{failure}");
    assert!(!failure.starts_with(UNREADABLE_CLEAN_DELETION), "{failure}");
}

/// A dry run that prints nothing would delete nothing,
/// it names each path it would delete as Git printed it,
/// and one Git refuses is an error rather than that answer.
#[test]
fn a_silent_dry_run_deletes_nothing() {
    let answer = |success: bool, stdout: &[u8], stderr: &[u8]| -> MetadataOutput {
        return MetadataOutput {
            success,
            code: Some(if success { 0 } else { 128 }),
            stdout: stdout.to_vec(),
            stderr: stderr.to_vec(),
        };
    };
    assert_eq!(clean_deleted_paths_from_answer(&answer(true, b"", b"")), Ok(Vec::<Vec<u8>>::new()));
    assert_eq!(clean_deleted_paths_from_answer(&answer(true, b"\n", b"")), Ok(Vec::<Vec<u8>>::new()));
    assert_eq!(
        clean_deleted_paths_from_answer(&answer(true, b"Would remove notes.txt\nWould remove ignored/\n", b"")),
        Ok(vec![b"Would remove notes.txt".to_vec(), b"Would remove ignored/".to_vec()])
    );
    assert_eq!(
        clean_deleted_paths_from_answer(&answer(false, b"", b"fatal: clean.requireForce is true\n")),
        Err(String::from(
            "Git could not read what the clean would delete: fatal: clean.requireForce is true"
        ))
    );
    // The policy tells Git's refusal of the clean from a reading that never answered by these words,
    //  so the sentence must begin with them.
    assert_eq!(UNREADABLE_CLEAN_DELETION, "Git could not read what the clean would delete:");
    let failure: String = clean_deleted_paths_from_answer(&answer(false, b"", b"fatal: refused\n")).expect_err("a refusal");
    assert!(failure.starts_with(UNREADABLE_CLEAN_DELETION), "{failure}");
}

/// Only the paths the typed answer names and the answer without the patterns does not are what those patterns add,
/// and the answer keeps the order of the dry run that named them.
#[test]
fn only_the_paths_the_patterns_add_are_reported() {
    let typed: Vec<Vec<u8>> = vec![b"Would remove .env".to_vec(), b"Would remove notes.txt".to_vec()];
    let otherwise: Vec<Vec<u8>> = vec![b"Would remove .env".to_vec()];
    assert_eq!(
        paths_only_the_exclusions_add(typed.as_slice(), otherwise.as_slice()),
        vec![b"Would remove notes.txt".to_vec()]
    );
    assert_eq!(paths_only_the_exclusions_add(typed.as_slice(), typed.as_slice()), Vec::<Vec<u8>>::new());
    assert_eq!(
        paths_only_the_exclusions_add(Vec::<Vec<u8>>::new().as_slice(), otherwise.as_slice()),
        Vec::<Vec<u8>>::new()
    );
}

/// Real Git:
///  the dry run prints the file an `-e` pattern makes this clean delete,
/// while the status calls the same file untracked and not ignored,
/// and the same clean without the pattern deletes nothing.
#[test]
fn the_dry_run_sees_the_exclusion_the_status_cannot() {
    let (directory, root): (PathBuf, PathBuf) = repository_with_tracked_file("clean-exclude-dry-run");
    std::fs::write(root.join("notes.txt"), b"the person notes\n").expect("write untracked file");
    let status = git_output(root.as_path(), os_arguments(&["status", "--porcelain=v1", "-unormal"]).as_slice());
    assert_eq!(String::from_utf8_lossy(status.stdout.as_slice()), "?? notes.txt\n");
    let read = |query: Vec<std::ffi::OsString>| -> Vec<Vec<u8>> {
        let dry_run = git_output(root.as_path(), query.as_slice());
        let answer = MetadataOutput {
            success: dry_run.status.success(),
            code: dry_run.status.code(),
            stdout: dry_run.stdout.clone(),
            stderr: dry_run.stderr.clone(),
        };
        return clean_deleted_paths_from_answer(&answer)
            .unwrap_or_else(|why| { panic!("{why}: {}", String::from_utf8_lossy(dry_run.stderr.as_slice())) });
    };
    let typed: Vec<Vec<u8>> = read(
        clean_dry_run_arguments(&[], os_arguments(&["-f", "-X", "--exclude=notes.txt"]).as_slice())
            .expect("Git accepts the region"),
    );
    let otherwise: Vec<Vec<u8>> = read(
        clean_dry_run_arguments_without_exclusions(&[], os_arguments(&["-f", "-X", "--exclude=notes.txt"]).as_slice())
            .expect("Git accepts the region"),
    );
    assert!(!typed.is_empty(), "the typed clean would delete nothing");
    assert!(otherwise.is_empty(), "the clean without the pattern named {otherwise:?}");
    assert_eq!(paths_only_the_exclusions_add(typed.as_slice(), otherwise.as_slice()), typed);
    assert!(root.join("notes.txt").exists());
    remove(directory.as_path());
}
