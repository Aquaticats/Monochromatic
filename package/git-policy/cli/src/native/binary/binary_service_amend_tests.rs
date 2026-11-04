//! What:
//!  `git commit --amend` landed through the service, compared field by field with the commit native `git commit --amend`
//!  makes from the same working state, including what is left staged, modified and untracked.
//! Why:
//!  An amend keeps the parents of the replaced commit, its author and (without a message option) its message; the service
//!  builds that from a tree the wrapper wrote. Any difference in a stored field, or in what the index and the worktree
//!  show afterwards, would be a silent change of what `--amend` means.

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

/// The stored fields that must agree, with the author's date; the committer's date is the time of the run and is left out.
fn stored(fixture: &Fixture, repo: &Path) -> Vec<String> {
    let formats: [&str; 6] = ["%T", "%B", "%an <%ae>", "%ad", "%cn <%ce>", "%P"];
    return formats.map(|format| { return git_text(fixture, repo, &["log", "-1", "--date=iso-strict", format!("--format={format}").as_str()]); }).to_vec();
}

/// What a commit left behind: the status, the staged difference from the commit and how many commits the branch has.
fn left_behind(fixture: &Fixture, repo: &Path) -> Vec<String> {
    return vec![
        git_text(fixture, repo, &["status", "--short"]),
        git_text(fixture, repo, &["diff", "--cached", "--name-status"]),
        git_text(fixture, repo, &["rev-list", "--count", "HEAD"]),
    ];
}

fn write(repo: &Path, name: &str, text: &str) {
    std::fs::write(repo.join(name), text).expect("write");
}

/// Which commit the branch is reset to before a case.
#[derive(Clone, Copy)]
enum Start {
    /// A commit with one parent, written by another person on another day.
    Second,
    /// The root commit.
    Root,
    /// A merge of two histories.
    Merge,
}

type Setup = fn(&Path);

fn nothing(_repo: &Path) {}

fn edit_a(repo: &Path) {
    write(repo, "a.txt", "base\nsecond\namended\n");
}

fn edit_a_and_write_message(repo: &Path) {
    edit_a(repo);
    write(repo, "message.txt", "from a file  \n\n\nsecond paragraph\n");
}

fn edit_a_and_stage(repo: &Path) {
    edit_a(repo);
    let output = std::process::Command::new("/usr/bin/git").current_dir(repo).args(["add", "a.txt"]).output().expect("add");
    assert!(output.status.success());
}

fn delete_b(repo: &Path) {
    std::fs::remove_file(repo.join("b.txt")).expect("remove");
}

fn edit_a_and_leave_b(repo: &Path) {
    edit_a(repo);
    write(repo, "b.txt", "other\nedited but not named\n");
    write(repo, "untracked.txt", "untracked\n");
}

/// Back to the content of the root commit, so the amended commit equals its parent.
fn edit_a_back_to_the_parent(repo: &Path) {
    write(repo, "a.txt", "base\n");
}

/// One command line, the state it starts from, and whether the service lands it.
struct Case {
    name: &'static str,
    start: Start,
    command: &'static [&'static str],
    prepare: Setup,
    /// Whether the author date is a stored field the line keeps (not renewed to the time of the run).
    date_kept: bool,
    /// Whether the service lands the line; otherwise it is declined and the transaction commits it.
    lands: bool,
}

