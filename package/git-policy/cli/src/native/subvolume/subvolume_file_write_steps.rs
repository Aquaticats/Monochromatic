//! What:
//!  Two file steps that the migration's writers share: write bytes and flush them to disk, and flush a directory.
//! Why:
//!  Each writer ran "write, and only if that worked, flush" as a chain of `and_then` closures, which
//!  `rust/avoidable-closure` reports; one named function per step reads the same everywhere.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await file.write(bytes); await file.sync();   // the second line runs only if the first did not throw
//! await (await open(directory)).sync();
//! ```

use std::io::Write as _;
use std::path::Path;

/// Write all of `bytes` into `file`, then flush the file to disk; the flush runs only if the write worked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function writeAllThenSync(file: FileHandle, bytes: Uint8Array): Promise<void>; // rejects with the first failure
/// ```
pub fn write_all_then_sync(file: &mut std::fs::File, bytes: &[u8]) -> std::io::Result<()> {
    file.write_all(bytes)?;
    return file.sync_all();
}

/// Flush a directory to disk, so a rename or a new name inside it survives a crash.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function syncDirectory(directory: string): Promise<void>; // (await open(directory)).sync()
/// ```
pub fn sync_directory_to_disk(directory: &Path) -> std::io::Result<()> {
    let handle: std::fs::File = std::fs::File::open(directory)?;
    return handle.sync_all();
}
