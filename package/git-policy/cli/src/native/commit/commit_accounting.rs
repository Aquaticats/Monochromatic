//! What:
//!  The two figures that say whether a commit's timing record can answer a question about a slow commit:
//!  the time its named phases cover,
//!  and the time no phase covers.
//! Why:
//!   The record exists to split a slow commit by phase,
//!  and a remainder that is not small says it cannot;
//!  a control reading a real commit's record states a tolerance and fails when the remainder passes it
//!  (`doc/planning/cli-git-commit-latency-alpha-11-against-alpha-10.md`).
//!  The type lives beside the record it belongs to,
//! in a file of its own,
//!  because the record's own file is at the length this package allows.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Accounting = { named: number; unnamed: number };
//! ```

use std::time::Duration;

/// What:
///  How much of a command's total the record's phases name,
///  and how much they leave unnamed.
/// Why:
///   A reader of a slow commit asks which phase spent the time;
///  a remainder that is not small says the record cannot answer,
///  and those are the two figures that say so.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Accounting = { named: number; unnamed: number };
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Accounting {
    /// Time the phases' names cover.
    pub named: Duration,
    /// Time no phase covers.
    pub unnamed: Duration,
}

impl Accounting {
    /// What:
    ///  Whether the record accounts for its own total within the tolerance a control states.
    /// Why:
    ///  The tolerance belongs to the control,
    ///  because only the control knows what its own fixture's remaining file work costs.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// accounting.within(TOLERANCE)
    /// ```
    pub fn within(&self, tolerance: Duration) -> bool {
        return self.unnamed <= tolerance;
    }
}
