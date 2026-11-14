//! What:
//!  The toolchain repair:
//! when the launchers copied into a new worktree's `.cache/toolchain-launchers` still name another
//!  checkout's `toolchain.lock.json`,
//! rebuild the worktree's own links and launchers with `mpm-shadow install
//!  --frozen-lockfile`.
//! Why:
//!  mpm-shadow owns `toolchain.plan.jsonc`,
//! `toolchain.lock.json` and the pinned tools.
//! Its exposed links and launcher records
//!  hold absolute paths,
//! so a copy would run the source checkout's tool versions and guard,
//! and a branch that changes the
//!  lock would silently use the source's.
//! The install is lock-constrained:
//! it relinks what is installed and,
//! when this
//!  worktree's lock names tools that are not installed,
//! downloads them.
//! The owner chose that on 2026-11-03 ("Install online
//!  during the add"),
//! knowing an add can then download and can retire tool copies another checkout still links.
//! The summary
//!  line mpm-shadow prints says what was installed.
//! A failed install never fails the add:
//! it prints the reason and
//!  `mise run toolchain:install`.
//! A launcher record whose name the worktree's own lock declares nowhere is a copy of a layout an earlier release
//!  had:
//! no install rewrites it,
//! so the relink removes it instead of reporting a stale record that no install can ever answer,
//! and the success is counted over the records the lock does declare.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const finding = await repairToolchain(destination, environment); // undefined when the launchers already name this worktree
//! ```

use crate::blocking_job::BlockingJobError;
use crate::blocking_jobs_within_the_budget::run_a_job_or_in_turn;
use crate::diagnostic_log::debug;
use crate::package_repair_request::RepairEnvironment;
use crate::package_repair_command::{ToolCommand, ToolLimits, ToolRun, find_program, run_refused, run_tool_awaiting};
use crate::package_repair_report::{Finding, Outcome, Tool, finding_from, missing_tool};
use crate::plain_file::{SMALL_FILE_LIMIT, read_plain_file_through_links};
use std::collections::HashSet;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::time::Instant;

/// The bytes of a `toolchain.lock.json` this repair reads, past which the lock is no lock it can read.
///  Measured on this repository on 2026-11-13: its lock is 63,625 bytes, so this bound is sixteen times it,
/// and a lock that large would declare some three thousand tools.
///  A longer lock is refused rather than read,
/// and a refusal is the answer an unreadable lock already gets:
/// every launcher record stays counted.
const LONGEST_LOCK: u64 = SMALL_FILE_LIMIT;

/// What:
///  The launcher names this worktree's lock declares,
///  which are the names an install may rewrite a record for,
///  or nothing when the lock cannot be read as one carrying names.
/// Why:
///   `mpm-shadow install` writes a launcher and its record for each bin the lock declares and touches nothing else,
///  so a record for any other name is a copy of a layout an earlier release had and no install answers for it.
///   A lock this release cannot read leaves every record something an install answers,
///  which keeps the check as strict as it was before the names were read at all.
///  The lock is read the way the record beside it is: only a plain file is opened and only so many bytes of it are read,
///  so a named pipe at that name holds neither the add's thread nor a job of the service's pool,
///  and a runaway file cannot be read into memory whole.
///  A lock behind a link of the person's own is read, because that is the lock the worktree means.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const declared = (destination) => { const lock = JSON.parse(readPlainFileThroughLinks(join(destination, 'toolchain.lock.json'), SMALL_FILE_LIMIT)); return new Set(Object.values(lock.tools).flatMap((tool) => tool.bins ?? [])); };
/// ```
fn declared_launcher_names(destination: &Path) -> Option<HashSet<String>> {
    let bytes: Vec<u8> = read_plain_file_through_links(destination.join(LOCK_FILE).as_path(), LONGEST_LOCK).ok()?;
    let lock: serde_json::Value = serde_json::from_slice::<serde_json::Value>(bytes.as_slice()).ok()?;
    let tools: &serde_json::Map<String, serde_json::Value> = lock.get("tools")?.as_object()?;
    let mut names: HashSet<String> = HashSet::new();
    for tool in tools.values() {
        for bin in tool.get("bins").and_then(serde_json::Value::as_array).into_iter().flatten() {
            if let Some(name) = bin.as_str() {
                names.insert(name.to_owned());
            }
        }
    }
    return (!names.is_empty()).then_some(names);
}

/// What:
///  The launcher name a record belongs to:
///  its own file name without the record's extension.
/// Why:
///   The launcher beside a record is named exactly so,
///  and the lock declares those names,
///  so this is what tells a record an install rewrites from a copy no install rewrites.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const launcherName = (record) => basename(record, '.launch');
/// ```
fn launcher_name(record: &Path) -> Option<String> {
    return record.file_stem()?.to_str().map(str::to_owned);
}

