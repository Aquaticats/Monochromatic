//! What:
//!  A guarded command behind a bare `!git` alias is judged for every place its nested Git acts in:
//!  the files of the worktree its words name,
//!  and the index and `HEAD` of the repository that the `GIT_DIR` Git exported to that shell keeps.
//! Why:
//!  Issue 6641,
//!  found by the second pass on `0.6.1-alpha.51`:
//!  from a linked worktree Git hands the alias's shell `GIT_DIR` of that worktree,
//!  and a caller's `--git-dir`,
//!  `--work-tree` or `-c` before the alias name is stored in the same environment,
//!  so `git g -C <other worktree> reset --hard` wrote that worktree's files with the caller's index and
//!  `HEAD`.
//!  The wrapper measured one place and ran a command that changed two:
//!  an untracked file of the other worktree was overwritten with the caller's committed text,
//!  and a staged new file of the main checkout disappeared from disk.
//!  Each control runs the spelling through the built executable with real Git behind it,
//!  beside the same words through real Git alone in a fixture of its own,
//!  and compares the files,
//!  the index and the untracked files of every worktree before and after.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const before = snapshot(fixture);                                      // every worktree's status, HEAD and files
//! const run = wrapped(fixture).args(['g', '-C', first, 'reset', '--hard']); // first on PATH, real Git behind it
//! expect(run.code).not.toBe(0);
//! expect(snapshot(fixture)).toEqual(before);
//! ```

use super::support::{
    Fixture,
    Observed,
    direct,
    fixture,
    git,
    observe,
    porcelain,
    remove,
    repository,
    run_wrapped,
    stderr_of,
};
use std::ffi::OsStr;
use std::path::{
    Path,
    PathBuf,
};
use std::process::Command;

/// The text of the untracked file the first worktree holds at a path the caller's branch commits.
const UNTRACKED_WORK_OF_THE_FIRST_WORKTREE: &[u8] =
    b"PRECIOUS untracked content of the first worktree\n";

/// The uncommitted edit the main checkout holds,
///  which a reset with the main checkout's own repository would take.
const THE_EDITED_TEXT: &[u8] = b"one\nMAIN-DIRTY\n";

/// What three worktrees hold together,
///  so one comparison covers every place a command could change.
/// Why:
///  The finding is a loss of files,
///  not a change of exit status:
///  alpha.51 printed `HEAD is now at ...` and exit 0 while the file it had overwritten was gone,
///  so a control that read the status alone would have called that a success.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const held = (fixture, place) => `${porcelain(place)} ${head} ${files(place).map(({ name, bytes }) => `${name}=${bytes}`)}`;
/// ```
fn everything_the_worktrees_hold(
    places: &[&Path],
    fixture: &Fixture,
) -> String {
    let mut held: String = String::new();
    for place in places {
        let status: String = porcelain(fixture, place);
        let head: String = String::from_utf8_lossy(
            git(fixture, place, &["rev-parse", "HEAD"])
                .stdout
                .as_slice(),
        )
        .into_owned();
        held.push_str(&format!(
            "{}\n  status={status:?} head={head:?}",
            place.display()
        ));
        let mut names: Vec<std::ffi::OsString> = Vec::new();
        for entry in std::fs::read_dir(place).expect("the worktree reads") {
            names.push(entry.expect("an entry of the worktree").file_name());
        }
        names.sort();
        for name in names {
            let path: PathBuf = place.join(name.as_os_str());
            if !std::fs::symlink_metadata(path.as_path())
                .expect("an entry reads")
                .is_file()
            {
                held.push_str(&format!("\n  {}/ (not a file)", name.to_string_lossy()));
                continue;
            }
            let bytes: Vec<u8> =
                std::fs::read(path.as_path()).expect("a file of the worktree reads");
            held.push_str(&format!(
                "\n  {} = {:?}",
                name.to_string_lossy(),
                String::from_utf8_lossy(bytes.as_slice())
            ));
        }
        held.push('\n');
    }
    return held;
}

