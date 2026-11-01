//! Differential controls for the files ref reader:
//!  every answer it gives equals what `git symbolic-ref --quiet HEAD` says,
//!  and every spelling it does not recognise is left to Git.

use super::*;
use crate::test_support::{fixture, git, git_output, git_text, remove, repository};
use std::path::PathBuf;
use std::process::Output;

/// The places and formats of one fixture repository,
///  read from real Git.
pub(crate) struct Probe {
    /// Absolute Git directory.
    pub(crate) git_dir: PathBuf,
    /// Absolute common directory.
    pub(crate) common_dir: PathBuf,
    /// Object format name.
    pub(crate) object_format: String,
    /// Ref storage backend.
    pub(crate) ref_format: RefFormat,
}

/// Ask real Git for the layout of the repository or worktree at `directory`.
pub(crate) fn probe(directory: &Path) -> Probe {
    let git_dir: String = git_text(directory, &["rev-parse", "--absolute-git-dir"]);
    let common: String = git_text(directory, &["rev-parse", "--path-format=absolute", "--git-common-dir"]);
    let reftable: bool = git_text(directory, &["rev-parse", "--show-ref-format"]) == "reftable";
    return Probe {
        git_dir: PathBuf::from(git_dir),
        common_dir: PathBuf::from(common),
        object_format: git_text(directory, &["rev-parse", "--show-object-format"]),
        ref_format: if reftable { RefFormat::Reftable } else { RefFormat::Files },
    };
}

/// What real Git answers for `symbolic-ref --quiet HEAD`:
///  the branch, detached, or an error code.
pub(crate) fn oracle(directory: &Path) -> Result<SymbolicHead, Option<i32>> {
    let output: Output = git_output(directory, &["symbolic-ref", "--quiet", "HEAD"]);
    return match output.status.code() {
        Some(0) => Ok(SymbolicHead::Branch(
            String::from_utf8(output.stdout).expect("UTF-8").trim().to_string(),
        )),
        Some(1) => Ok(SymbolicHead::Detached),
        other => Err(other),
    };
}

/// Read the head with the reader,
///  and require that an answer equals Git's.
fn compared(directory: &Path, found: &Probe, label: &str) -> Option<SymbolicHead> {
    let layout: RefFilesLayout<'_> = RefFilesLayout {
        git_dir: found.git_dir.as_path(),
        common_dir: found.common_dir.as_path(),
        ref_format: found.ref_format,
        object_format: found.object_format.as_str(),
    };
    let mine: Option<SymbolicHead> = read_symbolic_head(&layout);
    if let Some(answer) = &mine {
        assert_eq!(&oracle(directory), &Ok(answer.clone()), "{label}");
    }
    return mine;
}

/// Replace the `HEAD` file of the repository `found` describes with `bytes`.
fn write_head(found: &Probe, bytes: &[u8]) {
    std::fs::write(found.git_dir.join("HEAD"), bytes).expect("fixture");
}

/// A born branch, a slashed branch, an unborn branch and a detached `HEAD` are answered,
///  each equal to Git's answer.
#[test]
fn plain_spellings_are_answered_as_git_answers() {
    let root: PathBuf = fixture("ref-files-plain");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let found: Probe = probe(repo.as_path());
    let head: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    assert_eq!(compared(repo.as_path(), &found, "born main"), Some(SymbolicHead::Branch(String::from("refs/heads/main"))));
    for name in ["feature/deep/name", "release-1.2_x", "a.b", "HEAD", "-dash"] {
        let full: String = format!("refs/heads/{name}");
        git(repo.as_path(), &["update-ref", full.as_str(), head.as_str()]);
        write_head(&found, format!("ref: {full}\n").as_bytes());
        assert_eq!(compared(repo.as_path(), &found, name), Some(SymbolicHead::Branch(full)));
    }
    write_head(&found, b"ref: refs/heads/not-born-yet\n");
    assert_eq!(
        compared(repo.as_path(), &found, "unborn"),
        Some(SymbolicHead::Branch(String::from("refs/heads/not-born-yet")))
    );
    write_head(&found, format!("{head}\n").as_bytes());
    assert_eq!(compared(repo.as_path(), &found, "detached"), Some(SymbolicHead::Detached));
    remove(root.as_path());
}

