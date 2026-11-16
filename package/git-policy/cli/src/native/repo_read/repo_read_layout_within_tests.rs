//! Controls for the ceiling of the repository location read:
//! a read that is given a thread answers a plain layout,
//! and a read that is given none answers nothing at every entry point,
//! so no entry point can reach the file system on its caller's thread.

use super::*;
use crate::repo_read_layout::{ObjectFormat, git_path, read_layout, read_layout_after_success, read_layout_after_success_for, read_layout_after_success_with, read_layout_in, read_layout_with};
use crate::repo_read_layout_without::read_layout_without;
use crate::test_support::{fixture, repository};
use crate::work_awaited_within_a_ceiling::a_control_refuses_every_thread;

/// The variables Git would see for a repository no file of this machine describes:
/// no system file,
/// no global file,
/// no configuration directory of the person's.
fn overlay(root: &Path) -> Vec<(OsString, OsString)> {
    return vec![
        (OsString::from("HOME"), root.join("no-home").into_os_string()),
        (OsString::from("XDG_CONFIG_HOME"), root.join("no-xdg").into_os_string()),
        (OsString::from("GIT_CONFIG_NOSYSTEM"), OsString::from("1")),
        (OsString::from("GIT_CONFIG_GLOBAL"), OsString::from("/nonexistent-global-config")),
    ];
}

/// The configuration scope facts of that environment.
fn scope(root: &Path) -> ScopeEnvironment {
    return ScopeEnvironment::from_variables(overlay(root).as_slice());
}

/// The global options that start the read in `repository`.
fn at(repository: &Path) -> Vec<OsString> {
    return vec![OsString::from("-C"), repository.as_os_str().to_owned()];
}

/// A fixture with one repository whose Git directory is a plain directory.
fn plain_repository(name: &str) -> (std::path::PathBuf, std::path::PathBuf) {
    let root: std::path::PathBuf = fixture(name);
    let repo: std::path::PathBuf = repository(root.as_path(), "repo");
    return (root, repo);
}

/// The real path of the repository's own Git directory.
fn git_directory_of(repository: &Path) -> std::path::PathBuf {
    return std::fs::canonicalize(repository.join(".git")).expect("the Git directory of the fixture");
}

/// A read that is given a thread answers the layout of a plain repository,
/// and the layout is the one Git prints.
#[test]
fn a_plain_layout_is_answered_under_the_ceiling() {
    let (root, repo) = plain_repository("layout-within-plain");
    let Some(layout) = from_a_directory(repo.clone(), scope(root.as_path()), true) else {
        panic!("the read answered nothing for a plain repository");
    };
    assert_eq!(layout.git_dir, git_directory_of(repo.as_path()));
    assert_eq!(layout.common_dir, git_directory_of(repo.as_path()));
    assert_eq!(layout.worktree_root, repo);
    assert_eq!(layout.object_format, ObjectFormat::Sha1);
}

/// A read whose thread the system refuses answers nothing,
/// and the same call answers a layout once a thread is given,
/// so the answer above comes from the job and not from the fixture.
#[test]
fn a_read_whose_thread_is_refused_answers_nothing() {
    let (root, repo) = plain_repository("layout-within-refused");
    let refused = a_control_refuses_every_thread();
    let refused_answer: Option<RepoLayout> = from_a_directory(repo.clone(), scope(root.as_path()), true);
    drop(refused);
    assert!(refused_answer.is_none(), "the read answered without a thread: {refused_answer:?}");
    let given_answer: Option<RepoLayout> = from_a_directory(repo.clone(), scope(root.as_path()), true);
    assert!(given_answer.is_some(), "the same call answered nothing once a thread was given");
}

/// The link lookup `git_path` makes is the ceiling's work too:
/// a link and a plain file are told apart when a thread is given,
/// and neither is answered when the system refuses one.
#[test]
fn a_linked_name_is_read_under_the_ceiling() {
    let root: std::path::PathBuf = fixture("layout-within-link");
    let plain: std::path::PathBuf = root.join("plain");
    std::fs::write(plain.as_path(), "").expect("a plain file");
    let link: std::path::PathBuf = root.join("link");
    std::os::unix::fs::symlink(plain.as_path(), link.as_path()).expect("a link");
    assert_eq!(is_a_link(plain.as_path()), Some(false));
    assert_eq!(is_a_link(link.as_path()), Some(true));
    // A name that is not there is not a link: Git's own `--git-path` answers a path it did not find.
    assert_eq!(is_a_link(root.join("absent").as_path()), Some(false));
    let refused = a_control_refuses_every_thread();
    let refused_answers: (Option<bool>, Option<bool>) = (is_a_link(plain.as_path()), is_a_link(link.as_path()));
    drop(refused);
    assert_eq!(refused_answers, (None, None), "a lookup answered without a thread");
}

/// What every entry point of the read answers for one fixture.
fn every_entry_point(root: &Path, repo: &Path) -> Vec<(&'static str, Option<RepoLayout>)> {
    return vec![
        ("read_layout", read_layout(at(repo).as_slice(), overlay(root).as_slice())),
        ("read_layout_with", read_layout_with(repo, overlay(root).as_slice(), &scope(root))),
        ("read_layout_in", read_layout_in(repo, overlay(root).as_slice())),
        ("read_layout_without", read_layout_without(at(repo).as_slice(), overlay(root).as_slice(), &[], &[])),
        ("read_layout_after_success", read_layout_after_success(repo, overlay(root).as_slice())),
        ("read_layout_after_success_with", read_layout_after_success_with(repo, overlay(root).as_slice(), &scope(root))),
        (
            "read_layout_after_success_for",
            read_layout_after_success_for(at(repo).as_slice(), overlay(root).as_slice()),
        ),
    ];
}

/// Every entry point of the read asks the file system on the job's thread and never on its caller's:
/// with the thread refused each one answers nothing,
/// and with a thread given each one answers the same plain layout.
#[test]
fn every_entry_point_reads_under_the_ceiling() {
    let (root, repo) = plain_repository("layout-within-entry-points");
    let answered: Vec<(&'static str, Option<RepoLayout>)> = every_entry_point(root.as_path(), repo.as_path());
    for (name, answer) in &answered {
        assert!(answer.is_some(), "{name} answered nothing for a plain repository, so the control proves nothing");
    }
    let refused = a_control_refuses_every_thread();
    let refused_answers: Vec<(&'static str, Option<RepoLayout>)> = every_entry_point(root.as_path(), repo.as_path());
    drop(refused);
    let answered_anyway: Vec<&str> = refused_answers
        .iter()
        .filter(|(_, answer)| answer.is_some())
        .map(|(name, _)| *name)
        .collect();
    assert!(answered_anyway.is_empty(), "these entry points read the file system without the ceiling's thread: {answered_anyway:?}");
}

/// `git_path` decides a name with the ceiling's thread as well,
/// so a read that is given none leaves the name to Git instead of answering it.
#[test]
fn git_path_leaves_a_name_to_git_when_its_thread_is_refused() {
    let (root, repo) = plain_repository("layout-within-git-path");
    let layout: RepoLayout = from_a_directory(repo, scope(root.as_path()), true).expect("the plain layout");
    assert_eq!(git_path(&layout, "index"), Some(layout.git_dir.join("index")));
    let refused = a_control_refuses_every_thread();
    let refused_answer: Option<std::path::PathBuf> = git_path(&layout, "index");
    drop(refused);
    assert_eq!(refused_answer, None, "the name was answered without the ceiling's thread");
}
