//! What:
//! The commit services of worktrees that no longer exist:
//! the note that says which worktree a service belongs to,
//! the test of whether that worktree or its repository is gone,
//! and the removal of what a gone one left.
//! Why:
//! An instance is named by a hash of its worktree's own Git directory (`service_instance.rs`),
//! and a hash names nothing once the directory is gone:
//! removing a worktree left its socket and its service running,
//! and nothing could say whose they were.
//! A service exits only three hours after its last client,
//! and a socket file a killed one left stays,
//! so provisioning writes the Git directory beside the socket
//! (`<runtime>/monochromatic-git-policy-cli/<instance>.git-dir`),
//! and a service whose note is missing is mapped from the `gitdir=` line of its journal's records,
//! only when the text hashes to the instance.
//! A service that maps to nothing is reported and never stopped.
//! Nothing here asks a service manager or signals a process by name:
//! a service is asked to leave over its own address,
//! and its own note, which it writes when it listens,
//! says when its process is gone.
//! A service that starts and finds its own worktree gone removes its own address before it serves anything,
//! so a removal that Git ran with the wrapper replaced,
//! or a worktree deleted without Git,
//! takes its service with it at the first connection rather than at the next sweep a person remembers.
//! The journal of an instance whose service is gone goes with it
//! (`remove_journal_of_a_stopped_instance`),
//! because a stopped instance is never served again and its records name a worktree that is gone;
//! a journal that holds a batch no branch shows,
//! a record that was written and not published,
//! or a file this release does not know is kept and named instead of discarded,
//! and so is a journal whose landed batches a client may still read:
//! those keep the note and the socket file too,
//! so a later sweep finishes.
//! A service that stops its own socket never removes its own journal,
//! because it is still one of that journal's writers:
//! its after-commit task and that task's jobs write the note of a repository to look at and the plan of a push to try again,
//! and a write that lands between the listing and the removal would be taken with the whole tree.
//! It keeps the note and the socket file instead,
//! so the sweep that proves its process gone removes the journal once that is over.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! if (worktreeIsGone(await worktreeOf(folders, instance))) await stopUnits(folders, instance);
//! removeJournalOfAStoppedInstance(folders, instance);
//! ```

use crate::diagnostic_log::warn;
use crate::path_bytes_in_records as path_bytes;
use crate::path_bytes_in_records::{GITDIR_HEX_PREFIX, path_of_hex};
use crate::service_instance::{INSTANCE_LENGTH, SOCKET_DIRECTORY, instance_of, socket_path};
use crate::service_route::non_empty_path_variable;
use crate::wrapper_names_old_and_new::SERVICE_UNIT;
use crate::service_is_gone_after_a_stop::service_is_gone;
use crate::service_server_pid_note_of_an_instance::{path as pid_note_path, remove as remove_pid_note};
use crate::service_server_start_by_the_wrapper::answered;
use std::path::{Path, PathBuf};

/// Tag used for diagnostics from this module.
const TAG: &str = "serviceUnitsOfRemovedWorktrees";

/// Where the units of the user's instances are told apart from one another.
#[derive(Clone, Copy, Debug)]
pub struct UnitFolders<'a> {
    /// The user's runtime directory,
    /// which holds the sockets and the notes.
    pub runtime_dir: &'a Path,
    /// The user's state directory,
    /// whose `monochromatic-git-policy-cli/service/<instance>` folders hold the service journals.
    pub state_dir: Option<&'a Path>,
}

/// The journal folder of an instance and the state folder that holds it,
/// which the unit's `--journal` names.
pub use crate::service_journal_folder_of_an_instance::{JOURNAL_FOLDER, journal_of, state_dir_of_journal};

/// What an instance keeps beside its socket and in its journal,
/// read and written by the code here and by provisioning
/// (`service_files_of_a_stopped_instance.rs`).
pub use crate::service_files_of_a_stopped_instance::{
    JournalAfterStop, KeptJournal, NOTE_ENDING, NOTES_FOLDER, forget_stopped_instance, journal_of_the_instance, note_path, note_worktree,
    record_worktree, remove_journal_of_a_stopped_instance, work_a_person_may_want,
};

