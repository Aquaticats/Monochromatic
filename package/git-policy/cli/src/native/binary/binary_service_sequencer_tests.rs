//! What:
//!  A commit with named paths while a revert, a cherry-pick or a merge is in progress, through the built wrapper and the
//!  resident service, compared with real Git on an identically prepared repository.
//! Why:
//!  The wrapper must accept what real Git accepts and refuse what it refuses, with the same status, the same words and the
//!  same repository afterwards. A commit that concludes a sequencer state is built by real Git in a scratch repository; the
//!  options the wrapper adds there must never follow the `--` the person typed, where Git reads every token as a path.
//!  The paths include names that look like options (`-dash.txt`, `--quiet`, `-o`) and a pathspec file.

use super::commit::{git_text, remove_settled};
use super::service_commit::{Running, start, through_service};
use super::support::{Fixture, direct, fixture, git};
use git_policy_cli::service_journal::DONE_EXTENSION;
use std::path::{Path, PathBuf};
use std::process::Output;

/// The tracked files every state changes; three of them are named like options.
pub(super) const TRACKED: [&str; 4] = ["a.txt", "-dash.txt", "--quiet", "-o"];

/// A raw real-Git run that may fail, in the fixture's hermetic environment.
fn try_git(fixture: &Fixture, repo: &Path, arguments: &[&str]) -> Output {
    return direct(fixture).current_dir(repo).args(arguments).output().expect("real Git runs");
}

/// Replaces every tracked file with `text` and stages the lot, naming each after `--` so an option-like name is a path.
fn write_all(fixture: &Fixture, repo: &Path, text: &str) {
    for name in TRACKED {
        std::fs::write(repo.join(name), text).expect("write tracked file");
    }
    let mut arguments: Vec<&str> = vec!["add", "--"];
    arguments.extend(TRACKED);
    git(fixture, repo, &arguments);
}

/// The sequencer state a control starts from.
#[derive(Clone, Copy, Debug)]
pub(super) enum State {
    /// `git revert --no-commit` of a clean change.
    Revert,
    /// `git cherry-pick --no-commit` of a clean change.
    CherryPick,
    /// A revert that conflicted in `a.txt`, resolved and staged.
    ConflictedRevert,
    /// A cherry-pick that conflicted in `a.txt`, resolved and staged.
    ConflictedCherryPick,
    /// `git merge --no-commit --no-ff` of a clean change.
    Merge,
}

