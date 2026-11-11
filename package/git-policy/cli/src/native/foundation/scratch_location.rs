//! What:
//!  Where the wrapper keeps short-lived private files:
//!  the per-user scratch directory (`<state>/monochromatic-git-policy-cli/scratch`) for work that has no repository
//!       in hand,
//!  and the repository scratch directory (`<common-dir>/monochromatic-git-policy-cli-scratch`) for work inside one;
//!  the folder an earlier release made under its own name (`cli-git-scratch`) is swept by the same rules.
//!  Both are checked as private (a real directory of this user,
//! mode `0700`) and swept of entries
//!       older than a day each time they are prepared,
//!  except for a commit's own tree:
//! a commit makes its tree and collects nothing,
//! and the resident service collects beside its landing loop
//!  (`service_repository_scratch_sweep.rs`).
//! Why:
//!  The system temporary directory is never used (issue 1809,
//! repo rule XTD):
//!  which device backs it is unknowable (a small tmpfs,
//! a per-user quota,
//! a slow disk),
//!       and a full one broke every session.
//!  A file left by a killed process must not accumulate without bound,
//!       so every preparation of a directory also collects what is stale in it.
//!  `doc/planning/cli-git-scratch-locations.md` records the choice per site.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const scratch = join(env.XDG_STATE_HOME ?? join(env.HOME, '.local/state'), 'monochromatic-git-policy-cli', 'scratch');
//! await mkdir(scratch, { recursive: true, mode: 0o700 });
//! await sweepOlderThan(scratch, 24 * 60 * 60 * 1000);
//! ```

/// The environment lookup shared by the wrapper.
use crate::child_environment::environment_value;
/// Debug lines for problems that never fail a command.
use crate::diagnostic_log::debug;
/// The private creation primitives shared by every durable record.
use crate::private_storage::{create_private_directory, create_private_directory_all};
/// Removing one scratch entry within its ceiling,
/// and which commit scratch trees a later command takes for abandoned.
use crate::scratch_entry_removal::{Removal, SCRATCH_REMOVAL_CEILING, remove_within};
/// The mode a private directory ends with,
/// and the call that sets it,
/// which only Unix has.
#[cfg(unix)]
use crate::private_storage::{PRIVATE_DIRECTORY_MODE, protect_path};
/// The names the wrapper keeps on disk.
use crate::wrapper_names_old_and_new::REPOSITORY_SCRATCH;
/// OS strings,
/// paths and time.
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// The tag of this module's diagnostics.
const TAG: &str = "monochromatic-git-policy-cli";

/// The entry in a Git common directory that holds repository scratch.
pub const REPOSITORY_SCRATCH_NAME: &str = REPOSITORY_SCRATCH.current;

/// How old an entry must be before a preparation removes it:
///  far longer than any command of the wrapper runs,
/// so a live process never loses a file.
/// Measured:
/// the longest command in the
///  commit timing log (1,051 commits,
/// 2026-11-02T09:01:37Z to 2026-11-03T15:10:12Z) took 750.6 s and the longest worktree add in the
///  add timing log (48 adds) 539.7 s;
/// a day is about a hundred times the longest.
/// The patience for an owner of another namespace
///  (`FOREIGN_OWNER_PATIENCE`,
/// twice `MERGE_LOCK_CEILING`) is far under a day.
/// A wait for a live program's lock ends at a ceiling:
/// one minute (`LANDING_LOCK_CEILING`,
/// `MERGE_LOCK_CEILING`)
///  or ten for the service's own hold (`SERVICE_HOLD_CEILING`,
/// `wait_ceilings.rs`),
/// so no command that waits on one comes near this age.
///  Kept as the coordinator's pick under the owner's delegation,
/// open to the owner's veto (`doc/planning/cli-git-shipped-limits-audit.md`,
/// "Sizes and ages of the service's own housekeeping").
pub const SCRATCH_STALE_AFTER: Duration = Duration::from_secs(24 * 60 * 60);

/// How many times in the stale age a command that waits renews the modification time of its scratch directory.
const REFRESHES_PER_STALE_AGE: u64 = 24;

