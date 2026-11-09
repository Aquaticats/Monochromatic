//! What:
//! Unit tests of the invocation-name reader in `invoked_name.rs`.
//! Why:
//! This name picks between shadow mode and the command-line tool,
//! so every argument list the operating system can hand over needs an answer,
//! including the empty one an `execve` caller may pass.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // invoked-name.unit.test.ts
//! describe('invokedName', () => { ... });
//! ```

/// Imports the reader from this crate.
use crate::invoked_name::invoked_name;

/// Imports the owned operating-system text type the reader takes.
use std::ffi::OsString;

/// What:
/// `fn texts(items: &[&str]) -> Vec<OsString>` declares a helper turning a borrowed list of text slices into the
/// owned list the reader takes.
/// Why:
/// Each test spells its argument list as plain text and the helper does the conversion once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const texts = (items: readonly string[]): string[] => [...items];
/// ```
fn texts(items: &[&str]) -> Vec<OsString> {
    // `items.iter()` walks the slices without consuming them; `.map(...)` converts each one; `.collect()` gathers
    // them into the `Vec<OsString>` the annotation names.
    return items.iter().map(OsString::from).collect();
}

/// The name is the final segment of the first argument,
/// whatever directory it names,
/// so a link called `rg` selects shadow mode.
#[test]
fn the_name_is_the_last_segment_of_the_first_argument() {
    assert_eq!(invoked_name(&texts(&["/usr/local/bin/rg"])), "rg");
    assert_eq!(invoked_name(&texts(&["search-shadow", "link"])), "search-shadow");
    assert_eq!(invoked_name(&texts(&["rg"])), "rg");
}

/// An argument list with no argument at all answers the empty name,
/// which matches no shadowed name and so starts the command-line tool.
#[test]
fn an_empty_argument_list_has_no_name() {
    assert_eq!(invoked_name(&texts(&[])), "");
}
