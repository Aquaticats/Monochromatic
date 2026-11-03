//! What:
//! A reader of a Git repository's `HEAD`,
//! branch configuration and references (loose files and `packed-refs`),
//! for the plain file layouts only.
//! Why:
//! The shadow runs before every cargo command,
//! and the owner's rules are "Invoke real git as rarely as possible."
//! and
//! "embed instead of spawning" (`package/git-policy/cli/SPEC.md`).
//! Every function answers `Err(reason)` for a layout or setting it cannot read exactly:
//! the reftable reference format,
//! any repository extension,
//! a configuration `include` or `includeIf`,
//! a symbolic reference where a commit was expected,
//! an ambiguous short name,
//! a reference outside `refs/heads`, `refs/remotes` and `refs/tags`,
//! and the `GIT_*` variables that move the repository or the configuration.
//! The caller then asks Git once.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // git-refs.ts
//! export function findLayout(root: string, env: NativeEnvironment): GitLayout  // throws: ask Git
//! export function lookupRef(layout: GitLayout, name: string): string | undefined  // throws: ask Git
//! export function trackingRef(config: ParsedConfig, layout: GitLayout, branch: string): string | undefined  // throws: ask Git
//! ```

use std::fs;
use std::path::{Path, PathBuf};

use crate::git_config::{ParsedConfig, parse_config};
use crate::git_objects::OBJECT_NAME_DIGITS;
use crate::git_refspec::{is_plain_ref_name, map_through_refspecs};

/// The environment variables that move the repository,
/// its references, its objects or its configuration;
/// when one is set the answer is Git's, not this reader's.
const BLOCKING_VARIABLES: [&str; 17] = [
    "GIT_DIR",
    "GIT_COMMON_DIR",
    "GIT_WORK_TREE",
    "GIT_NAMESPACE",
    "GIT_CONFIG",
    "GIT_CONFIG_GLOBAL",
    "GIT_CONFIG_SYSTEM",
    "GIT_CONFIG_NOSYSTEM",
    "GIT_CONFIG_COUNT",
    "GIT_CONFIG_PARAMETERS",
    "GIT_CEILING_DIRECTORIES",
    "GIT_DISCOVERY_ACROSS_FILESYSTEM",
    "GIT_OBJECT_DIRECTORY",
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    "GIT_REPLACE_REF_BASE",
    "GIT_NO_REPLACE_OBJECTS",
    "GIT_REF_PARANOIA",
];

/// The reference namespaces whose files live in the common directory and that the reader looks up.
const SHARED_NAMESPACES: [&str; 3] = ["refs/heads/", "refs/remotes/", "refs/tags/"];

/// The place the process environment puts Git's inputs,
/// so tests can give a disposable home.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeEnvironment {
    /// The home directory (`HOME`),
    /// where `~/.gitconfig` and `~/.config/git/config` live.
    pub home: Option<PathBuf>,
    /// `XDG_CONFIG_HOME`, when set and not empty.
    pub xdg_config_home: Option<PathBuf>,
    /// The system configuration file.
    /// This is `/etc/gitconfig`, which distribution builds of Git read;
    /// a Git built for another prefix reads elsewhere and is not seen
    /// (the limit is stated in the README of this package).
    pub system_config: PathBuf,
    /// The name of the first blocking variable that is set.
    pub blocking_variable: Option<String>,
}

impl NativeEnvironment {
    /// What:
    /// `pub fn from_process() -> NativeEnvironment` reads the inputs from the process environment.
    /// Why:
    /// Production code asks the process; tests build the struct by hand.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static fromProcess(): NativeEnvironment
    /// ```
    pub fn from_process() -> NativeEnvironment {
        let home: Option<PathBuf> = std::env::var_os("HOME").map(PathBuf::from);
        let xdg_config_home: Option<PathBuf> = match std::env::var_os("XDG_CONFIG_HOME") {
            Some(value) if !value.is_empty() => Some(PathBuf::from(value)),
            _ => None,
        };
        let blocking_variable: Option<String> = BLOCKING_VARIABLES.iter().find(|name| { return std::env::var_os(name).is_some(); }).map(|name| { return name.to_string(); });
        return NativeEnvironment { home, xdg_config_home, system_config: PathBuf::from("/etc/gitconfig"), blocking_variable };
    }
}