/// How often a command that waits renews its scratch directory:
/// a twenty-fourth of `SCRATCH_STALE_AFTER`,
/// so a sweep that
///  runs at any moment finds a waiter's directory at most an hour old,
/// and a directory whose waiter died stops being renewed
///  and ages out as before.
/// The renewal is one `utimensat` call an hour.
pub const SCRATCH_REFRESH_EVERY: Duration = Duration::from_secs(SCRATCH_STALE_AFTER.as_secs() / REFRESHES_PER_STALE_AGE);

/// What:
///  Sets the modification time of `path` to now.
/// Why:
///   The sweep removes entries by modification time,
/// which says nothing about whether the command that made the entry still runs;
/// renewing the time while the command lives ties the entry's age to the
///  command's liveness,
/// with no lock and no record of the process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await utimes(path, new Date(), new Date())
/// ```
pub fn renew(path: &Path) {
    // The modification time is set only when the file opened; an open failure is the result as it was.
    let renewed: std::io::Result<()> = match std::fs::File::open(path) {
        Ok(opened) => opened.set_modified(SystemTime::now()),
        Err(error) => Err(error),
    };
    if let Err(error) = renewed {
        debug(TAG, format!("could not renew {}: {error}", path.display()).as_str());
    }
}

/// What:
///  The per-user state directory of the wrapper:
///  `monochromatic-git-policy-cli` under an absolute `XDG_STATE_HOME`,
///  or under `.local/state` in the home directory.
/// Why:
///   The injected environment decides the location,
///  never a fixed path;
///  a relative `XDG_STATE_HOME` is ignored as the XDG rules say.
///  It mirrors the cache directory of the version cache.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const base = isAbsolute(env.XDG_STATE_HOME ?? '') ? env.XDG_STATE_HOME : env.HOME && join(env.HOME, '.local/state');
/// ```
pub fn state_directory(environment: &[(OsString, OsString)]) -> Option<PathBuf> {
    if let Some(state_home) = environment_value(environment, "XDG_STATE_HOME") {
        let state_path: PathBuf = PathBuf::from(state_home);
        if state_path.is_absolute() {
            return Some(state_path.join("monochromatic-git-policy-cli"));
        }
    }
    let home: OsString = environment_value(environment, "HOME")?;
    if home.is_empty() {
        return None;
    }
    return Some(PathBuf::from(home).join(".local").join("state").join("monochromatic-git-policy-cli"));
}

/// What:
///  The per-user scratch directory's path,
///  or the sentence that says what the environment lacks.
///  Nothing is created.
/// Why:
///   Callers that run without a repository learn the location first and create it only when they need it,
///  so a command with nothing to write never touches the disk.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function userScratchPath(env): string; // throws string
/// ```
pub fn user_scratch_path(environment: &[(OsString, OsString)]) -> Result<PathBuf, String> {
    let Some(state) = state_directory(environment) else {
        return Err(String::from(
            "monochromatic-git-policy-cli keeps its temporary files in a private directory under XDG_STATE_HOME (or HOME/.local/state) and \
             never in the system temporary directory, and this command's environment has neither an absolute \
             XDG_STATE_HOME nor a non-empty HOME; set one of them",
        ));
    };
    return Ok(state.join("scratch"));
}

/// What:
///  Create the per-user scratch directory with its missing parents,
///  check that it is private,
///  sweep its stale entries,
///  and return it.
/// Why:
///   The linter and the push probe write there;
///  the check refuses a directory another account could write.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function prepareUserScratch(env): Promise<string>; // throws string
/// ```
pub fn prepare_user_scratch(environment: &[(OsString, OsString)]) -> Result<PathBuf, String> {
    let given: PathBuf = user_scratch_path(environment)?;
    // A relative home would name another place once a child changes directory.
    // A path that cannot be made absolute stays as given.
    let directory: PathBuf = match std::path::absolute(&given) {
        Ok(absolute) => absolute,
        Err(_refused) => given.clone(),
    };
    if let Err(error) = create_private_directory_all(directory.as_path()) {
        return Err(format!(
            "monochromatic-git-policy-cli could not create its private scratch directory {}: {error}",
            directory.display()
        ));
    }
    if let Err(error) = assert_private_directory(directory.as_path()) {
        return Err(format!(
            "monochromatic-git-policy-cli will not use {} for temporary files: {error}",
            directory.display()
        ));
    }
    sweep_stale(directory.as_path(), SystemTime::now(), SCRATCH_STALE_AFTER, SCRATCH_REMOVAL_CEILING);
    return Ok(directory);
}

