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
//!  so the prediction could miss that edit (the installed wrapper's
//!      `index-file-timestamps.ts`,
//!  issue #544).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // await using index = await createPrivateIndex(realIndexPath); // removed on dispose
//! ```

/// Import the layer's failure type and its closed list of causes.
use super::candidate_error::{CandidateError, CandidateFailure};
/// `Path`/`PathBuf` are borrowed/owned filesystem paths of raw OS bytes.
use std::path::{Path, PathBuf};
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
        format!("cli-git could not prepare a private copy of the index: {step}: {error}.").as_str(),
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
    // `match` on the metadata read: a missing file is an answer, every other error is a failure.
    let metadata: std::fs::Metadata = match std::fs::metadata(source) {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error),
    };
    // A trailing `?` returns the copy failure to our caller.
    std::fs::copy(source, destination)?;
    // What: `FileTimes::new().set_accessed(..).set_modified(..)` collects both times;
    //       `.set_times(..)` writes them to the open copy.
    // Why:  Git's racy-entry protection compares entry times with the index file's time.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // await utimes(destination, stats.atime, stats.mtime);
    // ```
    let times: std::fs::FileTimes = std::fs::FileTimes::new()
        .set_accessed(metadata.accessed()?)
        .set_modified(metadata.modified()?);
    std::fs::File::options()
        .write(true)
        .open(destination)?
        .set_times(times)?;
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
                "cli-git could not prepare a private copy of the index: Git named an index path without a directory.",
            ));
        };
        let directory: PathBuf = parent.join(format!("{PRIVATE_INDEX_PREFIX}{}", unique_suffix()));
        if let Err(error) = create_private_directory(directory.as_path()) {
            return Err(private_index_failure("creating its directory", &error));
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
