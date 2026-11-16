//! What:
//!  Controls of the one proof that a folder is a Git directory,
//!  judged against real Git over generated `HEAD` contents.
//! Why:
//!  The proof answers "is this a repository" without starting Git,
//!  and it may accept only folders Git itself would use:
//!  a `HEAD` whose text the proof accepted but Git rejected let the abandoned-lock pass unlink a file in a folder Git
//!  refuses as no repository
//!  (a review of `b0ea691cc1`,
//! finding 1).
//!  The one-directional comparison is what keeps the proof honest:
//!  accepting a spelling Git rejects is the defect,
//!  while refusing one Git accepts only costs a lock nothing clears.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const [name, bytes] of heads) { writeFileSync(join(gitDir, 'HEAD'), bytes); const gitRuns = statusOk(gitDir); const proof = proveGitDirectory(gitDir); expect(proof && !gitRuns, name).toBe(false) }
//! ```

/// The proof under test, and the answer about a folder it gives.
use crate::git_directory_proof::prove_git_directory;
/// Real Git in a fixture, with no system or global configuration and a fixed identity.
use crate::test_support::{fixture, git, git_output, remove};
/// Owned and borrowed paths of the fixtures.
use std::path::{Path, PathBuf};

/// What:
///  The `HEAD` contents the comparison walks:
///  the two spellings a repository really has,
///  and the spellings a reader could accept where Git refuses.
/// Why:
///   A corpus of only invalid spellings would pass by refusing everything,
///  and a corpus of only valid ones would never meet the defect,
///  so both ends are needed,
///  with the bytes below 0x20 and 0x7f beside them
///  (the review's suggestion for finding 1).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// heads(oid) // [['valid-symref', Buffer.from('ref: refs/heads/main\n')], ['symref-nul', Buffer.from('ref: \u0000\n')], …]
/// ```
fn head_contents(oid: &str) -> Vec<(&'static str, Vec<u8>)> {
    return vec![
        ("valid-symref", b"ref: refs/heads/main\n".to_vec()),
        ("valid-symref-no-newline", b"ref: refs/heads/main".to_vec()),
        ("valid-detached", format!("{oid}\n").into_bytes()),
        ("valid-detached-no-newline", oid.as_bytes().to_vec()),
        ("symref-nul", b"ref: \0\n".to_vec()),
        ("symref-nul-after-name", b"ref: refs/heads/main\0junk\n".to_vec()),
        ("symref-empty", b"ref: \n".to_vec()),
        ("symref-two-spaces", b"ref:  refs/heads/main\n".to_vec()),
        ("symref-carriage-return", b"ref: refs/heads/main\r\n".to_vec()),
        ("symref-second-line", b"ref: refs/heads/main\nsecond\n".to_vec()),
        ("symref-space-in-name", b"ref: refs/heads/main name\n".to_vec()),
        ("symref-control-byte", b"ref: refs/heads/mai\x01n\n".to_vec()),
        ("symref-delete-byte", b"ref: refs/heads/mai\x7fn\n".to_vec()),
        ("symref-tab", b"ref: refs/heads/main\t\n".to_vec()),
        ("symref-parent-component", b"ref: refs/heads/../main\n".to_vec()),
        ("symref-dotdot-in-name", b"ref: refs/heads/mai..n\n".to_vec()),
        ("symref-lock-suffix", b"ref: refs/heads/main.lock\n".to_vec()),
        ("symref-dot-component", b"ref: refs/heads/.hidden\n".to_vec()),
        ("symref-double-slash", b"ref: refs/heads//main\n".to_vec()),
        ("symref-trailing-slash", b"ref: refs/heads/main/\n".to_vec()),
        ("symref-one-level", b"ref: HEAD\n".to_vec()),
        ("symref-no-space", b"ref:refs/heads/main\n".to_vec()),
        ("symref-unborn", b"ref: refs/heads/unborn\n".to_vec()),
        ("symref-tags", b"ref: refs/tags/v1\n".to_vec()),
        ("symref-bare-refs", b"ref: refs/\n".to_vec()),
        ("symref-refs-nul", b"ref: refs/\0\n".to_vec()),
        ("empty", Vec::new()),
        ("nul-nul-nul", b"\0\0\0".to_vec()),
        ("word-head", b"HEAD\n".to_vec()),
        ("hex-39", format!("{}\n", &oid[1..]).into_bytes()),
        ("hex-41", format!("{oid}0\n").into_bytes()),
        ("hex-uppercase", format!("{}\n", oid.to_uppercase()).into_bytes()),
        ("hex-64", format!("{}\n", "a".repeat(64)).into_bytes()),
        ("hex-40-missing", format!("{}\n", "a".repeat(40)).into_bytes()),
        ("hex-40-uppercase-missing", format!("{}\n", "A".repeat(40)).into_bytes()),
        ("long-refname", format!("ref: refs/heads/{}\n", "a".repeat(300)).into_bytes()),
        ("long-garbage", "x".repeat(4096).into_bytes()),
    ];
}

