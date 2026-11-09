//! What:
//!  A private copy of the real index,
//!  in its own directory beside it,
//!  removed when
//!       the copy is dropped.
//! Why:
//!  Predicting what `git add` would stage means running it against an index nobody
//!      else reads,
//!  so the real index never changes before the policies have answered.
//!      The copy keeps the real index's access and modification times:
//!  Git re-reads an
//!      entry whose stat data is not older than the index file,
//!  and a copy with a fresh
//!      time would make it trust stat data from the same second as a later edit of the
//!      same size,
//!  so the prediction could miss that edit (issue #544).
//!
//! What a kill leaves,
//! and what removes it:
//!  the directory and its copy,
//!  whose name carries the process that made it and
//!  whose `owner` file carries that process's identity and PID namespace;
//!  every copy made later removes the copies
//!  whose owner is proven gone (see `reap_copies_of_gone_owners`),
//!  and keeps every other one,
//!  because a copy nothing proves abandoned may belong to a command that is still
//!  reading it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // await using index = await createPrivateIndex(realIndexPath); // removed on dispose
//! ```

/// Import the layer's failure type and its closed list of causes.
use crate::candidate_error::{CandidateError, CandidateFailure};
/// `Path`/`PathBuf` are borrowed/owned filesystem paths of raw OS bytes.
use std::path::{Path, PathBuf};
/// The record a copy writes of its owner,
///  and the proof that the owner is gone.
use crate::worktree_copy_owner::{current_copy_owner, token_owner_is_dead};
/// What:
///  `AtomicU64` is a counter that several threads may increase without a lock;
///       `Ordering::Relaxed` asks only that each increase is counted once.
/// Why:
///   Two private indexes made by one process in the same nanosecond still get
///       different directory names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// let created = 0n;
/// ```
use std::sync::atomic::{AtomicU64, Ordering};

/// The name prefix of every private index directory;
///  the installed wrapper uses the same one.
pub const PRIVATE_INDEX_PREFIX: &str = "cli-git-add-policy-";

/// The file inside a private index directory that names the process which made it:
///  its process id,
///  the platform's identity of its birth,
///  and its PID namespace.
/// Why:
///  A copy made later must be able to tell an abandoned directory from one a running
///       command is reading,
///  and a process id alone is reused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const OWNER_RECORD = 'owner';
/// ```
pub const OWNER_RECORD: &str = "owner";

/// The end of the name a copy is built under before one rename publishes it.
/// Why:
///   A copy must hold its owner record before any sweep can see it under the name a sweep visits.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const STAGING_SUFFIX = '.making';
/// ```
pub const STAGING_SUFFIX: &str = ".making";

/// How many unique names this process has made,
///  for private index directories and sibling files.
static CREATED: AtomicU64 = AtomicU64::new(0);

/// What:
///  A name part no other file of this process or another one chose:
///  the process ID,
///       the time in nanoseconds and this process's count,
///  joined by `-`.
/// Why:
///   Private index directories and the direct fix's sibling files are created beside
///       files other processes use,
///  so their names must not collide.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const uniqueSuffix = () => `${process.pid}-${nowNanoseconds()}-${created++}`;
/// ```
pub fn unique_suffix() -> String {
    // What: `.fetch_add(1, ..)` returns the old count and stores one more;
    //       `.as_nanos()` is the time since the epoch in nanoseconds.
    // Why:  Process ID, time and count together name a file no one else chose.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const count = created++;
    // ```
    let count: u64 = CREATED.fetch_add(1, Ordering::Relaxed);
    let nanoseconds: u128 = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
    {
        Ok(elapsed) => elapsed.as_nanos(),
        // A clock before 1970 still leaves the process ID and the count to tell names apart.
        Err(_) => 0,
    };
    return format!("{}-{nanoseconds}-{count}", std::process::id());
}

/// What:
///  A private index file in a directory this value owns.
///  Fields are private,
///  so the
///       directory can be removed only by dropping the value.
/// Why:
///   Every path out of a prediction,
///  failures included,
///  removes the directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class PrivateIndex { readonly #directory: string; readonly #index: string; [Symbol.dispose]() {} }
/// ```
#[derive(Debug)]
pub struct PrivateIndex {
    /// The directory that holds the copy and nothing else of the repository.
    directory: PathBuf,
    /// The private index file Git is pointed at.
    index: PathBuf,
}

/// What:
///  Whether a directory's name is one this release gives a private index copy,
///  including the name a copy
///       is built under before it is published.
/// Why:
///   The sweep visits only these,
/// so it never looks at another kind of directory in the
///       Git directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isCopyName = (name: string) => name.startsWith(PRIVATE_INDEX_PREFIX);
/// ```
fn is_copy_name(name: &str) -> bool {
    return name.starts_with(PRIVATE_INDEX_PREFIX);
}

