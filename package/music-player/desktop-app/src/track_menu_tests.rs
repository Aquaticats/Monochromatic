// What:     Unit tests for `track_menu.rs`, pulled in by
//           `#[cfg(test)] #[path = "track_menu_tests.rs"] mod tests;` at the bottom of `track_menu.rs`.
//           Mirrors the Kotlin `TrackMenuActionsTest` cases, the divider cases of `TrackMenuPopupModelTest`,
//           and the details suffix cases of `TrackMenuPopupModelTest`. The glyph cases are not ported.
// Why:      Keep the tests beside the code. Each expected value is the accepted D7 text or order.

// What:     `use super::*;` imports everything from the parent module into the test scope.
// Why:      Tests need the action enum, `label`, `track_menu_actions_for`, and the helpers.
use super::*;

use std::collections::HashSet;

// What:     `#[test] fn order_matches_decision_d7()` checks the list follows the D7 actions in order (six since D113 removed Re-analyze).
// Why:      The menu must show the actions in exactly the order the decision lists them.
#[test]
fn order_matches_decision_d7() {
    assert_eq!(
        track_menu_actions_for(),
        [
            TrackMenuAction::Play,
            TrackMenuAction::ShuffleFromHere,
            TrackMenuAction::Details,
            TrackMenuAction::ShowInFileManager,
            TrackMenuAction::CopyFilename,
            TrackMenuAction::MoveToTrash,
        ]
    );
}

// What:     `#[test] fn play_label_is_play()` checks the first action's wording.
// Why:      The first D7 row must read Play exactly.
#[test]
fn play_label_is_play() {
    assert_eq!(label(TrackMenuAction::Play), "Play");
}

// What:     `#[test] fn shuffle_label_is_accepted()` checks the shuffle action's wording.
// Why:      The accepted D7 wording must be reproduced exactly, including case and spacing.
#[test]
fn shuffle_label_is_accepted() {
    assert_eq!(label(TrackMenuAction::ShuffleFromHere), "Start shuffle from here");
}

// What:     `#[test] fn details_label_is_accepted()` checks the file details action's wording.
// Why:      The inline peak value is a separate caller concern, so this label is the bare D7 text.
#[test]
fn details_label_is_accepted() {
    assert_eq!(label(TrackMenuAction::Details), "File details");
}

// What:     `#[test] fn no_row_offers_to_analyze_again()` checks that no menu row names re-analysis or a rescan.
// Why:      D113: refresh and re-analysis are automatic and never exposed, so the menu has no entry for either.
#[test]
fn no_row_offers_to_analyze_again() {
    for action in track_menu_actions_for() {
        let text = label(*action).to_lowercase();
        assert!(!text.contains("analy") && !text.contains("rescan") && !text.contains("refresh"), "{text}");
    }
}

// What:     `#[test] fn show_label_is_accepted()` checks the file manager action's wording.
// Why:      The accepted D7 wording names the file manager explicitly.
#[test]
fn show_label_is_accepted() {
    assert_eq!(label(TrackMenuAction::ShowInFileManager), "Show in file manager");
}

// What:     `#[test] fn copy_label_is_accepted()` checks the copy action's wording.
// Why:      The accepted D7 wording uses the single word filename.
#[test]
fn copy_label_is_accepted() {
    assert_eq!(label(TrackMenuAction::CopyFilename), "Copy filename");
}

// What:     `#[test] fn trash_label_is_accepted()` checks the destructive action's wording.
// Why:      Decision D8 names the action Move to trash, and the label must not hide that meaning.
#[test]
fn trash_label_is_accepted() {
    assert_eq!(label(TrackMenuAction::MoveToTrash), "Move to trash");
}

// What:     `#[test] fn only_trash_is_destructive()` checks which actions carry the destructive flag.
// Why:      Exactly one action removes a file, and the menu must mark only that one.
#[test]
fn only_trash_is_destructive() {
    let destructive: Vec<TrackMenuAction> = track_menu_actions_for()
        .iter()
        .copied()
        .filter(|action| return action.destructive())
        .collect();
    assert_eq!(destructive, [TrackMenuAction::MoveToTrash]);
}

// What:     `#[test] fn groups_never_decrease()` checks that the display order keeps each group contiguous.
// Why:      Dividers are drawn at group changes, so a group that reappears would split a section.
#[test]
fn groups_never_decrease() {
    let groups: Vec<u8> = track_menu_actions_for()
        .iter()
        .map(|action| return action.group())
        .collect();
    for pair in groups.windows(2) {
        assert!(pair[0] <= pair[1], "group {} followed by {}", pair[0], pair[1]);
    }
}

// What:     `#[test] fn labels_are_distinct()` checks that no two actions share a visible label.
// Why:      Identical labels would make two rows indistinguishable to a user and to a screen reader.
#[test]
fn labels_are_distinct() {
    let labels: Vec<&str> = track_menu_actions_for()
        .iter()
        .map(|action| return label(*action))
        .collect();
    let distinct: HashSet<&str> = labels.iter().copied().collect();
    assert_eq!(labels.len(), distinct.len());
}

// What:     `#[test] fn non_destructive_actions_are_not_marked()` checks that Play is not destructive.
// Why:      Play is the default action and must never carry the destructive marking.
#[test]
fn non_destructive_actions_are_not_marked() {
    assert!(!TrackMenuAction::Play.destructive());
}

