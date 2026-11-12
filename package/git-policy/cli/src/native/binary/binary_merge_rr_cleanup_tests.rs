//! What:
//!  A conflicted merge that a wrapped commit concludes,
//!  compared with real Git on the rerere state it leaves:
//!  `MERGE_RR`,
//!  the cache `git rerere` rewrites while a merge waits.
//! Why:
//!  Real `git commit` runs rerere and rewrites `MERGE_RR` as it records the resolution;
//! a wrapped commit runs in the shadow repository,
//! and the worktree must end with the same bytes the same commit leaves natively.
//! A copy-back that keeps an entry native Git cleared names a conflict the merge no longer has.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await mergeSide(); write('a.txt', 'resolved\n'); add('a.txt');
//! expecting(<git> commit --no-edit).toLeaveTheSameMergeRrAsNativeGit();
//! ```

use super::service_commit::{Running, start};
use super::service_landing_support::{Case, case, compare};
use super::support::{Fixture, direct, fixture, git};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Runs real Git and accepts any status,
/// because a conflicted merge exits 1.
fn attempt(fixture: &Fixture, repo: &Path, arguments: &[&str]) {
    let mut command: Command = direct(fixture);
    command.current_dir(repo).args(arguments).env("GIT_EDITOR", "true");
    let _output = command.output().expect("Git starts");
}

/// What:
///  A repository whose merge of `side` conflicts on a.txt,
/// with rerere on so Git writes `MERGE_RR`.
/// Why:
///   The cache is what this control is about,
/// and Git writes it only when rerere is enabled.
fn conflicted_merge_with_rerere(fixture: &Fixture, name: &str) -> PathBuf {
    let repo: PathBuf = super::support::repository(fixture, std::ffi::OsStr::new(name));
    git(fixture, repo.as_path(), &["config", "rerere.enabled", "true"]);
    std::fs::write(repo.join("a.txt"), "base\n").expect("tracked file");
    git(fixture, repo.as_path(), &["add", "--", "a.txt"]);
    git(fixture, repo.as_path(), &["commit", "--quiet", "--message=base"]);
    git(fixture, repo.as_path(), &["checkout", "--quiet", "-b", "side"]);
    std::fs::write(repo.join("a.txt"), "side\n").expect("side edit");
    git(fixture, repo.as_path(), &["commit", "--quiet", "--all", "--message=side"]);
    git(fixture, repo.as_path(), &["checkout", "--quiet", "main"]);
    std::fs::write(repo.join("a.txt"), "main\n").expect("main edit");
    git(fixture, repo.as_path(), &["commit", "--quiet", "--all", "--message=main"]);
    return repo;
}

/// A conflicted merge committed through the service leaves `MERGE_RR` as real Git leaves it.
#[test]
fn a_concluded_merge_leaves_the_rerere_cache_as_native_git_does() {
    let fixture: Fixture = fixture("landing-merge-rr");
    let name: String = format!("landing-merge-rr-{}", std::process::id());
    let mut service: Running = start(&fixture, name.as_str());
    let repo: PathBuf = conflicted_merge_with_rerere(&fixture, "merge-rr");
    attempt(&fixture, repo.as_path(), &["merge", "side"]);
    assert!(
        repo.join(".git/MERGE_RR").exists(),
        "the conflicted merge wrote MERGE_RR: {}",
        std::fs::read_to_string(repo.join(".git/MERGE_RR")).unwrap_or_default()
    );
    std::fs::write(repo.join("a.txt"), "resolved\n").expect("resolution");
    git(&fixture, repo.as_path(), &["add", "--", "a.txt"]);
    let line: Case = Case { index_commit: true, ..case("conclude the merge", &["commit", "--no-edit"]) };
    let (native, served) = compare(&fixture, repo.as_path(), name.as_str(), &line);
    let _killed = service.kill();
    let _waited = service.wait();
    assert_eq!(native.code, Some(0), "{}", native.output);
    assert_eq!(
        served.left.iter().find(|(label, _)| { return label == "state MERGE_RR"; }).map(|(_, value)| { return value.as_str(); }),
        native.left.iter().find(|(label, _)| { return label == "state MERGE_RR"; }).map(|(_, value)| { return value.as_str(); }),
        "MERGE_RR after the commit"
    );
}

/// The bytes of a file,
/// or the text that says it is absent.
fn bytes_of(path: &Path) -> String {
    let Ok(bytes) = std::fs::read(path) else {
        return String::from("(absent)");
    };
    return format!("{bytes:?}");
}

