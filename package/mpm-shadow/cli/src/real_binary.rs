//! What:
//! Finds the real `mpm` on PATH while skipping the shadow itself.
//! Why:
//! The shadow is first on PATH under the name `mpm`,
//! so a plain PATH search would find the shadow again and
//!           run itself forever.
//! The repository and global shadow directories can both be on PATH,
//! so every candidate
//!           that resolves to this executable is skipped,
//! not only the first.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // real-binary.ts
//! export function findInPath(name: string, pathVariable: string, skip: string): string | undefined { ... }
//! ```

/// Imports the file-system module (resolves symlinks,
/// reads permissions).
use std::fs;

/// What:
/// `use std::os::unix::fs::PermissionsExt;` imports a trait (a named bundle of methods,
/// like a TS interface
///           that types can opt into) that makes `.mode()` available on file permissions.
/// Why:
/// The execute bits live in the Unix mode number.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // No equivalent: in TS `stats.mode` is always available.
/// ```
use std::os::unix::fs::PermissionsExt;

/// Imports the borrowed and owned path types.
use std::path::{Path, PathBuf};

/// What:
/// `const EXECUTE_BITS: u32 = 0o111;` is a constant unsigned 32-bit integer (siblings `i32`,
/// `u64`).
/// The `0o`
///           prefix writes it in base 8:
/// the owner,
/// group and other execute bits.
/// Why:
/// A file is runnable when any of those bits is set;
/// `u32` because `.mode()` returns that type.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const EXECUTE_BITS = 0o111;
/// ```
const EXECUTE_BITS: u32 = 0o111;

/// What:
/// `fn is_executable_file(path: &Path) -> bool` declares a private function over a borrowed path.
/// Why:
/// The search must ignore directories,
/// missing files and files without an execute bit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isExecutableFile(path: string): boolean { return fs.statSync(path, { throwIfNoEntry: false })?.isFile() === true && (fs.statSync(path).mode & 0o111) !== 0; }
/// ```
fn is_executable_file(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    // `fs::metadata(path)` returns a `Result`; `if let Ok(metadata)` runs the block on success.
    if let Ok(metadata) = fs::metadata(path) {
        return (metadata.permissions().mode() & EXECUTE_BITS) != 0;
    }
    return false;
}

/// What:
/// `pub fn find_in_path(name: &str, path_variable: &str, skip: &Path) -> Option<PathBuf>` declares a public
///           function with three borrowed inputs returning `Some(path)` when found,
/// `None` when not.
/// Why:
/// `skip` is the shadow executable's own resolved path,
/// so the loop never returns the shadow.
/// `PathBuf`
///           and not `&Path` because the result outlives the borrowed inputs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function findInPath(name: string, pathVariable: string, skip: string): string | undefined
/// ```
pub fn find_in_path(name: &str, path_variable: &str, skip: &Path) -> Option<PathBuf> {
    for directory in path_variable.split(':') {
        // An empty PATH entry means the current directory in a shell; the shadow ignores it on purpose.
        if directory.is_empty() {
            continue;
        }
        let candidate: PathBuf = Path::new(directory).join(name);
        if !is_executable_file(&candidate) {
            continue;
        }

        // What:     `fs::canonicalize(&candidate)` resolves every symlink to the final path (a `Result`); the `&`
        //           lends the path. The `&&` joins two conditions in one `if let ... && ...`.
        // Why:      Every link to the shadow resolves to the one executable, so comparing resolved paths recognizes
        //           the shadow under whatever name it was found.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (fs.realpathSync(candidate) === skip) continue;
        // ```
        if let Ok(resolved) = fs::canonicalize(&candidate)
            && resolved == skip
        {
            continue;
        }

        // What:     `Some(candidate)` wraps the found path in the present case of `Option`.
        // Why:      The first real `mpm` in PATH order wins.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return candidate;
        // ```
        return Some(candidate);
    }
    return None;
}