/// Where a repository's files are.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitLayout {
    /// The directory of this working tree's own files (`HEAD`).
    pub git_dir: PathBuf,
    /// The directory of the shared files (`config`, `refs`, `packed-refs`, `objects`).
    pub common_dir: PathBuf,
}

/// What `HEAD` holds.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Head {
    /// A symbolic reference to a branch,
    /// with the full reference name.
    Branch(
        /// The full reference name, for example `refs/heads/main`.
        String,
    ),
    /// A detached `HEAD`: an object name.
    Detached(
        /// The object name.
        String,
    ),
}

/// The base the reader chose,
/// before the lockfile is read.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeBase {
    /// How Git would spell it.
    pub spelling: String,
    /// The commit,
    /// or the message to show when the base names no commit.
    pub commit: Result<String, String>,
}

/// What:
/// `fn trim_space(text: &str) -> &str` trims the characters Git's `isspace` knows (space, tab, newline, carriage return).
/// Why:
/// Rust's `trim` also drops Unicode spaces such as the no-break space,
/// which are legal inside a reference name or a path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const trimSpace = (text: string): string => text.replace(/^[ \t\r\n]+|[ \t\r\n]+$/g, '');
/// ```
fn trim_space(text: &str) -> &str {
    return text.trim_matches(|character| { return matches!(character, ' ' | '\t' | '\n' | '\r'); });
}

/// What:
/// `fn is_object_name(text: &str) -> bool` says whether the text is a SHA-1 object name of lowercase or uppercase hexadecimal digits.
/// Why:
/// Other lengths belong to other object formats,
/// which the reader leaves to Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isObjectName = (text: string): boolean => /^[0-9a-fA-F]{40}$/.test(text);
/// ```
fn is_object_name(text: &str) -> bool {
    return text.len() == OBJECT_NAME_DIGITS && text.chars().all(|character| { return character.is_ascii_hexdigit(); });
}

/// What:
/// `fn cannot_read(path: &Path, error: &std::io::Error) -> String` words a read failure with the path.
/// Why:
/// Every error names what could not be read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const cannotRead = (path: string, error: Error) => `${path} cannot be read: ${error}`;
/// ```
fn cannot_read(path: &Path, error: &std::io::Error) -> String {
    return format!("{} cannot be read: {error}", path.display());
}

/// What:
/// `fn read_text(path: &Path) -> Result<String, String>` reads a text file.
/// Why:
/// Failures carry the path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const readText = (path: string): string => readFileSync(path, 'utf8');
/// ```
fn read_text(path: &Path) -> Result<String, String> {
    return fs::read_to_string(path).map_err(|error| { return cannot_read(path, &error); });
}

