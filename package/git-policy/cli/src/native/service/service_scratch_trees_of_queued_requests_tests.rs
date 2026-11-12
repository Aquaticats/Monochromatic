//! What:
//!  The record of the trees this service's requests hold:
//!  that a note is kept by the name of the tree,
//!  that a second send of one request replaces the tree of the attempt it left,
//!  that the tree outlasts every send still noted for that request,
//!  and that releasing the last send lets its tree go.
//! Why:
//!  The collection that spares a queued request's tree is only as good as this record,
//!  and a record that kept every attempt of a command would keep a tree nothing lands on.
//!  A request identity is not one send:
//!  the landing answers a request before the group it is in is over,
//!  and a client whose answer was lost sends the same request again,
//!  so two sends of one identity can be noted at once and the tree each of them reads must stay held
//!  until the service is done with that send.
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

/// The request identity the controls of one record's race use.
const A_SEND_AGAIN: &str = "a-request-sent-again";

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
/// and only that tree is held;
/// the send of the attempt it left stays noted until its own release arrives.
#[test]
fn a_second_send_of_one_request_holds_only_the_tree_it_names() {
    let sent_relatively: PathBuf = Path::new("monochromatic-git-policy-cli-service-7d8e9f0a-0").to_path_buf();
    note("a-request", sent_relatively.as_path());
    note("a-request", tree(1).as_path());
    assert!(is_held(tree(1).as_path()), "the tree the second send names is held");
    assert!(!is_held(sent_relatively.as_path()), "the tree of the attempt it left is not");
    release("a-request");
    assert!(is_held(tree(1).as_path()), "one release is one send's end, and the other send still reads this tree");
    release("a-request");
    assert!(!is_held(tree(1).as_path()), "the last send's release gives the tree back to the age rule");
}

/// What:
///  The tree of a send still noted after the group of an earlier send of the same request is over.
/// Why:
///   The landing answers a request before the group it is in is over
///  (`land_group` sends the answer,
///  `service_server.rs` then ends the group and releases its trees),
///  so a client told to prepare again sends the same identity with the next attempt's tree
///  inside that window.
///  A release that forgot the identity alone took the hold off the tree that attempt is still to read,
///  and a client killed while that attempt waited left the private index and the conclusion copies
///  to a collection of their age (`service_scratch_abandonment.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// note(id, left); note(id, retried); release(id); expect(isHeld(retried)).toBe(true);
/// ```
#[test]
fn a_release_after_the_next_send_keeps_the_tree_that_send_names() {
    let left: PathBuf = tree(3);
    let retried: PathBuf = tree(4);
    note(A_SEND_AGAIN, left.as_path());
    note(A_SEND_AGAIN, retried.as_path());
    release(A_SEND_AGAIN);
    assert!(is_held(retried.as_path()), "the attempt still queued keeps its tree out of a collection's reach");
    release(A_SEND_AGAIN);
    assert!(!is_held(retried.as_path()), "the release of that attempt gives its tree back to the age rule");
}

/// What:
///  The tree of a request sent again with the tree it already had, until both sends are over.
/// Why:
///   A client whose answer was lost sends the same request again with the identity and the tree it already had
///  (`submit_loop`,
///  `service_client.rs`),
///  and the group of the send it repeated can be over before the send it made afterwards is landed.
///  One hold for two noted sends let the tree go to a collection while a landing had still to read it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// note(id, sent); note(id, sent); release(id); expect(isHeld(sent)).toBe(true);
/// ```
#[test]
fn a_request_sent_twice_with_one_tree_holds_it_until_both_sends_are_over() {
    let sent: PathBuf = tree(5);
    note(A_SEND_AGAIN, sent.as_path());
    note(A_SEND_AGAIN, sent.as_path());
    release(A_SEND_AGAIN);
    assert!(is_held(sent.as_path()), "the send still queued keeps the tree its landing reads");
    release(A_SEND_AGAIN);
    assert!(!is_held(sent.as_path()), "the release of the last send gives the tree back");
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
