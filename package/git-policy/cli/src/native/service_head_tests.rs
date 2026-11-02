//! Controls for reading a worktree's head: what a `HEAD` file can and cannot say.

use super::*;

/// A symbolic head names its branch; the text after the prefix, up to the line end, is the reference.
#[test]
fn a_symbolic_head_names_its_branch() {
    assert_eq!(interpret("ref: refs/heads/main\n"), HeadFile::Branch(String::from("refs/heads/main")));
    assert_eq!(interpret("ref: refs/heads/a b\r\n"), HeadFile::Branch(String::from("refs/heads/a b")));
}

/// An object name is a detached head.
#[test]
fn an_object_name_is_a_detached_head() {
    assert_eq!(interpret("0123456789abcdef0123456789abcdef01234567\n"), HeadFile::Detached);
}

/// The reftable placeholder and an empty file say nothing, so Git is asked.
#[test]
fn the_reftable_placeholder_and_an_empty_file_cannot_answer() {
    assert_eq!(interpret(format!("ref: {REFTABLE_PLACEHOLDER}\n").as_str()), HeadFile::Unreadable);
    assert_eq!(interpret(""), HeadFile::Unreadable);
}

/// A directory that is not a Git directory of the repository is not trusted, so a `HEAD` file there is never read.
#[test]
fn a_directory_of_another_repository_is_not_trusted() {
    let root = crate::test_support::fixture("head-foreign");
    let ours = root.join("ours");
    let theirs = root.join("theirs");
    std::fs::create_dir_all(&ours).expect("ours");
    std::fs::create_dir_all(&theirs).expect("theirs");
    assert!(belongs_to(ours.as_path(), ours.as_path()));
    assert!(!belongs_to(theirs.as_path(), ours.as_path()));
    std::fs::write(theirs.join("commondir"), "../ours\n").expect("pointer");
    assert!(belongs_to(theirs.as_path(), ours.as_path()), "a linked worktree points back to its common directory");
    crate::test_support::remove(root.as_path());
}
