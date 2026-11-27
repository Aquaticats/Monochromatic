//! What:
//!  The records a copy leaves that name a worktree rather than its own private stage:
//!  the record of a worktree whose ignored-state copy did not finish,
//!  moved aside once what it named is found again from the records beside it,
//!  and the destination record of an add whose stage claim is gone,
//!  settled or removed while the worktree it names stands.
//! Why:
//!  Each of these records is what still names a worktree,
//!  so one that cannot be read,
//!  or whose claim a command moved aside,
//!  is settled from the records of the same copy that stand beside it,
//!  and what may still stand on disk is said once.
//!  The records of the copy's own private stage are in
//!  `worktree_copy_unreadable_records.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await settleUnreadableUnfilledRecords(commonDir, notices);
//! const notices = await settleRecordsWithoutAClaim(root);
//! ```

use crate::json_record::quote;
use crate::worktree_add_binding::decode_record;
use crate::worktree_copy_error::{
    WorktreeCopyError,
    raw_failure,
};
use crate::worktree_copy_interrupted_add_settlement::settle_interrupted_add;
use crate::worktree_copy_journal::journal_root;
use crate::worktree_copy_journal_encoding_and_validation::parse_journal_reading;
use crate::worktree_copy_journal_removal::remove_file_if_present;
use crate::worktree_copy_stage_claim::DESTINATION_SUFFIX;
use crate::worktree_copy_unfilled_record::{
    UnfilledFile,
    unfilled_file_reading,
    unfilled_names,
};
use crate::worktree_copy_unreadable_record_aside::move_aside;
use crate::worktree_copy_unreadable_record_words::settled_notice;
use std::ffi::OsStr;
use std::path::{
    Path,
    PathBuf,
};
/// What:
///  The destination one add's record names,
/// when the record beside a claim reads.
/// Why:
///   The stage folder of an add sits in the destination's parent,
/// so the destination is what finds the folder when the record of its object is gone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const destination = decodeRecord(read(claim.replace(/\.stage$/, '.add')))?.destination;
/// ```
pub(super) fn destination_record_of(record: &Path) -> Option<PathBuf> {
    let text: String =
        std::fs::read_to_string(record.with_extension(&DESTINATION_SUFFIX[1..])).ok()?;
    return match decode_record(text.as_str())? {
        crate::worktree_add_binding::Recorded::Planned(destination) => Some(destination),
        crate::worktree_add_binding::Recorded::Registered(binding) => Some(binding.destination),
    };
}

/// What:
///  Moves every record of a worktree whose copy gave up aside,
/// where nothing waits for the copy that wrote it,
/// and names what may stand.
/// Why:
///   The record is what tells every command that a worktree holds less than its source ignores;
/// when it cannot be read,
/// the worktree it named is not named by it any more,
/// and the note says so once.
/// The writer lives only in the record's own bytes,
/// so a record that cannot be read names none,
/// and nothing waits for one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const name of await unfilledNames(root)) { const read = await unfilledFileReading(join(root, name)); if (read.kind !== 'unreadable') continue; ... }
/// ```
pub(super) fn settle_unreadable_unfilled_records(
    common_dir: &Path,
    notices: &mut Vec<String>,
) -> Result<(), WorktreeCopyError> {
    let root: PathBuf = journal_root(common_dir);
    for name in unfilled_names(root.as_path())? {
        let path: PathBuf = root.join(name.as_str());
        let Some(UnfilledFile::Unreadable {
            reason,
        }) = unfilled_file_reading(path.as_path())?
        else {
            continue;
        };
        // The worktree is still named by the records of the copy it belongs to, where they stand and read.
        let destination: Option<PathBuf> = destination_of_the_copy(&root, path.as_path());
        let Some(moved) = move_aside(common_dir, path.as_path(), OsStr::new(name.as_str()))? else {
            continue;
        };
        let left: String = match destination.as_deref() {
            Some(found) => format!(
                "The worktree that copy was filling, {}, may hold less than its source's ignored state, and nothing names it as one that does; `git worktree list` shows the worktrees of this repository, and the record of an unfilled copy is written again only by a copy that runs there.",
                quote(found.to_string_lossy().as_ref())
            ),
            None => String::from(
                "The worktree that copy was filling may hold less than its source's ignored state, and no record of this repository names it any more; `git worktree list` shows the worktrees of this repository.",
            ),
        };
        notices.push(settled_notice(
            "record of a worktree whose ignored-state copy did not finish",
            path.as_path(),
            moved.as_path(),
            reason.as_str(),
            left.as_str(),
        ));
    }
    return Ok(());
}

