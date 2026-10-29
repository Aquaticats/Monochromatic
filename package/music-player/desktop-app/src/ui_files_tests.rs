//! Tests for the pure parts of `ui_files`: the error wording and the Undo capability flag.

use super::*;

#[test]
fn no_parent_error_names_the_path() {
    let message = FileActionError::NoParent(PathBuf::from("/")).to_string();
    assert!(message.contains('/'));
}

#[test]
fn spawn_error_names_the_program_detail() {
    let message = FileActionError::Spawn("xdg-open: not found".to_string()).to_string();
    assert!(message.contains("xdg-open: not found"));
}

#[test]
fn restore_capability_matches_the_trash_crate_platforms() {
    let expected = cfg!(any(
        windows,
        all(unix, not(any(target_os = "macos", target_os = "ios", target_os = "android")))
    ));
    assert_eq!(can_restore_from_trash(), expected);
}

/// What: `scratch_dir` makes an empty disposable folder under the system temp directory.
/// Why: the symlink tests need real folders, and must never touch the user's files or trash (THR).
fn scratch_dir(tag: &str) -> PathBuf {
    let nanos: u128 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| return elapsed.as_nanos());
    let dir = crate::test_scratch::scratch_root().join(format!("music-player-files-{}-{nanos}-{tag}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("the scratch folder is created");
    return dir;
}

#[cfg(unix)]
#[test]
fn a_trash_item_recorded_under_the_real_folder_matches_a_path_through_a_symlink() {
    let base = scratch_dir("symlink");
    let real = base.join("real");
    std::fs::create_dir_all(&real).expect("the real folder is created");
    let link = base.join("link");
    std::os::unix::fs::symlink(&real, &link).expect("the symlink is created");
    // The trashed file no longer exists: only its folder does.
    let recorded = real.join("01 Another Xronixle.wav");
    let opened_through_link = link.join("01 Another Xronixle.wav");
    assert!(same_original_path(&recorded, &opened_through_link), "the raw comparison alone failed here (PB3)");
    assert!(same_original_path(&opened_through_link, &recorded));
    std::fs::remove_dir_all(&base).ok();
}

#[test]
fn a_different_file_name_or_folder_does_not_match() {
    let base = scratch_dir("different");
    let other = base.join("other");
    std::fs::create_dir_all(&other).expect("the other folder is created");
    assert!(!same_original_path(&base.join("a.wav"), &base.join("b.wav")));
    assert!(!same_original_path(&base.join("a.wav"), &other.join("a.wav")));
    std::fs::remove_dir_all(&base).ok();
}

#[test]
fn identical_paths_match_even_when_their_folder_is_gone() {
    let gone = PathBuf::from("/nonexistent-music-player-folder/a.wav");
    assert!(same_original_path(&gone, &gone));
    assert!(!same_original_path(&gone, Path::new("/nonexistent-music-player-folder/b.wav")));
}

#[test]
fn restore_failures_are_plain_sentences_without_a_path() {
    let message = FileActionError::Restore("01.wav is not in the trash any more, so it could not be put back.".to_string()).to_string();
    assert_eq!(message, "01.wav is not in the trash any more, so it could not be put back.");
    assert!(!message.contains('/'));
    assert!(!message.contains("The file could not be restored"));
}

#[test]
fn a_path_moved_to_the_trash_by_this_player_counts_as_just_trashed() {
    let path = PathBuf::from("/music-player-test/just-trashed-once.wav");
    assert!(!was_just_trashed(&path));
    remember_trashed(&path);
    assert!(was_just_trashed(&path));
    assert!(!was_just_trashed(Path::new("/music-player-test/another.wav")));
}
