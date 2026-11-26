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
pub const ERRNO_NAMES: &[(&str, i32)] = &[
    ("EACCES", libc::EACCES),
    ("EBUSY", libc::EBUSY),
    ("EEXIST", libc::EEXIST),
    ("EFAULT", libc::EFAULT),
    ("EINVAL", libc::EINVAL),
    ("EIO", libc::EIO),
    ("ELOOP", libc::ELOOP),
    ("EMFILE", libc::EMFILE),
    ("ENAMETOOLONG", libc::ENAMETOOLONG),
    ("ENODEV", libc::ENODEV),
    ("ENFILE", libc::ENFILE),
    ("ENOENT", libc::ENOENT),
    ("ENOMEM", libc::ENOMEM),
    ("ENOSPC", libc::ENOSPC),
    ("ENOSYS", libc::ENOSYS),
    ("ENOTDIR", libc::ENOTDIR),
    ("ENXIO", libc::ENXIO),
    ("EOPNOTSUPP", libc::EOPNOTSUPP),
    ("EPERM", libc::EPERM),
    ("EROFS", libc::EROFS),
    ("EXDEV", libc::EXDEV),
];

/// What:
///  One status line's errno field:
///  nothing where the refusal was a comparison,
///  a name where this build knows the number,
///  or the number itself where it does not.
/// Why:
///  A number this build names nowhere must never read as `none`,
///  because the record reserves `none` for a comparison or a policy check,
///  and a reader would lose the kernel's answer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ErrnoWord {
    /// The refusal was a comparison or a policy check.
    None,
    /// The number has one of this build's names.
    Named(&'static str),
    /// The number has no name in this build,
    /// so it stays the kernel's number.
    Number(i32),
}

impl ErrnoWord {
    /// The field one optional name becomes.
    pub fn of_name(name: Option<&'static str>) -> Self {
        return match name {
            Some(word) => ErrnoWord::Named(word),
            None => ErrnoWord::None,
        };
    }

    /// The field one optional number becomes:
    /// its name where this build has one,
    /// its number where it does not,
    /// and nothing where the call answered none.
    pub fn of_number(number: Option<i32>) -> Self {
        return match number {
            Some(value) => match errno_of(value) {
                Some(name) => ErrnoWord::Named(name),
                None => ErrnoWord::Number(value),
            },
            None => ErrnoWord::None,
        };
    }

    /// The field's text,
    /// which is `none`,
    /// a name or the number.
    pub fn text(self) -> String {
        return match self {
            ErrnoWord::None => String::from("none"),
            ErrnoWord::Named(name) => String::from(name),
            ErrnoWord::Number(number) => number.to_string(),
        };
    }

    /// The name this field carries,
    /// where it carries one.
    pub fn named(self) -> Option<&'static str> {
        return match self {
            ErrnoWord::Named(name) => Some(name),
            ErrnoWord::None | ErrnoWord::Number(_) => None,
        };
    }

    /// The field one text becomes,
    /// or nothing where the text is neither `none`,
    /// a name nor a number.
    pub fn parse(word: &str) -> Option<Self> {
        if word == "none" {
            return Some(ErrnoWord::None);
        }
        if let Some(name) = errno_name(word) {
            return Some(ErrnoWord::Named(name));
        }
        return match word.parse::<i32>() {
            Ok(number) if number > 0 => Some(ErrnoWord::Number(number)),
            _ => None,
        };
    }
}

/// The errno names a status line may carry,
/// one per word.
pub fn errno_name(word: &str) -> Option<&'static str> {
    return ERRNO_NAMES
        .iter()
        .find(|(name, _)| return *name == word)
        .map(|(name, _)| return *name);
}

/// The errno name one call's answer stands for,
/// or nothing when this build does not name it.
pub fn errno_of(number: i32) -> Option<&'static str> {
    return ERRNO_NAMES
        .iter()
        .find(|(_, value)| return *value == number)
        .map(|(name, _)| return *name);
}

/// The reason words a step that did not run may carry.
pub const REASON_WORDS: &[&str] = &[
    "earlier-arm-failed",
    "source-held",
    "exclusive-create-unsupported",
    "no-source-name",
    "no-checkout-path",
    "configuration-missing",
    "configuration-unreadable",
    "configuration-not-under-root-control",
    "configuration-unparsable",
    "name-absent",
    "name-not-text",
    "name-refused",
    "missing-source",
    "mount-table-unreadable",
    "no-mounted-checkout",
    "several-mounted-checkouts",
    "configuration-empty",
];

/// Whether a reason word is one this build writes.
pub fn known_reason_word(word: &str) -> Option<&'static str> {
    return REASON_WORDS
        .iter()
        .copied()
        .find(|known| return *known == word);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "probe_errno_names_tests.rs"]
mod tests;
