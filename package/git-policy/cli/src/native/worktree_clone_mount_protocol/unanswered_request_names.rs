//! What:
//!  The names the folder of unanswered requests holds:
//!  one entry per attempt,
//!  named `<guid>.<attempt>`,
//!  the staging name a writer links it from,
//!  and the rule that numbers the next attempt.
//! Why:
//!  One file per attempt is the shape the slot identity store took,
//!  so two waits ending together write two files rather than one rewrite
//!  that loses the other,
//!  and the number is what tells a claim from the attempt under it
//!  (`doc/planning/cli-git-zfs-lost-last-event-triggers-design.md`,
//!  "The record:
//! one file per clone").
//!  The attempt is counted from the names present,
//!  empty or readable,
//!  so a claim is possible after a crash left an empty file above the
//!  readable one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const name = entryName(guid, 2); // '1234567890123456789.2'
//! ```

use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_mount_protocol::request_nonce::{
    NonceRefusal,
    RequestNonce,
};

/// The digits a staging name's random token holds,
/// which is the slot identity store's own staging shape.
const STAGING_NAME_DIGITS: usize = 32;

/// The separator between the entry's GUID and its attempt.
const ATTEMPT_SEPARATOR: char = '.';

/// What:
///  Why a name in the folder is not one attempt's entry.
/// Why:
///   The folder holds the entries and every killed writer's staging name,
///  and a reader must tell them apart without reading any of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type NameRefusal = 'staging' | 'guid' | 'attempt';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NameRefusal {
    /// The name is a staging name a writer left,
    /// or a name shaped like one.
    Staging,
    /// The part before the separator is not a canonical dataset GUID.
    Guid,
    /// The part after the separator is not a canonical attempt number.
    Attempt,
}

/// What:
///  One attempt's entry:
///  the clone it belongs to and the attempt's number.
/// Why:
///  The GUID names the receipt that answers it and every attempt beside it,
///  and the number orders the attempts of one clone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type EntryName = { guid; attempt };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct EntryName {
    /// The clone the attempt belongs to.
    pub(crate) guid: DatasetGuid,
    /// Which attempt of that clone this is,
    /// counted from one.
    pub(crate) attempt: u32,
}

impl EntryName {
    /// The name this attempt's file is written under.
    pub(crate) fn file_name(self) -> String {
        return format!(
            "{}{ATTEMPT_SEPARATOR}{}",
            self.guid.file_name(),
            self.attempt
        );
    }

    /// Read one name in the folder as an attempt's entry.
    ///
    /// # Errors
    /// The name is a staging name,
    /// its GUID part is not canonical,
    /// or its attempt part is not a canonical number from one.
    pub(crate) fn parse(name: &str) -> Result<Self, NameRefusal> {
        if is_a_staging_name(name) {
            return Err(NameRefusal::Staging);
        }
        let Some((guid_part, attempt_part)) = name.split_once(ATTEMPT_SEPARATOR) else {
            return Err(NameRefusal::Attempt);
        };
        let guid: DatasetGuid = DatasetGuid::parse(guid_part).map_err(|_refusal| {
            return NameRefusal::Guid;
        })?;
        let Ok(attempt) = attempt_part.parse::<u32>() else {
            return Err(NameRefusal::Attempt);
        };
        // What: a leading zero or a zero names no attempt this route writes.
        // Why:  two spellings of one number would be two names for one attempt, and the count of
        //       attempts beside a clone decides the next attempt's number.
        if attempt == 0 || (attempt_part.len() > 1 && attempt_part.starts_with('0')) {
            return Err(NameRefusal::Attempt);
        }
        return Ok(EntryName {
            guid,
            attempt,
        });
    }
}

/// Whether one name in the folder is a staging name a writer made.
/// The shape is the slot identity store's:
/// a dot and 32 lowercase hexadecimal digits.
pub(crate) fn is_a_staging_name(name: &str) -> bool {
    let Some(token) = name.strip_prefix(ATTEMPT_SEPARATOR) else {
        return false;
    };
    if token.len() != STAGING_NAME_DIGITS {
        return false;
    }
    return token
        .bytes()
        .all(|byte| return byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
}

/// What:
///  A staging name no other writer of this process or of another shares.
/// Why:
///   A killed writer's leftover blocks no later writer only while every
///  writer's own name is free,
///  and the link from the staging name is what decides the race.
///
/// # Errors
/// The operating system's random source failed.
pub(crate) fn fresh_staging_name() -> Result<String, NonceRefusal> {
    let token: RequestNonce = RequestNonce::fresh()?;
    return Ok(format!("{ATTEMPT_SEPARATOR}{}", token.as_str()));
}

/// What:
///  The number the next claim of one clone takes:
///  one past the highest attempt present,
///  empty or readable.
/// Why:
///   A crash can leave the newest attempt empty,
///  and the attempt under it is the clone's readable trigger,
///  so a count of readable attempts alone would hand out a name that stands.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const next = onePastTheHighest([1, 3]); // 4
/// ```
pub(crate) fn one_past_the_highest(attempts: &[u32]) -> u32 {
    let highest: u32 = attempts.iter().copied().max().unwrap_or(0);
    return highest.saturating_add(1);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "unanswered_request_names_tests.rs"]
mod tests;