/// What:
///  This worktree's own `toolchain.lock.json`,
///  as its path is written and as the filesystem resolves it,
///  so a record that names either spelling of it is not a copy of another checkout's.
/// Why:
///  One read of that pair serves every record of a pass,
///  and the removal asks the same question again of the bytes it holds,
///  so the two answers cannot be given by different comparisons.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const own = (destination) => ({ own: join(destination, 'toolchain.lock.json'), real: realpath(destination) + '/toolchain.lock.json' });
/// ```
struct ThisWorktreesLock {
    /// The lock as a record spells it.
    own: PathBuf,
    /// The same lock as the filesystem resolves it, when the worktree resolves at all.
    real: Option<PathBuf>,
}

impl ThisWorktreesLock {
    /// The lock of `destination`.
    fn of(destination: &Path) -> ThisWorktreesLock {
        let real: Option<PathBuf> = match std::fs::canonicalize(destination) {
            Ok(resolved) => Some(resolved.join(LOCK_FILE)),
            Err(_) => None,
        };
        return ThisWorktreesLock { own: destination.join(LOCK_FILE), real };
    }

    /// Whether `record` names a lock other than this worktree's:
    /// `Some(true)` when it does,
    /// `Some(false)` when it names this one,
    /// and nothing when it cannot be read as a record holding a lock.
    fn names_another_lock(&self, record: &Path) -> Option<bool> {
        let named: PathBuf = lock_named_by_record(record)?;
        let mine: bool = named == self.own || Some(&named) == self.real.as_ref();
        return Some(!mine);
    }
}

/// What:
///  A launcher record and the launcher beside it,
///  as one pass classified them:
///  a copy of an earlier release's layout that no install rewrites.
/// Why:
///   The removal proves what it deletes,
///   and a launcher has no bytes that say whether an install wrote it:
///   the object that was beside the record when the pass read the directory is the only thing to compare,
///   because a launcher another command writes is another object under the same name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Copy = { record: string; launcher: string; name?: string; launcherIdentity?: Identity };
/// ```
struct CopyOfAnEarlierLayout {
    /// The record.
    record: PathBuf,
    /// The launcher beside it, whether or not there was one.
    launcher: PathBuf,
    /// The launcher's name the lock is asked about.
    name: Option<String>,
    /// The object that was at the launcher's name.
    launcher_identity: Option<Identity>,
}

/// What:
///  The object a name held:
///  its device and inode, its size, and the moment it was written.
/// Why:
///   A launcher a concurrent install writes is a new object under the same name
///   (`mpm-shadow` places it with a rename),
///   so an object equal in all four is the one the pass classified and no other.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Identity = { device: bigint; inode: bigint; size: bigint; modified: [bigint, bigint] };
/// ```
#[derive(Clone, Copy, Eq, PartialEq)]
struct Identity {
    /// The device the object is on.
    device: u64,
    /// Its inode.
    inode: u64,
    /// Its size in bytes.
    size: u64,
    /// When it was last written, in whole seconds since the epoch.
    modified_seconds: i64,
    /// And the nanoseconds of that moment.
    modified_nanoseconds: i64,
}

/// What:
///  The object `path` names now,
///  or nothing when it is not there.
/// Why:
///   A launcher is compared with the object the pass classified, and an absent one was removed by someone else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const identity = (path) => { const s = statSync(path); return [s.dev, s.ino, s.size, s.mtime]; };
/// ```
fn identity_of(path: &Path) -> Option<Identity> {
    use std::os::unix::fs::MetadataExt as _;
    let found: std::fs::Metadata = std::fs::symlink_metadata(path).ok()?;
    return Some(Identity {
        device: found.dev(),
        inode: found.ino(),
        size: found.size(),
        modified_seconds: found.mtime(),
        modified_nanoseconds: found.mtime_nsec(),
    });
}

/// What:
///  What a worktree's launcher directory holds:
///  the records an install rewrites,
///  and the copies of an earlier release's layout beside them.
/// Why:
///  The two are answered differently:
///  a stale record an install rewrites is a failure when the install leaves it stale,
///  while no install can rewrite a record for a name the lock declares nowhere,
///  so failing over it would report work no tool of this release can do.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Launchers = { stale: string[]; unrewritable: string[] };
/// ```
struct Launchers {
    /// Records naming another lock that the worktree's own lock declares a launcher for.
    stale: Vec<PathBuf>,
    /// Records naming another lock for a name the worktree's own lock declares nowhere, and what they were read beside.
    unrewritable: Vec<CopyOfAnEarlierLayout>,
}

