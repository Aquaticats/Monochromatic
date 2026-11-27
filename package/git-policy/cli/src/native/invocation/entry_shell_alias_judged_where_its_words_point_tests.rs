//! What:
//!  A bare `!git` is judged for the worktree and the repository its caller's own words name,
//!  and each control here compares that spelling with the direct spelling of the same target.
//! Why:
//!  Git appends the caller's own words to an alias value before the shell reads it
//!  (`git.c:401` to `git.c:402`),
//!  so `-c alias.x=!git x -C <another worktree> reset --hard` resets that other worktree.
//!  The guard read the caller's own directory instead,
//!  so from a clean linked worktree that spelling reset a dirty main checkout while the direct
//!  spelling was refused,
//!  and from the main checkout it was refused where Git would have reset the clean linked worktree
//!  (the alpha.50 review's M1).
//!  Comparing each alias spelling with the direct spelling of the same target keeps a row from passing
//!  on a reading that differs from Git's own,
//!  and the two directions are told apart by a direct spelling that is refused in one and runs in the
//!  other.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const command of GUARDED) expect(verdict(behindAlias(linked, ['-C', main], command))).toEqual(verdict(direct(['-C', main], command)));
//! ```

#![cfg(unix)]

/// Import the decision under test and the fixtures shared with the entry's other controls.
use super::plan_invocation;
use super::tests::{
    inputs,
    text,
};
use crate::action::Action;
use crate::test_support::{
    fixture,
    git,
    remove,
    repository,
};
use std::ffi::OsString;
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  What the planning answered about one spelling:
///  a refusal with its status and its report,
///  or a command that runs.
/// Why:
///  A row's claim is that the alias spelling gets the verdict the direct spelling gets,
///  so one value has to carry both what a person is told and whether anything runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Verdict = { kind: 'refused'; code: number; stderr: string } | { kind: 'ran' };
/// ```
#[derive(Debug, Eq, PartialEq)]
enum Verdict {
    /// The planning stopped the command, with the report a person reads.
    Refused {
        /// The status the wrapper exits with.
        code: i32,
        /// The report on standard error.
        stderr: String,
    },
    /// The command reaches Git.
    Ran,
}

/// The verdict of one planned command line.
fn verdict(action: Action) -> Verdict {
    return match action {
        Action::Exit {
            code,
            stderr,
            ..
        } => Verdict::Refused {
            code,
            stderr,
        },
        _ => Verdict::Ran,
    };
}

/// What:
///  One fixture repository whose main checkout holds an uncommitted edit
///  and whose linked worktree is clean.
/// Why:
///  The two directions of the finding are exactly those two worktrees,
///  and a row that named one of them for the wrong reason would read as right if both were clean.
struct TwoWorktrees {
    /// The fixture folder, which is where every control is started.
    root: PathBuf,
    /// The main checkout, which holds the edit.
    main: PathBuf,
    /// The linked worktree, which is clean.
    linked: PathBuf,
}

impl TwoWorktrees {
    /// The Git directory of the linked worktree, which `--git-dir` names for it.
    fn linked_git_directory(&self) -> PathBuf {
        return self.main.join(".git").join("worktrees").join("linked");
    }
}

/// Whether a worktree holds state Git would report as uncommitted.
fn has_uncommitted_state(worktree: &Path) -> bool {
    let output: std::process::Output = git(worktree, &["status", "--porcelain"]);
    assert!(output.status.success(), "the fixture's status reads");
    return !output.stdout.is_empty();
}

