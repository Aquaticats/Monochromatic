//! Controls for what an amend reads from the commit it replaces:
//! which stored commits it can carry over,
//! and the author
//!  line it makes.

use super::*;

const TREE: &str = "4b825dc642cb6eb9a060e54bf8d69288fbee4904";
const PARENT: &str = "0123456789abcdef0123456789abcdef01234567";
const OTHER: &str = "89abcdef0123456789abcdef0123456789abcdef";

fn stored(headers: &[&str], message: &str) -> String {
    return format!("{}\n\n{message}", headers.join("\n"));
}

/// A commit with a tree,
/// parents,
/// an author and a committer is read whole,
/// with the message kept as stored.
#[test]
fn a_plain_commit_is_read_with_its_parents_author_and_message() {
    let author: &str = "A U <a@example.invalid> 1700000000 +0200";
    let raw: String = stored(
        &[
            format!("tree {TREE}").as_str(),
            format!("parent {PARENT}").as_str(),
            format!("parent {OTHER}").as_str(),
            format!("author {author}").as_str(),
            "committer C <c@example.invalid> 1700000001 +0000",
        ],
        "subject\n\nbody  \n",
    );
    let read: StoredCommit = parse_stored_commit(raw.as_str()).expect("covered");
    assert_eq!(read.parents, vec![PARENT.to_string(), OTHER.to_string()]);
    assert_eq!(read.author, author);
    assert_eq!(read.message, "subject\n\nbody  \n");
}

/// A root commit has no parents.
#[test]
fn a_root_commit_has_no_parents() {
    let raw: String = stored(&[format!("tree {TREE}").as_str(), "author A <a@b> 1 +0000", "committer C <c@d> 2 +0000"], "root\n");
    assert_eq!(parse_stored_commit(raw.as_str()).map(|read| { return read.parents; }), Some(Vec::new()));
}

/// The signature and its continuation lines are dropped,
/// as native Git drops them;
/// the message after them is kept.
#[test]
fn a_signature_is_dropped_with_its_continuation_lines() {
    let raw: String = format!(
        "tree {TREE}\nauthor A <a@b> 1 +0000\ncommitter C <c@d> 2 +0000\ngpgsig -----BEGIN PGP SIGNATURE-----\n \n abc\n -----END PGP SIGNATURE-----\n\nsigned subject\n"
    );
    let read: StoredCommit = parse_stored_commit(raw.as_str()).expect("covered");
    assert_eq!(read.message, "signed subject\n");
}

/// A header the amend would have to carry,
/// an encoding or a merge tag,
/// leaves the line to native Git;
/// so does a damaged commit.
#[test]
fn a_header_that_would_have_to_be_carried_is_not_covered() {
    for extra in ["encoding ISO-8859-1", "mergetag object abc", "unknown header"] {
        let raw: String = stored(
            &[format!("tree {TREE}").as_str(), "author A <a@b> 1 +0000", "committer C <c@d> 2 +0000", extra],
            "subject\n",
        );
        assert_eq!(parse_stored_commit(raw.as_str()), None, "{extra}");
    }
    assert_eq!(parse_stored_commit("author A <a@b> 1 +0000\n\nno tree\n"), None, "no tree");
    assert_eq!(parse_stored_commit(format!("tree {TREE}\ncommitter C <c@d> 2 +0000\n\nno author\n").as_str()), None, "no author");
    assert_eq!(parse_stored_commit("no blank line"), None);
}

/// The author line splits into name,
/// address and raw date,
/// whatever the name holds.
#[test]
fn an_author_line_splits_into_name_address_and_date() {
    assert_eq!(
        split_author("Name With <angle> Words <a@example.invalid> 1700000000 -0130"),
        Some((String::from("Name With <angle> Words"), String::from("a@example.invalid"), String::from("1700000000 -0130")))
    );
    assert_eq!(split_author("no address 1 +0000"), None);
}
