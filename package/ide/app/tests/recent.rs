//! Consumer tests for the explicitly required Ctrl+digit navigation contract.

/// Import production history and key decoding,
///  plus owned test paths.
use ide_app::recent::{RecentFiles, shortcut_slot};
/// PathBuf owns test names independently of temporary formatting strings.
use std::path::{Path, PathBuf};

/// Verify ten unique slots,
///  eviction,
///  and current-file stability.
#[test]
fn history_caps_at_ten_and_promotes_duplicates() {
    // What: Default creates empty history; mut permits in-memory updates.
    // Why: Every test starts without persisted session data.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const recent = new RecentFiles();
    // ```
    let mut recent = RecentFiles::default();
    // Integer range is exclusive at twelve, matching a counted TS loop.
    for index in 0..12 {
        // What: format! creates an owned string and PathBuf owns its path value.
        // Why: Distinct names exercise actual eviction rather than repeated input.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // recent.opened(`file${index}.ts`);
        // ```
        recent.opened(PathBuf::from(format!("file{index}.ts")));
    }
    assert_eq!(recent.paths().len(), 10);
    // Some denotes an existing slot; Path::new borrows the expected name.
    assert_eq!(recent.at(0), Some(Path::new("file11.ts")));
    assert_eq!(recent.at(9), Some(Path::new("file2.ts")));
    recent.opened(PathBuf::from("file5.ts"));
    assert_eq!(recent.at(0), Some(Path::new("file5.ts")));
    assert_eq!(recent.at(1), Some(Path::new("file11.ts")));
    recent.opened(PathBuf::from("file5.ts"));
    assert_eq!(recent.at(1), Some(Path::new("file11.ts")));
}

/// Verify repeated Ctrl+1 alternates the last two opened files,
///  as editord does.
#[test]
fn previous_slot_toggles_after_promotion() {
    let mut recent = RecentFiles::default();
    recent.opened(PathBuf::from("a.ts"));
    recent.opened(PathBuf::from("b.ts"));
    // What: expect fails this test if its known-filled slot is absent.
    // Why: The target must be read before opened changes the recency ordering.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const previous = nonNullishOrThrow(recent.at(1));
    // ```
    let previous = recent.at(1).expect("filled previous slot").to_path_buf();
    recent.opened(previous);
    assert_eq!(recent.at(0), Some(Path::new("a.ts")));
    assert_eq!(recent.at(1), Some(Path::new("b.ts")));
}

/// Verify unfilled and out-of-range slots do nothing.
#[test]
fn empty_slots_are_absent() {
    let recent = RecentFiles::default();
    assert!(recent.at(0).is_none());
    assert!(recent.at(9).is_none());
    assert!(recent.at(10).is_none());
}

/// Verify every digit and every rejected modifier combination.
#[test]
fn control_digits_decode_without_accepting_other_shortcuts() {
    for index in 0..10 {
        let key = index.to_string();
        assert_eq!(shortcut_slot(&key, true, false, false), Some(index));
        assert_eq!(shortcut_slot(&key, false, false, false), None);
        assert_eq!(shortcut_slot(&key, true, true, false), None);
        assert_eq!(shortcut_slot(&key, true, false, true), None);
    }
    for key in ["", "10", "a", "猫", "９"] {
        assert_eq!(shortcut_slot(key, true, false, false), None);
    }
}
