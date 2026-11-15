//! What:
//!  The record the wrapper writes about the processes it starts in a repository:
//!  one file per wrapper process
//!  naming that process and every Git it started,
//!  each by a birth identity that also names the boot,
//!  the PID namespace and the machine.
//! Why:
//!  A Git lock file names its holder only by a process number
//!  (`index~pid.lock`,
//! written because the wrapper turns `core.lockfilePid` on),
//!  and a number proves nothing:
//!  it means a process only inside the namespace that wrote it,
//!  it is reused,
//!  and a Git of a container that shares the repository writes one this process table has never had.
//!  So the wrapper keeps its own account of what it started,
//!  and a later command may act on a dead holder only where that account and this process table agree.
//!  The same identity the commit service's lock marker carries
//!  (`service_index_identity.rs`) is used here,
//!  so one rule judges every holder the wrapper knows:
//!  a process of another boot is gone,
//!  a process of another PID namespace or another machine is unknown and is never cleared,
//!  and only the same boot,
//! namespace and machine lets the birth identity decide.
//!  The file is written immediately before the wrapper's process becomes Git,
//!  and for every Git it starts and waits for,
//!  so a kill the wrapper cannot catch leaves it behind;
//!  it is removed by the pass that finds every holder it names gone and no lock of the repository standing in
//!  their names,
//!  and the folder goes with the last record it held.
//!  The reads of these files are in `lock_holder_record_reads.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! rememberRepository(commonDir); beforeBecomingGit(); noteChildStarted(child.pid);
//! ```

/// The identity of this process:
/// its birth,
/// the boot,
/// the PID namespace and the machine.
use crate::claim_time::{current_boot_id, current_machine_id, is_machine_id};
/// The birth identity the platform reports for a process.
use crate::process_identity::{birth_identity_of_a_process_that_may_have_ended, process_birth_identity};
/// The one identity spelling the commit service's marker uses,
/// so one reader judges both.
use crate::service_index_identity::{BOOT_SEPARATOR, current_pid_scope, own_identity};
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{Path, PathBuf};

/// The folder below a repository's common Git directory that holds the records.
pub(crate) const HOLDER_FOLDER: &str = "cli-git-lock-holders";

/// The first line of every record.
pub const RECORD_HEADER: &str = "monochromatic-git-policy-cli lock-holder-record v1";

/// Tag used for diagnostics from this module.
const TAG: &str = "monochromatic-git-policy-cli";

/// What a line naming the writing process is called.
pub(crate) const SELF_KIND: &str = "self";

/// What a line naming a Git the writing process started is called.
pub(crate) const CHILD_KIND: &str = "child";

/// What:
///  The repository this command works in,
///  remembered so every Git it starts is noted there.
/// Why:
///   The functions that start Git take no repository,
///  and passing one through every forward would touch a dozen signatures for one line of bookkeeping.
///  A wrapper process handles one command,
///  so one value serves it,
///  and a process that remembers none (the commit service) writes no record,
///  whose locks carry the service's own marker instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// let remembered: string | undefined;
/// ```
static REPOSITORY: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();

/// What:
///  Make `common_dir` the repository whose records this process adds to.
/// Why:
///   Called once,
/// before the first Git of the command starts;
///  a second call with another repository changes nothing,
///  because one process never works in two.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// rememberRepository(commonDir) // the first call wins
/// ```
pub fn remember_repository(common_dir: &Path) {
    let _set = REPOSITORY.set(common_dir.to_path_buf());
}

/// Whether a repository is remembered,
/// which a control asks so it knows whether its fixture is armed.
pub fn remembers_a_repository() -> bool {
    return REPOSITORY.get().is_some();
}

/// What:
///  Add the line for `pid` to the remembered repository's record of what this process started,
///  and say whether it was written.
///  `Ok(())` also when this process remembers no repository, which is the commit service's case: it writes no records,
///  and its locks carry its own marker.
/// Why:
///   Called where a Git child is started and nowhere else,
///  so the account names every process that can take a lock of this repository.
///  A record that was not written leaves a lock no proof can explain,
///  so the caller decides what to do about it:
///  a child is stopped before it can take a lock
///  (`forwarding.rs`,
/// `spawn_naming_its_child`),
///  and an error is never dropped
///  (a review of `429503238b`,
/// finding 1).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// noteChildStarted(child.pid) // throws when the line could not be written
/// ```
pub fn note_child_started(pid: u32) -> std::io::Result<()> {
    let Some(common_dir) = REPOSITORY.get() else {
        return Ok(());
    };
    return record_a_child_started(common_dir.as_path(), pid);
}