/// What a unit instance's worktree is,
/// as far as the notes and the journals say.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UnitWorktree {
    /// The worktree's Git directory,
    /// whose text hashes to the instance.
    Known(PathBuf),
    /// Nothing maps the instance to a worktree.
    Unmapped,
}

/// Which instances have a unit,
/// whose worktree is gone,
/// and which cannot be told.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Survey {
    /// The instances whose worktree's Git directory no longer exists.
    pub gone: Vec<String>,
    /// The instances that map to nothing.
    pub unmapped: Vec<String>,
    /// The instances whose worktree is still there.
    pub present: usize,
}

/// What a stop of the units of gone worktrees did.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SweepOutcome {
    /// The instances whose units were stopped.
    pub stopped: Vec<String>,
    /// The instances whose units could not be stopped,
    /// each with the reason.
    pub failed: Vec<(String, String)>,
    /// The journals that stayed after their instance's units were stopped,
    /// each with the instance and the entry that kept it.
    pub journals_kept: Vec<(String, KeptJournal)>,
    /// The survey the stop was made from.
    pub survey: Survey,
}

/// What:
/// Whether the worktree whose own Git directory is `git_dir` is certainly gone.
/// Why:
///   A Git directory that is missing while the place that holds it is there is the worktree Git removed:
/// a linked worktree's directory is `<common>/worktrees/<id>`,
/// and Git removes `worktrees` itself with the last one,
/// so the common directory is the place that must be there;
/// a main checkout's Git directory is held by the checkout.
/// A directory that is on a drive which is not mounted has a missing Git directory too,
/// and its worktree is not gone,
/// so a place that is missing as well is read by the same proof the pushes waiting for a new try are read by
/// (`unreachable_git_dir_proven_gone.rs`):
/// the nearest directory above the Git directory that exists,
/// which proves the repository deleted when it is the root of a mounted filesystem or holds other entries,
/// and proves nothing when it holds nothing at all.
/// Both questions are asked here,
/// so the service,
/// the survey,
/// the sweep and `doctor` never disagree about a gone worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// worktreeIsGone('/repo/.git/worktrees/topic') // true when /repo/.git is a directory and topic is not there
/// ```
pub fn worktree_is_gone(git_dir: &Path) -> bool {
    match std::fs::symlink_metadata(git_dir) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        _ => return false,
    }
    let Some(parent) = git_dir.parent() else { return false };
    let anchor: Option<&Path> = if parent.file_name() == Some(std::ffi::OsStr::new("worktrees")) { parent.parent() } else { Some(parent) };
    if let Some(place) = anchor
        && place.is_dir()
    {
        return true;
    }
    // The place that held the Git directory is missing too: the whole repository may have been deleted, or it may be on a
    //  drive that is not mounted. The proof the retry plans use is what tells the two apart, and it reads no worktree of its
    //  own: a worktree that is still there is one whose Git directory moved, which the caller of the proof names where it knows it.
    return matches!(
        crate::unreachable_git_dir_proven_gone::prove(git_dir, None, &crate::unreachable_git_dir_proven_gone::the_mounts()),
        crate::unreachable_git_dir_proven_gone::Proof::Gone(_)
    );
}

/// What:
///  The Git directory a note or a record line names for an instance,
/// when the path it names hashes to that instance.
/// Why:
///   A note of this release holds the bytes of the directory and one line break after them;
/// a note of an earlier release holds its text and nothing after it,
/// so the last byte of that text is the path's even when it is a line break.
/// A note whose bytes do not decode names nothing,
/// and its units stay where they are rather than being stopped on a guess.
fn named_for(instance: &str, text: &str) -> Option<PathBuf> {
    let (bytes, plain): (Option<&str>, Option<&str>) = match text.strip_prefix(GITDIR_HEX_PREFIX) {
        Some(hexadecimal) => (Some(hexadecimal.strip_suffix('\n').unwrap_or(hexadecimal)), None),
        None if text.is_empty() => (None, None),
        None => (None, Some(text)),
    };
    let path: PathBuf = path_bytes::path_of_the_two_fields(bytes, plain).ok().flatten()?;
    return (instance_of(path.as_path()) == instance).then_some(path);
}

