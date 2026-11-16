//! What:
//!  The note that says which process is a worktree's commit service:
//! its process number and the birth identity that proves the number still
//! names that process,
//! written by the service once it listens and removed as it ends.
//! Why:
//!   Without a service manager nothing else knows which process serves a worktree.
//! A socket that a killed service left behind refuses a connection and is replaced by
//! the next bind,
//! but a service that stopped inside its own process still holds its socket and answers nothing,
//! and the command that needs an answer has to be able to end it and start a fresh one.
//! A process number alone is not enough:
//! the kernel reuses numbers,
//! and killing a stranger because it inherited the number would be a defect nobody would find.
//! The birth identity is the repository's own answer to that
//! (`process_identity.rs`:
//! the start tick from `/proc/<pid>/stat`,
//! field 22),
//! and every lock in this program already records an owner this way.
//! The note is written after the socket is bound,
//! so a note that exists always means "this process had the socket",
//! and it is written through a temporary file and a rename,
//! so a reader never sees half of it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! writeAtomic(join(instanceDir, `${instance}.pid`), `${process.pid} ${currentBirthIdentity()}\n`);
//! ```

use crate::process_identity::{current_birth_identity, process_birth_identity};
use std::path::{Path, PathBuf};

/// The ending of the note that names an instance's process.
pub const PID_NOTE_ENDING: &str = ".pid";

/// One running commit service,
/// as its note names it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServerProcess {
    /// The process number the service had.
    pub pid: i32,
    /// The birth identity that proves the number still names that process.
    pub identity: String,
}

/// The note of one instance,
/// beside its socket.
pub fn path(socket_directory: &Path, instance: &str) -> PathBuf {
    return socket_directory.join(format!("{instance}{PID_NOTE_ENDING}"));
}

/// What:
///  Writes the complete text of one note into a private file beside it,
///  and gives that file's path.
/// Why:
///   The note is published under its own name by one exclusive link,
/// so its text must be whole before that name exists:
/// a reader that opens the note must never find half of it,
/// which is what a write under the note's own name would let it do.
/// The private file carries this process's number,
/// so two processes writing at once never write into one file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writePrivate(pidNotePath, `${process.pid} ${currentBirthIdentity()}\n`) // the path of the private file
/// ```
fn write_private(note: &Path, pid: i32) -> std::io::Result<PathBuf> {
    let identity: String = match current_birth_identity() {
        Ok(identity) => identity,
        // A platform without an identity source cannot make a note a later command may trust, so none is made.
        Err(unsupported) => return Err(std::io::Error::other(format!("{unsupported:?}"))),
    };
    // The sweep of the claimer's own file is the shared one, so a kill between the write and the link leaves nothing that a later
    //  claim would not collect (`writings_of_dead_writers.rs`, and the eighth review of issue 6293, findings 3 and 7).
    crate::writings_of_dead_writers::remove_writings_of_dead_writers(note);
    let private: PathBuf = crate::writings_of_dead_writers::temporary_file(note, next_temporary_number())?;
    if let Err(refused) = std::fs::write(&private, format!("{pid} {identity}\n")) {
        let _left = std::fs::remove_file(&private);
        return Err(refused);
    }
    return Ok(private);
}

/// A number no other write of this process has taken.
fn next_temporary_number() -> u64 {
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    return COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
}

pub fn claim(note: &Path, pid: i32) -> std::io::Result<()> {
    for _round in 0..2 {
        if let Some(held) = held_by_another_live_process(note) {
            return Err(another_live_service_holds(held.pid));
        }
        run_the_controls_step_before_the_publish();
        let private: PathBuf = write_private(note, pid)?;
        // The one step that decides the owner: the name is given to whoever the kernel lets link it first.
        let published: std::io::Result<()> = std::fs::hard_link(private.as_path(), note);
        let _removed = std::fs::remove_file(private.as_path());
        match published {
            Ok(()) => return Ok(()),
            // What: The note a further claimer would have to replace is taken aside under a name of this claimer's own.
            // Why:  Removing it by path was what let a late claimer delete the note a service had published between this
            //       claimer's look and its act, so two services served one instance (the ninth review of issue 6293, finding 1).
            // A rename gives the note's name to exactly one claimer, and what it took is then read: a live owner's note is put
            //  back and this claim refuses, and a killed service's note is the leftovers the next round publishes over.
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                let aside: PathBuf = taken_aside_name(note, pid);
                if std::fs::rename(note, aside.as_path()).is_ok() {
                    let taken: Option<ServerProcess> = read(aside.as_path());
                    if let Some(held) = taken.filter(|service| { return is_still_the_process(service); }) {
                        // A link puts the live owner's note back, and fails where another claimer published while it was aside:
                        //  that claimer's note is never replaced by this one.
                        let _restored = std::fs::hard_link(aside.as_path(), note);
                        let _removed = std::fs::remove_file(aside.as_path());
                        return Err(another_live_service_holds(held.pid));
                    }
                    // A killed service's leftovers: the next round publishes this claimer's note.
                    let _removed = std::fs::remove_file(aside.as_path());
                }
            }
            Err(refused) => return Err(refused),
        }
    }
    return Err(std::io::Error::other("another process claimed this instance at the same moment, so this one does not claim it"));
}

