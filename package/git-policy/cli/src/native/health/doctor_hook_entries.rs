//! What:
//!  The entries of one hooks directory:
//!  which of them Git would run as a traditional hook,
//!  how the directory is listed,
//!  and the bytes the report spells them as.
//! Why:
//!  A traditional hook is an executable file,
//!  the samples Git ships beside one are not hooks,
//!  and both file system reads of the scan are bounded;
//!  they keep one place of their own
//!  so the doctor's decision keeps within its line budget.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // doctor_hook_entries.ts: one hooks directory, listed and rendered.
//! ```

/// A missing directory is not a failure;
/// one that cannot be read is.
use std::io::ErrorKind;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};
/// The ceiling of each file system read of the report.
use std::time::Duration;
/// The ceiling a scan of the report runs under.
use crate::wait_ceilings::REPORT_SCAN_CEILING;
/// The job of such a read,
/// run on a thread of its own.
use crate::work_awaited_within_a_ceiling::work_within;

/// File name suffix of the samples Git installs,
/// which are never hooks.
const SAMPLE_SUFFIX: &[u8] = b".sample";

/// What:
///  Whether a file mode makes the file runnable by someone.
/// Why:
///   Git runs a hook only when it is executable and prints a hint for a file it skips,
/// so a plain file is not a hook.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (stats.mode & 0o111) !== 0
/// ```
#[cfg(unix)]
fn is_executable(metadata: &std::fs::Metadata) -> bool {
    use std::os::unix::fs::PermissionsExt;
    return metadata.permissions().mode() & 0o111 != 0;
}

/// What:
///  Whether a file is runnable.
/// Why:
///   Without Unix permission bits every regular file counts,
/// because the platform has no mode to test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// true
/// ```
#[cfg(not(unix))]
fn is_executable(_metadata: &std::fs::Metadata) -> bool {
    return true;
}

/// What:
///  Whether one entry of a hooks directory is a hook now,
/// or nothing when it is not
///  a hooks directory this scan can look at.
/// Why:
///   The entry was listed a moment before it is looked at,
/// and the directory changes under the scan:
/// the person deletes a
///  hook in another terminal,
/// a script reinstalls them,
/// or a `git-lfs` upgrade rewrites the directory.
///  An entry that is gone by the time it is looked at is not a hook now,
/// which is the same answer a missing directory gets,
/// and
///  every other failure to look is still reported,
/// because silence would pass a file the command could not examine.
///  The listing hands over names and not open entries,
/// so the predicate takes the path it is asked about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stats = await lstat(path).catch((error) => (error.code === 'ENOENT' ? undefined : error));
/// stats instanceof Error ? throw stats : isSymlink(stats) || (isFile(stats) && isExecutable(stats));
/// ```
pub fn is_hook_entry(path: &Path) -> std::io::Result<bool> {
    let Some(name) = path.file_name() else {
        return Ok(false);
    };
    if is_a_sample(name) {
        return Ok(false);
    }
    let metadata: std::fs::Metadata = match std::fs::symlink_metadata(path) {
        Ok(looked) => looked,
        // Gone between the listing and the look: it is not a hook now, and one vanished name must not cost the whole listing.
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error),
    };
    let kind: std::fs::FileType = metadata.file_type();
    return Ok(kind.is_symlink() || (kind.is_file() && is_executable(&metadata)));
}

/// What:
///  Whether one entry's name ends the way a sample Git installs ends.
/// Why:
///   A sample is never a hook,
/// and the check is one condition of the scan,
/// so it is a named rule rather than a closure
///  inside it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isASample = (name: string) => name.endsWith('.sample');
/// ```
fn is_a_sample(name: &std::ffi::OsStr) -> bool {
    return name.as_encoded_bytes().ends_with(SAMPLE_SUFFIX);
}

/// What:
///  The names in one directory,
/// as one read of it.
///  `fn(&Path) -> std::io::Result<Vec<PathBuf>>` is a function pointer,
/// so a control can stand a read that never answers where the
///  listing stands.
/// Why:
///   The listing of a directory is where a mount that stopped answering waits,
/// so it is the read the bound is placed
///  on,
/// and it is a value the report is given rather than a call it makes itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type EntryListing = (directory: string) => string[];
/// ```
pub type EntryListing = fn(&Path) -> std::io::Result<Vec<PathBuf>>;

/// What:
///  The names in one directory,
/// read by the system.
/// Why:
///   This is the production listing the bounded scan takes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const listEntries = (directory) => readdirSync(directory, { withFileTypes: true }).map((entry) => join(directory, entry.name));
/// ```
fn list_entries(directory: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut entries: Vec<PathBuf> = Vec::new();
    for entry in std::fs::read_dir(directory)? {
        entries.push(entry?.path());
    }
    return Ok(entries);
}