/// What:
///  An anonymous file (no name in the directory) in the per-user scratch directory of `environment`,
///  open for reading and writing.
/// Why:
///   Child output goes to a file so that no reader thread can outlive a stopped child;
///  the file has no name from the start (an unnamed temporary file where the system has them,
///  otherwise a name removed at once),
///  so a killed process leaves nothing to collect,
///  and the system temporary directory is never used.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const handle = await open(await prepareUserScratch(env), 'wx+'); // unlinked at once
/// ```
pub fn scratch_file(environment: &[(OsString, OsString)]) -> std::io::Result<std::fs::File> {
    let directory: PathBuf = prepare_user_scratch(environment).map_err(std::io::Error::other)?;
    let file: std::fs::File = tempfile::tempfile_in(directory)?;
    append_only(&file)?;
    return Ok(file);
}

/// What:
///  Puts the file in append mode:
/// every write goes to the end of the file,
/// whatever offset the writer last left.
/// Why:
///   The callers give clones of the file to a child as its standard error,
/// and the child and any helper it starts share one
///  offset:
/// a write at a position another process has moved past overwrites that process's line (measured:
/// a leader's line
///  was lost in 5 of 150 runs of a leader and a helper writing to one shared file at a 20% CPU quota,
/// and in none of 300 runs in
///  append mode,
/// `process/output_capture.rs`).
/// The mode belongs to the open file,
/// so every clone has it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// fcntl(fd, F_SETFL, fcntl(fd, F_GETFL) | O_APPEND);
/// ```
fn append_only(file: &std::fs::File) -> std::io::Result<()> {
    use std::os::unix::io::AsRawFd;
    // SAFETY: the descriptor belongs to `file`, which outlives both calls; `F_GETFL` and `F_SETFL` take no pointers.
    let flags: libc::c_int = unsafe { libc::fcntl(file.as_raw_fd(), libc::F_GETFL) };
    if flags < 0 {
        return Err(std::io::Error::last_os_error());
    }
    // SAFETY: as above.
    if unsafe { libc::fcntl(file.as_raw_fd(), libc::F_SETFL, flags | libc::O_APPEND) } < 0 {
        return Err(std::io::Error::last_os_error());
    }
    return Ok(());
}