/// Methods over the states.
impl State {
    /// The words the control names itself by.
    fn label(self) -> &'static str {
        return match self {
            State::Revert => "revert",
            State::CherryPick => "cherry-pick",
            State::ConflictedRevert => "conflicted-revert",
            State::ConflictedCherryPick => "conflicted-cherry-pick",
            State::Merge => "merge",
        };
    }

    /// A short tag for the service's socket path, which must fit in a few dozen characters.
    fn tag(self) -> &'static str {
        return match self {
            State::Revert => "rv",
            State::CherryPick => "cp",
            State::ConflictedRevert => "crv",
            State::ConflictedCherryPick => "ccp",
            State::Merge => "mg",
        };
    }

    /// Whether real Git 2.56.0 takes a partial commit in this state (measured; each control also checks it against real Git).
    fn git_takes_paths(self) -> bool {
        return matches!(self, State::Revert | State::CherryPick | State::ConflictedRevert);
    }

    /// A repository in this state: a base commit of the tracked files, a change to all of them on `other`, and the state made.
    pub(super) fn prepare(self, fixture: &Fixture, name: &str) -> PathBuf {
        let repo: PathBuf = super::support::repository(fixture, std::ffi::OsStr::new(name));
        git(fixture, &repo, &["config", "maintenance.autoDetach", "false"]);
        write_all(fixture, &repo, "base\n");
        std::fs::write(repo.join("b.txt"), "base\n").expect("b.txt");
        git(fixture, &repo, &["add", "--", "b.txt"]);
        git(fixture, &repo, &["commit", "--quiet", "--message=base"]);
        git(fixture, &repo, &["switch", "--quiet", "--create", "other"]);
        write_all(fixture, &repo, "changed\n");
        git(fixture, &repo, &["commit", "--quiet", "--message=change"]);
        git(fixture, &repo, &["switch", "--quiet", "main"]);
        match self {
            State::Revert => {
                git(fixture, &repo, &["merge", "--quiet", "--ff-only", "other"]);
                git(fixture, &repo, &["revert", "--no-commit", "HEAD"]);
            }
            State::CherryPick => {
                git(fixture, &repo, &["cherry-pick", "--no-commit", "other"]);
            }
            State::ConflictedRevert => {
                git(fixture, &repo, &["merge", "--quiet", "--ff-only", "other"]);
                std::fs::write(repo.join("a.txt"), "later\n").expect("a.txt");
                git(fixture, &repo, &["commit", "--quiet", "--message=later", "--", "a.txt"]);
                let reverted: Output = try_git(fixture, &repo, &["revert", "--no-edit", "HEAD~1"]);
                assert!(!reverted.status.success(), "the revert was meant to conflict");
                std::fs::write(repo.join("a.txt"), "resolved\n").expect("a.txt");
                git(fixture, &repo, &["add", "--", "a.txt"]);
            }
            State::ConflictedCherryPick => {
                std::fs::write(repo.join("a.txt"), "main\n").expect("a.txt");
                git(fixture, &repo, &["commit", "--quiet", "--message=main", "--", "a.txt"]);
                let picked: Output = try_git(fixture, &repo, &["cherry-pick", "other"]);
                assert!(!picked.status.success(), "the cherry-pick was meant to conflict");
                std::fs::write(repo.join("a.txt"), "resolved\n").expect("a.txt");
                git(fixture, &repo, &["add", "--", "a.txt"]);
            }
            State::Merge => {
                std::fs::write(repo.join("b.txt"), "main\n").expect("b.txt");
                git(fixture, &repo, &["commit", "--quiet", "--message=main", "--", "b.txt"]);
                git(fixture, &repo, &["merge", "--no-commit", "--no-ff", "other"]);
            }
        }
        return repo;
    }
}

/// What a finished commit shows the person and leaves behind, with the parts that differ between two runs masked.
#[derive(Debug, Eq, PartialEq)]
struct Outcome {
    code: Option<i32>,
    stdout: String,
    stderr: String,
    tree: String,
    parents: usize,
    message: String,
    status: String,
    markers: String,
}

/// Masks the commit name in Git's `[branch name] subject` line and the date line, which differ between two runs.
fn masked(text: &str) -> String {
    let mut lines: Vec<String> = Vec::new();
    for line in text.lines() {
        if line.starts_with(" Date:") {
            lines.push(String::from(" Date: MASKED"));
            continue;
        }
        let words: Vec<String> = line
            .split(' ')
            .map(|word| {
                let name: &str = word.trim_end_matches(']');
                let named: bool = word.ends_with(']') && name.len() >= 7 && name.chars().all(|letter| { return letter.is_ascii_hexdigit(); });
                return if named { String::from("NAME]") } else { word.to_string() };
            })
            .collect();
        lines.push(words.join(" "));
    }
    return lines.join("\n");
}

/// The outcome of one run in `repo`.
fn outcome(fixture: &Fixture, repo: &Path, run: &Output) -> Outcome {
    let markers: Vec<String> = ["MERGE_HEAD", "CHERRY_PICK_HEAD", "REVERT_HEAD", "MERGE_MSG", "COMMIT_EDITMSG"]
        .iter()
        .filter(|name| { return repo.join(".git").join(name).exists(); })
        .map(|name| { return (*name).to_string(); })
        .collect();
    return Outcome {
        code: run.status.code(),
        stdout: masked(String::from_utf8_lossy(&run.stdout).as_ref()),
        stderr: String::from_utf8_lossy(&run.stderr).into_owned(),
        tree: git_text(fixture, repo, &["rev-parse", "HEAD^{tree}"]),
        parents: git_text(fixture, repo, &["rev-list", "--parents", "-1", "HEAD"]).split_whitespace().count() - 1,
        message: git_text(fixture, repo, &["log", "-1", "--format=%an <%ae>|%B"]),
        status: git_text(fixture, repo, &["status", "--short"]),
        markers: markers.join(","),
    };
}

