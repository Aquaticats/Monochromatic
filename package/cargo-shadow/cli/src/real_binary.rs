//! What:
//! Finds the real `cargo` on PATH while skipping the shadow itself.
//! Why:
//! The shadow is first on PATH under the name `cargo`,
//! so a plain PATH search would find the shadow again and run
//! itself forever.
//! Every candidate that resolves to this executable is skipped,
//! not only the first,
//! because a
//! repository directory and a global directory can both hold a link to it.
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
/// `use std::os::unix::fs::PermissionsExt;` imports a trait (a named bundle of methods that types opt into,
/// like a
/// TS interface) that makes `.mode()` available on file permissions.
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
/// `const EXECUTE_BITS: u32 = 0o111;` is an unsigned 32-bit integer (siblings:
/// `i32`,
/// `u64`).
/// The `0o` prefix writes
/// it in base 8:
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
/// `fn is_executable_file(path: &Path) -> bool` is a private function over a borrowed path.
/// Why:
/// The search must ignore directories,
/// missing files and files without an execute bit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isExecutableFile(path: string): boolean { return statSync(path, { throwIfNoEntry: false })?.isFile() === true && (statSync(path).mode & 0o111) !== 0; }
/// ```
fn is_executable_file(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    // `fs::metadata(path)` returns a `Result`; `if let Ok(metadata)` runs the block only on success.
    if let Ok(metadata) = fs::metadata(path) {
        return (metadata.permissions().mode() & EXECUTE_BITS) != 0;
    }
    return false;
}

/// What:
/// `pub fn find_in_path(name: &str, path_variable: &str, skip: &[PathBuf]) -> Option<PathBuf>` takes three borrowed
/// inputs and returns `Some(path)` when found,
/// `None` when not.
/// `Option` is Rust's "value or nothing" (a TS
/// `T | undefined`).
/// `&[PathBuf]` is a borrowed view of a list of owned paths (siblings:
/// `Vec<PathBuf>`,
/// the owned list).
/// Why:
/// `skip` holds resolved paths the loop must never return:
/// the shadow's own executable,
/// and every candidate an earlier
/// shadow instance already picked (`guard.rs`),
/// because a launcher or other link that leads back to the shadow resolves to
/// a path that is not the shadow's own.
/// `PathBuf` (owned) and not `&Path` (borrowed) because the result outlives the
/// borrowed inputs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function findInPath(name: string, pathVariable: string, skip: string[]): string | undefined
/// ```
pub fn find_in_path(name: &str, path_variable: &str, skip: &[PathBuf]) -> Option<PathBuf> {
    for directory in path_variable.split(':') {
        // An empty PATH entry means the current directory in a shell; the shadow ignores it on purpose.
        if directory.is_empty() {
            continue;
        }
        let candidate: PathBuf = Path::new(directory).join(name);
        if !is_executable_file(&candidate) {
            continue;
        }

        // `fs::canonicalize(&candidate)` resolves every symlink to the final path, so every link to the shadow
        // resolves to the one executable and is recognized under whatever name it was found. The `&&` joins two
        // conditions inside one `if let`.
        if let Ok(resolved) = fs::canonicalize(&candidate)
            && skip.contains(&resolved)
        {
            continue;
        }

        // `Some(candidate)` wraps the found path in the present case of `Option`: the first real cargo in PATH order wins.
        return Some(candidate);
    }
    return None;
}

/// What:
/// `pub fn own_real_path() -> PathBuf` is the shadow's own resolved executable path,
/// or an empty path when the system cannot
/// say.
/// Why:
/// The PATH search skips every entry that resolves to it;
/// an empty path matches nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function ownRealPath(): string { return fs.realpathSync(process.execPath); }
/// ```
pub fn own_real_path() -> PathBuf {
    if let Ok(executable) = std::env::current_exe() {
        if let Ok(resolved) = fs::canonicalize(&executable) {
            return resolved;
        }
        return executable;
    }
    return PathBuf::new();
}

/// What:
/// `pub fn find_real_cargo() -> Option<PathBuf>` is the first `cargo` on PATH that is not this binary.
/// Why:
/// The install step and the catalog commands run cargo themselves,
/// and they must never run the shadow again.
/// The result keeps
/// the PATH entry's own name (not its resolved path):
/// `~/.cargo/bin/cargo` is a rustup proxy that picks the toolchain from
/// the name it was started under,
/// so only the skip test uses resolved paths.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function findRealCargo(): string | undefined { return findInPath('cargo', process.env.PATH ?? '', [ownRealPath()]); }
/// ```
pub fn find_real_cargo() -> Option<PathBuf> {
    let path_variable: String = std::env::var("PATH").unwrap_or_default();
    return find_in_path(crate::constants::SHADOWED_NAME, &path_variable, &[own_real_path()]);
}