/// What:
///  The two kinds of stale launcher record of `destination`,
/// read from the launcher directory and the lock alone.
/// Why:
///   Both faces of the repair decide the install from this,
///  and the count after the install asks it again,
///  so the rule that tells an install's work from a copy lives in one place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const launchersOf = (destination) => { const declared = declaredLauncherNames(destination); return { stale: namedAnother.filter((r) => declared?.has(launcherName(r)) ?? true), unrewritable: declared ? namedAnother.filter((r) => !declared.has(launcherName(r))) : [] }; };
/// ```
fn launchers_of(destination: &Path) -> Launchers {
    let named_another: Vec<PathBuf> = stale_launch_records(destination);
    let Some(declared) = declared_launcher_names(destination) else {
        return Launchers { stale: named_another, unrewritable: Vec::new() };
    };
    let mut stale: Vec<PathBuf> = Vec::new();
    let mut unrewritable: Vec<CopyOfAnEarlierLayout> = Vec::new();
    for record in named_another {
        let name: Option<String> = launcher_name(record.as_path());
        if name.as_ref().is_some_and(|launcher| return declared.contains(launcher)) {
            stale.push(record);
            continue;
        }
        let launcher: PathBuf = record.with_extension("");
        let launcher_identity: Option<Identity> = identity_of(launcher.as_path());
        unrewritable.push(CopyOfAnEarlierLayout { record, launcher, name, launcher_identity });
    }
    return Launchers { stale, unrewritable };
}

/// What:
///  Remove the launcher records, and the launchers beside them,
///  that this worktree's lock declares nowhere,
///  once the bytes being removed are proved to be that copy and not what another command wrote.
/// Why:
///   They are copies of a layout an earlier release had,
///  carried in by the copy that made the worktree,
///  and they name another checkout's lock file,
///  which is the state this repair exists to end.
///  No install rewrites them and nothing runs them,
///  so leaving them would keep a record that names another checkout in every worktree copied from this one.
///  A record that cannot be removed is said and left:
///  it is no install's work, so the relink neither fails nor waits on it.
///  The classification reads the lock and the directory once,
///  and a person's `toolchain:update` and install can write a correct record and launcher for a name the lock
///  declares after that read:
///  so each record is renamed to a name only this pass writes,
///  read again,
///  and put back when the install declared its name meanwhile or its bytes name this worktree's own lock,
///  and the launcher goes only when it is the object this pass read beside the record.
///  Whatever a kill leaves is a record under a name nothing reads, which the next pass sweeps.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const copy of copies) { const aside = takeAside(copy.record); if (!stillACopy(destination, aside, copy.name)) { putBack(aside, copy.record); continue; } unlink(aside); if (identityOf(copy.launcher) === copy.launcherIdentity) unlink(copy.launcher); }
/// ```
fn remove_what_no_install_rewrites(destination: &Path, copies: &[CopyOfAnEarlierLayout]) {
    // A control's own concurrent `toolchain:update` and install, run once as the removal begins: the window a
    //  classification cannot see lies between its read and the removal, and a control has to stand in it to show what
    //  the pass does when something is written there. Taken out, so it runs once; only a control build has it.
    #[cfg(test)]
    if let Some(write) = WHEN_THE_REMOVAL_BEGINS.with(|cell| return cell.borrow_mut().take()) {
        write();
    }
    sweep_what_a_kill_left(destination);
    let own: ThisWorktreesLock = ThisWorktreesLock::of(destination);
    // The lock as it stands after the classification, read once for the whole removal: a name it declares now is one an
    //  install has just taken on, and a record the pass holds for it may not go. Reading it once per record would read
    //  the whole lock a thousand times in a worktree carrying a thousand copies.
    let declared_now: Option<HashSet<String>> = declared_launcher_names(destination);
    for copy in copies {
        let Some(aside) = take_aside(copy.record.as_path()) else { continue };
        if !still_a_copy_of_an_earlier_layout(aside.as_path(), copy.name.as_deref(), &own, declared_now.as_ref()) {
            // Another command declared this name or wrote its own record there: the bytes are not a copy any more,
            //  and a name that holds a newer file keeps it.
            put_back(aside.as_path(), copy.record.as_path());
            continue;
        }
        match std::fs::remove_file(aside.as_path()) {
            Ok(()) => debug(TAG, format!("{}: removed {}, whose launcher name {} declares nowhere", destination.display(), copy.record.display(), LOCK_FILE).as_str()),
            Err(error) => {
                debug(TAG, format!("{}: {} is no record any install rewrites and it could not be removed ({error})", destination.display(), aside.display()).as_str());
                continue;
            }
        }
        // The launcher beside the record goes with it, and only the object this pass read: an install that wrote its own
        //  launcher meanwhile leaves a different object, which is not this pass's to remove.
        if copy.launcher_identity.is_none() {
            continue;
        }
        if identity_of(copy.launcher.as_path()) != copy.launcher_identity {
            debug(TAG, format!("{}: {} was written by another command and is left", destination.display(), copy.launcher.display()).as_str());
            continue;
        }
        match std::fs::remove_file(copy.launcher.as_path()) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => debug(TAG, format!("{}: {} could not be removed ({error})", destination.display(), copy.launcher.display()).as_str()),
        }
    }
}

