//! What:
//!  The two looks of a clean:
//!  what the first one saves,
//!  and the repository made between them that the second one saves too.
//! Why:
//!  The clean runs after the saves, so a selection made once lets a repository made in that window be
//!  deleted with the ignored folder: the seventh review delayed the first save rename, made a second
//!  repository in `ignored/`, and the wrapped `git clean -ffXd` exited 0 with that repository gone
//!  and only the first one saved.
//!  A reading that looks again after every save finds it, and the clean may only delete what was
//!  looked at.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // the second dry run names ignored/, and a repository made then is saved with the first one
//! ```

use super::{AllowedForm, allow_a_discarding_form};
use crate::command_test_support::{fixture, git, remove};
use crate::conflict::discarded_prune_limits::Limits;
use crate::policy_engine::PolicyOutcome;
use crate::repo_read::repository_facts::{GitFacts, RepositoryFacts, git_facts};
use crate::rule_linked_worktree::GuardedCommand;
use crate::test_support::{REAL_GIT, isolated_overlay};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// What:
///  A real Git replaced by a script that makes a second ignored repository while it answers the
///  second dry run,
///  and is real Git for every other query.
///  `&Path` borrows the fixture's directory for its counter and the worktree.
/// Why:
///  The window the seventh review used is the time between the first look and the real clean;
///  a control cannot hold that window open, so the appearance is put where the next look happens:
///  the second dry run's answer.
///  A reading that looks once never asks it, and the repository the answer makes is never saved,
///  which is exactly what the control has to show.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const shim = writeShimThatMakesASecondRepositoryOnTheSecondDryRun(directory, linked);
/// ```
fn a_git_that_makes_a_repository_on_the_second_dry_run(directory: &Path, linked: &Path) -> PathBuf {
    let shim: PathBuf = directory.join("git-that-makes-a-repository");
    let counter: PathBuf = directory.join("dry-runs");
    let script: String = format!(
        r#"#!/bin/sh
case "$*" in
  *'clean -n'*)
    n=$(cat '{counter}' 2>/dev/null || echo 0); n=$((n + 1)); printf '%s' "$n" > '{counter}'
    if [ "$n" -eq 2 ] && [ ! -d '{linked}/ignored/b/.git' ]; then
      mkdir -p '{linked}/ignored/b/.git'
      printf 'made while the clean was being saved\n' > '{linked}/ignored/b/untracked.txt'
    fi;;
esac
exec {REAL_GIT} "$@"
"#,
        counter = counter.display(),
        linked = linked.display(),
        REAL_GIT = REAL_GIT
    );
    std::fs::write(shim.as_path(), script.as_bytes()).expect("the shim");
    let mut permissions = std::fs::metadata(shim.as_path()).expect("the shim").permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
    std::fs::set_permissions(shim.as_path(), permissions).expect("the shim runs");
    return shim;
}

/// What:
///  A clean's two looks save what each found:
///  the repository the first dry run named, and the one made while that was being saved.
/// Why:
///  A single look is what the seventh review found deleting a repository made after it;
///  the control's shim makes that repository on the second look's answer,
///  so a reading that looks once saves one repository and this asserts two.
///  The clean then runs as typed, which is the owner's answer for nested work.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// allowADiscardingForm({ command: 'CleanIgnoredOnly', region: ['-f','-f','-d','-X'], facts: realGitWithTheShim })
/// // both ignored/a and ignored/b stand under the saved copies
/// ```
#[test]
fn a_repository_made_between_two_looks_is_saved_too() {
    let root: PathBuf = fixture("policy-nested-two-looks");
    let main: PathBuf = root.join("main");
    let linked: PathBuf = root.join("linked");
    std::fs::create_dir_all(main.as_path()).expect("the main checkout");
    git(main.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    std::fs::write(main.join(".gitignore"), b"ignored/\n").expect("ignore rules");
    std::fs::write(main.join("tracked.txt"), b"one\n").expect("tracked file");
    git(main.as_path(), &["add", "--", ".gitignore", "tracked.txt"]);
    git(main.as_path(), &["commit", "--quiet", "--message=base"]);
    git(main.as_path(), &["worktree", "add", "--quiet", "-b", "side", "../linked"]);
    // The first repository, with its own history and an uncommitted file.
    std::fs::create_dir_all(linked.join("ignored/a/.git")).expect("the first repository");
    std::fs::write(linked.join("ignored/a/untracked.txt"), b"the first repository's work\n").expect("its work");
    let shim: PathBuf = a_git_that_makes_a_repository_on_the_second_dry_run(root.as_path(), linked.as_path());
    let prefix: Vec<OsString> = vec![OsString::from("-C"), linked.as_os_str().to_os_string()];
    let mut facts: GitFacts = git_facts(shim.as_path(), prefix.as_slice(), isolated_overlay().as_slice());
    let location = facts.location().expect("the linked worktree's identity");
    let region: Vec<OsString> = ["-f", "-f", "-d", "-X"].iter().map(OsString::from).collect();
    let seen: Vec<OsString> = ["clean", "-f", "-f", "-d", "-X"].iter().map(OsString::from).collect();
    let mut notices: Vec<String> = Vec::<String>::new();
    let outcome: PolicyOutcome = allow_a_discarding_form(AllowedForm {
        facts: &mut facts,
        command: GuardedCommand::CleanIgnoredOnly,
        location: &location,
        region: region.as_slice(),
        seen: seen.as_slice(),
        shell_directory: None,
        dirty: None,
        discarded_limits: Limits::default(),
        notices: &mut notices,
    });
    assert!(matches!(outcome, PolicyOutcome::Findings(ref found) if found.is_empty()), "{outcome:?}");
    // Both repositories are gone from the worktree and both stand under the saved copies, each in the folder of
    //  the look that saved it.
    assert!(!linked.join("ignored/a").exists(), "the first repository is where the clean deletes it");
    assert!(
        !linked.join("ignored/b").exists(),
        "the repository made while the first was saved is where the clean deletes it, because the second look saved it"
    );
    let saved: PathBuf = main.join(".git/monochromatic-git-policy-cli/discarded");
    let mut found: Vec<PathBuf> = Vec::<PathBuf>::new();
    let mut pending: Vec<PathBuf> = vec![saved];
    while let Some(directory) = pending.pop() {
        let entries = std::fs::read_dir(directory.as_path());
        for entry in entries.into_iter().flatten().flatten() {
            let path: PathBuf = entry.path();
            if entry.file_type().map(|kind| return kind.is_dir()).unwrap_or(false) {
                pending.push(path.clone());
            }
            found.push(path);
        }
    }
    assert!(
        found.iter().any(|path| return path.ends_with("ignored/a/untracked.txt")),
        "the first repository's work is under the saved copies: {found:?}"
    );
    assert!(
        found.iter().any(|path| return path.ends_with("ignored/b/untracked.txt")),
        "the repository made while the first was saved is under the saved copies too: {found:?}"
    );
    // The notice names both, because the person has to find both.
    let said: String = notices.join("\n");
    assert!(said.contains("ignored/a"), "{said}");
    assert!(said.contains("ignored/b"), "{said}");
    remove(root.as_path());
}
