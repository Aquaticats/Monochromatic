//! What:
//!  `git lfs install` through the built executable inside a repository:
//!  it writes the global filters and is not sent into the hook write that cannot work.
//! Why:
//!  The wrapper starts every real Git with traditional hooks switched off,
//!  so git-lfs's shims went into `core.hooksPath` below `/dev/null`:
//!  `mkdir /dev/null: not a directory`,
//!  status 2,
//!  and advice to run `git lfs update --manual` or `--force`,
//!  neither of which can work while that directory cannot exist.
//!  The stand-in `git-lfs` records the arguments it was given,
//!  so the `--skip-repo` the wrapper adds and the notice that holds its reason are both read here;
//!  the real program against a real account's configuration is not needed for either,
//!  and the gate's image has none.

use super::support::{Fixture, Observed, executable, fixture, git, repository, run_direct, run_wrapped, stderr_of};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// What:
///  A stand-in `git-lfs` that records its arguments and answers as a successful `install` does.
/// Why:
///   The control is about what the wrapper hands the program and what it tells the person,
///  not about git-lfs's own configuration work,
///  which the real program does on a host that has it.
fn recording_stand_in(fixture: &Fixture, record: &Path) {
    let source: String = format!("#!/bin/sh\nprintf '%s\\n' \"$@\" > '{}'\nprintf 'Git LFS initialized.\\n'\nexit 0\n", record.display());
    executable(fixture.root.join("bin/git-lfs").as_path(), source.as_bytes());
}

/// What:
///  Inside a repository `git lfs install` runs with `--skip-repo` and says why,
///  and it keeps a caller's own options.
/// Why:
///   A person who runs the first command of every Git LFS setup inside a repository under the wrapper
///  must be told the truth (where this command asks for the filters,
///  and that no hook is needed because the wrapper uploads the objects itself)
///  and must not be sent to advice that cannot work.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ran = runWrapped(fixture, repo, ['lfs', 'install', '--force']);
/// expect(read(record)).toContain('--skip-repo'); expect(ran.stderr).toContain('no `pre-push` hook is installed')
/// ```
#[test]
fn an_install_in_a_repository_runs_with_skip_repo_and_explains_why() {
    let fixture: Fixture = fixture("lfs-install-in-a-repository");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let record: PathBuf = fixture.root.join("git-lfs-arguments.txt");
    recording_stand_in(&fixture, record.as_path());
    let ran: Observed = run_wrapped(&fixture, repo.as_path(), &["lfs", "install", "--force"]);
    let said: String = stderr_of(&ran);
    assert_eq!(ran.code, Some(0), "{said}");
    let given: String = std::fs::read_to_string(record.as_path()).unwrap_or_default();
    let words: Vec<&str> = given.lines().collect();
    assert_eq!(words, vec!["install", "--skip-repo", "--force"], "the program's own options are kept, and one is added");
    assert!(said.contains("`git lfs install` ran with `--skip-repo`"), "the person is told what was added: {said}");
    assert!(said.contains("no `pre-push` hook is installed in this repository and none is needed"), "{said}");
    assert!(said.contains("This command asks git-lfs to put the `filter.lfs.*` keys in your global configuration"), "{said}");
    assert!(!said.contains("keys were written"), "the notice never promises a write: {said}");
    assert!(!said.contains("git lfs update --manual"), "no advice that cannot work is offered: {said}");
}

/// What:
///  A caller who wrote `--skip-repo=false` gets the wrapper's copy right after that word,
///  and the notice says the caller's word did not win.
/// Why:
///   git-lfs keeps the last value of a repeated flag,
///  so the copy the wrapper puts right after `install` is undone by a later `--skip-repo=false`:
///  measured with git-lfs 3.8.1 through the wrapper, that command wrote the global keys and then ended with status 2 and
///  `mkdir /dev/null: not a directory`, the failure the option exists to prevent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ran = runWrapped(fixture, repo, ['lfs', 'install', '--skip-repo=false']); expect(read(record)).toEqual(['install', '--skip-repo=false', '--skip-repo'])
/// ```
#[test]
fn a_caller_who_turned_skip_repo_off_gets_the_wrappers_copy_after_it() {
    let fixture: Fixture = fixture("lfs-install-skip-repo-off");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    let record: PathBuf = fixture.root.join("git-lfs-arguments.txt");
    recording_stand_in(&fixture, record.as_path());
    let ran: Observed = run_wrapped(&fixture, repo.as_path(), &["lfs", "install", "--skip-repo=false"]);
    let said: String = stderr_of(&ran);
    assert_eq!(ran.code, Some(0), "{said}");
    let given: String = std::fs::read_to_string(record.as_path()).unwrap_or_default();
    let words: Vec<&str> = given.lines().collect();
    assert_eq!(
        words,
        vec!["install", "--skip-repo=false", "--skip-repo"],
        "the program reads the wrapper's copy last, so the caller's false cannot undo it"
    );
    assert!(said.contains("The last word that set `--skip-repo` turned it off"), "the person is told which word won: {said}");
    assert!(said.contains("they are skipped all the same"), "the person is told the hooks are still skipped: {said}");
}

