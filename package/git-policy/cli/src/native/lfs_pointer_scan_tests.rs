//! Controls for finding the pointers a push introduces, against repositories built by real Git.

use super::*;
use crate::repo_read_object::from_hex;
use crate::test_support::{fixture, git, git_output, remove};
use std::path::{Path, PathBuf};

/// The object name that the `n`-th fixture pointer names: 64 hexadecimal digits.
fn oid(n: u8) -> String {
    return format!("{n:02x}").repeat(32);
}

/// The text of a pointer file for the `n`-th fixture object.
fn pointer(n: u8) -> String {
    return format!("version https://git-lfs.github.com/spec/v1\noid sha256:{}\nsize {}\n", oid(n), 100 + u32::from(n));
}

/// Write `files` (path, text) into the repository and commit them; the commit's name.
fn commit(repo: &Path, files: &[(&str, &str)], message: &str) -> String {
    for (path, text) in files {
        let target: PathBuf = repo.join(path);
        std::fs::create_dir_all(target.parent().expect("a parent")).expect("directories");
        std::fs::write(target, text).expect("file");
    }
    git(repo, &["add", "--all"]);
    git(repo, &["commit", "--quiet", "--allow-empty", &format!("--message={message}")]);
    return name_of(repo, "HEAD");
}

/// The object name `revision` stands for.
fn name_of(repo: &Path, revision: &str) -> String {
    return String::from_utf8(git_output(repo, &["rev-parse", revision]).stdout).expect("UTF-8").trim().to_owned();
}

