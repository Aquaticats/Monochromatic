//! Controls for the raw difference reader and the worktree states.

use super::*;
use crate::test_support::{fixture, git, git_context, git_text, remove, repository};

/// Two NUL-separated entries read into their sides.
#[test]
fn raw_entries_are_read_with_exact_path_bytes() {
    let zero: String = "0".repeat(40);
    let mut output: Vec<u8> = Vec::new();
    output.extend_from_slice(format!(":100644 100755 {} {} M\0a b.txt\0", "1".repeat(40), "2".repeat(40)).as_bytes());
    output.extend_from_slice(format!(":000000 100644 {zero} {} A\0", "3".repeat(40)).as_bytes());
    output.extend_from_slice(b"dir/\xff.bin\0");
    output.extend_from_slice(format!(":120000 000000 {} {zero} D\0link\0", "4".repeat(40)).as_bytes());
    let changes: Vec<Change> = parse_raw_changes(&output).expect("well-formed output");
    assert_eq!(changes.len(), 3);
    assert_eq!(changes[0].path, b"a b.txt".to_vec());
    assert_eq!((changes[0].old_mode.as_str(), changes[0].new_mode.as_str()), ("100644", "100755"));
    assert!(changes[0].existed() && changes[0].exists_after());
    assert_eq!(changes[1].path, b"dir/\xff.bin".to_vec());
    assert!(!changes[1].existed() && changes[1].exists_after());
    assert_eq!(changes[1].new_oid, "3".repeat(40));
    assert!(changes[2].existed() && !changes[2].exists_after());
    assert_eq!(changes[2].old_oid, "4".repeat(40));
}

/// Output that is not raw differences is a failure.
#[test]
fn malformed_raw_output_is_refused() {
    assert_eq!(parse_raw_changes(b"").expect("empty output is no change"), Vec::<Change>::new());
    assert!(parse_raw_changes(b"path-without-header\0x\0").is_err());
    assert!(parse_raw_changes(b":100644 100644 aa bb\0path\0").is_err());
    assert!(parse_raw_changes(b":100644 100644 aa bb M\0").is_err());
}

/// The state of a path is the object Git would record for it.
#[test]
fn a_worktree_path_is_hashed_the_way_git_records_it() {
    let parent: PathBuf = fixture("merge-changes-states");
    let repo: PathBuf = repository(parent.as_path(), "repo");
    std::fs::write(repo.join("file.txt"), b"content\n").expect("file");
    std::fs::create_dir(repo.join("dir")).expect("directory");
    std::os::unix::fs::symlink("file.txt", repo.join("link")).expect("link");
    let context = git_context();
    let expected_file: String = git_text(repo.as_path(), &["hash-object", "file.txt"]);
    assert_eq!(
        worktree_state(&context, repo.as_path(), b"file.txt").expect("file state"),
        WorktreeState::Content(expected_file)
    );
    let target_blob: String = {
        let mut command = std::process::Command::new("/usr/bin/git");
        command.current_dir(repo.as_path()).args(["hash-object", "--stdin"]);
        command.stdin(std::process::Stdio::piped()).stdout(std::process::Stdio::piped());
        let mut child = command.spawn().expect("hash-object");
        std::io::Write::write_all(&mut child.stdin.take().expect("stdin"), b"file.txt").expect("input");
        String::from_utf8(child.wait_with_output().expect("output").stdout).expect("text").trim().to_owned()
    };
    assert_eq!(
        worktree_state(&context, repo.as_path(), b"link").expect("link state"),
        WorktreeState::Content(target_blob),
        "a link is hashed from its target text, never from the file it points at"
    );
    assert_eq!(worktree_state(&context, repo.as_path(), b"dir").expect("directory"), WorktreeState::Other);
    assert_eq!(worktree_state(&context, repo.as_path(), b"absent.txt").expect("absent"), WorktreeState::Missing);
    assert_eq!(
        worktree_state(&context, repo.as_path(), b"file.txt/below").expect("below a file"),
        WorktreeState::Missing
    );
    remove(parent.as_path());
}

/// A text-conversion rule applies to the hash, as it does when Git stages the file.
#[test]
fn a_conversion_rule_applies_to_the_hash() {
    let parent: PathBuf = fixture("merge-changes-conversion");
    let repo: PathBuf = repository(parent.as_path(), "repo");
    std::fs::write(repo.join(".gitattributes"), b"*.txt text eol=crlf\n").expect("attributes");
    std::fs::write(repo.join("crlf.txt"), b"one\r\ntwo\r\n").expect("file");
    git(repo.as_path(), &["add", "--", ".gitattributes", "crlf.txt"]);
    let staged: String = git_text(repo.as_path(), &["ls-files", "--format=%(objectname)", "--", "crlf.txt"]);
    assert_eq!(
        worktree_state(&git_context(), repo.as_path(), b"crlf.txt").expect("state"),
        WorktreeState::Content(staged)
    );
    remove(parent.as_path());
}