/// The write git-lfs makes when `--worktree` is asked for.
///
/// git-lfs runs `git config --includes --worktree --replace-all` for each of its four keys
/// (`commands/command_install.go:40-58`, `git/config.go:137-140` in v3.8.1),
/// so this is where the keys land when the real program runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const written = runDirect(fixture, directory, ['config', '--includes', '--worktree', '--replace-all', key, value])
/// ```
fn worktree_write(fixture: &Fixture, directory: &Path, key: &str, value: &str) -> Observed {
    return run_direct(fixture, directory, &["config", "--includes", "--worktree", "--replace-all", key, value]);
}

/// What:
///  The notice names the file git-lfs's `--worktree` write reached in each layout:
///  `config.worktree` in this worktree's Git directory with the extension set,
///  this repository's `.git/config` for one worktree without it,
///  and no file at all for several without it, where Git refuses the write.
/// Why:
///   Measured with git-lfs 3.8.1 and plain `git` 2.56.1:
///  a linked worktree's keys landed in `<repo>/.git/worktrees/<name>/config.worktree`,
///  not in the `.git/config.worktree` a notice naming that path would have printed,
///  and two worktrees without the extension made git-lfs exit 2 after Git refused with status 128 and a fatal
///  message, writing nothing while an earlier notice promised the keys were written.
///  The write here is Git's own call, the one git-lfs makes, so the test needs no real git-lfs binary.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(notice).toContain('no file at all when it is not and the repository holds several')
/// ```
#[test]
fn the_worktree_notice_matches_where_git_writes_in_each_worktree_layout() {
    let fixture: Fixture = fixture("lfs-install-worktree-layouts");
    let record: PathBuf = fixture.root.join("git-lfs-arguments.txt");
    recording_stand_in(&fixture, record.as_path());

    // One worktree with the extension set: this repository's `.git/config.worktree`.
    let extended: PathBuf = repository(&fixture, OsStr::new("extended"));
    git(&fixture, extended.as_path(), &["config", "extensions.worktreeConfig", "true"]);
    let in_extended: Observed = worktree_write(&fixture, extended.as_path(), "probe.worktree", "x");
    assert_eq!(in_extended.code, Some(0), "{}", stderr_of(&in_extended));
    let extended_value: std::process::Output = git(&fixture, extended.as_path(), &["config", "--file", ".git/config.worktree", "--get", "probe.worktree"]);
    assert_eq!(String::from_utf8_lossy(&extended_value.stdout).trim(), "x", "Git wrote the worktree file with the extension set");
    let extended_run: Observed = run_wrapped(&fixture, extended.as_path(), &["lfs", "install", "--worktree"]);
    let extended_said: String = stderr_of(&extended_run);
    assert!(
        extended_said.contains("`config.worktree` in this worktree's Git directory) while `extensions.worktreeConfig` is set"),
        "the notice names the file Git wrote: {extended_said}"
    );
    assert!(!extended_said.contains("`.git/config.worktree`"), "the notice names no path a linked worktree lacks: {extended_said}");

    // One worktree without the extension: this repository's `.git/config`.
    let single: PathBuf = repository(&fixture, OsStr::new("single"));
    let in_single: Observed = worktree_write(&fixture, single.as_path(), "probe.worktree", "x");
    assert_eq!(in_single.code, Some(0), "{}", stderr_of(&in_single));
    let single_value: std::process::Output = git(&fixture, single.as_path(), &["config", "--file", ".git/config", "--get", "probe.worktree"]);
    assert_eq!(String::from_utf8_lossy(&single_value.stdout).trim(), "x", "Git wrote the repository file without the extension");
    let single_run: Observed = run_wrapped(&fixture, single.as_path(), &["lfs", "install", "--worktree"]);
    let single_said: String = stderr_of(&single_run);
    assert!(
        single_said.contains("this repository's `.git/config` when it is not and the repository holds one worktree"),
        "the notice names the repository file for one worktree: {single_said}"
    );

    // A linked worktree with the extension set: the file under that worktree's Git directory.
    let linked_base: PathBuf = repository(&fixture, OsStr::new("linked-base"));
    git(&fixture, linked_base.as_path(), &["config", "extensions.worktreeConfig", "true"]);
    let linked: PathBuf = fixture.root.join("linked-worktree");
    let linked_text: String = linked.to_string_lossy().into_owned();
    git(&fixture, linked_base.as_path(), &["worktree", "add", linked_text.as_str(), "--detach"],);
    let in_linked: Observed = worktree_write(&fixture, linked.as_path(), "probe.worktree", "x");
    assert_eq!(in_linked.code, Some(0), "{}", stderr_of(&in_linked));
    let linked_name: &str = linked.file_name().and_then(OsStr::to_str).expect("the worktree directory's name");
    let linked_file: String = linked_base.join(format!(".git/worktrees/{linked_name}/config.worktree")).to_string_lossy().into_owned();
    let linked_value: std::process::Output = git(&fixture, linked.as_path(), &["config", "--file", linked_file.as_str(), "--get", "probe.worktree"]);
    assert_eq!(String::from_utf8_lossy(&linked_value.stdout).trim(), "x", "Git wrote the linked worktree's file");
    let linked_run: Observed = run_wrapped(&fixture, linked.as_path(), &["lfs", "install", "--worktree"]);
    let linked_said: String = stderr_of(&linked_run);
    assert!(
        linked_said.contains("`config.worktree` in this worktree's Git directory"),
        "the notice covers the linked worktree's location: {linked_said}"
    );

    // Two worktrees without the extension: Git refuses, and the notice says no file is written.
    let multi: PathBuf = repository(&fixture, OsStr::new("multi"));
    let second: PathBuf = fixture.root.join("second-worktree");
    let second_text: String = second.to_string_lossy().into_owned();
    git(&fixture, multi.as_path(), &["worktree", "add", second_text.as_str(), "--detach"],);
    let refused: Observed = worktree_write(&fixture, multi.as_path(), "probe.worktree", "x");
    assert_eq!(refused.code, Some(128), "Git refuses the write: {}", stderr_of(&refused));
    assert!(
        stderr_of(&refused).contains("--worktree cannot be used with multiple working trees"),
        "Git names the extension it needs: {}",
        stderr_of(&refused)
    );
    assert!(!multi.join(".git/config.worktree").exists(), "Git wrote nothing for two worktrees without the extension");
    let multi_value: Observed = run_direct(&fixture, multi.as_path(), &["config", "--file", ".git/config", "--get", "probe.worktree"]);
    assert_ne!(multi_value.code, Some(0), "the refused write reached no file: {}", stderr_of(&multi_value));
    assert!(!std::fs::read_to_string(multi.join(".git/config")).expect("the repository's own file").contains("[probe]"));
    let multi_run: Observed = run_wrapped(&fixture, multi.as_path(), &["lfs", "install", "--worktree"]);
    let multi_said: String = stderr_of(&multi_run);
    assert!(
        multi_said.contains("no file at all when it is not and the repository holds several, where Git refuses the write and git-lfs ends with status 2"),
        "the notice states the refused outcome instead of promising a write: {multi_said}"
    );
}