/// A branch that lives only in `packed-refs` is answered like a loose one.
#[test]
fn a_packed_only_branch_is_answered() {
    let root: PathBuf = fixture("ref-files-packed");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let found: Probe = probe(repo.as_path());
    git(repo.as_path(), &["pack-refs", "--all", "--prune"]);
    assert!(!probe(repo.as_path()).common_dir.join("refs/heads/main").exists());
    assert_eq!(compared(repo.as_path(), &found, "packed"), Some(SymbolicHead::Branch(String::from("refs/heads/main"))));
    remove(root.as_path());
}

/// Every spelling outside the two plain ones is left to Git,
///  and where Git answers anyway the table records that it was not guessed.
#[test]
fn other_spellings_are_left_to_git() {
    let root: PathBuf = fixture("ref-files-other");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let found: Probe = probe(repo.as_path());
    let head: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let upper: String = head.to_uppercase();
    let cases: Vec<(&str, Vec<u8>)> = vec![
        ("tag target", b"ref: refs/tags/t\n".to_vec()),
        ("remote target", b"ref: refs/remotes/o/x\n".to_vec()),
        ("no newline", b"ref: refs/heads/main".to_vec()),
        ("carriage return", b"ref: refs/heads/main\r\n".to_vec()),
        ("trailing space", b"ref: refs/heads/main \n".to_vec()),
        ("two newlines", b"ref: refs/heads/main\n\n".to_vec()),
        ("no space", b"ref:refs/heads/main\n".to_vec()),
        ("two spaces", b"ref:  refs/heads/main\n".to_vec()),
        ("lock suffix", b"ref: refs/heads/a.lock\n".to_vec()),
        ("dot component", b"ref: refs/heads/.hidden\n".to_vec()),
        ("double dot", b"ref: refs/heads/a..b\n".to_vec()),
        ("trailing slash", b"ref: refs/heads/a/\n".to_vec()),
        ("empty component", b"ref: refs/heads/a//b\n".to_vec()),
        ("trailing dot", b"ref: refs/heads/a.\n".to_vec()),
        ("space in name", b"ref: refs/heads/a b\n".to_vec()),
        ("bare prefix", b"ref: refs/heads/\n".to_vec()),
        ("not a ref", b"ref: main\n".to_vec()),
        ("upper hex", format!("{upper}\n").into_bytes()),
        ("short hex", format!("{}\n", &head[..39]).into_bytes()),
        ("long hex", format!("{head}0\n").into_bytes()),
        ("sha256 length in sha1", format!("{head}{}\n", &head[..24]).into_bytes()),
        ("null name", format!("{}\n", "0".repeat(40)).into_bytes()),
        ("detached no newline", head.clone().into_bytes()),
        ("detached trailing space", format!("{head} \n").into_bytes()),
        ("empty", Vec::new()),
        ("garbage", vec![0xff, 0xfe, 0x00, b'\n']),
    ];
    for (label, bytes) in cases {
        write_head(&found, bytes.as_slice());
        assert_eq!(compared(repo.as_path(), &found, label), None, "{label} must be left to Git");
    }
    remove(root.as_path());
}

/// A branch that is itself a symbolic ref makes `symbolic-ref` print the end of the chain,
///  so the reader leaves it to Git.
#[test]
fn a_symbolic_branch_is_left_to_git() {
    let root: PathBuf = fixture("ref-files-chain");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let found: Probe = probe(repo.as_path());
    git(repo.as_path(), &["symbolic-ref", "refs/heads/alias", "refs/heads/main"]);
    write_head(&found, b"ref: refs/heads/alias\n");
    assert_eq!(
        oracle(repo.as_path()),
        Ok(SymbolicHead::Branch(String::from("refs/heads/main"))),
        "the oracle follows the chain"
    );
    assert_eq!(compared(repo.as_path(), &found, "alias"), None);
    remove(root.as_path());
}

