//! Controls for the capture a merge's record holds:
//! its JSON shape,
//! the bytes of a path that is not text,
//! the slots that keep its objects,
//! and the damage that is refused.

use super::*;

/// One capture of two paths,
/// one of them with both layers and one with only a staged entry.
fn capture() -> Capture {
    return Capture {
        commit: String::from("c".repeat(40)),
        reference: Some(String::from("refs/cli-git/carried/7-42-0")),
        paths: vec![
            CapturedPath {
                path: b"dir/a b.txt".to_vec(),
                worktree: Some(CapturedFile { mode: String::from("100755"), raw: String::from("d".repeat(40)), blob: String::from("e".repeat(40)) }),
                index: Some(CapturedIndex { mode: String::from("100644"), oid: String::from("f".repeat(40)) }),
            },
            CapturedPath {
                path: b"not text \xff".to_vec(),
                worktree: None,
                index: Some(CapturedIndex { mode: String::from("100644"), oid: String::from("a".repeat(40)) }),
            },
        ],
    };
}

/// A capture reads back exactly,
/// with the bytes of a path that is not text.
#[test]
fn a_capture_round_trips() {
    let original: Capture = capture();
    let text: String = encode(&original);
    let value: serde_json::Value = serde_json::from_str(text.as_str()).expect("the capture is JSON");
    assert_eq!(decode(&value), Some(original));
}

/// What:
///  Every object a capture names is a slot of its tree,
/// once each.
/// Why:
///   A slot is what `git mktree` writes,
/// and an object named twice needs no second entry;
/// a slot missing would let
///  `git gc` collect bytes the record still promises.
#[test]
fn every_object_a_capture_names_is_one_slot() {
    let mut value: Capture = capture();
    // The same bytes in both layers is one object, and one slot.
    value.paths[0].index = Some(CapturedIndex { mode: String::from("100755"), oid: "d".repeat(40) });
    let found: Vec<Slot> = slots(value.paths.as_slice());
    let oids: Vec<&str> = found.iter().map(|slot| { return slot.oid.as_str(); }).collect();
    assert_eq!(oids, ["d".repeat(40).as_str(), "e".repeat(40).as_str(), "a".repeat(40).as_str()]);
    assert_eq!(found[0].name, "s0");
    assert_eq!(found[2].name, "s2");
}

/// What:
///  A capture whose bytes are not the shape this file writes is refused.
/// Why:
///   The bytes it names are the person's only copy once the checkout changed,
/// so a guess would lose them.
#[test]
fn a_damaged_capture_is_refused() {
    let text: String = encode(&capture());
    for broken in [
        String::from("{}"),
        String::from("[]"),
        text.replace("\"commit\"", "\"other\""),
        text.replace("\"raw\"", "\"other\""),
        text.replace("6469722f6120622e747874", "zz"),
        text.replace("\"paths\":[", "\"paths\":[{\"path\":\"a\",\"pathHex\":\"61\"},"),
    ] {
        let value: Option<serde_json::Value> = serde_json::from_str(broken.as_str()).ok();
        assert_eq!(value.as_ref().and_then(decode), None, "{broken}");
    }
}
