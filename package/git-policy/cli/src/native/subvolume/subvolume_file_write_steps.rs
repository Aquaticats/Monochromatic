//! What:
//!  The file steps that the migration's writers share:
//! write bytes and flush them to disk,
//! flush a directory,
//! and replace a
//!  small file beside the tree without following a link at its name.
//! Why:
//!  Each writer ran "write,
//! and only if that worked,
//! flush" as a chain of `and_then` closures,
//! which
//!  `rust/avoidable-closure` reports;
//! one named function per step reads the same everywhere.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await file.write(bytes); await file.sync();   // the second line runs only if the first did not throw
//! await (await open(directory)).sync();
//! ```

use std::io::Write as _;
use std::path::Path;

/// Write all of `bytes` into `file`,
/// then flush the file to disk;
/// the flush runs only if the write worked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function writeAllThenSync(file: FileHandle, bytes: Uint8Array): Promise<void>; // rejects with the first failure
/// ```
pub fn write_all_then_sync(file: &mut std::fs::File, bytes: &[u8]) -> std::io::Result<()> {
    file.write_all(bytes)?;
    return file.sync_all();
}

/// Flush a directory to disk,
/// so a rename or a new name inside it survives a crash.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function syncDirectory(directory: string): Promise<void>; // (await open(directory)).sync()
/// ```
pub fn sync_directory_to_disk(directory: &Path) -> std::io::Result<()> {
    let handle: std::fs::File = std::fs::File::open(directory)?;
    return handle.sync_all();
}

/// What:
///  Write `bytes` to `target`,
/// by writing them into a sibling created exclusively under a name of its own and renaming it over
///  `target`.
/// Why:
///   A plain write follows a symbolic link and truncates whatever it points at,
/// so a link planted at a
///  list's name would let the command overwrite a file of the person's own.
/// A rename replaces the link itself,
/// never its
///  target,
/// and the exclusive create refuses a name that already exists.
/// A failure removes what this call made,
/// so a
///  failed write leaves neither a half file nor a temporary one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeBesideThenRename(listPath, bytes); // O_EXCL sibling, fsync, rename, fsync the directory
/// ```
pub fn write_beside_then_rename(target: &Path, bytes: &[u8]) -> std::io::Result<()> {
    use std::os::unix::fs::OpenOptionsExt as _;
    let nanos: u128 = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(elapsed) => elapsed.as_nanos(),
        Err(_before_epoch) => 0,
    };
    let mut name: std::ffi::OsString = target.as_os_str().to_os_string();
    name.push(format!(".tmp.{}.{nanos}", std::process::id()));
    let sibling: std::path::PathBuf = std::path::PathBuf::from(name);
    let mut file: std::fs::File = std::fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(sibling.as_path())?;
    if let Err(error) = write_all_then_sync(&mut file, bytes) {
        drop(file);
        let _removed: std::io::Result<()> = std::fs::remove_file(sibling.as_path());
        return Err(error);
    }
    drop(file);
    if let Err(error) = std::fs::rename(sibling.as_path(), target) {
        let _removed: std::io::Result<()> = std::fs::remove_file(sibling.as_path());
        return Err(error);
    }
    return match target.parent() {
        Some(directory) => sync_directory_to_disk(directory),
        None => Ok(()),
    };
}

/// File-write controls stay out of the release executable.
#[cfg(test)]
#[path = "subvolume_file_write_steps_tests.rs"]
mod tests;
