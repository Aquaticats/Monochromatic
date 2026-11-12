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
    CLEAN_TABLE, CleanDryRun, CleanRegion, UNREADABLE_CLEAN_DELETION, clean_changes_worktree, clean_deleted_paths_from_answer,
    clean_dry_run_arguments, clean_dry_run_arguments_keeping_nested_repositories,
    clean_dry_run_arguments_without_exclusions, parse_clean_region, paths_only_the_exclusions_add,
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
    let run_in: &std::path::Path = std::path::Path::new("/work/here");
    for spelling in [
        vec!["-f", "-X", "--exclude=notes.txt"],
        vec!["-f", "-X", "--exclude", "notes.txt"],
        vec!["-f", "-X", "-e", "notes.txt"],
        vec!["-f", "-X", "-enotes.txt"],
        vec!["-fX", "--excl", "notes.txt"],
    ] {
        let region: Vec<std::ffi::OsString> = os_arguments(spelling.as_slice());
        let query: CleanDryRun = clean_dry_run_arguments(&[], region.as_slice(), run_in).expect("Git accepts the region");
        assert_eq!(query.run_in, PathBuf::from("/work/here"), "{spelling:?}");
        assert_eq!(
            query.arguments,
            os_arguments(&["--no-optional-locks", "clean", "-n", "-f", "-X", "-e", "notes.txt", "--"]),
            "{spelling:?}"
        );
    }
    // Everything that changes what Git deletes is carried: the directory descent, each force, and the paths.
    let region: Vec<std::ffi::OsString> =
        os_arguments(&["-ffdX", "-i", "-q", "-e", "one", "-e", "two", "--", "sub", "other"]);
    assert_eq!(
        clean_dry_run_arguments(&[], region.as_slice(), run_in),
        Some(CleanDryRun {
            run_in: PathBuf::from("/work/here"),
            arguments: os_arguments(&[
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
            ]),
        })
    );
    // A clean that deletes untracked files as well keeps its own mode, and one that names neither has no mode.
    assert_eq!(
        clean_dry_run_arguments(&[], os_arguments(&["-fx"]).as_slice(), run_in),
        Some(CleanDryRun {
            run_in: PathBuf::from("/work/here"),
            arguments: os_arguments(&["--no-optional-locks", "clean", "-n", "-f", "-x", "--"]),
        })
    );
    assert_eq!(
        clean_dry_run_arguments(&[], os_arguments(&["-d"]).as_slice(), run_in),
        Some(CleanDryRun {
            run_in: PathBuf::from("/work/here"),
            arguments: os_arguments(&["--no-optional-locks", "clean", "-n", "-f", "-d", "--"]),
        })
    );
    // A region Git refuses is nothing to ask about.
    assert_eq!(clean_dry_run_arguments(&[], os_arguments(&["--no-such-option"]).as_slice(), run_in), None);
    // The same clean without its own patterns is the second reading: every fact is the same but each `-e` pattern,
    //  and a `-e` in a clustered short option goes with the whole cluster.
    assert_eq!(
        clean_dry_run_arguments_without_exclusions(&[], os_arguments(&["-fdX", "-e", "notes.txt"]).as_slice(), run_in),
        Some(CleanDryRun {
            run_in: PathBuf::from("/work/here"),
            arguments: os_arguments(&["--no-optional-locks", "clean", "-n", "-f", "-d", "-X", "--"]),
        })
    );
    assert_eq!(
        clean_dry_run_arguments_without_exclusions(&[], os_arguments(&["--exclude=notes.txt"]).as_slice(), run_in),
        Some(CleanDryRun {
            run_in: PathBuf::from("/work/here"),
            arguments: os_arguments(&["--no-optional-locks", "clean", "-n", "-f", "--"]),
        })
    );
    assert_eq!(
        clean_dry_run_arguments_without_exclusions(&[], os_arguments(&["--no-such-option"]).as_slice(), run_in),
        None
    );
}