/// What:
///  The worktree a record of an unfinished copy was written about,
/// as the records of the same copy name it.
/// Why:
///   The record's name shares its identifier with the journal the copy wrote,
/// or with the claim an add wrote,
/// so those records still name the worktree even when the record itself does not read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const destination = journalOf({ root, record })?.destinationRoot ?? destinationRecordOf(claimBeside({ root, record }));
/// ```
fn destination_of_the_copy(
    root: &Path,
    record: &Path,
) -> Option<PathBuf> {
    // The record's name is `<identifier>.unfilled`, and its copy's journal is `<identifier>.<owner>.json`: the owner token
    // sits between them, so the identifier is the prefix a journal of this copy starts with.
    let identifier: String = record
        .file_name()?
        .to_str()?
        .strip_suffix(".unfilled")?
        .to_string();
    let prefix: String = format!("{identifier}.");
    let listing = match std::fs::read_dir(root) {
        Ok(listing) => listing,
        Err(_unreadable) => {
            return destination_record_of(root.join(format!("{identifier}.stage")).as_path());
        }
    };
    let mut journals: Vec<PathBuf> = Vec::new();
    for listed in listing.flatten() {
        let path: PathBuf = listed.path();
        let name: String = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        if name.starts_with(prefix.as_str()) && name.ends_with(".json") {
            journals.push(path);
        }
    }
    journals.sort_by(|left, right| {
        return crate::worktree_copy_selection::js_order(
            &left.to_string_lossy(),
            &right.to_string_lossy(),
        );
    });
    for journal in journals {
        let Ok(bytes) = std::fs::read(journal.as_path()) else {
            continue;
        };
        let Ok(text) = String::from_utf8(bytes) else {
            continue;
        };
        if let Ok(found) = parse_journal_reading(text.as_str(), journal.as_path(), root) {
            return Some(found.destination_root);
        }
    }
    return destination_record_of(root.join(format!("{identifier}.stage")).as_path());
}

/// What:
///  Settles what records of an add remain when no claim names them,
/// and removes the markers of adds whose claim is gone.
/// Why:
///   A record of an add is read through the claim beside it,
/// so a claim a command moved aside leaves the record unread;
/// the worktree it names must still be named,
/// which is what its own settlement does,
/// and the record is kept while something stands that it describes.
/// A marker whose claim is gone says nothing about anything.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const notices = await settleRecordsWithoutAClaim(root);
/// ```
pub fn settle_records_without_a_claim(root: &Path) -> Result<Vec<String>, WorktreeCopyError> {
    let listing = match std::fs::read_dir(root) {
        Ok(listing) => listing,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(raw_failure(&error)),
    };
    let mut records: Vec<PathBuf> = Vec::new();
    let mut markers: Vec<PathBuf> = Vec::new();
    for listed in listing {
        let path: PathBuf = listed
            .map_err(|error| {
                return raw_failure(&error);
            })?
            .path();
        let name: String = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        // A claim's own files end in `.stage`; everything a claim names keeps that name and changes only its suffix.
        let (identifier, marker): (&str, bool) = match name.strip_suffix(DESTINATION_SUFFIX) {
            Some(stem) => (stem, false),
            None => match name.strip_suffix(crate::worktree_copy_stage_claim::COMPLETE_SUFFIX) {
                Some(stem) => (stem, true),
                None => continue,
            },
        };
        // A live add marks its worktree complete and removes its claim a moment later, so only a marker whose claim is
        // already gone is one no add stands behind.
        if root.join(format!("{identifier}.stage")).exists() {
            continue;
        }
        if marker {
            markers.push(path);
        } else {
            records.push(path);
        }
    }
    markers.sort_by(|left, right| {
        return crate::worktree_copy_selection::js_order(
            &left.to_string_lossy(),
            &right.to_string_lossy(),
        );
    });
    records.sort_by(|left, right| {
        return crate::worktree_copy_selection::js_order(
            &left.to_string_lossy(),
            &right.to_string_lossy(),
        );
    });
    let mut notices: Vec<String> = Vec::new();
    for record in records {
        let settled = settle_interrupted_add(record.as_path());
        if let Some(notice) = settled.notice {
            notices.push(notice);
        }
        if settled.keep {
            continue;
        }
        remove_file_if_present(record.as_path())?;
        remove_file_if_present(
            record
                .with_extension(&crate::worktree_copy_stage_claim::COMPLETE_SUFFIX[1..])
                .as_path(),
        )?;
    }
    for marker in markers {
        // The marker says an add finished; with no record beside it there is no add it could be about.
        remove_file_if_present(marker.as_path())?;
    }
    return Ok(notices);
}