/// The records of a service journal,
/// sorted by name.
fn journal_records(journal: &Path) -> Vec<PathBuf> {
    let Ok(listing) = std::fs::read_dir(journal) else { return Vec::new() };
    let mut records: Vec<PathBuf> = Vec::new();
    for entry in listing.flatten() {
        let path: PathBuf = entry.path();
        if path.extension() == Some(std::ffi::OsStr::new("done")) {
            records.push(path);
        }
    }
    records.sort();
    return records;
}

/// The `gitdir=` text of the first record of the instance's service journal that has one,
/// and names a directory that hashes to the instance.
fn from_journal(state_dir: &Path, instance: &str) -> Option<PathBuf> {
    let journal: PathBuf = state_dir.join(SERVICE_UNIT.current).join("service").join(instance);
    for record in journal_records(journal.as_path()) {
        let Ok(text) = std::fs::read_to_string(&record) else { continue };
        // A line of this release names the bytes of the directory, which is the only spelling that keeps a path that is not
        //  text; a line of an earlier release names its plain text.
        let mut plain: Option<PathBuf> = None;
        for line in text.lines() {
            if let Some(hexadecimal) = line.strip_prefix(GITDIR_HEX_PREFIX) {
                if let Some(found) = path_of_hex(hexadecimal)
                    && instance_of(found.as_path()) == instance
                {
                    return Some(found);
                }
                continue;
            }
            if plain.is_none()
                && let Some(named) = line.strip_prefix("gitdir=")
            {
                plain = named_for(instance, named);
            }
        }
        if plain.is_some() {
            return plain;
        }
    }
    return None;
}

/// What:
/// The worktree of an instance:
/// its note,
/// else the Git directory in a record of its service journal,
/// each only when its text hashes to the instance.
/// Why:
///   A text that does not hash to the instance names some other directory,
/// and stopping the units of a worktree on a wrong guess would cut a live one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// worktreeOf(folders, instance) // { known: gitDir } or unmapped
/// ```
pub fn worktree_of(folders: &UnitFolders<'_>, instance: &str) -> UnitWorktree {
    if let Ok(text) = std::fs::read_to_string(note_path(folders.runtime_dir, instance))
        && let Some(found) = named_for(instance, text.as_str())
    {
        return UnitWorktree::Known(found);
    }
    if let Some(state) = folders.state_dir
        && let Some(found) = from_journal(state, instance)
    {
        return UnitWorktree::Known(found);
    }
    return UnitWorktree::Unmapped;
}

/// Whether one character is a lower-case hexadecimal digit.
fn is_hexadecimal_digit(character: char) -> bool {
    return character.is_ascii_digit() || ('a'..='f').contains(&character);
}

/// Whether a file name's stem is an instance name:
/// the digits of a hash.
fn is_instance_name(name: &str) -> bool {
    return name.len() == INSTANCE_LENGTH && name.chars().all(is_hexadecimal_digit);
}

/// The instance a file name belongs to when it ends with `ending` after an instance name.
fn instance_before<'a>(name: &'a str, ending: &str) -> Option<&'a str> {
    let stem: &str = name.strip_suffix(ending)?;
    return is_instance_name(stem).then_some(stem);
}

/// Adds the instances that the files of `folder` ending with `ending` name.
fn add_instances_of(folder: &Path, ending: &str, found: &mut Vec<String>) {
    let Ok(listing) = std::fs::read_dir(folder) else { return };
    for entry in listing.flatten() {
        let name: String = entry.file_name().to_string_lossy().into_owned();
        if let Some(instance) = instance_before(name.as_str(), ending) {
            found.push(instance.to_owned());
        }
    }
}

