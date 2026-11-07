//! What:
//! Controls proving the reply generator reaches both kinds of reply and that the checks hold on a spread of inputs.
//! Why:
//! An invariant that is never reached proves nothing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(kindsReachedBy(generatedReply)).toEqual([2, 3]);
//! ```

/// Import the generator and the invariants under control.
use super::{check_generated_reply, check_raw_reply, generated_reply};

/// Inputs over a spread of leading bytes,
/// with longer derived ones.
fn inputs() -> Vec<Vec<u8>> {
    let mut all: Vec<Vec<u8>> = Vec::new();
    for first in 0..=255_u8 {
        for second in [0_u8, 1, 2, 3, 64, 127, 255] {
            all.push(vec![first, second, first ^ second, second]);
            all.push((0..48_u8).map(|index| { return first.wrapping_mul(31).wrapping_add(index.wrapping_mul(second)); }).collect());
        }
    }
    return all;
}

/// Both kinds of reply are reached,
/// with a name and a text on the errors.
#[test]
fn the_generator_reaches_returns_and_errors() {
    let mut returns: usize = 0;
    let mut errors: usize = 0;
    for data in inputs() {
        let (_, expected) = generated_reply(data.as_slice());
        if expected.error_name.is_some() {
            errors += 1;
            assert!(expected.first_text.is_some());
        } else {
            returns += 1;
        }
    }
    assert!(returns > 0 && errors > 0, "returns {returns}, errors {errors}");
}

/// The checks hold on every generated input.
#[test]
fn the_checks_hold_on_the_spread() {
    for data in inputs() {
        check_generated_reply(data.as_slice());
        check_raw_reply(data.as_slice());
    }
}
