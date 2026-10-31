//! What:
//! Finds the real tool on PATH while skipping the shadow itself.
//! Why:
//! The shadow is first on PATH under the real tool's name,
//! so a plain PATH search would find the shadow again and run itself forever.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // real-binary.ts
//! export function findInPath(name: string, pathVariable: string, skip: string): string | undefined { ... }
//! ```

/// What:
/// `use std::fs;` brings the standard library's file-system module into scope under the name `fs`.
/// Why:
/// We read file metadata and resolve symlinks below.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as fs from 'node:fs';
/// ```
use std::fs;

/// What:
/// `use std::os::unix::fs::PermissionsExt;` imports a trait
/// (a named bundle of methods,
/// like a TS interface,
/// that types can opt into).
/// Importing it makes the `.mode()` method available on file permissions.
/// Why:
/// The execute bits live in the Unix mode number,
/// and without this import `.mode()` does not exist.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // No equivalent: in TS `stats.mode` is always available.
/// ```
use std::os::unix::fs::PermissionsExt;

/// What:
/// Imports `Path` (a borrowed read-only file path)
/// and its sibling `PathBuf` (the owned growable one).
/// Why:
/// `Path` is for arguments we only read,
/// and `PathBuf` is for the path we hand back to the caller to keep.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS uses plain strings for both.
/// ```
use std::path::{Path, PathBuf};

/// What:
/// `const EXECUTE_BITS: u32 = 0o111;` is a constant unsigned 32-bit integer (the siblings are `i32` / `u64`).
/// The `0o` prefix writes the number in base 8,
/// so `0o111` is the owner and group and other execute bits.
/// Why:
/// A file counts as runnable if any of those three bits is set.
/// It is `u32` because that is the type `.mode()` returns,
/// so no cast is needed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const EXECUTE_BITS = 0o111;
/// ```
const EXECUTE_BITS: u32 = 0o111;

/// What:
/// `fn is_executable_file(path: &Path) -> bool` declares a private function.
/// `&Path` means "a borrowed read-only path:
/// I only look at it and I do not take it".
/// `bool` is true or false.
/// Why:
/// The PATH search must ignore directories and missing files and files without an execute bit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isExecutableFile(path: string): boolean {
///   if (!fs.statSync(path, { throwIfNoEntry: false })?.isFile()) return false;
///   return (fs.statSync(path).mode & 0o111) !== 0;
/// }
/// ```
fn is_executable_file(path: &Path) -> bool {
    // What:     `path.is_file()` asks the file system whether the path names a regular file (following symlinks).
    // `!` negates it. Missing paths answer false rather than failing.
    // Why:      Skip directories and missing entries cheaply.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (!fs.statSync(path, { throwIfNoEntry: false })?.isFile()) return false;
    // ```
    if !path.is_file() {
        return false;
    }

    // What:     `if let Ok(metadata) = fs::metadata(path)` runs the block only when the call succeeded. `fs::metadata`
    // returns a `Result` (a value that is either `Ok(data)` or `Err(error)`; Rust has no thrown exceptions
    // for this). `Ok(metadata)` on the left unwraps the success case into the name `metadata`. `path` is
    // passed as it is because it is already a borrow.
    // Why:      We need the permission bits; if reading them fails we fall through to "not executable".
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let metadata;
    // try { metadata = fs.statSync(path); } catch { metadata = undefined; }
    // if (metadata !== undefined) { ... }
    // ```
    if let Ok(metadata) = fs::metadata(path) {
        // What:     `metadata.permissions().mode()` reads the Unix mode number; `& EXECUTE_BITS` keeps only the
        // execute bits; `!= 0` asks whether any is set.
        // Why:      This is the "may it be run" test.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return (metadata.mode & EXECUTE_BITS) !== 0;
        // ```
        return (metadata.permissions().mode() & EXECUTE_BITS) != 0;
    }

    // What:     `return false;` leaves the function with `false`.
    // Why:      The metadata could not be read, so treat the file as not runnable.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return false;
    // ```
    return false;
}

/// What:
/// `pub fn find_in_path(...) -> Option<PathBuf>` declares a public function with three borrowed inputs
/// (`&str` text slices and a `&Path`) that returns an `Option<PathBuf>`:
/// `Some(path)` when found and `None` when not (Rust's replacement for returning `undefined`).
/// `PathBuf` is the owned path (the sibling `&Path` is the borrowed one).
/// Why:
/// Called for the real tool and for `ionice`.
/// `skip` is the shadow executable's own canonical path,
/// so the loop never returns the shadow.
/// The result is `PathBuf` and not `&Path` because it outlives the borrowed inputs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function findInPath(name: string, pathVariable: string, skip: string): string | undefined {
///   for (const directory of pathVariable.split(':')) {
///     if (directory === '') continue;
///     const candidate = path.join(directory, name);
///     if (!isExecutableFile(candidate)) continue;
///     if (fs.realpathSync(candidate) === skip) continue;
///     return candidate;
///   }
///   return undefined;
/// }
/// ```
pub fn find_in_path(name: &str, path_variable: &str, skip: &Path) -> Option<PathBuf> {
    // What:     `for directory in path_variable.split(':')` loops over the pieces of the PATH text. `split(':')`
    // yields each piece lazily; `directory` is a borrowed `&str`.
    // Why:      PATH is a colon-separated list searched in order.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (const directory of pathVariable.split(':')) {
    // ```
    for directory in path_variable.split(':') {
        // An empty PATH entry means the current directory in a shell; the shadow ignores it on purpose.
        if directory.is_empty() {
            continue;
        }

        // What:     `Path::new(directory).join(name)` wraps the text as a path and appends the file name, giving an
        // owned `PathBuf` (type annotation `: PathBuf` written out so the reader sees it).
        // Why:      This is the full path of the candidate tool.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const candidate: string = path.join(directory, name);
        // ```
        let candidate: PathBuf = Path::new(directory).join(name);

        // `&candidate` lends the path to the check without giving it away.
        if !is_executable_file(&candidate) {
            continue;
        }

        // What:     `if let Ok(real) = fs::canonicalize(&candidate)` resolves every symlink to the final path and runs
        // the block only on success. `&candidate` lends the path.
        // Why:      The shadow links (`rg`, `grep`, `find`) all resolve to the one `search-shadow` executable, so
        // comparing resolved paths recognizes the shadow whatever name it was found under.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const real = fs.realpathSync(candidate);
        // if (real === skip) continue;
        // ```
        if let Ok(real) = fs::canonicalize(&candidate)
            && real == skip
        {
            continue;
        }

        // What:     `return Some(candidate);` wraps the found path in `Some` (the "present" case of `Option`) and
        // returns it.
        // Why:      Hand the caller the first real tool in PATH order.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return candidate;
        // ```
        return Some(candidate);
    }

    // What:     `return None;` returns the "absent" case of `Option`.
    // Why:      No directory on PATH held a runnable file of that name.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return undefined;
    // ```
    return None;
}
