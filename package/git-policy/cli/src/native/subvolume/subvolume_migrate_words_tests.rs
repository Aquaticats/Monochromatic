//! Controls for the line that names a process standing in the tree a migration replaces.
//! Why:
//!  The line is the person's pointer to what must be restarted before the old tree goes,
//!  and its name is a name another program chose (`/proc/<pid>/comm`),
//!  so a name that would end the parentheses around it,
//!  or that would move the cursor or start a line,
//!  must come back spelled out.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(aProcessLeftInTheOldTreeLine(42, 'x) FORGED\u001b[2K')).toBe('  pid 42 (x\\) FORGED\\u{1b}[2K)');
//! ```

use super::*;

/// A name that would end the parentheses it is shown in,
/// and one that would start a line,
/// both come back spelled out,
/// so the list of what stands in the old tree says only what the process table showed.
#[test]
fn a_name_that_would_end_the_parentheses_cannot_rewrite_the_line() {
    assert_eq!(
        a_process_left_in_the_old_tree_line(42, Some("x) FORGED\u{1b}[2K")),
        "  pid 42 (x\\) FORGED\\u{1b}[2K)"
    );
    assert_eq!(
        a_process_left_in_the_old_tree_line(7, Some("\nmonochromatic-git-policy-cli: x")),
        "  pid 7 (\\nmonochromatic-git-policy-cli: x)"
    );
}

/// A name that could not be read is said in this program's own words,
/// and a backtick inside the name is not this sentence's wrapping,
/// so it is shown as the program spelled it.
#[test]
fn an_unreadable_name_and_a_backtick_are_shown_as_they_are() {
    assert_eq!(
        a_process_left_in_the_old_tree_line(42, None),
        "  pid 42 (name unreadable)"
    );
    assert_eq!(
        a_process_left_in_the_old_tree_line(42, Some("a`b")),
        "  pid 42 (a`b)"
    );
}