/// Which instances have a socket file in the socket directory or a note in the notes folder,
/// sorted.
pub fn instances_with_units(runtime_dir: &Path) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    let folder: PathBuf = runtime_dir.join(SOCKET_DIRECTORY);
    add_instances_of(folder.as_path(), ".sock", &mut found);
    add_instances_of(folder.as_path(), NOTE_ENDING, &mut found);
    found.sort();
    found.dedup();
    return found;
}

/// What:
/// Sorts the instances that have a unit into those whose worktree is gone,
/// those that map to nothing,
/// and the count of those whose worktree is still there.
/// Why:
///   The sort reads files only,
/// so `doctor` can report it without a manager.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { gone, unmapped, present } = survey(folders);
/// ```
pub fn survey(folders: &UnitFolders<'_>) -> Survey {
    let mut found: Survey = Survey::default();
    for instance in instances_with_units(folders.runtime_dir) {
        match worktree_of(folders, instance.as_str()) {
            UnitWorktree::Unmapped => found.unmapped.push(instance),
            UnitWorktree::Known(git_dir) if worktree_is_gone(git_dir.as_path()) => found.gone.push(instance),
            UnitWorktree::Known(_present) => found.present += 1,
        }
    }
    return found;
}

/// What:
/// The Git directory of an instance whose worktree is gone,
/// and nothing when it is still there,
/// when it cannot be told whose the units are,
/// or when the place that held the directory is not there either.
/// Why:
///   This is the question a service asks about itself before it serves:
/// a service of a worktree that no longer exists has nothing to commit to,
/// and the files of its socket and its note are the only record of whose they are.
/// The answer is the same one the survey and the sweep give,
/// so a service and the sweep never disagree about a gone worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ownWorktreeGone(folders, instance) // '/repo/.git/worktrees/topic' or undefined
/// ```
pub fn own_worktree_gone(folders: &UnitFolders<'_>, instance: &str) -> Option<PathBuf> {
    let UnitWorktree::Known(git_dir) = worktree_of(folders, instance) else { return None };
    return worktree_is_gone(git_dir.as_path()).then_some(git_dir);
}

/// What a service's own units are to it,
/// once it asks about the worktree they serve.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OwnWorktree {
    /// The worktree's Git directory is certainly gone:
    /// the service has nothing to commit to and stops its own socket.
    Gone(PathBuf),
    /// The worktree is there,
    /// or it is on a device that cannot be reached right now,
    /// so the service serves it.
    Served(PathBuf),
    /// Nothing maps the instance to a worktree,
    /// so the service serves and nothing is stopped.
    Unmapped,
}

/// What:
/// What this service's own units are,
/// read from the runtime directory of the process and the journal it was started with.
/// Why:
///   This is the question a serving service asks of itself:
/// the note provisioning wrote beside the socket,
/// else the `gitdir=` records of its own journal,
/// name a directory that hashes to the instance,
/// and the same rule the sweep and `doctor` use decides whether it is gone.
/// A worktree on a device that is simply not mounted is served,
/// and so is one neither the note nor the journal names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ownWorktreeState(instance, journal) // { gone: gitDir } | { served: gitDir } | 'unmapped'
/// ```
pub fn own_worktree_state(instance: &str, journal: &Path) -> OwnWorktree {
    let Some(runtime_dir) = non_empty_path_variable("XDG_RUNTIME_DIR") else { return OwnWorktree::Unmapped };
    let state: Option<PathBuf> = state_dir_of_journal(journal, instance);
    let folders: UnitFolders<'_> = UnitFolders { runtime_dir: runtime_dir.as_path(), state_dir: state.as_deref() };
    return match worktree_of(&folders, instance) {
        UnitWorktree::Unmapped => OwnWorktree::Unmapped,
        UnitWorktree::Known(git_dir) if worktree_is_gone(git_dir.as_path()) => OwnWorktree::Gone(git_dir),
        UnitWorktree::Known(git_dir) => OwnWorktree::Served(git_dir),
    };
}