/// What:
/// `fn read_optional(path: &Path) -> Result<Option<String>, String>` reads a text file that may be absent.
/// Why:
/// Only a missing file is absence;
/// a permission error or invalid UTF-8 is a failure the reader does not guess around.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const readOptional = (path: string): string | undefined => ...
/// ```
fn read_optional(path: &Path) -> Result<Option<String>, String> {
    return match fs::read_to_string(path) {
        Ok(text) => Ok(Some(text)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(cannot_read(path, &error)),
    };
}

/// What:
/// `fn check_git_directory(directory: &Path) -> Result<(), String>` requires the files every Git directory has.
/// Why:
/// A directory without `HEAD`, `objects` and `refs` is not one Git would accept.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkGitDirectory(directory: string): void  // throws
/// ```
fn check_git_directory(directory: &Path) -> Result<(), String> {
    for required in ["HEAD", "objects", "refs"] {
        if !directory.join(required).exists() {
            return Err(format!("{} has no {required}, so it is not a Git directory", directory.display()));
        }
    }
    return Ok(());
}

/// What:
/// `pub fn find_layout(root: &Path, environment: &NativeEnvironment) -> Result<GitLayout, String>` finds the Git directory of a
/// working tree and its common directory.
/// Why:
/// A `.git` file says `gitdir: ` and then a path that keeps any leading space (only trailing whitespace is dropped);
/// a `commondir` file is absent or it is read exactly, and any other failure is the caller's cue to ask Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function findLayout(root: string, env: NativeEnvironment): GitLayout  // throws: ask Git
/// ```
pub fn find_layout(root: &Path, environment: &NativeEnvironment) -> Result<GitLayout, String> {
    if let Some(name) = &environment.blocking_variable {
        return Err(format!("{name} is set, so only Git can say where the repository is"));
    }
    let dot_git: PathBuf = root.join(".git");
    let metadata = fs::metadata(&dot_git).map_err(|error| { return cannot_read(&dot_git, &error); })?;
    let git_dir: PathBuf = if metadata.is_dir() {
        dot_git
    } else {
        let text: String = read_text(&dot_git)?;
        let Some(target) = text.strip_prefix("gitdir: ") else {
            return Err(format!("{} is a file but does not start with \"gitdir: \"", dot_git.display()));
        };
        let path: &str = target.trim_end_matches(|character| { return matches!(character, ' ' | '\t' | '\n' | '\r'); });
        if path.is_empty() {
            return Err(format!("{} names no directory", dot_git.display()));
        }
        root.join(path)
    };
    let common_dir: PathBuf = match read_optional(&git_dir.join("commondir"))? {
        Some(text) => git_dir.join(text.trim_end_matches(|character| { return matches!(character, ' ' | '\t' | '\n' | '\r'); })),
        None => git_dir.clone(),
    };
    check_git_directory(&common_dir)?;
    return Ok(GitLayout { git_dir, common_dir });
}

/// What:
/// `fn check_repository_settings(config: &ParsedConfig) -> Result<(), String>` refuses the settings that change what the
/// reader would have to read.
/// Why:
/// Any `extensions.*` variable (a different reference format, object format, worktree configuration or one this reader has never
/// heard of) and any repository format other than 0 or 1 are Git's to judge.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkRepositorySettings(config: ParsedConfig): void  // throws: ask Git
/// ```
fn check_repository_settings(config: &ParsedConfig) -> Result<(), String> {
    if let Some(entry) = config.entries.iter().find(|entry| { return entry.name.starts_with("extensions."); }) {
        return Err(format!("the configuration sets {}, a repository extension that only Git applies", entry.name));
    }
    if let Some(format) = config.last("core", None, "repositoryformatversion")
        && format != "0"
        && format != "1"
    {
        return Err(format!("core.repositoryformatversion is {format:?}, which only Git judges"));
    }
    return Ok(());
}

/// What:
/// `pub fn read_config(layout: &GitLayout, environment: &NativeEnvironment) -> Result<ParsedConfig, String>` reads the
/// configuration files in Git's order (system, XDG or home, home, repository) and refuses what only Git can apply.
/// Why:
/// Later files override earlier ones;
/// an `include`, an unreadable file or a refused setting makes the caller ask Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function readConfig(layout: GitLayout, env: NativeEnvironment): ParsedConfig  // throws: ask Git
/// ```
pub fn read_config(layout: &GitLayout, environment: &NativeEnvironment) -> Result<ParsedConfig, String> {
    let mut files: Vec<PathBuf> = vec![environment.system_config.clone()];
    if let Some(xdg) = &environment.xdg_config_home {
        files.push(xdg.join("git/config"));
    } else if let Some(home) = &environment.home {
        files.push(home.join(".config/git/config"));
    }
    if let Some(home) = &environment.home {
        files.push(home.join(".gitconfig"));
    }
    files.push(layout.common_dir.join("config"));
    let mut merged: ParsedConfig = ParsedConfig::default();
    for file in &files {
        let Some(text) = read_optional(file)? else {
            continue;
        };
        let parsed: ParsedConfig = parse_config(&text).map_err(|message| { return format!("{}: {message}", file.display()); })?;
        merged.extend_with(parsed);
    }
    if merged.has_include {
        return Err("a configuration file has include or includeIf, which only Git follows".to_string());
    }
    check_repository_settings(&merged)?;
    return Ok(merged);
}

/// What:
/// `pub fn read_head(layout: &GitLayout) -> Result<Head, String>` reads `HEAD`.
/// Why:
/// It is a symbolic reference or an object name;
/// anything else is Git's to explain.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function readHead(layout: GitLayout): Head  // throws
/// ```
pub fn read_head(layout: &GitLayout) -> Result<Head, String> {
    let path: PathBuf = layout.git_dir.join("HEAD");
    let text: String = read_text(&path)?;
    let trimmed: &str = trim_space(&text);
    if let Some(name) = trimmed.strip_prefix("ref:") {
        return Ok(Head::Branch(trim_space(name).to_string()));
    }
    if is_object_name(trimmed) {
        return Ok(Head::Detached(trimmed.to_string()));
    }
    return Err(format!("{} holds neither a reference nor an object name", path.display()));
}

/// What:
/// `fn check_ref_name(name: &str) -> Result<(), String>` refuses a name that could leave the repository directory.
/// Why:
/// The name becomes a path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkRefName(name: string): void  // throws
/// ```
fn check_ref_name(name: &str) -> Result<(), String> {
    let escapes: bool = name.is_empty() || name.starts_with('/') || name.split('/').any(|part| { return part.is_empty() || part == ".." || part == "."; });
    if escapes {
        return Err(format!("{name:?} is not a reference name this reader will turn into a path"));
    }
    return Ok(());
}

/// What:
/// `fn read_reference(layout: &GitLayout, name: &str) -> Result<Option<String>, String>` reads the object name stored in one
/// reference, loose first and then packed, in the common directory.
/// Why:
/// A loose file shadows `packed-refs`;
/// a symbolic reference or anything that is not an object name is an error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readReference(layout: GitLayout, name: string): string | undefined  // throws
/// ```
fn read_reference(layout: &GitLayout, name: &str) -> Result<Option<String>, String> {
    check_ref_name(name)?;
    let loose: PathBuf = layout.common_dir.join(name);
    match fs::read_to_string(&loose) {
        Ok(text) => {
            let trimmed: &str = trim_space(&text);
            if is_object_name(trimmed) {
                return Ok(Some(trimmed.to_string()));
            }
            return Err(format!("{} holds something other than an object name (a symbolic reference?)", loose.display()));
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound || loose.is_dir() => {}
        Err(error) => return Err(cannot_read(&loose, &error)),
    }
    let Some(packed) = read_optional(&layout.common_dir.join("packed-refs"))? else {
        return Ok(None);
    };
    for line in packed.lines() {
        if line.starts_with('#') || line.starts_with('^') {
            continue;
        }
        if let Some((object, reference)) = line.split_once(' ')
            && reference == name
            && is_object_name(object)
        {
            return Ok(Some(object.to_string()));
        }
    }
    return Ok(None);
}

/// What:
/// `pub fn lookup_ref(layout: &GitLayout, name: &str) -> Result<Option<String>, String>` reads the object name stored in a full
/// reference name under `refs/heads/`, `refs/remotes/` or `refs/tags/`.
/// Why:
/// Other namespaces include the worktree-private ones (`refs/worktree/`, `refs/bisect/`, `refs/rewritten/`) whose files are not in
/// the common directory;
/// they are Git's to read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function lookupRef(layout: GitLayout, name: string): string | undefined  // throws: ask Git
/// ```
pub fn lookup_ref(layout: &GitLayout, name: &str) -> Result<Option<String>, String> {
    if !SHARED_NAMESPACES.iter().any(|namespace| { return name.starts_with(namespace); }) {
        return Err(format!("{name:?} is outside refs/heads, refs/remotes and refs/tags, so only Git reads it"));
    }
    return read_reference(layout, name);
}

/// What:
/// `pub fn lookup_short(layout: &GitLayout, name: &str) -> Result<Option<String>, String>` resolves a short name the way Git's
/// reference lookup does for a remote-tracking branch (`origin/main`),
/// or answers `Err` when anything else could also be named by it.
/// Why:
/// Git tries `name`, `refs/name`, `refs/tags/name`, `refs/heads/name`, `refs/remotes/name` and `refs/remotes/name/HEAD` in that
/// order and warns when several exist;
/// the reader answers only when exactly the remote-tracking reference exists and nothing earlier does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function lookupShort(layout: GitLayout, name: string): string | undefined  // throws: ask Git
/// ```
pub fn lookup_short(layout: &GitLayout, name: &str) -> Result<Option<String>, String> {
    check_ref_name(name)?;
    for directory in [&layout.common_dir, &layout.git_dir] {
        if directory.join(name).exists() {
            return Err(format!("{name:?} also names a file in the repository directory"));
        }
    }
    for prefix in ["refs/", "refs/tags/", "refs/heads/"] {
        if read_reference(layout, &format!("{prefix}{name}"))?.is_some() {
            return Err(format!("{name:?} is ambiguous: {prefix}{name} exists"));
        }
    }
    let found: Option<String> = lookup_ref(layout, &format!("refs/remotes/{name}"))?;
    if found.is_none() && read_reference(layout, &format!("refs/remotes/{name}/HEAD"))?.is_some() {
        return Err(format!("{name:?} resolves through refs/remotes/{name}/HEAD, which only Git reads"));
    }
    return Ok(found);
}

/// What:
/// `fn single_value<'a>(values: &[Option<&'a str>], key: &str) -> Result<Option<&'a str>, String>` takes the last value of a
/// variable and refuses an occurrence without a value or an empty one.
/// Why:
/// Git reports a missing value for `branch.<name>.remote` as an error instead of reading it as a string.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function singleValue(values: (string | undefined)[], key: string): string | undefined  // throws on a bare key
/// ```
fn single_value<'a>(values: &[Option<&'a str>], key: &str) -> Result<Option<&'a str>, String> {
    if values.iter().any(|value| { return value.is_none_or(str::is_empty); }) {
        return Err(format!("{key} is written without a value, which Git rejects"));
    }
    return Ok(values.last().copied().flatten());
}

