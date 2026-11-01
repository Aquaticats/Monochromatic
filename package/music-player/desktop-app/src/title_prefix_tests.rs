// What:     Unit tests for `title_prefix.rs`, pulled in by `#[cfg(test)] #[path = "title_prefix_tests.rs"] mod tests;`.
// Why:      D86 prefix stripping must remove a real shared prefix, and must not eat numbers, words or whole titles.

use super::*;

/// What: `shown` applies the shared prefix of `titles` to each title, with `folder` as the directory name.
/// Why: the tests read what a person would see in the rows.
fn shown(titles: &[&str], folder: &str) -> Vec<String> {
    let prefix: usize = shared_prefix_len(titles);
    return titles.iter().map(|title| return display_title(title, prefix, folder).to_string()).collect();
}

#[test]
fn an_artist_prefix_shared_by_the_folder_is_removed() {
    assert_eq!(
        shown(&["かめりあ(Camellia) - Aegis", "かめりあ(Camellia) - Another Xronixle", "かめりあ(Camellia) - 光"], "かめりあ(Camellia)"),
        vec!["Aegis", "Another Xronixle", "光"]
    );
}

#[test]
fn track_numbers_are_not_a_shared_prefix() {
    assert_eq!(shown(&["01 Hyperballad", "02 Jóga", "03 Army of Me"], "Björk"), vec!["01 Hyperballad", "02 Jóga", "03 Army of Me"]);
}

#[test]
fn a_shared_word_is_cut_back_to_a_separator() {
    assert_eq!(shown(&["Aegis", "Another"], "x"), vec!["Aegis", "Another"]);
    assert_eq!(shown(&["Artist - Aegis", "Artist - Another"], "x"), vec!["Aegis", "Another"]);
}

#[test]
fn a_numbered_name_loses_the_shared_word_and_keeps_its_number() {
    assert_eq!(shown(&["Track 001", "Track 002", "Track 100"], "x"), vec!["001", "002", "100"]);
}

#[test]
fn one_title_has_nothing_to_share() {
    assert_eq!(shared_prefix_len(&["Artist - Aegis"]), 0);
    assert_eq!(shared_prefix_len(&[]), 0);
}

#[test]
fn a_title_that_would_become_empty_stops_the_removal() {
    assert_eq!(shared_prefix_len(&["Artist - ", "Artist - Song"]), 0);
    assert_eq!(shown(&["Artist -", "Artist - Song"], "x"), vec!["Artist -", "Artist - Song"]);
}

#[test]
fn a_shared_start_without_a_letter_or_digit_is_kept() {
    assert_eq!(shared_prefix_len(&["- one", "- two"]), 0);
    assert_eq!(shared_prefix_len(&["(one", "(two"]), 0);
}

#[test]
fn identical_titles_are_kept_whole() {
    assert_eq!(shown(&["Same Name", "Same Name"], "x"), vec!["Same Name", "Same Name"]);
}

#[test]
fn only_exactly_equal_characters_count_as_shared() {
    assert_eq!(shown(&["Camellia - A", "camellia - B"], "x"), vec!["Camellia - A", "camellia - B"]);
}

#[test]
fn multibyte_titles_are_cut_on_character_boundaries() {
    assert_eq!(shown(&["米津玄師 - Lemon", "米津玄師 - 馬と鹿"], "x"), vec!["Lemon", "馬と鹿"]);
    assert_eq!(shown(&["Ólafur_Near", "Ólafur_Saman"], "x"), vec!["Near", "Saman"]);
}

#[test]
fn a_single_track_loses_its_folder_name() {
    assert_eq!(shown(&["Camellia - Aegis"], "Camellia"), vec!["Aegis"]);
    assert_eq!(shown(&["CAMELLIA: Aegis"], "camellia"), vec!["Aegis"]);
}

#[test]
fn the_folder_name_is_kept_when_it_is_part_of_a_longer_word_or_is_the_whole_title() {
    assert_eq!(strip_folder_name("Camellias bloom", "Camellia"), "Camellias bloom");
    assert_eq!(strip_folder_name("Camellia", "Camellia"), "Camellia");
    assert_eq!(strip_folder_name("Camellia - ", "Camellia"), "Camellia - ");
    assert_eq!(strip_folder_name("Aegis", ""), "Aegis");
    assert_eq!(strip_folder_name("Cam", "Camellia"), "Cam");
}

#[test]
fn the_folder_name_is_not_removed_when_titles_share_a_prefix_already() {
    assert_eq!(shown(&["Camellia - Aegis", "Camellia - Ghost"], "Camellia"), vec!["Aegis", "Ghost"]);
}

#[test]
fn a_prefix_length_that_is_not_a_boundary_of_this_title_falls_back_to_the_folder_rule() {
    assert_eq!(display_title("Z", 5, "Z"), "Z");
    assert_eq!(display_title("é - x", 1, "é"), "x");
}
