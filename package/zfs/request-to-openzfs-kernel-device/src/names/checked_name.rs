// Adapted from `set_name` and `set_value` in `src/ioctl.rs` of
// `Ukko-Ylijumala/zfsdev-rs` at commit 4559ffc711; see PROVENANCE.md.
// Copyright (c) 2026 Mikko Tanner, MIT OR Apache-2.0.
//! What:
//!  A name checked against the fixed field it will be copied into,
//! and the
//!  refusal that names the field,
//! the reason and the bound.
//! Why:
//!  The kernel reads these fields as NUL-terminated C strings;
//! text that is
//!  empty,
//! carries an interior NUL,
//! or does not fit would be read as a
//!  different string,
//! sometimes a *different existing dataset*.
//!  Checking once,
//! before the request is built,
//! keeps the failure next to the
//!  caller's input instead of turning it into a kernel error number.
//!  The text is owned so the request outlives the string it was built from.

/// The field a name is bound for.
use super::name_field::NameField;
/// The struct a name is copied into.
use crate::layout::ZfsCmd;
/// The formatting traits the diagnostics implement.
use std::fmt;

/// Why a name cannot be written into its field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameProblem {
    /// The text had no bytes.
    Empty,
    /// The text does not fit the field with its terminating NUL.
    TooLong {
        /// Bytes of the text.
        length: usize,
        /// Bytes the field holds,
        /// NUL included.
        maximum: usize,
    },
    /// The text carries a NUL byte,
    /// which would end the C string early.
    InteriorNul {
        /// Byte offset of the first NUL.
        at: usize,
    },
}

/// A name that a request may copy into its fixed field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckedName {
    /// The field the name is bound for.
    field: NameField,
    /// The checked text.
    text: String,
}

/// The methods of `CheckedName`.
impl CheckedName {
    /// Check `text` against the bound of `field`.
    ///
    /// # Throws
    ///
    /// [`NameError`] when the text is empty,
    /// carries a NUL byte,
    /// or does not
    /// fit the field with its terminating NUL.
    pub fn new(field: NameField, text: &str) -> Result<CheckedName, NameError> {
        let bytes: &[u8] = text.as_bytes();
        if bytes.is_empty() {
            return Err(NameError {
                field,
                problem: NameProblem::Empty,
            });
        }
        if let Some(at) = bytes.iter().position(|byte| {
            return *byte == 0;
        }) {
            return Err(NameError {
                field,
                problem: NameProblem::InteriorNul { at },
            });
        }
        if bytes.len() >= field.max_bytes() {
            return Err(NameError {
                field,
                problem: NameProblem::TooLong {
                    length: bytes.len(),
                    maximum: field.max_bytes(),
                },
            });
        }
        return Ok(CheckedName {
            field,
            text: text.to_string(),
        });
    }

    /// The field this name belongs to.
    pub fn field(&self) -> NameField {
        return self.field;
    }

    /// The checked text.
    pub fn as_str(&self) -> &str {
        return &self.text;
    }

    /// Copy the text and its terminating NUL into the command's field.
    ///
    /// # Panics
    ///
    /// Never for a value built by [`CheckedName::new`]:
    /// the constructor
    /// guarantees the text fits,
    /// and the fields are private.
    pub fn write_into(&self, command: &mut ZfsCmd) {
        let text: &[u8] = self.text.as_bytes();
        let field: &mut [u8] = match self.field {
            NameField::CommandName => &mut command.zc_name,
            NameField::CommandValue => &mut command.zc_value,
            NameField::CommandString => &mut command.zc_string,
        };
        field[..text.len()].copy_from_slice(text);
        field[text.len()] = 0;
    }
}

/// A name the kernel's field cannot carry,
/// with the field,
/// reason and bound.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NameError {
    /// The field the name was destined for.
    pub field: NameField,
    /// What was wrong with the text.
    pub problem: NameProblem,
}

/// The methods of `NameError`.
impl NameError {
    /// The error number the kernel would have answered with for this text,
    /// so
    /// a caller can report an input refusal like a kernel refusal.
    pub fn errno(&self) -> i32 {
        return match self.problem {
            NameProblem::TooLong { .. } => libc::ENAMETOOLONG,
            NameProblem::Empty | NameProblem::InteriorNul { .. } => libc::EINVAL,
        };
    }
}

/// How the value prints to a person.
impl fmt::Display for NameError {
    /// Explains which field is wrong and why.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let field: &str = self.field.as_str();
        return match self.problem {
            NameProblem::Empty => write!(formatter, "{field} may not be empty"),
            NameProblem::TooLong { length, maximum } => write!(
                formatter,
                "{field} holds at most {} bytes and the terminating NUL, but the name is {length} bytes",
                maximum - 1
            ),
            NameProblem::InteriorNul { at } => {
                write!(formatter, "{field} carries a NUL byte at offset {at}")
            }
        };
    }
}

/// The standard-error view of the refusal.
impl std::error::Error for NameError {}

/// The device-free tests of this module.
#[cfg(test)]
#[path = "checked_name_tests.rs"]
mod tests;
