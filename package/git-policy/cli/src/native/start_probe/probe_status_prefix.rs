//! What:
//!  The word every status line begins with.
//! Why:
//!  A reader tells one of these lines from another program's line by its
//!  first word,
//!  and the writer and the refusal that names a foreign line must not spell
//!  that word twice.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const STATUS_PREFIX = 'probe-step';
//! ```

/// The word every status line begins with,
/// so a line of another kind is never read as one of these.
pub const STATUS_PREFIX: &str = "probe-step";
