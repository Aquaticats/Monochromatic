//! The durable intent: its text, its refusals, and its life on disk.

use super::*;
use crate::test_support::{fixture, remove};
use crate::worktree_copy_request::new_request_id;

fn sample(id: &str) -> Intent {
    return Intent {
        id: id.to_string(),
        source_root: PathBuf::from("/repository"),
        selection: CopySelection::Ignored,
        before_admin_ids: ["alpha", "beta"].into_iter().map(String::from).collect(),
        added_path: Some(PathBuf::from("../topic")),
        git_returned: false,
    };
}

/// An intent survives its own text with either selection, with and without a named path.
#[test]
fn an_intent_round_trips_through_its_text() {
    let named: Intent = sample("0001-a");
    let unnamed: Intent = Intent {
        added_path: None,
        git_returned: true,
        selection: CopySelection::Paths(vec![String::from(".cache/rules.txt")]),
        before_admin_ids: BTreeSet::new(),
        ..sample("0002-b")
    };
    for intent in [named, unnamed] {
        let text: String = intent.encode();
        assert_eq!(text.lines().count(), 1);
        assert_eq!(Intent::decode(text.as_str()).expect("round trip"), intent);
    }
}

/// Every field is checked on read.
#[test]
fn an_intent_with_an_unsafe_field_is_refused() {
    let good: String = sample("0001-a").encode();
    for (from, to) in [
        ("\"id\":\"0001-a\"", "\"id\":\"../escape\""),
        ("\"sourceRoot\":\"/repository\"", "\"sourceRoot\":\"relative\""),
        ("\"version\":1", "\"version\":2"),
        ("\"gitReturned\":false", "\"gitReturned\":\"no\""),
        ("\"beforeAdminIds\":[\"alpha\",\"beta\"]", "\"beforeAdminIds\":[1]"),
        ("\"addedPath\":\"../topic\"", "\"addedPath\":5"),
        ("{\"kind\":\"ignored\"}", "{\"kind\":\"paths\",\"paths\":[\"../outside\"]}"),
    ] {
        assert!(good.contains(from), "{from}");
        assert!(Intent::decode(good.replace(from, to).as_str()).is_err(), "{to} was accepted");
    }
    assert!(Intent::decode("not json").is_err());
}

/// An intent is written whole, listed in the order it was written, and removed; a missing one is removed already.
#[test]
fn an_intent_is_written_listed_and_removed() {
    let root: PathBuf = fixture("intent-life");
    assert!(!intents_exist(root.as_path()).expect("none"));
    let first: Intent = sample(new_request_id().expect("identity").as_str());
    std::thread::sleep(std::time::Duration::from_millis(3));
    let second: Intent = sample(new_request_id().expect("identity").as_str());
    write_intent(root.as_path(), &second).expect("written");
    write_intent(root.as_path(), &first).expect("written");
    assert!(intents_exist(root.as_path()).expect("some"));
    let (read, unreadable) = read_intents(root.as_path()).expect("read");
    assert!(unreadable.is_empty());
    assert_eq!(read, vec![first.clone(), second.clone()]);
    remove_intent(root.as_path(), &first).expect("removed");
    remove_intent(root.as_path(), &first).expect("removed twice");
    assert_eq!(read_intents(root.as_path()).expect("read").0, vec![second.clone()]);
    remove_intent(root.as_path(), &second).expect("removed");
    assert!(!intents_exist(root.as_path()).expect("none"));
    remove(root.as_path());
}

/// A file that does not parse is reported and does not hide the others.
#[test]
fn an_unreadable_intent_is_reported_and_the_others_are_read() {
    let root: PathBuf = fixture("intent-unreadable");
    let good: Intent = sample("0001-a");
    write_intent(root.as_path(), &good).expect("written");
    std::fs::write(intents_directory(root.as_path()).join("0000-broken.json"), b"{").expect("broken");
    let (read, unreadable) = read_intents(root.as_path()).expect("read");
    assert_eq!(read, vec![good]);
    assert_eq!(unreadable.len(), 1, "{unreadable:?}");
    assert!(unreadable[0].starts_with("0000-broken.json"));
    remove(root.as_path());
}

/// The time an intent was written is the one its identity starts with.
#[test]
fn an_intent_knows_when_it_was_written() {
    assert_eq!(sample("00000000000001ff-abc").written_at(), Some(0x1ff));
    assert_eq!(sample("not-hex").written_at(), None);
}
