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
use crate::process_identity::process_birth_identity;
/// The one identity spelling the commit service's marker uses,
/// so one reader judges both.
use crate::service_index_identity::{BOOT_SEPARATOR, current_pid_scope, own_identity};
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{Path, PathBuf};

/// The folder below a repository's common Git directory that holds the records.
pub(crate) const HOLDER_FOLDER: &str = "cli-git-lock-holders";

/// The first line of every record.
pub const RECORD_HEADER: &str = "monochromatic-git-policy-cli lock-holder-record v1";

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
///  Add the line for `pid` to the remembered repository's record of what this process started.
/// Why:
///   Called where a Git child is started and nowhere else,
///  so the account names every process that can take a lock of this repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// noteChildStarted(child.pid)
/// ```
pub fn note_child_started(pid: u32) {
    let Some(common_dir) = REPOSITORY.get() else {
        return;
    };
    record_a_child_started(common_dir.as_path(), pid);
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
///  The record file this process writes,
///  named by its process number and the digest of its identity.
/// Why:
///   A number alone names a different process after a reuse,
///  so the file a new process writes cannot be mistaken for the file of the process that had its number before.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// join(folder, `${pid}.${digest(identity)}`)
/// ```
fn record_file_of_this_process(folder: &Path) -> Option<PathBuf> {
    let identity: String = own_identity().ok()?;
    let digest: String = crate::service_instance::digest_of_bytes(identity.as_bytes());
    return Some(folder.join(format!("{}.{digest}", std::process::id())));
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
    let folder: PathBuf = folder_of(common_dir);
    let Some(path) = record_file_of_this_process(folder.as_path()) else {
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
    record_this_process(common_dir.as_path());
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
pub fn record_this_process(common_dir: &Path) {
    let folder: PathBuf = folder_of(common_dir);
    let _noted = note(folder.as_path(), SELF_KIND, std::process::id());
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
pub fn record_a_child_started(common_dir: &Path, pid: u32) {
    let folder: PathBuf = folder_of(common_dir);
    let _noted = note(folder.as_path(), CHILD_KIND, pid);
}

/// What:
///  Add the line for `pid` to this process's record,
///  making the file and its header first when they are not there,
///  and making the folder again once when it went between the two.
/// Why:
///   One record per process keeps the folder small enough to read whole,
///  and an append of one short line is all a Git start costs.
///  The pass that sweeps a repository's records removes the folder with the last record it held,
///  so a folder read as present can be gone before the file is made in it;
///  one more try closes that window,
///  and a second failure leaves the record unwritten,
///  which keeps a lock rather than clearing it on an account that is not there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// appendFileSync(file, recordLine(kind, pid, identity)) // the folder made again once when it went
/// ```
fn note(folder: &Path, kind: &str, pid: u32) -> std::io::Result<()> {
    let Some(path) = record_file_of_this_process(folder) else {
        return Err(std::io::Error::other("the identity of this process is unavailable"));
    };
    let Some(identity) = identity_of_process(i64::from(pid)) else {
        return Err(std::io::Error::other("the identity of that process is unavailable"));
    };
    let mut made_again: bool = false;
    loop {
        match write_a_record_line_once(folder, path.as_path(), kind, pid, identity.as_str()) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound && !made_again => {
                made_again = true;
                continue;
            }
            other => return other,
        }
    }
}

/// What:
///  One attempt at making the record when it is not there and appending the line when it is.
///       `pub(crate)` so a control can make this attempt fail the way a folder removed under it does.
/// Why:
///   `note` makes this attempt twice, and the second one is the folder being made again;
///  a `NotFound` here is the folder or the file going between the look and the write,
///  which is the one failure the second attempt can answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (!existsSync(file)) { mkdirSync(folder, { recursive: true }); writeFileSync(file, text, { flag: 'wx' }) } else { appendFileSync(file, line) }
/// ```
pub(crate) fn write_a_record_line_once(folder: &Path, path: &Path, kind: &str, pid: u32, identity: &str) -> std::io::Result<()> {
    use std::io::Write as _;
    if !path.exists() {
        std::fs::create_dir_all(folder)?;
        let mut made: std::fs::File = match std::fs::OpenOptions::new().write(true).create_new(true).open(path) {
            Ok(file) => file,
            // Another thread of this process made it: the header is there and the append below is enough.
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => std::fs::OpenOptions::new().append(true).open(path)?,
            Err(error) => return Err(error),
        };
        made.write_all(record_text(&[(kind, i64::from(pid), identity.to_owned())]).as_bytes())?;
        return Ok(());
    }
    let mut file: std::fs::File = std::fs::OpenOptions::new().append(true).open(path)?;
    file.write_all(record_line(kind, i64::from(pid), identity).as_bytes())?;
    return Ok(());
}