/// What:
///  The commit's scratch tree,
///  made in the repository scratch directory the sweep covers,
///  or in the Git directory when that directory cannot be used,
///  and returns its path.
/// Why:
///   A tree in the repository scratch directory is collected after a day even when the command that made it is killed at
///  any instruction and no service ever holds it,
///  which a tree in the Git directory is not.
///  The collection is the resident service's,
///  run beside its landing loop,
///  and never this command's:
///  the listing and the question to the file system per entry cost a commit 13,482 ms of 48 on 2026-11-11
///  (`doc/planning/cli-git-commit-latency-under-many-agents.md`,
///  "The causes,
/// ranked by the time they cost").
///  A repository whose scratch directory cannot be made private still commits,
///  into the Git directory,
///  and says so in the debug log:
///  a commit must not fail over where its scratch files go.
///  A leftover of this command's own earlier attempt is moved aside before it is removed,
///  and the removal is given only the aside's name,
///  so a slow device cannot fail a retry and a removal that outlives its ceiling cannot take the tree the retry makes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const tree = prepareCommitScratch(commonDir, gitDir, name);
/// ```
pub fn prepare_commit_scratch(common_dir: &Path, git_dir: &Path, name: &str, ceiling: Duration) -> std::io::Result<PathBuf> {
    let covered: Option<PathBuf> = match prepare_repository_scratch_for_a_commit(common_dir) {
        Ok(directory) => Some(directory),
        Err(error) => {
            debug(
                TAG,
                format!(
                    "the repository scratch directory of {} cannot be used ({error}), so this commit's scratch tree goes in the Git \
                     directory, where nothing sweeps it if this command is killed; repair that directory to have the sweep cover it",
                    common_dir.display()
                )
                .as_str(),
            );
            None
        }
    };
    let directory: PathBuf = match covered {
        Some(found) => found,
        None => git_dir.to_path_buf(),
    };
    let mut tree: PathBuf = directory.join(name);
    // A tree of this command's own earlier attempt is removed first, so every attempt of one command starts from an empty one:
    //  the name carries this command's identity and no other command makes it.
    let present: bool = std::fs::symlink_metadata(tree.as_path()).is_ok();
    if present {
        // The old tree is moved aside before anything removes it, and only the aside's name is given to the removal: a removal
        //  that outlives its ceiling keeps the pathname it was given and never learns that the tree moved, so one handed this
        //  attempt's name could empty the tree this attempt makes next. The aside's name is one no attempt ever makes.
        let aside: PathBuf = directory.join(format!("{name}.removing-{}", crate::random_id::random_uuid().unwrap_or_default()));
        match std::fs::rename(tree.as_path(), aside.as_path()) {
            Ok(()) => {
                // A control checks which name the removal below is given (`scratch_location_removal_targets_test_support.rs`).
                #[cfg(test)]
                test_targets::note(aside.as_path());
                match remove_within(aside.as_path(), ceiling) {
                    Removal::Finished => {}
                    Removal::Unfinished => debug(
                        TAG,
                        format!(
                            "the leftover {} could not be removed within {ceiling:?} and stays at {}, where the sweep of a later command collects it by age",
                            tree.display(),
                            aside.display()
                        )
                        .as_str(),
                    ),
                    Removal::Failed(error) => debug(
                        TAG,
                        format!("the leftover was moved to {} and could not be removed: {error}", aside.display()).as_str(),
                    ),
                }
            }
            Err(error) => {
                // The name cannot be moved aside, so this attempt cannot have it while a removal of the old tree may still run:
                //  the old tree is removed where it is, and an attempt that cannot wait for that takes a name of its own instead,
                //  which no removal was ever given and which the wrapper's scratch rules accept.
                debug(TAG, format!("the leftover {} cannot be moved aside: {error}", tree.display()).as_str());
                #[cfg(test)]
                test_targets::note(tree.as_path());
                if !remove_within(tree.as_path(), ceiling).finished() {
                    let fresh: PathBuf = directory.join(format!("{name}-again-{}", crate::random_id::random_uuid().unwrap_or_default()));
                    debug(
                        TAG,
                        format!("the leftover {} is still being removed, so this attempt makes {} instead", tree.display(), fresh.display()).as_str(),
                    );
                    tree = fresh;
                }
            }
        }
    }
    create_private_directory(tree.as_path())?;
    assert_private_directory(tree.as_path())?;
    return Ok(tree);
}

/// What:
///  Create the repository scratch directory beside the common Git directory's other private storage,
///  check that it is private,
///  sweep its stale entries,
///  and return it.
/// Why:
///   Git input for a commit transaction is written there:
///  it is on the repository's own device,
///  needs no environment variable,
///  and is collected with the repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function prepareRepositoryScratch(commonDir: string): Promise<string>;
/// ```
pub fn prepare_repository_scratch(common_dir: &Path) -> std::io::Result<PathBuf> {
    let directory: PathBuf = prepare_repository_scratch_for_a_commit(common_dir)?;
    sweep_repository_scratch(common_dir);
    return Ok(directory);
}

/// What:
///  Create the repository scratch directory beside the common Git directory's other private storage,
///  check that it is private,
///  and return it,
///  collecting nothing.
/// Why:
///   A commit makes its own tree and never reads what a collection answered,
///  so the listing and the question to the file system per entry are free work on its path:
///  they were 13,482 ms of the 48 wrapped commits of 2026-11-11T12:33:40Z to 13:15:27Z,
///  4,775 ms in one
///  (`doc/planning/cli-git-commit-latency-under-many-agents.md`,
///  "The causes,
/// ranked by the time they cost").
///  The resident service collects beside its landing loop
///  (`service_repository_scratch_sweep.rs`),
///  so nothing a command killed at any instruction left is lost,
///  and no commit waits for the collection.
///  Every other caller keeps the collecting form above.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function prepareRepositoryScratchForACommit(commonDir: string): Promise<string>; // no sweep
/// ```
pub fn prepare_repository_scratch_for_a_commit(common_dir: &Path) -> std::io::Result<PathBuf> {
    let directory: PathBuf = common_dir.join(REPOSITORY_SCRATCH_NAME);
    match create_private_directory(directory.as_path()) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(error) => return Err(error),
    }
    assert_private_directory(directory.as_path())?;
    return Ok(directory);
}

