//! What:
//!  The record the wrapper writes about the processes it starts in a repository:
//!  one file per wrapper process
//!  naming that process and every Git it started,
//!  each by a birth identity that also names the boot,
//!  the PID namespace and the machine.
//! Why:
//!  A Git lock file names its holder only by a process number
//!  (`index~pid.lock`, written because the wrapper turns `core.lockfilePid` on),
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
//!  and only the same boot, namespace and machine lets the birth identity decide.
//!  The file is written before the child can take any lock,
//!  so a kill the wrapper cannot catch leaves it behind;
//!  it is removed by the pass that finds every holder it names gone and no lock of the repository standing in
//!  their names.
//!  The reads of these files are in `lock_holder_record_reads.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! rememberRepository(commonDir); recordThisProcess(commonDir); noteChildStarted(child.pid);
//! ```

/// The identity of this process:
/// its birth,
/// the boot,
/// the PID namespace and the machine.
use crate::claim_time::{current_boot_id, current_machine_id, is_machine_id};
/// The birth identity the platform reports for a process.
use crate::process_identity::process_birth_identity;
/// The one identity spelling the commit service's marker uses, so one reader judges both.
use crate::service_index_identity::{BOOT_SEPARATOR, current_pid_scope, own_identity};
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{Path, PathBuf};

/// The folder below a repository's common Git directory that holds the records.
pub(crate) const HOLDER_FOLDER: &str = "cli-git-lock-holders";

/// The first line of every record.
pub const RECORD_HEADER: &str = "monochromatic-git-policy-cli lock-holder-record v1";

/// What a line naming the writing process begins with.
pub(crate) const SELF_LINE: &str = "self ";

/// What a line naming a Git the writing process started begins with.
pub(crate) const CHILD_LINE: &str = "child ";

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
///   Called once, before the first Git of the command starts;
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
///  Whether this process is the one that wrote its own record,
///  claimed once per process.
/// Why:
///   A command may pass the lock pass twice (the client, then a forward that resolved its own Git directory),
///  and the second pass must neither add a second line nor read the record again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// let claimed = false; const first = !claimed && (claimed = true);
/// ```
fn the_self_record_is_this_processs_once() -> bool {
    static CLAIMED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    return !CLAIMED.swap(true, std::sync::atomic::Ordering::SeqCst);
}

/// What:
///  Whether this call is the one that may write this process's own record.
/// Why:
///   The first caller records,
///  and every later one returns at once,
///  because a record names a process once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (!claimSelfRecord()) return;
/// ```
pub fn the_self_record_is_this_processs() -> bool {
    return the_self_record_is_this_processs_once();
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
    let machine: Option<&str> = Some(current_machine_id()).filter(|text| { return is_machine_id(text); });
    let boot: &str = current_boot_id();
    for part in [Some(boot).filter(|text| { return !text.is_empty(); }), scope, machine.filter(|_| { return scope.is_some(); })] {
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
        text.push_str(format!("{kind} {pid} {identity}\n").as_str());
    }
    return text;
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
///  Note in the repository's record that this process runs,
///  creating the record when it is this process's first note.
/// Why:
///   A forwarded command replaces this process with real Git,
///  which keeps the number and the birth identity,
///  so a record written before the replacement names the Git that then takes the lock.
///  A record that cannot be written leaves nothing behind and holds back nothing:
///  the next command then treats the lock as it does today,
///  which is the safe side.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// recordProcess(commonDir) // creates <folder>/<pid>.<digest> with `self <pid> <identity>`
/// ```
pub fn record_this_process(common_dir: &Path) {
    let folder: PathBuf = folder_of(common_dir);
    let _noted = note(folder.as_path(), SELF_LINE, std::process::id());
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
    let _noted = note(folder.as_path(), CHILD_LINE, pid);
}

/// What:
///  Add the line for `pid` to this process's record,
///  making the file and its header first when they are not there.
/// Why:
///   One record per process keeps the folder small enough to read whole,
///  and an append of one short line is all a Git start costs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// appendFileSync(file, `${kind}${pid} ${identity}\n`)
/// ```
fn note(folder: &Path, kind: &str, pid: u32) -> std::io::Result<()> {
    let Some(path) = record_file_of_this_process(folder) else {
        return Err(std::io::Error::other("the identity of this process is unavailable"));
    };
    let Some(identity) = identity_of_process(i64::from(pid)) else {
        return Err(std::io::Error::other("the identity of that process is unavailable"));
    };
    use std::io::Write as _;
    if !path.exists() {
        std::fs::create_dir_all(folder)?;
        let mut made: std::fs::File = match std::fs::OpenOptions::new().write(true).create_new(true).open(path.as_path()) {
            Ok(file) => file,
            // Another thread of this process made it: the header is there and the append below is enough.
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => std::fs::OpenOptions::new().append(true).open(path.as_path())?,
            Err(error) => return Err(error),
        };
        made.write_all(record_text(&[(kind, i64::from(pid), identity)]).as_bytes())?;
        return Ok(());
    }
    let mut file: std::fs::File = std::fs::OpenOptions::new().append(true).open(path.as_path())?;
    file.write_all(format!("{kind}{pid} {identity}\n").as_bytes())?;
    return Ok(());
}