/// What:
///  Whether the owner of the copy at `directory`,
///  named `name`,
///  is proven gone.
/// Why:
///   Nothing else may remove a copy:
///  a running command's copy that disappeared under it would
///       make that command's prediction fail.
///  An owner record is judged by the proof the whole crate uses
///  (`token_owner_is_dead`:
///  the process's birth identity in this PID namespace,
///  and for an owner of
///       another namespace its record's age),
///  and a directory an earlier release made,
///  which holds no record,
///  is judged by the process id in its name alone.
///  Anything the platform will not report is kept.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ownerIsGone = (path, name) => tokenOwnerIsDead(readOwner(path), path) ?? pidIsGone(pidOfName(name));
/// ```
fn owner_is_proven_gone(directory: &Path, name: &str) -> bool {
    let record: PathBuf = directory.join(OWNER_RECORD);
    match std::fs::symlink_metadata(record.as_path()) {
        // What: A record that exists is the only evidence there is, so one that cannot be read proves nothing.
        // Why:  A corrupt record is not a dead owner; the copy stays and the person can look at it.
        Ok(_) => {
            let Ok(text) = std::fs::read_to_string(record.as_path()) else {
                return false;
            };
            if text.trim().is_empty() {
                return false;
            }
            // `unwrap_or(false)`: a refusal of another namespace's expired record keeps the copy.
            return token_owner_is_dead(Some(text.trim()), directory).unwrap_or(false);
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        // A record that cannot be inspected at all proves nothing either.
        Err(_) => return false,
    }
    // What: A copy with no record is kept, whatever its name says: a process id means something only inside its own
    //       PID namespace, so a live command of another namespace looks exactly like a dead one from here.
    // Why:  Removing a live command's copy makes that command fail while it copies or reads the index, and no rule on
    //       the name can tell the two apart.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return false; // nothing proves this owner dead
    // ```
    let _ = name;
    return false;
}

/// What:
///  Remove the private index copies in `parent` whose owner is proven gone.
/// Why:
///  A killed check,
///  a closed terminal or the OOM killer leaves one copy per invocation for
///       ever otherwise,
///  and they fill the worktree's Git directory until a commit fails.
///  Every copy made later
///       sweeps them,
///  so no separate command or service is needed,
///  and no lock protects the sweep:
///  each
///       directory has one owner and only a proven-dead owner's directory is removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const entry of await readdir(parent)) if (isCopy(entry) && ownerIsGone(entry)) await rm(entry, { recursive: true, force: true });
/// ```
fn reap_copies_of_gone_owners(parent: &Path) {
    let Ok(listed) = std::fs::read_dir(parent) else {
        // A directory that cannot be listed proves nothing about any copy in it.
        return;
    };
    for entry in listed.flatten() {
        let path: PathBuf = entry.path();
        let name: String = entry.file_name().to_string_lossy().into_owned();
        if !is_copy_name(name.as_str()) {
            continue;
        }
        if !path.is_dir() {
            continue;
        }
        if !owner_is_proven_gone(path.as_path(), name.as_str()) {
            continue;
        }
        // `.remove_dir_all` takes the copy and whatever Git wrote beside it; a failure leaves it for the next
        // sweep, and is not a failure of this command.
        let _ = std::fs::remove_dir_all(path.as_path());
    }
}

/// What:
///  Build the failure of a private index that could not be prepared.
///       `&std::io::Error` borrows the operating system's reason.
/// Why:
///   The message names the step and the reason;
///  the directory is under the Git
///       directory,
///  never a candidate pathname.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function privateIndexFailure(step: string, error: Error): CandidateError;
/// ```
fn private_index_failure(step: &str, error: &std::io::Error) -> CandidateError {
    return CandidateError::new(
        CandidateFailure::PrivateIndexUnavailable,
        format!("monochromatic-git-policy-cli could not prepare a private copy of the index: {step}: {error}.").as_str(),
    );
}

/// What:
///  Create a directory only this user can read,
///  failing when it already exists.
/// Why:
///   The copy holds the repository's staged entries;
///  a fresh directory also proves
///       no other process chose the same name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await mkdir(directory, { mode: 0o700 }); // throws EEXIST
/// ```
fn create_private_directory(directory: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        /// `DirBuilderExt` adds `mode`,
        ///  the permissions a new directory is created with.
        use std::os::unix::fs::DirBuilderExt;
        // `.mode(0o700)` configures the temporary builder; `.create` makes the directory.
        return std::fs::DirBuilder::new().mode(0o700).create(directory);
    }
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(not(unix))]
    {
        // Other systems have no mode bits to set; the directory is created as it is.
        return std::fs::DirBuilder::new().create(directory);
    }
}

/// What:
///  Copy the real index into `destination` and give the copy the real index's times.
///       `Ok(false)` means the real index does not exist,
///  which Git reads as an empty index.
/// Why:
///   A missing index needs no copy:
///  Git treats a missing index file as empty,
///  so the
///       private index path can simply name a file that does not exist yet.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function copyIndex(source: string, destination: string): Promise<boolean>;
/// ```
fn copy_index(source: &Path, destination: &Path) -> std::io::Result<bool> {
    // The metadata read answers only whether there is an index to copy: a missing one is an answer, every other error is a
    // failure, and the copy reads what it needs for the mode and the times itself.
    if let Err(error) = std::fs::metadata(source) {
        if error.kind() == std::io::ErrorKind::NotFound {
            return Ok(false);
        }
        return Err(error);
    }
    // What: The one index copy of this crate does this work, so every copy of an index is made, timed and recorded alike.
    // Why:  It keeps the source's mode and times, which Git's racy-entry protection compares with the entry times, and it
    // asks the filesystem to clone the bytes instead of reading them: this copy of a three-megabyte index waited 2,288 ms
    // and 4,813 ms in two commits traced under this host's pressure.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // await copyIndexWithTimestamps(source, destination);
    // ```
    crate::commit_index_copying::copy_index_with_timestamps(source, destination)?;
    return Ok(true);
}

