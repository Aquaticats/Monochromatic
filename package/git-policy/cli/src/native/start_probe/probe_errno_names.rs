//! What:
//!  The closed sets a status line's errno and reason words come from.
//! Why:
//!  The record names an errno rather than a number,
//!  and a word this build does not know is a line it cannot read,
//!  so the names are one table each and the reverse lookup is the same data.
//!  A number the table omits stays unnamed rather than being given an
//!  invented word.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const errnoName = (n) => ERRNO_NAMES.find(([, v]) => v === n)?.[0];
//! ```

/// The errno names a status line may carry,
/// with the values they stand for.
pub const ERRNO_NAMES: [(&str, i32); 14] = [
    ("EACCES", libc::EACCES),
    ("EBUSY", libc::EBUSY),
    ("EEXIST", libc::EEXIST),
    ("EINVAL", libc::EINVAL),
    ("ELOOP", libc::ELOOP),
    ("EMFILE", libc::EMFILE),
    ("ENFILE", libc::ENFILE),
    ("ENOENT", libc::ENOENT),
    ("ENOMEM", libc::ENOMEM),
    ("ENOSPC", libc::ENOSPC),
    ("EOPNOTSUPP", libc::EOPNOTSUPP),
    ("EPERM", libc::EPERM),
    ("EROFS", libc::EROFS),
    ("EXDEV", libc::EXDEV),
];

/// The errno names a status line may carry,
/// one per word.
pub fn errno_name(word: &str) -> Option<&'static str> {
    return ERRNO_NAMES.iter().find(|(name, _)| return *name == word).map(|(name, _)| return *name);
}

/// The errno name one call's answer stands for,
/// or nothing when this build does not name it.
pub fn errno_of(number: i32) -> Option<&'static str> {
    return ERRNO_NAMES.iter().find(|(_, value)| return *value == number).map(|(name, _)| return *name);
}

/// The reason words a step that did not run may carry.
pub const REASON_WORDS: [&str; 12] = [
    "earlier-arm-failed",
    "source-held",
    "exclusive-create-unsupported",
    "no-source-name",
    "no-checkout-path",
    "configuration-missing",
    "configuration-unreadable",
    "configuration-unparsable",
    "name-absent",
    "name-not-text",
    "name-refused",
    "missing-source",
];

/// Whether a reason word is one this build writes.
pub fn known_reason_word(word: &str) -> Option<&'static str> {
    return REASON_WORDS.iter().copied().find(|known| return *known == word);
}


/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "probe_errno_names_tests.rs"]
mod tests;