fn landing(name: &'static str, command: &'static [&'static str], prepare: Setup) -> Case {
    return Case { name, start: Start::Second, command, prepare, date_kept: true, lands: true };
}

/// Every covered amend gives the commit native Git gives, leaves the repository as native Git leaves it, and the service
///  (not the transaction) landed it.
#[test]
fn amends_through_the_service_equal_the_native_amend() {
    let fixture: Fixture = fixture("service-amend");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    write(&repo, "a.txt", "base\n");
    write(&repo, "b.txt", "other\n");
    git(&fixture, &repo, &["add", "a.txt", "b.txt"]);
    git(&fixture, &repo, &["commit", "--quiet", "--message=root"]);
    git(&fixture, &repo, &["config", "maintenance.autoDetach", "false"]);
    let root: String = query(&fixture, &repo, &["rev-parse", "HEAD"]);
    git(&fixture, &repo, &["checkout", "--quiet", "-b", "side"]);
    write(&repo, "side.txt", "side\n");
    git(&fixture, &repo, &["add", "side.txt"]);
    git(&fixture, &repo, &["commit", "--quiet", "--message=side"]);
    git(&fixture, &repo, &["checkout", "--quiet", "main"]);
    write(&repo, "a.txt", "base\nsecond\n");
    git(&fixture, &repo, &["add", "a.txt"]);
    git(
        &fixture,
        &repo,
        &["commit", "--quiet", "--author=Original Author <original@example.invalid>", "--date=2019-03-04T05:06:07+0300", "--message=second subject\n\nsecond body  \n"],
    );
    let second: String = query(&fixture, &repo, &["rev-parse", "HEAD"]);
    git(&fixture, &repo, &["merge", "--quiet", "--no-ff", "--message=merge subject", "side"]);
    let merge: String = query(&fixture, &repo, &["rev-parse", "HEAD"]);
    let cases: Vec<Case> = vec![
        landing("new message", &["commit", "--amend", "-m", "reworded", "--", "a.txt"], edit_a),
        landing("new message and body", &["commit", "--amend", "-m", "reworded", "-m", "body  \n\n\n", "--", "a.txt"], edit_a),
        landing("message kept", &["commit", "--amend", "--no-edit", "--", "a.txt"], edit_a),
        landing("message kept, nothing changed", &["commit", "--amend", "--no-edit"], nothing),
        landing("message kept, staged index", &["commit", "--no-enforce-only", "--amend", "--no-edit"], edit_a_and_stage),
        landing("message kept, deletion", &["commit", "--amend", "--no-edit", "--", "b.txt"], delete_b),
        landing("only the named file", &["commit", "--amend", "--no-edit", "--", "a.txt"], edit_a_and_leave_b),
        landing("message from a file", &["commit", "--amend", "-F", "message.txt", "--", "a.txt"], edit_a_and_write_message),
        landing("sign-off", &["commit", "--amend", "--no-edit", "-s", "--", "a.txt"], edit_a),
        landing("trailer", &["commit", "--amend", "-m", "t", "--trailer", "Fixes: 12", "--", "a.txt"], edit_a),
        landing("author", &["commit", "--amend", "--no-edit", "--author=Other One <other@example.invalid>", "--", "a.txt"], edit_a),
        landing("date", &["commit", "--amend", "--no-edit", "--date=2021-02-03T04:05:06Z", "--", "a.txt"], edit_a),
        landing("author and date", &["commit", "--amend", "-m", "s", "--author=A <a@b.invalid>", "--date=2021-05-06T07:08:09+0200", "--", "a.txt"], edit_a),
        landing("reused commit", &["commit", "--amend", "-C", "HEAD~1", "--", "a.txt"], edit_a),
        landing("verbatim", &["commit", "--amend", "-m", "line  \n\n\n\ntrailing  ", "--cleanup=verbatim", "--", "a.txt"], edit_a),
        landing("allow empty against the parent", &["commit", "--amend", "--allow-empty", "--no-edit", "--", "a.txt"], edit_a_back_to_the_parent),
        Case { date_kept: false, ..landing("reset author", &["commit", "--amend", "--no-edit", "--reset-author", "--", "a.txt"], edit_a) },
        Case { start: Start::Root, ..landing("root commit", &["commit", "--amend", "-m", "reroot", "--", "a.txt"], edit_a) },
        Case { start: Start::Root, ..landing("root commit, message kept", &["commit", "--amend", "--no-edit", "--", "a.txt"], edit_a) },
        Case { start: Start::Merge, ..landing("merge commit", &["commit", "--amend", "-m", "remerged", "--", "a.txt"], edit_a) },
        Case { start: Start::Merge, ..landing("merge commit, message kept, nothing changed", &["commit", "--amend", "--no-edit"], nothing) },
        // The amended commit would equal its parent: native Git refuses or accepts it as it always did, and the transaction does that.
        Case { lands: false, ..landing("empty against the parent", &["commit", "--amend", "--no-edit", "--", "a.txt"], edit_a_back_to_the_parent) },
    ];
    let name: String = format!("amend-{}", std::process::id());
    let mut service: Running = start(&fixture, name.as_str());
    for entry in &cases {
        let target: &str = match entry.start {
            Start::Second => second.as_str(),
            Start::Root => root.as_str(),
            Start::Merge => merge.as_str(),
        };
        let mut outcomes: Vec<(Vec<String>, Vec<String>, Option<i32>)> = Vec::new();
        for through in [true, false] {
            git(&fixture, &repo, &["reset", "--hard", "--quiet", target]);
            git(&fixture, &repo, &["clean", "--force", "--quiet", "-d"]);
            (entry.prepare)(&repo);
            let before: usize = finished(&fixture);
            let output: Output = if through {
                through_service(&fixture, &repo, name.as_str(), entry.command)
            } else {
                direct(&fixture).current_dir(&repo).args(entry.command.iter().copied().filter(|word| { return *word != "--no-enforce-only"; })).output().expect("native")
            };
            if through {
                let expected: usize = before + usize::from(entry.lands);
                assert_eq!(finished(&fixture), expected, "{}: {}", entry.name, String::from_utf8_lossy(&output.stderr));
            }
            outcomes.push((stored(&fixture, &repo), left_behind(&fixture, &repo), output.status.code()));
        }
        assert_eq!(outcomes[0].2, outcomes[1].2, "{}: exit status", entry.name);
        assert_eq!(outcomes[0].1, outcomes[1].1, "{}: what is left behind", entry.name);
        for (index, (left, right)) in outcomes[0].0.iter().zip(&outcomes[1].0).enumerate() {
            // Field 3 is the author date; a renewed one is the time of the run, which differs between runs.
            if index == 3 && !entry.date_kept {
                continue;
            }
            assert_eq!(left, right, "{}: stored field {index}", entry.name);
        }
        if entry.lands {
            assert_eq!(outcomes[0].2, Some(0), "{}", entry.name);
        }
    }
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
}

/// An amend in a repository whose configuration the service would ignore (here a clean-up mode) is built by real Git and
///  landed by the service, and the setting holds.
#[test]
fn an_amend_with_a_configuration_the_service_would_ignore_has_git_build_it() {
    let fixture: Fixture = fixture("service-amend-declined");
    let repo: PathBuf = repository(&fixture, OsStr::new("repo"));
    write(&repo, "a.txt", "base\n");
    git(&fixture, &repo, &["add", "a.txt"]);
    git(&fixture, &repo, &["commit", "--quiet", "--message=base"]);
    git(&fixture, &repo, &["config", "maintenance.autoDetach", "false"]);
    git(&fixture, &repo, &["config", "commit.cleanup", "verbatim"]);
    let name: String = format!("amend-declined-{}", std::process::id());
    let mut service: Running = start(&fixture, name.as_str());
    write(&repo, "a.txt", "base\nmore\n");
    let before: usize = finished(&fixture);
    let output: Output = through_service(&fixture, &repo, name.as_str(), &["commit", "--amend", "-m", "kept   \n\n\n", "--", "a.txt"]);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(finished(&fixture), before + 1, "Git built it and the service landed it");
    assert_eq!(git_text(&fixture, &repo, &["log", "-1", "--format=%B"]), "kept   \n\n\n\n", "the setting held");
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
}
