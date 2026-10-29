// What:     Unit tests for `dialog_backend.rs`, pulled in by `#[cfg(test)] #[path = "dialog_backend_tests.rs"] mod tests;`.
// Why:      PB5: the program search is exercised with disposable folders, and the message must tell the user how to go on.

use super::*;

/// What: `scratch` makes an empty disposable folder.
/// Why: the search must never depend on the real PATH of the machine running the tests.
fn scratch(tag: &str) -> std::path::PathBuf {
    let nanos: u128 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| return elapsed.as_nanos());
    let dir = crate::test_scratch::scratch_root().join(format!("music-player-dialog-{}-{nanos}-{tag}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("the scratch folder is created");
    return dir;
}

#[test]
fn a_program_in_a_listed_folder_is_found() {
    let first = scratch("first");
    let second = scratch("second");
    std::fs::write(second.join("zenity"), b"#!/bin/sh\n").expect("the stub is written");
    let joined = std::env::join_paths([&first, &second]).expect("the folders join");
    assert!(executable_on_path("zenity", &joined));
    std::fs::remove_dir_all(&first).ok();
    std::fs::remove_dir_all(&second).ok();
}

#[test]
fn a_program_that_is_in_no_listed_folder_is_not_found() {
    let only = scratch("only");
    let joined = std::env::join_paths([&only]).expect("the folder joins");
    assert!(!executable_on_path("zenity", &joined));
    assert!(!executable_on_path("zenity", OsStr::new("")));
    std::fs::remove_dir_all(&only).ok();
}

#[test]
fn a_folder_named_like_the_program_is_not_the_program() {
    let only = scratch("folder");
    std::fs::create_dir_all(only.join("zenity")).expect("the folder is created");
    let joined = std::env::join_paths([&only]).expect("the folder joins");
    assert!(!executable_on_path("zenity", &joined));
    std::fs::remove_dir_all(&only).ok();
}

#[test]
fn the_message_names_the_cause_and_both_ways_forward_in_plain_words() {
    assert!(NO_DIALOG_MESSAGE.contains("folder chooser"));
    assert!(NO_DIALOG_MESSAGE.contains("Install zenity"));
    assert!(NO_DIALOG_MESSAGE.contains("start the player with a folder"));
    assert!(!NO_DIALOG_MESSAGE.contains("portal"), "no jargon");
    assert!(!NO_DIALOG_MESSAGE.contains('—') && !NO_DIALOG_MESSAGE.contains('–'));
}

#[test]
fn a_chooser_is_missing_only_when_neither_zenity_nor_the_portal_is_there() {
    assert!(no_chooser_can_open(false, false));
    assert!(!no_chooser_can_open(true, false));
    assert!(!no_chooser_can_open(false, true));
    assert!(!no_chooser_can_open(true, true));
}
