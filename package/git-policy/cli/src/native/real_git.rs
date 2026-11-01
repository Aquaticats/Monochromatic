//! What:
//!  Find the real Git executable on PATH without ever selecting this wrapper.
//! Why:
//!  The wrapper is installed as `git` ahead of real Git;
//!  forwarding needs the
//!      first usable candidate that is not a cli-git wrapper.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const gitPath = resolveRealGit(processResolutionInputs()); // '/usr/bin/git'
//! ```

/// What:
///  `use super::...` imports sibling modules of this crate.
/// Why:
///   Inputs are read with getenv semantics and each candidate is classified by
///       the self-exclusion rules.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { classifyCandidate } from './real-git-candidate.ts';
/// ```
use super::child_environment::environment_value;
use super::real_git_candidate::{CandidateKind, classify_candidate};
/// What:
///  `OsStr`/`OsString` are borrowed/owned operating-system text of raw OS bytes
///       (siblings `&str`/`String` must be UTF-8).
/// Why:
///   PATH entries and executable paths need not be UTF-8.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Node strings; no byte-preserving equivalent.
/// ```
use std::ffi::{OsStr, OsString};
/// `Path`/`PathBuf` are borrowed/owned filesystem paths made of the same raw bytes.
use std::path::{Path, PathBuf};

/// Windows executable extensions in shell lookup order when `PATHEXT` is unset.
pub const DEFAULT_WINDOWS_PATH_EXTENSIONS: &str = ".COM;.EXE;.BAT;.CMD";

/// What:
///  Which operating-system family's lookup rules apply.
///       `#[derive(...)]` generates copying,
///  debug printing and `==`.
/// Why:
///   Executable names,
///  conventional install locations and path identity differ by
///       family;
///  passing the family as a value keeps every branch testable on one host.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Platform = 'linux' | 'darwin' | 'win32';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Platform {
    /// Linux and other Unix-like systems:
    ///  `git`,
    ///  case-sensitive paths.
    Unix,
    /// macOS:
    ///  Unix rules plus Homebrew and MacPorts locations.
    MacOs,
    /// Windows:
    ///  `git` plus `PATHEXT` extensions,
    ///  case-insensitive paths.
    Windows,
}

/// What:
///  Everything resolution reads from the process,
///  gathered into one value.
///       `Vec<PathBuf>` is an owned list of owned paths.
/// Why:
///   Tests inject a disposable PATH and executable identity instead of mutating
///       the real process environment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ResolutionInputs = { platform: Platform; path: string; pathExtensions: string;
///   currentDirectory: string; commonPaths: string[]; ownExecutable: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolutionInputs {
    /// Lookup rules to apply.
    pub platform: Platform,
    /// The `PATH` value;
    ///  empty when unset.
    pub path: OsString,
    /// The `PATHEXT` value;
    ///  only read on Windows.
    pub path_extensions: OsString,
    /// Directory that relative PATH entries are resolved against.
    pub current_directory: PathBuf,
    /// Conventional Git locations promoted to the front when PATH exposes them.
    pub common_paths: Vec<PathBuf>,
    /// This wrapper's own executable,
    ///  excluded from selection.
    pub own_executable: PathBuf,
}

/// What:
///  The failure when no PATH candidate can act as real Git.
///       `usize` counts items;
///  it is the length type of every Rust list.
/// Why:
///   The counts tell the user whether Git is missing or only wrappers were found.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class RealGitNotFoundError extends Error { candidateCount: number; skippedWrapperCount: number }
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RealGitNotFound {
    /// PATH-derived candidates examined before failure.
    pub candidate_count: usize,
    /// Candidates rejected because they are cli-git wrappers.
    pub skipped_wrapper_count: usize,
}

/// What:
///  `impl std::fmt::Display for ...` supplies Rust's "print me" interface.
/// Why:
///   The executable prints the same guidance the TypeScript resolver gave.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// toString(): string { return `Could not find a real Git executable ...`; }
/// ```
impl std::fmt::Display for RealGitNotFound {
    /// `&self` borrows the failure;
    ///  `&mut` lends the formatter for writing;
    ///  `'_` is an unnamed lifetime.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // `write!` formats into the borrowed formatter and returns its result.
        return write!(
            formatter,
            "Could not find a real Git executable after examining {} PATH candidates and \
             skipping {} cli-git wrappers. Ensure Git is installed and PATH/PATHEXT expose \
             its executable.",
            self.candidate_count, self.skipped_wrapper_count
        );
    }
}

/// An empty `impl` marks the type as a standard error value.
impl std::error::Error for RealGitNotFound {}