/// What:
///  Asks one instance's service to finish its batch and exit,
///  over its own address,
///  and removes what a killed service left once its process is proven gone.
/// Why:
///   Without a service manager the only thing that can stop a service is the service:
/// the message it already understands is the one a replacement sends.
/// A service that answers removes its own socket and its own note as it ends;
/// one that was killed is proven gone by its note
/// (`service_is_gone_after_a_stop.rs`),
/// and only then are the files it left removed.
/// A socket nothing answers on is not an error:
/// there is no service to stop.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await stopOne(dir, instance); // { asked } | { nothing-listens } then unlink what is stale
/// ```
pub fn stop_units(folders: &UnitFolders<'_>, instance: &str) -> Result<(), String> {
    let socket: PathBuf = socket_path(folders.runtime_dir, instance);
    if answered(socket.as_path()) {
        // The service finishes its batch, removes its own files and exits.
        crate::service_client_listen_and_ask_to_retire::ask_to_retire(socket.as_path());
        return Ok(());
    }
    // Nothing serves here; what is left is a file, and it goes when no process of the instance is left.
    remove_what_a_stopped_instance_left(folders.runtime_dir, instance);
    return Ok(());
}

/// What:
///  Removes the socket file, the note and the claim file of an instance that is gone.
/// Why:
///   A killed service leaves them,
/// and a file that stayed would count the instance again and the next sweep stop it again.
/// The socket file goes only when nothing answers on it:
/// a service that still runs owns it.
///  A claim file belongs to the same set: the takeover it was made for is one this worktree can no longer make, and a
/// worktree that is gone needs nothing of it (`service_address_takeover_claim.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (!serviceIsGone(dir, instance) || accepts(socket)) return; unlink(socket); unlink(note); unlink(claimPathOf(socket));
/// ```
fn remove_what_a_stopped_instance_left(runtime_dir: &Path, instance: &str) {
    let directory: PathBuf = runtime_dir.join(SOCKET_DIRECTORY);
    let socket: PathBuf = socket_path(runtime_dir, instance);
    // What: The note's death and the kernel's refusal are both required before either file goes.
    // Why:  A note can be stale while a server is alive (a service whose note another process removed, or one killed and
    //       started again between the two), and the address is then the only proof that a process holds it: the review of
    //       53726ade2d showed a live service whose note was gone, and unlinking its address would have left it unreachable.
    if !service_is_gone(directory.as_path(), instance) || crate::service_server_start_by_the_wrapper::accepts(socket.as_path()) {
        return;
    }
    let claim: PathBuf = crate::service_address_takeover_claim::path(socket.as_path());
    let _removed = std::fs::remove_file(socket);
    let _also = std::fs::remove_file(claim);
    remove_pid_note(pid_note_path(directory.as_path(), instance).as_path());
}

