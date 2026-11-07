//! What:
//!  Index commits and `commit -a` landed through the service,
//! compared with the commit native `git commit` makes from the
//!  same working state,
//! including what is left staged,
//! modified and untracked afterwards.
//! Why:
//!  The service takes the whole index (and,
//! for `-a`,
//! every tracked change);
//! it must leave the index and the worktree
//!  exactly as native Git does,
//! so the next command of the person sees the same repository.

use super::commit::{git_text, remove_settled};
use super::service_commit::{Running, query, start, through_service};
use super::support::{Fixture, direct, fixture, git, repository};
use git_policy_cli::service_journal::DONE_EXTENSION;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Output;

fn finished(fixture: &Fixture) -> usize {
    return std::fs::read_dir(fixture.root.join("journal")).map_or(0, |entries| {
        return entries
            .flatten()
            .filter(|entry| { return entry.path().extension().is_some_and(|extension| { return extension == DONE_EXTENSION; }); })
            .count();
    });
}

/// What a commit left behind:
/// the stored tree and message,
/// the short status,
/// and the staged difference from the commit.
fn left_behind(fixture: &Fixture, repo: &Path) -> Vec<String> {
    return vec![
        git_text(fixture, repo, &["log", "-1", "--format=%T%n%B"]),
        git_text(fixture, repo, &["status", "--short"]),
        git_text(fixture, repo, &["diff", "--cached", "--name-status"]),
        git_text(fixture, repo, &["rev-list", "--count", "HEAD"]),
    ];
}

type Setup = fn(&Path);

/// One working state,
/// and whether anything can be committed from it.
struct Case {
    name: &'static str,
    command: &'static [&'static str],
    prepare: Setup,
    commits: bool,
}

fn write(repo: &Path, name: &str, text: &str) {
    std::fs::write(repo.join(name), text).expect("write");
}

fn stage_two_and_leave_one(repo: &Path) {
    write(repo, "a.txt", "base\nstaged\n");
    write(repo, "c.txt", "new file\n");
    write(repo, "b.txt", "other\nunstaged\n");
    let output = std::process::Command::new("/usr/bin/git").current_dir(repo).args(["add", "a.txt", "c.txt"]).output().expect("add");
    assert!(output.status.success());
}

fn edit_two_and_add_untracked(repo: &Path) {
    write(repo, "a.txt", "base\nedited\n");
    write(repo, "b.txt", "other\nedited too\n");
    write(repo, "d.txt", "untracked\n");
}

fn delete_one_and_edit_one(repo: &Path) {
    std::fs::remove_file(repo.join("b.txt")).expect("remove");
    write(repo, "a.txt", "base\nedited\n");
}

fn nothing_staged(repo: &Path) {
    write(repo, "a.txt", "base\nunstaged only\n");
}

/// Each working state commits the same through the service as natively,
/// and what is left behind is the same.
#[test]
fn index_commits_and_commit_all_through_the_service_equal_the_native_commit() {
    let fixture: Fixture = fixture("service-selection");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    write(&repo, "a.txt", "base\n");
    write(&repo, "b.txt", "other\n");
    git(&fixture, &repo, &["add", "a.txt", "b.txt"]);
    git(&fixture, &repo, &["commit", "--quiet", "--message=base"]);
    git(&fixture, &repo, &["config", "maintenance.autoDetach", "false"]);
    let start_commit: String = query(&fixture, &repo, &["rev-parse", "HEAD"]);
    let cases: [Case; 5] = [
        Case { name: "index with staged and unstaged", command: &["commit", "--no-enforce-only", "-m", "s"], prepare: stage_two_and_leave_one, commits: true },
        Case { name: "all", command: &["commit", "--no-enforce-only", "-a", "-m", "s"], prepare: edit_two_and_add_untracked, commits: true },
        Case { name: "all with a deletion", command: &["commit", "--no-enforce-only", "--all", "-m", "s"], prepare: delete_one_and_edit_one, commits: true },
        Case { name: "all short form together", command: &["commit", "--no-enforce-only", "-am", "s"], prepare: edit_two_and_add_untracked, commits: true },
        Case { name: "nothing staged", command: &["commit", "--no-enforce-only", "-m", "s"], prepare: nothing_staged, commits: false },
    ];
    let name: String = format!("selection-{}", std::process::id());
    let mut service: Running = start(&fixture, name.as_str());
    for entry in &cases {
        let mut outcomes: Vec<(Vec<String>, Option<i32>)> = Vec::new();
        for through in [true, false] {
            git(&fixture, &repo, &["reset", "--hard", "--quiet", start_commit.as_str()]);
            git(&fixture, &repo, &["clean", "--force", "--quiet", "-d"]);
            (entry.prepare)(&repo);
            let before: usize = finished(&fixture);
            let output: Output = if through {
                through_service(&fixture, &repo, name.as_str(), entry.command)
            } else {
                direct(&fixture).current_dir(&repo).args(entry.command.iter().copied().filter(|word| { return *word != "--no-enforce-only"; })).output().expect("native")
            };
            if through {
                let expected: usize = before + usize::from(entry.commits);
                assert_eq!(finished(&fixture), expected, "{}: {}", entry.name, String::from_utf8_lossy(&output.stderr));
            }
            outcomes.push((left_behind(&fixture, &repo), output.status.code()));
        }
        assert_eq!(outcomes[0], outcomes[1], "{}", entry.name);
        assert_eq!(outcomes[0].1 == Some(0), entry.commits, "{}", entry.name);
    }
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
}
