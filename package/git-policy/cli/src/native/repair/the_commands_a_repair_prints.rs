//! What:
//!  The commands a repair prints for a person to run by hand,
//!  in the order the repair's own words name them.
//! Why:
//!  A repair that cannot finish says what the person runs to finish it,
//!  and the words that say so are printed by more than one file:
//!  the toolchain repair and its install report the command in their findings,
//!  and the relink notice prints the same command for a relink given up on.
//!  The command therefore lives in a file of its own that both name,
//!  rather than in either one,
//!  so neither file uses the other to say one word.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export const MANUAL_COMMAND = 'mise run toolchain:install';
//! ```

/// What:
///  `pub const MANUAL_COMMAND: &str` is a name bound to a borrowed string
///  literal: `&str` is a read-only view of UTF-8 bytes baked into the
///  binary, and its sibling `String` would be an owned buffer on the heap.
/// Why:
///  The command is a fixed word this release prints,
///  so nothing here needs to grow or be freed,
///  and every finder and notice that names it copies the same constant.
/// Gotcha:
///  `&str` is not a character list: it is a byte view that must be valid
///  UTF-8, and the `'static` lifetime this literal has means it lives for
///  the whole program, which the type as written does not spell out.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MANUAL_COMMAND = "mise run toolchain:install";
/// ```
pub const MANUAL_COMMAND: &str = "mise run toolchain:install";
