//! What:
//!  The one helper that gives a fixture file a new inode while its bytes and its modification time stay as they were,
//!  shared by the unit tests and the binary-level controls.
//! Why:
//!  A worktree copied to another disk keeps every file's bytes and times and gives each a new inode,
//!  which leaves the stat data every index entry recorded stale while the content is unchanged.
//!  That is the state a commit has to survive,
//!  and writing the file again would move its modification time as well.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await copyFile(path, beside); await utimes(beside, atime, mtime); await rename(beside, path);
//! ```

/// `OsString` is owned operating-system text and `PathBuf` an owned path.
use std::ffi::OsString;
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{Path, PathBuf};

/// What:
///  Give the file at `path` a new inode with the same bytes and the same modification time.
/// Why:
///   A copy beside the file and a rename over it change nothing a reader sees but the inode and the change time,
///  which is exactly what a worktree copied to another disk leaves behind.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await copyFile(path, beside); await utimes(beside, atime, mtime); await rename(beside, path);
/// ```
pub fn renew_inode(path: &Path) {
    let metadata: std::fs::Metadata = std::fs::metadata(path).expect("the file");
    let modified: std::time::SystemTime = metadata.modified().expect("a modification time");
    // The name beside the file keeps its own extension, so two fixtures never share a staging name.
    let mut staged_name: OsString = path.as_os_str().to_os_string();
    staged_name.push(".renewed");
    let beside: PathBuf = PathBuf::from(staged_name);
    std::fs::copy(path, beside.as_path()).expect("a copy beside the file");
    std::fs::File::options()
        .write(true)
        .open(beside.as_path())
        .expect("the copy")
        .set_times(std::fs::FileTimes::new().set_modified(modified))
        .expect("the copy's times");
    std::fs::rename(beside.as_path(), path).expect("the copy over the file");
}