/// A loose branch file that is garbage, a directory or a link is left to Git.
#[test]
fn an_unusual_loose_branch_is_left_to_git() {
    let root: PathBuf = fixture("ref-files-loose");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let found: Probe = probe(repo.as_path());
    let common: PathBuf = found.common_dir.clone();
    std::fs::write(common.join("refs/heads/garbage"), "not-an-object\n").expect("fixture");
    write_head(&found, b"ref: refs/heads/garbage\n");
    assert_eq!(compared(repo.as_path(), &found, "garbage branch"), None);
    git(repo.as_path(), &["update-ref", "refs/heads/dir/inner", "main"]);
    write_head(&found, b"ref: refs/heads/dir\n");
    assert_eq!(compared(repo.as_path(), &found, "directory branch"), None);
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink("main", common.join("refs/heads/link")).expect("fixture");
        write_head(&found, b"ref: refs/heads/link\n");
        assert_eq!(compared(repo.as_path(), &found, "linked branch"), None);
    }
    remove(root.as_path());
}

/// A `HEAD` that is a link or a directory is left to Git.
#[test]
#[cfg(unix)]
fn a_linked_head_is_left_to_git() {
    let root: PathBuf = fixture("ref-files-link");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let found: Probe = probe(repo.as_path());
    let git_dir: PathBuf = found.git_dir.clone();
    std::fs::write(root.join("elsewhere"), "ref: refs/heads/main\n").expect("fixture");
    std::fs::remove_file(git_dir.join("HEAD")).expect("fixture");
    std::os::unix::fs::symlink(root.join("elsewhere"), git_dir.join("HEAD")).expect("fixture");
    assert_eq!(compared(repo.as_path(), &found, "linked HEAD"), None);
    remove(root.as_path());
}

/// A linked worktree reads its own `HEAD`,
///  not the main worktree's,
///  and the branch from the shared common directory.
#[test]
fn a_linked_worktree_reads_its_own_head() {
    let root: PathBuf = fixture("ref-files-linked");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let found: Probe = probe(repo.as_path());
    let linked: PathBuf = root.join("linked");
    git(repo.as_path(), &["worktree", "add", "--quiet", "-b", "feature", linked.to_str().expect("fixture")]);
    let linked_found: Probe = probe(linked.as_path());
    assert_eq!(compared(linked.as_path(), &linked_found, "linked"), Some(SymbolicHead::Branch(String::from("refs/heads/feature"))));
    assert_eq!(compared(repo.as_path(), &found, "main"), Some(SymbolicHead::Branch(String::from("refs/heads/main"))));
    git(linked.as_path(), &["checkout", "--quiet", "--detach"]);
    assert_eq!(compared(linked.as_path(), &linked_found, "linked detached"), Some(SymbolicHead::Detached));
    assert_eq!(compared(repo.as_path(), &found, "main still"), Some(SymbolicHead::Branch(String::from("refs/heads/main"))));
    remove(root.as_path());
}

