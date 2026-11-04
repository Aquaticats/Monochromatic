use super::*;

fn entry(path: &[u8], stage: u8, id: &str) -> StartEntry {
    return StartEntry { path: path.to_vec(), stage, mode: 0o100_644, id: id.to_string(), extended: 0 };
}

/// Names that hold every separator and a byte that is not text round-trip.
#[test]
fn entries_round_trip_with_awkward_names() {
    let entries: Vec<StartEntry> = vec![entry(b"a\tb:c\nd", 0, "ab"), entry(&[0xff, 0xfe, b'/', b'x'], 2, "cd"), entry(b"", 3, "ef")];
    assert_eq!(decode(encode(entries.as_slice()).as_str()), Some(entries));
}

/// A damaged item is no record at all, so the caller never reads half a record as the whole.
#[test]
fn a_damaged_item_is_no_record() {
    assert_eq!(decode("zz:0:100644:ab:0"), None);
    assert_eq!(decode("61:0:100644:ab"), None);
    assert_eq!(decode("61:0:100644:ab:0:extra"), None);
    assert_eq!(decode("6:0:100644:ab:0"), None);
}

/// A path with an entry in neither list, or the same entry in both, is unchanged; any other entry is a change.
#[test]
fn a_path_is_unchanged_only_when_its_entries_are_the_recorded_ones() {
    let recorded: Vec<StartEntry> = vec![entry(b"same", 0, "aa"), entry(b"edited", 0, "bb"), entry(b"unmerged", 1, "cc"), entry(b"unmerged", 2, "dd")];
    let current: Vec<StartEntry> = vec![entry(b"same", 0, "aa"), entry(b"edited", 0, "ee"), entry(b"unmerged", 1, "cc"), entry(b"added", 0, "ff")];
    let names: Vec<Vec<u8>> = vec![b"same".to_vec(), b"edited".to_vec(), b"unmerged".to_vec(), b"added".to_vec(), b"absent".to_vec()];
    let (unchanged, changed) = split_by_start(recorded.as_slice(), current.as_slice(), names.as_slice());
    assert_eq!(unchanged, vec![b"same".to_vec(), b"absent".to_vec()]);
    assert_eq!(changed, vec![b"edited".to_vec(), b"unmerged".to_vec(), b"added".to_vec()]);
}
