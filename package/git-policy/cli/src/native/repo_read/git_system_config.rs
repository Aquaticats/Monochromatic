//! What:
//!  The system configuration file that one real Git executable reads when `GIT_CONFIG_SYSTEM` is not set,
//!  learned once per executable identity and cached beside the `git version` cache.
//! Why:
//!  Git compiles the path in (`ETC_GITCONFIG`,
//! `/etc/gitconfig` for a build with prefix `/usr`,
//! and under the prefix
//!  for any other build,
//! resolved against the executable's own location in a runtime-prefix build),
//! so no reader may
//!  assume `/etc/gitconfig`.
//!  `git var GIT_CONFIG_SYSTEM` prints the path the executable would read,
//! with the environment's own overrides
//!  removed from its start,
//! so the path is a property of the executable's bytes and location,
//! which the identity of
//!  `git_version_cache` captures.
//!  An executable without an identity (a script,
//! a shim) or an answer that is not an absolute path is unknown,
//! and
//!  the readers then leave the question to Git.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const path = cached(identity, 'git-system-config') ?? (await run(realGit, ['var', 'GIT_CONFIG_SYSTEM'])); // undefined: unknown
//! ```

use crate::child_environment::{FORWARD_TARGET_VARIABLE, environment_value};
use crate::git_version_cache::{BinaryIdentity, binary_identity, cache_directory, read_cached_text, store_cached_text};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// The kind of the cache file that holds the compiled-in system configuration path.
pub const SYSTEM_CONFIG_KIND: &str = "git-system-config";

/// The kind of the cache file that holds the compiled-in system attributes path.
pub const SYSTEM_ATTRIBUTES_KIND: &str = "git-system-attributes";

/// The longest path the cache keeps and the question accepts.
const PATH_LIMIT: usize = 4096;

/// What:
///  The path text `git var GIT_CONFIG_SYSTEM` printed,
/// when it is one absolute path on one line.
/// Why:
///   Any other output (nothing,
/// a relative path,
/// several lines) is not a fact the readers can use.
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
    return ask_git_var(real_git, "GIT_CONFIG_SYSTEM");
}

/// Start the executable at `real_git` with `var GIT_ATTR_SYSTEM` and the environment's overrides removed.
fn ask_git_attributes(real_git: &Path) -> Option<Vec<u8>> {
    return ask_git_var(real_git, "GIT_ATTR_SYSTEM");
}