/// What:
///  The name this pass reads `record` under once it has taken it out of the launcher directory,
///  or nothing when it could not be taken.
/// Why:
///  The name is ours alone (this process's number and a count),
///  so no other command can write what is read there,
///  which is what makes the deletion provable:
///  whatever is at the record's own name afterwards was written by that other command and is left.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const takeAside = (record) => { const aside = record.withName(`${basename(record)}.removing-${pid}-${counter++}`); return rename(record, aside) ? aside : undefined; };
/// ```
fn take_aside(record: &Path) -> Option<PathBuf> {
    static NEXT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let token: u32 = NEXT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let name: String = format!("{}.removing-{}-{token}", record.file_name()?.to_string_lossy(), std::process::id());
    let aside: PathBuf = record.with_file_name(name);
    return match std::fs::rename(record, aside.as_path()) {
        Ok(()) => Some(aside),
        Err(error) => {
            debug(TAG, format!("{} could not be taken aside ({error}), so it is left", record.display()).as_str());
            None
        }
    };
}

/// What:
///  Put a record this pass took aside back under its name,
///  unless another command has written one there meanwhile.
/// Why:
///   A rename would replace whatever that command wrote;
///   a name taken only when it is free leaves the newer record and drops the copy this pass holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const putBack = (aside, record) => { if (link(aside, record) fails with EEXIST) unlink(aside); else unlink(aside); };
/// ```
fn put_back(aside: &Path, record: &Path) {
    match std::fs::hard_link(aside, record) {
        Ok(()) => {
            let _removed = std::fs::remove_file(aside);
            debug(TAG, format!("{} was written by another command and is left", record.display()).as_str());
        }
        // A record another command wrote is newer than the copy in this pass's hand, which is dropped here.
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            let _removed = std::fs::remove_file(aside);
        }
        Err(error) => debug(TAG, format!("{} could not be put back from {} ({error}); the next pass sweeps it", record.display(), aside.display()).as_str()),
    }
}

/// What:
///  Whether the record this pass took aside still says it is a copy no install rewrites:
///  it names a lock other than this worktree's,
///  and the worktree's lock, as the removal read it, declares no launcher of its name.
/// Why:
///   The record is read again here because it may have changed since the classification,
///   and a record an install has just written for a name the lock declares is not this pass's to remove.
///   A lock that cannot be read keeps the rule the classification used: every record counts as one an install answers,
///  so a copy is still a copy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stillACopy = (destination, aside, name, own) => own.namesAnotherLock(aside) === true && (name === undefined || !(declaredLauncherNames(destination)?.has(name) ?? false));
/// ```
fn still_a_copy_of_an_earlier_layout(aside: &Path, name: Option<&str>, own: &ThisWorktreesLock, declared_now: Option<&HashSet<String>>) -> bool {
    match own.names_another_lock(aside) {
        Some(true) => {}
        Some(false) | None => return false,
    }
    let Some(launcher) = name else { return true };
    return match declared_now {
        Some(declared) => !declared.contains(launcher),
        None => true,
    };
}

/// What:
///  Remove what a killed pass left in the launcher directory:
///  a record under a name this module gives a record it took aside,
///  whose process is gone.
/// Why:
///   A pass killed between taking a record aside and removing or putting it back leaves that name behind,
///   and nothing else would ever read it.
///   A name whose process still runs is a pass in flight and is left.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const entry of readdir(launchers)) if (entry.includes('.removing-') && !existsSync(`/proc/${pidOf(entry)}`)) unlink(entry);
/// ```
fn sweep_what_a_kill_left(destination: &Path) {
    let Ok(listing) = std::fs::read_dir(destination.join(LAUNCHERS)) else { return };
    for item in listing.flatten() {
        let name: std::ffi::OsString = item.file_name();
        let Some(text) = name.to_str() else { continue };
        let Some((_, owner)) = text.rsplit_once(".removing-") else { continue };
        let Some((written_pid, _token)) = owner.split_once('-') else { continue };
        let Ok(pid) = written_pid.parse::<u32>() else { continue };
        if std::path::Path::new(format!("/proc/{pid}").as_str()).exists() {
            continue;
        }
        match std::fs::remove_file(item.path()) {
            Ok(()) => debug(TAG, format!("{}: removed what a killed pass left", item.path().display()).as_str()),
            Err(error) => debug(TAG, format!("{} could not be removed ({error})", item.path().display()).as_str()),
        }
    }
}

