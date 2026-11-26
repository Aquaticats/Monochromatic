//! What:
//!  Writing a note:
//!  the landing's own write for the service that starts next,
//! and the store's write of one directory under the first free name that is not another's.
//! Why:
//!  A person's next command in a worktree depends on the work a kill left unrun,
//! so the landing writes the repository's note before it answers the commit.
//! Both writers publish one file the same way,
//! and both are read back by the store in `service_claim_watch.rs`.
//! The store re-exports what this file holds,
//!  so a caller names the store's file alone.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! noteForTheNextService(store, instance, gitDir) // write it down unless it belongs to another instance
//! writeDown(directory, gitDir) // an exclusive create, then the next name when the first is another's
//! ```

use super::service_claim_watch_note_format::{
    NOTE_FILE_LIMIT,
    decode,
    encode,
    watch_name,
};
use crate::private_storage::{
    ReadRefusal,
    create_private_directory_all,
    read_regular_file,
    write_private_file_unsynced,
};
use crate::service_claim_watch_whose_repository_it_is::of_another_instance;
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  The point at which the landing writes its note for the service that starts next,
///  about `git_dir`,
///  for a control that holds that write (`service_fault_hooks_test_support.rs`).
///  The work loop writes the same note through `write_down` and never reaches this point,
///  so a hold here is the landing's own write and nothing else.
/// Why:
///   The point names the Git directory the note is about,
///  so two controls of one process hold the write of their own repository and neither answers the other (issue 6497),
///  as `collection_point` names the directory it collects.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const nextServiceNotePoint = (gitDir: string) => `the-next-service-note-of-${gitDir}`;
/// ```
#[cfg(test)]
pub(crate) fn next_service_note_point(git_dir: &std::path::Path) -> String {
    return format!("the-next-service-note-of-{}", git_dir.display());
}

/// What:
///  Writes `git_dir` down for the next service of `instance`,
/// from a thread that does not own the watch.
/// Why:
///   The landing thread knows the repository before any client wakes anything,
/// and a wrapper killed between promoting a landed commit's push and waking the service leaves a `pending` record that
///  nothing would drain.
/// The promise that a kill is followed by the work being run with no commit needed rests on the journal naming the
///  repository,
/// so the note is written where the landing already records what it owes,
/// before the client is answered.
///   It is the same exclusive publish the work loop's `remember` uses,
/// so the two writers cannot disagree:
/// a note that is already there and holds this directory is left as it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const noteForTheNextService = (store, instance, gitDir) => { if (!ofAnotherInstance(instance, gitDir)) writeDown(store, gitDir); };
/// ```
pub fn note_for_the_next_service(
    store: &Path,
    instance: Option<&str>,
    git_dir: &Path,
) {
    // A control holds this write on whichever thread runs it, which `note_for_the_next_service_within` made a thread of its own
    //  (`service_claim_watch_note_within_a_ceiling.rs`).
    #[cfg(test)]
    {
        let held_point: String = next_service_note_point(git_dir);
        crate::service_fault::hooks::run_on_any_thread(held_point.as_str());
    }
    if of_another_instance(instance, git_dir) {
        return;
    }
    if let Err(reason) = write_down(store, git_dir) {
        crate::service_warn!(
            "{} could not be kept for a restart, so a service that starts may not look at it: {reason}",
            git_dir.display()
        );
    }
}

/// Writes the file of `git_dir` under the first name that is free or already holds this directory.
/// A file that holds no whole path is
/// replaced;
/// one that holds another directory's path is left,
/// and the next name is tried.
/// What holds a taken name is read the way every record of this wrapper is read,
/// so a name holding a pipe,
/// a link,
/// a directory or more bytes than a note can hold is answered at once,
/// left where it is,
/// and passed over for the next name:
/// no note this store wrote is any of those,
/// and a plain read of such a name waits in the kernel,
/// reads a link through,
/// or costs memory for nothing.
pub(crate) fn write_down(
    directory: &Path,
    git_dir: &Path,
) -> Result<(), String> {
    if let Err(error) = create_private_directory_all(directory) {
        return Err(format!("creating the note directory failed: {error}"));
    }
    let text: String = encode(git_dir);
    let mut attempt: u32 = 0;
    loop {
        let path: PathBuf = directory.join(watch_name(git_dir, attempt));
        match write_private_file_unsynced(path.as_path(), text.as_bytes()) {
            Ok(()) => return Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(format!("writing the note failed: {error}")),
        }
        // What holds the taken name decides what this write does with it.
        let next_name: bool = match read_regular_file(path.as_path(), NOTE_FILE_LIMIT) {
            Ok(held) if held == text.as_bytes() => return Ok(()),
            // Another directory's note, which keeps its name.
            Ok(held) if decode(held.as_slice()).is_ok() => true,
            // A note of this store that was torn is replaced by the note of this name.
            Ok(_) => {
                if let Err(removal) = std::fs::remove_file(path.as_path()) {
                    return Err(format!(
                        "removing the damaged note {} failed: {removal}",
                        path.display()
                    ));
                }
                false
            }
            // A pipe, a link or a directory, and a file longer than a note can hold: left where they are.
            Err(ReadRefusal::NotRegular | ReadRefusal::TooLarge) => true,
            // The name went between the create that found it taken and this read, so the same name is tried again.
            Err(ReadRefusal::Missing) => false,
            Err(ReadRefusal::Io(error)) => {
                return Err(format!(
                    "reading the note {} failed: {error}",
                    path.display()
                ));
            }
        };
        if next_name {
            attempt = attempt.saturating_add(1);
        }
    }
}
