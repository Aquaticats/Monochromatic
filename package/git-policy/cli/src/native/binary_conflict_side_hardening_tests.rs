//! What:
//!  Command lines that look like one conflicted file to the conflict-side guard and are more to Git: pathspec modes that
//!  widen a match, a symbolic link where a directory was, an option value that reads as an option, and a file that changes
//!  while it is copied, each through the built executable against real Git.
//! Why:
//!  A cross-vendor review of the conflict-side carve-out named these four. Each control makes the situation with real Git,
//!  gives the file the person had, runs the wrapper, and compares what is left with what the person had.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await makeConflict('merge'); edit('A.rs'); run(['--icase-pathspecs', 'checkout', '--theirs', '--', 'a.rs']);
//! expect(readFileSync('A.rs')).toEqual(whatThePersonHad);
//! ```

use super::super::support::{Fixture, Observed, direct, fixture, git, observe, remove, run_wrapped, stderr_of, wrapped};
use super::{HAND_EDITED, Stop, audit_lines, conflicted, rejected_by_policy, saved_root};
use std::path::PathBuf;

/// What a person had in a clean-looking file that only a case-insensitive match would reach.
const LOOK_ALIKE_EDIT: &[u8] = b"my unsaved edit of the look-alike\n";

/// A merge stopped with `a.rs` conflicted and a tracked, staged `A.rs` that has an unsaved edit.
fn with_look_alike(fixture: &Fixture) -> PathBuf {
    let repo: PathBuf = conflicted(fixture, Stop::Merge);
    std::fs::write(repo.join("A.rs"), "staged\n").expect("look-alike");
    git(fixture, repo.as_path(), &["add", "--", "A.rs"]);
    std::fs::write(repo.join("A.rs"), LOOK_ALIKE_EDIT).expect("unsaved edit");
    return repo;
}

/// Nothing was saved or replaced: the person's files are as they were, and no copy or saved-conflict record exists.
fn assert_untouched(repo: &std::path::Path, said: &str) {
    assert_eq!(std::fs::read(repo.join("a.rs")).expect("a.rs"), HAND_EDITED, "a.rs is as the person had it: {said}");
    assert_eq!(std::fs::read(repo.join("A.rs")).expect("A.rs"), LOOK_ALIKE_EDIT, "A.rs is as the person had it: {said}");
    assert!(!saved_root(repo).exists(), "no copy was made: {said}");
    assert!(audit_lines(repo).iter().all(|line| { return !line.contains("saved-conflict"); }), "no saved-conflict record: {said}");
}

/// Real Git, given the pathspec mode, overwrites the look-alike too: the situation exists, and the control can show a difference.
#[test]
fn real_git_with_case_insensitive_pathspecs_overwrites_the_look_alike() {
    let fixture: Fixture = fixture("icase-oracle");
    let repo: PathBuf = with_look_alike(&fixture);
    let mut command = direct(&fixture);
    command.current_dir(&repo).env("GIT_ICASE_PATHSPECS", "1").args(["checkout", "--theirs", "--", "a.rs"]);
    let observed: Observed = observe(&mut command, b"");
    assert_eq!(observed.code, Some(0), "{}", stderr_of(&observed));
    assert_eq!(std::fs::read(repo.join("A.rs")).expect("A.rs"), b"staged\n", "Git replaced the look-alike with the index copy");
    remove(&fixture);
}

/// Every global pathspec mode, as a variable and as a global option, keeps the rejection: a mode changes which files a
///  pathspec names, and the guard knows only the literal reading.
#[test]
fn a_global_pathspec_mode_keeps_the_rejection() {
    let fixture: Fixture = fixture("icase-modes");
    let repo: PathBuf = with_look_alike(&fixture);
    for variable in ["GIT_ICASE_PATHSPECS", "GIT_GLOB_PATHSPECS", "GIT_NOGLOB_PATHSPECS", "GIT_LITERAL_PATHSPECS"] {
        let mut command = wrapped(&fixture);
        command.current_dir(&repo).env(variable, "1").args(["checkout", "--theirs", "--", "a.rs"]);
        let observed: Observed = observe(&mut command, b"");
        let said: String = stderr_of(&observed);
        assert!(rejected_by_policy(&observed), "{variable}: {said}");
        assert_untouched(repo.as_path(), said.as_str());
    }
    for option in ["--icase-pathspecs", "--glob-pathspecs", "--noglob-pathspecs", "--literal-pathspecs"] {
        let observed: Observed = run_wrapped(&fixture, repo.as_path(), &[option, "checkout", "--theirs", "--", "a.rs"]);
        let said: String = stderr_of(&observed);
        assert!(rejected_by_policy(&observed), "{option}: {said}");
        assert_untouched(repo.as_path(), said.as_str());
    }
    for token in [":(icase)a.rs", ":(glob)a.rs", ":(exclude)A.rs", ":!A.rs", ":/a.rs", ":(literal)a.rs"] {
        let observed: Observed = run_wrapped(&fixture, repo.as_path(), &["checkout", "--theirs", "--", token]);
        let said: String = stderr_of(&observed);
        assert!(rejected_by_policy(&observed), "{token}: {said}");
        assert_untouched(repo.as_path(), said.as_str());
    }
    // The same fixture is let through without a mode, so the rejections above are the modes' doing and not the fixture's.
    let plain: Observed = run_wrapped(&fixture, repo.as_path(), &["checkout", "--theirs", "--", "a.rs"]);
    assert_eq!(plain.code, Some(0), "{}", stderr_of(&plain));
    assert_eq!(std::fs::read(repo.join("A.rs")).expect("A.rs"), LOOK_ALIKE_EDIT, "the look-alike is not named, so it stays");
    remove(&fixture);
}
