//! Controls for the merge and rebuild steps on disposable repositories:
//!  the merge base, clean and conflicting merges, the conflicting commit, the raw rewrite, and the SSH re-sign.

use super::*;
use crate::replay_object::parse_raw_commit;
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};
use std::ffi::OsStr;
use std::path::PathBuf;

/// The empty tree of the SHA-1 object format.
const EMPTY_TREE: &str = "4b825dc642cb6eb9a060e54bf8d69288fbee4904";

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

/// A commit with the merge base at a commit is returned as is; an unborn base becomes a commit over the empty tree.
#[test]
fn the_merge_base_is_the_base_or_a_root_over_the_empty_tree() {
    let root: PathBuf = fixture("replay-merge-base");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let shadow: PathBuf = repo.join(".git");
    let context = git_context();
    assert_eq!(
        replay_merge_base(&context, shadow.as_path(), repo.as_path(), Some("abc123"), EMPTY_TREE).expect("base"),
        "abc123"
    );
    let unborn: String = replay_merge_base(&context, shadow.as_path(), repo.as_path(), None, EMPTY_TREE).expect("unborn");
    assert_eq!(git_text(repo.as_path(), &["cat-file", "-t", unborn.as_str()]).trim(), "commit");
    assert_eq!(rev(repo.as_path(), &format!("{unborn}^{{tree}}")), EMPTY_TREE);
    remove(root.as_path());
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

/// Overlapping hunks conflict: the conflicted paths are reported and the commit they lost to is the landed one.
#[test]
fn overlapping_hunks_conflict_and_name_the_winning_commit() {
    let (root, base, current, prepared) = scenario("replay-merge-conflict", "PREPARED", "line 11");
    let repo: PathBuf = root.join("repo");
    let merged: ReplayMerge = merge_replay_tree(&git_context(), repo.join(".git").as_path(), repo.as_path(), &base, &current, &prepared)
        .expect("merge");
    assert_eq!(merged, ReplayMerge::Conflict { paths: vec![b"c.txt".to_vec()] });
    let winner: String = first_commit_touching(&git_context(), repo.as_path(), Some(&base), &current, &[b"c.txt".to_vec()])
        .expect("winner");
    assert_eq!(winner, current);
    let nothing: String = first_commit_touching(&git_context(), repo.as_path(), Some(&current), &current, &[b"c.txt".to_vec()])
        .expect("nothing");
    assert_eq!(nothing, current);
    remove(root.as_path());
}

/// A conflict over a name that is not UTF-8 reports its exact bytes, and the winning commit is found by them.
#[test]
fn a_conflict_over_a_name_that_is_not_utf8_reports_its_exact_bytes() {
    use std::os::unix::ffi::OsStringExt;
    let file: std::ffi::OsString = std::ffi::OsString::from_vec(b"caf\xe9.txt".to_vec());
    let (root, base, current, prepared) = scenario_for_file("replay-merge-conflict-odd", file.as_os_str(), "PREPARED", "line 11");
    let repo: PathBuf = root.join("repo");
    let merged: ReplayMerge = merge_replay_tree(&git_context(), repo.join(".git").as_path(), repo.as_path(), &base, &current, &prepared)
        .expect("merge");
    assert_eq!(merged, ReplayMerge::Conflict { paths: vec![b"caf\xe9.txt".to_vec()] });
    let winner: String = first_commit_touching(&git_context(), repo.as_path(), Some(&base), &current, &[b"caf\xe9.txt".to_vec()])
        .expect("winner");
    assert_eq!(winner, current);
    // A path that no commit touched leaves the current target as the answer.
    let other: String = first_commit_touching(&git_context(), repo.as_path(), Some(&base), &current, &[b"caf\xe8.txt".to_vec()])
        .expect("other");
    assert_eq!(other, current);
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
    assert_eq!(
        merged,
        ReplayMerge::Conflict { paths: vec![b"a.txt".to_vec(), b"caf\xe9.txt".to_vec(), b"z.txt".to_vec()] }
    );
    remove(root.as_path());
}

/// A merge with an unknown merge base is a Git failure, not a conflict.
#[test]
fn an_unusable_merge_base_is_a_failure() {
    let (root, _base, current, prepared) = scenario("replay-merge-failure", "PREPARED", "line 11");
    let repo: PathBuf = root.join("repo");
    let failure = merge_replay_tree(
        &git_context(),
        repo.join(".git").as_path(),
        repo.as_path(),
        "0000000000000000000000000000000000000001",
        &current,
        &prepared,
    );
    assert!(matches!(failure, Err(ReplayMergeError::Git(_))), "{failure:?}");
    remove(root.as_path());
}

/// An unsigned prepared commit is rebuilt by rewriting its object: the tree and parent change, the rest of the bytes stay.
#[test]
fn an_unsigned_commit_is_rebuilt_by_rewriting_its_object() {
    let (root, base, current, prepared) = scenario("replay-merge-unsigned", "line 2", "OWN");
    let repo: PathBuf = root.join("repo");
    let shadow: PathBuf = repo.join(".git");
    let original: RawCommit = parse_raw_commit(&git_raw(repo.as_path(), &prepared)).expect("prepared");
    let tree: String = rev(repo.as_path(), &format!("{prepared}^{{tree}}"));
    let scratch: PathBuf = root.join("scratch");
    std::fs::create_dir(&scratch).expect("scratch");
    let replayed: ReplayedCommit = write_replayed_commit(
        &git_context(),
        shadow.as_path(),
        repo.as_path(),
        scratch.as_path(),
        1,
        &original,
        &tree,
        &current,
        None,
        &[],
    )
    .expect("rebuild");
    assert!(replayed.dropped_headers.is_empty());
    let raw: String = String::from_utf8_lossy(&git_raw(repo.as_path(), &replayed.oid)).into_owned();
    assert!(raw.starts_with(&format!("tree {tree}\nparent {current}\n")), "{raw}");
    assert!(raw.ends_with("\nprepared\n"));
    assert!(!raw.contains(&base));
    remove(root.as_path());
}

/// A signed prepared commit is re-signed under its own identities; a custom header it carries is reported as dropped.
#[test]
fn a_signed_commit_is_resigned_and_reports_dropped_headers() {
    let (root, _base, current, _prepared) = scenario("replay-merge-signed", "line 2", "OWN");
    let repo: PathBuf = root.join("repo");
    let shadow: PathBuf = repo.join(".git");
    let key: PathBuf = root.join("key");
    let generated = std::process::Command::new("ssh-keygen")
        .args(["-q", "-t", "ed25519", "-N", "", "-C", "replay", "-f"])
        .arg(&key)
        .status()
        .expect("ssh-keygen");
    assert!(generated.success());
    let public: String = std::fs::read_to_string(root.join("key.pub")).expect("public key");
    let tree: String = rev(repo.as_path(), &format!("{current}^{{tree}}"));
    let body: String = format!(
        "tree {tree}\nparent {current}\nauthor Ada <ada@example.test> 1700000000 +0100\ncommitter Grace <grace@example.test> 1700000100 -0500\ngpgsig -----BEGIN SSH SIGNATURE-----\n abc\n -----END SSH SIGNATURE-----\nx-custom value\n\nsigned prepared\n"
    );
    let object: PathBuf = root.join("prepared.commit");
    std::fs::write(&object, body).expect("object text");
    let object_path: String = object.to_string_lossy().into_owned();
    let prepared_oid: String = git_text(repo.as_path(), &["hash-object", "-t", "commit", "-w", object_path.as_str()])
        .trim()
        .to_string();
    let original: RawCommit = parse_raw_commit(&git_raw(repo.as_path(), &prepared_oid)).expect("prepared");
    assert!(original.signed);
    let scratch: PathBuf = root.join("scratch");
    std::fs::create_dir(&scratch).expect("scratch");
    let globals: Vec<String> = vec![
        String::from("-c"),
        String::from("gpg.format=ssh"),
        String::from("-c"),
        format!("user.signingkey={}", key.display()),
    ];
    let replayed: ReplayedCommit = write_replayed_commit(
        &git_context(),
        shadow.as_path(),
        repo.as_path(),
        scratch.as_path(),
        1,
        &original,
        &tree,
        &current,
        None,
        &globals,
    )
    .expect("re-sign");
    assert_eq!(replayed.dropped_headers, vec![String::from("x-custom")]);
    let raw: String = String::from_utf8_lossy(&git_raw(repo.as_path(), &replayed.oid)).into_owned();
    assert!(raw.contains("gpgsig -----BEGIN SSH SIGNATURE-----"), "{raw}");
    assert!(raw.contains("author Ada <ada@example.test> 1700000000 +0100"), "{raw}");
    assert!(raw.contains("committer Grace <grace@example.test> 1700000100 -0500"), "{raw}");
    assert!(!public.is_empty());
    remove(root.as_path());
}

/// The raw object of a commit, as `git cat-file commit` writes it.
fn git_raw(repo: &Path, oid: &str) -> Vec<u8> {
    return git(repo, &["cat-file", "commit", oid]).stdout;
}

/// Of several landed commits that touch the path, the oldest one after the base is the winner.
#[test]
fn the_oldest_landed_commit_that_touches_the_path_wins() {
    let (root, base, current, _prepared) = scenario("replay-merge-oldest", "PREPARED", "line 11");
    let repo: PathBuf = root.join("repo");
    std::fs::write(repo.join("c.txt"), lines("LANDED", "LATER")).expect("later");
    git(repo.as_path(), &["commit", "--quiet", "--all", "--message=later"]);
    let later: String = rev(repo.as_path(), "HEAD");
    assert_ne!(later, current);
    let winner: String = first_commit_touching(&git_context(), repo.as_path(), Some(&base), &later, &[b"c.txt".to_vec()])
        .expect("winner");
    assert_eq!(winner, current);
    remove(root.as_path());
}
