//! What:
//!  The words one stage claim file holds,
//! and the readers that say what a claim file holds:
//! the container the claim names on its first line,
//! the same container's bytes on the line after it when it needs them,
//! and the owner that took the claim.
//! Why:
//!  The claim's writer (`worktree_copy_stage_claim.rs`)
//! and the sweeps that read it (`worktree_copy_stage_claim_reaping.rs`)
//! must spell the same lines,
//! and a sweep may act only on a claim this release reads,
//! so the two field names and the readers of a claim live together here,
//! below every sweep that asks.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const fields = claimFields(text); // the container's text, its bytes and the owner
//! const claim = claimFileReading(claimPath); // a claim this release reads, or the reason it is not one
//! ```

use crate::json_record::quote;
use crate::path_bytes_in_records::path_of_the_two_fields;
use crate::worktree_copy_error::{
    WorktreeCopyError,
    failure_with_cause,
    raw_failure,
};
use crate::worktree_copy_journal::{
    Role,
    assert_private_path,
};
use crate::worktree_copy_selection::STAGE_PREFIX;
use std::path::{
    Path,
    PathBuf,
};

/// The line of a claim's text that names its owner,
/// after the line that names the container.
pub(crate) const OWNER_LINE_PREFIX: &str = "owner=";

/// What:
///  The line of a claim's text that holds the container path's bytes.
/// Why:
///   The first line stays the path itself when the path is text and fits one line,
/// because the release installed before this line reads it there;
/// a path that is text cannot spell another one on that line,
/// so both readers name the same directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const CONTAINER_HEX_PREFIX = 'containerHex=';
/// ```
pub(crate) const CONTAINER_HEX_PREFIX: &str = "containerHex=";

/// What:
///  The container and the owner a claim names.
/// Why:
///   `read_claim` acts on these values,
/// so the shape of a record is decided in one place.
pub(crate) struct ClaimFields<'a> {
    /// The container as text,
    /// when the record spells it on one line.
    pub(crate) text: Option<&'a str>,
    /// The container's bytes,
    /// when the record holds them.
    pub(crate) bytes: Option<&'a str>,
    /// The owner token,
    /// when the record names one;
    /// the reader that names a container reads it here.
    pub(crate) owner: Option<String>,
}

/// What:
///  Whether a line of a claim holds the container's bytes.
/// Why:
///   A reader must place the bytes line and count it,
/// so both questions are asked in one way of one name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const holdsBytes = (line: string) => line.startsWith('containerHex=')
/// ```
fn holds_bytes(line: &str) -> bool {
    return line.starts_with(CONTAINER_HEX_PREFIX);
}

/// What:
///  The lines of a claim as this release and the one before it write them:
/// the container as text,
/// its bytes on the line after it,
/// then the owner.
/// Why:
///   The container is removed recursively,
/// so a line of bytes is believed only in the two places a writer puts one,
/// and a claim with any other line is refused:
/// the earlier release wrote a container path verbatim,
/// so a path holding a line break put lines of its own in the record,
/// and one of them could name any private directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const claimFields = (text: string) => { const lines = text.split('\n'); const textLine = lines[0]?.startsWith('containerHex=') ? undefined : lines[0]; ... }
/// ```
pub(crate) fn claim_fields(text: &str) -> Result<ClaimFields<'_>, String> {
    let lines: Vec<&str> = text.lines().collect();
    // The first line is the container's bytes or the container's text; its bytes then sit where this release puts them.
    let named_text: Option<&str> = match lines.first() {
        Some(line) if holds_bytes(line) => None,
        first => first.copied(),
    };
    let named_bytes: Option<&str> = match lines.get(usize::from(named_text.is_some())) {
        Some(line) => line.strip_prefix(CONTAINER_HEX_PREFIX),
        None => None,
    };
    let read: usize = usize::from(named_text.is_some()) + usize::from(named_bytes.is_some());
    // A line of container bytes past those is a path that ended early, not a line a writer put there.
    if lines.iter().skip(read).copied().any(holds_bytes) {
        return Err(String::from(
            "its line of container bytes is not where this release writes one",
        ));
    }
    // The owner, when the record names one, is the last line.
    let mut following = lines.iter().skip(read);
    let Some(first) = following.next() else {
        return Ok(ClaimFields {
            text: named_text,
            bytes: named_bytes,
            owner: None,
        });
    };
    let Some(token) = first.strip_prefix(OWNER_LINE_PREFIX) else {
        return Err(String::from(
            "a line this release does not write follows its container",
        ));
    };
    if following.next().is_some() {
        return Err(String::from("more lines follow its owner"));
    }
    return Ok(ClaimFields {
        text: named_text,
        bytes: named_bytes,
        owner: Some(token.to_string()),
    });
}

