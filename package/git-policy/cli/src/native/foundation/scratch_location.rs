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
//! What a folder must be before anything is written in it,
//! and the sweep that removes its stale entries,
//! are in `a_private_scratch_folder_and_the_stale_entries_a_sweep_removes.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const scratch = join(env.XDG_STATE_HOME ?? join(env.HOME, '.local/state'), 'monochromatic-git-policy-cli', 'scratch');
//! await mkdir(scratch, { recursive: true, mode: 0o700 });
//! await sweepOlderThan(scratch, 24 * 60 * 60 * 1000);
//! ```

/// The tag this module's diagnostic lines carry,
/// which the sweep's file defines.
use crate::a_private_scratch_folder_and_the_stale_entries_a_sweep_removes::TAG;
/// The check that a folder is this account's private directory,
/// which every preparation of a folder it did not make makes.
pub(crate) use crate::a_private_scratch_folder_and_the_stale_entries_a_sweep_removes::assert_private_directory;
/// What any scratch folder must be and what a sweep takes out of it:
/// how old an entry must be,
/// and the sweep that removes the stale ones.
pub use crate::a_private_scratch_folder_and_the_stale_entries_a_sweep_removes::{
    SCRATCH_STALE_AFTER,
    sweep_stale,
};
/// The environment lookup shared by the wrapper.
use crate::child_environment::environment_value;
/// Debug lines for problems that never fail a command.
use crate::diagnostic_log::debug;
/// The private creation primitives shared by every durable record.
use crate::private_storage::{
    create_private_directory,
    create_private_directory_all,
};
/// Removing one scratch entry within its ceiling,
/// and which commit scratch trees a later command takes for abandoned.
use crate::scratch_entry_removal::{
    Removal,
    SCRATCH_REMOVAL_CEILING,
    remove_within,
};
/// The names the wrapper keeps on disk.
use crate::wrapper_names_old_and_new::REPOSITORY_SCRATCH;
/// OS strings,
/// paths and time.
use std::ffi::OsString;
use std::path::{
    Path,
    PathBuf,
};
use std::time::{
    Duration,
    SystemTime,
};
/// The entry in a Git common directory that holds repository scratch.
pub const REPOSITORY_SCRATCH_NAME: &str = REPOSITORY_SCRATCH.current;
/// How many times in the stale age a command that waits renews the modification time of its scratch directory.
const REFRESHES_PER_STALE_AGE: u64 = 24;

/// How often a command that waits renews its scratch directory:
/// a twenty-fourth of `SCRATCH_STALE_AFTER`,
/// so a sweep that
///  runs at any moment finds a waiter's directory at most an hour old,
/// and a directory whose waiter died stops being renewed
///  and ages out as before.
/// The renewal is one `utimensat` call an hour.
pub const SCRATCH_REFRESH_EVERY: Duration =
    Duration::from_secs(SCRATCH_STALE_AFTER.as_secs() / REFRESHES_PER_STALE_AGE);

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
    // The open is the non-blocking one (`crate::plain_file::open_without_blocking`): a scratch entry may be a directory, and a
    //  name that holds a named pipe answers at once instead of waiting for a writer that may never come.
    let renewed: std::io::Result<()> = match crate::plain_file::open_without_blocking(path) {
        Ok(opened) => opened.set_modified(SystemTime::now()),
        Err(error) => Err(error),
    };
    if let Err(error) = renewed {
        debug(
            TAG,
            format!("could not renew {}: {error}", path.display()).as_str(),
        );
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
    return Some(
        PathBuf::from(home)
            .join(".local")
            .join("state")
            .join("monochromatic-git-policy-cli"),
    );
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
    sweep_stale(
        directory.as_path(),
        SystemTime::now(),
        SCRATCH_STALE_AFTER,
        SCRATCH_REMOVAL_CEILING,
    );
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
pub fn prepare_commit_scratch(
    common_dir: &Path,
    git_dir: &Path,
    name: &str,
    ceiling: Duration,
) -> std::io::Result<PathBuf> {
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
    // What:
    //  The tree this attempt makes, from the one call that also proves it.
    // Why:
    //   `mkdir` gives the new directory to this account with mode `0700` less the umask,
    //  so a metadata call after a creation of this process cannot change the answer,
    //  and it is one round trip to the device that a commit waits for before it can answer.
    //  The audit of 2026-11-12 charged 8,594 ms of one 9,892 ms commit to this preparation,
    //  and a device that stalls a metadata call is what its slowest requests met
    //  (`doc/planning/cli-git-commit-latency-under-many-agents.md`).
    //  The proof stays for a path that was there already,
    //  which is another process's until the leftover handling below has taken it away.
    match create_private_directory(tree.as_path()) {
        // A directory this call made, which no other process could have made under this name.
        Ok(()) => return Ok(tree),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(error) => return Err(error),
    }
    // A tree of this command's own earlier attempt is removed first, so every attempt of one command starts from an empty one:
    //  the name carries this command's identity and no other command makes it,
    //  and the creation above found it there.
    {
        // The old tree is moved aside before anything removes it, and only the aside's name is given to the removal: a removal
        //  that outlives its ceiling keeps the pathname it was given and never learns that the tree moved, so one handed this
        //  attempt's name could empty the tree this attempt makes next. The aside's name is one no attempt ever makes.
        let aside: PathBuf = directory.join(format!(
            "{name}.removing-{}",
            crate::random_id::random_uuid().unwrap_or_default()
        ));
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
                debug(
                    TAG,
                    format!(
                        "the leftover {} cannot be moved aside: {error}",
                        tree.display()
                    )
                    .as_str(),
                );
                #[cfg(test)]
                test_targets::note(tree.as_path());
                if !remove_within(tree.as_path(), ceiling).finished() {
                    let fresh: PathBuf = directory.join(format!(
                        "{name}-again-{}",
                        crate::random_id::random_uuid().unwrap_or_default()
                    ));
                    debug(
                        TAG,
                        format!("the leftover {} is still being removed, so this attempt makes {} instead", tree.display(), fresh.display()).as_str(),
                    );
                    tree = fresh;
                }
            }
        }
    }
    // The name is this attempt's now, because the leftover that held it is gone, and the creation is its own proof.
    create_private_directory(tree.as_path())?;
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
    proved_or_made_private(directory.as_path())?;
    return Ok(directory);
}