/// What:
///  The folder a repository's records live in.
/// Why:
///   Every worktree of a repository shares one common Git directory,
///  so a lock in a linked worktree and a record written by a command in the main worktree are found together.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// join(commonDir, 'cli-git-lock-holders')
/// ```
pub fn folder_of(common_dir: &Path) -> PathBuf {
    return common_dir.join(HOLDER_FOLDER);
}

/// What:
///  The identity of `pid` as this host reports it now,
/// or nothing when the platform will not report it.
/// Why:
///   A child is recorded the moment it exists,
///  while its identity is its own;
///  a record naming a birth this table later reads differently is a holder that is gone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const identity = identityOfProcess(child.pid); // `linux:8423337#<boot>#<scope>#<machine>`
/// ```
pub fn identity_of_process(pid: i64) -> Option<String> {
    let birth: String = process_birth_identity(pid).ok().flatten()?;
    return Some(identity_of_birth(birth.as_str()));
}

/// What:
///  The identity text a birth belongs to,
///  ending in the boot,
///  the PID namespace and the machine when they are known.
/// Why:
///   `service_index_identity::holder_state_in` reads exactly this shape,
///  and a part that cannot be read is left out rather than guessed,
///  which makes the judgement unknown instead of wrong.
///  A control that plants a holder builds its text here,
///  so the control and the writer cannot drift apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// identityOfBirth('linux:8423337') // `linux:8423337#<boot>#<scope>#<machine>`
/// ```
pub fn identity_of_birth(birth: &str) -> String {
    let mut identity: String = String::from(birth);
    let scope: Option<&str> = current_pid_scope();
    let machine_text: &str = current_machine_id();
    let known_machine: Option<&str> = match is_machine_id(machine_text) {
        true => Some(machine_text),
        false => None,
    };
    // The machine comes only after the namespace: a reader of a release before machine IDs then sees a namespace it does not
    //  know, which keeps the lock.
    let machine: Option<&str> = match scope {
        Some(_) => known_machine,
        None => None,
    };
    let boot: &str = current_boot_id();
    let boot_part: Option<&str> = match boot.is_empty() {
        true => None,
        false => Some(boot),
    };
    for part in [boot_part, scope, machine] {
        if let Some(text) = part {
            identity.push(BOOT_SEPARATOR);
            identity.push_str(text);
        }
    }
    return identity;
}

/// What:
///  The text of a record naming the given holders,
/// header first.
/// Why:
///   One spelling for the writer and for a control,
///  which plants a record instead of running the process it names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// recordText([['child', 42, identity]]) // header + "child 42 <identity>\n"
/// ```
pub fn record_text(holders: &[(&str, i64, String)]) -> String {
    let mut text: String = format!("{RECORD_HEADER}\n");
    for (kind, pid, identity) in holders {
        text.push_str(record_line(kind, *pid, identity).as_str());
    }
    return text;
}

/// What:
///  One line of a record:
///  the kind,
///  a space,
///  the process number,
///  a space,
///  the identity.
/// Why:
///  The writer and the reader had two spellings:
///  this function wrote `self  <pid>` (a kind that already ended in a space,
/// then another),
///  the append path wrote `child <pid>`,
///  and the reader parsed only the second,
///  so a forwarded command's own record named nobody and could never explain its lock
///  (a review of `675771e759`,
/// finding 1).
///  One formatter serves both writers now,
///  and the reader accepts the two-space form older records carry
///  (`lock_holder_record_reads.rs`,
/// `holders_of`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// recordLine('self', 42, identity) // "self 42 <identity>\n"
/// ```
pub fn record_line(kind: &str, pid: i64, identity: &str) -> String {
    return format!("{kind} {pid} {identity}\n");
}

/// What:
///  The record file this process writes and the identity its own line carries,
///  the file named by its process number and the digest of that identity.
/// Why:
///   A number alone names a different process after a reuse,
///  so the file a new process writes cannot be mistaken for the file of the process that had its number before.
///  The identity comes back with the path because the record's own line is written when the file is made,
///  and it is the one statement of who may append to that file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// [join(folder, `${pid}.${digest(identity)}`), identity]
/// ```
fn record_of_this_process(folder: &Path) -> Option<(PathBuf, String)> {
    let identity: String = own_identity().ok()?;
    let digest: String = crate::service_instance::digest_of_bytes(identity.as_bytes());
    return Some((folder.join(format!("{}.{digest}", std::process::id())), identity));
}

/// What:
///  The record file this process writes in a repository,
///  or nothing when its own identity cannot be read.
/// Why:
///   The pass removes a record only when the process it names as its writer is proved gone,
///  so the process that may remove its own record needs the very path the writer made.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// thisProcessRecordPath(commonDir) // '<common>/cli-git-lock-holders/<pid>.<digest>'
/// ```
pub fn this_process_record_path(common_dir: &Path) -> Option<PathBuf> {
    let folder: PathBuf = folder_of(common_dir);
    return record_of_this_process(folder.as_path()).map(|(path, _)| { return path; });
}

