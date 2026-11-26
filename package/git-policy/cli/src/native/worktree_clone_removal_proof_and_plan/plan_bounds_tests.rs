//! What:
//!  The controls of the plan's byte bounds:
//!  the widest header is under the design's cap,
//!  the widest attempt line is the design's 361,
//!  and the file bound grows by one attempt line at a time.
//! Why:
//!  The reader refuses an oversized file before it reads an attempt byte,
//!  so a bound that was smaller than what the writer can produce would refuse the removal's own
//!  record
//!  (`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
//! `F11`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(widestHeaderBytes()).toBeLessThanOrEqual(PLAN_HEADER_MAXIMUM_BYTES);
//! ```

use super::{
    ATTEMPT_LINE_MAXIMUM_BYTES,
    PLAN_HEADER_MAXIMUM_BYTES,
    bare_line_bytes,
    header_field_maximum_bytes,
    line_bytes,
    plan_maximum_bytes,
    widest_attempt_line_bytes,
    widest_header_bytes,
};
use crate::worktree_clone_removal_proof_and_plan::plan_words::{
    PLAN_HEADER,
    PLAN_KEYS_IN_ORDER,
};

#[test]
fn the_widest_header_a_writer_reaches_is_under_the_readers_cap() {
    let widest: u64 = widest_header_bytes();
    assert!(
        widest <= PLAN_HEADER_MAXIMUM_BYTES,
        "the writer can reach {widest} bytes and the reader caps at {PLAN_HEADER_MAXIMUM_BYTES}"
    );
    // The margin is small on purpose: a field added here without its own bound would pass the cap
    // only because the arithmetic above counts keys, so the sum is asserted against the design's
    // number and not against itself.
    assert!(
        PLAN_HEADER_MAXIMUM_BYTES - widest < 100,
        "the cap and the sum stay close, so the sum is a bound and not a rounding"
    );
}

#[test]
fn every_key_in_the_header_has_one_bound() {
    for key in PLAN_KEYS_IN_ORDER {
        assert!(
            header_field_maximum_bytes(key).is_some(),
            "`{key}` has a bound"
        );
    }
    assert_eq!(header_field_maximum_bytes("notAKey"), None);
    assert_eq!(
        header_field_maximum_bytes("run"),
        Some(line_bytes("run", 32))
    );
}

#[test]
fn the_widest_attempt_line_is_the_designs_361_bytes() {
    assert_eq!(widest_attempt_line_bytes(), ATTEMPT_LINE_MAXIMUM_BYTES);
    assert_eq!(ATTEMPT_LINE_MAXIMUM_BYTES, 361);
}

#[test]
fn the_file_bound_is_the_header_plus_one_line_per_declared_attempt() {
    assert_eq!(plan_maximum_bytes(0), PLAN_HEADER_MAXIMUM_BYTES);
    assert_eq!(
        plan_maximum_bytes(1),
        PLAN_HEADER_MAXIMUM_BYTES + ATTEMPT_LINE_MAXIMUM_BYTES
    );
    assert_eq!(
        plan_maximum_bytes(100),
        PLAN_HEADER_MAXIMUM_BYTES + 100 * ATTEMPT_LINE_MAXIMUM_BYTES
    );
}

#[test]
fn the_header_line_is_counted_at_its_own_length() {
    assert_eq!(bare_line_bytes(PLAN_HEADER), PLAN_HEADER.len() as u64 + 1);
}