/// Tag used for diagnostics from this module.
const TAG: &str = "packageRepairToolchain";

// What a control writes inside the removal pass, taken out when it runs so it runs once, and absent in a release build.
#[cfg(test)]
thread_local! {
    /// The writer a control stands in the classification-to-removal window with.
    pub(crate) static WHEN_THE_REMOVAL_BEGINS: std::cell::RefCell<Option<Box<dyn FnOnce()>>> = const { std::cell::RefCell::new(None) };
}

/// Where mpm-shadow keeps the launchers and their records,
/// relative to a worktree.
const LAUNCHERS: &str = ".cache/toolchain-launchers";

/// The record that sits beside a launcher,
/// named after it.
const RECORD_EXTENSION: &str = "launch";

/// The lock file a launcher record names,
/// relative to a worktree.
const LOCK_FILE: &str = "toolchain.lock.json";

/// The key of a launcher record that names the lock file.
const LOCK_KEY: &str = "lockFile";

/// The program that owns the toolchain,
/// and where a worktree keeps a link to it.
const PROGRAM: &str = "mpm-shadow";

/// Where the worktree's own link to the program sits when the search path does not provide one.
const OWN_LINK: &str = ".cache/toolchain-bin/mpm-shadow";

/// The variable that names the worktree whose toolchain a command acts on.
const ROOT_VARIABLE: &str = "MONOCHROMATIC_WORKTREE_ROOT";

/// The command a person runs to finish the repair:
/// the repository's own task,
/// which runs the same installer.
pub const MANUAL_COMMAND: &str = "mise run toolchain:install";

/// What:
///  The launcher records of a worktree that name a lock file other than the worktree's own.
/// Why:
///   They are what a copy leaves behind;
/// a record naming this worktree's lock is already right and needs no repair.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function staleLaunchRecords(destination: string): string[];
/// ```
pub fn stale_launch_records(destination: &Path) -> Vec<PathBuf> {
    let Ok(listing) = std::fs::read_dir(destination.join(LAUNCHERS)) else { return Vec::new() };
    let own: ThisWorktreesLock = ThisWorktreesLock::of(destination);
    let mut stale: Vec<PathBuf> = Vec::new();
    for item in listing.flatten() {
        let path: PathBuf = item.path();
        let is_record: bool = match path.extension() {
            Some(extension) => extension == RECORD_EXTENSION,
            None => false,
        };
        if !is_record {
            continue;
        }
        match own.names_another_lock(path.as_path()) {
            Some(true) => stale.push(path),
            Some(false) => {}
            None => debug(TAG, format!("{} names no lock file and is left alone", path.display()).as_str()),
        }
    }
    stale.sort();
    return stale;
}

/// What:
///  The lock file a launcher record names,
/// when the record can be read and holds the key.
///  `Option<PathBuf>` is "the path,
/// or no answer".
/// Why:
///   A record that cannot be read,
/// is not JSON or names no lock is left alone,
/// and each step of reading it may fail the same way.
/// Only a regular file is opened:
/// a named pipe with no writer would block the read until a writer came,
/// which is never.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lockNamedByRecord(path: string): string | undefined;
/// ```
fn lock_named_by_record(path: &Path) -> Option<PathBuf> {
    // Only a regular file is read: opening a named pipe that has no writer blocks until a writer comes, and the add
    // would then wait for ever on a record it could not learn anything from.
    let regular: bool = match std::fs::metadata(path) {
        Ok(found) => found.is_file(),
        Err(_) => false,
    };
    if !regular {
        return None;
    }
    let text: String = std::fs::read_to_string(path).ok()?;
    let record: serde_json::Value = serde_json::from_str::<serde_json::Value>(text.as_str()).ok()?;
    let lock: &str = record.get(LOCK_KEY).and_then(serde_json::Value::as_str)?;
    return Some(PathBuf::from(lock));
}

/// What:
///  The command that starts mpm-shadow with the worktree as the toolchain root.
/// Why:
///   An inherited `MONOCHROMATIC_WORKTREE_ROOT` names the checkout the person works in,
/// which is not the new worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function mpmCommand(destination: string, verb: string[]): ToolCommand;
/// ```
fn mpm_command(destination: &Path, arguments: &[&str], environment: &RepairEnvironment) -> ToolCommand {
    let mut set: Vec<(OsString, OsString)> = vec![(OsString::from(ROOT_VARIABLE), destination.as_os_str().to_os_string())];
    set.extend(crate::package_repair_request::with_the_persons_path(environment));
    return ToolCommand { arguments: arguments.iter().map(OsString::from).collect(), directory: PathBuf::from(destination), set };
}