// What:     `#[test] fn divider_flags_follow_d7_groups()` checks where the dividers fall in the full menu.
// Why:      Dividers appear before File details and before Show in file manager only (Re-analyze is gone, D113).
#[test]
fn divider_flags_follow_d7_groups() {
    assert_eq!(
        track_menu_divider_flags(track_menu_actions_for()),
        [false, false, true, true, false, false]
    );
}

// What:     `#[test] fn empty_list_has_no_dividers()` checks that an empty action list yields no flags.
// Why:      An empty menu draws no dividers.
#[test]
fn empty_list_has_no_dividers() {
    assert_eq!(track_menu_divider_flags(&[]), Vec::<bool>::new());
}

// What:     `#[test] fn single_action_has_no_divider()` checks that a lone action never gets a divider.
// Why:      The first action has no neighbour above it, so it never starts a section.
#[test]
fn single_action_has_no_divider() {
    assert_eq!(track_menu_divider_flags(&[TrackMenuAction::Play]), [false]);
}

// What:     `#[test] fn same_group_has_no_divider()` checks that neighbours with equal groups get no divider.
// Why:      Actions in one section must stay together without a rule between them.
#[test]
fn same_group_has_no_divider() {
    assert_eq!(
        track_menu_divider_flags(&[TrackMenuAction::Play, TrackMenuAction::ShuffleFromHere]),
        [false, false]
    );
}

// What:     `#[test] fn every_group_change_draws_a_divider()` checks that each group change gets a divider.
// Why:      Every neighbour that changes group starts a new section.
#[test]
fn every_group_change_draws_a_divider() {
    assert_eq!(
        track_menu_divider_flags(&[
            TrackMenuAction::Play,
            TrackMenuAction::Details,
            TrackMenuAction::CopyFilename,
        ]),
        [false, true, true]
    );
}

// What:     `#[test] fn details_suffix_is_empty_for_null()` checks that an absent value adds no text.
// Why:      A track without analysis must show File details with no trailing separator.
#[test]
fn details_suffix_is_empty_for_null() {
    assert_eq!(track_menu_details_suffix(None), "");
}

// What:     `#[test] fn details_suffix_is_empty_for_blank_value()` checks that a whitespace-only value adds no text.
// Why:      A blank value would otherwise render a dangling middle dot beside File details.
#[test]
fn details_suffix_is_empty_for_blank_value() {
    assert_eq!(track_menu_details_suffix(Some("   ")), "");
}

// What:     `#[test] fn details_suffix_joins_value_with_separator()` checks the separator and the value text.
// Why:      The inline peak must read as the label, a middle dot, and the dB value.
#[test]
fn details_suffix_joins_value_with_separator() {
    assert_eq!(track_menu_details_suffix(Some("-0.8 dBTP")), " · -0.8 dBTP");
}

// What:     `#[test] fn details_suffix_keeps_value_exact()` checks that the value text is not trimmed.
// Why:      The analysed value must appear exactly as authored, including its own spacing.
#[test]
fn details_suffix_keeps_value_exact() {
    assert_eq!(track_menu_details_suffix(Some(" -1.2 dBTP ")), " ·  -1.2 dBTP ");
}

// What:     `#[test] fn every_row_has_one_unique_access_letter_taken_from_its_own_label()` checks the access table.
// Why:      D114: one unique letter per row, from the row's own words, drawn underlined; the letters live in one table.
#[test]
fn every_row_has_one_unique_access_letter_taken_from_its_own_label() {
    let mut seen: HashSet<char> = HashSet::new();
    for action in track_menu_actions_for() {
        let letter = access_key(*action);
        assert!(letter.is_alphabetic() && letter.is_lowercase(), "{action:?} has the access letter {letter:?}");
        assert!(label(*action).to_lowercase().contains(letter), "{letter} is a letter of {}", label(*action));
        assert!(seen.insert(letter), "{letter} is used by two rows");
    }
    assert_eq!(seen.len(), track_menu_actions_for().len());
}

// What:     `#[test] fn each_access_letter_finds_its_row_whatever_its_case()` checks the lookup.
// Why:      The key classifier names a row from the pressed letter, so the lookup must invert the table exactly.
#[test]
fn each_access_letter_finds_its_row_whatever_its_case() {
    for action in track_menu_actions_for() {
        let letter = access_key(*action);
        assert_eq!(access_action(letter), Some(*action));
        assert_eq!(access_action(letter.to_ascii_uppercase()), Some(*action));
    }
    assert_eq!(access_action('z'), None);
    assert_eq!(access_action('1'), None);
}

// What:     `#[test] fn the_label_splits_around_its_access_letter_and_joins_back()` checks the underline split.
// Why:      The window draws the middle part underlined; the three parts must join into the visible label.
#[test]
fn the_label_splits_around_its_access_letter_and_joins_back() {
    for action in track_menu_actions_for() {
        let (before, key, after) = access_label(*action);
        assert_eq!(format!("{before}{key}{after}"), label(*action));
        assert_eq!(key.to_lowercase(), access_key(*action).to_string());
    }
    assert_eq!(access_label(TrackMenuAction::Details), ("File ".to_string(), "d".to_string(), "etails".to_string()));
    assert_eq!(access_label(TrackMenuAction::ShowInFileManager).1, "f");
}
