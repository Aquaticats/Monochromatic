//! What:
//!  The commit states that used to be declined, landed through the service and compared with native Git: the conclusion of a
//!  merge, a cherry-pick and a revert (with the state files it removes), a detached head, a branch that does not exist yet,
//!  an unmerged index, nothing to commit, a policy rewrite, and change lists that the embedded route cannot carry.
//! Why:
//!  The landing of each is the service's compare-and-swap, and the person must find the commit, the head, the index, the
//!  state files and Git's own words as native Git leaves them.

use super::commit::remove_settled;
use super::service_commit::{Running, start};
use super::service_landing_selection::base_repository;
use super::service_landing_support::{Case, case, compare, copy_beside, run_native, run_served, finished};
use super::support::{Fixture, direct, fixture, git};
use std::path::{Path, PathBuf};
use std::process::Command;

fn write(repo: &Path, name: &str, text: &str) {
    std::fs::write(repo.join(name), text).expect("write");
}

/// Runs real Git and accepts any status (a conflicted merge exits 1).
fn attempt(fixture: &Fixture, repo: &Path, arguments: &[&str]) {
    let mut command: Command = direct(fixture);
    command.current_dir(repo).args(arguments).env("GIT_EDITOR", "true");
    let _output = command.output().expect("Git starts");
}

/// A side branch and the main branch both changed `b.txt`; the main branch is checked out.
fn diverged(fixture: &Fixture, repo: &Path) -> (String, String) {
    git(fixture, repo, &["checkout", "--quiet", "-b", "side"]);
    write(repo, "b.txt", "side\n");
    write(repo, "s.txt", "side only\n");
    git(fixture, repo, &["add", "b.txt", "s.txt"]);
    git(fixture, repo, &["commit", "--quiet", "--message=side change"]);
    let side: String = text(fixture, repo, &["rev-parse", "HEAD"]);
    git(fixture, repo, &["checkout", "--quiet", "main"]);
    write(repo, "b.txt", "main\n");
    git(fixture, repo, &["add", "b.txt"]);
    git(fixture, repo, &["commit", "--quiet", "--message=main change"]);
    let main: String = text(fixture, repo, &["rev-parse", "HEAD"]);
    return (side, main);
}

fn text(fixture: &Fixture, repo: &Path, arguments: &[&str]) -> String {
    return super::service_landing_support::text_of(fixture, repo, arguments).trim().to_string();
}

fn resolve(fixture: &Fixture, repo: &Path) {
    write(repo, "b.txt", "resolved\n");
    git(fixture, repo, &["add", "b.txt"]);
}

