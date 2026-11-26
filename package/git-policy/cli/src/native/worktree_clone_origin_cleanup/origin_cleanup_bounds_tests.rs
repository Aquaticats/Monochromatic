//! What:
//!  The controls of the record's bounds:
//!  every key has a width,
//!  and the widest fields sum under the reader's cap.
//! Why:
//!  The design computes 9,217 bytes field by field,
//!  so a table that missed a key would let a writer publish a record the reader refuses.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(fieldMaximumBytes('slotPath')).toBe(8190);
//! ```

use crate::worktree_clone_origin_cleanup::origin_cleanup_bounds::{
    FIELD_WIDEST_BYTES,
    ORIGIN_CLEANUP_MAXIMUM_BYTES,
    field_maximum_bytes,
    header_bytes,
};
use crate::worktree_clone_origin_cleanup::origin_cleanup_words::ORIGIN_CLEANUP_KEYS_IN_ORDER;

#[test]
fn every_key_of_the_grammar_has_one_width_and_nothing_else_does() {
    for key in ORIGIN_CLEANUP_KEYS_IN_ORDER {
        assert!(field_maximum_bytes(key).is_some(), "`{key}` has a width");
    }
    assert_eq!(field_maximum_bytes("cloneGuidExtra"), None);
    assert_eq!(FIELD_WIDEST_BYTES.len(), ORIGIN_CLEANUP_KEYS_IN_ORDER.len());
}

#[test]
fn the_widest_record_fits_the_caps_the_design_computed() {
    let mut total: u64 = header_bytes() as u64;
    for (key, widest) in FIELD_WIDEST_BYTES {
        total += key.len() as u64 + 1 + widest as u64 + 1;
    }
    assert!(
        total >= ORIGIN_CLEANUP_MAXIMUM_BYTES,
        "the widest fields reach the design's cap, which is what makes it the cap: {total}"
    );
    assert_eq!(field_maximum_bytes("slotPath"), Some(2 * 4_095));
}