/// What:
///  Prove `directory` is this account's private directory,
///  making it first when it is not there,
///  in the fewest round trips to the device that hold the proof.
/// Why:
///  A commit writes the private index and the conclusion copies inside this directory,
///  so it must be this account's and closed to other accounts before anything of the commit is written there,
///  and a directory that was there already may be another account's or carry a loosened mode.
///  A directory this call makes needs no proof after it:
///  `mkdir` gives the new directory to this account with mode `0700` less the umask,
///  so the proof would be a round trip the device answers with what the call already did.
///  The proof comes first and the creation only when there is nothing to prove,
///  because a proof of the directory found there is what a commit waits for when it exists,
///  which is every commit but the first.
///  The audit of 2026-11-12 charged 8,594 ms of one 9,892 ms commit to this preparation,
///  and each metadata call is a round trip a stalled device can hold for seconds
///  (`doc/planning/cli-git-commit-latency-under-many-agents.md`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// proveOrMakePrivate(directory); // throws when another account owns it
/// ```
fn proved_or_made_private(directory: &Path) -> std::io::Result<()> {
    match assert_private_directory(directory) {
        Ok(()) => return Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    match create_private_directory(directory) {
        Ok(()) => return Ok(()),
        // Another process made it between the proof and this creation, so it is proved as any directory found there is.
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            return assert_private_directory(directory);
        }
        Err(error) => return Err(error),
    }
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
    let removed: usize = sweep_stale(
        directory.as_path(),
        SystemTime::now(),
        SCRATCH_STALE_AFTER,
        SCRATCH_REMOVAL_CEILING,
    );
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
            debug(
                TAG,
                format!("could not inspect {}: {error}", earlier.display()).as_str(),
            );
            return;
        }
    }
    if let Err(error) = assert_private_directory(earlier.as_path()) {
        debug(
            TAG,
            format!("will not sweep {}: {error}", earlier.display()).as_str(),
        );
        return;
    }
    sweep_stale(
        earlier.as_path(),
        SystemTime::now(),
        SCRATCH_STALE_AFTER,
        SCRATCH_REMOVAL_CEILING,
    );
}

/// The names a removal of a leftover was handed,
/// for the controls that prove it never names the tree an attempt makes.
#[cfg(test)]
#[path = "scratch_location_removal_targets_test_support.rs"]
pub(crate) mod test_targets;

/// The paths whose privacy a preparation proved,
/// for the control that it proves the directory it did not make and not the tree it did.
#[cfg(test)]
#[path = "scratch_location_privacy_proofs_test_support.rs"]
pub(crate) mod test_proofs;

/// Environment resolution,
/// preparation,
/// privacy checks and the sweep are controlled in the test module.
#[cfg(test)]
#[path = "scratch_location_tests.rs"]
mod tests;