/// What:
///  List the file names PATH lookup tries,
///  in order.
/// Why:
///   Unix runs `git`;
///  Windows tries `git` plus each `PATHEXT` extension,
///  adding a
///       missing leading dot.
///  An extension that is not UTF-8 cannot name a file
///       portably and is skipped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function executableNames(platform: Platform, pathExtensions: string): string[];
/// ```
pub fn executable_names(platform: Platform, path_extensions: &OsStr) -> Vec<OsString> {
    if platform != Platform::Windows {
        // `vec![...]` builds an owned list; `OsString::from` copies text into OS form.
        return vec![OsString::from("git")];
    }
    // `Vec::<OsString>::new()` is an empty owned list; `mut` allows pushing.
    let mut names: Vec<OsString> = Vec::<OsString>::new();
    // `.split(...)` with a named predicate cuts the raw bytes at each semicolon.
    for piece in path_extensions.as_encoded_bytes().split(is_semicolon) {
        if piece.is_empty() {
            continue;
        }
        // What: `let Ok(text) = ... else { continue; }` keeps valid UTF-8, skips the rest.
        // Why:  Bytes can only become an OS string safely through checked text.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const text = decodeUtf8OrSkip(piece);
        // ```
        let Ok(extension) = std::str::from_utf8(piece) else {
            continue;
        };
        if extension.starts_with('.') {
            names.push(OsString::from(format!("git{extension}")));
        } else {
            names.push(OsString::from(format!("git.{extension}")));
        }
    }
    return names;
}

/// Named predicate for splitting `PATHEXT`;
///  `&u8` borrows one byte.
fn is_semicolon(byte: &u8) -> bool {
    return *byte == b';';
}

/// What:
///  List conventional Git install locations for a platform.
///       `&[(OsString, OsString)]` borrows the environment as name/value pairs.
/// Why:
///   A conventional location wins over an earlier PATH entry,
///  so a stray `git`
///       script early on PATH cannot displace the system Git.
///  Windows roots come from
///       the environment;
///  a trailing separator on a root is dropped before joining.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function commonGitPaths(platform: Platform, environment: [string, string][]): string[];
/// ```
pub fn common_git_paths(platform: Platform, environment: &[(OsString, OsString)]) -> Vec<PathBuf> {
    if platform == Platform::Unix {
        return vec![
            PathBuf::from("/usr/bin/git"),
            PathBuf::from("/usr/local/bin/git"),
        ];
    }
    if platform == Platform::MacOs {
        return vec![
            PathBuf::from("/usr/bin/git"),
            PathBuf::from("/usr/local/bin/git"),
            PathBuf::from("/opt/homebrew/bin/git"),
            PathBuf::from("/opt/local/bin/git"),
        ];
    }
    let mut roots: Vec<OsString> = Vec::<OsString>::new();
    for name in ["ProgramFiles", "ProgramW6432", "ProgramFiles(x86)"] {
        // `if let Some(x) = ...` runs only when the variable is set.
        if let Some(root) = environment_value(environment, name) {
            roots.push(root);
        }
    }
    roots.push(OsString::from("C:\\Program Files"));
    if let Some(local) = environment_value(environment, "LOCALAPPDATA") {
        let mut programs: OsString = trimmed_root(local.as_os_str());
        programs.push("\\Programs");
        roots.push(programs);
    }
    let mut paths: Vec<PathBuf> = Vec::<PathBuf>::new();
    // `&roots` lends the list so the loop does not consume it.
    for root in &roots {
        for tail in ["\\Git\\cmd\\git.exe", "\\Git\\bin\\git.exe"] {
            let mut path: OsString = trimmed_root(root.as_os_str());
            path.push(tail);
            let candidate: PathBuf = PathBuf::from(path);
            // `.contains(&candidate)` borrows the new path to skip repeated roots.
            if !paths.contains(&candidate) {
                paths.push(candidate);
            }
        }
    }
    return paths;
}

/// What:
///  Copy a Windows root without trailing `\` or `/` separators.
/// Why:
///   Joining `C:\Program Files\` and `\Git` must not produce a doubled separator
///       that no PATH candidate would equal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function trimmedRoot(root: string): string { return root.replace(/[\\/]+$/, ''); }
/// ```
fn trimmed_root(root: &OsStr) -> OsString {
    let bytes: &[u8] = root.as_encoded_bytes();
    let mut end: usize = bytes.len();
    while end > 0 && (bytes[end - 1] == b'\\' || bytes[end - 1] == b'/') {
        end -= 1;
    }
    // What: `std::str::from_utf8` checks the kept bytes; `match` picks by outcome.
    // Why:  Cutting ASCII separators off valid text leaves valid text; a root that is
    //       not UTF-8 is returned whole rather than re-encoded.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return root.slice(0, end);
    // ```
    match std::str::from_utf8(&bytes[..end]) {
        Ok(text) => return OsString::from(text),
        Err(_) => return root.to_os_string(),
    }
}

/// What:
///  The form of a path used to compare candidates for equality.
/// Why:
///   Windows paths are case-insensitive;
///  ASCII case is folded there.
///  Other
///       platforms compare the exact bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function candidateIdentity(path: string, platform: Platform): string;
/// ```
fn candidate_identity(path: &Path, platform: Platform) -> OsString {
    if platform == Platform::Windows {
        return path.as_os_str().to_ascii_lowercase();
    }
    return path.as_os_str().to_os_string();
}

