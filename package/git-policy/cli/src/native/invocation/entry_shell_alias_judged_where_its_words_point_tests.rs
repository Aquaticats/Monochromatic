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
//!  Two more places the nested Git acts in reach that reading (issue 6641):
//!  the `--git-dir`, `--work-tree` and `-c` a caller types before the alias name,
//!  which Git exports to the shell and are dropped from the facts no longer,
//!  and the `GIT_DIR` Git gives the shell from a linked worktree,
//!  which makes a nested `-C` write one worktree's files with another repository's index.
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
    /// The planning stopped the command,
    /// with the report a person reads.
    Refused {
        /// The status the wrapper exits with.
        code: i32,
        /// The report on standard error.
        stderr: String,
    },
    /// The command reaches Git.
    Ran,
}

/// What:
///  Whether a verdict stopped the command before Git ran it.
/// Why:
///  A control that holds one spelling to another's verdict compares this,
///  not the refusal's words:
///  an alias spelling and the direct spelling of the same target act in two states
///  when the shell keeps a Git directory the caller's words do not name,
///  so the same refusal may name either.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isStopped = (verdict) => verdict.kind === 'refused';
/// ```
fn is_stopped(verdict: &Verdict) -> bool {
    return matches!(verdict, Verdict::Refused { .. });
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
    /// The fixture folder,
    /// which is where every control is started.
    root: PathBuf,
    /// The main checkout,
    /// which holds the edit.
    main: PathBuf,
    /// The linked worktree,
    /// which is clean.
    linked: PathBuf,
}