/// What:
///  One stage claim as read:
/// the container it names and its owner token,
/// or the reason its bytes are not a record this release writes.
/// Why:
///   The sweep that settles an unreadable claim needs that reason for its one line,
/// and every other reader needs only the claims it may act on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ClaimFile = { kind: 'named'; container: string; owner?: string } | { kind: 'unreadable'; reason: string };
/// ```
pub enum ClaimFile {
    /// A claim this release reads.
    Named {
        /// The stage container it names,
        /// absolute and under a name this wrapper reserves.
        container: PathBuf,
        /// The owner token,
        /// when the record names one.
        owner: Option<String>,
    },
    /// A file at a claim's name whose bytes are not the record this release writes.
    Unreadable {
        /// What one line says about why it does not read.
        reason: String,
    },
}

/// What:
///  One claim file as read,
/// or nothing when it is gone,
/// with the damage a crash can leave named instead of refused.
/// Why:
///   A claim is published whole by one rename,
/// so a file at its name that does not read is the mark of a write a crash cut short;
/// the command that found it moves it aside and settles what it named
/// (`worktree_copy_unreadable_records`).
/// A claim another account owns,
/// or a link or a pipe at its name,
/// stays a refusal:
/// no crash writes one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reading = await claimFileReading(claim); // undefined when the claim is gone
/// ```
pub fn claim_file_reading(claim: &Path) -> Result<Option<ClaimFile>, WorktreeCopyError> {
    // `match` on the lookup: a claim already gone is nobody's fault, and nothing is left to reap.
    match std::fs::symlink_metadata(claim) {
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(raw_failure(&error)),
    }
    assert_private_path(claim, Role::JournalFile)?;
    // `match` on the read: the claim can vanish between the lookup and the read, and bytes that are not text are damage.
    let text: String = match std::fs::read_to_string(claim) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) if error.kind() == std::io::ErrorKind::InvalidData => {
            return Ok(Some(ClaimFile::Unreadable {
                reason: String::from("its bytes are not text"),
            }));
        }
        Err(error) => {
            let message: String = format!(
                "monochromatic-git-policy-cli: could not read worktree-copy stage claim {}.",
                quote(claim.to_string_lossy().as_ref())
            );
            return Err(failure_with_cause(message.as_str(), Some(&error)));
        }
    };
    let fields: ClaimFields<'_> = match claim_fields(text.as_str()) {
        Ok(fields) => fields,
        Err(reason) => {
            return Ok(Some(ClaimFile::Unreadable {
                reason: reason.to_string(),
            }));
        }
    };
    let container: PathBuf = match path_of_the_two_fields(fields.bytes, fields.text) {
        Ok(Some(path)) => path,
        Ok(None) => PathBuf::new(),
        Err(reason) => {
            return Ok(Some(ClaimFile::Unreadable {
                reason: format!("it holds a container in bytes that is not a path ({reason})"),
            }));
        }
    };
    // Both lines of a claim this release writes name one path; a line of text that says otherwise is a shorter path
    // that ended where the line break was, and the bytes it stood beside are not the record's own.
    if let (Some(named), Some(_)) = (fields.text, fields.bytes)
        && container.as_path() != Path::new(named)
    {
        return Ok(Some(ClaimFile::Unreadable {
            reason: String::from(
                "its line of text and its line of container bytes name different paths",
            ),
        }));
    }
    let stage_named: bool = match container.file_name() {
        Some(name) => name.to_string_lossy().starts_with(STAGE_PREFIX),
        None => false,
    };
    let named: bool = container.is_absolute() && stage_named;
    if !named {
        return Ok(Some(ClaimFile::Unreadable {
            reason: String::from("it does not name a stage container"),
        }));
    }
    return Ok(Some(ClaimFile::Named {
        container,
        owner: fields.owner,
    }));
}

/// What:
///  One claim file as read:
/// the stage container it names,
/// refused unless it is a monochromatic-git-policy-cli stage container name,
/// and the owner token,
/// with a claim this release cannot read read as none.
/// Why:
///   The container is deleted recursively,
/// so a claim that names anything else is never acted on;
/// the owner decides whether
///  the claim may be reaped at all.
/// A claim that vanished while it was read was removed by its owner or by another command.
/// A claim whose lines are not a record this release writes is settled by the command that found it,
/// so a sweep that may only act on claims this release reads has nothing there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const named = await readClaim(claimPath); // undefined when gone, unreadable, or of another shape
/// ```
pub fn read_claim(claim: &Path) -> Result<Option<(PathBuf, Option<String>)>, WorktreeCopyError> {
    return match claim_file_reading(claim)? {
        Some(ClaimFile::Named {
            container,
            owner,
        }) => Ok(Some((container, owner))),
        Some(ClaimFile::Unreadable {
            ..
        })
        | None => Ok(None),
    };
}