/// The number of batches the service finished, one for each commit it landed.
fn landed(fixture: &Fixture) -> usize {
    return std::fs::read_dir(fixture.root.join("journal")).map_or(0, |entries| {
        return entries
            .flatten()
            .filter(|entry| { return entry.path().extension().is_some_and(|extension| { return extension == DONE_EXTENSION; }); })
            .count();
    });
}

/// Every command line a control puts to the wrapper and to real Git, beyond `commit --message=undo`.
fn command_lines(fixture: &Fixture) -> Vec<(&'static str, Vec<String>)> {
    let list_of = |name: &str, text: &str| -> String {
        let file: PathBuf = fixture.root.join(name);
        std::fs::write(&file, text).expect("pathspec file");
        return format!("--pathspec-from-file={}", file.display());
    };
    let named = |tail: &[&str]| -> Vec<String> {
        let mut line: Vec<String> = vec![String::from("commit"), String::from("--message=undo")];
        line.extend(tail.iter().map(|word| { return (*word).to_string(); }));
        return line;
    };
    return vec![
        ("plain pathspec", named(&["--", "a.txt"])),
        ("pathspec file", named(&[list_of("paths-plain", "a.txt\n").as_str()])),
        ("dash-led pathspec", named(&["--", "-dash.txt"])),
        ("file named like an option", named(&["--", "--quiet"])),
        ("file named like a commit option", named(&["--", "-o"])),
        ("dash-led name in a pathspec file", named(&[list_of("paths-dash", "-dash.txt\n").as_str()])),
        ("typed quiet", vec![String::from("commit"), String::from("--quiet"), String::from("--message=undo"), String::from("--"), String::from("a.txt")]),
        ("message that is a separator", vec![String::from("commit"), String::from("-m"), String::from("--"), String::from("--"), String::from("a.txt")]),
    ];
}

