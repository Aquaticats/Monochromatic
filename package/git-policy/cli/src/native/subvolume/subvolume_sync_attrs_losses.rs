//! What:
//!  What a copy of one entry could not carry:
//!  how many owner,
//!  extended-attribute,
//!  permission-bit and time changes an unprivileged user must expect to be refused,
//!  and the one refusal the copy cannot tolerate.
//! Why:
//!  A refusal the operating system owes this user is counted and reported,
//!  never silent,
//!  and a refusal for any other reason is an error that names what was being set,
//!  so a copy is never recorded as faithful when it is not.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type AttributeLosses = { owner: number; extendedAttributes: number; mode: number; times: number };
//! ```

/// What:
///  How many attribute operations could not be applied,
/// by kind.
/// Why:
///   The report says so,
/// so a person knows what the copy does not carry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AttributeLosses = { owner: number; extendedAttributes: number; mode: number; times: number };
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AttributeLosses {
    /// Entries whose owner or group could not be set.
    pub owner: u64,
    /// Extended attributes that could not be copied.
    pub extended_attributes: u64,
    /// Entries whose permission bits the operating system refused to change for lack of ownership.
    pub mode: u64,
    /// Entries whose times the operating system refused to change for lack of ownership.
    pub times: u64,
}

/// What:
///  An attribute operation that failed in a way the copy cannot tolerate.
/// Why:
///   The caller names the path and turns it into the migration's error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AttributeFailure = { action: string; error: Error };
/// ```
#[derive(Debug)]
pub struct AttributeFailure {
    /// What was being set,
    /// in words for `cannot <action> <path>`.
    pub action: &'static str,
    /// The operating system's refusal.
    pub error: std::io::Error,
}