/// What:
///  Collect what earlier commands left in the repository scratch directory,
///  and in the folder an earlier release made under its own name,
///  and answer how many entries went.
/// Why:
///   A process killed between making a scratch file and removing it leaves the file behind,
///  and nothing else would ever collect it.
///  The work is housekeeping no command reads,
/// so it is a job of its own that a command may run
///  off its own path
///  (`service_repository_scratch_sweep.rs`).
///  A directory that is not there collects nothing and is no failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const removed = sweepRepositoryScratch(commonDir);
/// ```
pub fn sweep_repository_scratch(common_dir: &Path) -> usize {
    let directory: PathBuf = common_dir.join(REPOSITORY_SCRATCH_NAME);
    let removed: usize = sweep_stale(directory.as_path(), SystemTime::now(), SCRATCH_STALE_AFTER, SCRATCH_REMOVAL_CEILING);
    sweep_earlier_repository_scratch(common_dir);
    return removed;
}

/// What:
///  Sweep the repository scratch folder an earlier release made (`cli-git-scratch`) by the rules of this release's own:
/// it must be a
///  real directory of this user (a looser mode is tightened),
/// entries older than `SCRATCH_STALE_AFTER` go,
/// and everything else
///  stays,
/// including entries dated in the future and the folder itself.
/// Nothing is made under the earlier name,
/// and a folder
///  that fails the check is left alone and logged,
/// because the command that asked for scratch must not fail over a folder it
///  does not use.
/// Why:
///   The folder is in the Git directory the repository scratch is in,
/// the earlier release's commands wrote there and
///  nothing else would ever collect what a killed one left (owner,
/// 2026-11-07,
/// "Pair the names":
/// one set of rules,
/// and nothing
///  deleted that those rules would keep).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await sweepStale(await privateDirectoryOrNothing(join(commonDir, REPOSITORY_SCRATCH.earlier)), STALE_AFTER)
/// ```
fn sweep_earlier_repository_scratch(common_dir: &Path) {
    let earlier: PathBuf = common_dir.join(REPOSITORY_SCRATCH.earlier);
    match std::fs::symlink_metadata(&earlier) {
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return,
        Err(error) => {
            debug(TAG, format!("could not inspect {}: {error}", earlier.display()).as_str());
            return;
        }
    }
    if let Err(error) = assert_private_directory(earlier.as_path()) {
        debug(TAG, format!("will not sweep {}: {error}", earlier.display()).as_str());
        return;
    }
    sweep_stale(earlier.as_path(), SystemTime::now(), SCRATCH_STALE_AFTER, SCRATCH_REMOVAL_CEILING);
}

/// What:
///  Accept `path` only if it is a real directory (not a link) of this user that no one else can enter,
///  tightening a looser mode of a directory this user owns.
/// Why:
///   Another account that could write into the directory could plant a link or read the files,
///  and a link in place of the directory could redirect every write.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const s = await lstat(path); if (!s.isDirectory() || s.uid !== process.geteuid()) throw ...; if (s.mode & 0o077) await chmod(path, 0o700);
/// ```
#[cfg(unix)]
fn assert_private_directory(path: &Path) -> std::io::Result<()> {
    // `MetadataExt` gives the owner and the mode bits.
    use std::os::unix::fs::MetadataExt;
    let metadata: std::fs::Metadata = std::fs::symlink_metadata(path)?;
    if !metadata.is_dir() {
        return Err(std::io::Error::other("it is a link or a file, not a directory"));
    }
    // SAFETY: `geteuid` has no preconditions.
    let owner: u32 = unsafe { libc::geteuid() };
    if metadata.uid() != owner {
        return Err(std::io::Error::other("it belongs to another account"));
    }
    if metadata.mode() & 0o077 != 0 {
        protect_path(path, PRIVATE_DIRECTORY_MODE)?;
    }
    return Ok(());
}

/// What:
///  Accept `path` only if it is a real directory.
/// Why:
///   Ownership and modes are Unix notions;
///  other platforms keep their inherited access rules.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (!(await lstat(path)).isDirectory()) throw ...;
/// ```
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(unix))]
fn assert_private_directory(path: &Path) -> std::io::Result<()> {
    if !std::fs::symlink_metadata(path)?.is_dir() {
        return Err(std::io::Error::other("it is a link or a file, not a directory"));
    }
    return Ok(());
}

