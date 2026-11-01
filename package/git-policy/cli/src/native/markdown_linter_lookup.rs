//! What:
//!  Find the `monochromatic-lint` executable the `markdown/autofix` policy runs,
//!  on
//!       the PATH of the invocation's environment.
//! Why:
//!  Repository configuration cannot choose the program (`doc/planning/cli-git-rust-implementation.md`,
//!      lines 181 to 189:
//!  first-party linter selection is installation wiring).
//!  The linter
//!      is installed through mise as `cargo:monochromatic-lint`,
//!  and mise exposes installed
//!      tools on PATH,
//!  through `mise activate` or its shims directory;
//!  real Git is found on
//!      PATH the same way.
//!  Only absolute PATH entries are searched:
//!  a relative or empty entry
//!      names a directory relative to where the command runs,
//!  usually inside the very
//!      repository whose content the linter checks,
//!  and that repository must not be able to
//!      supply the program.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const linter = findLinter(env.PATH); // '/home/u/.local/share/mise/shims/monochromatic-lint'
//! ```

/// Import the executable-bit test real-Git lookup uses.
use super::real_git_candidate::is_executable;
/// `OsStr` is borrowed operating-system text:
///  PATH need not be UTF-8.
use std::ffi::OsStr;
/// `PathBuf` is an owned filesystem path.
use std::path::PathBuf;

/// The linter's executable name without the platform's suffix.
pub const LINTER_NAME: &str = "monochromatic-lint";

/// What:
///  The linter's file name on this platform:
///  `monochromatic-lint`,
///  or with `.exe` on
///       Windows.
///  `std::env::consts::EXE_SUFFIX` is the platform's executable suffix.
/// Why:
///   Cargo installs the executable under exactly this name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const linterFileName = () => process.platform === 'win32' ? 'monochromatic-lint.exe' : 'monochromatic-lint';
/// ```
pub fn linter_file_name() -> String {
    return format!("{LINTER_NAME}{}", std::env::consts::EXE_SUFFIX);
}

/// What:
///  The first absolute PATH entry holding an executable regular file of the linter's
///       name,
///  following symbolic links.
///  `Option<&OsStr>` is the PATH value or nothing.
/// Why:
///   This is the shell's lookup order,
///  without the entries that would let the
///       checked repository choose the program.
///  `None` means the linter is not installed
///       where this invocation can see it,
///  which the policy reports as a failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function findLinter(path?: string): string | undefined;
/// ```
pub fn find_linter(path_variable: Option<&OsStr>) -> Option<PathBuf> {
    // A trailing `?` on an absent PATH returns `None` at once: there is nothing to search.
    let path: &OsStr = path_variable?;
    let name: String = linter_file_name();
    // `std::env::split_paths` cuts PATH by the platform's separator.
    for directory in std::env::split_paths(path) {
        if !directory.is_absolute() {
            continue;
        }
        let candidate: PathBuf = directory.join(name.as_str());
        // `std::fs::metadata` follows symbolic links, as running the file would.
        if let Ok(metadata) = std::fs::metadata(&candidate)
            && metadata.is_file()
            && is_executable(&metadata)
        {
            return Some(candidate);
        }
    }
    return None;
}

/// Lookup-order controls stay out of the release executable.
#[cfg(test)]
#[path = "markdown_linter_lookup_tests.rs"]
mod tests;
