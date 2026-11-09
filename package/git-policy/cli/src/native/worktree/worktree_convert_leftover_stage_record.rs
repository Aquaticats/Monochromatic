//! What:
//!  The record,
//! beside a worktree,
//! of the two objects one conversion may leave at `<worktree>.cli-git-converting`,
//!  so `--abandon` can tell that leftover from a directory someone put there instead.
//! Why:
//!  A conversion keeps the original tree at the converting name once the swap happened,
//! and it destroys the subvolume it
//!  created there before the swap.
//! Both are objects the conversion recorded by identity,
//! but the journal that records them lives in the
//!  worktree's registration,
//! and the registration goes with the worktree:
//! a person who removes a worktree after a stop is left with
//!  a full copy that no journal can name any more.
//! The name alone proves nothing,
//! so the two identities are written beside the worktree,
//! in its
//!  parent,
//! where the exchange leaves them:
//! `--abandon` destroys what is there only while its identity is one of
//!  the two,
//! and says what it found otherwise.
//! The record is written once per conversion and removed when the conversion
//!  finishes or its leftover is destroyed,
//! so it never names an object a later conversion made.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await recordLeftoverStage(worktree, { original, created }); // before the swap
//! const known = await leftoverStageOf(worktree);               // { original, created } | undefined
//! await forgetLeftoverStage(worktree);                         // once nothing is left
//! ```

use crate::diagnostic_log::warn;
use crate::fs_beneath::{ObjectId, identity_at, open_root};
use crate::subvolume_file_write_steps::write_beside_then_rename;
use crate::worktree_convert_inspection::converting_path;
use crate::plain_file::read_plain_file;
use std::path::{Path, PathBuf};

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeConvertLeftoverStage";

/// What follows the worktree's name in the record's file name,
/// after the dotted wrapper prefix:
/// `.cli-git-convert-<name>.leftover-stage.json` in the worktree's parent.
const RECORD_SUFFIX: &str = ".leftover-stage.json";

/// The most bytes of this record that are read.
const RECORD_LIMIT: u64 = 4096;

/// The wrapper's prefix of every name it keeps beside a worktree.
const RECORD_PREFIX: &str = ".cli-git-convert-";

/// The identity of the object the conversion expects at the converting name,
/// keyed by when it is there.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LeftoverStage {
    /// The original worktree directory,
    /// which sits at the converting name from the exchange on.
    pub original: ObjectId,
    /// The subvolume the conversion created,
    /// which sits there before the exchange.
    pub created: Option<ObjectId>,
}

/// The record's path for `worktree`:
/// `<parent>/.cli-git-convert-<name>.leftover-stage.json`.
pub fn record_path(worktree: &Path) -> Option<PathBuf> {
    let name: &std::ffi::OsStr = worktree.file_name()?;
    let parent: &Path = worktree.parent()?;
    let mut text: std::ffi::OsString = std::ffi::OsString::from(RECORD_PREFIX);
    text.push(name);
    text.push(RECORD_SUFFIX);
    return Some(parent.join(text));
}

/// What:
///  Record the two identities one conversion may leave at its converting name.
/// Why:
///   See the module description.
/// A record that cannot be written costs only the automatic abandon,
/// so the conversion goes
///  on and says so in the diagnostic log.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recordLeftoverStage(worktree, { original, created });
/// ```
pub fn record_leftover_stage(worktree: &Path, stage: &LeftoverStage) {
    let Some(path) = record_path(worktree) else { return };
    let created: String = match stage.created {
        Some(found) => found.encode(),
        None => String::new(),
    };
    let text: String = format!("original={}\ncreated={created}\n", stage.original.encode());
    // The record is written into a sibling created exclusively under a name of its own and renamed over the record, so a kill
    // leaves a usable one and a link planted at a temporary name is never written through.
    if let Err(error) = write_beside_then_rename(path.as_path(), text.as_bytes()) {
        warn(TAG, format!("{} cannot be written ({error}), so a leftover stage of {} is only named by its path", path.display(), worktree.display()).as_str());
    }
}

/// What:
///  The two identities a conversion recorded for the worktree at `worktree`,
/// or nothing when no record is there and when the record cannot be read.
/// Why:
///   A record that cannot be read proves nothing,
/// so the caller refuses to destroy what the name holds rather than guess.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const known = await leftoverStageOf(worktree); // LeftoverStage | undefined
/// ```
pub fn leftover_stage_of(worktree: &Path) -> Option<LeftoverStage> {
    let path: PathBuf = record_path(worktree)?;
    // Only a plain file is this command's record: opening a named pipe would wait for a writer that never comes.
    let bytes: Vec<u8> = read_plain_file(path.as_path(), RECORD_LIMIT).ok()?;
    let text: String = String::from_utf8(bytes).ok()?;
    let mut original: Option<ObjectId> = None;
    let mut created: Option<ObjectId> = None;
    for line in text.lines() {
        let Some((name, value)) = line.split_once('=') else { continue };
        match name {
            "original" => original = ObjectId::decode(value),
            "created" if !value.is_empty() => created = ObjectId::decode(value),
            _ => {}
        }
    }
    return Some(LeftoverStage { original: original?, created });
}

/// What:
///  The identity of the object at `<worktree>.cli-git-converting` when that object is one a conversion recorded,
/// or nothing when the name is free,
/// holds something else,
/// or no record is there.
/// Why:
///   The record alone says what a conversion *may* have left,
/// not what is there now:
/// a record whose stage is gone is stale,
///  and one whose name holds another object is not about that object.
/// A removal refuses only on this answer,
/// so a stale record
///  cannot block it while a real leftover still does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stage = await recordedStageOf(worktree); // ObjectId | undefined
/// ```
pub fn recorded_stage_of(worktree: &Path) -> Option<ObjectId> {
    let recorded: LeftoverStage = leftover_stage_of(worktree)?;
    let stage: PathBuf = converting_path(worktree);
    let parent: PathBuf = worktree.parent()?.to_path_buf();
    let held: std::fs::File = open_root(parent.as_path()).ok()?;
    let found: ObjectId = identity_at(&held, stage.file_name()?).ok()?;
    if found == recorded.original || recorded.created == Some(found) {
        return Some(found);
    }
    return None;
}

/// Drop the record of `worktree`,
/// which has nothing left to name.
pub fn forget_leftover_stage(worktree: &Path) {
    let Some(path) = record_path(worktree) else { return };
    match std::fs::remove_file(path.as_path()) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => warn(TAG, format!("{} cannot be removed ({error}); a later abandon refuses the stage it names", path.display()).as_str()),
    }
}

/// Record controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_convert_leftover_stage_record_tests.rs"]
mod tests;