/// What:
/// Asks one instance's service to leave,
/// ends the pushes it kept for a repository that is gone,
/// and keeps the journal,
/// the note and the socket file for the sweep that follows this process's end,
/// when the worktree of that instance is gone,
/// and nothing when it is not.
/// Why:
///   This is what a service of a gone worktree does about itself:
/// it serves nothing,
/// and no connection starts it again;
/// the retry plan of every branch of the gone repository ends
/// (`service_plans_of_a_gone_repository.rs`),
/// so a service started later has no try left to spend;
/// nothing of the journal is removed here,
/// because this process is still one of that journal's writers,
/// so `doctor` counts the instance until the sweep removes it,
/// which is the price of never taking a record this process may still publish.
/// Only the socket file is removed,
/// and the service's own end is what ends its process:
/// asking itself to retire from inside a request would end the batch it is finishing.
/// The file goes first so no connection reaches this service again.
/// The journal is the one file this process may still write,
/// so its removal waits for a sweep that proves the service gone,
/// and the files that name the instance stay until then.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await stopOwnSocketOfAGoneWorktree(folders, instance) // the Git directory, or undefined
/// ```
pub fn stop_own_socket_of_a_gone_worktree(folders: &UnitFolders<'_>, instance: &str) -> Option<PathBuf> {
    let gone: PathBuf = own_worktree_gone(folders, instance)?;
    stop_own_socket(folders, instance);
    // What: The pushes this service kept for the repository that is gone end here.
    // Why:  Nothing is left to deliver, and a plan that stayed would be read by a service of this instance started later, which
    //       would spend a try on a directory nothing can reach: the owner's "nothing should ever wait forever or indefinitely".
    //       A plan of a repository that is still there, and every other record, stays and keeps the journal.
    if let Some(state) = folders.state_dir {
        let journal: PathBuf = journal_of(state.join(SERVICE_UNIT.current).as_path(), instance);
        let ended: Vec<String> = crate::service_plans_of_a_gone_repository::end_plans_of_gone_repositories(journal.as_path());
        if !ended.is_empty() {
            crate::service_log!(
                "the pushes this commit service kept for {} end without another try, because its repository is gone: {}; nothing is left there for a try to reach",
                gone.display(),
                ended.join(", ")
            );
        }
    }
    // Nothing of the journal is removed here, because this process is one of its writers and still runs:
    //  the task that runs the after-commit work writes the note that names a repository to look at and the plan of a push
    //  to try again, on a thread of its own and from jobs of the blocking pool, and both may write between the listing that
    //  says the journal holds nothing unresolved and the removal of the whole tree.
    // Removing it there took a record published in that window: the durable record of a pending push, of a repository still
    //  to look at, or of a batch a client whose answer was lost still reads.
    // So the journal, the note and the socket file stay for the sweep that runs after this process has ended and the manager
    //  reports its service gone (`stop_units_of_gone_worktrees`), and what the journal holds is still named here.
    match work_a_person_may_want(folders.state_dir, instance) {
        Some(kept) => crate::service_warn!(
            "the journal of this commit service is kept at {}, because it holds {}, {}; look at it and remove it yourself if nothing in it is wanted",
            kept.journal.display(),
            kept.entry,
            kept.reason
        ),
        // Landed records a client may still read, and the landing record of nothing in flight, ask nothing of a person:
        //  the sweep removes them together with the files that name this instance once this process is gone.
        None => crate::service_log!(
            "the journal of this commit service is kept at {} for the sweep that follows this process's end, because this service is still one of its writers",
            journal_folder_in_words(folders.state_dir, instance)
        ),
    }
    return Some(gone);
}

/// The journal folder of an instance in words,
/// for a line that names where the journal a service leaves is kept.
fn journal_folder_in_words(state_dir: Option<&Path>, instance: &str) -> String {
    let Some(journal) = journal_of_the_instance(state_dir, instance) else {
        return String::from("a folder this process was given no state directory for");
    };
    return journal.display().to_string();
}

/// What:
///  Removes the socket file and the note of one instance this process serves,
///  and only while the note still names this process.
/// Why:
///  A note that names another process is another server's, and its address is not this process's to unlink:
///  the review of 53726ade2d found a service's exit unlinking a later server's address and note by their paths.
/// A note that cannot be read names nobody, so nothing is removed and the sweep finishes the instance later.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (readNote(note)?.pid !== process.pid) return; unlink(socket); unlink(note)
/// ```
fn stop_own_socket(folders: &UnitFolders<'_>, instance: &str) {
    use crate::service_server_pid_note_of_an_instance::{is_still_the_process, read as read_note};
    let directory: PathBuf = folders.runtime_dir.join(SOCKET_DIRECTORY);
    let note: PathBuf = pid_note_path(directory.as_path(), instance);
    let mine: bool = matches!(read_note(note.as_path()), Some(server) if server.pid == i32::try_from(std::process::id()).unwrap_or(0) && is_still_the_process(&server));
    if !mine {
        return;
    }
    let _removed = std::fs::remove_file(socket_path(folders.runtime_dir, instance));
    remove_pid_note(note.as_path());
}

