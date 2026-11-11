//! What:
//!  A pooled shadow member this invocation refuses,
//!  seen from the terminal a person reads and from the record a slow commit explains itself with.
//! Why:
//!  A member is the wrapper's own cache:
//! a person can neither cause a refusal nor fix one,
//!  so a line about it in the middle of their commit's output only alarms them
//!  (`shadow_pool.rs`,
//! `note_refusal`).
//! The evidence still has a home,
//!  which is the commit's own timing record beside the creation the refusal forced.
//! This is the whole path a person runs:
//! the built executable,
//! a repository,
//! and a commit that Git builds in a shadow.

use super::commit::{remove_settled, run_commit};
use super::support::{Fixture, Observed, fixture, git, repository, stderr_of};
use git_policy_cli::commit_timing::TIMING_FILE;
use git_policy_cli::wrapper_names_old_and_new::SHADOW_POOL_FOLDER;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// A repository whose commit real Git builds,
/// because a native-commit key in its configuration asks for it,
///  so the wrapper takes or makes a shadow.
fn repository_needing_a_shadow(fixture: &Fixture) -> PathBuf {
    let repo: PathBuf = repository(fixture, OsStr::new("repo"));
    git(fixture, repo.as_path(), &["config", "commit.verbose", "false"]);
    std::fs::write(repo.join("a.txt"), b"one\n").expect("a");
    git(fixture, repo.as_path(), &["add", "--", "a.txt"]);
    return repo;
}

/// The last timing record the repository wrote.
fn last_record(repo: &Path) -> serde_json::Value {
    let path: PathBuf = repo.join(".git").join(TIMING_FILE);
    let text: String = std::fs::read_to_string(path.as_path()).expect("the commit's timing record");
    let last: &str = text
        .lines()
        .filter(|line| { return !line.trim().is_empty(); })
        .next_back()
        .expect("one record");
    return serde_json::from_str(last).expect("a record of one commit");
}

/// What:
///  A refused member writes nothing on the terminal,
///  and the commit's record names the refusal and the creation it forced.
/// Why:
///   This is the line the person saw,
///  `[warn] ... [shadowPool] refused the pooled shadow member-...`,
///  and nothing they can act on was in it.
///  The route stays visible where a coordinator reads a slow commit:
///  the record already names the creation's stretches,
///  and `shadow pool refused` says why the build paid for one.
#[test]
fn a_refused_member_is_quiet_on_the_terminal_and_named_in_the_record() {
    let fixture: Fixture = fixture("pool-refusal-quiet");
    let repo: PathBuf = repository_needing_a_shadow(&fixture);
    let pool: PathBuf = repo.join(".git").join(SHADOW_POOL_FOLDER).join("main");
    // A member no claim can use: a creation's config is not there.
    std::fs::create_dir_all(pool.join("member-control").as_path()).expect("a member no claim can use");
    let landed: Observed = run_commit(&fixture, repo.as_path(), &["commit", "--quiet", "--message=one", "--", "a.txt"]);
    assert_eq!(landed.code, Some(0), "{}", stderr_of(&landed));
    let stderr: String = stderr_of(&landed);
    assert!(!stderr.contains("refused the pooled shadow"), "a refusal of the wrapper's own cache reached the terminal: {stderr}");
    assert!(!stderr.contains("shadowPool"), "no pool line belongs on the terminal: {stderr}");
    assert!(
        std::fs::symlink_metadata(pool.join("member-control").as_path()).is_err(),
        "the refused member is removed rather than trusted"
    );
    let record: serde_json::Value = last_record(repo.as_path());
    let parts: &serde_json::Map<String, serde_json::Value> = record["buildParts"]
        .as_object()
        .unwrap_or_else(|| { panic!("the record has no build parts: {record}") });
    assert!(parts.contains_key("shadow pool refused"), "the record names the refusal: {record}");
    assert!(parts.contains_key("shadow skeleton"), "and the creation the refusal forced: {record}");
    remove_settled(&fixture);
}