/// A merge whose one conflicted file the wrapper's own pass resolves,
/// finished inside the merge command,
/// leaves `MERGE_RR` as real Git leaves the same conclusion.
#[test]
fn a_merge_the_wrapper_finishes_itself_leaves_the_rerere_cache_as_native_git_does() {
    let fixture: Fixture = fixture("word-merge-rr");
    let repo: PathBuf = super::word_merge::prepared(&fixture, super::word_merge::Stop::Merge, &[super::word_merge::ADJACENT]);
    git(&fixture, repo.as_path(), &["config", "rerere.enabled", "true"]);
    let native: PathBuf = super::service_landing_support::copy_beside(&fixture, repo.as_path(), "native");
    // Real Git stops on the same input, and the person settles the file the way the pass does.
    attempt(&fixture, native.as_path(), &["merge", "--no-edit", "topic"]);
    let resolved: PathBuf = native.join(super::word_merge::tracked(super::word_merge::ADJACENT));
    std::fs::write(resolved.as_path(), super::word_merge::corpus(super::word_merge::ADJACENT, "word.out")).expect("the settled file");
    let merged: String = super::word_merge::tracked(super::word_merge::ADJACENT);
    git(&fixture, native.as_path(), &["add", "--", merged.as_str()]);
    let committed = {
        let mut command: Command = direct(&fixture);
        command.current_dir(native.as_path()).args(["commit", "--no-edit", "--cleanup=whitespace"]).env("GIT_EDITOR", "true");
        command.output().expect("native Git starts")
    };
    assert!(committed.status.success(), "{}", String::from_utf8_lossy(committed.stderr.as_slice()));
    let wrapped: super::support::Observed = super::support::run_wrapped(&fixture, repo.as_path(), &["merge", "--no-edit", "topic"]);
    assert_eq!(wrapped.code, Some(0), "{}", super::support::stderr_of(&wrapped));
    let expected: String = bytes_of(native.join(".git/MERGE_RR").as_path());
    let found: String = bytes_of(repo.join(".git/MERGE_RR").as_path());
    remove_settled_with(&fixture, &[native.as_path(), repo.as_path()]);
    assert_eq!(found, expected, "MERGE_RR after the wrapper finished the merge");
}

/// Remove a fixture and the copies made inside it.
fn remove_settled_with(fixture: &Fixture, copies: &[&Path]) {
    for copy in copies {
        let _ = std::fs::remove_dir_all(copy);
    }
    super::commit::remove_settled(fixture);
}

/// The rerere cache a merge the wrapper finishes itself leaves in a linked worktree:
/// no entry may name a conflict the merge no longer has,
/// which is the state the `/var/home/user/temp/agent/pi-runs` report found in
/// `.git/worktrees/<name>/MERGE_RR`.
#[test]
fn a_merge_finished_in_a_linked_worktree_leaves_no_conflict_entry_behind() {
    let fixture: Fixture = fixture("word-merge-rr-linked");
    let repo: PathBuf = super::word_merge::prepared(&fixture, super::word_merge::Stop::Merge, &[super::word_merge::ADJACENT]);
    git(&fixture, repo.as_path(), &["config", "rerere.enabled", "true"]);
    let linked: PathBuf = fixture.root.join("linked");
    git(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "merged", linked.to_str().expect("UTF-8 path"), "main"]);
    let wrapped: super::support::Observed = super::support::run_wrapped(&fixture, linked.as_path(), &["merge", "--no-edit", "topic"]);
    let said: String = super::support::stderr_of(&wrapped);
    let git_dir: String = {
        let answer = direct(&fixture).current_dir(linked.as_path()).args(["rev-parse", "--absolute-git-dir"]).output().expect("native Git starts");
        String::from_utf8_lossy(answer.stdout.as_slice()).trim().to_string()
    };
    let left: String = bytes_of(Path::new(git_dir.as_str()).join("MERGE_RR").as_path());
    let _ = std::fs::remove_dir_all(&linked);
    super::commit::remove_settled(&fixture);
    assert_eq!(wrapped.code, Some(0), "{said}");
    assert!(
        left == "(absent)" || left == "[]",
        "no conflict entry may survive the merge: {left} (in {git_dir}, {said})"
    );
}

/// Where rerere is off,
/// nothing writes `MERGE_RR` at all:
/// not the `git merge` that stopped,
/// and not the finish the wrapper runs after its second pass,
/// in a linked worktree as in the main one.
/// A cache left there would be the wrapper's own,
/// because native Git writes none:
/// measured with Git 2.56.0 on a conflicted merge:
/// `MERGE_RR` is absent with
/// `rerere.enabled` false and holds one entry per conflicted path with it true.
#[test]
fn a_merge_finished_in_a_linked_worktree_writes_no_rerere_cache_where_rerere_is_off() {
    let fixture: Fixture = fixture("word-merge-rr-off");
    let repo: PathBuf = super::word_merge::prepared(&fixture, super::word_merge::Stop::Merge, &[super::word_merge::ADJACENT]);
    let enabled: String = String::from_utf8_lossy(
        direct(&fixture)
            .current_dir(repo.as_path())
            .args(["config", "--get", "rerere.enabled"])
            .output()
            .expect("native Git starts")
            .stdout
            .as_slice(),
    )
    .trim()
    .to_string();
    assert_eq!(enabled, "", "the fixture must not enable rerere");
    let linked: PathBuf = fixture.root.join("linked");
    git(&fixture, repo.as_path(), &["worktree", "add", "--quiet", "-b", "merged", linked.to_str().expect("UTF-8 path"), "main"]);
    let wrapped: super::support::Observed = super::support::run_wrapped(&fixture, linked.as_path(), &["merge", "--no-edit", "topic"]);
    let said: String = super::support::stderr_of(&wrapped);
    let git_dir: String = {
        let answer = direct(&fixture).current_dir(linked.as_path()).args(["rev-parse", "--absolute-git-dir"]).output().expect("native Git starts");
        String::from_utf8_lossy(answer.stdout.as_slice()).trim().to_string()
    };
    let left: String = bytes_of(Path::new(git_dir.as_str()).join("MERGE_RR").as_path());
    let _ = std::fs::remove_dir_all(&linked);
    super::commit::remove_settled(&fixture);
    assert_eq!(wrapped.code, Some(0), "{said}");
    assert_eq!(left, "(absent)", "no rerere cache may be written where rerere is off (in {git_dir}, {said})");
}