/// What:
///  A fixture repository with one tracked file,
///  a linked worktree of it,
///  and an uncommitted edit of that file in the main checkout alone.
/// Why:
///  Every guarded form is measured against tracked state or against the main checkout itself,
///  so the fixture holds both:
///  the main checkout is dirty,
///  the linked worktree is clean,
///  and the direct spelling of a hard reset is therefore refused in one and allowed in the other.
fn a_dirty_main_checkout_beside_a_clean_linked_worktree(name: &str) -> TwoWorktrees {
    let root: PathBuf = fixture(name);
    let main: PathBuf = repository(root.as_path(), "main");
    std::fs::write(main.join("tracked.txt"), "one\n").expect("the tracked file");
    git(main.as_path(), &["add", "tracked.txt"]);
    git(main.as_path(), &["commit", "--quiet", "--message=content"]);
    git(
        main.as_path(),
        &["worktree", "add", "--quiet", "-b", "topic", "../linked"],
    );
    std::fs::write(main.join("tracked.txt"), "two\n").expect("the uncommitted edit");
    let linked: PathBuf = root.join("linked");
    return TwoWorktrees {
        root,
        main,
        linked,
    };
}

/// What:
///  Every command word the guard judges,
///  each in a form that measures the target worktree or its state.
/// Why:
///  The finding is about the worktree a spelling is judged for,
///  and every guarded word reads that worktree through its own question;
///  a table keeps one form's answer from standing for the others.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const GUARDED = [['reset', '--hard'], ['stash', 'drop'], ['rm', '-f', 'tracked.txt'], ...];
/// ```
const GUARDED_COMMANDS: &[&[&str]] = &[
    &["reset", "--hard"],
    &["reset", "--merge"],
    &["clean", "-f"],
    &["clean", "-fdx"],
    &["rm", "-f", "tracked.txt"],
    &["mv", "-f", "tracked.txt", "moved.txt"],
    &["checkout", "--", "tracked.txt"],
    &["switch", "--force", "main"],
    &["restore", "tracked.txt"],
    &["checkout-index", "-f", "--", "tracked.txt"],
    &["read-tree", "-u", "--reset", "HEAD"],
    &["stash"],
    &["stash", "drop"],
    &["merge", "--abort"],
    &["rebase", "--abort"],
    &["worktree", "remove", "linked"],
    &["branch", "-d", "topic"],
];

/// What:
///  The words that name one worktree's repository,
///  in the spellings the finding names:
///  `-C`,
///  `--git-dir` and `--work-tree` with their value in the next token,
///  and the same two with an `=`.
/// Why:
///  Git reads all three before it runs a command,
///  and a reading that served only `-C` would leave the other two judged for the caller's worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const target = [[ '-C', top ], [ '--git-dir', gitDir, '--work-tree', top ], [ `--git-dir=${gitDir}`, `--work-tree=${top}` ]];
/// ```
fn spellings_of_the_target(
    git_directory: &Path,
    worktree: &Path,
) -> Vec<Vec<OsString>> {
    let top: OsString = worktree.as_os_str().to_os_string();
    let directory: OsString = git_directory.as_os_str().to_os_string();
    return vec![
        vec![OsString::from("-C"), top.clone()],
        vec![
            OsString::from("--git-dir"),
            directory.clone(),
            OsString::from("--work-tree"),
            top.clone(),
        ],
        vec![
            OsString::from(format!("--git-dir={}", git_directory.display())),
            OsString::from(format!("--work-tree={}", worktree.display())),
        ],
    ];
}

/// What:
///  The words one control types:
///  where the caller stands,
///  the bare `!git` alias,
///  the target that alias's own words name,
///  and the command.
/// Why:
///  Git hands the words after the alias name to the nested Git,
///  so the target stands after the alias word exactly as a person writes it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // ['-C', linked, '-c', 'alias.fixturebare=!git', 'fixturebare', '-C', main, 'reset', '--hard']
/// ```
fn behind_the_alias(
    caller: &Path,
    target: &[OsString],
    command: &[&str],
) -> Vec<OsString> {
    let mut words: Vec<OsString> = vec![OsString::from("-C"), caller.as_os_str().to_os_string()];
    words.push(OsString::from("-c"));
    words.push(OsString::from("alias.fixturebare=!git"));
    words.push(OsString::from("fixturebare"));
    words.extend(target.iter().cloned());
    words.extend(text(command));
    return words;
}

