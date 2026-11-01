//! What:
//!  The system configuration file that one real Git executable reads when `GIT_CONFIG_SYSTEM` is not set,
//!  learned once per executable identity and cached beside the `git version` cache.
//! Why:
//!  Git compiles the path in (`ETC_GITCONFIG`, `/etc/gitconfig` for a build with prefix `/usr`, and under the prefix
//!  for any other build, resolved against the executable's own location in a runtime-prefix build), so no reader may
//!  assume `/etc/gitconfig`.
//!  `git var GIT_CONFIG_SYSTEM` prints the path the executable would read, with the environment's own overrides
//!  removed from its start, so the path is a property of the executable's bytes and location, which the identity of
//!  `git_version_cache` captures.
//!  An executable without an identity (a script, a shim) or an answer that is not an absolute path is unknown, and
//!  the readers then leave the question to Git.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const path = cached(identity, 'git-system-config') ?? (await run(realGit, ['var', 'GIT_CONFIG_SYSTEM'])); // undefined: unknown
//! ```

use super::child_environment::{FORWARD_TARGET_VARIABLE, environment_value};
use super::git_version_cache::{BinaryIdentity, binary_identity, cache_directory, read_cached_text, store_cached_text};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// The kind of the cache file that holds the compiled-in system configuration path.
pub const SYSTEM_CONFIG_KIND: &str = "git-system-config";

/// The longest path the cache keeps and the question accepts.
const PATH_LIMIT: usize = 4096;

/// What:
///  The path text `git var GIT_CONFIG_SYSTEM` printed, when it is one absolute path on one line.
/// Why:
///   Any other output (nothing, a relative path, several lines) is not a fact the readers can use.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const path = stdout.endsWith('\n') && !stdout.slice(0, -1).includes('\n') && isAbsolute(stdout.slice(0, -1)) ? stdout.slice(0, -1) : undefined;
/// ```
pub fn parse_path_line(stdout: &[u8]) -> Option<PathBuf> {
    let line: &[u8] = stdout.strip_suffix(b"\n")?;
    if line.is_empty() || line.len() > PATH_LIMIT || line.contains(&b'\n') || line.contains(&0) {
        return None;
    }
    let path: PathBuf = PathBuf::from(std::str::from_utf8(line).ok()?);
    return if path.is_absolute() { Some(path) } else { None };
}

/// Start the executable at `real_git` with `var GIT_CONFIG_SYSTEM` and the environment's overrides removed.
fn ask_git(real_git: &Path) -> Option<Vec<u8>> {
    let output = std::process::Command::new(real_git)
        .args(["var", "GIT_CONFIG_SYSTEM"])
        .env_clear()
        .env("LC_ALL", "C")
        .current_dir("/")
        .stdin(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .output()
        .ok()?;
    return if output.status.success() { Some(output.stdout) } else { None };
}

/// What:
///  The system configuration path of the executable at `real_git`: from the cache when the identity matches,
///  otherwise from `ask`, recorded for the next start.
/// Why:
///   The question runs once per executable; `ask` is a parameter so a control can count the starts and give a Git
///  whose prefix is not `/usr`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const path = (await readCache(dir, identity, kind)) ?? (await ask(realGit)); // undefined: unknown
/// ```
pub fn system_config_of(real_git: &Path, cache: Option<&Path>, ask: &dyn Fn(&Path) -> Option<Vec<u8>>) -> Option<PathBuf> {
    let identity: BinaryIdentity = binary_identity(real_git)?;
    if let Some(directory) = cache
        && let Some(path) = read_cached_text(directory, &identity, SYSTEM_CONFIG_KIND).and_then(|text| { return parse_path_line(&[text.as_slice(), b"\n"].concat()); })
    {
        return Some(path);
    }
    let answered: Vec<u8> = ask(real_git)?;
    let path: PathBuf = parse_path_line(answered.as_slice())?;
    // The identity is taken again after the start, so an executable replaced during it is never recorded under the old one.
    if let Some(directory) = cache
        && binary_identity(real_git).as_ref() == Some(&identity)
    {
        store_cached_text(directory, &identity, SYSTEM_CONFIG_KIND, path.as_os_str().as_encoded_bytes());
    }
    return Some(path);
}

/// What:
///  The system configuration path the Git a child would run reads, learned from the forward-target variable the wrapper
///  puts in every child's environment.
/// Why:
///   `None` (no variable, no identity, no answer) makes the readers decline whenever the system file would be read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const path = systemConfigFor(env); // undefined: unknown
/// ```
pub fn system_config_for(variables: &[(OsString, OsString)]) -> Option<PathBuf> {
    let target: OsString = environment_value(variables, FORWARD_TARGET_VARIABLE)?;
    return system_config_of(Path::new(&target), cache_directory(variables).as_deref(), &ask_git);
}

#[cfg(test)]
#[path = "git_system_config_tests.rs"]
mod tests;
