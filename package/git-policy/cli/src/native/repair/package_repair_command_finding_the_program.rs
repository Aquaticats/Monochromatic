//! What:
//!  Finding the program a repair starts:
//! the first executable file the person's search path holds under the tool's name,
//! named by its absolute path.
//! Why:
//!  Every repair starts a program the person's environment resolves,
//! and the lookup is one rule that reads nothing else of a run,
//! so the Cargo,
//! pnpm and toolchain repairs can share it
//! without reaching the file that starts and waits for a child.
//! A missing program is a verdict the repair words,
//! never a failure to start.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function findProgram(name: string, searchPath?: string): string | undefined;
//! ```

/// Import the borrowed,
/// platform-native string a search path is.
use std::ffi::OsStr;
/// Import the owned filesystem path a found program is named by.
use std::path::PathBuf;
/// What:
///  The first executable file named `name` in a search path,
///  named by its absolute path,
///  with a relative entry read from the directory this command was started in.
/// Why:
///   The program is the one the person's environment resolves;
/// a missing one is a verdict,
/// not a failure to start.
/// An entry without a leading slash names a directory of the directory this lookup happens in,
/// and every repair starts its tool in the worktree,
/// so the file is named where it was found rather than with a path the child would read from its own directory.
/// A directory of this process that cannot be read names no file to start,
/// which is the same answer as a program the search path does not hold.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function findProgram(name: string, searchPath?: string): string | undefined;
/// ```
pub fn find_program(
    name: &str,
    search_path: Option<&OsStr>,
    start: &crate::start_directory::StartDirectory,
) -> Option<PathBuf> {
    use std::os::unix::fs::PermissionsExt;
    let search: &OsStr = search_path?;
    for directory in std::env::split_paths(search) {
        // An empty entry means the current directory in a shell; a repair never runs a program found there.
        if directory.as_os_str().is_empty() {
            continue;
        }
        // A name that is already absolute is kept as the person wrote it; only a relative one needs the directory this
        //  command was started in, which is where the person's own shell would read it.
        let candidate: PathBuf = match start.path() {
            Some(started_in) => started_in.join(&directory).join(name),
            None if directory.is_absolute() => directory.join(name),
            None => return None,
        };
        let runnable: bool = match std::fs::metadata(&candidate) {
            Ok(found) => found.is_file() && found.permissions().mode() & 0o111 != 0,
            Err(_) => false,
        };
        if runnable {
            return std::path::absolute(candidate).ok();
        }
    }
    return None;
}
