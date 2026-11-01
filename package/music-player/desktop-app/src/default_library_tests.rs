// What:     Unit tests for `default_library.rs`, pulled in by `#[cfg(test)] #[path = "default_library_tests.rs"] mod tests;`.
// Why:      EM7: the home `Music` folder is the last candidate, a missing folder is skipped, and later candidates are not
//           computed once one is found. Every case uses a disposable home folder under the system temp directory.

use super::*;

/// What: `scratch_home` makes an empty disposable folder and returns it.
/// Why: the tests must never look at the real home folder (HOM, THR).
fn scratch_home(tag: &str) -> PathBuf {
    let nanos: u128 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| return elapsed.as_nanos());
    let home: PathBuf = crate::test_scratch::scratch_root().join(format!("music-player-home-{}-{nanos}-{tag}", std::process::id()));
    std::fs::create_dir_all(&home).expect("the scratch home is created");
    return home;
}

#[test]
fn the_home_music_folder_is_found_when_nothing_else_is_set() {
    let home = scratch_home("home-music");
    std::fs::create_dir_all(home_music_folder(&home)).expect("the Music folder is created");
    let found = first_existing_dir([None, None, Some(home_music_folder(&home))].into_iter());
    assert_eq!(found, Some(home.join("Music")));
    std::fs::remove_dir_all(&home).ok();
}

#[test]
fn a_home_without_a_music_folder_gives_none() {
    let home = scratch_home("no-music");
    assert_eq!(first_existing_dir([None, Some(home_music_folder(&home))].into_iter()), None);
    std::fs::remove_dir_all(&home).ok();
}

#[test]
fn a_set_but_missing_folder_is_skipped_for_the_next_candidate() {
    let home = scratch_home("skip-missing");
    std::fs::create_dir_all(home_music_folder(&home)).expect("the Music folder is created");
    let missing = home.join("Gone");
    assert_eq!(first_existing_dir([Some(missing), Some(home_music_folder(&home))].into_iter()), Some(home.join("Music")));
    std::fs::remove_dir_all(&home).ok();
}

#[test]
fn an_earlier_folder_wins_and_later_candidates_are_not_computed() {
    let home = scratch_home("order");
    let configured = home.join("Configured");
    std::fs::create_dir_all(&configured).expect("the configured folder is created");
    std::fs::create_dir_all(home_music_folder(&home)).expect("the Music folder is created");
    let computed = std::cell::Cell::new(false);
    let candidates = std::iter::once(Some(configured.clone())).chain(std::iter::once_with(|| {
        computed.set(true);
        return Some(home_music_folder(&home));
    }));
    assert_eq!(first_existing_dir(candidates), Some(configured));
    assert!(!computed.get(), "the later candidate is never asked for");
    std::fs::remove_dir_all(&home).ok();
}

#[test]
fn a_file_named_music_is_not_a_folder() {
    let home = scratch_home("file");
    std::fs::write(home_music_folder(&home), b"x").expect("the file is written");
    assert_eq!(first_existing_dir([Some(home_music_folder(&home))].into_iter()), None);
    std::fs::remove_dir_all(&home).ok();
}