/// What:
/// `pub fn tracking_ref(config: &ParsedConfig, layout: &GitLayout, branch: &str) -> Result<Option<String>, String>` returns the
/// reference that `@{upstream}` of a branch names:
/// the branch's first `merge` value, mapped through the remote's fetch refspecs (or as is for the remote `.`).
/// Why:
/// Git's `@{upstream}` uses the first `branch.<name>.merge` value and the last `branch.<name>.remote`,
/// and rejects the remote when any of its refspecs is invalid.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function trackingRef(config: ParsedConfig, layout: GitLayout, branch: string): string | undefined  // throws: ask Git
/// ```
pub fn tracking_ref(config: &ParsedConfig, layout: &GitLayout, branch: &str) -> Result<Option<String>, String> {
    let remotes: Vec<Option<&str>> = config.values("branch", Some(branch), "remote");
    let merges: Vec<Option<&str>> = config.values("branch", Some(branch), "merge");
    let remote_name: Option<&str> = single_value(&remotes, &format!("branch.{branch}.remote"))?;
    if merges.iter().any(|value| { return value.is_none(); }) {
        return Err(format!("branch.{branch}.merge is written without a value, which Git rejects"));
    }
    let (Some(remote), Some(Some(merge))) = (remote_name, merges.first().copied()) else {
        return Ok(None);
    };
    if !merge.starts_with("refs/") || !is_plain_ref_name(merge) {
        return Err(format!("branch.{branch}.merge is {merge:?}, not a plain full reference name"));
    }
    if remote == "." {
        return Ok(Some(merge.to_string()));
    }
    if !is_plain_ref_name(remote) {
        return Err(format!("branch.{branch}.remote is {remote:?}, which is not a plain remote name"));
    }
    for legacy in ["remotes", "branches"] {
        if layout.common_dir.join(legacy).join(remote).exists() {
            return Err(format!("the remote {remote:?} is also described by a legacy {legacy}/ file, which only Git reads"));
        }
    }
    let specs: Vec<Option<&str>> = config.values("remote", Some(remote), "fetch");
    let mut texts: Vec<&str> = Vec::new();
    for spec in specs {
        match spec {
            Some(text) => texts.push(text),
            None => return Err(format!("remote.{remote}.fetch is written without a value, which Git rejects")),
        }
    }
    return map_through_refspecs(&texts, merge);
}