impl TwoWorktrees {
    /// The Git directory of the linked worktree,
    /// which `--git-dir` names for it.
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
///  A bare `!git` whose caller stands in the main checkout and whose words name the clean linked worktree
///  is judged exactly as the direct spelling of that target is,
///  in every guarded form and every spelling of the target.
/// Why:
///  The alpha.50 review measured the reminder:
///  from the main checkout,
///  `-c alias.x=!git x -C <clean worktree> reset --hard` was refused where Git would have reset the clean
///  worktree,
///  because the facts were asked of the caller and never of the words' target.
///  From the main checkout Git exports no `GIT_DIR` to the alias's shell,
///  so the nested Git finds the repository of the directory the words name and the two spellings act in one state;
///  a caller in a linked worktree does not, and the control below holds that direction.
///  The fixture's two worktrees are asserted before the rows read it:
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

/// What:
///  The options a caller's own words can carry before the alias name,
///  in every spelling that moves the place the nested Git acts on:
///  `--git-dir` and `--work-tree` with their value attached and separated,
///  and a `-c` that sets the work tree.
///  `&TwoWorktrees` borrows the fixture those options name.
/// Why:
///  Git stores a `--git-dir` or a `--work-tree` in the environment it starts the alias's shell with
///  (`git.c:215` to `:250`),
///  so the nested Git keeps the repository or the work tree those options named
///  whatever directory the command then moves to.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const placeOptions = (wt) => [['--git-dir', wt.mainGit], [`--work-tree=${wt.main}`], ['-c', `core.worktree=${wt.main}`]];
/// ```
fn options_that_name_the_place_before_the_alias(worktrees: &TwoWorktrees) -> Vec<Vec<OsString>> {
    let main_git: OsString = worktrees.main.join(".git").into_os_string();
    let main: OsString = worktrees.main.as_os_str().to_os_string();
    let linked: OsString = worktrees.linked.as_os_str().to_os_string();
    return vec![
        vec![OsString::from("--git-dir"), main_git.clone()],
        vec![OsString::from(format!(
            "--git-dir={}",
            worktrees.main.join(".git").display()
        ))],
        vec![OsString::from("--work-tree"), main.clone()],
        vec![OsString::from(format!(
            "--work-tree={}",
            worktrees.main.display()
        ))],
        vec![
            OsString::from("--git-dir"),
            main_git,
            OsString::from("--work-tree"),
            main,
        ],
        vec![OsString::from("--work-tree"), linked],
        vec![
            OsString::from("-c"),
            OsString::from(format!("core.worktree={}", worktrees.main.display())),
        ],
    ];
}

/// What:
///  The words one control types:
///  the caller's own place options,
///  the bare `!git` alias,
///  the target its own words name,
///  and the command.
///  `&[OsString]` borrows the place options in the spelling under test.
/// Why:
///  Git reads the options before the alias name itself
///  and hands the shell what they set,
///  so a control that typed them anywhere else would not be the spelling the finding names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // ['-C', caller, '--git-dir', mainGit, '-c', 'alias.fixturebare=!git', 'fixturebare', '-C', main, 'reset', '--hard']
/// ```
fn a_place_before_the_alias(
    caller: &Path,
    place_options: &[OsString],
    target: &[OsString],
    command: &[&str],
) -> Vec<OsString> {
    let mut words: Vec<OsString> = vec![OsString::from("-C"), caller.as_os_str().to_os_string()];
    words.extend(place_options.iter().cloned());
    words.push(OsString::from("-c"));
    words.push(OsString::from("alias.fixturebare=!git"));
    words.push(OsString::from("fixturebare"));
    words.extend(target.iter().cloned());
    words.extend(text(command));
    return words;
}

/// What:
///  The same place options, the same target and the same command, spelled directly,
///  for the comparison each row makes.
/// Why:
///  The wrapper sees the caller's options in both spellings and has to judge them the same way;
///  a row that only asked the alias spelling would pass on any answer at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // ['-C', caller, '--git-dir', mainGit, '-C', main, 'reset', '--hard']
/// ```
fn the_same_place_directly(
    caller: &Path,
    place_options: &[OsString],
    target: &[OsString],
    command: &[&str],
) -> Vec<OsString> {
    let mut words: Vec<OsString> = vec![OsString::from("-C"), caller.as_os_str().to_os_string()];
    words.extend(place_options.iter().cloned());
    words.extend(target.iter().cloned());
    words.extend(text(command));
    return words;
}

/// What:
///  A bare `!git` whose caller typed `--git-dir`, `--work-tree` or `-c` before the alias name
///  is refused exactly where the direct spelling of the same words is,
///  in every guarded form,
///  both against the dirty main checkout and against the clean linked worktree.
/// Why:
///  The alpha.51 review's H1:
///  `git --git-dir=<main>/.git --work-tree=<main> g -C <linked> reset --hard` typed in a linked worktree reset the
///  main checkout while alpha.50 refused it,
///  because the facts were asked with the caller's options and then replaced by the words after the alias name
///  alone,
///  which dropped the options Git exports to the shell.
///  The direct spelling of the same words was refused by both releases,
///  so this control holds the two spellings to one answer.
#[test]
fn a_bare_git_is_judged_as_the_direct_spelling_of_the_place_its_callers_options_name() {
    let fixture_worktrees: TwoWorktrees =
        a_dirty_main_checkout_beside_a_clean_linked_worktree("guard-alias-place-options");
    assert!(
        has_uncommitted_state(fixture_worktrees.main.as_path()),
        "the main checkout holds the edit the linked worktree does not"
    );
    assert!(
        !has_uncommitted_state(fixture_worktrees.linked.as_path()),
        "the linked worktree is clean"
    );
    let resolution: crate::real_git::ResolutionInputs = inputs(fixture_worktrees.root.as_path());
    let place_options: Vec<Vec<OsString>> =
        options_that_name_the_place_before_the_alias(&fixture_worktrees);
    let toward_main: Vec<OsString> = spellings_of_the_target(
        fixture_worktrees.main.join(".git").as_path(),
        fixture_worktrees.main.as_path(),
    )
    .remove(0);
    let toward_linked: Vec<OsString> = spellings_of_the_target(
        fixture_worktrees.linked_git_directory().as_path(),
        fixture_worktrees.linked.as_path(),
    )
    .remove(0);
    // Both answers are counted, so a table that refused every row, or ran every row, cannot pass it.
    let mut ran: usize = 0;
    let mut refused: usize = 0;
    // The caller stands in the main checkout: Git exports it no `GIT_DIR`, so the two spellings act in one state
    //  for every one of these place options, which is what makes the comparison below exact. A caller in a linked
    //  worktree acts in the state `a_shell_alias_is_judged_as_the_state_its_nested_git_acts_in` holds.
    for command in GUARDED_COMMANDS {
        for place in &place_options {
            for (caller, target) in [
                (fixture_worktrees.main.as_path(), &toward_main),
                (fixture_worktrees.main.as_path(), &toward_linked),
            ] {
                let behind: Verdict = verdict(plan_invocation(
                    a_place_before_the_alias(caller, place, target.as_slice(), command).as_slice(),
                    &[],
                    &resolution,
                ));
                let direct: Verdict = verdict(plan_invocation(
                    the_same_place_directly(caller, place, target.as_slice(), command).as_slice(),
                    &[],
                    &resolution,
                ));
                assert_eq!(
                    is_stopped(&behind),
                    is_stopped(&direct),
                    "the alias spelling of {command:?} with {place:?} is refused where the direct spelling is"
                );
                match direct {
                    Verdict::Ran => ran += 1,
                    Verdict::Refused {
                        ..
                    } => refused += 1,
                }
            }
        }
    }
    assert!(
        ran > 0,
        "no row of this table runs, so it says nothing about a right answer"
    );
    assert!(
        refused > 0,
        "no row of this table is refused, so it says nothing about a wrong answer"
    );
    remove(fixture_worktrees.root.as_path());
}

/// What:
///  One repository with a clean linked worktree and a dirty one,
///  which is the setting Git's exported `GIT_DIR` acts in.
/// Why:
///  Git gives the shell of a `!` alias the `GIT_DIR` of the worktree it started in
///  (`setup.c:1126` to `:1129`),
///  so the nested Git keeps the caller's index and `HEAD` while its own `-C` moves the files,
///  and a control needs both worktrees to see that.
struct ThreeWorktrees {
    /// The fixture folder,
    /// which is where every control is started.
    root: PathBuf,
    /// The main checkout,
    /// which stays clean.
    main: PathBuf,
    /// The linked worktree the words name,
    /// which stays clean.
    clean: PathBuf,
    /// The linked worktree the caller stands in,
    /// which holds uncommitted work.
    dirty: PathBuf,
}

/// What:
///  A fixture repository with one tracked file,
///  a clean linked worktree and a dirty one,
///  and an alias whose shell prints the `GIT_DIR` Git gives it.
/// Why:
///  The finding is about the Git directory Git exports,
///  so the fixture measures that export instead of assuming it,
///  and the clean worktree is the place the words name,
///  whose own state refuses nothing.
fn a_dirty_linked_worktree_beside_a_clean_one(name: &str) -> ThreeWorktrees {
    let root: PathBuf = fixture(name);
    let main: PathBuf = repository(root.as_path(), "main");
    std::fs::write(main.join("tracked.txt"), "one\n").expect("the tracked file");
    git(main.as_path(), &["add", "tracked.txt"]);
    git(main.as_path(), &["commit", "--quiet", "--message=content"]);
    git(
        main.as_path(),
        &["config", "alias.printgitdir", "!printf %s \"$GIT_DIR\""],
    );
    git(
        main.as_path(),
        &["worktree", "add", "--quiet", "-b", "clean", "../clean"],
    );
    git(
        main.as_path(),
        &["worktree", "add", "--quiet", "-b", "dirty", "../dirty"],
    );
    let dirty: PathBuf = root.join("dirty");
    std::fs::write(dirty.join("tracked.txt"), "one\nDIRTY\n").expect("the uncommitted edit");
    std::fs::write(dirty.join("staged.txt"), "staged\n").expect("the staged new file");
    git(dirty.as_path(), &["add", "staged.txt"]);
    std::fs::write(dirty.join("untracked.txt"), "untracked\n").expect("the untracked file");
    return ThreeWorktrees {
        clean: root.join("clean"),
        dirty,
        main,
        root,
    };
}

/// What:
///  What a `!` alias's shell is given as `GIT_DIR` from one place,
///  read by running real Git there.
///  `&Path` borrows the place the caller stands in.
/// Why:
///  The control's premise is Git's own answer,
///  never this crate's rule for it:
///  a place Git exports nothing from would make the rows below pass for a reason the finding is not about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const gitDirOf = (place) => git(place, ['printgitdir']).stdout;
/// ```
fn the_git_dir_git_gives_a_shell_in(place: &Path) -> String {
    let output: std::process::Output = git(place, &["printgitdir"]);
    assert!(output.status.success(), "the fixture's own alias prints");
    return String::from_utf8_lossy(output.stdout.as_slice()).into_owned();
}

/// What:
///  A bare `!git` whose caller stands in a linked worktree and whose words name another one
///  is judged as the direct spelling of the state its nested Git acts in:
///  the caller's Git directory with the other worktree's files.
/// Why:
///  The alpha.51 review's H2:
///  from a linked worktree Git hands the shell `GIT_DIR=<main>/.git/worktrees/<name>`,
///  so `git g -C <other worktree> reset --hard` writes that other worktree's files with the caller's index and
///  `HEAD`;
///  alpha.51 ran it from a dirty caller and overwrote a file the clean worktree held,
///  while alpha.50 refused it because it judged the caller by accident.
///  The premise is measured twice by real Git:
///  the alias's shell is given a Git directory here and none in the main checkout,
///  and the direct spelling of that state is refused where the direct spelling of the clean worktree alone runs.
///  Each row then holds the alias spelling to that state's own answer,
///  whatever the guarded form is,
///  so a form the state lets through cannot make a row pass.
#[test]
fn a_shell_alias_is_judged_as_the_state_its_nested_git_acts_in() {
    let fixture_worktrees: ThreeWorktrees =
        a_dirty_linked_worktree_beside_a_clean_one("guard-alias-exported-git-dir");
    assert!(
        !has_uncommitted_state(fixture_worktrees.clean.as_path()),
        "the worktree the words name is clean"
    );
    assert!(
        has_uncommitted_state(fixture_worktrees.dirty.as_path()),
        "the caller holds uncommitted work"
    );
    // Git gives the caller's shell a Git directory, which is what makes the nested Git keep the caller's index,
    //  and it gives the main checkout's shell none.
    assert!(
        !the_git_dir_git_gives_a_shell_in(fixture_worktrees.dirty.as_path()).is_empty(),
        "Git gives the alias's shell no GIT_DIR here, so these rows would pass for another reason"
    );
    assert!(
        !the_git_dir_git_gives_a_shell_in(fixture_worktrees.clean.as_path()).is_empty(),
        "a linked worktree's alias shell is given a Git directory"
    );
    assert!(
        the_git_dir_git_gives_a_shell_in(fixture_worktrees.main.as_path()).is_empty(),
        "the main checkout's alias shell is given none, and the nested Git finds the repository itself"
    );
    let resolution: crate::real_git::ResolutionInputs = inputs(fixture_worktrees.root.as_path());
    let toward_clean: Vec<OsString> = spellings_of_the_target(
        fixture_worktrees
            .main
            .join(".git")
            .join("worktrees")
            .join("clean")
            .as_path(),
        fixture_worktrees.clean.as_path(),
    )
    .remove(0);
    let toward_main: Vec<OsString> = spellings_of_the_target(
        fixture_worktrees.main.join(".git").as_path(),
        fixture_worktrees.main.as_path(),
    )
    .remove(0);
    // What the nested Git acts in for one direction: the caller's Git directory and the other place's files.
    let the_state_of = |caller: &str, worktree: &Path| -> Vec<OsString> {
        return vec![
            OsString::from("--git-dir"),
            fixture_worktrees
                .main
                .join(".git")
                .join("worktrees")
                .join(caller)
                .into_os_string(),
            OsString::from("--work-tree"),
            worktree.as_os_str().to_os_string(),
        ];
    };
    /// What one direction of the table is about.
    /// Why:
    ///  The two directions give different answers,
    ///  and a control that asserted one answer for both would have to be wrong about one of them.
    struct Direction<'a> {
        /// Where the caller stands.
        caller: &'a Path,
        /// The words after the alias name.
        target: &'a Vec<OsString>,
        /// The state the nested Git acts in there, spelled directly.
        state: Vec<OsString>,
        /// Whether a hard reset of that target alone is refused.
        the_target_alone_is_refused: bool,
        /// Whether a hard reset of that state is refused.
        the_state_is_refused: bool,
    }
    // The dirty caller over the clean worktree is the loss the review measured; the clean caller over the main
    //  checkout acts in a state whose files and index agree, and that one runs.
    let rows: [Direction; 2] = [
        Direction {
            caller: fixture_worktrees.dirty.as_path(),
            target: &toward_clean,
            state: the_state_of("dirty", fixture_worktrees.clean.as_path()),
            the_target_alone_is_refused: false,
            the_state_is_refused: true,
        },
        Direction {
            caller: fixture_worktrees.clean.as_path(),
            target: &toward_main,
            state: the_state_of("clean", fixture_worktrees.main.as_path()),
            the_target_alone_is_refused: true,
            the_state_is_refused: false,
        },
    ];
    for direction in &rows {
        let the_state: &Vec<OsString> = &direction.state;
        // The two answers the rows rely on, so a fixture that refuses everything cannot pass them.
        let the_target_alone: Verdict = verdict(plan_invocation(
            directly(direction.target.as_slice(), &["reset", "--hard"]).as_slice(),
            &[],
            &resolution,
        ));
        assert_eq!(
            is_stopped(&the_target_alone),
            direction.the_target_alone_is_refused,
            "a hard reset of {} alone is the answer this direction was built on",
            direction.target.first().expect("a -C").to_string_lossy()
        );
        let the_state_alone: Verdict = verdict(plan_invocation(
            directly(the_state.as_slice(), &["reset", "--hard"]).as_slice(),
            &[],
            &resolution,
        ));
        assert_eq!(
            is_stopped(&the_state_alone),
            direction.the_state_is_refused,
            "a hard reset of the caller's index over the other place's files is the answer this direction was built on"
        );
        for command in GUARDED_COMMANDS {
            let behind: Verdict = verdict(plan_invocation(
                behind_the_alias(direction.caller, direction.target.as_slice(), command).as_slice(),
                &[],
                &resolution,
            ));
            let direct: Verdict = verdict(plan_invocation(
                directly(the_state.as_slice(), command).as_slice(),
                &[],
                &resolution,
            ));
            assert_eq!(
                behind, direct,
                "the alias spelling of {command:?} is judged as the state its nested Git acts in is"
            );
        }
    }
    remove(fixture_worktrees.root.as_path());
}
