//! Controls for the tree and commit parsers; the comparisons with real Git are in `repo_read_query_tests.rs`.

use super::*;

/// A tree entry in the stored layout: mode, space, name, NUL, raw name.
fn stored(mode: &str, name: &str, id: u8) -> Vec<u8> {
    let mut bytes: Vec<u8> = format!("{mode} {name}").into_bytes();
    bytes.push(0);
    bytes.extend_from_slice(&[id; 20]);
    return bytes;
}

/// The subject of a message that holds a NUL byte, which Git's string handling would cut.
fn parse_subject_with_nul() -> Option<Vec<u8>> {
    return subject_of(b"left\0right\n");
}

/// Entries come back in stored order with their modes, and malformed ones are refused.
#[test]
fn trees_parse_and_malformed_ones_are_refused() {
    let mut bytes = stored("100644", "a", 1);
    bytes.extend(stored("40000", "dir", 2));
    let entries = parse_tree(bytes.as_slice(), 20).expect("plain tree");
    assert_eq!(entries.len(), 2);
    assert!(is_tree_mode(entries[1].mode));
    assert!(!is_tree_mode(entries[0].mode));
    assert!(parse_tree(&bytes[..bytes.len() - 1], 20).is_none(), "a short name");
    assert!(parse_tree(stored("0100644", "a", 1).as_slice(), 20).is_none(), "a leading zero");
    assert!(parse_tree(stored("100644", "a/b", 1).as_slice(), 20).is_none(), "a slash in a name");
    assert!(parse_tree(stored("10x644", "a", 1).as_slice(), 20).is_none(), "a non-octal mode");
    assert!(parse_tree(stored("100664", "a", 1).as_slice(), 20).is_none(), "a mode Git canonicalizes to 100644");
    assert!(parse_subject_with_nul().is_none(), "a message with a NUL");
}

/// The tree, parents and message of a commit are read; an `encoding` header is declined.
#[test]
fn commits_parse_and_encodings_are_declined() {
    let tree = "1".repeat(40);
    let parent = "2".repeat(40);
    let text = format!("tree {tree}\nparent {parent}\nauthor A <a@e> 1 +0000\ncommitter A <a@e> 1 +0000\n\nsubject\n\nbody\n");
    let fields = parse_commit(text.as_bytes(), ObjectFormat::Sha1).expect("commit");
    assert_eq!(fields.parents.len(), 1);
    assert_eq!(fields.message, b"subject\n\nbody\n".to_vec());
    let encoded = format!("tree {tree}\nauthor A <a@e> 1 +0000\ncommitter A <a@e> 1 +0000\nencoding latin1\n\nsubject\n");
    assert!(parse_commit(encoded.as_bytes(), ObjectFormat::Sha1).is_none());
    assert!(parse_commit(b"no header", ObjectFormat::Sha1).is_none());
}

/// The subject is the first paragraph on one line; a carriage return is declined.
#[test]
fn subjects_join_the_first_paragraph() {
    assert_eq!(subject_of(b"one\ntwo\n\nbody\n"), Some(b"one two".to_vec()));
    assert_eq!(subject_of(b"\n\nlead\n"), Some(b"lead".to_vec()));
    assert_eq!(subject_of(b""), Some(Vec::new()));
    assert!(subject_of(b"a\r\nb\n").is_none());
}

/// A second listing of the same tree, through a database opened afresh as each in-process question does, decodes no pack
/// entry: one commit asks a dozen questions about the same base tree, and each used to read all of it again.
#[test]
fn a_tree_listed_before_is_not_read_again() {
    use crate::repo_read_object::from_hex;
    use crate::test_support::{fixture, git, git_output, remove};
    let root = fixture("repo-read-tree-shared");
    let repo = root.join("repo");
    std::fs::create_dir(&repo).expect("dir");
    git(repo.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    for directory in 0..30 {
        std::fs::create_dir_all(repo.join(format!("d{directory}/sub"))).expect("dirs");
        std::fs::write(repo.join(format!("d{directory}/sub/f.txt")), format!("{directory}\n")).expect("file");
    }
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=first"]);
    git(repo.as_path(), &["repack", "-a", "-d", "-f", "--quiet"]);
    git(repo.as_path(), &["prune-packed"]);
    let named = String::from_utf8(git_output(repo.as_path(), &["rev-parse", "HEAD^{tree}"]).stdout).expect("UTF-8");
    let tree = from_hex(named.trim().as_bytes(), ObjectFormat::Sha1).expect("name");
    let first_database = ObjectDatabase::open(repo.join(".git").as_path(), None, ObjectFormat::Sha1).expect("database");
    let first = flatten_tree_shared(&first_database, tree.as_slice(), ObjectFormat::Sha1).expect("listing");
    assert!(first_database.decoded_entries() > 0, "the first listing reads the pack");
    let second_database = ObjectDatabase::open(repo.join(".git").as_path(), None, ObjectFormat::Sha1).expect("database");
    let second = flatten_tree_shared(&second_database, tree.as_slice(), ObjectFormat::Sha1).expect("listing");
    assert_eq!(second_database.decoded_entries(), 0, "the second listing decodes nothing");
    assert_eq!(first, second);
    remove(root.as_path());
}