/// What:
///  Leaves the address of a service whose worktree is gone unused,
///  with the line that says why,
///  and says whether the worktree is gone.
/// Why:
///   A removal Git ran with the wrapper replaced (`worktree remove --force`,
/// `worktree prune`),
/// and a worktree deleted without Git,
/// leave a socket file for a directory that is gone,
/// and a service has nothing to commit to there.
/// Nothing else asks,
/// so the service asks itself:
/// when it starts,
/// and while it serves (`service_server.rs`).
/// It removes its own socket file and its own note,
/// which is what it can do from inside itself,
/// and the sweep removes the journal after this process has ended.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// stopOwnSocketWhenTheWorktreeIsGone(instance, journal) // true: gone, so serve nothing
/// ```
pub fn stop_own_socket_when_the_worktree_is_gone(instance: &str, journal: &Path) -> bool {
    let Some(runtime_dir) = non_empty_path_variable("XDG_RUNTIME_DIR") else { return false };
    let state: Option<PathBuf> = state_dir_of_journal(journal, instance);
    let folders: UnitFolders<'_> = UnitFolders { runtime_dir: runtime_dir.as_path(), state_dir: state.as_deref() };
    let OwnWorktree::Gone(gone) = own_worktree_state(instance, journal) else { return false };
    stop_own_socket(&folders, instance);
    crate::service_warn!(
        "the worktree of this commit service is gone ({}), so it serves nothing and its address is removed; the sweep removes the journal it leaves once this process has ended",
        gone.display()
    );
    return true;
}

/// What:
///  Stops the service of every instance whose worktree is gone,
///  and removes the note and the socket file of each one that was stopped,
///  and its journal when it holds nothing a person may still want and no process serves it.
/// Why:
///   An instance that maps to nothing,
/// or whose worktree is still there,
/// is never asked about.
/// A journal kept for a client that may still read it,
/// or for a service whose process is still there,
/// keeps the note and the socket file,
/// so the next sweep tries the instance again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { stopped, failed, survey } = await stopUnitsOfGoneWorktrees(folders);
/// ```
pub fn stop_units_of_gone_worktrees(folders: &UnitFolders<'_>) -> SweepOutcome {
    let surveyed: Survey = survey(folders);
    let mut outcome: SweepOutcome = SweepOutcome { survey: surveyed.clone(), ..SweepOutcome::default() };
    let directory: PathBuf = folders.runtime_dir.join(SOCKET_DIRECTORY);
    for instance in &surveyed.gone {
        match stop_units(folders, instance.as_str()) {
            Ok(()) => {
                // The service is asked to leave, and its own note says when its process is gone; only then may the journal it
                //  wrote be removed.
                let quiet: bool = service_is_gone(directory.as_path(), instance.as_str());
                match remove_journal_of_a_stopped_instance(folders.state_dir, instance.as_str(), quiet) {
                    Some(JournalAfterStop::HoldsWork(kept)) => {
                        forget_stopped_instance(folders.runtime_dir, instance.as_str());
                        outcome.journals_kept.push((instance.clone(), kept));
                    }
                    // The journal is kept for the clients that may still read it, or for a service that is still going:
                    //  the note and the socket file stay, so the next sweep finds the instance and finishes.
                    Some(JournalAfterStop::StillRead(_)) => {}
                    Some(JournalAfterStop::Removed) | None => forget_stopped_instance(folders.runtime_dir, instance.as_str()),
                }
                outcome.stopped.push(instance.clone());
            }
            Err(reason) => {
                warn(TAG, format!("the service of {instance} was not stopped: {reason}").as_str());
                outcome.failed.push((instance.clone(), reason));
            }
        }
    }
    return outcome;
}

/// Controls of the sweep and of a service's own files stay out of the release executable.
#[cfg(test)]
#[path = "service_units_of_removed_worktrees_tests.rs"]
mod tests;
