//! What:
//!  The fixed `zfs_cmd_t` text field a checked name will occupy,
//! and how many
//!  bytes that field holds,
//! terminating NUL included.
//! Why:
//!  A name that is one byte too long for its field would be truncated to a
//!  *different* existing dataset instead of refused,
//!  and each of the three fields has a different bound,
//!  so the bound travels with the field rather than with the check.

/// The bounds each name field reports.
use crate::layout::{MAX_VALUE_LEN, MAXNAMELEN, MAXPATHLEN};

/// One of the fixed text fields a request writes a name into.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameField {
    /// `zc_name`:
    /// the primary pool or dataset name.
    CommandName,
    /// `zc_value`:
    /// a secondary name,
    /// for example a rename target.
    CommandValue,
    /// `zc_string`:
    /// a short secondary string,
    /// for example a clone origin.
    CommandString,
}

/// The methods of `NameField`.
impl NameField {
    /// Bytes the field holds,
    /// its terminating NUL included.
    pub fn max_bytes(self) -> usize {
        return match self {
            NameField::CommandName => MAXPATHLEN,
            NameField::CommandValue => MAX_VALUE_LEN,
            NameField::CommandString => MAXNAMELEN,
        };
    }

    /// The field's name in the C struct,
    /// for a diagnostic.
    pub fn as_str(self) -> &'static str {
        return match self {
            NameField::CommandName => "zc_name",
            NameField::CommandValue => "zc_value",
            NameField::CommandString => "zc_string",
        };
    }
}
