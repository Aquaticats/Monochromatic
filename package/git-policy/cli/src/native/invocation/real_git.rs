//! What:
//!  Find the real Git executable on PATH without ever selecting this wrapper.
//! Why:
//!  The wrapper is installed as `git` ahead of real Git;
//!  forwarding needs the
//!      first usable candidate that is not a monochromatic-git-policy-cli wrapper.
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
use crate::child_environment::environment_value;
/// The two answers to "has this process a controlling terminal",
///  which decides the overlay of a command the caller typed.
use crate::control_terminal::{Terminal, control_terminal};
use crate::real_git_candidate::{CandidateKind, classify_candidate};
use crate::real_git_record::service_unit_file;
/// `pub use` re-exports names so callers keep importing the resolution from this module.
pub use crate::real_git_resolution::{RecordedGit, ResolvedGit, resolve_real_git};
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
///  Every fact about this process that planning and resolution read,
///  gathered into one value.
///       `Vec<PathBuf>` is an owned list of owned paths.
/// Why:
///   Tests inject a disposable PATH,
///  executable identity,
/// current directory and terminal
///       instead of mutating the real process environment;
///  the planned action stays a function of these values alone,
///  so a control states which case it pins instead of inheriting the runner's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ResolutionInputs = { platform: Platform; path: string; pathExtensions: string;
///   currentDirectory: string; commonPaths: string[]; ownExecutable: string; terminal: Terminal };
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
    /// The unit file of the commit service,
    ///  whose `--git` option records the real Git the wrapper last set the service up with;
    ///  nothing when the runtime directory is unknown.
    ///  It is read only when `PATH` offers no real Git.
    pub service_unit: Option<PathBuf>,
    /// Whether this process has a controlling terminal,
    ///  read once when these inputs are gathered.
    ///  A command without one gets the never-prompt pair in the overlay of every Git child,
    ///  so Git's and git-lfs's prompt programs cannot open a desktop window nobody can answer;
    ///  a command at a terminal keeps Git's own prompting.
    pub terminal: Terminal,
}

/// What:
///  One PATH candidate the search examined and what it found.
///       `CandidateKind` is the verdict (real Git,
/// wrapper,
/// unusable).
/// Why:
///   The failure message lists every candidate with its verdict,
/// so the person can see which entries led back to monochromatic-git-policy-cli
///  and which were not executable files at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SearchedCandidate = { path: string; kind: 'real-git' | 'wrapper' | 'unusable' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchedCandidate {
    /// The candidate's absolute path as the search built it from PATH.
    pub path: PathBuf,
    /// The verdict about it.
    pub kind: CandidateKind,
}

/// What:
///  The failure when no PATH candidate can act as real Git.
///       `usize` counts items;
///  it is the length type of every Rust list.
///       `Vec<SearchedCandidate>` is an owned list of owned records (siblings:
/// `&[T]`,
/// a borrowed view).
/// Why:
///   The counts tell the user whether Git is missing or only wrappers were found,
/// and the list and the `PATH` value
///  name what was searched.
///  The record owns text,
/// so it is not `Copy`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class RealGitNotFoundError extends Error { candidateCount: number; skippedWrapperCount: number; searched: SearchedCandidate[]; path: string }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RealGitNotFound {
    /// PATH-derived candidates examined before failure.
    pub candidate_count: usize,
    /// Candidates rejected because they are monochromatic-git-policy-cli wrappers.
    pub skipped_wrapper_count: usize,
    /// Every candidate examined,
    /// in the order the search took them,
    /// with its verdict.
    pub searched: Vec<SearchedCandidate>,
    /// The `PATH` value the search read;
    /// empty when unset.
    pub path: OsString,
    /// The Git the commit service's unit records and the verdict on it,
    ///  when the unit names one that was refused;
    ///  nothing when there is no record or it is not readable.
    pub recorded: Option<RecordedGit>,
}

/// What:
///  The words for one candidate's verdict.
/// Why:
///   The message says why a candidate was passed over in terms a person can act on,
/// with no internal names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reason = { wrapper: 'leads back to monochromatic-git-policy-cli ...', unusable: 'is not an executable file ...' }[kind];
/// ```
fn verdict_words(kind: CandidateKind) -> &'static str {
    match kind {
        CandidateKind::Wrapper => {
            return "skipped, it leads back to monochromatic-git-policy-cli (this executable, a link or copy of it, its toolchain launcher, or a script that starts it)";
        }
        CandidateKind::Unusable => return "skipped, it is missing, not an executable file, or unreadable",
        CandidateKind::RealGit => return "usable",
    }
}

