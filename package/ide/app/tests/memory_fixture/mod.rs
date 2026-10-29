//! Disposable fixture directories for the integration tests,
//!  in memory when the system has one.

/// Directory paths.
use std::path::Path;

/// Memory-backed directory for fixtures,
///  when the system has one.
const MEMORY_DIRECTORY: &str = "/dev/shm";

/// What:
///  A fresh disposable directory whose name starts with `prefix`,
///  in `/dev/shm` when that
///       exists and the usual temporary directory otherwise.
///  `tempfile::TempDir` removes the
///       directory when it is dropped.
/// Why:
///  On btrfs,
///  the first read of a freshly written file updates its access time inside a
///      filesystem transaction,
///  and creating or removing files joins one too.
///  While the machine
///      flushes,
///  those calls waited in the kernel (btrfs `handle_reserve_ticket` and
///      `wait_current_trans`) for seconds,
///  past the bounds the worker tests wait.
///  Memory-backed
///      files never wait for the disk.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function directory(prefix: string): TempDir { return existsSync('/dev/shm') ? mkdtemp('/dev/shm/' + prefix) : mkdtemp(tmpdir() + '/' + prefix); }
/// ```
pub fn directory(prefix: &str) -> tempfile::TempDir {
    let memory = Path::new(MEMORY_DIRECTORY);
    if memory.is_dir()
        && let Ok(directory) = tempfile::Builder::new().prefix(prefix).tempdir_in(memory)
    {
        return directory;
    }
    return tempfile::tempdir().expect("disposable fixture directory");
}
