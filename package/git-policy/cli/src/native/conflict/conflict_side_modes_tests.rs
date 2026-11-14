//! What:
//!  The pathspec modes a command runs under.
//! Why:
//!  A mode that is missed tells a reader that a token names one file when it names a set,
//! and a mode named wrongly sends a person to change something that does not matter.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(modesInEffect(['--icase-pathspecs'], [])).toEqual(['--icase-pathspecs']);
//! ```

use super::{MODE_OPTIONS, modes_in_effect};
use std::ffi::OsString;

/// Every global option is found in the prefix,
/// in the order the table lists them,
/// and nothing else is.
#[test]
fn every_mode_option_is_found_in_the_prefix() {
    for option in MODE_OPTIONS {
        let prefix: Vec<OsString> = vec![OsString::from("-C"), OsString::from("."), OsString::from(option)];
        assert_eq!(modes_in_effect(prefix.as_slice(), &[]), vec![String::from(option)], "{option}");
    }
    assert!(modes_in_effect(&[OsString::from("-C"), OsString::from(".")], &[]).is_empty());
}

/// A variable in the child overlay is a mode,
/// whatever its value.
#[test]
fn a_variable_in_the_overlay_is_a_mode() {
    let overlay: Vec<(OsString, OsString)> = vec![(OsString::from("GIT_GLOB_PATHSPECS"), OsString::from("0"))];
    assert_eq!(modes_in_effect(&[], overlay.as_slice()), vec![String::from("GIT_GLOB_PATHSPECS")]);
}