/// A repository in each state a conclusion can be in, made once and compared case by case.
fn conclusion_repositories(fixture: &Fixture, name: &str) -> Vec<(&'static str, PathBuf)> {
    let mut made: Vec<(&'static str, PathBuf)> = Vec::new();
    let template: PathBuf = base_repository(fixture);
    let states: [&'static str; 7] = ["conflicted merge, resolved", "merge without a commit", "squash merge", "conflicted cherry-pick, resolved", "conflicted revert, resolved", "two picks, first resolved", "merge of a side branch with a new file"];
    for (number, label) in states.into_iter().enumerate() {
        let repo: PathBuf = copy_beside(fixture, template.as_path(), format!("{name}-{number}").as_str());
        let (side, main) = diverged(fixture, &repo);
        match number {
            0 => {
                attempt(fixture, &repo, &["merge", "side"]);
                resolve(fixture, &repo);
            }
            1 => {
                git(fixture, &repo, &["checkout", "--quiet", "-b", "clean-side", "HEAD~1"]);
                write(&repo, "n.txt", "new\n");
                git(fixture, &repo, &["add", "n.txt"]);
                git(fixture, &repo, &["commit", "--quiet", "--message=new file"]);
                git(fixture, &repo, &["checkout", "--quiet", "main"]);
                attempt(fixture, &repo, &["merge", "--no-commit", "--no-ff", "clean-side"]);
            }
            2 => {
                git(fixture, &repo, &["checkout", "--quiet", "-b", "clean-squash", "HEAD~1"]);
                write(&repo, "q.txt", "squashed\n");
                git(fixture, &repo, &["add", "q.txt"]);
                git(fixture, &repo, &["commit", "--quiet", "--message=for squash"]);
                git(fixture, &repo, &["checkout", "--quiet", "main"]);
                attempt(fixture, &repo, &["merge", "--squash", "clean-squash"]);
            }
            3 => {
                attempt(fixture, &repo, &["cherry-pick", side.as_str()]);
                resolve(fixture, &repo);
            }
            4 => {
                attempt(fixture, &repo, &["revert", main.as_str()]);
                write(&repo, "b.txt", "reverted by hand\n");
                git(fixture, &repo, &["add", "b.txt"]);
            }
            5 => {
                git(fixture, &repo, &["checkout", "--quiet", "side"]);
                write(&repo, "t.txt", "second pick\n");
                git(fixture, &repo, &["add", "t.txt"]);
                git(fixture, &repo, &["commit", "--quiet", "--message=second pick"]);
                let second: String = text(fixture, &repo, &["rev-parse", "HEAD"]);
                git(fixture, &repo, &["checkout", "--quiet", "main"]);
                attempt(fixture, &repo, &["cherry-pick", side.as_str(), second.as_str()]);
                resolve(fixture, &repo);
            }
            _ => {
                git(fixture, &repo, &["checkout", "--quiet", "-b", "tidy", "HEAD~1"]);
                write(&repo, "u.txt", "tidy\n");
                git(fixture, &repo, &["add", "u.txt"]);
                git(fixture, &repo, &["commit", "--quiet", "--message=tidy"]);
                git(fixture, &repo, &["checkout", "--quiet", "main"]);
                attempt(fixture, &repo, &["merge", "--no-ff", "--no-commit", "tidy"]);
                write(&repo, "u.txt", "tidy, then edited\n");
                git(fixture, &repo, &["add", "u.txt"]);
            }
        }
        made.push((label, repo));
    }
    return made;
}

/// The conclusion of a merge, a cherry-pick and a revert lands through the service, leaves the state files as native Git
///  leaves them, and has the parents native Git gives (the merged commit after the branch tip).
#[test]
fn a_conclusion_lands_through_the_service_and_removes_the_state_native_git_removes() {
    let fixture: Fixture = fixture("landing-conclusion");
    let name: String = format!("landing-conclusion-{}", std::process::id());
    let mut service: Running = start(&fixture, name.as_str());
    let states = conclusion_repositories(&fixture, "state");
    for (label, repo) in &states {
        let lines: Vec<Case> = vec![
            Case { index_commit: true, ..case("no-edit message", &["commit", "--no-edit"]) },
            Case { index_commit: true, ..case("typed message", &["commit", "-m", "typed"]) },
            Case { index_commit: true, ..case("editor that accepts the prepared message", &["commit"]) },
            Case { index_commit: true, ..case("commit all", &["commit", "-a", "-m", "all"]) },
        ];
        for line in &lines {
            let described: Case = Case { name: Box::leak(format!("{label}: {}", line.name).into_boxed_str()), index_commit: line.index_commit, ..case("x", line.command) };
            let (native, served) = compare(&fixture, repo, name.as_str(), &described);
            // A conclusion removes its own markers: in the states that make a commit, none is left in either copy.
            if native.code == Some(0) {
                for (state, value) in served.left.iter().filter(|(entry, _)| { return ["state MERGE_HEAD", "state CHERRY_PICK_HEAD", "state REVERT_HEAD", "state MERGE_MSG", "state MERGE_MODE", "state SQUASH_MSG"].contains(&entry.as_str()); }) {
                    assert_eq!(value, "(absent)", "{label}: {state} is removed by the landing");
                }
            }
        }
    }
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
}

/// A detached head is moved as `HEAD` itself: the commit lands on it, the branches stay, and the head stays detached.
#[test]
fn a_commit_on_a_detached_head_moves_head_and_no_branch() {
    let fixture: Fixture = fixture("landing-detached");
    let base: PathBuf = base_repository(&fixture);
    git(&fixture, &base, &["checkout", "--quiet", "--detach"]);
    write(&base, "a.txt", "changed\n");
    write(&base, "b.txt", "changed b\n");
    git(&fixture, &base, &["add", "b.txt"]);
    let name: String = format!("landing-detached-{}", std::process::id());
    let mut service: Running = start(&fixture, name.as_str());
    for entry in [
        case("named path", &["commit", "-m", "detached", "--", "a.txt"]),
        Case { index_commit: true, ..case("index", &["commit", "-m", "detached"]) },
        Case { index_commit: true, ..case("all", &["commit", "-a", "-m", "detached"]) },
        Case { index_commit: true, ..case("amend", &["commit", "--amend", "--no-edit"]) },
    ] {
        let (native, served) = compare(&fixture, &base, name.as_str(), &entry);
        assert_eq!(native.code, Some(0), "{}", entry.name);
        let symbolic = served.left.iter().find(|(label, _)| { return label == "symbolic head"; }).map(|(_, value)| { return value.clone(); });
        assert_eq!(symbolic.as_deref(), Some(""), "{}: the head is still detached", entry.name);
    }
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
}

/// A branch that does not exist yet gets its first commit by a compare against the all-zero name.
#[test]
fn the_first_commit_of_a_branch_lands_through_the_service() {
    let fixture: Fixture = fixture("landing-unborn");
    let base: PathBuf = fixture.root.join("unborn");
    std::fs::create_dir(&base).expect("directory");
    git(&fixture, &base, &["init", "--quiet", "--initial-branch=main"]);
    git(&fixture, &base, &["config", "maintenance.autoDetach", "false"]);
    write(&base, "a.txt", "first\n");
    write(&base, "b.txt", "second\n");
    write(&base, "c.txt", "untracked\n");
    git(&fixture, &base, &["add", "a.txt", "b.txt"]);
    let name: String = format!("landing-unborn-{}", std::process::id());
    let mut service: Running = start(&fixture, name.as_str());
    for entry in [
        Case { index_commit: true, ..case("index", &["commit", "-m", "first"]) },
        case("named path", &["commit", "-m", "first", "--", "a.txt"]),
        Case { index_commit: true, ..case("empty commit", &["commit", "--allow-empty", "-m", "first"]) },
        Case { index_commit: true, ..case("nothing to amend", &["commit", "--amend", "-m", "first"]) },
        Case { index_commit: true, full_output: true, ..case("editor-free message", &["commit", "--no-edit"]) },
    ] {
        compare(&fixture, &base, name.as_str(), &entry);
    }
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
}

/// `-a` stages the working copy of a conflicted file, which is a resolution, so the service commits it as native Git does. A plain
///  commit with unmerged files is no longer refused as native Git refuses it: it finishes the merge with a warning
///  (`binary_commit_during_merge_tests.rs`, owner, 2026-11-03: "Warn and finish the merge").
#[test]
fn a_commit_with_all_staged_resolves_an_unmerged_file_as_native_git_does() {
    let fixture: Fixture = fixture("landing-unmerged");
    let base: PathBuf = base_repository(&fixture);
    diverged(&fixture, &base);
    attempt(&fixture, &base, &["merge", "side"]);
    write(&base, "c.txt", "edited elsewhere\n");
    let name: String = format!("landing-unmerged-{}", std::process::id());
    let mut service: Running = start(&fixture, name.as_str());
    // `-a` stages the working copy of the conflicted file, which is a resolution, so native Git commits it and so must the service.
    let resolved = Case { index_commit: true, ..case("all", &["commit", "-a", "-m", "x"]) };
    let (native, _served) = compare(&fixture, &base, name.as_str(), &resolved);
    assert_eq!(native.code, Some(0), "native Git commits the staged working copy");
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
}

/// Nothing to commit is said in Git's own words with Git's own status, and nothing lands.
#[test]
fn nothing_to_commit_is_said_in_the_words_of_native_git() {
    let fixture: Fixture = fixture("landing-nothing");
    let base: PathBuf = base_repository(&fixture);
    write(&base, "d.txt", "untracked\n");
    let name: String = format!("landing-nothing-{}", std::process::id());
    let mut service: Running = start(&fixture, name.as_str());
    for entry in [
        Case { index_commit: true, full_output: true, ..case("index, only untracked files", &["commit", "-m", "x"]) },
        Case { index_commit: true, full_output: true, ..case("all, only untracked files", &["commit", "-a", "-m", "x"]) },
        Case { full_output: true, ..case("a tracked path with no change", &["commit", "-m", "x", "--", "c.txt"]) },
    ] {
        let (native, _served) = compare(&fixture, &base, name.as_str(), &entry);
        assert_ne!(native.code, Some(0), "{}: native Git has nothing to commit", entry.name);
    }
    git(&fixture, &base, &["clean", "--force", "--quiet", "-d"]);
    let clean = Case { index_commit: true, full_output: true, ..case("a clean tree", &["commit", "-m", "x"]) };
    compare(&fixture, &base, name.as_str(), &clean);
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
}

/// A policy rewrite is installed into the worktree after the landing; the commit holds the corrected file, as a commit of
///  the corrected file by native Git does.
#[test]
fn a_policy_rewrite_lands_and_reaches_the_worktree() {
    let fixture: Fixture = fixture("landing-rewrite");
    let base: PathBuf = base_repository(&fixture);
    let name: String = format!("landing-rewrite-{}", std::process::id());
    let mut service: Running = start(&fixture, name.as_str());
    let served_repo: PathBuf = copy_beside(&fixture, &base, "served");
    // Native Git is given the corrected file; the policy corrects the other copy.
    write(&base, "a.txt", "changed\n");
    write(&served_repo, "a.txt", "changed");
    let entry: Case = case("a missing final newline", &["commit", "-m", "corrected", "--", "a.txt"]);
    let before: usize = finished(&fixture);
    let native = run_native(&fixture, &base, &entry);
    let served = run_served(&fixture, &served_repo, name.as_str(), &entry);
    assert_eq!(served.code, native.code, "{}", served.output);
    for ((label, expected), (_, found)) in native.left.iter().zip(&served.left) {
        assert_eq!(found, expected, "{label}");
    }
    assert_eq!(std::fs::read(served_repo.join("a.txt")).expect("worktree copy"), b"changed\n");
    assert_eq!(finished(&fixture), before + 1, "the service landed it");
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
}

/// Change lists the embedded route cannot carry are built by Git and landed: more paths than a request may list, names that
///  are not text, a gitlink, an executable, a symbolic link, and a file replaced by a directory.
#[test]
fn awkward_change_lists_land_as_native_git_commits_them() {
    use std::os::unix::ffi::OsStrExt;
    let fixture: Fixture = fixture("landing-odd");
    let base: PathBuf = base_repository(&fixture);
    let many: PathBuf = base.join("many");
    std::fs::create_dir(&many).expect("directory");
    for number in 0..git_policy_cli::service_protocol::MAX_CHANGES + 100 {
        std::fs::write(many.join(format!("f{number}.txt")), format!("{number}\n")).expect("file");
    }
    std::fs::write(base.join(std::ffi::OsStr::from_bytes(b"caf\xe9.txt")), "not text\n").expect("non-UTF-8 name");
    std::fs::write(base.join("run.sh"), "#!/bin/sh\n").expect("script");
    let mut permissions = std::fs::metadata(base.join("run.sh")).expect("metadata").permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
    std::fs::set_permissions(base.join("run.sh"), permissions).expect("mode");
    std::os::unix::fs::symlink("a.txt", base.join("link")).expect("link");
    git(&fixture, &base, &["update-index", "--add", "--cacheinfo", &format!("160000,{},sub", text(&fixture, &base, &["rev-parse", "HEAD"]))]);
    std::fs::remove_file(base.join("c.txt")).expect("remove");
    std::fs::create_dir(base.join("c.txt")).expect("directory in place of the file");
    std::fs::write(base.join("c.txt/inner.txt"), "inside\n").expect("inner");
    git(&fixture, &base, &["add", "--all"]);
    let name: String = format!("landing-odd-{}", std::process::id());
    let mut service: Running = start(&fixture, name.as_str());
    for entry in [
        Case { index_commit: true, ..case("everything staged", &["commit", "-m", "awkward"]) },
        Case { index_commit: true, ..case("everything staged, with a message from a file", &["commit", "-F", "../message.txt"]) },
    ] {
        std::fs::write(fixture.root.join("message.txt"), "from a file\n").expect("message");
        compare(&fixture, &base, name.as_str(), &entry);
    }
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
}
