//! What:
//!  Controls that one failed call's errno reaches the record's closed
//!  vocabulary,
//!  and a number the build does not name stays unnamed.
//! Why:
//!  An invented word would make the record read as if a call answered
//!  something it did not.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(errnoName({ call: 'mount', errno: EBUSY })).toBe('EBUSY');
//! ```

use super::CallError;

/// A named errno becomes its word,
/// and an unnamed one becomes nothing.
#[test]
fn the_errno_becomes_its_word_or_nothing() {
    assert_eq!(
        CallError {
            call: "mount",
            errno: libc::EBUSY
        }
        .errno_name(),
        Some("EBUSY")
    );
    assert_eq!(
        CallError {
            call: "mount",
            errno: 4242
        }
        .errno_name(),
        None
    );
}
