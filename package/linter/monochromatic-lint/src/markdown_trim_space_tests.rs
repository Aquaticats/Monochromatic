//! What:
//!  Controls for the trim whitespace set.
//! Why:
//!  The byte order mark and NEXT LINE are the two places the set differs from Rust's `White_Space`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Trim with the set and compare the exact remainder.
//! ```

/// Import the production membership test rather than a copied list.
use super::trim_space;

/// The exact incumbent trim behavior differs from Rust for BOM and NEXT LINE.
#[test]
fn padding_keeps_the_incumbent_unicode_boundaries() {
    assert_eq!("\u{feff}\u{00a0}🚀\u{feff}".trim_matches(trim_space), "🚀");
    assert_eq!("\u{0085}".trim_matches(trim_space), "\u{0085}");
    assert_eq!("".trim_matches(trim_space), "");
}