/// What:
///  Whether this process has a record in the repository's folder.
/// Why:
///   A command that started no Git wrote no record,
///  so the sweep it makes when it ends has nothing to decide and costs one look;
///  and a command that wrote one is the one whose record may be litter.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// existsSync(join(folder, `${process.pid}.${digest(ownIdentity)}`))
/// ```
pub fn this_process_has_a_record(common_dir: &Path) -> bool {
    let Some(path) = this_process_record_path(common_dir) else {
        return false;
    };
    return path.exists();
}

/// What:
///  Note in the repository's record that this process runs,
///  which is done just before this process is replaced by real Git.
/// Why:
///   A replaced process keeps the number and the birth identity,
///  so a record written before the replacement names the Git that then takes the lock,
///  and the record is there before that Git can take one.
///  A command that never reaches the replacement changes nothing in the repository,
///  so it leaves no record and makes no folder.
///  A record that cannot be written leaves nothing behind and holds back nothing:
///  the next command then treats the lock as it does today,
///  which is the safe side.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// beforeBecomingGit() // creates <folder>/<pid>.<digest> with `self <pid> <identity>`
/// ```
pub fn record_this_process_before_it_becomes_git() {
    let Some(common_dir) = REPOSITORY.get() else {
        // The commit service works in the repositories it is told about and remembers none: its locks carry its own marker.
        return;
    };
    // What: A record that could not be written is said, and the command the person typed goes on.
    // Why:  The lock this process is about to become Git for may then be one nothing can explain, which costs a later
    //       command a bounded wait; refusing the person's own command over the wrapper's bookkeeping would cost more.
    if let Err(error) = record_this_process(common_dir.as_path()) {
        crate::diagnostic_log::debug(
            TAG,
            format!("this process's lock-holder record could not be written, so a lock it leaves may have no proof: {error}").as_str(),
        );
    }
}

/// What:
///  Note in the repository's record that this process runs,
///  creating the record when it is this process's first note.
/// Why:
///   The caller has already decided that a record of this process belongs in that repository,
///  which only a Git that works there needs.
///  One record per process keeps the folder small enough to read whole,
///  and an append of one short line is all a Git start costs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// recordProcess(commonDir) // creates <folder>/<pid>.<digest> with `self <pid> <identity>`
/// ```
pub fn record_this_process(common_dir: &Path) -> std::io::Result<()> {
    let folder: PathBuf = folder_of(common_dir);
    return note(folder.as_path(), SELF_KIND, std::process::id());
}

/// What:
///  Note in the repository's record that this process started the Git `pid`.
/// Why:
///   The child may take a lock of this repository,
///  and a later command needs to know whose number that is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// recordChild(commonDir, child.pid) // appends `child <pid> <identity>`
/// ```
pub fn record_a_child_started(common_dir: &Path, pid: u32) -> std::io::Result<()> {
    let folder: PathBuf = folder_of(common_dir);
    return note(folder.as_path(), CHILD_KIND, pid);
}

/// What:
///  Add the line for `pid` to this process's record,
///  making the file, its header and this process's own line first when they are not there,
///  and making the folder again once when it went between the two.
/// Why:
///   One record per process keeps the folder small enough to read whole,
///  and an append of one short line is all a Git start costs.
///  The writer's own line is written when the file is made,
///  because it is what says who may append to this file and when nothing can add to it:
///  a process that creates a record for a child it started names itself with the child,
///  so the record can be told from one whose writer is gone.
///  The pass that removes a repository's records removes the folder with the last record it held,
///  so a folder read as present can be gone before the file is made in it;
///  one more try closes that window,
///  and a second failure leaves the record unwritten,
///  which keeps a lock rather than clearing it on an account that is not there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// appendFileSync(file, recordLine(kind, pid, identity)) // the folder and the writer's own line made again once when they went
/// ```
fn note(folder: &Path, kind: &str, pid: u32) -> std::io::Result<()> {
    let Some((path, writer)) = record_of_this_process(folder) else {
        return Err(std::io::Error::other("the identity of this process is unavailable"));
    };
    #[cfg(test)]
    self::hold_before_reading_a_childs_identity::wait_if_a_control_placed_one();
    // What: A child's birth is read whatever state the child is in, so a Git that took a lock and ended before this
    //       reading is still named; this process's own birth is read as the living process it is.
    // Why:  The record is the only proof that a lock's holder is gone, and the window between a child's start and this
    //       reading is enough for it to take a lock and die (a review of `429503238b`, finding 1).
    //       A zombie's record still carries its start tick, and the judgement reads a zombie as gone.
    let birth: Option<String> = match kind == CHILD_KIND {
        true => birth_identity_of_a_process_that_may_have_ended(i64::from(pid)).ok().flatten(),
        false => process_birth_identity(i64::from(pid)).ok().flatten(),
    };
    let Some(found_birth) = birth else {
        return Err(std::io::Error::other("the identity of that process is unavailable"));
    };
    let identity: String = identity_of_birth(found_birth.as_str());
    let mut made_again: bool = false;
    loop {
        match write_a_record_line_once(folder, path.as_path(), kind, pid, identity.as_str(), writer.as_str()) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound && !made_again => {
                made_again = true;
                continue;
            }
            other => return other,
        }
    }
}

