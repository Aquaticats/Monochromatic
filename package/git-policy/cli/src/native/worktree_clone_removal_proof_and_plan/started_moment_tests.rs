//! What:
//!  The controls of the `started` field:
//!  this moment's own shape,
//!  and each shape the grammar refuses.
//! Why:
//!  A reader that read a moment out of an unchecked string would decide something by a value nothing
//!  guarantees,
//!  and the plan's own rule is that no reader decides anything by the moment at all
//!  (`doc/planning/cli-git-zfs-removal-destruction-plan-record-design.md`,
//!  "Field by field").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(isSecondsWithOffset(nowWithOffset())).toBe(true);
//! ```

use super::{
    STAMP_WITH_OFFSET_BYTES,
    is_seconds_with_offset,
    now_with_offset,
};

#[test]
fn the_moment_this_process_writes_is_one_the_grammar_reads() {
    let now: String = now_with_offset();
    assert_eq!(
        now.len(),
        STAMP_WITH_OFFSET_BYTES,
        "the stamp is its own fixed width: {now}"
    );
    assert!(
        is_seconds_with_offset(now.as_str()),
        "{now} is an ISO 8601 second with an offset"
    );
}

#[test]
fn a_stamp_without_an_offset_a_fraction_or_another_shape_is_refused() {
    assert!(is_seconds_with_offset("2026-11-22T18:40:12-05:00"));
    assert!(is_seconds_with_offset("2026-11-22T23:40:12+00:00"));
    for refused in [
        "2026-11-22T18:40:12",
        "2026-11-22T18:40:12.123-05:00",
        "2026-11-22T18:40:12Z",
        "2026-11-22 18:40:12-05:00",
        "2026-11-22T18:40:12-0500",
        "2026-13-22T18:40:12-05:00",
        "2026-11-22T18:40-05:00",
        "",
    ] {
        assert!(
            !is_seconds_with_offset(refused),
            "`{refused}` is not this field's shape"
        );
    }
}
