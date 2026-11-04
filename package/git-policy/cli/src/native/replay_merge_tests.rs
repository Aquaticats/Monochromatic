//! Controls for the merge and rebuild steps on disposable repositories:
//!  the merge base, clean and conflicting merges, the conflicting commit, the raw rewrite, and the SSH re-sign.

use super::*;
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};
use std::ffi::OsStr;
use std::path::PathBuf;

/// Revision text of a commit, blob, or tree, trimmed.
fn rev(repo: &Path, spec: &str) -> String {
    return git_text(repo, &["rev-parse", spec]).trim().to_string();
}

/// A twelve-line file with one edit on line 2 (`landed`) or line 11 (`own`), or none.
fn lines(second: &str, eleventh: &str) -> String {
    return (1..=12)
        .map(|number| { return match number {
            2 => format!("{second}\n"),
            11 => format!("{eleventh}\n"),
            _ => format!("line {number}\n"),
        }; })
        .collect();
}

/// Base, landed (`main`), and prepared (`prepared`) commits: base is line text, landed edits line 2, prepared edits `prepared_second`.
fn scenario(name: &str, prepared_second: &str, prepared_eleventh: &str) -> (PathBuf, String, String, String) {
    return scenario_for_file(name, OsStr::new("c.txt"), prepared_second, prepared_eleventh);
}

/// The same three commits over the file `file`, whose name may be any bytes.
fn scenario_for_file(name: &str, file: &OsStr, prepared_second: &str, prepared_eleventh: &str) -> (PathBuf, String, String, String) {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join(file), lines("line 2", "line 11")).expect("base");
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    let base: String = rev(repo.as_path(), "HEAD");
    git(repo.as_path(), &["checkout", "--quiet", "-b", "prepared"]);
    std::fs::write(repo.join(file), lines(prepared_second, prepared_eleventh)).expect("prepared");
    git(repo.as_path(), &["commit", "--quiet", "--all", "--message=prepared"]);
    let prepared: String = rev(repo.as_path(), "HEAD");
    git(repo.as_path(), &["checkout", "--quiet", "main"]);
    std::fs::write(repo.join(file), lines("LANDED", "line 11")).expect("landed");
    git(repo.as_path(), &["commit", "--quiet", "--all", "--message=landed"]);
    let current: String = rev(repo.as_path(), "HEAD");
    return (root, base, current, prepared);
}

/// The conflicted paths of a merge that conflicted.
fn conflict_paths(merged: &ReplayMerge) -> Vec<Vec<u8>> {
    let ReplayMerge::Conflict { paths, .. } = merged else { panic!("expected a conflict") };
    return paths.clone();
}

/// Non-overlapping hunks in one file merge cleanly and the merged tree holds both edits.
#[test]
fn non_overlapping_hunks_merge_into_one_tree() {
    let (root, base, current, prepared) = scenario("replay-merge-clean", "line 2", "OWN");
    let repo: PathBuf = root.join("repo");
    let merged: ReplayMerge = merge_replay_tree(&git_context(), repo.join(".git").as_path(), repo.as_path(), &base, &current, &prepared)
        .expect("merge");
    let ReplayMerge::Merged { tree_oid } = merged else {
        panic!("expected a clean merge");
    };
    let spec: String = format!("{tree_oid}:c.txt");
    let text: String = git_text(repo.as_path(), &["cat-file", "-p", spec.as_str()]);
    assert!(text.contains("LANDED\n") && text.contains("OWN\n"), "{text}");
    remove(root.as_path());
}

/// Overlapping hunks conflict: the conflicted paths are reported.
#[test]
fn overlapping_hunks_conflict_and_name_their_paths() {
    let (root, base, current, prepared) = scenario("replay-merge-conflict", "PREPARED", "line 11");
    let repo: PathBuf = root.join("repo");
    let merged: ReplayMerge = merge_replay_tree(&git_context(), repo.join(".git").as_path(), repo.as_path(), &base, &current, &prepared)
        .expect("merge");
    assert_eq!(conflict_paths(&merged), vec![b"c.txt".to_vec()]);
    remove(root.as_path());
}

/// A conflict over a name that is not UTF-8 reports its exact bytes.
#[test]
fn a_conflict_over_a_name_that_is_not_utf8_reports_its_exact_bytes() {
    use std::os::unix::ffi::OsStringExt;
    let file: std::ffi::OsString = std::ffi::OsString::from_vec(b"caf\xe9.txt".to_vec());
    let (root, base, current, prepared) = scenario_for_file("replay-merge-conflict-odd", file.as_os_str(), "PREPARED", "line 11");
    let repo: PathBuf = root.join("repo");
    let merged: ReplayMerge = merge_replay_tree(&git_context(), repo.join(".git").as_path(), repo.as_path(), &base, &current, &prepared)
        .expect("merge");
    assert_eq!(conflict_paths(&merged), vec![b"caf\xe9.txt".to_vec()]);
    remove(root.as_path());
}

/// Conflicting names come back in byte order, each once.
#[test]
fn conflicting_names_are_sorted_by_their_bytes() {
    use std::os::unix::ffi::OsStringExt;
    let root: PathBuf = fixture("replay-merge-conflict-order");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let names: [&[u8]; 3] = [b"z.txt", b"caf\xe9.txt", b"a.txt"];
    for name in names {
        std::fs::write(repo.join(std::ffi::OsString::from_vec(name.to_vec())), lines("line 2", "line 11")).expect("base");
    }
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    let base: String = rev(repo.as_path(), "HEAD");
    git(repo.as_path(), &["checkout", "--quiet", "-b", "prepared"]);
    for name in names {
        std::fs::write(repo.join(std::ffi::OsString::from_vec(name.to_vec())), lines("PREPARED", "line 11")).expect("prepared");
    }
    git(repo.as_path(), &["commit", "--quiet", "--all", "--message=prepared"]);
    let prepared: String = rev(repo.as_path(), "HEAD");
    git(repo.as_path(), &["checkout", "--quiet", "main"]);
    for name in names {
        std::fs::write(repo.join(std::ffi::OsString::from_vec(name.to_vec())), lines("LANDED", "line 11")).expect("landed");
    }
    git(repo.as_path(), &["commit", "--quiet", "--all", "--message=landed"]);
    let current: String = rev(repo.as_path(), "HEAD");
    let merged: ReplayMerge = merge_replay_tree(&git_context(), repo.join(".git").as_path(), repo.as_path(), &base, &current, &prepared)
        .expect("merge");
    assert_eq!(conflict_paths(&merged), vec![b"a.txt".to_vec(), b"caf\xe9.txt".to_vec(), b"z.txt".to_vec()]);
    remove(root.as_path());
}