/// What:
///  The part of mpm-shadow's last summary line that says what was installed,
/// such as `3 installed, 62 unchanged, 0 failed`.
/// Why:
///   The person is told what an add installed,
/// in mpm-shadow's own words.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function installSummary(stdout: string): string | undefined;
/// ```
fn install_summary(stdout: &[u8]) -> Option<String> {
    const PREFIX: &str = "toolchain install: ";
    let text: String = String::from_utf8_lossy(stdout).into_owned();
    for line in text.lines().rev() {
        if let Some(rest) = line.strip_prefix(PREFIX)
            && rest.contains("installed")
        {
            return Some(rest.to_owned());
        }
    }
    return None;
}

/// What:
///  One install of the worktree's toolchain,
/// with everything its start and its report need.
/// Why:
///  The launcher records are looked at once,
/// and the two faces of the repair
/// (`repair_toolchain` and `repair_toolchain_awaiting`) then differ only in how
///  they wait for the tool.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Install = { program: string; command: ToolCommand; destination: string; limits: ToolLimits };
/// ```
struct Install {
    /// The mpm-shadow the search path or the worktree's own link resolved.
    program: PathBuf,
    /// What it is started with and where.
    command: ToolCommand,
    /// The worktree whose launchers the install rewrites.
    destination: PathBuf,
    /// The two bounds the tool runs under.
    limits: ToolLimits,
}

/// What a toolchain repair is,
/// once the launcher records have been read.
/// Why:
///  The three answers differ in what they start and report,
/// and the records are read twice
/// (before and after the install),
/// so the
///  first reading decides what the repair is before either face waits.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ToolchainRepair = { kind: 'nothing' } | { kind: 'noProgram'; finding: Finding } | { kind: 'install'; work: Install };
/// ```
enum ToolchainRepair {
    /// Every launcher record names this worktree,
    /// so no tool runs and nothing is reported.
    NothingToDo,
    /// The worktree needs the install and the environment provides no mpm-shadow;
    /// the finding names the command to run.
    NoProgram(Finding),
    /// The install to start.
    Install(Install),
}

/// What:
///  `prepare_install` for the state a job of the blocking pool owns.
/// Why:
///  A job reads only what it was handed,
/// so the worktree and the environment travel together as owned values.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function prepareInstallInAJob([destination, environment]: [string, RepairEnvironment]): ToolchainRepair;
/// ```
fn prepare_install_in_a_job((destination, environment): (PathBuf, RepairEnvironment)) -> ToolchainRepair {
    return prepare_install_on_a_caller_thread(destination.as_path(), &environment);
}

/// What:
///  `prepare_install` as a job of the process's blocking pool.
/// Why:
///  The launcher records are storage work,
/// and the task that polls the three repairs must never do any of it:
/// the toolchain wait shares that task with the pnpm ceiling and the Cargo waits,
///  so a read that waited there would stop
/// their deadlines with it.
/// A refusal from the system runs the same read in turn,
/// under a place of the process's bound on blocking jobs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const decided = await prepareInstallWithinTheBudget(destination, environment);
/// ```
async fn prepare_install_within_the_budget(destination: &Path, environment: &RepairEnvironment) -> Result<ToolchainRepair, BlockingJobError> {
    let state: (PathBuf, RepairEnvironment) = (destination.to_path_buf(), environment.clone());
    return run_a_job_or_in_turn(state, prepare_install_in_a_job).await;
}

/// What:
///  How many launcher records the worktree's lock declares still name another lock file,
/// as a job of the process's blocking pool.
/// Why:
///  The records are read again after the install,
/// on the same task the install was awaited from,
/// so the same rule holds:
/// no storage read of the launcher directory runs on the task that polls the repairs.
///  The count leaves out the copies no install rewrites,
///  because an install that did everything the lock asks of it is not a failed relink.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const left = await countStaleWithinTheBudget(destination);
/// ```
fn count_stale_in_a_job(destination: PathBuf) -> usize {
    return launchers_of(destination.as_path()).stale.len();
}

/// What:
///  `count_stale_in_a_job` awaited on the runtime.
/// Why:
///  The awaiting face has no read of the launcher directory of its own,
/// so a worktree whose records changed under the install is counted without stopping the task's other waits.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const left = await countStaleWithinTheBudget(destination);
/// ```
async fn count_stale_within_the_budget(destination: &Path) -> Result<usize, BlockingJobError> {
    return run_a_job_or_in_turn(destination.to_path_buf(), count_stale_in_a_job).await;
}

