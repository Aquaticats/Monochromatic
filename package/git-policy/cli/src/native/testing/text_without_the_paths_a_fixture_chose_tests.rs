//! Controls for cutting the paths a fixture chose out of the text a control judges.

use super::without_the_paths;
use std::path::Path;

/// A path in the middle of a sentence goes, and the words on both sides of it stay.
#[test]
fn a_path_goes_and_the_words_around_it_stay() {
    let said: &str = "batch 7 in the repository at /run/x/repo is held";
    assert_eq!(without_the_paths(said, &[Path::new("/run/x")]), "batch 7 in the repository at /repo is held");
}

/// A word that nothing but a folder's name supplied is not in the words a control judges.
#[test]
fn a_word_only_a_folder_supplied_is_not_judged() {
    let said: &str = "the lock /run/a-folder-that-says-refus-inside/repo/.git/index.lock was kept";
    assert!(!without_the_paths(said, &[Path::new("/run/a-folder-that-says-refus-inside")]).contains("refus"));
}

/// A word the wrapper wrote beside a path stays, so forbidding it still fails the control.
#[test]
fn a_word_the_wrapper_wrote_beside_a_path_stays() {
    let said: &str = "the lock /run/x/repo/.git/index.lock was not cleared";
    assert!(without_the_paths(said, &[Path::new("/run/x")]).contains("cleared"));
}
