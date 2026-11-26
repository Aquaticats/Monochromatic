//! What:
//!  A dataset's canonical numeric GUID:
//!  the only name a clone-mount receipt is filed under,
//!  and the identity a receipt,
//! a request and a wait agree on.
//! Why:
//!  The design says to publish receipts under the canonical numeric dataset
//!  GUID and to put the nonce only in the file's data
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Requests,
//! results and ordering",
//!  "Decided defaults").
//!  OpenZFS generates a random 64-bit GUID for every dataset
//!  (`module/zfs/dsl_dataset.c:1203` to `:1204`,
//!  `:1768` to `:1769`,
//!  OpenZFS `zfs-2.4.4`),
//!  so the decimal form is a name no caller's text can steer.
//!  Its canonical spelling takes no sign and no leading zero,
//!  so one GUID has exactly one file name.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const guid = DatasetGuid.parse('1234567890123456789'); guid.fileName();
//! ```

/// What:
///  Why a value is not a dataset GUID as this route names one.
/// Why:
///   The file name carries the GUID,
///  so a value with a sign,
///  a leading zero or a digit beyond `u64` would make two names for one
///  dataset or a name that is no dataset.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DatasetGuidRefusal = { kind: 'not-digits' | 'leading-zero' | 'too-large'; value: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DatasetGuidRefusal {
    /// The value is empty or holds a byte that is not a decimal digit.
    NotDigits {
        /// The value as read.
        value: String,
    },
    /// The value begins with a zero and names no canonical number.
    LeadingZero {
        /// The value as read.
        value: String,
    },
    /// The value is larger than a dataset GUID can be.
    TooLarge {
        /// The value as read.
        value: String,
    },
}

impl DatasetGuidRefusal {
    /// The refusal in words,
    /// naming the value and the way forward.
    pub fn message(&self) -> String {
        return match self {
            DatasetGuidRefusal::NotDigits {
                value,
            } => format!(
                "`{value}` is not a dataset GUID: a GUID here is one or more decimal digits, with no sign, space or prefix. Nothing was filed, read or matched under it. Ways forward: read the number as `zfs get -H -o value guid <dataset>` prints it."
            ),
            DatasetGuidRefusal::LeadingZero {
                value,
            } => format!(
                "`{value}` begins with a zero, and a canonical dataset GUID has no leading zero, so two spellings would name one receipt. Nothing was filed, read or matched under it. Ways forward: write the number without leading zeros."
            ),
            DatasetGuidRefusal::TooLarge {
                value,
            } => format!(
                "`{value}` is larger than the largest dataset GUID this kernel can report ({}). Nothing was filed, read or matched under it. Ways forward: read the GUID again with `zfs get -H -o value guid <dataset>`.",
                u64::MAX
            ),
        };
    }
}

/// What:
///  One canonical dataset GUID.
/// Why:
///   A value of this type can only have come through the parse,
///  so a file name,
///  a receipt field and a request's match all compare the same number.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class DatasetGuid { private constructor(readonly value: bigint) {} }
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DatasetGuid {
    /// The number the kernel reported.
    value: u64,
}

impl DatasetGuid {
    /// Read a value as a canonical numeric GUID.
    ///
    /// # Errors
    /// The value is empty,
    /// holds a non-digit,
    /// begins with a zero or does not fit 64 bits.
    pub fn parse(text: &str) -> Result<Self, DatasetGuidRefusal> {
        if text.is_empty() || !text.bytes().all(|byte| return byte.is_ascii_digit()) {
            return Err(DatasetGuidRefusal::NotDigits {
                value: String::from(text),
            });
        }
        if text.len() > 1 && text.starts_with('0') {
            return Err(DatasetGuidRefusal::LeadingZero {
                value: String::from(text),
            });
        }
        let Ok(value) = text.parse::<u64>() else {
            return Err(DatasetGuidRefusal::TooLarge {
                value: String::from(text),
            });
        };
        return Ok(DatasetGuid {
            value,
        });
    }

    /// The number itself.
    pub fn value(self) -> u64 {
        return self.value;
    }

    /// The canonical spelling,
    /// which is also the receipt's file name.
    pub fn file_name(self) -> String {
        return self.value.to_string();
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "dataset_guid_tests.rs"]
mod tests;
