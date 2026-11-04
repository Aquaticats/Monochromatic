//! What:
//!  The pathspec modes a command runs under and the sentences that name them.
//! Why:
//!  A sentence that names the wrong mode,
//! or a mode that is missed,
//! tells a person to change something that does not matter.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(modesInEffect(['--icase-pathspecs'], [])).toEqual(['--icase-pathspecs']);
//! ```

use super::{MODE_OPTIONS, magic_note, mode_note, modes_in_effect};
use crate::conflict_side_state::Operation;
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

/// Each note names what to change,
/// the operation,
/// and that the line is then saved first and then allowed.
#[test]
fn the_notes_name_the_mode_or_the_magic_and_the_way_out() {
    let mode: String = mode_note(&[String::from("--icase-pathspecs")], Operation::Rebase);
    assert!(mode.contains("This rebase is in progress") && mode.contains("the same command without `--icase-pathspecs`"), "{mode}");
    assert!(mode.contains("saved first and then allowed"), "{mode}");
    let magic: String = magic_note(&[(String::from(":(icase)"), String::from("sub/a.rs"))], Operation::Revert);
    assert!(magic.contains("This revert is in progress") && magic.contains("`:(icase)`") && magic.contains("`sub/a.rs`"), "{magic}");
    assert!(magic.contains("saved first and then allowed"), "{magic}");
}