/// Start the executable at `real_git` with `var <variable>` and the environment's overrides removed.
fn ask_git_var(real_git: &Path, variable: &str) -> Option<Vec<u8>> {
    let mut command = std::process::Command::new(real_git);
    command
        .args(["var", variable])
        .env_clear()
        .env("LC_ALL", "C")
        .current_dir("/")
        .stdin(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    // The environment was cleared, so the marker of this start is added after it (`started_as_git.rs`), and it is a start of
    //  the wrapper's own reading: nothing of the person's runs under it.
    crate::started_as_git::mark_real_git_start(&mut command, crate::started_as_git::StartMode::Query);
    let output = crate::captured_child_run_to_its_end::output_of_within_bound(&mut command, crate::wait_ceilings::PLUMBING_STEP_CEILING).ok()?;
    return if output.status.success() { Some(output.stdout) } else { None };
}

/// What:
///  The system configuration path of the executable at `real_git`:
/// from the cache when the identity matches,
///  otherwise from `ask`,
/// recorded for the next start.
/// Why:
///   The question runs once per executable;
/// `ask` is a parameter so a control can count the starts and give a Git
///  whose prefix is not `/usr`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const path = (await readCache(dir, identity, kind)) ?? (await ask(realGit)); // undefined: unknown
/// ```
pub fn system_config_of(real_git: &Path, cache: Option<&Path>, ask: &dyn Fn(&Path) -> Option<Vec<u8>>) -> Option<PathBuf> {
    return learned_path(real_git, cache, SYSTEM_CONFIG_KIND, ask);
}

/// What:
///  The system attributes file of the executable at `real_git`,
/// learned like the system configuration path.
/// Why:
///   `GIT_CONFIG_SYSTEM` moves the configuration file and nothing else:
/// the attributes file is at `$(prefix)/etc/gitattributes`
///  of that build,
/// which only the executable itself reports (`git var GIT_ATTR_SYSTEM`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const path = (await readCache(dir, identity, 'git-system-attributes')) ?? (await run(realGit, ['var', 'GIT_ATTR_SYSTEM'])); // undefined: unknown
/// ```
pub fn system_attributes_of(real_git: &Path, cache: Option<&Path>, ask: &dyn Fn(&Path) -> Option<Vec<u8>>) -> Option<PathBuf> {
    return learned_path(real_git, cache, SYSTEM_ATTRIBUTES_KIND, ask);
}

/// A path the executable at `real_git` reports for itself,
/// cached under `kind` for its identity.
fn learned_path(real_git: &Path, cache: Option<&Path>, kind: &str, ask: &dyn Fn(&Path) -> Option<Vec<u8>>) -> Option<PathBuf> {
    let identity: BinaryIdentity = binary_identity(real_git)?;
    if let Some(directory) = cache
        && let Some(text) = read_cached_text(directory, &identity, kind)
        && let Some(path) = parse_path_line(&[text.as_slice(), b"\n"].concat())
    {
        return Some(path);
    }
    let answered: Vec<u8> = ask(real_git)?;
    let path: PathBuf = parse_path_line(answered.as_slice())?;
    // The identity is taken again after the start, so an executable replaced during it is never recorded under the old one.
    if let Some(directory) = cache
        && binary_identity(real_git).as_ref() == Some(&identity)
    {
        store_cached_text(directory, &identity, kind, path.as_os_str().as_encoded_bytes());
    }
    return Some(path);
}

/// What:
///  `variables` with the forward-target variable added for `real_git` when the variables do not name one.
/// Why:
///   The wrapper process learns the system configuration path from the real Git named by that variable.
///  The environment `entry.rs` builds always carries it,
/// but a resident process (the service) and callers that hold only a
///  Git path and a bare overlay do not,
/// and without it every read there would decline;
/// the caller knows the Git it runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const named = { [FORWARD_TARGET]: realGit, ...variables }; // the variables win when they name one
/// ```
pub fn with_real_git(variables: &[(OsString, OsString)], real_git: &Path) -> Vec<(OsString, OsString)> {
    let mut named: Vec<(OsString, OsString)> = variables.to_vec();
    if environment_value(variables, FORWARD_TARGET_VARIABLE).is_none() {
        named.push((OsString::from(FORWARD_TARGET_VARIABLE), real_git.as_os_str().to_os_string()));
    }
    return named;
}

/// What:
///  The system configuration path the Git a child would run reads,
/// learned from the forward-target variable the wrapper
///  puts in every child's environment.
/// Why:
///   `None` (no variable,
/// no identity,
/// no answer) makes the readers decline whenever the system file would be read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const path = systemConfigFor(env); // undefined: unknown
/// ```
pub fn system_config_for(variables: &[(OsString, OsString)]) -> Option<PathBuf> {
    let target: OsString = environment_value(variables, FORWARD_TARGET_VARIABLE)?;
    return system_config_of(Path::new(&target), cache_directory(variables).as_deref(), &ask_git);
}

/// What:
///  The system attributes path the Git a child would run reads,
/// learned from the forward-target variable.
/// Why:
///   `None` (no variable,
/// no identity,
/// no answer) makes the content decision refuse whenever that file would be read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const path = systemAttributesFor(env); // undefined: unknown
/// ```
pub fn system_attributes_for(variables: &[(OsString, OsString)]) -> Option<PathBuf> {
    let target: OsString = environment_value(variables, FORWARD_TARGET_VARIABLE)?;
    return system_attributes_of(Path::new(&target), cache_directory(variables).as_deref(), &ask_git_attributes);
}

#[cfg(test)]
#[path = "git_system_config_tests.rs"]
mod tests;