/// What:
///  The repository both classes of issue 6641 are measured in:
///  a main checkout holding an uncommitted edit and a staged new file,
///  a first linked worktree whose branch commits `c2.txt` while the worktree holds an untracked file of that name,
///  and a second linked worktree,
///  which is the caller,
///  holding uncommitted work of its own.
/// Why:
///  The first class's row needs a main checkout with work to lose behind `--git-dir` and `--work-tree`;
///  the second needs two linked worktrees on branches that differ at one path,
///  so the caller's index over the other worktree's files has something to overwrite.
///  `git worktree add` through this repository's wrapper would take the clone route on this host,
///  so the fixture is made with real Git and the bare alias is configured in the repository,
///  which every linked worktree of it reads.
struct TheTwoClassesOfIssue6641 {
    /// The fixture folder.
    fixture: Fixture,
    /// The main checkout.
    main: PathBuf,
    /// The clean linked worktree the words name,
    ///  holding the untracked file the caller's branch commits.
    first: PathBuf,
    /// The dirty linked worktree the caller stands in.
    second: PathBuf,
}

/// What:
///  The fixture `TheTwoClassesOfIssue6641` describes.
/// Why:
///  One builder keeps the two controls on one shape,
///  and every control asserts that shape before its row runs,
///  so a row cannot pass because the fixture lost what it was about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const at = twoClasses('name'); // { fixture, main, first, second }
/// ```
fn the_two_classes_of_issue_6641(name: &str) -> TheTwoClassesOfIssue6641 {
    let the_fixture: Fixture = fixture(name);
    let main: PathBuf = repository(&the_fixture, OsStr::new("main"));
    std::fs::write(main.join("tracked.txt"), b"one\n").expect("the tracked file");
    git(&the_fixture, main.as_path(), &["add", "--", "tracked.txt"]);
    git(
        &the_fixture,
        main.as_path(),
        &["commit", "--quiet", "--message=base"],
    );
    git(&the_fixture, main.as_path(), &["config", "alias.g", "!git"]);
    for (branch, directory) in [("first", "first"), ("second", "second")] {
        let path: PathBuf = the_fixture.root.join(directory);
        git(
            &the_fixture,
            main.as_path(),
            &[
                "worktree",
                "add",
                "--quiet",
                "-b",
                branch,
                path.to_str().expect("the fixture's path is UTF-8"),
            ],
        );
    }
    let first: PathBuf = the_fixture.root.join("first");
    let second: PathBuf = the_fixture.root.join("second");
    // The caller's branch commits `c2.txt`, and the first worktree holds an untracked file of that name.
    std::fs::write(
        second.join("c2.txt"),
        b"committed in the caller's worktree only\n",
    )
    .expect("the caller's committed text");
    git(&the_fixture, second.as_path(), &["add", "--", "c2.txt"]);
    git(
        &the_fixture,
        second.as_path(),
        &["commit", "--quiet", "--message=c2"],
    );
    std::fs::write(first.join("c2.txt"), UNTRACKED_WORK_OF_THE_FIRST_WORKTREE)
        .expect("the untracked work of the first worktree");
    // The main checkout's uncommitted work: an edit and a staged new file.
    std::fs::write(main.join("tracked.txt"), THE_EDITED_TEXT).expect("the main checkout's edit");
    std::fs::write(main.join("staged.txt"), b"staged in main\n").expect("the staged new file");
    git(&the_fixture, main.as_path(), &["add", "--", "staged.txt"]);
    std::fs::write(main.join("untracked.txt"), b"untracked in main\n").expect("the untracked file");
    // The caller's own uncommitted work, so a guard that judged the caller refuses it.
    std::fs::write(second.join("tracked.txt"), b"one\nSECOND-DIRTY\n").expect("the caller's edit");
    std::fs::write(second.join("staged.txt"), b"staged in the caller\n")
        .expect("the caller's staged file");
    git(&the_fixture, second.as_path(), &["add", "--", "staged.txt"]);
    std::fs::write(second.join("untracked.txt"), b"untracked in the caller\n")
        .expect("the caller's untracked file");
    return TheTwoClassesOfIssue6641 {
        fixture: the_fixture,
        main,
        first,
        second,
    };
}