/// What:
///  `impl std::fmt::Display for ...` supplies Rust's "print me" interface.
/// Why:
///   The executable prints what was searched,
/// why nothing qualified,
/// and every way to fix it,
/// naming the `PATH` value as
///  the input.
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
        // `write!` formats into the borrowed formatter; `?` returns early if writing failed.
        write!(
            formatter,
            "Could not find a real Git executable after examining {} PATH candidates and \
             skipping {} monochromatic-git-policy-cli wrappers, so no Git command was run. PATH is {}.",
            self.candidate_count,
            self.skipped_wrapper_count,
            if self.path.is_empty() { String::from("empty or unset") } else { self.path.to_string_lossy().into_owned() }
        )?;
        for candidate in &self.searched {
            write!(formatter, "\n  {}: {}", candidate.path.display(), verdict_words(candidate.kind))?;
        }
        // `if let Some(x) = &field` runs only when a unit named a Git;
        //  the line says where the name was read and why it was not used.
        if let Some(recorded) = &self.recorded {
            write!(
                formatter,
                "\n  {}: recorded by the commit service unit {} as the Git it starts, {}",
                recorded.candidate.path.display(),
                recorded.unit_file.display(),
                verdict_words(recorded.candidate.kind)
            )?;
        }
        // The last write is the function's result.
        return write!(
            formatter,
            "\nmonochromatic-git-policy-cli never starts itself as Git, so a candidate that leads back to monochromatic-git-policy-cli is passed over on purpose. \
             To fix it, install Git; or add the directory that holds the real Git executable (for example /usr/bin) to \
             the PATH of the program that ran this command; or, when an entry of that PATH should not be used, remove it."
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
///  Select the first `PATH` candidate that is real Git.
///       `Result<PathBuf, RealGitNotFound>` is the selected path or the typed failure.
/// Why:
///   Wrappers are skipped and counted,
///  unusable entries are skipped,
///  and the
///       first remaining candidate in priority order is forwarded to.
///  This is the search of `PATH` alone;
///  every start of Git goes through `resolve_real_git`,
///  which adds the recorded fallback.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function searchPathForRealGit(inputs: ResolutionInputs): string;
/// ```
pub fn search_path_for_real_git(inputs: &ResolutionInputs) -> Result<PathBuf, RealGitNotFound> {
    let candidates: Vec<PathBuf> = candidate_sequence(inputs);
    let mut skipped_wrapper_count: usize = 0;
    // `mut` lets the loop record each verdict for the failure message.
    let mut searched: Vec<SearchedCandidate> = Vec::<SearchedCandidate>::new();
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
        searched.push(SearchedCandidate { path: candidate.clone(), kind });
    }
    // `Err(...)` is the failure variant carrying the evidence: counts, every verdict and the PATH read.
    return Err(RealGitNotFound {
        candidate_count: candidates.len(),
        skipped_wrapper_count,
        searched,
        path: inputs.path.clone(),
        recorded: None,
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
///  The unit file of the commit service below a runtime directory,
///  when that directory is an absolute path.
/// Why:
///   A relative runtime directory would name a different file for each directory the wrapper runs in,
///  which is no record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function serviceUnitBelow(runtime: string): string | undefined { return isAbsolute(runtime) ? serviceUnitFile(runtime) : undefined; }
/// ```
fn service_unit_below(runtime: OsString) -> Option<PathBuf> {
    let directory: PathBuf = PathBuf::from(runtime);
    if !directory.is_absolute() {
        // `None` says there is no record to look for.
        return None;
    }
    // `Some(...)` wraps the unit file's path.
    return Some(service_unit_file(directory.as_path()));
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
    // Identification only: this path is excluded from the search for the real Git so the wrapper never selects itself;
    //  nothing is started from it (doc/planning/cli-git-no-self-invocation.md).
    #[allow(
        clippy::disallowed_methods,
        reason = "identifies the wrapper so the search for the real Git skips it; nothing is started from the path"
    )]
    let own_executable: PathBuf = std::env::current_exe()?;
    crate::service_route::remember_own_executable(own_executable.as_path());
    let current_directory: PathBuf = std::env::current_dir()?;
    // `.unwrap_or_default()` substitutes an empty value when the variable is unset.
    let path: OsString = environment_value(environment, "PATH").unwrap_or_default();
    let path_extensions: OsString = environment_value(environment, "PATHEXT")
        .unwrap_or(OsString::from(DEFAULT_WINDOWS_PATH_EXTENSIONS));
    // What: The unit file is looked for below an absolute runtime directory only.
    // Why:  A relative one would name a different file for each directory the wrapper runs in, which is no record.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const runtime = env.XDG_RUNTIME_DIR; const serviceUnit = runtime && isAbsolute(runtime) ? serviceUnitFile(runtime) : undefined;
    // ```
    let service_unit: Option<PathBuf> =
        environment_value(environment, "XDG_RUNTIME_DIR").and_then(service_unit_below);
    // What: The controlling terminal is read here, once, beside the other process facts.
    // Why:  Every plan this process makes is decided from these inputs, so a control hands in the answer it pins
    //  instead of taking whatever terminal the test runner happens to have.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const terminal = hasControllingTerminal() ? 'present' : 'absent';
    // ```
    let terminal: Terminal = control_terminal();
    return Ok(ResolutionInputs {
        platform,
        path,
        path_extensions,
        current_directory,
        common_paths: common_git_paths(platform, environment),
        own_executable,
        service_unit,
        terminal,
    });
}

/// Disposable-PATH controls stay out of the release executable.
#[cfg(test)]
#[path = "real_git_tests.rs"]
mod tests;

/// Controls for the fallback to the Git the commit service's unit records.
#[cfg(test)]
#[path = "real_git_fallback_tests.rs"]
mod fallback_tests;