/// The order two stale entries are removed in:
/// the older one first,
/// so a machine that keeps making them loses the oldest.
fn older_first(left: &(SystemTime, PathBuf), right: &(SystemTime, PathBuf)) -> std::cmp::Ordering {
    return left.0.cmp(&right.0);
}

/// What:
///  Remove every entry of `directory` last modified more than `max_age` before `now`,
///  within `ceiling` for each entry,
///  and return how many it removed.
///  A link is removed itself and never followed;
///  a directory is removed with its contents.
/// Why:
///   A process killed between making a scratch file and removing it leaves the file behind,
///  and nothing else would ever collect it.
///  The directory belongs to the wrapper alone,
///  so every entry in it is the wrapper's;
///  an entry modified in the future is kept,
///  because a wrong clock must not delete live files.
///  Each removal goes through `remove_within`,
///  so an entry whose deletion stalls costs the caller `ceiling` and no more,
///  and what did not finish stays for a later sweep.
///  Which entry may go is `service_scratch_abandonment::may_go`'s to decide,
///  the same decision the collection a preparation does makes,
///  so a tree whose owner still runs is kept however old it is.
///  At most `SCRATCH_TREES_REMOVED_PER_COMMAND` entries go per call,
///  oldest first,
///  so a backlog left by killed commands delays one command by that many removals and never by its whole length.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stale = (await readdir(dir)).filter(mayGo).sort(byAge).slice(0, 8);
/// for (const entry of stale) await removeWithin(entry, ceiling);
/// ```
pub fn sweep_stale(directory: &Path, now: SystemTime, max_age: Duration, ceiling: Duration) -> usize {
    let entries: std::fs::ReadDir = match std::fs::read_dir(directory) {
        Ok(found) => found,
        Err(error) => {
            debug(TAG, format!("could not list {} for stale entries: {error}", directory.display()).as_str());
            return 0;
        }
    };
    // What may go, in the order it goes: the oldest first, so the entries a machine keeps making are the ones that survive.
    let mut stale: Vec<(SystemTime, PathBuf)> = Vec::new();
    for listed in entries {
        let entry: std::fs::DirEntry = match listed {
            Ok(found) => found,
            Err(error) => {
                debug(TAG, format!("could not read an entry of {}: {error}", directory.display()).as_str());
                continue;
            }
        };
        let path: PathBuf = entry.path();
        let metadata: std::fs::Metadata = match std::fs::symlink_metadata(&path) {
            Ok(found) => found,
            Err(error) => {
                debug(TAG, format!("could not inspect {}: {error}", path.display()).as_str());
                continue;
            }
        };
        // One decision, shared with the collection a preparation does: a commit tree stays while the command that made it runs,
        //  however old it is, and every other entry of the place goes by its age.
        if !crate::service_scratch_abandonment::may_go(path.as_path(), &metadata, now, max_age) {
            continue;
        }
        stale.push((metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH), path));
    }
    stale.sort_by(older_first);
    let mut removed: usize = 0;
    for (_, path) in stale.iter().take(crate::service_scratch_abandonment::SCRATCH_TREES_REMOVED_PER_COMMAND) {
        // The removal is bounded like every other one: a device that stalls costs this call the ceiling and leaves the entry to the
        //  next sweep, instead of holding the command that asked for scratch for as long as the device takes.
        match remove_within(path.as_path(), ceiling) {
            Removal::Finished => removed += 1,
            Removal::Unfinished => debug(TAG, format!("the stale {} was still being removed after {ceiling:?}", path.display()).as_str()),
            Removal::Failed(error) => debug(TAG, format!("could not remove the stale {}: {error}", path.display()).as_str()),
        }
    }
    return removed;
}

/// The names a removal of a leftover was handed,
/// for the controls that prove it never names the tree an attempt makes.
#[cfg(test)]
#[path = "scratch_location_removal_targets_test_support.rs"]
pub(crate) mod test_targets;

/// Environment resolution,
/// preparation,
/// privacy checks and the sweep are controlled in the test module.
#[cfg(test)]
#[path = "scratch_location_tests.rs"]
mod tests;