/// What:
///  `impl PrivateIndex { ... }` attaches the constructor and the accessor.
/// Why:
///   The directory is created and filled in one step,
///  so no caller sees a half-made copy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class PrivateIndex { static create(realIndex: string): PrivateIndex; get path(): string }
/// ```
impl PrivateIndex {
    /// What:
    ///  Create the directory beside `real_index` and copy the index into it.
    /// Why:
    ///   Beside the real index keeps the copy on the same filesystem and inside the
    ///       Git directory,
    ///  where the installed wrapper also put it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static async create(realIndex: string): Promise<PrivateIndex>;
    /// ```
    pub fn create(real_index: &Path) -> Result<PrivateIndex, CandidateError> {
        let Some(parent) = real_index.parent() else {
            return Err(CandidateError::new(
                CandidateFailure::PrivateIndexUnavailable,
                "monochromatic-git-policy-cli could not prepare a private copy of the index: Git named an index path without a directory.",
            ));
        };
        // Before a new copy exists, the copies of commands that will never finish leave.
        reap_copies_of_gone_owners(parent);
        let suffix: String = unique_suffix();
        let staging: PathBuf = parent.join(format!("{PRIVATE_INDEX_PREFIX}{suffix}{STAGING_SUFFIX}"));
        if let Err(error) = create_private_directory(staging.as_path()) {
            return Err(private_index_failure("creating its directory", &error));
        }
        // What: The owner record is written while the directory still carries the staging name, and one `rename`
        //       publishes it under the name a sweep visits.
        // Why:  A sweep must never see a copy of this release without its record. A process id read in another PID
        //       namespace would look dead, so a copy with no record can only be kept, and a copy that is only ever
        //       published with its record can never need that guess.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const staging = `${final}.making`; mkdir(staging); writeOwner(staging); rename(staging, final);
        // ```
        let written: std::io::Result<()> = match current_copy_owner() {
            Ok(owner) => std::fs::write(staging.join(OWNER_RECORD), owner.token()),
            // A platform that cannot report this process's identity cannot record one, and the copy is then kept by
            // every later sweep rather than guessed at.
            Err(_unreported) => Ok(()),
        };
        let directory: PathBuf = parent.join(format!("{PRIVATE_INDEX_PREFIX}{suffix}"));
        let published: std::io::Result<()> = match written {
            Ok(()) => std::fs::rename(staging.as_path(), directory.as_path()),
            Err(error) => Err(error),
        };
        if let Err(error) = published {
            let _ = std::fs::remove_dir_all(staging.as_path());
            return Err(private_index_failure("publishing its directory", &error));
        }
        // From here on the value owns the directory, so a failure below removes it on drop.
        let private: PrivateIndex = PrivateIndex {
            index: directory.join("index"),
            directory,
        };
        if let Err(error) = copy_index(real_index, private.index.as_path()) {
            return Err(private_index_failure("copying the index", &error));
        }
        return Ok(private);
    }

    /// What:
    ///  The private index file Git is pointed at.
    /// Why:
    ///   The caller sets `GIT_INDEX_FILE` to it for every command of the prediction.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// get path(): string
    /// ```
    pub fn path(&self) -> &Path {
        return self.index.as_path();
    }
}

/// What:
///  `impl Drop for PrivateIndex` runs `drop` when the value goes out of scope,
///  like
///       a `[Symbol.dispose]` that the language calls on every exit path.
/// Why:
///   The directory and everything Git wrote into it are removed whether the
///       prediction succeeded or failed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// [Symbol.dispose]() { rmSync(this.#directory, { recursive: true, force: true }); }
/// ```
impl Drop for PrivateIndex {
    /// What:
    ///  Remove the directory and everything in it.
    ///  `&mut self` lends the value for
    ///       its last use.
    /// Why:
    ///   Called by the language on every exit path,
    ///  failures included.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// [Symbol.dispose](): void
    /// ```
    fn drop(&mut self) {
        // What: `let _ = ...` discards the removal's `Result` on purpose.
        // Why:  A drop cannot report a failure; a leftover directory under the Git
        //       directory is inert and carries the prefix that names its owner.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // try { rmSync(directory, { recursive: true }); } catch { /* inert leftover */ }
        // ```
        let _ = std::fs::remove_dir_all(self.directory.as_path());
    }
}

/// Copy,
///  timestamp and removal controls stay out of the release executable.
#[cfg(test)]
#[path = "candidate_private_index_tests.rs"]
mod tests;
