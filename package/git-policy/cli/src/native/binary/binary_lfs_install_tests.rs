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

use super::support::{Fixture, Observed, executable, fixture, repository, run_wrapped, stderr_of};
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
///  must be told the truth (the filters were written,
///  no hook is needed because the wrapper uploads the objects itself)
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
    assert!(said.contains("`filter.lfs.*` keys were written"), "{said}");
    assert!(!said.contains("git lfs update --manual"), "no advice that cannot work is offered: {said}");
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