/// A SHA-256 repository is answered with 64-digit names and refuses 40-digit ones.
#[test]
fn a_sha256_repository_is_answered_with_its_own_name_length() {
    let root: PathBuf = fixture("ref-files-sha256");
    let repo: PathBuf = root.join("repo");
    std::fs::create_dir(&repo).expect("fixture");
    git(repo.as_path(), &["init", "--quiet", "--initial-branch=main", "--object-format=sha256"]);
    git(repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=initial"]);
    let head: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let found: Probe = probe(repo.as_path());
    assert_eq!(head.len(), 64);
    assert_eq!(compared(repo.as_path(), &found, "sha256 branch"), Some(SymbolicHead::Branch(String::from("refs/heads/main"))));
    write_head(&found, format!("{head}\n").as_bytes());
    assert_eq!(compared(repo.as_path(), &found, "sha256 detached"), Some(SymbolicHead::Detached));
    write_head(&found, format!("{}\n", &head[..40]).as_bytes());
    assert_eq!(compared(repo.as_path(), &found, "sha1 length in sha256"), None);
    remove(root.as_path());
}

/// A reftable repository keeps a placeholder `HEAD` file,
///  so the reader never answers for it and Git names the real branch.
#[test]
fn a_reftable_repository_is_left_to_git() {
    let root: PathBuf = fixture("ref-files-reftable");
    let repo: PathBuf = root.join("repo");
    std::fs::create_dir(&repo).expect("fixture");
    git(repo.as_path(), &["init", "--quiet", "--initial-branch=trunk", "--ref-format=reftable"]);
    git(repo.as_path(), &["commit", "--quiet", "--allow-empty", "--message=initial"]);
    let found: Probe = probe(repo.as_path());
    assert_eq!(found.ref_format, RefFormat::Reftable);
    let placeholder: String = std::fs::read_to_string(found.git_dir.join("HEAD")).expect("fixture");
    assert_eq!(placeholder, "ref: refs/heads/.invalid\n");
    assert_eq!(compared(repo.as_path(), &found, "reftable"), None);
    assert_eq!(oracle(repo.as_path()), Ok(SymbolicHead::Branch(String::from("refs/heads/trunk"))));
    remove(root.as_path());
}

/// The reftable placeholder is refused by its own name,
///  and the backend check refuses on its own too:
///  a layout that says reftable is never answered from the `HEAD` file, whatever the file holds.
#[test]
fn the_backend_check_refuses_without_the_placeholder() {
    let root: PathBuf = fixture("ref-files-backend");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let found: Probe = probe(repo.as_path());
    let layout: RefFilesLayout<'_> = RefFilesLayout {
        git_dir: found.git_dir.as_path(),
        common_dir: found.common_dir.as_path(),
        ref_format: RefFormat::Reftable,
        object_format: found.object_format.as_str(),
    };
    assert_eq!(read_symbolic_head(&layout), None);
    remove(root.as_path());
}

/// An object format the reader does not know is left to Git.
#[test]
fn an_unknown_object_format_is_left_to_git() {
    let root: PathBuf = fixture("ref-files-format");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let found: Probe = probe(repo.as_path());
    let layout: RefFilesLayout<'_> = RefFilesLayout {
        git_dir: found.git_dir.as_path(),
        common_dir: found.common_dir.as_path(),
        ref_format: RefFormat::Files,
        object_format: "sha3",
    };
    assert_eq!(read_symbolic_head(&layout), None);
    remove(root.as_path());
}

/// Each call reads the file again:
///  a `HEAD` rewritten between two calls changes the second answer,
///  which is what the landing's compare-and-swap protection relies on.
#[test]
fn every_call_reads_the_file_again() {
    let root: PathBuf = fixture("ref-files-fresh");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let found: Probe = probe(repo.as_path());
    let head: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    git(repo.as_path(), &["update-ref", "refs/heads/other", head.as_str()]);
    assert_eq!(compared(repo.as_path(), &found, "first"), Some(SymbolicHead::Branch(String::from("refs/heads/main"))));
    write_head(&found, b"ref: refs/heads/other\n");
    assert_eq!(compared(repo.as_path(), &found, "second"), Some(SymbolicHead::Branch(String::from("refs/heads/other"))));
    write_head(&found, format!("{head}\n").as_bytes());
    assert_eq!(compared(repo.as_path(), &found, "third"), Some(SymbolicHead::Detached));
    remove(root.as_path());
}

/// The plain-name rule accepts and refuses the shapes Git's ref-name rules decide.
#[test]
fn the_plain_name_rule_matches_git_check_ref_format() {
    let root: PathBuf = fixture("ref-files-names");
    let accepted: [&str; 6] = ["refs/heads/a", "refs/heads/a/b", "refs/heads/a.b", "refs/heads/A_b-c", "refs/heads/-a", "refs/heads/a.lockx"];
    let refused: [&str; 12] = [
        "refs/heads/", "refs/heads/a/", "refs/heads/a//b", "refs/heads/.a", "refs/heads/a/.b", "refs/heads/a..b",
        "refs/heads/a.", "refs/heads/a.lock", "refs/heads/a/b.lock", "refs/tags/a", "heads/a", "refs/heads/a b",
    ];
    for name in accepted {
        assert!(is_plain_branch_name(name), "{name}");
        let output: Output = git_output(root.as_path(), &["check-ref-format", name]);
        assert!(output.status.success(), "Git must accept {name}");
    }
    for name in refused {
        assert!(!is_plain_branch_name(name), "{name}");
    }
    remove(root.as_path());
}
