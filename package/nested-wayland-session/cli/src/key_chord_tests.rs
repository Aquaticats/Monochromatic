//! Unit tests for the chord grammar.

use super::{parse_chord, Chord};

/// A bare name is a chord of one key.
#[test]
fn a_bare_name_is_one_key() {
    assert_eq!(parse_chord("f10"), Ok(Chord { keys: vec![68] }));
    assert_eq!(parse_chord("code:68"), Ok(Chord { keys: vec![68] }));
}

/// Modifiers come first in press order and the target is last.
#[test]
fn modifiers_precede_the_target() {
    assert_eq!(parse_chord("ctrl+shift+f10"), Ok(Chord { keys: vec![29, 42, 68] }));
    assert_eq!(parse_chord("Shift+F10"), Ok(Chord { keys: vec![42, 68] }));
    assert_eq!(parse_chord("alt+Left"), Ok(Chord { keys: vec![56, 105] }));
    assert_eq!(parse_chord("ctrl+code:30"), Ok(Chord { keys: vec![29, 30] }));
}

/// The plus key is written as `++` and is the `=` key.
#[test]
fn the_plus_key_is_spelled_with_two_pluses() {
    assert_eq!(parse_chord("+"), Ok(Chord { keys: vec![13] }));
    assert_eq!(parse_chord("ctrl++"), Ok(Chord { keys: vec![29, 13] }));
    assert_eq!(parse_chord("ctrl+shift++"), Ok(Chord { keys: vec![29, 42, 13] }));
    assert_eq!(parse_chord("ctrl+plus"), Ok(Chord { keys: vec![29, 13] }));
}

/// A malformed chord fails whole and names the problem.
#[test]
fn malformed_chords_fail_whole() {
    assert_eq!(parse_chord(""), Err("key requires a name".to_string()));
    assert_eq!(parse_chord("ctrl+nokey"), Err("unknown key: nokey".to_string()));
    assert_eq!(parse_chord("bogus+f10"), Err("unknown key: bogus".to_string()));
    assert_eq!(
        parse_chord("ctrl+"),
        Err("key chord ctrl+ ends with a separator; write the plus key as ++".to_string()),
    );
}