/// A repository with the commits `base` (a pointer 1 and plain text), `added` (pointer 2 beside it), `removed` (pointer 2
///  gone again, a plain edit) and a plain `tip`; their names in that order.
fn history(root: &Path) -> (PathBuf, [String; 4]) {
    let repo: PathBuf = root.join("repo");
    std::fs::create_dir(&repo).expect("dir");
    git(repo.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    let base: String = commit(repo.as_path(), &[("docs/readme.txt", "one\n"), ("assets/one.bin", pointer(1).as_str())], "base");
    let added: String = commit(repo.as_path(), &[("assets/two.bin", pointer(2).as_str())], "added");
    std::fs::remove_file(repo.join("assets/two.bin")).expect("remove");
    let removed: String = commit(repo.as_path(), &[("docs/readme.txt", "two\n")], "removed");
    let tip: String = commit(repo.as_path(), &[("docs/other.txt", "three\n")], "tip");
    return (repo, [base, added, removed, tip]);
}

/// The scan from `include` tips and `stop` tips, in the repository's own format.
fn scan(repo: &Path, include: &[&str], stop: &[&str]) -> Option<Vec<String>> {
    let format = ObjectFormat::Sha1;
    let database = ObjectDatabase::open(repo.join(".git").as_path(), None, format).expect("plain database");
    let names = |tips: &[&str]| -> Vec<Vec<u8>> {
        return tips.iter().map(|tip| { return from_hex(tip.as_bytes(), format).expect("name"); }).collect();
    };
    return new_pointer_objects(&database, format, names(include).as_slice(), names(stop).as_slice(), SCAN_BUDGET);
}

/// Commits that touch no pointer introduce none, whatever pointers the history behind them holds.
#[test]
fn commits_without_a_pointer_introduce_none() {
    let root = fixture("lfs-scan-none");
    let (repo, [base, _added, removed, tip]) = history(root.as_path());
    assert_eq!(scan(repo.as_path(), &[tip.as_str()], &[removed.as_str()]), Some(Vec::new()));
    assert_eq!(scan(repo.as_path(), &[tip.as_str()], &[tip.as_str()]), Some(Vec::new()), "nothing is ahead of itself");
    assert_eq!(scan(repo.as_path(), &[removed.as_str()], &[base.as_str()]).map(|found| { return found.len(); }), Some(1), "the control: the range holding the intermediate commit finds one");
    remove(root.as_path());
}

/// A push that adds one pointer names exactly that object, and not the ones the destination already has.
#[test]
fn a_new_pointer_is_named_and_a_known_one_is_not() {
    let root = fixture("lfs-scan-one");
    let (repo, [base, added, _removed, _tip]) = history(root.as_path());
    assert_eq!(scan(repo.as_path(), &[added.as_str()], &[base.as_str()]), Some(vec![oid(2)]));
    remove(root.as_path());
}

/// A pointer that one commit adds and the next removes is still uploaded: the destination will receive both commits.
#[test]
fn a_pointer_added_and_removed_inside_the_range_is_still_named() {
    let root = fixture("lfs-scan-transient");
    let (repo, [base, _added, _removed, tip]) = history(root.as_path());
    assert_eq!(scan(repo.as_path(), &[tip.as_str()], &[base.as_str()]), Some(vec![oid(2)]));
    remove(root.as_path());
}

/// With nothing known to the destination the whole history is read and every pointer in it is named, once.
#[test]
fn without_known_tips_the_whole_history_is_scanned() {
    let root = fixture("lfs-scan-all");
    let (repo, [_base, _added, _removed, tip]) = history(root.as_path());
    assert_eq!(scan(repo.as_path(), &[tip.as_str()], &[]), Some(vec![oid(1), oid(2)]));
    remove(root.as_path());
}

/// A merge of an old side branch names only the side branch's own pointer: the history behind the known tip is known.
#[test]
fn a_merge_names_what_neither_parent_holds_and_each_object_once() {
    let root = fixture("lfs-scan-merge");
    let (repo, [base, _added, _removed, tip]) = history(root.as_path());
    git(repo.as_path(), &["checkout", "--quiet", "-b", "side", base.as_str()]);
    let side: String = commit(repo.as_path(), &[("assets/three.bin", pointer(3).as_str())], "side");
    git(repo.as_path(), &["checkout", "--quiet", "main"]);
    git(repo.as_path(), &["merge", "--quiet", "--no-ff", "--message=merge", "side"]);
    let merge: String = name_of(repo.as_path(), "HEAD");
    assert_eq!(scan(repo.as_path(), &[merge.as_str()], &[tip.as_str()]), Some(vec![oid(3)]), "pointer 1 sits behind the known tip and is not dragged in");
    assert_eq!(scan(repo.as_path(), &[merge.as_str()], &[tip.as_str(), side.as_str()]), Some(Vec::new()), "both parents known: the merge adds nothing");
    remove(root.as_path());
}

/// Run Git in `repo` with the author and committer dates fixed to `date` (a raw Unix time and zone).
fn dated(repo: &Path, arguments: &[&str], date: &str) {
    let output = std::process::Command::new(crate::test_support::REAL_GIT)
        .current_dir(repo)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/nonexistent-global-config")
        .env("GIT_AUTHOR_NAME", "Fixture")
        .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
        .env("GIT_COMMITTER_NAME", "Fixture")
        .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid")
        .env("GIT_AUTHOR_DATE", date)
        .env("GIT_COMMITTER_DATE", date)
        .args(arguments)
        .output()
        .expect("git starts");
    assert!(output.status.success(), "git {arguments:?}: {}", String::from_utf8_lossy(output.stderr.as_slice()));
}

/// A side branch cut from an old commit and merged after a long known chain: the history behind the known tip is not
///  read as new. The committer times order the walk so that the known chain reaches the shared ancestor before the walk
///  decides it is done; the chain is longer than the few extra pops the walk allows for clock skew.
#[test]
fn an_old_side_branch_merged_late_drags_no_known_history() {
    let root = fixture("lfs-scan-order");
    let repo: PathBuf = root.join("repo");
    std::fs::create_dir(&repo).expect("dir");
    git(repo.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    std::fs::create_dir_all(repo.join("assets")).expect("assets");
    std::fs::write(repo.join("assets/one.bin"), pointer(1)).expect("pointer");
    git(repo.as_path(), &["add", "--all"]);
    dated(repo.as_path(), &["commit", "--quiet", "--message=old"], "1000000000 +0000");
    dated(repo.as_path(), &["checkout", "--quiet", "-b", "side"], "1000000000 +0000");
    std::fs::write(repo.join("side.txt"), "side\n").expect("side");
    git(repo.as_path(), &["add", "--all"]);
    dated(repo.as_path(), &["commit", "--quiet", "--message=side"], "1000001000 +0000");
    dated(repo.as_path(), &["checkout", "--quiet", "main"], "1000000000 +0000");
    for step in 0..8 {
        dated(repo.as_path(), &["commit", "--quiet", "--allow-empty", &format!("--message=chain {step}")], &format!("{} +0000", 1_000_002_000 + step));
    }
    let known: String = name_of(repo.as_path(), "HEAD");
    dated(repo.as_path(), &["merge", "--quiet", "--no-ff", "--no-edit", "side"], "1000003000 +0000");
    let merge: String = name_of(repo.as_path(), "HEAD");
    assert_eq!(scan(repo.as_path(), &[merge.as_str()], &[known.as_str()]), Some(Vec::new()), "the old pointer is behind the known tip");
    assert_eq!(scan(repo.as_path(), &[merge.as_str()], &[]), Some(vec![oid(1)]), "the control: with nothing known it is found");
    remove(root.as_path());
}

/// The walk goes newest first and stops soon after the new commits are done, so the history far behind the known tip is
///  never read: a commit deep in that history is deleted from the store (as a shallow clone lacks it), and the scan of
///  what lies above the tip still answers. A walk that read oldest first would reach the hole and decline.
#[test]
fn history_far_behind_the_known_tip_is_never_read() {
    let root = fixture("lfs-scan-deep");
    let repo: PathBuf = root.join("repo");
    std::fs::create_dir(&repo).expect("dir");
    git(repo.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    std::fs::create_dir_all(repo.join("assets")).expect("assets");
    std::fs::write(repo.join("assets/one.bin"), pointer(1)).expect("pointer");
    git(repo.as_path(), &["add", "--all"]);
    let mut names: Vec<String> = Vec::new();
    for step in 0..30_u32 {
        let date: String = format!("{} +0000", 1_000_000_000 + step * 10);
        dated(repo.as_path(), &["commit", "--quiet", "--allow-empty", &format!("--message=known {step}")], &date);
        names.push(name_of(repo.as_path(), "HEAD"));
    }
    let hole: &String = &names[3];
    std::fs::remove_file(repo.join(".git/objects").join(&hole[..2]).join(&hole[2..])).expect("the commit is stored loose");
    let known: &String = names.last().expect("a tip");
    std::fs::write(repo.join("docs.txt"), "new\n").expect("file");
    git(repo.as_path(), &["add", "--all"]);
    dated(repo.as_path(), &["commit", "--quiet", "--message=new"], "1000001000 +0000");
    let top: String = name_of(repo.as_path(), "HEAD");
    assert_eq!(scan(repo.as_path(), &[top.as_str()], &[known.as_str()]), Some(Vec::new()));
    remove(root.as_path());
}

/// Blobs that are not pointers (large, or lookalikes) are never named, whatever their path.
#[test]
fn lookalikes_and_large_files_are_not_named() {
    let root = fixture("lfs-scan-lookalike");
    let (repo, [_base, _added, _removed, tip]) = history(root.as_path());
    let lookalike: String = pointer(9).replace("size", "length");
    let large: String = format!("{}{}", pointer(8), "x".repeat(2000));
    let top: String = commit(repo.as_path(), &[("notes/mention.txt", "see git-lfs docs\n"), ("lookalike.bin", lookalike.as_str()), ("large.bin", large.as_str())], "lookalikes");
    assert_eq!(scan(repo.as_path(), &[top.as_str()], &[tip.as_str()]), Some(Vec::new()));
    remove(root.as_path());
}

/// Whatever the readers cannot answer declines instead of guessing: an unknown tip, a stop that is not a commit's parent
///  chain in the store, a spent budget and a pointer with an extension line.
#[test]
fn what_the_readers_cannot_answer_declines() {
    let root = fixture("lfs-scan-decline");
    let (repo, [_base, _added, _removed, tip]) = history(root.as_path());
    let missing: String = "ab".repeat(20);
    assert_eq!(scan(repo.as_path(), &[missing.as_str()], &[]), None, "an unknown tip");
    let format = ObjectFormat::Sha1;
    let database = ObjectDatabase::open(repo.join(".git").as_path(), None, format).expect("database");
    let tip_name: Vec<Vec<u8>> = vec![from_hex(tip.as_bytes(), format).expect("name")];
    assert_eq!(new_pointer_objects(&database, format, tip_name.as_slice(), &[], Duration::ZERO), None, "a spent budget");
    let extended: String = format!("version https://git-lfs.github.com/spec/v1\next-0-foo sha256:{}\noid sha256:{}\nsize 9\n", oid(4), oid(5));
    let top: String = commit(repo.as_path(), &[("ext.bin", extended.as_str())], "extension");
    assert_eq!(scan(repo.as_path(), &[top.as_str()], &[tip.as_str()]), None, "an extension line");
    remove(root.as_path());
}