/// What:
///  The traditional hooks in one directory:
/// executable regular files and symbolic links,
/// except `*.sample`.
/// Why:
///   A missing directory holds none;
/// any other failure to read it is reported,
/// because silence would pass a
///  directory the command could not look at.
///  A link counts whatever it points at (a dangling link too),
/// because Git follows it when it starts the hook.
///  One entry that is gone by the time it is looked at is not a hook and does not end the listing
/// (`is_hook_entry`).
///  Every call here reads the file system,
/// and a mount that stops answering holds such a call in the kernel with no end,
/// so
///  the whole scan runs on a thread of its own and is awaited only to `REPORT_SCAN_CEILING`.
///  A scan that gives no answer is an error,
/// never an empty directory:
/// silence would read as "no hooks here".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (await readdir(directory)).filter((name) => !name.endsWith('.sample') && (isSymlink(name) || isExecutableFile(name)));
/// ```
pub fn scan_hooks_directory(directory: &Path) -> std::io::Result<Vec<PathBuf>> {
    return scan_hooks_directory_with(list_entries, REPORT_SCAN_CEILING, directory);
}

/// What:
///  `scan_hooks_directory` with the listing and the ceiling given.
/// Why:
///   The listing and the ceiling are the two things a control varies;
/// `REPORT_SCAN_CEILING` says why the production value is
///  what it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// scanHooksDirectoryWith(listEntries, REPORT_SCAN_CEILING, directory);
/// ```
pub fn scan_hooks_directory_with(listing: EntryListing, ceiling: Duration, directory: &Path) -> std::io::Result<Vec<PathBuf>> {
    let named: PathBuf = directory.to_path_buf();
    let scanned: Option<std::io::Result<Vec<PathBuf>>> = work_within(ceiling, move || {
        let entries: Vec<PathBuf> = match listing(named.as_path()) {
            Ok(found) => found,
            Err(error) if error.kind() == ErrorKind::NotFound || error.kind() == ErrorKind::NotADirectory => {
                return Ok(Vec::new());
            }
            Err(error) => return Err(error),
        };
        let mut hooks: Vec<PathBuf> = Vec::new();
        // Each entry is looked at through `is_hook_entry`, so a name that vanished since the listing is skipped and the
        //  hooks that are still there are listed.
        for path in entries {
            if is_hook_entry(path.as_path())? {
                hooks.push(path);
            }
        }
        hooks.sort();
        return Ok(hooks);
    });
    let Some(answer) = scanned else {
        return Err(std::io::Error::new(
            ErrorKind::TimedOut,
            format!("the listing of {} gave no answer within {ceiling:?}", directory.display()),
        ));
    };
    return answer;
}

/// What:
///  The traditional hooks in every directory,
/// in directory order and then name order.
/// Why:
///   The report lists them as a person reads them:
/// the default directory first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (await Promise.all(directories.map(scanHooksDirectory))).flat();
/// ```
pub fn find_traditional_hooks(directories: &[PathBuf]) -> std::io::Result<Vec<PathBuf>> {
    let mut hooks: Vec<PathBuf> = Vec::new();
    for directory in directories {
        hooks.extend(scan_hooks_directory(directory.as_path())?);
    }
    return Ok(hooks);
}

/// What:
///  The report:
/// one hook pathname per line,
/// as the bytes the pathname is made of.
///  `Vec<u8>` is the whole report,
/// so the caller can write it as it stands.
/// Why:
///   The output is for people and scripts alike,
/// so it carries nothing but the pathnames.
///  A pathname is a byte string:
/// `Path::display()` replaces each byte that is not UTF-8 with U+FFFD,
/// and a directory
///  whose name is exactly that replacement is a different directory,
/// so a text report named a hook the command
///  never inspected,
/// and a person who deleted what it named would delete somebody else's script.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Buffer.concat(hooks.map((hook) => Buffer.concat([Buffer.from(hook), Buffer.from('\n')])));
/// ```
pub fn render_hooks(hooks: &[PathBuf]) -> Vec<u8> {
    let mut text: Vec<u8> = Vec::new();
    for hook in hooks {
        text.extend_from_slice(path_bytes(hook.as_path()));
        text.push(b'\n');
    }
    return text;
}

/// What:
///  The bytes a pathname is made of.
/// Why:
///   A pathname is a byte string,
/// and both the report and the paths written into records are read from these bytes
///  (`foundation/path_bytes_in_records.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Buffer.from(path); // Node pathnames are byte strings
/// ```
#[cfg(unix)]
fn path_bytes(path: &Path) -> &[u8] {
    use std::os::unix::ffi::OsStrExt;
    return path.as_os_str().as_bytes();
}

/// What:
///  The bytes a pathname is made of,
/// on a platform that spells pathnames in text.
/// Why:
///   Without byte pathnames the platform's text is the best answer,
/// and naming a pathname approximately is better
///  than not naming it at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Buffer.from(path, 'utf8');
/// ```
#[cfg(not(unix))]
fn path_bytes(path: &Path) -> Vec<u8> {
    return path.to_string_lossy().into_owned().into_bytes();
}
