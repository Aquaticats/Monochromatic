//! Space held for hints that have left or have not arrived:
//!  block heights stay until the hints return or the hold's time passes.

use crate::fixtures::{
    SHOWN,
    TEXT,
    block_lines,
    blocks,
    diagnostics,
    hint,
    problem,
    ranged,
    unwrapped,
};
use helix_core::Rope;
use ide_app::{
    annotation::{
        Annotations,
        Held,
    },
    language::identity::DocumentStamp,
    virtual_row::{
        BLOCK_GAP,
        ROW_HEIGHT,
    },
};
use std::time::{
    Duration,
    Instant,
};

/// After an external change the previous rows' space is held above the lines it is mapped to:
///  hint space until
/// hints for those lines arrive,
///  message space until its time has passed,
///  and nothing after that.
#[test]
fn held_space_keeps_block_heights_until_hints_return_or_time_passes() {
    let text = Rope::from(TEXT);
    let start = Instant::now();
    let until = start + Duration::from_secs(1);
    let mut store = Annotations::default();
    let before = store.version();
    store.hold(
        SHOWN,
        vec![
            Held {
                line: 1,
                hints: ROW_HEIGHT,
                messages: 2.0 * ROW_HEIGHT,
            },
            Held {
                line: 3,
                hints: ROW_HEIGHT,
                messages: 0.0,
            },
        ],
        until,
    );
    assert_ne!(
        store.version(),
        before,
        "holding space must rebuild the map"
    );
    let held = blocks(&mut store, SHOWN);
    assert_eq!(held.len(), 2);
    assert_eq!(held[0].height(), BLOCK_GAP + 3.0 * ROW_HEIGHT);
    assert_eq!(held[1].height(), BLOCK_GAP + ROW_HEIGHT);
    assert!(
        held[0].hints.is_empty() && held[0].messages.is_empty(),
        "held space paints nothing"
    );
    // Held space belongs to one text only.
    let other = DocumentStamp {
        file: 3,
        revision: 8,
    };
    assert!(
        blocks(&mut store, other).is_empty(),
        "held space was applied to another text"
    );
    // Hints answering for lines 0 to 2 give up the hint space of line 1; its message space stays.
    let answer = ranged(SHOWN, (0, 3), vec![hint(5, ": i32")]);
    assert!(store.accept_hints(SHOWN, &text, unwrapped(answer)));
    let answered = blocks(&mut store, SHOWN);
    let lines: Vec<usize> = block_lines(&answered);
    assert_eq!(lines, [0, 1, 3]);
    assert_eq!(
        answered[1].height(),
        BLOCK_GAP + 2.0 * ROW_HEIGHT,
        "hint space was not given up when hints for the line arrived"
    );
    assert_eq!(
        answered[2].height(),
        BLOCK_GAP + ROW_HEIGHT,
        "a line the answer did not cover lost its space"
    );
    // Diagnostics of the new text fill the held message space instead of adding to it.
    let found = diagnostics(
        SHOWN,
        vec![("rustc", vec![problem(11, 14, None, "one row")])],
    );
    assert!(store.accept_diagnostics(SHOWN, &text, found.expect("diagnostics snapshot")));
    let filled = blocks(&mut store, SHOWN);
    assert_eq!(filled[1].messages.len(), 1);
    assert_eq!(
        filled[1].height(),
        BLOCK_GAP + 2.0 * ROW_HEIGHT,
        "message space must stay held until its time has passed"
    );
    assert!(
        !store.expire(start + Duration::from_millis(999)),
        "space was given up early"
    );
    assert!(
        store.expire(until),
        "space was not given up when its time had passed"
    );
    let after = blocks(&mut store, SHOWN);
    let remaining: Vec<usize> = block_lines(&after);
    assert_eq!(remaining, [0, 1], "held space outlived its time");
    assert_eq!(after[1].height(), BLOCK_GAP + ROW_HEIGHT);
    assert!(!store.expire(until + Duration::from_secs(5)));
}