/// What:
///  Rebuild a path from its components.
/// Why:
///   This removes `.` segments and repeated separators,
///  so `/usr/./bin//git` and
///       `/usr/bin/git` compare equal.
///  `..` is kept:
///  resolving it without the
///       filesystem would be wrong across symbolic links.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lexicallyTidied(path: string): string;
/// ```
fn lexically_tidied(path: &Path) -> PathBuf {
    let mut result: PathBuf = PathBuf::new();
    // `.components()` yields each path segment in order.
    for component in path.components() {
        result.push(component.as_os_str());
    }
    return result;
}

/// What:
///  Build the ordered,
///  de-duplicated list of absolute candidates.
/// Why:
///   Every PATH directory contributes each executable name in order;
///  conventional
///       locations that PATH exposes are promoted to the front in their PATH spelling.
///       Relative PATH entries are resolved against the current directory,
///  as a shell does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function candidateSequence(inputs: ResolutionInputs): string[];
/// ```
pub fn candidate_sequence(inputs: &ResolutionInputs) -> Vec<PathBuf> {
    let names: Vec<OsString> =
        executable_names(inputs.platform, inputs.path_extensions.as_os_str());
    let mut path_candidates: Vec<PathBuf> = Vec::<PathBuf>::new();
    // `std::env::split_paths` cuts PATH by the host platform's separator.
    for directory in std::env::split_paths(inputs.path.as_os_str()) {
        for name in &names {
            // `.join` appends a segment; an absolute `directory` replaces the base.
            let candidate: PathBuf = inputs.current_directory.join(&directory).join(name);
            path_candidates.push(lexically_tidied(candidate.as_path()));
        }
    }
    let mut sequence: Vec<PathBuf> = Vec::<PathBuf>::new();
    for common in &inputs.common_paths {
        let wanted: OsString = candidate_identity(common.as_path(), inputs.platform);
        for candidate in &path_candidates {
            if candidate_identity(candidate.as_path(), inputs.platform) == wanted
                && !sequence.contains(candidate)
            {
                // `.clone()` copies the path so both lists own their entries.
                sequence.push(candidate.clone());
            }
        }
    }
    for candidate in path_candidates {
        if !sequence.contains(&candidate) {
            sequence.push(candidate);
        }
    }
    return sequence;
}

/// What:
///  Select the first candidate that is real Git.
///       `Result<PathBuf, RealGitNotFound>` is the selected path or the typed failure.
/// Why:
///   Wrappers are skipped and counted,
///  unusable entries are skipped,
///  and the
///       first remaining candidate in priority order is forwarded to.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function resolveRealGit(inputs: ResolutionInputs): string;
/// ```
pub fn resolve_real_git(inputs: &ResolutionInputs) -> Result<PathBuf, RealGitNotFound> {
    let candidates: Vec<PathBuf> = candidate_sequence(inputs);
    let mut skipped_wrapper_count: usize = 0;
    for candidate in &candidates {
        let kind: CandidateKind =
            classify_candidate(candidate.as_path(), inputs.own_executable.as_path());
        if kind == CandidateKind::RealGit {
            // `Ok(...)` is the success variant carrying an owned copy of the path.
            return Ok(candidate.clone());
        }
        if kind == CandidateKind::Wrapper {
            skipped_wrapper_count += 1;
        }
    }
    // `Err(...)` is the failure variant carrying the evidence counts.
    return Err(RealGitNotFound {
        candidate_count: candidates.len(),
        skipped_wrapper_count,
    });
}

/// What:
///  The lookup rules of the platform this executable was compiled for.
///       `cfg!(...)` is a compile-time `true`/`false` about the target platform.
/// Why:
///   Production uses the host's rules;
///  tests pass a platform explicitly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function hostPlatform(): Platform { return process.platform; }
/// ```
pub fn host_platform() -> Platform {
    if cfg!(windows) {
        return Platform::Windows;
    }
    if cfg!(target_os = "macos") {
        return Platform::MacOs;
    }
    return Platform::Unix;
}

/// What:
///  Gather resolution inputs from the running process.
///       `std::io::Result<T>` is `Result<T, std::io::Error>`.
/// Why:
///   The wrapper must know its own executable to exclude it;
///  if the operating
///       system cannot report it (or the current directory),
///  resolution fails rather
///       than risk selecting itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function processResolutionInputs(environment: [string, string][]): ResolutionInputs;
/// ```
pub fn process_resolution_inputs(
    environment: &[(OsString, OsString)],
) -> std::io::Result<ResolutionInputs> {
    let platform: Platform = host_platform();
    // A trailing `?` returns the OS error to our caller, or unwraps the value.
    let own_executable: PathBuf = std::env::current_exe()?;
    let current_directory: PathBuf = std::env::current_dir()?;
    // `.unwrap_or_default()` substitutes an empty value when the variable is unset.
    let path: OsString = environment_value(environment, "PATH").unwrap_or_default();
    let path_extensions: OsString = environment_value(environment, "PATHEXT")
        .unwrap_or(OsString::from(DEFAULT_WINDOWS_PATH_EXTENSIONS));
    return Ok(ResolutionInputs {
        platform,
        path,
        path_extensions,
        current_directory,
        common_paths: common_git_paths(platform, environment),
        own_executable,
    });
}

/// Disposable-PATH controls stay out of the release executable.
#[cfg(test)]
#[path = "real_git_tests.rs"]
mod tests;