/// What:
///  Whether real Git runs in the repository whose `HEAD` is `bytes`,
///  asked as the wrapper's own commands would be.
/// Why:
///   The question is not what one Git function thinks of the file,
///  but whether Git uses the folder at all:
///  a `HEAD` naming an object that is not there is no repository to Git,
///  and the status of the command is the answer a person gets.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const realGitRuns = (gitDir) => statusOk(gitDir);
/// ```
fn real_git_runs(git_dir: &Path, repository: &Path) -> bool {
    let status: std::process::ExitStatus = git_output(git_dir, &["--git-dir", git_dir.to_string_lossy().as_ref(), "--work-tree", repository.to_string_lossy().as_ref(), "status", "--short"]).status;
    return status.success();
}

/// What:
///  No `HEAD` the proof accepts is one real Git refuses,
///  and the two spellings a real repository carries are accepted.
/// Why:
///   The proof's whole purpose is to keep the lock pass out of folders Git refuses,
///  and this is the comparison that holds it to that over the bytes a person's file can carry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(heads.some(([name, bytes]) => proofAccepts(bytes) && !gitRuns(bytes))).toBe(false)
/// expect(proofAccepts('ref: refs/heads/main\n')).toBe(true)
/// ```
#[test]
fn no_head_the_proof_accepts_is_one_real_git_refuses() {
    let root: PathBuf = fixture("git-directory-proof-heads");
    let repository: PathBuf = root.join("repository");
    std::fs::create_dir_all(repository.as_path()).expect("the repository folder");
    git(repository.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    std::fs::write(repository.join("tracked.txt"), b"base\n").expect("a tracked file");
    git(repository.as_path(), &["add", "--", "tracked.txt"]);
    git(repository.as_path(), &["commit", "--quiet", "--message=base"]);
    let oid: String = String::from_utf8(git(repository.as_path(), &["rev-parse", "HEAD"]).stdout).expect("the commit is hexadecimal").trim().to_owned();
    let git_dir: PathBuf = repository.join(".git");
    let mut accepted: Vec<&str> = Vec::new();
    let mut refused: Vec<(&str, String)> = Vec::new();
    for (name, bytes) in head_contents(oid.as_str()) {
        std::fs::write(git_dir.join("HEAD"), bytes.as_slice()).expect("the HEAD under test");
        let git_uses_the_folder: bool = real_git_runs(git_dir.as_path(), repository.as_path());
        let proof_accepts: bool = prove_git_directory(git_dir.as_path(), git_dir.as_path()).is_some();
        assert!(
            !(proof_accepts && !git_uses_the_folder),
            "the proof accepted `{name}`, whose bytes real Git refuses as no repository"
        );
        if proof_accepts {
            accepted.push(name);
        } else if git_uses_the_folder {
            refused.push((name, String::from_utf8_lossy(bytes.as_slice()).into_owned()));
        }
    }
    // What: The comparison is not vacuous in either direction.
    // Why:  A proof that refused every folder, or one that accepted every folder, would pass the assertion above for a
    //       wrong reason, and the cases Git refuses are the ones the defect was made of.
    assert!(accepted.contains(&"valid-symref") && accepted.contains(&"valid-detached"), "the proof accepts the two spellings a real repository carries: {accepted:?}");
    assert!(accepted.len() >= 2, "the comparison holds cases both accept: {accepted:?}");
    assert!(!accepted.contains(&"symref-nul"), "the proof refuses a symref carrying a NUL, which real Git refuses");
    assert!(!accepted.contains(&"hex-40-missing"), "the proof refuses a detached HEAD whose object is not there, which real Git refuses");
    // A refusal costs a lock nothing clears, so it is allowed and named here rather than failed.
    assert!(refused.len() < head_contents(oid.as_str()).len(), "the comparison does not refuse every spelling: {} refused", refused.len());
    remove(root.as_path());
}
