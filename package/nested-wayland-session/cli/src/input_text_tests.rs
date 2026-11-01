//! Unit tests for the typing chunker.

use super::{check_typable, next_chunk};
use crate::unicode_keymap::MAX_SPARE_KEYS;

/// US text needs no spare keys and is one chunk.
#[test]
fn ascii_text_needs_no_spare_keys() {
    let characters: Vec<char> = "Hello, World! 123\t".chars().collect();
    assert_eq!(next_chunk(&characters), (characters.len(), vec![]));
}

/// Off-layout characters are collected once each in first-use order.
#[test]
fn off_layout_characters_are_collected_once() {
    let characters: Vec<char> = "Bj\u{f6}rk \u{f6}\u{304b}\u{3081}".chars().collect();
    assert_eq!(next_chunk(&characters), (characters.len(), vec!['\u{f6}', '\u{304b}', '\u{3081}']));
}

/// A text with more distinct off-layout characters than one keymap carries is split.
#[test]
fn a_long_text_splits_at_the_spare_key_limit() {
    let characters: Vec<char> = (0..(MAX_SPARE_KEYS + 5)).filter_map(|offset| return char::from_u32(0x4e00 + offset as u32)).collect();
    let (taken, spare) = next_chunk(&characters);
    assert_eq!(taken, MAX_SPARE_KEYS);
    assert_eq!(spare.len(), MAX_SPARE_KEYS);
    let (rest, more) = next_chunk(&characters[taken..]);
    assert_eq!(rest, 5);
    assert_eq!(more.len(), 5);
}

/// A repeat of a character already in the keymap does not end the chunk.
#[test]
fn repeats_do_not_count_against_the_limit() {
    let mut characters: Vec<char> = (0..MAX_SPARE_KEYS).filter_map(|offset| return char::from_u32(0x4e00 + offset as u32)).collect();
    characters.extend(['\u{4e00}', 'a']);
    assert_eq!(next_chunk(&characters).0, MAX_SPARE_KEYS + 2);
}

/// Control characters other than tab and line feed are refused.
#[test]
fn control_characters_are_refused() {
    assert!(check_typable("tab\there").is_ok());
    assert!(check_typable("\u{f6}\u{1F600}").is_ok());
    let error = check_typable("a\u{1b}b").map_err(|error| return error.to_string());
    assert_eq!(error, Err("type cannot send the control character U+001B at position 1".to_string()));
}