/// What:
///  What the toolchain repair of `destination` is,
/// decided from the launcher records and the search path alone,
/// for a caller that is not a task of the runtime.
/// Why:
///  Both faces decide it the same way before they wait,
/// so neither can start a tool the other would not.
///  A launcher record whose name this worktree's lock declares nowhere is removed here,
///  so every face that asks what the worktree needs also leaves it holding nothing another checkout wrote.
/// This face runs on the caller's own thread,
/// which polls no wait of the repair,
/// so its reads need no place of the pool;
/// the awaiting face reads them as a job (`prepare_install_within_the_budget`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function prepareInstall(destination: string, environment: RepairEnvironment): ToolchainRepair;
/// ```
fn prepare_install_on_a_caller_thread(destination: &Path, environment: &RepairEnvironment) -> ToolchainRepair {
    let launchers: Launchers = launchers_of(destination);
    // A copy of an earlier release's layout is removed rather than reported: no install rewrites it, and it names another
    //  checkout's lock until it goes. The removal proves the bytes it deletes again, because a concurrent update and
    //  install can write a correct record and launcher for a name the lock declares between the two.
    remove_what_no_install_rewrites(destination, launchers.unrewritable.as_slice());
    if launchers.stale.is_empty() {
        debug(TAG, format!("{}: every launcher record this worktree's lock declares names this worktree", destination.display()).as_str());
        return ToolchainRepair::NothingToDo;
    }
    let own_link: PathBuf = destination.join(OWN_LINK);
    // The link in the worktree is looked at only when the search path gave no program, as `.or_else(...)` ran it.
    let found_program: Option<PathBuf> = match find_program(PROGRAM, environment.path()) {
        Some(program) => Some(program),
        None => {
            if own_link.is_file() { Some(own_link.clone()) } else { None }
        }
    };
    let Some(program) = found_program else {
        return ToolchainRepair::NoProgram(missing_tool(Tool::Toolchain, PROGRAM, MANUAL_COMMAND));
    };
    return ToolchainRepair::Install(Install { program, command: mpm_command(destination, &["install", "--frozen-lockfile"], environment), destination: PathBuf::from(destination), limits: environment.limits() });
}

/// What:
///  What the worktree needs of the toolchain repair,
/// decided from the launcher records and the search path alone.
/// Why:
///   The add no longer runs the install itself,
/// so it asks this before it writes the record the service runs:
/// a worktree whose launchers already name it leaves no work,
/// and one whose mpm-shadow is missing gets the line that names the command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ToolchainNeed = 'nothing' | 'needed' | { missing: Finding };
/// ```
pub(crate) enum ToolchainNeed {
    /// Every launcher record names this worktree,
    /// so no install is needed.
    Nothing,
    /// The install is needed and a program was found;
    /// running it is the commit service's work.
    Needed,
    /// The install is needed and no mpm-shadow was found;
    /// the finding names the command to run.
    NoProgram(Finding),
}

/// What:
///  What the toolchain repair is,
/// as one of the three answers the add asks about.
/// Why:
///  The install carries the program and the command it built,
/// and the add needs only which of the three states the worktree is in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const needOfPrepared = (prepared) => prepared.kind === 'nothing' ? 'nothing' : prepared.kind === 'install' ? 'needed' : prepared.finding;
/// ```
fn need_of_prepared(prepared: ToolchainRepair) -> ToolchainNeed {
    return match prepared {
        ToolchainRepair::NothingToDo => ToolchainNeed::Nothing,
        ToolchainRepair::NoProgram(finding) => ToolchainNeed::NoProgram(finding),
        ToolchainRepair::Install(_work) => ToolchainNeed::Needed,
    };
}

/// What:
///  `need_of_prepared` on the records of the worktree,
/// read as a job of the process's blocking pool.
/// Why:
///   The add asks this on the task that also runs the Cargo repair,
/// so the read of the launcher directory must never run there;
/// the pool's refusal is handed back so the add can say what it could not decide.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const needWithinTheBudget = async (destination, environment) => needOfPrepared(await prepareInstallWithinTheBudget(destination, environment));
/// ```
pub(crate) async fn need_within_the_budget(destination: &Path, environment: &RepairEnvironment) -> Result<ToolchainNeed, BlockingJobError> {
    return match prepare_install_within_the_budget(destination, environment).await {
        Ok(prepared) => Ok(need_of_prepared(prepared)),
        Err(refusal) => Err(refusal),
    };
}

/// The install,
/// awaited on the runtime the caller runs on.
async fn install_awaiting(work: &Install) -> ToolRun {
    return run_tool_awaiting(work.program.as_path(), &work.command, work.limits).await;
}