/// What:
///  The name a claimer takes a note aside under,
///  so that only one claimer can ask what the note says.
/// Why:
///  A rename gives the note's own name to exactly one process, which is the step the kernel arbitrates;
/// two names that differed would let two claimers take the same note, one after the other.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// takenAsideName(note, pid) // `${note}.${pid}.${counter}.taking`
/// ```
fn taken_aside_name(note: &Path, pid: i32) -> PathBuf {
    let number: u64 = next_temporary_number();
    return note.with_file_name(format!("{}.{pid}.{number}.taking", note.file_name().and_then(std::ffi::OsStr::to_str).unwrap_or("note")));
}

/// The refusal that names the process holding an instance.
fn another_live_service_holds(pid: i32) -> std::io::Error {
    return std::io::Error::other(format!(
        "the commit service of this instance is process {pid} and is still that process, so this one does not claim it"
    ));
}

// What:
//  A control's step between the claim's look and its publish, which is where two claimers of one instance can meet.
// Why:
//   No test can make one process's claim land between another's look and its publish by timing alone,
//   and that instant is the whole of what the claim's atomicity is about.
#[cfg(test)]
thread_local! {
    /// What a control runs between the look and the publish,
    /// once.
    static BEFORE_THE_PUBLISH: std::cell::RefCell<Option<Box<dyn FnOnce()>>> = const { std::cell::RefCell::new(None) };
}

/// What:
///  Runs a control's step between the claim's look at the note and its publish.
/// Why:
///   The ordering a control of one owner needs is this instant,
/// and a seam here holds it without any load or timing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// aControlRunsBeforeThePublish(() => { tellTheParent(); waitForTheParent(); });
/// ```
#[cfg(test)]
pub(crate) fn a_control_runs_before_the_publish(step: impl FnOnce() + 'static) {
    BEFORE_THE_PUBLISH.with(|slot| {
        *slot.borrow_mut() = Some(Box::new(step));
    });
}

/// Runs the step a control left for this instant,
/// if one did.
fn run_the_controls_step_before_the_publish() {
    #[cfg(test)]
    BEFORE_THE_PUBLISH.with(|slot| {
        let left: Option<Box<dyn FnOnce()>> = slot.borrow_mut().take();
        if let Some(step) = left {
            step();
        }
    });
}

/// What:
///  The process that holds this instance and is still that process,
/// when it is not this one.
/// Why:
///   A note whose process is gone is a killed service's leftovers,
/// which the next service claims;
/// a note whose process is alive is the service of this instance,
/// and neither a second service nor a bind may take its address away.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// heldByAnotherLiveProcess(note) // the note's server when its pid is not this process's and it is still that process
/// ```
/// Whether another process that is still that process holds the instance the note names,
/// for the one question a bind asks.
pub fn another_live_process_holds(note: &Path) -> bool {
    return held_by_another_live_process(note).is_some();
}

pub fn held_by_another_live_process(note: &Path) -> Option<ServerProcess> {
    let held: ServerProcess = read(note)?;
    if held.pid == i32::try_from(std::process::id()).unwrap_or(0) || !is_still_the_process(&held) {
        return None;
    }
    return Some(held);
}

/// The note as text,
/// or nothing when it is not there.
pub fn read(note: &Path) -> Option<ServerProcess> {
    return parse(std::fs::read_to_string(note).ok()?.as_str());
}

/// What:
///  The server a note's text names,
/// or nothing when the text is not a note this program wrote.
/// Why:
///   A file someone else put at that path must never be a reason to signal a process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// parse(text) // { pid, identity } or undefined
/// ```
pub fn parse(text: &str) -> Option<ServerProcess> {
    let (pid, identity) = text.trim_end().split_once(' ')?;
    return Some(ServerProcess { pid: pid.parse().ok()?, identity: identity.to_string() });
}

/// What:
///  Whether the process a note names is still that process.
/// Why:
///   A note outlives the service when the service was killed,
/// and acting on it then would signal whatever now holds the number.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// stillTheProcess(note) // processBirthIdentity(note.pid) === note.identity
/// ```
pub fn is_still_the_process(server: &ServerProcess) -> bool {
    return matches!(process_birth_identity(i64::from(server.pid)), Ok(Some(identity)) if identity == server.identity);
}

/// Removes the note;
/// a note that is not there is already removed.
pub fn remove(note: &Path) {
    let _removed = std::fs::remove_file(note);
}

/// Controls of the note stay out of the release executable.
#[cfg(test)]
#[path = "service_server_pid_note_of_an_instance_tests.rs"]
mod tests;