/// The three worktrees of one fixture,
///  in the order every comparison reads them.
fn the_three_worktrees(at: &TheTwoClassesOfIssue6641) -> Vec<&Path> {
    return vec![at.main.as_path(), at.first.as_path(), at.second.as_path()];
}

/// What:
///  The same words through real Git alone,
///  in a fixture of their own.
/// Why:
///  The oracle has to be run for real:
///  this repository's wrapper refuses the words,
///  and what they would have done is the reason the refusal is owed.
///  Its own fixture keeps its loss from hiding the wrapper's answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const oracle = (fixture, place, words) => spawnSync('/usr/bin/git', words, { cwd: place, env: hermetic });
/// ```
fn through_real_git_alone(
    the_fixture: &Fixture,
    place: &Path,
    words: &[&str],
) -> Observed {
    let mut command: Command = direct(the_fixture);
    command.current_dir(place).args(words);
    return observe(&mut command, b"");
}

/// What:
///  With the words naming another worktree and the caller standing in a linked worktree,
///  the alias spelling is refused and every worktree keeps what it held,
///  while the same words through real Git alone overwrite the untracked file.
/// Why:
///  Issue 6641's second class:
///  `git g -C <first worktree> reset --hard` typed in the second,
///  where the first worktree holds an untracked `c2.txt` its own index does not track and the caller's branch commits
///  that path.
///  Real Git keeps the caller's `GIT_DIR`,
///  so it writes the caller's committed text over the untracked file and
///  resets the caller's index;
///  nothing records the untracked text afterwards.
///  alpha.50 refused this by accident,
///  judging the dirty caller,
///  and alpha.51 ran it.
#[test]
fn the_alias_spelling_of_a_reset_leaves_every_worktree_as_it_was() {
    let at: TheTwoClassesOfIssue6641 =
        the_two_classes_of_issue_6641("binary-shell-alias-two-places");
    let places: Vec<&Path> = the_three_worktrees(&at);
    // The shape the row is about: the caller's branch commits the path the other worktree holds untracked work at.
    assert_eq!(
        std::fs::read(at.first.join("c2.txt")).expect("the untracked file"),
        UNTRACKED_WORK_OF_THE_FIRST_WORKTREE,
        "the first worktree holds the untracked work this row is about"
    );
    assert!(
        porcelain(&at.fixture, at.second.as_path()).contains("staged.txt"),
        "the caller holds a staged new file"
    );
    let before: String = everything_the_worktrees_hold(places.as_slice(), &at.fixture);
    let observed: Observed = run_wrapped(
        &at.fixture,
        at.second.as_path(),
        &[
            "g",
            "-C",
            at.first.to_str().expect("the fixture's path is UTF-8"),
            "reset",
            "--hard",
        ],
    );
    assert_ne!(
        observed.code,
        Some(0),
        "the wrapper ran a reset that writes one worktree from another repository's index: {}",
        stderr_of(&observed)
    );
    assert_eq!(
        everything_the_worktrees_hold(places.as_slice(), &at.fixture),
        before,
        "the refused command changed a worktree: {}",
        stderr_of(&observed)
    );
    // The words through real Git alone, in a fixture of their own, are what the refusal is for.
    let oracle: TheTwoClassesOfIssue6641 =
        the_two_classes_of_issue_6641("binary-shell-alias-two-places-oracle");
    let the_words: [&str; 5] = [
        "g",
        "-C",
        oracle.first.to_str().expect("the fixture's path is UTF-8"),
        "reset",
        "--hard",
    ];
    let direct_run: Observed =
        through_real_git_alone(&oracle.fixture, oracle.second.as_path(), &the_words);
    assert_eq!(
        direct_run.code,
        Some(0),
        "real Git runs these words: {}",
        stderr_of(&direct_run)
    );
    assert_eq!(
        std::fs::read(oracle.first.join("c2.txt")).expect("the untracked file"),
        b"committed in the caller's worktree only\n",
        "real Git overwrites the untracked work of the other worktree, which is the loss the refusal prevents"
    );
    remove(&at.fixture);
    remove(&oracle.fixture);
}

