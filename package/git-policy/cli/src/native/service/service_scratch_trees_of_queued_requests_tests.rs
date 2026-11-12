//! What:
//!  The record of the trees this service's requests hold:
//!  that a note is kept by the name of the tree,
//!  that a second send of one request replaces the tree of the attempt it left,
//!  and that releasing a request lets its tree go.
//! Why:
//!  The collection that spares a queued request's tree is only as good as this record,
//!  and a record that kept every attempt of a command would keep a tree nothing lands on.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // note('a', tree); expect(isHeld(tree)).toBe(true); release('a'); expect(isHeld(tree)).toBe(false);
//! ```

/// The record under control.
use super::{is_held, note, release};
use std::path::{Path, PathBuf};

/// A tree of the wrapper's own name,
/// as a collection sees it inside the repository scratch directory.
fn tree(attempt: u32) -> PathBuf {
    return Path::new("/nowhere/.git/monochromatic-git-policy-cli-scratch")
        .join(format!("monochromatic-git-policy-cli-service-0a1b2c3d-{attempt}"));
}

/// A tree a request named is held under the name the collection sees it by,
/// and is no longer held once the service is done with that request.
#[test]
fn a_request_holds_its_tree_until_the_service_is_done_with_it() {
    let first: PathBuf = tree(0);
    note("a-request", first.as_path());
    assert!(is_held(first.as_path()), "the tree the request named is held");
    assert!(!is_held(tree(1).as_path()), "another attempt's tree is not");
    release("a-request");
    assert!(!is_held(first.as_path()), "the tree goes back to the age rule with the request");
}

/// A command that prepared again sends the same identity with a new attempt's tree,
/// and only that tree is held afterwards.
#[test]
fn a_second_send_of_one_request_holds_only_the_tree_it_names() {
    let sent_relatively: PathBuf = Path::new("monochromatic-git-policy-cli-service-7d8e9f0a-0").to_path_buf();
    note("a-request", sent_relatively.as_path());
    note("a-request", tree(1).as_path());
    assert!(is_held(tree(1).as_path()), "the tree the second send names is held");
    assert!(!is_held(sent_relatively.as_path()), "the tree of the attempt it left is not");
    release("a-request");
}

/// Releasing one request says nothing about the trees another request holds.
#[test]
fn releasing_one_request_keeps_the_trees_another_request_holds() {
    let mine: PathBuf = tree(2);
    let theirs: PathBuf = Path::new("/elsewhere/.git/monochromatic-git-policy-cli-scratch")
        .join("monochromatic-git-policy-cli-service-1a2b3c4d-0");
    note("mine", mine.as_path());
    note("theirs", theirs.as_path());
    release("mine");
    assert!(!is_held(mine.as_path()), "the released request holds nothing");
    assert!(is_held(theirs.as_path()), "the other request still lands on its tree");
    release("theirs");
}

/// A path that names no file is no tree of a scratch place,
/// so recording it keeps nothing and asking about it spares nothing.
#[test]
fn a_path_with_no_file_name_is_no_tree_to_hold() {
    note("root", Path::new("/"));
    assert!(!is_held(Path::new("/")), "a directory that names no entry is not a tree");
    release("root");
}
