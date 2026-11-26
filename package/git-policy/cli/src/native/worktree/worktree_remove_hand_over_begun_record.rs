//! What:
//!  The record that says the removal of one worktree has begun,
//!  with the identity it is being removed under.
//! Why:
//!  Git's own removal deletes the worktree's files one by one,
//!  and a service killed in the middle leaves a tree whose tracked files are gone while the registration stays.
//!  Git then reads that tree as changed and refuses to remove it,
//!  so without this record the next attempt would keep the half-deleted worktree for good.
//!  Written before Git deletes anything,
//!  it says two things a later attempt needs:
//!  that the proofs held when this worktree was handed to Git,
//!  and which worktree they held for.
//!  It goes with the worktree once the worktree is gone.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await recordBegun(commonDir, id, identity); // before `git worktree remove`
//! forgetBegun(commonDir, id); // once the worktree is gone
//! const begun = begunIdentity(commonDir, id); // string | undefined
//! ```

use crate::diagnostic_log::warn;
use crate::private_storage::write_private_file_unsynced;
use std::path::{
    Path,
    PathBuf,
};

/// The tag every log line of this subject carries.
///
/// What:
///  `const TAG: &str = "worktreeRemoveHandOver"` is the name this program's own log lines are tagged with.
///  The subject is one,
/// so every file it was split into carries the same word,
///  and a person's search of the log keeps finding all of it.
/// Why:
///  A diagnostic is read beside the others of its subject,
///  and a tag that changed with the file would hide half of them.
const TAG: &str = "worktreeRemoveHandOver";

/// The folder of the common Git directory that holds one record per removal the service has begun,
/// named by the record's id.
///
/// It says that the proofs held and that the deletion started,
/// which is what lets the next attempt finish a removal a kill cut short instead of refusing it.
pub const BEGUN_FOLDER: &str = "monochromatic-git-policy-cli/removals-begun";

/// What:
///  Record that the removal of this worktree has begun,
///  with the identity it is being removed under,
///  before Git deletes anything.
/// Why:
///  Git's own removal deletes the worktree's files one by one,
///  and a service killed in the middle leaves a tree whose tracked files are gone while the registration and the
///  ignored state stay.
///  Git then reads that tree as changed and refuses to remove it,
///  so the next attempt would settle the record as failed and the half-deleted worktree would stay for good,
///  with the command the notice promises failing on a `.git` that does not exist
///  (an audit of the release changeover,
///  finding 3).
///  A record written before the deletion says two things a later attempt needs:
///  that the proofs held when this worktree was handed to Git,
///  and which worktree they held for.
///  A write that fails costs nothing that was not already at risk and is logged:
///  the removal still runs,
///  and a kill then leaves the state this record exists to prevent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recordBegun(commonDir, id, identity); // before `git worktree remove`
/// ```
pub(crate) fn record_begun(
    common_dir: &Path,
    id: &str,
    identity: &str,
) {
    let folder: PathBuf = common_dir.join(BEGUN_FOLDER);
    if let Err(error) = crate::private_storage::create_private_directory_all(folder.as_path()) {
        warn(TAG, format!("the removal of {id} is not recorded as begun, so a service killed in it leaves a worktree Git refuses: {error}").as_str());
        return;
    }
    if let Err(error) = write_private_file_unsynced(
        folder.join(id).as_path(),
        format!("{identity}\n").as_bytes(),
    ) {
        warn(TAG, format!("the removal of {id} is not recorded as begun, so a service killed in it leaves a worktree Git refuses: {error}").as_str());
    }
}

/// What:
///  Forget that this removal began,
///  once its worktree is gone.
/// Why:
///   A record that outlived its worktree would let a later record of the same id remove a worktree that never had
///  these proofs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// forgetBegun(commonDir, id);
/// ```
pub(crate) fn forget_begun(
    common_dir: &Path,
    id: &str,
) {
    let _removed = std::fs::remove_file(common_dir.join(BEGUN_FOLDER).join(id));
}

/// What:
///  The identity this removal was begun under,
///  or nothing when no removal of this id has begun.
/// Why:
///   The record is the only thing that says a half-deleted tree is this program's own work and not a person's changes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const begun = begunIdentity(commonDir, id); // string | undefined
/// ```
pub(crate) fn begun_identity(
    common_dir: &Path,
    id: &str,
) -> Option<String> {
    // The begun record is this program's own, so a name that holds a named pipe answers as no record at once.
    let text: String =
        crate::plain_file::read_plain_text(&common_dir.join(BEGUN_FOLDER).join(id)).ok()?;
    let identity: &str = text.trim_end();
    return (!identity.is_empty()).then(|| return identity.to_owned());
}
