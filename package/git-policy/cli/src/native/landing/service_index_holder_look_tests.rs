//! Controls for the field that names a proven lock holder in the service's answer.
//! Why:
//!  The field is read by the service and by the line it writes,
//!  and its name is a name another program chose (`/proc/<pid>/comm`),
//!  so a name that would end the parentheses around it,
//!  or that would move the cursor or start a line,
//!  must come back spelled out.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(holderWords(42, 'x) FORGED\u001b[2K')).toBe('Process 42 (x\\) FORGED\\u{1b}[2K)');
//! ```

use super::*;

/// A name that would end the parentheses it is shown in,
/// and one that would start a line,
/// both come back spelled out,
/// so the field says which process holds the lock and nothing else.
#[test]
fn a_name_that_would_end_the_parentheses_cannot_rewrite_the_holders_name() {
    assert_eq!(
        holder_words(42, Some("x) FORGED\u{1b}[2K")),
        "Process 42 (x\\) FORGED\\u{1b}[2K)"
    );
    assert_eq!(
        holder_words(7, Some("\nmonochromatic-git-policy-cli: x")),
        "Process 7 (\\nmonochromatic-git-policy-cli: x)"
    );
}

/// A holder with no readable name is named by its number alone,
/// and an ordinary name is shown unchanged.
#[test]
fn a_holder_without_a_name_is_named_by_its_number() {
    assert_eq!(holder_words(42, None), "Process 42");
    assert_eq!(holder_words(42, Some("git-lfs")), "Process 42 (git-lfs)");
}
