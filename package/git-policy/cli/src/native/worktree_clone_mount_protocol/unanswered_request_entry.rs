//! What:
//!  One line of the folder of unanswered requests:
//!  the clone's GUID,
//!  the request's nonce and operation,
//!  when the entry was written,
//!  the attempt's outcome,
//!  and the clone's dataset,
//!  read and written as the one text the folder holds.
//! Why:
//!  A wait writes its entry before the durable intent,
//!  so a wait killed before its deadline leaves a trigger behind,
//!  and the entry is what a later command reads to learn that a clone
//!  still waits
//!  (`doc/planning/cli-git-zfs-lost-last-event-triggers-design.md`,
//!  "The record:
//! one file per clone").
//!  Every field is checked and bounded before another is read,
//!  so a half-written or foreign file is refused rather than believed,
//!  and the outcome word is what tells a wait nothing has run from an
//!  attempt the helper recorded.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const text = entryText({ guid, nonce, operation, writtenAt, outcome: 'waiting', dataset });
//! ```

use crate::foundation::iso_time::instant_of_iso;
use crate::worktree_clone_mount_protocol::clone_mount_intent::MountOperation;
use crate::worktree_clone_mount_protocol::clone_mount_property_names::VALUE_VERSION;
use crate::worktree_clone_mount_protocol::clone_mount_receipt::MAXIMUM_DATASET_NAME_BYTES;
use crate::worktree_clone_mount_protocol::dataset_guid::{
    DatasetGuid,
    DatasetGuidRefusal,
};
use crate::worktree_clone_mount_protocol::request_nonce::{
    NonceRefusal,
    RequestNonce,
};

/// The most bytes one entry of the folder may take,
/// which is one page:
/// the bound the start-time gate gives each small file it reads.
/// An entry's fields are bounded,
/// so a page holds every entry this route writes,
/// and a longer file is one a reader refuses as corrupt.
pub(crate) const MAXIMUM_ENTRY_BYTES: usize = 4096;

/// The fields one entry's line holds,
/// which is the version and the six below it.
const ENTRY_FIELDS: usize = 7;

/// What:
///  What one attempt of one clone stands in.
/// Why:
///  A wait's entry is `waiting` before any writer claims it,
///  `claimed` once a command or a wait took the attempt,
///  `nudged` once the helper wrote the property,
///  and the three refusals and the gone clone are recorded once instead of
///  being printed by every command
///  (`doc/planning/cli-git-zfs-lost-last-event-triggers-design.md`,
///  "The record:
/// one file per clone").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Outcome = 'waiting' | 'claimed' | 'nudged' | 'refused-gate' | 'refused-device' | 'clone-gone';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RequestOutcome {
    /// The wait wrote the entry and no writer has claimed it.
    Waiting,
    /// A writer took the attempt and the helper has recorded nothing yet.
    Claimed,
    /// The helper wrote the clone's sweep nonce.
    Nudged,
    /// The compatibility gate refused,
    /// so no request was made.
    RefusedGate,
    /// The clone's ZFS device could not be opened.
    RefusedDevice,
    /// The write found no dataset of the clone's name.
    CloneGone,
}

impl RequestOutcome {
    /// The word the entry carries.
    pub(crate) fn word(self) -> &'static str {
        return match self {
            RequestOutcome::Waiting => "waiting",
            RequestOutcome::Claimed => "claimed",
            RequestOutcome::Nudged => "nudged",
            RequestOutcome::RefusedGate => "refused-gate",
            RequestOutcome::RefusedDevice => "refused-device",
            RequestOutcome::CloneGone => "clone-gone",
        };
    }

    /// Read one outcome word,
    /// or nothing when it is not one this route writes.
    pub(crate) fn from_word(word: &str) -> Option<Self> {
        return match word {
            "waiting" => Some(RequestOutcome::Waiting),
            "claimed" => Some(RequestOutcome::Claimed),
            "nudged" => Some(RequestOutcome::Nudged),
            "refused-gate" => Some(RequestOutcome::RefusedGate),
            "refused-device" => Some(RequestOutcome::RefusedDevice),
            "clone-gone" => Some(RequestOutcome::CloneGone),
            _ => None,
        };
    }
}

/// What:
///  One entry of the folder:
///  the request it stands for and what has become of its attempt.
/// Why:
///   The GUID names the receipt that answers it and the attempts beside it,
///  so one value carries every field a reader compares.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Entry = { guid; nonce; operation; writtenAt; outcome; dataset };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RequestEntry {
    /// The clone's canonical numeric dataset GUID.
    pub(crate) guid: DatasetGuid,
    /// The nonce the request was filed under.
    pub(crate) nonce: RequestNonce,
    /// What the request asked for.
    pub(crate) operation: MountOperation,
    /// When this attempt's entry was written,
    /// as an ISO 8601 stamp with its offset.
    pub(crate) written_at: String,
    /// What this attempt stands in.
    pub(crate) outcome: RequestOutcome,
    /// The clone's dataset name.
    pub(crate) dataset: String,
}

impl RequestEntry {
    /// The line this entry is written as,
    /// ended by a newline:
    /// `v1 <guid> <nonce> <operation> <written-at> <outcome> <dataset>`.
    pub(crate) fn text(&self) -> String {
        return format!(
            "{VALUE_VERSION} {} {} {} {} {} {}\n",
            self.guid.file_name(),
            self.nonce.as_str(),
            self.operation.word(),
            self.written_at,
            self.outcome.word(),
            self.dataset
        );
    }
}

