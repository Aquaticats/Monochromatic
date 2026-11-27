//! What:
//!  One refusal and the words it is built from:
//!  the cause,
//!  which names what is wrong with which path,
//!  and the remedy,
//!  which names every way forward.
//! Why:
//!  A person never has to guess,
//!  and a caller that words its own refusal reads the same two fields.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Refusal = { cause: string; remedy: string };
//! ```

/// What:
///  One refusal.
/// Why:
///   The cause says what is wrong with which path;
/// the remedy lists every way forward,
/// so the person never has to guess.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Refusal = { cause: string; remedy: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Refusal {
    /// What is wrong.
    pub cause: String,
    /// What can be done about it.
    pub remedy: String,
}

/// Shorthand for building a refusal from two texts.
pub(super) fn refuse(
    cause: String,
    remedy: &str,
) -> Refusal {
    return Refusal {
        cause,
        remedy: remedy.to_string(),
    };
}
