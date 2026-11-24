//! Controls for the fill's private index:
//!  the normalized bytes keep every entry's facts,
//!  the private name is written whole,
//!  the publication is one rename,
//!  and every state a kill leaves is told apart.

use super::{index_for_the_fill, pending_stands, publish_the_private_index, published_stands, the_publication_ran, write_the_private_index};
use crate::test_support::{fixture, git, remove};
use crate::worktree_clean::stored_stats;
use std::path::PathBuf;

/// A repository with two files and one commit.
fn a_repository(name: &str) -> PathBuf {
    let root: PathBuf = fixture(name);
    git(root.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    std::fs::write(root.join("a.txt"), b"a\n").expect("the file");
    std::fs::write(root.join("b.txt"), b"b\n").expect("the file");
    git(root.as_path(), &["add", "."]);
    git(root.as_path(), &["-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid", "commit", "--quiet", "-m", "one"]);
    return root;
}

#[test]
fn the_fill_index_keeps_every_entry_and_its_facts() {
    let root: PathBuf = a_repository("index-normalized");
    let index: Vec<u8> = std::fs::read(root.join(".git").join("index")).expect("the index");
    let normalized: Vec<u8> = index_for_the_fill(index.as_slice(), 20).expect("the index is normalized");
    let before: Vec<crate::worktree_clean_stat_verdicts::StatEntry> = stored_stats(index.as_slice(), 20).expect("the entries");
    let after: Vec<crate::worktree_clean_stat_verdicts::StatEntry> = stored_stats(normalized.as_slice(), 20).expect("the entries");
    assert_eq!(before.len(), after.len());
    for (one, other) in before.iter().zip(after.iter()) {
        assert_eq!(one.path, other.path);
        assert_eq!(one.mode, other.mode);
        assert_eq!(one.stored, other.stored, "no stat field is taken from any file");
    }
    let body: &[u8] = &normalized[..normalized.len() - 20];
    assert_eq!(&normalized[normalized.len() - 20..], crate::subvolume_sha::sha1(body).as_slice(), "the checksum is remade over the whole file");
    remove(root.as_path());
}

#[test]
fn a_written_private_index_takes_the_worktrees_name_in_one_rename() {
    let root: PathBuf = a_repository("index-publication");
    let registration: PathBuf = root.join(".git").join("worktrees").join("topic");
    std::fs::create_dir_all(registration.as_path()).expect("the registration");
    let admin: std::fs::File = std::fs::File::open(registration.as_path()).expect("the registration opens");
    let index: Vec<u8> = std::fs::read(root.join(".git").join("index")).expect("the index");
    let normalized: Vec<u8> = index_for_the_fill(index.as_slice(), 20).expect("the index is normalized");
    write_the_private_index(&admin, normalized.as_slice()).expect("the private write");
    assert!(pending_stands(registration.as_path()));
    assert!(!published_stands(registration.as_path()), "no index stands at the worktree's own name");
    assert!(!the_publication_ran(registration.as_path()));
    publish_the_private_index(&admin, registration.as_path()).expect("the publication");
    assert!(!pending_stands(registration.as_path()));
    assert!(published_stands(registration.as_path()));
    assert!(the_publication_ran(registration.as_path()));
    assert_eq!(std::fs::read(registration.join("index")).expect("the published index"), normalized);
    remove(root.as_path());
}

#[test]
fn a_second_private_write_and_a_publication_over_a_name_refuse() {
    let root: PathBuf = a_repository("index-refusals");
    let registration: PathBuf = root.join(".git").join("worktrees").join("topic");
    std::fs::create_dir_all(registration.as_path()).expect("the registration");
    let admin: std::fs::File = std::fs::File::open(registration.as_path()).expect("the registration opens");
    let index: Vec<u8> = std::fs::read(root.join(".git").join("index")).expect("the index");
    let normalized: Vec<u8> = index_for_the_fill(index.as_slice(), 20).expect("the index is normalized");
    write_the_private_index(&admin, normalized.as_slice()).expect("the first write");
    let second: String = write_the_private_index(&admin, normalized.as_slice()).expect_err("a second write replaces nothing");
    assert!(second.contains("cannot be made"), "{second}");
    std::fs::write(registration.join("index"), b"another command's index").expect("a name stands");
    let refused: String = publish_the_private_index(&admin, registration.as_path()).expect_err("the name is taken");
    assert!(refused.contains("cannot take the worktree's own name"), "{refused}");
    assert!(pending_stands(registration.as_path()), "the private name still stands");
    remove(root.as_path());
}

#[test]
fn an_index_that_is_no_index_refuses() {
    let refusal: String = index_for_the_fill(b"not an index", 20).expect_err("the bytes are no index");
    assert!(refusal.contains("cannot be written for the fill"), "{refusal}");
    assert!(refusal.contains("Ways forward:"), "{refusal}");
}