/// What:
///  One attempt at making the record when it is not there, with its header and this process's own line, and appending
///  the line when it is.
///       `pub(crate)` so a control can make this attempt fail the way a folder removed under it does.
/// Why:
///   `note` makes this attempt twice, and the second one is the folder being made again;
///  a `NotFound` here is the folder or the file going between the look and the write,
///  which is the one failure the second attempt can answer.
///  The writer's own line is written once, when the file is made:
///  a process's second note appends only its child's line,
///  and the own line of a process asked for again is already the file's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (!existsSync(file)) { mkdirSync(folder, { recursive: true }); writeFileSync(file, header + selfLine + (kind === 'self' ? '' : line), { flag: 'wx' }) } else { appendFileSync(file, line) }
/// ```
pub(crate) fn write_a_record_line_once(folder: &Path, path: &Path, kind: &str, pid: u32, identity: &str, writer: &str) -> std::io::Result<()> {
    use std::io::Write as _;
    // The writer's own line, written when the file is made: it names the one process that appends to this file.
    let own: (i64, String) = (i64::from(std::process::id()), writer.to_owned());
    // The folder the wrapper writes in is its own name inside the Git directory, and a person may have put a file, a link
    //  or a pipe there: a folder that is not a directory is refused rather than followed.
    std::fs::create_dir_all(folder)?;
    if !std::fs::symlink_metadata(folder)?.file_type().is_dir() {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "the record folder is not a directory"));
    }
    // What: Every open of a record is the plain-file open: it cannot wait, follows no link, and refuses a pipe, a device
    //       or a folder at the name.
    // Why:  A named pipe at the record's path held a Git start for good, before the ceiling of the command it served had
    //       even begun (a review of `429503238b`, finding 4).
    let mut made: std::fs::File = match crate::plain_file::create_plain_file(path) {
        Ok(file) => file,
        // Another thread of this process made it, or it is this process's own record already: the append below is enough.
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            // The writer's own line is already the file's, so asking for it again writes nothing.
            if kind == SELF_KIND {
                return Ok(());
            }
            let mut existing: std::fs::File = crate::plain_file::open_plain_file_to_append(path)?;
            return existing.write_all(record_line(kind, i64::from(pid), identity).as_bytes());
        }
        Err(error) => return Err(error),
    };
    made.write_all(record_text(&[(SELF_KIND, own.0, own.1.clone())]).as_bytes())?;
    if kind != SELF_KIND {
        made.write_all(record_line(kind, i64::from(pid), identity).as_bytes())?;
    }
    return Ok(());
}

/// A hold a control places between a child's start and the reading of its identity,
/// so the window a start gate closes can be shown without making a real machine slow.
#[cfg(test)]
pub(crate) mod hold_before_reading_a_childs_identity {
    use std::cell::Cell;
    use std::time::Duration;

    thread_local! {
        /// How long this thread waits before it reads the identity of a process it is naming.
        static HOLD: Cell<Option<Duration>> = const { Cell::new(None) };
    }

    /// What:
    ///  Make this thread wait `hold` before every identity it reads while naming a child,
    ///  until it is given nothing.
    /// Why:
    ///   A child that dies at once is one the parent may be too late to name;
    ///  a control that widens that window shows whether the start held the child until it was named.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// holdBeforeReadingAChildsIdentity(200) // milliseconds
    /// ```
    pub(crate) fn hold(hold: Option<Duration>) {
        HOLD.with(|cell| { cell.set(hold); });
    }

    /// What:
    ///  Wait the time a control placed, if it placed any.
    /// Why:
    ///   The wait is the window: the child runs freely while it lasts.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// if (hold) Atomics.wait(..., hold)
    /// ```
    pub(crate) fn wait_if_a_control_placed_one() {
        if let Some(hold) = HOLD.with(|cell| { return cell.get(); }) {
            std::thread::sleep(hold);
        }
    }
}
