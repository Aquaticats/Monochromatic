//! What:
//!  The record of an index update a landed batch still owes,
//! written in the worktree's own Git directory where both the service and
//!  the person's next command read it,
//! and the repair that repeats the update for the entries the landing itself left.
//! Why:
//!   A batch whose branch moved and whose index update did not finish is finished by a restart,
//! and the journal's intent names the paths,
//!  the index and the commit.
//! The intent is one file of many fields,
//! and a build that cannot read it finishes nothing:
//! a person's
//!  next commit then records a file the landed commit added as a staged deletion,
//! because the index still describes the branch before
//!  the landing (`doc/planning/cli-git-commit-and-landing-route-findings-on-main.md`,
//! finding 1 and finding 2).
//! This record is the
//!  obligation on its own:
//! small,
//! fixed in shape,
//! and beside the worktree the update belongs to,
//! so it outlives the intent and is read by the command
//!  that would record the deletion.
//!   The repair resets only the entries that still hold what the batch started from,
//! so staging that is the person's
//!  own is kept,
//! and a path the person staged since the landing is named and left.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await writeOwed({ gitDir, index, old, new, paths }); const left = await leftovers(owed, realIndex); await reset(left);
//! ```

use crate::capture_invocation::InvocationCapture;
use crate::commit_transaction::TransactionStop;
pub use crate::owed_index_repair::{
    exclude_what_a_landed_commit_left,
    exclude_with_note,
    excluded_notice,
    unreadable_notice,
};
use crate::private_index_request::CaptureIndexes;
use crate::private_storage::{
    create_private_directory_all,
    write_private_file_unsynced,
};
use crate::transaction_git::GitContext;
use crate::transaction_journal::TransactionMode;
use std::path::{
    Path,
    PathBuf,
};

use crate::the_record_of_an_owed_index_update::encode;
pub use crate::the_record_of_an_owed_index_update::{
    OWED_EXTENSION,
    OWED_VERSION,
    Owed,
    SETTLED_EXTENSION,
    batch_number_of_path,
    directory_of,
    of_intent,
    path_of,
    records_and_unreadable,
    records_of,
};

/// What:
///  Records that a batch's index update is owed,
/// before anything tries to install it.
/// Why:
///   A kill,
/// a failing step or a restart that declines to replay leaves the obligation here,
/// where the person's own next
///  command reads it.
/// The write is not flushed:
/// what a power loss loses is the intent's own record too,
/// and a record that is
///  lost costs the notice,
/// never the commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeOwed(owed) // the path, or nothing when the folder cannot be made
/// ```
pub fn write(owed: &Owed) -> Option<PathBuf> {
    let path: PathBuf = path_of(owed.git_dir.as_path(), owed.seq);
    let folder: PathBuf = directory_of(owed.git_dir.as_path());
    if let Err(error) = create_private_directory_all(folder.as_path()) {
        crate::service_error!(
            "batch {} {}: the record of its owed index update cannot be made in {} ({error})",
            owed.seq,
            crate::service_timing::in_the_repository_at(owed.repository.as_path()),
            folder.display()
        );
        return None;
    }
    // A replay writes what the first landing wrote, and a batch built again under its own number replaces the commit it names: an
    //  identical record is left as it is and anything else goes before the new one is published.
    let text: String = encode(owed);
    // A record that is a named pipe is not the record this batch wrote, and it answers at once rather than waiting on.
    let unchanged: bool = match crate::plain_file::read_plain_text(path.as_path()) {
        Ok(existing) => existing == text,
        Err(_) => false,
    };
    if unchanged {
        return Some(path);
    }
    let _gone: std::io::Result<()> = std::fs::remove_file(path.as_path());
    return match write_private_file_unsynced(path.as_path(), text.as_bytes()) {
        Ok(()) => Some(path),
        Err(error) => {
            crate::service_error!(
                "batch {} {}: the record of its owed index update cannot be written in {} ({error})",
                owed.seq,
                crate::service_timing::in_the_repository_at(owed.repository.as_path()),
                path.display()
            );
            None
        }
    };
}

/// The path that says one batch's obligation is over.
pub fn settled_path_of(
    git_dir: &Path,
    seq: u64,
) -> PathBuf {
    return directory_of(git_dir).join(format!("{seq}.{SETTLED_EXTENSION}"));
}

/// What:
///  Says that a batch's owed index update is settled.
/// Why:
///   The rename is atomic,
/// so a command that finds the owed name still sees an obligation,
/// and one that finds the
///  settled name sees that there is none.
/// The settled file is removed at once;
/// one a kill leaves is removed by the next command,
/// and it
///  carries no obligation either way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// settle(gitDir, seq) // rename `.owed` to `.settled`, then unlink
/// ```
pub fn settle(
    git_dir: &Path,
    seq: u64,
) {
    let settled: PathBuf = settled_path_of(git_dir, seq);
    if std::fs::rename(path_of(git_dir, seq), settled.as_path()).is_ok() {
        let _removed: std::io::Result<()> = std::fs::remove_file(settled.as_path());
    }
}

/// Removes the record of one batch,
/// settled or not;
/// a record that cannot be removed stays for the next command to find.
pub fn remove(
    git_dir: &Path,
    seq: u64,
) {
    let _removed: std::io::Result<()> = std::fs::remove_file(path_of(git_dir, seq));
    let _settled: std::io::Result<()> = std::fs::remove_file(settled_path_of(git_dir, seq));
}

/// What:
///  What this commit does about a landed batch's owed index update,
/// or the stop that ends the command when the repair it decided on cannot be made.
/// Why:
///   The repair returns a reason,
/// and a commit that would otherwise record the content the landing replaced must not
///  be made:
/// the reason becomes the command's own message.
/// This is the whole of the wiring between two files,
/// so it is
///  asked here rather than inlined.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// excludeOrStop(context, capture, indexes, mode, witness)
/// ```
pub fn exclude_or_stop(
    context: &GitContext,
    capture: &InvocationCapture,
    indexes: &CaptureIndexes,
    mode: TransactionMode,
    captured: &crate::service_land_start::IndexWitness,
) -> Result<(Vec<String>, Option<String>), TransactionStop> {
    let mut note: Option<String> = None;
    let repair: Result<Vec<String>, String> = crate::owed_index_update::exclude_with_note(
        context,
        capture.layout.repository_root.as_path(),
        capture.layout.git_dir.as_path(),
        capture.layout.real_index_path.as_path(),
        match mode {
            TransactionMode::Index => Some(indexes.commit_index.as_path()),
            TransactionMode::ExplicitPath => None,
        },
        Some(indexes.captured_index.as_path()),
        captured,
        &mut note,
    );
    return match repair {
        Ok(excluded) => Ok((excluded, note)),
        Err(reason) => Err(crate::commit_transaction::failed(reason)),
    };
}
