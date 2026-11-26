//! What:
//!  The common Git directory a worktree's own Git directory belongs to:
//!  the directory its `commondir` file names,
//!  else the directory itself.
//! Why:
//!  A linked worktree's own Git directory goes away with the worktree and is not where the repository keeps what belongs to
//!  all of it:
//!  the timing records,
//!  the repository scratch directory a commit's scratch tree lives in,
//!  and the discard audit.
//!  Three readers needed the same answer,
//!  and each wrote the few lines again
//!  (`commit_timing.rs` said so in its own copy),
//!  so the rule lives here once.
//!  Git strips every trailing carriage return and line feed and resolves a relative path against the Git directory.
//!  The name it holds is read as bytes,
//! because a directory whose name is no UTF-8 text is still the directory named.
//!  A path that cannot be resolved is used as written,
//!  so a Git directory whose `commondir` names a missing place still gets a record beside itself rather than none.
//!  The file is opened as a plain one (`crate::plain_file`):
//!  a named pipe at its name answers at once as a Git directory with no common directory
//!  instead of holding every command that asks.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! existsSync(join(gitDir, 'commondir')) ? realpath(resolve(gitDir, readFile(...).trimEnd())) : gitDir
//! ```

/// What:
///  `std::ffi::OsStr` is a borrowed piece of an operating system's own text,
/// which is what one part of a path is made of;
///  its siblings are `str`,
/// which holds UTF-8 text alone,
/// and `OsString`,
/// the owned version.
/// Why:
///  A `commondir` names a directory by bytes,
/// and a name that is no UTF-8 text still names a directory a record belongs in.
use std::ffi::OsStr;

/// What:
///  `std::os::unix::ffi::OsStrExt` is the Unix extension that builds an `OsStr` from bytes (`OsStr::from_bytes`),
///  where `String::from_utf8` refuses bytes that are no text.
/// Why:
///  It is the one place the name the file holds becomes a path part without asking whether it is text.
use std::os::unix::ffi::OsStrExt;

/// Paths of directories.
use std::path::{
    Path,
    PathBuf,
};

/// The one reader of a Git directory's `commondir` file,
/// and the bound it reads to.
use crate::plain_file::{
    SMALL_FILE_LIMIT,
    read_plain_file,
};

/// What:
///  `fn bytes_named(git_dir) -> std::io::Result<Option<Vec<u8>>>` is the bytes of the `commondir` file of `git_dir`,
///  `Ok(None)` when the Git directory holds no such name,
///  and `Err` when it holds something this command cannot read as a plain file.
/// Why:
///  The three readers of this file tell a missing one from a refused one in different ways,
///  and one read serves all three:
///  the reader that takes a Git directory with no common directory,
///  the one that answers nothing for a directory that is not a linked worktree,
///  and the location reader that declines the whole repository.
///  `std::fs::read` is not used here because opening a named pipe for reading waits for a writer that may never come.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function bytesNamed(gitDir: string): Buffer | undefined; // undefined when there is no file, throws when it is no plain file
/// ```
pub fn bytes_named(git_dir: &Path) -> std::io::Result<Option<Vec<u8>>> {
    // `SMALL_FILE_LIMIT` is the bound the wrapper reads a small file to; this repository's registrations hold six bytes.
    let read = read_plain_file(git_dir.join("commondir").as_path(), SMALL_FILE_LIMIT);
    return match read {
        // The name holds a plain file this command read whole.
        Ok(bytes) => Ok(Some(bytes)),
        // The Git directory holds no `commondir`, so it names no other directory.
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        // The name holds a pipe, a directory, a device or a file past the bound, and the caller decides what that means.
        Err(error) => Err(error),
    };
}

/// What:
///  `fn named_without_line_endings(git_dir) -> Option<Vec<u8>>` is the bytes the `commondir` file of `git_dir` holds
///  without Git's trailing carriage returns and line feeds,
///  or nothing when there is no such file,
///  it cannot be read as a plain file,
///  or it says nothing.
///  `Vec<u8>` is a growable list of bytes this function owns (siblings:
/// `&[u8]`,
/// a borrowed view of bytes;
///  `String`,
/// the same buffer with the promise that it is UTF-8).
/// Why:
///  Both answers this module gives start from the same name,
///  so the stripping of Git's own trailing line endings is written once.
///  The name stays bytes rather than becoming text,
/// because `String` would refuse a directory name that is no UTF-8 and
///  this function would then answer nothing,
/// which is the wrong directory for a record to land in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const named = readFileSync(join(gitDir, 'commondir')).subarray(0, endWithoutLineEndings)
/// ```
fn named_without_line_endings(git_dir: &Path) -> Option<Vec<u8>> {
    let bytes: Vec<u8> = bytes_named(git_dir).ok().flatten()?;
    // Where the name ends: just past its last byte that is neither of Git's line endings.
    let mut end: usize = bytes.len();
    while end > 0 && (bytes[end - 1] == b'\r' || bytes[end - 1] == b'\n') {
        end -= 1;
    }
    // The name itself, lent from the bytes read above rather than copied.
    let named: &[u8] = &bytes[..end];
    if named.is_empty() {
        return None;
    }
    return Some(named.to_vec());
}

/// What:
///  The common Git directory of `git_dir`.
/// Why:
///  Every caller asks the same question of a Git directory Git itself handed it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const commonDir = commonDirectoryOf(gitDir);
/// ```
pub fn common_directory_of(git_dir: &Path) -> PathBuf {
    let Some(named) = named_without_line_endings(git_dir) else {
        return git_dir.to_path_buf();
    };
    // The name as one path part: its bytes are the directory's own name, with no text conversion between them.
    let joined: PathBuf = git_dir.join(OsStr::from_bytes(named.as_slice()));
    return std::fs::canonicalize(&joined).unwrap_or(joined);
}

/// What:
///  The common directory `git_dir`'s `commondir` file names,
///  resolved by its own bytes alone,
///  or nothing when the Git directory names none or names one that climbs out of the file system root.
/// Why:
///  A linked worktree's registration records a path relative to the worktree's own Git directory,
///  and the answer must spell the directory the same way that record does rather than as a symlink's target;
///  a directory that is no linked worktree gets nothing rather than its own name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// path.resolve(gitDir, named) // undefined when the file is missing, empty or climbs past the root
/// ```
pub fn common_directory_named_by(git_dir: &Path) -> Option<PathBuf> {
    let named: Vec<u8> = named_without_line_endings(git_dir)?;
    // The path built one part at a time, so a `..` is taken off the part before it rather than left in the text.
    let mut resolved: PathBuf = PathBuf::new();
    for part in git_dir
        .join(OsStr::from_bytes(named.as_slice()))
        .components()
    {
        match part {
            // `.` names the directory itself, so it adds nothing.
            std::path::Component::CurDir => {}
            // `..` names the directory above, which is the part just added.
            std::path::Component::ParentDir => {
                if !resolved.pop() {
                    return None;
                }
            }
            other => resolved.push(other.as_os_str()),
        }
    }
    return Some(resolved);
}

/// The resolution is controlled in the test module.
#[cfg(test)]
#[path = "common_directory_of_a_git_directory_tests.rs"]
mod tests;
