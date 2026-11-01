//! Tests for the focus ring movement inside lists (D106).
//!
//! What: lists are a flag per entry, true where the ring can rest, and false for headers and headings.
//! Why: Up and Down must skip headers, Home and End must find the first and last real entry, and a letter must find the
//! first folder with that initial.

use super::*;

/// What: `rows` is a list shaped like a track list: a header, two tracks, a header, three tracks.
const ROWS: [bool; 7] = [false, true, true, false, true, true, true];

#[test]
fn down_skips_a_header_and_up_skips_it_back() {
    assert_eq!(move_focus(&ROWS, Some(2), ListMove::Down, 3), Some(4));
    assert_eq!(move_focus(&ROWS, Some(4), ListMove::Up, 3), Some(2));
}

#[test]
fn the_ring_stops_at_the_ends_and_does_not_wrap() {
    assert_eq!(move_focus(&ROWS, Some(6), ListMove::Down, 3), Some(6));
    assert_eq!(move_focus(&ROWS, Some(1), ListMove::Up, 3), Some(1));
}

#[test]
fn home_and_end_go_to_the_first_and_last_entry_that_can_hold_the_ring() {
    assert_eq!(move_focus(&ROWS, Some(4), ListMove::Home, 3), Some(1));
    assert_eq!(move_focus(&ROWS, Some(2), ListMove::End, 3), Some(6));
}

#[test]
fn page_keys_move_by_a_page_of_entries_and_stop_at_the_ends() {
    assert_eq!(move_focus(&ROWS, Some(1), ListMove::PageDown, 3), Some(5));
    assert_eq!(move_focus(&ROWS, Some(5), ListMove::PageDown, 3), Some(6));
    assert_eq!(move_focus(&ROWS, Some(6), ListMove::PageUp, 3), Some(2));
    assert_eq!(move_focus(&ROWS, Some(2), ListMove::PageUp, 9), Some(1));
}

#[test]
fn a_page_of_zero_still_moves_one_entry() {
    assert_eq!(move_focus(&ROWS, Some(1), ListMove::PageDown, 0), Some(2));
}

#[test]
fn a_ring_that_is_not_on_a_valid_entry_starts_at_the_first_or_last_entry() {
    assert_eq!(move_focus(&ROWS, None, ListMove::Down, 3), Some(1));
    assert_eq!(move_focus(&ROWS, Some(3), ListMove::Up, 3), Some(1));
    assert_eq!(move_focus(&ROWS, Some(99), ListMove::Down, 3), Some(1));
    assert_eq!(move_focus(&ROWS, None, ListMove::End, 3), Some(6));
    assert_eq!(move_focus(&ROWS, None, ListMove::PageDown, 3), Some(6));
}

#[test]
fn a_list_without_selectable_entries_has_no_ring() {
    assert_eq!(move_focus(&[false, false], Some(0), ListMove::Down, 3), None);
    assert_eq!(move_focus(&[], None, ListMove::Home, 3), None);
}

#[test]
fn the_start_position_is_the_preferred_entry_or_the_first_selectable_one() {
    assert_eq!(start_focus(&ROWS, Some(5)), Some(5));
    assert_eq!(start_focus(&ROWS, Some(3)), Some(1));
    assert_eq!(start_focus(&ROWS, None), Some(1));
    assert_eq!(start_focus(&ROWS, Some(40)), Some(1));
    assert_eq!(start_focus(&[false], None), None);
}

#[test]
fn a_letter_finds_the_first_label_that_starts_with_it_ignoring_case() {
    let labels = [None, Some("Abba"), Some("beck"), None, Some("Bowie"), Some("Cher")];
    assert_eq!(first_starting_with(&labels, 'b'), Some(2));
    assert_eq!(first_starting_with(&labels, 'B'), Some(2));
    assert_eq!(first_starting_with(&labels, 'c'), Some(5));
    assert_eq!(first_starting_with(&labels, 'z'), None);
}

#[test]
fn a_digit_finds_a_label_that_starts_with_it() {
    let labels = [Some("10cc"), Some("2Pac")];
    assert_eq!(first_starting_with(&labels, '2'), Some(1));
}
