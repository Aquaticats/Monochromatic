//! What:
//!  The controls of the record's closed sets:
//!  the two states,
//!  the four reasons,
//!  and the folder and name constants the record is keyed by.
//! Why:
//!  A word outside a set refuses a record rather than being guessed,
//!  so every word must round trip and nothing else may parse.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(CleanupState.parse('owed')).toBe(CleanupState.Owed);
//! expect(CleanupState.parse('done')).toBeUndefined();
//! ```

use crate::worktree_clone_origin_cleanup::origin_cleanup_words::{
    BlockReason,
    CleanupState,
    ORIGIN_CLEANUP_HEADER,
    ORIGIN_CLEANUP_KEYS_IN_ORDER,
    ORIGIN_CLEANUP_NAME_BYTES,
    ORIGIN_CLEANUPS_FOLDER,
    ORIGIN_SNAPSHOT_NONCE_DIGITS,
    ORIGIN_SNAPSHOT_PREFIX,
};

#[test]
fn every_state_round_trips_and_no_other_word_parses() {
    for state in [CleanupState::Owed, CleanupState::Blocked] {
        assert_eq!(CleanupState::parse(state.word()), Some(state));
    }
    assert_eq!(
        CleanupState::parse("done"),
        None,
        "completion is an absence, not a word"
    );
}

#[test]
fn every_reason_round_trips_and_no_other_word_parses() {
    for reason in [
        BlockReason::Dependency,
        BlockReason::Deferred,
        BlockReason::Unproved,
        BlockReason::Refused,
    ] {
        assert_eq!(BlockReason::parse(reason.word()), Some(reason));
    }
    assert_eq!(BlockReason::parse("busy"), None);
}

#[test]
fn the_header_and_the_name_agree_with_the_key_order() {
    assert!(ORIGIN_CLEANUP_HEADER.contains("version 1"));
    assert_eq!(
        ORIGIN_CLEANUP_KEYS_IN_ORDER.len(),
        15,
        "the design's fifteen keys"
    );
    assert_eq!(
        ORIGIN_CLEANUP_NAME_BYTES,
        ORIGIN_SNAPSHOT_PREFIX.len() + ORIGIN_SNAPSHOT_NONCE_DIGITS
    );
    assert_eq!(ORIGIN_CLEANUPS_FOLDER, "origin-cleanups");
}