/// One state, every command line: the wrapper through the service and real Git agree on everything the person can see and
///  everything left behind, and a commit real Git takes lands through the service.
fn agrees_with_git(state: State) {
    let label: String = format!("sequencer-{}", state.label());
    let fixture: Fixture = fixture(label.as_str());
    let lines: Vec<(&'static str, Vec<String>)> = command_lines(&fixture);
    let pairs: Vec<(PathBuf, PathBuf)> = (0..lines.len())
        .map(|index| {
            return (state.prepare(&fixture, format!("wrapped-{index}").as_str()), state.prepare(&fixture, format!("native-{index}").as_str()));
        })
        .collect();
    let name: String = format!("sq-{}-{}", state.tag(), std::process::id());
    let mut service: Running = start(&fixture, name.as_str());
    let mut differences: Vec<String> = Vec::new();
    for ((what, line), (wrapped_repo, native_repo)) in lines.iter().zip(pairs.iter()) {
        let words: Vec<&str> = line.iter().map(String::as_str).collect();
        let before: usize = landed(&fixture);
        let served: Output = through_service(&fixture, wrapped_repo, name.as_str(), &words);
        let native: Output = direct(&fixture).current_dir(native_repo).args(&words).output().expect("real Git runs");
        assert_eq!(native.status.success(), state.git_takes_paths(), "{} / {what}: real Git: {}", state.label(), String::from_utf8_lossy(&native.stderr));
        let wrapped_outcome: Outcome = outcome(&fixture, wrapped_repo, &served);
        let native_outcome: Outcome = outcome(&fixture, native_repo, &native);
        if wrapped_outcome != native_outcome {
            differences.push(format!("{} / {what}: the wrapper and real Git differ\n  wrapper: {wrapped_outcome:?}\n  real Git: {native_outcome:?}", state.label()));
        }
        let landed_now: usize = landed(&fixture) - before;
        if landed_now != usize::from(native.status.success()) {
            differences.push(format!("{} / {what}: a commit lands through the service exactly when Git takes it, landed {landed_now}", state.label()));
        }
    }
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

/// Named paths while `git revert --no-commit` is in progress.
#[test]
fn a_pathspec_commit_during_a_revert_is_what_git_makes() {
    agrees_with_git(State::Revert);
}

/// Named paths while `git cherry-pick --no-commit` is in progress.
#[test]
fn a_pathspec_commit_during_a_cherry_pick_is_what_git_makes() {
    agrees_with_git(State::CherryPick);
}

/// Named paths after a conflicted revert was resolved.
#[test]
fn a_pathspec_commit_after_a_conflicted_revert_is_what_git_makes() {
    agrees_with_git(State::ConflictedRevert);
}

/// One state that real Git refuses named paths in, every command line: real Git refuses, and the wrapper through the service
///  lands an ordinary commit of the named paths on `HEAD` and carries the state forward (the owner's rule that commits never
///  fail, `SPEC.md` "Merges and commits never fail"). `marker` is the file that names the state in the Git directory.
fn lands_beside_the_state(state: State, marker: &str) {
    let label: String = format!("sequencer-carry-{}", state.label());
    let fixture: Fixture = fixture(label.as_str());
    let lines: Vec<(&'static str, Vec<String>)> = command_lines(&fixture);
    let pairs: Vec<(PathBuf, PathBuf)> = (0..lines.len())
        .map(|index| {
            return (state.prepare(&fixture, format!("wrapped-{index}").as_str()), state.prepare(&fixture, format!("native-{index}").as_str()));
        })
        .collect();
    let name: String = format!("sc-{}-{}", state.tag(), std::process::id());
    let mut service: Running = start(&fixture, name.as_str());
    let mut differences: Vec<String> = Vec::new();
    for ((what, line), (wrapped_repo, native_repo)) in lines.iter().zip(pairs.iter()) {
        let words: Vec<&str> = line.iter().map(String::as_str).collect();
        let before: usize = landed(&fixture);
        let base: String = git_text(&fixture, wrapped_repo, &["rev-parse", "HEAD"]).trim().to_owned();
        let served: Output = through_service(&fixture, wrapped_repo, name.as_str(), &words);
        let native: Output = direct(&fixture).current_dir(native_repo).args(&words).output().expect("real Git runs");
        assert!(!native.status.success(), "{} / {what}: real Git refuses named paths here", state.label());
        let parent: String = git_text(&fixture, wrapped_repo, &["rev-parse", "HEAD^"]).trim().to_owned();
        let changed: String = git_text(&fixture, wrapped_repo, &["diff-tree", "-r", "--name-only", "HEAD^", "HEAD"]);
        if served.status.code() != Some(0) || parent != base || changed.trim().is_empty() || !wrapped_repo.join(".git").join(marker).exists() || landed(&fixture) - before != 1 {
            differences.push(format!(
                "{} / {what}: the wrapper should land an ordinary commit and keep {marker}: code {:?}, parent {parent} (base {base}), changed {changed:?}, {}",
                state.label(),
                served.status.code(),
                String::from_utf8_lossy(&served.stderr)
            ));
        }
    }
    let _killed = service.kill();
    let _waited = service.wait();
    remove_settled(&fixture);
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

/// Named paths after a conflicted cherry-pick was resolved: Git refuses; the wrapper lands them and the pick is carried on.
#[test]
fn a_pathspec_commit_after_a_conflicted_cherry_pick_lands_and_carries_the_pick() {
    lands_beside_the_state(State::ConflictedCherryPick, "CHERRY_PICK_HEAD");
}

/// Named paths during a merge: Git refuses; the wrapper lands them and the merge is carried on.
#[test]
fn a_pathspec_commit_during_a_merge_lands_and_carries_the_merge() {
    lands_beside_the_state(State::Merge, "MERGE_HEAD");
}
