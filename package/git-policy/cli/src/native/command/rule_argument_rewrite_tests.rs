//! What:
//!  Token insertion positions for fixed transforms.
//! Why:
//!  An off-by-one would put `-o` before `commit` or after a pathspec.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(insertTokens(['a', 'b'], 1, ['x'])).toEqual(['a', 'x', 'b']);
//! ```

/// The builder under test and the shared argument builder.
use super::insert_tokens;
use crate::command_test_support::os_arguments;
use std::ffi::OsString;

/// Tokens land before the indexed argument,
///  or at the end when the index is the length.
#[test]
fn inserts_before_the_indexed_argument() {
    let arguments: Vec<OsString> = os_arguments(&["a", "b"]);
    for (at, expected) in [
        (0, vec!["x", "y", "a", "b"]),
        (1, vec!["a", "x", "y", "b"]),
        (2, vec!["a", "b", "x", "y"]),
        (9, vec!["a", "b", "x", "y"]),
    ] {
        assert_eq!(
            insert_tokens(arguments.as_slice(), at, &["x", "y"]),
            os_arguments(expected.as_slice()),
            "{at}"
        );
    }
    assert_eq!(insert_tokens(&[], 0, &["x"]), os_arguments(&["x"]));
    assert_eq!(
        insert_tokens(arguments.as_slice(), 1, &[]),
        os_arguments(&["a", "b"])
    );
}
