//! What:
//!  Why bytes are not an origin cleanup record,
//!  and the sentence a person reads about each.
//! Why:
//!  The design refuses a record that does not parse,
//!  changes nothing and names the file,
//!  and waits for a person
//!  (`doc/decision/cli-git-zfs-clone-worktree-records-seven-answers.md`,
//!  "A record that cannot be read"),
//!  so one type carries the cause and the words rather than a string built at the throw site.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! try { parse(bytes) } catch (refusal) { console.error(refusal.message(path)); }
//! ```

/// What:
///  Which rule the record's bytes broke.
/// Why:
///  A person needs the key or the line,
///  and a control asserts the rule rather than the sentence,
///  so the cause is its own value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Cause = 'unreadable' | 'too-large' | 'ends-early' | ...;
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CleanupRefusalCause {
    /// The file could not be read at all.
    Unreadable,
    /// The name is no regular file,
    /// which includes a link and a folder.
    NotARegularFile,
    /// The bytes are longer than the record's bound.
    Oversized,
    /// The bytes stop before a line the record declares.
    EndsEarly,
    /// A key appears twice.
    RepeatedKey,
    /// A key the grammar does not hold.
    UnknownKey,
    /// The keys are not in the grammar's order.
    OutOfOrder,
    /// A key the state requires is absent.
    MissingKey,
    /// A key the state forbids is present.
    UnexpectedKey,
    /// A value is not the spelling its kind allows.
    NotCanonical,
    /// A value is wider than its key's bound.
    FieldTooWide,
    /// The record's folder or file is not this account's,
    /// or its mode lets another account write it.
    NotThisAccounts,
}

impl CleanupRefusalCause {
    /// The word a control and `doctor` name.
    pub fn name(self) -> &'static str {
        return match self {
            CleanupRefusalCause::Unreadable => "unreadable",
            CleanupRefusalCause::NotARegularFile => "not-a-regular-file",
            CleanupRefusalCause::Oversized => "oversized",
            CleanupRefusalCause::EndsEarly => "ends-early",
            CleanupRefusalCause::RepeatedKey => "repeated-key",
            CleanupRefusalCause::UnknownKey => "unknown-key",
            CleanupRefusalCause::OutOfOrder => "out-of-order",
            CleanupRefusalCause::MissingKey => "missing-key",
            CleanupRefusalCause::UnexpectedKey => "unexpected-key",
            CleanupRefusalCause::NotCanonical => "not-canonical",
            CleanupRefusalCause::FieldTooWide => "field-too-wide",
            CleanupRefusalCause::NotThisAccounts => "not-this-accounts",
        };
    }
}

/// What:
///  One record that refused,
///  with the rule,
///  the key and the value a person needs.
/// Why:
///  The message names the file and the record's own field,
///  and never prints the whole record,
///  which may hold another slot's path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// refusal.message(path); // "the origin cleanup record at … is not one this reader accepts (…)"
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CleanupRefusal {
    /// Which rule refused.
    pub cause: CleanupRefusalCause,
    /// The key or region the rule names.
    pub field: String,
    /// What the bytes held there.
    pub said: String,
}

impl CleanupRefusal {
    /// One refusal of one field.
    pub fn at(cause: CleanupRefusalCause, field: &str, said: String) -> CleanupRefusal {
        return CleanupRefusal { cause, field: String::from(field), said };
    }

    /// One refusal that names no field.
    pub fn of(cause: CleanupRefusalCause, said: String) -> CleanupRefusal {
        return CleanupRefusal { cause, field: String::from("record"), said };
    }

    /// The sentence a person reads,
    /// with the file it was found in.
    pub fn message(&self, path: &std::path::Path) -> String {
        return format!(
            "the origin cleanup record at {} is not one this reader accepts ({} on `{}`: {}). Nothing was destroyed, the record is unchanged, and the origin snapshot stays until a person acts. Ways forward: read the record, destroy the snapshot it names by hand if you can, and remove the file once it is gone; the wrapper never rewrites a record it cannot read.",
            path.display(),
            self.cause.name(),
            self.field,
            self.said
        );
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "origin_cleanup_refusal_tests.rs"]
mod tests;