/// What:
///  Outside a repository the command is forwarded as it was typed,
///  and the wrapper says nothing about hooks.
/// Why:
///  git-lfs writes no hook there,
///  so the option would answer a question nobody asked,
///  and a notice about this repository would name one that is not there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ran = runWrapped(fixture, fixture.root, ['lfs', 'install']);
/// expect(read(record)).not.toContain('--skip-repo')
/// ```
#[test]
fn an_install_outside_a_repository_is_forwarded_as_it_was_typed() {
    let fixture: Fixture = fixture("lfs-install-outside-a-repository");
    let outside: PathBuf = fixture.root.join("outside-any-repository");
    std::fs::create_dir(outside.as_path()).expect("a directory outside every repository");
    let record: PathBuf = fixture.root.join("git-lfs-arguments.txt");
    recording_stand_in(&fixture, record.as_path());
    let ran: Observed = run_wrapped(&fixture, outside.as_path(), &["lfs", "install"]);
    let said: String = stderr_of(&ran);
    assert_eq!(ran.code, Some(0), "{said}");
    let given: String = std::fs::read_to_string(record.as_path()).unwrap_or_default();
    assert_eq!(given.lines().collect::<Vec<&str>>(), vec!["install"], "nothing is added outside a repository");
    assert!(!said.contains("--skip-repo"), "nothing is said about a hook that is never written: {said}");
}