/// What:
///  With `--git-dir` and `--work-tree` typed before the alias name and the words naming a worktree,
///  the alias spelling is refused and every worktree keeps what it held,
///  while the same words through real Git alone reset the main checkout.
/// Why:
///  Issue 6641's first class:
///  the options before the alias name move the repository and the files the nested Git acts on,
///  and a reading that dropped them judged the caller's place instead,
///  so alpha.51 ran it and the main checkout lost an unstaged edit and a staged new file.
///  The direct spelling of the same words was refused by alpha.50 and alpha.51,
///  which is the answer the alias spelling owes as well.
#[test]
fn the_alias_spelling_whose_caller_named_the_repository_before_it_leaves_the_main_checkout_alone() {
    let at: TheTwoClassesOfIssue6641 =
        the_two_classes_of_issue_6641("binary-shell-alias-place-options");
    let places: Vec<&Path> = the_three_worktrees(&at);
    assert_eq!(
        std::fs::read(at.main.join("tracked.txt")).expect("the main checkout's tracked file"),
        THE_EDITED_TEXT,
        "the main checkout holds the edit this row is about"
    );
    assert!(
        at.main.join("staged.txt").exists(),
        "the main checkout holds the staged new file this row is about"
    );
    let before: String = everything_the_worktrees_hold(places.as_slice(), &at.fixture);
    let the_repository: String = format!("--git-dir={}", at.main.join(".git").display());
    let the_work_tree: String = format!("--work-tree={}", at.main.display());
    let observed: Observed = run_wrapped(
        &at.fixture,
        at.second.as_path(),
        &[
            the_repository.as_str(),
            the_work_tree.as_str(),
            "g",
            "-C",
            at.first.to_str().expect("the fixture's path is UTF-8"),
            "reset",
            "--hard",
        ],
    );
    assert_ne!(
        observed.code,
        Some(0),
        "the wrapper ran a reset of the main checkout from options typed before the alias name: {}",
        stderr_of(&observed)
    );
    assert_eq!(
        everything_the_worktrees_hold(places.as_slice(), &at.fixture),
        before,
        "the refused command changed a worktree: {}",
        stderr_of(&observed)
    );
    // The same words through real Git alone, in a fixture of their own, reset the main checkout.
    let oracle: TheTwoClassesOfIssue6641 =
        the_two_classes_of_issue_6641("binary-shell-alias-place-options-oracle");
    let oracle_repository: String = format!("--git-dir={}", oracle.main.join(".git").display());
    let oracle_work_tree: String = format!("--work-tree={}", oracle.main.display());
    let the_words: [&str; 7] = [
        oracle_repository.as_str(),
        oracle_work_tree.as_str(),
        "g",
        "-C",
        oracle.first.to_str().expect("the fixture's path is UTF-8"),
        "reset",
        "--hard",
    ];
    let direct_run: Observed =
        through_real_git_alone(&oracle.fixture, oracle.second.as_path(), &the_words);
    assert_eq!(
        direct_run.code,
        Some(0),
        "real Git runs these words: {}",
        stderr_of(&direct_run)
    );
    assert_eq!(
        std::fs::read(oracle.main.join("tracked.txt")).expect("the main checkout's tracked file"),
        b"one\n",
        "real Git resets the main checkout, which is the loss the refusal prevents"
    );
    assert!(
        !oracle.main.join("staged.txt").exists(),
        "real Git deletes the staged new file from disk, which is the loss the refusal prevents"
    );
    remove(&at.fixture);
    remove(&oracle.fixture);
}