/// What the install reports:
/// the line the person reads,
/// and the records that still name another lock file.
/// Why:
///  Both faces report the same run the same way,
/// whatever waited for it,
/// and the read of the records is the caller's:
/// the synchronous face reads them on its caller's thread,
/// and the awaiting face hands over a job of the pool.
/// A success whose records could not be read again is a failure that says so,
/// never a success the wrapper did not check.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const installFinding = (work: Install, run: ToolRun, remaining: number | Refusal) => Finding | undefined;
/// ```
fn install_finding(work: &Install, run: &ToolRun, remaining: Result<usize, BlockingJobError>) -> Option<Finding> {
    debug(TAG, format!("{}: install ended {:?} after {} ms", work.destination.display(), run.end, run.millis).as_str());
    let detail: String = match install_summary(run.stdout.as_slice()) {
        Some(summary) => format!("mpm-shadow rebuilt the toolchain links ({summary})"),
        None => String::from("mpm-shadow rebuilt the toolchain links"),
    };
    let finding: Finding = finding_from(Tool::Toolchain, run, detail.as_str(), MANUAL_COMMAND);
    if !matches!(finding.outcome, Outcome::Repaired { .. }) {
        return Some(finding);
    }
    let left: Option<String> = match remaining {
        Ok(0) => None,
        Ok(count) => Some(format!("{count} launcher records still name another lock file")),
        Err(refusal) => Some(format!("the launcher records could not be read again ({refusal})")),
    };
    return match left {
        None => Some(finding),
        Some(said) => Some(Finding { tool: Tool::Toolchain, outcome: Outcome::Failed { reason: said, run_instead: String::from(MANUAL_COMMAND) } }),
    };
}

/// What:
///  Rebuild the worktree's toolchain links and launchers,
/// when its launcher records are stale.
/// Why:
///   See the module comment:
/// one frozen install,
/// with the network when the lock names tools that are not installed yet.
/// The callers of this face are synchronous (`package_repair.rs` inside the repair the add runs),
///  so it enters the process's runtime once and waits there;
/// from inside the runtime it refuses,
/// and `repair_toolchain_awaiting` is
///  the face such a caller uses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function repairToolchain(destination: string, environment: RepairEnvironment): Finding | undefined;
/// ```
pub fn repair_toolchain(destination: &Path, environment: &RepairEnvironment) -> Option<Finding> {
    return match prepare_install_on_a_caller_thread(destination, environment) {
        ToolchainRepair::NothingToDo => None,
        ToolchainRepair::NoProgram(finding) => Some(finding),
        ToolchainRepair::Install(work) => {
            let started: Instant = Instant::now();
            let run: ToolRun = match crate::lazy_runtime::run_on_runtime(install_awaiting(&work)) {
                Ok(run) => run,
                // No install was made, and the runtime's own reason is what the person is told.
                Err(refusal) => run_refused(work.program.as_path(), started, &refusal),
            };
            install_finding(&work, &run, Ok(launchers_of(work.destination.as_path()).stale.len()))
        }
    };
}

/// What:
///  `repair_toolchain` on the runtime:
/// the install is awaited,
/// never blocked on,
/// and every read of the launcher records is a job of the pool.
/// Why:
///  A caller that is itself a task of the runtime cannot enter it
/// (`RuntimeError::Nested`),
/// so it awaits this face;
/// the finding is the synchronous face's.
/// Nothing of this face's work runs on the task that polls it,
/// so the pnpm and Cargo repairs beside it keep their deadlines
/// whatever the worktree holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function repairToolchainAwaiting(destination: string, environment: RepairEnvironment): Promise<Finding | undefined>;
/// ```
pub async fn repair_toolchain_awaiting(destination: &Path, environment: &RepairEnvironment) -> Option<Finding> {
    return match prepare_install_within_the_budget(destination, environment).await {
        Ok(ToolchainRepair::NothingToDo) => None,
        Ok(ToolchainRepair::NoProgram(finding)) => Some(finding),
        Ok(ToolchainRepair::Install(work)) => {
            let run: ToolRun = install_awaiting(&work).await;
            install_finding(&work, &run, count_stale_within_the_budget(work.destination.as_path()).await)
        }
        // The records were not read, so nothing about the launchers is known; the person is told the reason.
        Err(refusal) => Some(Finding {
            tool: Tool::Toolchain,
            outcome: Outcome::Failed { reason: format!("the launcher records of the worktree could not be read ({refusal})"), run_instead: String::from(MANUAL_COMMAND) },
        }),
    };
}

/// Toolchain repair controls stay out of the release executable.
#[cfg(test)]
#[path = "package_repair_toolchain_tests.rs"]
mod tests;