/// What:
///  Why a file in the folder is not one entry this route wrote.
/// Why:
///   Each cause names the field and the bound,
///  so a person reads which byte is wrong rather than only that a file was
///  refused,
///  and none of them is read as a request.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type EntryRefusal = { kind: 'too-long' | 'fields' | 'version' | 'guid' | 'nonce' | 'operation' | 'time' | 'outcome' | 'dataset'; ... };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum EntryRefusal {
    /// The file is larger than one entry may be.
    TooLong {
        /// How many bytes it holds.
        found: usize,
    },
    /// The line does not hold the seven fields.
    Fields {
        /// How many fields were read.
        found: usize,
    },
    /// The version word is not this route's.
    Version {
        /// The word as read.
        found: String,
    },
    /// The GUID field is not a canonical numeric dataset GUID.
    Guid(DatasetGuidRefusal),
    /// The nonce field is not 32 lowercase hexadecimal digits.
    Nonce(NonceRefusal),
    /// The operation field is neither `mount` nor `unmount`.
    Operation {
        /// The word as read.
        found: String,
    },
    /// The written-at field is not an ISO 8601 stamp.
    Time {
        /// The text as read.
        found: String,
    },
    /// The outcome field is not a word this route writes.
    Outcome {
        /// The word as read.
        found: String,
    },
    /// The dataset field is empty or longer than a dataset name may be.
    Dataset {
        /// How many bytes it holds.
        found: usize,
    },
}

impl EntryRefusal {
    /// The refusal in words,
    /// naming the field and the way forward.
    pub(crate) fn message(&self) -> String {
        let what: String = match self {
            EntryRefusal::TooLong {
                found,
            } => format!(
                "the file holds {found} bytes, and one entry may take at most {MAXIMUM_ENTRY_BYTES}, which is one page; every field of an entry is bounded, so a longer file is a corrupt one"
            ),
            EntryRefusal::Fields {
                found,
            } => format!(
                "the line holds {found} fields, and an entry is `{VALUE_VERSION} <guid> <nonce> <operation> <written-at> <outcome> <dataset>`"
            ),
            EntryRefusal::Version {
                found,
            } => format!("the version word is `{found}`, and this route writes `{VALUE_VERSION}`"),
            EntryRefusal::Guid(refusal) => refusal.message(),
            EntryRefusal::Nonce(refusal) => refusal.message(),
            EntryRefusal::Operation {
                found,
            } => format!(
                "`{found}` is not an operation word: a request asks for `mount` or `unmount`"
            ),
            EntryRefusal::Time {
                found,
            } => format!("`{found}` is not an ISO 8601 instant with its offset"),
            EntryRefusal::Outcome {
                found,
            } => format!(
                "`{found}` is not an outcome word: one attempt stands in `waiting`, `claimed`, `nudged`, `refused-gate`, `refused-device` or `clone-gone`"
            ),
            EntryRefusal::Dataset {
                found,
            } => format!(
                "the dataset field holds {found} bytes, and a dataset name holds between 1 and {MAXIMUM_DATASET_NAME_BYTES}"
            ),
        };
        return format!(
            "{what}. Nothing was read as an unanswered request, so no trigger was written on this file's behalf."
        );
    }
}

/// Read one file's bytes as an entry of the folder.
///
/// # Errors
/// The file is longer than one page,
/// one of its fields is missing,
/// or one field is not a value this route writes.
pub(crate) fn parse_entry(text: &str) -> Result<RequestEntry, EntryRefusal> {
    if text.len() > MAXIMUM_ENTRY_BYTES {
        return Err(EntryRefusal::TooLong {
            found: text.len(),
        });
    }
    // What: one trailing newline is the writer's, and any second line is refused.
    // Why:  a file that holds two lines is two entries in one file, which the folder's one-name-per-attempt
    //       grammar cannot answer, so it is read as corrupt rather than as its first line.
    let line: &str = text.strip_suffix('\n').unwrap_or(text);
    if line.contains('\n') {
        return Err(EntryRefusal::Fields {
            found: line.lines().count() + 1,
        });
    }
    let fields: Vec<&str> = line.splitn(ENTRY_FIELDS, ' ').collect();
    if fields.len() != ENTRY_FIELDS {
        return Err(EntryRefusal::Fields {
            found: fields.len(),
        });
    }
    let version: &str = fields[0];
    if version != VALUE_VERSION {
        return Err(EntryRefusal::Version {
            found: String::from(version),
        });
    }
    let guid: DatasetGuid = DatasetGuid::parse(fields[1]).map_err(EntryRefusal::Guid)?;
    let nonce: RequestNonce = RequestNonce::parse(fields[2]).map_err(EntryRefusal::Nonce)?;
    let operation: MountOperation = MountOperation::from_word(fields[3]).ok_or_else(|| {
        return EntryRefusal::Operation {
            found: String::from(fields[3]),
        };
    })?;
    let written_at: &str = fields[4];
    if instant_of_iso(written_at).is_none() {
        return Err(EntryRefusal::Time {
            found: String::from(written_at),
        });
    }
    let outcome: RequestOutcome = RequestOutcome::from_word(fields[5]).ok_or_else(|| {
        return EntryRefusal::Outcome {
            found: String::from(fields[5]),
        };
    })?;
    let dataset: &str = fields[6];
    if dataset.is_empty() || dataset.len() > MAXIMUM_DATASET_NAME_BYTES {
        return Err(EntryRefusal::Dataset {
            found: dataset.len(),
        });
    }
    return Ok(RequestEntry {
        guid,
        nonce,
        operation,
        written_at: String::from(written_at),
        outcome,
        dataset: String::from(dataset),
    });
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "unanswered_request_entry_tests.rs"]
mod tests;