/// What:
///  Every dry run carries the directory the clean runs in,
///  so it is started there and not where the caller's own `-C` chain ended,
///  and the caller's `-C` chain is left out of the arguments.
/// Why:
///  A clean a `!` alias runs starts at the top of the working tree
///  (`git.c:388` to `git.c:407`),
///  so a dry run started below that top measures a narrower subtree than the clean deletes,
///  and a repository or an untracked file above the caller is then deleted unsaved.
///  A `-C` written in front would not do: Git reads a command's prefix from the directory it was started in,
///  so `git -C <top> clean -ffdX` from a subdirectory still names the caller's own prefix
///  and refuses to remove it,
///  while the same clean started at the top names the subdirectory and removes it.
///  Every other option the caller wrote stays, in the order written,
///  because those decide which repository and which ignore rules the clean reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// cleanDryRunArguments(['-C', 'below', '--git-dir=../.git'], ['-ffdX'], '/worktree/top')
/// //   => { runIn: '/worktree/top', arguments: ['--git-dir=../.git', '--no-optional-locks', 'clean', '-n', ...] }
/// ```
#[test]
fn a_dry_run_starts_where_the_clean_runs() {
    let region: Vec<std::ffi::OsString> = os_arguments(&["-ffdX"]);
    let prefix: Vec<std::ffi::OsString> = os_arguments(&["-C", "below", "--git-dir=../.git", "-c", "core.excludesFile=local-ignore"]);
    let run_in: &std::path::Path = std::path::Path::new("/worktree/top");
    let expected_head: Vec<std::ffi::OsString> = os_arguments(&[
        "--git-dir=../.git",
        "-c",
        "core.excludesFile=local-ignore",
        "--no-optional-locks",
        "clean",
        "-n",
    ]);
    for reading in [
        clean_dry_run_arguments(prefix.as_slice(), region.as_slice(), run_in),
        clean_dry_run_arguments_without_exclusions(prefix.as_slice(), region.as_slice(), run_in),
        clean_dry_run_arguments_keeping_nested_repositories(prefix.as_slice(), region.as_slice(), run_in),
    ] {
        let query: CleanDryRun = reading.expect("Git accepts the region");
        assert_eq!(query.run_in, PathBuf::from("/worktree/top"), "{query:?}");
        assert_eq!(query.arguments[..expected_head.len()], expected_head[..], "{query:?}");
        assert!(
            !query.arguments.iter().any(|token| { return token.as_encoded_bytes() == b"-C" }),
            "the caller's directory chain is started in, not written again: {query:?}"
        );
    }
    // Every `-C` the caller wrote goes, so Git moves once and not twice.
    let twice: Vec<std::ffi::OsString> = os_arguments(&["-C", "below", "-C", "../elsewhere"]);
    let query: CleanDryRun =
        clean_dry_run_arguments(twice.as_slice(), region.as_slice(), run_in).expect("Git accepts the region");
    assert_eq!(query.run_in, PathBuf::from("/worktree/top"));
    assert!(
        !query.arguments.iter().any(|token| {
            let bytes: &[u8] = token.as_encoded_bytes();
            return bytes == b"below" || bytes == b"../elsewhere";
        }),
        "{query:?}"
    );
    // A value that spells an option is a value: only the `-C` before it goes, and the option it names stays.
    let tricky: Vec<std::ffi::OsString> = os_arguments(&["-C", "--literal-pathspecs", "--no-pager"]);
    assert_eq!(
        clean_dry_run_arguments(tricky.as_slice(), region.as_slice(), run_in),
        Some(CleanDryRun {
            run_in: PathBuf::from("/worktree/top"),
            arguments: os_arguments(&[
                "--no-pager",
                "--no-optional-locks",
                "clean",
                "-n",
                "-f",
                "-f",
                "-d",
                "-X",
                "--"
            ]),
        })
    );
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
    // The dry run is started where the value says, never where this test stands, which is what the pair is for.
    let read = |query: CleanDryRun| -> Vec<Vec<u8>> {
        let started: std::process::Output = git_output(query.run_in.as_path(), query.arguments.as_slice());
        let answer = MetadataOutput {
            success: started.status.success(),
            code: started.status.code(),
            stdout: started.stdout.clone(),
            stderr: started.stderr.clone(),
        };
        return clean_deleted_paths_from_answer(&answer)
            .unwrap_or_else(|why| { panic!("{why}: {}", String::from_utf8_lossy(started.stderr.as_slice())) });
    };
    let typed: Vec<Vec<u8>> = read(
        clean_dry_run_arguments(&[], os_arguments(&["-f", "-X", "--exclude=notes.txt"]).as_slice(), root.as_path())
            .expect("Git accepts the region"),
    );
    let otherwise: Vec<Vec<u8>> = read(
        clean_dry_run_arguments_without_exclusions(
            &[],
            os_arguments(&["-f", "-X", "--exclude=notes.txt"]).as_slice(),
            root.as_path(),
        )
        .expect("Git accepts the region"),
    );
    assert!(!typed.is_empty(), "the typed clean would delete nothing");
    assert!(otherwise.is_empty(), "the clean without the pattern named {otherwise:?}");
    assert_eq!(paths_only_the_exclusions_add(typed.as_slice(), otherwise.as_slice()), typed);
    assert!(root.join("notes.txt").exists());
    remove(directory.as_path());
}