/// The same target and command without the alias.
fn directly(
    target: &[OsString],
    command: &[&str],
) -> Vec<OsString> {
    let mut words: Vec<OsString> = target.to_vec();
    words.extend(text(command));
    return words;
}

/// What:
///  A bare `!git` whose caller's words name another worktree is judged for that worktree,
///  in every guarded form and every spelling of the target.
/// Why:
///  The alpha.50 review measured the reminder:
///  from a clean linked worktree,
///  `-c alias.x=!git x -C <main> reset --hard` reset a dirty main checkout while the direct spelling
///  was refused,
///  and the slot refusals took their records from the caller's repository by the same construction.
///  Each row is the alias spelling against the direct spelling of the one target it names,
///  and the fixture's two worktrees are asserted before the rows read it:
///  the direct spelling of a hard reset is refused for the dirty main checkout and runs for the
///  clean linked worktree.
#[test]
fn a_bare_git_is_judged_for_the_worktree_its_callers_words_name() {
    let fixture_worktrees: TwoWorktrees =
        a_dirty_main_checkout_beside_a_clean_linked_worktree("guard-alias-another-worktree");
    assert!(
        has_uncommitted_state(fixture_worktrees.main.as_path()),
        "the main checkout holds the edit the linked worktree does not"
    );
    assert!(
        !has_uncommitted_state(fixture_worktrees.linked.as_path()),
        "the linked worktree is clean"
    );
    let resolution: crate::real_git::ResolutionInputs = inputs(fixture_worktrees.root.as_path());
    let main_targets: Vec<Vec<OsString>> = spellings_of_the_target(
        fixture_worktrees.main.join(".git").as_path(),
        fixture_worktrees.main.as_path(),
    );
    let linked_targets: Vec<Vec<OsString>> = spellings_of_the_target(
        fixture_worktrees.linked_git_directory().as_path(),
        fixture_worktrees.linked.as_path(),
    );
    // The two directions are told apart before any row uses them.
    assert!(
        matches!(
            verdict(plan_invocation(
                directly(&main_targets[0], &["reset", "--hard"]).as_slice(),
                &[],
                &resolution,
            )),
            Verdict::Refused { .. }
        ),
        "a hard reset of the dirty main checkout is refused"
    );
    assert_eq!(
        verdict(plan_invocation(
            directly(&linked_targets[0], &["reset", "--hard"]).as_slice(),
            &[],
            &resolution,
        )),
        Verdict::Ran,
        "a hard reset of the clean linked worktree runs"
    );
    for command in GUARDED_COMMANDS {
        // Toward the dirty main checkout, from the clean linked worktree.
        for target in &main_targets {
            let behind: Verdict = verdict(plan_invocation(
                behind_the_alias(
                    fixture_worktrees.linked.as_path(),
                    target.as_slice(),
                    command,
                )
                .as_slice(),
                &[],
                &resolution,
            ));
            let direct: Verdict = verdict(plan_invocation(
                directly(target.as_slice(), command).as_slice(),
                &[],
                &resolution,
            ));
            assert_eq!(
                behind,
                direct,
                "the alias spelling of {command:?} toward {} is judged as the direct spelling is",
                fixture_worktrees.main.display()
            );
        }
        // Toward the clean linked worktree, from the dirty main checkout.
        for target in &linked_targets {
            let behind: Verdict = verdict(plan_invocation(
                behind_the_alias(fixture_worktrees.main.as_path(), target.as_slice(), command)
                    .as_slice(),
                &[],
                &resolution,
            ));
            let direct: Verdict = verdict(plan_invocation(
                directly(target.as_slice(), command).as_slice(),
                &[],
                &resolution,
            ));
            assert_eq!(
                behind,
                direct,
                "the alias spelling of {command:?} toward {} is judged as the direct spelling is",
                fixture_worktrees.linked.display()
            );
        }
    }
    remove(fixture_worktrees.root.as_path());
}
