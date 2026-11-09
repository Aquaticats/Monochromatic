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
use crate::fs_beneath::ObjectId;
use std::path::{Path, PathBuf};

/// Tag used for diagnostics from this module.
const TAG: &str = "worktreeConvertLeftoverStage";

/// What follows the worktree's name in the record's file name,
/// after the dotted wrapper prefix:
/// `.cli-git-convert-<name>.leftover-stage.json` in the worktree's parent.
const RECORD_SUFFIX: &str = ".leftover-stage.json";

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
    // The record is written whole under a name of its own and renamed over the record, so a kill leaves a usable one.
    let sibling: PathBuf = path.with_extension(format!("json.{}", std::process::id()));
    if let Err(error) = std::fs::write(sibling.as_path(), text.as_bytes()) {
        warn(TAG, format!("{} cannot be written ({error}), so a leftover stage of {} is only named by its path", sibling.display(), worktree.display()).as_str());
        return;
    }
    if let Err(error) = std::fs::rename(sibling.as_path(), path.as_path()) {
        warn(TAG, format!("{} cannot be put in place ({error}), so a leftover stage of {} is only named by its path", path.display(), worktree.display()).as_str());
        let _removed: std::io::Result<()> = std::fs::remove_file(sibling.as_path());
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
    let text: String = std::fs::read_to_string(path.as_path()).ok()?;
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
