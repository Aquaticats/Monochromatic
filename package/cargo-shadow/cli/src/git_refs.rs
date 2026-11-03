//! What:
//! A reader of a Git repository's `HEAD`,
//! branch configuration and references (loose files and `packed-refs`),
//! for the plain file layouts only,
//! and the choice of the trust base made from them without starting Git.
//! Why:
//! The shadow runs before every cargo command,
//! and the owner's rules are "Invoke real git as rarely as possible."
//! and
//! "embed instead of spawning" (`package/git-policy/cli/SPEC.md`).
//! Every function answers `Err(reason)` for a layout or setting it cannot read exactly:
//! the reftable reference format,
//! a worktree-specific configuration,
//! a configuration `include` or `includeIf`,
//! a symbolic reference where a commit was expected,
//! an ambiguous short name,
//! and the `GIT_*` variables that move the
//! repository or the configuration.
//! The caller then asks Git once,
//! as it did before this reader existed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // git-refs.ts
//! export function nativeBase(root: string, inCi: boolean, env: NativeEnvironment): NativeBase  // throws: ask Git
//! ```

/// What:
/// `use std::fs;` imports
/// `fs`
/// so this file can name them unqualified.
/// Why:
/// `fs` reads the repository files (`HEAD`,
/// `config`,
/// reference files and `packed-refs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { readFileSync, statSync } from 'node:fs';
/// ```
use std::fs;
/// What:
/// `use std::path::{Path, PathBuf};` imports
/// `Path`,
/// `PathBuf`
/// so this file can name them unqualified.
/// Why:
/// `Path` is the borrowed path and `PathBuf` the owned one (the same pair as `&str` and `String`);
/// stored directories are owned,
/// lent ones borrowed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // TS has one type for this: string
/// ```
use std::path::{Path, PathBuf};

/// What:
/// `use crate::git_config::{ParsedConfig, parse_config};` imports
/// `ParsedConfig`,
/// `parse_config`
/// so this file can name them unqualified.
/// Why:
/// The branch and remote settings are read from the configuration files with the reader in `git_config.rs`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ParsedConfig, parseConfig } from 'crate/git_config';
/// ```
use crate::git_config::{ParsedConfig, parse_config};

/// The environment variables that move the repository,
/// its references or its configuration;
/// when one is set the answer is Git's,
/// not this reader's.
const BLOCKING_VARIABLES: [&str; 14] = [
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
    "GIT_REPLACE_REF_BASE",
];

/// Where Git looks for configuration besides the repository's own file,
/// and whether the process environment forbids this reader.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeEnvironment {
    /// The home directory (`~/.gitconfig`,
    /// `~/.config/git/config`).
    pub home: Option<PathBuf>,
    /// `XDG_CONFIG_HOME`,
    /// when set and not empty.
    pub xdg_config_home: Option<PathBuf>,
    /// The system configuration file (`/etc/gitconfig`).
    pub system_config: PathBuf,
    /// The name of a `GIT_*` variable that moves the repository or the configuration,
    /// when one is set.
    pub blocking_variable: Option<String>,
}

/// The constructor that reads the running process's environment.
impl NativeEnvironment {
    /// What:
    /// `pub fn from_process() -> NativeEnvironment` reads the running process's environment.
    /// Why:
    /// The tests build the struct by hand with a disposable home so they never read the real user's configuration.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static fromProcess(): NativeEnvironment
    /// ```
    pub fn from_process() -> NativeEnvironment {
        // What:     `std::env::var_os(name)` is the environment variable as an `Option<OsString>` (an OS-native string that may not be valid UTF-8); `.map(PathBuf::from)` turns the present value into a path (sibling: `std::env::var`, which fails on invalid UTF-8).
        // Why:      Paths from the environment must not be forced through UTF-8.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const home = process.env.HOME ? process.env.HOME : undefined;
        // ```
        let home: Option<PathBuf> = std::env::var_os("HOME").map(PathBuf::from);
        // `match`: picks the arm whose pattern fits the value.
        // Typed binding: `xdg_config_home: Option<PathBuf>`.
        let xdg_config_home: Option<PathBuf> = match std::env::var_os("XDG_CONFIG_HOME") {
            // `Some`: the present or the absent value of an `Option`.
            Some(value) if !value.is_empty() => Some(PathBuf::from(value)),
            // `None`: the present or the absent value of an `Option`.
            _ => None,
        };
        // `None`: the present or the absent value of an `Option`.
        // What:     `let mut x` declares a variable that can be reassigned or changed; plain `let x` is a constant binding (TS `const`).
        // Why:      The value is built up or replaced step by step below.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // let x = ...;   // plain Rust `let` is TS `const`
        // ```
        // Typed binding: `blocking_variable: Option<String>`.
        let mut blocking_variable: Option<String> = None;
        for name in BLOCKING_VARIABLES {
            if std::env::var_os(name).is_some() {
                // `Some`: the present or the absent value of an `Option`.
                // What:     `.to_string()` (also `.to_owned()`, `String::from`) copies borrowed or displayable text into a new owned `String` (sibling: `&str`, a borrowed view that owns nothing).
                // Why:      A struct field or a returned value must own its text because it may outlive the place the text came from.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // String(value)  // JS strings are always owned values
                // ```
                blocking_variable = Some(name.to_string());
                break;
            }
        }
        return NativeEnvironment { home, xdg_config_home, system_config: PathBuf::from("/etc/gitconfig"), blocking_variable };
    }
}

/// Where a repository keeps its files.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GitLayout {
    /// The directory holding this working tree's `HEAD` (a linked worktree's own directory under the common one).
    pub git_dir: PathBuf,
    /// The directory holding `config`,
    /// `packed-refs` and `refs/` (the same as `git_dir` unless this is a linked worktree).
    pub common_dir: PathBuf,
}

/// What `HEAD` names.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Head {
    /// A branch:
    /// the full reference name,
    /// such as `refs/heads/main`.
    Branch(
        /// The full reference name,
        /// as owned text (`String`,
        /// not the borrowed `&str`,
        /// because the value outlives the file text it was cut from).
        String,
    ),
    /// A detached commit:
    /// its object name in hexadecimal.
    Detached(
        /// The object name in hexadecimal,
        /// as owned text.
        String,
    ),
}

/// The base chosen without Git:
/// how it is spelled and what it names.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeBase {
    /// How the base is spelled.
    pub spelling: String,
    /// The commit it names,
    /// or why it names none.
    pub commit: Result<String, String>,
}

/// What:
/// `fn is_object_name(text: &str) -> bool` says whether a text is a full hexadecimal object name
/// (40 digits for SHA-1,
/// 64 for SHA-256).
/// Why:
/// A reference file holds exactly an object name and a newline,
/// or `ref: <name>` for a symbolic reference;
/// anything else is not read as a commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isObjectName = (text: string) => /^([0-9a-f]{40}|[0-9a-f]{64})$/.test(text);
/// ```
fn is_object_name(text: &str) -> bool {
    if text.len() != 40 && text.len() != 64 {
        return false;
    }
    for character in text.chars() {
        if !character.is_ascii_hexdigit() {
            return false;
        }
    }
    return true;
}

/// What:
/// `fn cannot_read(path: &Path, error: &std::io::Error) -> String` words the failure to read a repository file.
/// Why:
/// Every reader here reports a file it cannot read the same way,
/// so the caller can show why Git was asked instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const cannotRead = (path: string, error: Error): string => `${path} cannot be read: ${error.message}`;
/// ```
fn cannot_read(path: &Path, error: &std::io::Error) -> String {
    // What:     A name ending in `!` is a macro: code that expands before compiling. `format!("..{}..", x)` builds a `String` from a template (a TS template literal); `eprintln!`/`println!` print a line to standard error/output; `vec![..]` builds a `Vec`; `matches!(v, Pattern)` tests a pattern.
    // Why:      Text building and printing are written as one call instead of string concatenation.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // `..${x}..`   // template literal
    // console.error(text);   // eprintln!
    // [a, b]            // vec![a, b]
    // ```
    return format!("{} cannot be read: {error}", path.display());
}

/// What:
/// `fn read_text(path: &Path) -> Result<String, String>` reads a file as text,
/// with the error already worded.
/// Why:
/// `HEAD` and the `.git` pointer file are read the same way and fail the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readText(path: string): string  // throws a worded error
/// ```
fn read_text(path: &Path) -> Result<String, String> {
    // `match`: picks the arm whose pattern fits the value.
    return match fs::read_to_string(path) {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        Ok(text) => Ok(text),
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // Borrow `&error`: lends the value without copying or giving it away.
        Err(error) => Err(cannot_read(path, &error)),
    };
}

/// What:
/// `fn setting_is(config: &ParsedConfig, key: &str, expected: &str) -> bool` says whether `extensions.<key>` is set to a value
/// equal to `expected` without regard to case.
/// Why:
/// Git compares these extension values without case.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const settingIs = (config: ParsedConfig, key: string, expected: string): boolean => config.last('extensions', undefined, key)?.toLowerCase() === expected;
/// ```
fn setting_is(config: &ParsedConfig, key: &str, expected: &str) -> bool {
    // `None`: the present or the absent value of an `Option`.
    // `match`: picks the arm whose pattern fits the value.
    return match config.last("extensions", None, key) {
        // `Some`: the present or the absent value of an `Option`.
        Some(value) => value.eq_ignore_ascii_case(expected),
        // `None`: the present or the absent value of an `Option`.
        None => false,
    };
}

/// What:
/// `pub fn find_layout(root, environment) -> Result<GitLayout, String>` finds the repository whose working tree is `root`:
/// `root/.git` as a directory,
/// or as a file `gitdir: <path>` (a linked worktree or a submodule),
/// following `commondir`.
/// Why:
/// Git would search the parent directories as well;
/// the shadow's root is where `rust-catalog.toml` is,
/// normally the top of the
/// working tree,
/// and anything else is Git's to answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function findLayout(root: string, environment: NativeEnvironment): GitLayout  // throws: ask Git
/// ```
pub fn find_layout(root: &Path, environment: &NativeEnvironment) -> Result<GitLayout, String> {
    // `Some`: the present or the absent value of an `Option`.
    // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
    // Borrow `&environment.blocking_variable`: lends the value without copying or giving it away.
    if let Some(name) = &environment.blocking_variable {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        return Err(format!("{name} is set, so only Git can say where the repository is"));
    }
    // Typed binding: `dot_git: PathBuf`.
    let dot_git: PathBuf = root.join(".git");
    // What:     `fs::metadata(path)` reads a file's type and size; `Err` when it does not exist. `.is_dir()` / `.is_file()` test the kind.
    // Why:      A `.git` directory is a plain repository; a `.git` file points to the real directory.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const stat = statSync(dotGit);
    // ```
    let metadata = match fs::metadata(&dot_git) {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        Ok(found) => found,
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // Borrow `&dot_git`: lends the value without copying or giving it away.
        Err(error) => return Err(cannot_read(&dot_git, &error)),
    };
    // Typed binding: `git_dir: PathBuf`.
    let git_dir: PathBuf = if metadata.is_dir() {
        dot_git
    } else {
        // What:     `expr?` is the propagation operator: if `expr` is `Err` (or `None`) the whole function returns that failure at once, otherwise the success value is unwrapped and used.
        // Why:      A failure stops the step at its source and reaches the caller as a returned value, with no `try`/`catch` in between.
        // Gotcha:   `?` only works in a function whose own return type is a `Result` (or `Option`) of a compatible kind.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const value = await expr;   // throws to the caller on failure
        // ```
        // Borrow `&dot_git`: lends the value without copying or giving it away.
        // Typed binding: `text: String`.
        let text: String = read_text(&dot_git)?;
        // `match`: picks the arm whose pattern fits the value.
        // Typed binding: `target: &str`.
        let target: &str = match text.trim().strip_prefix("gitdir:") {
            // `Some`: the present or the absent value of an `Option`.
            Some(found) => found.trim(),
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            // `None`: the present or the absent value of an `Option`.
            // Macro `format!`: builds or prints text (or a list) from a template.
            None => return Err(format!("{} is a file but does not say gitdir:", dot_git.display())),
        };
        // A relative target is relative to the directory that holds the `.git` file.
        root.join(target)
    };
    // A linked worktree's directory has a `commondir` file naming the directory that holds the shared files.
    let common_dir: PathBuf = match fs::read_to_string(git_dir.join("commondir")) {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        Ok(text) => git_dir.join(text.trim()),
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // What:     `.clone()` makes an independent copy of an owned value (a new `String`, `Vec`, ...), leaving the original usable.
        // Why:      The copy can be stored or moved elsewhere while the original stays in place.
        // Gotcha:   Cloning a large value copies all of it; it is not free like passing a reference.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // structuredClone(value)  // or [...list] / {...object}
        // ```
        Err(_) => git_dir.clone(),
    };
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    return Ok(GitLayout { git_dir, common_dir });
}

/// What:
/// `pub fn read_config(layout, environment) -> Result<ParsedConfig, String>` reads the system,
/// user and repository
/// configuration files in Git's order (later overrides earlier) and refuses what it cannot read exactly.
/// Why:
/// `branch.<name>.merge` and `remote.<name>.fetch` can be set at any of those levels.
/// An `include`,
/// an `includeIf`,
/// a reftable reference store (`extensions.refStorage`) or a worktree-specific configuration
/// (`extensions.worktreeConfig`) change what a key means,
/// so each makes the answer Git's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function readConfig(layout: GitLayout, environment: NativeEnvironment): ParsedConfig  // throws: ask Git
/// ```
pub fn read_config(layout: &GitLayout, environment: &NativeEnvironment) -> Result<ParsedConfig, String> {
    // `.clone()`: an independent owned copy.
    // `let mut`: a binding that may change.
    // Typed binding: `files: Vec<PathBuf>`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    // What:     `Vec<T>` is an owned growable list; the siblings are `&[T]`, a borrowed view of a list, and `[T; N]`, a fixed-size array.
    // Why:      The list is built or returned here, so `Vec<T>` and not a borrowed slice.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: T[] / Set / Map / number
    // ```
    let mut files: Vec<PathBuf> = vec![environment.system_config.clone()];
    // The user's file: `$XDG_CONFIG_HOME/git/config` or `~/.config/git/config`, then `~/.gitconfig`.
    if let Some(xdg) = &environment.xdg_config_home {
        files.push(xdg.join("git/config"));
    // `Some`: the present or the absent value of an `Option`.
    // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
    // Borrow `&environment.home`: lends the value without copying or giving it away.
    } else if let Some(home) = &environment.home {
        files.push(home.join(".config/git/config"));
    }
    // `Some`: the present or the absent value of an `Option`.
    // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
    // Borrow `&environment.home`: lends the value without copying or giving it away.
    if let Some(home) = &environment.home {
        files.push(home.join(".gitconfig"));
    }
    files.push(layout.common_dir.join("config"));
    // `let mut`: a binding that may change.
    // Typed binding: `merged: ParsedConfig`.
    let mut merged: ParsedConfig = ParsedConfig::default();
    // Borrow `&files`: lends the value without copying or giving it away.
    for file in &files {
        // A file that does not exist is skipped; one that exists and cannot be read or parsed makes the answer Git's.
        let text: String = match fs::read_to_string(file) {
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            Ok(found) => found,
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            // Macro `format!`: builds or prints text (or a list) from a template.
            Err(error) => return Err(format!("{} cannot be read: {error}", file.display())),
        };
        // `match`: picks the arm whose pattern fits the value.
        // Borrow `&text`: lends the value without copying or giving it away.
        // Typed binding: `parsed: ParsedConfig`.
        let parsed: ParsedConfig = match parse_config(&text) {
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            Ok(found) => found,
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            // Macro `format!`: builds or prints text (or a list) from a template.
            Err(message) => return Err(format!("{}: {message}", file.display())),
        };
        merged.extend_with(parsed);
    }
    if merged.has_include {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        return Err("a configuration file has include or includeIf, which only Git follows".to_string());
    }
    // Borrow `&merged`: lends the value without copying or giving it away.
    if setting_is(&merged, "refstorage", "reftable") {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        return Err("the repository stores references as reftable, which only Git reads".to_string());
    }
    // Borrow `&merged`: lends the value without copying or giving it away.
    if setting_is(&merged, "worktreeconfig", "true") {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        return Err("the repository has a worktree-specific configuration (extensions.worktreeConfig)".to_string());
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    return Ok(merged);
}

/// What:
/// `pub fn read_head(layout) -> Result<Head, String>` reads `HEAD`:
/// `ref: refs/heads/<branch>` or a detached object name.
/// Why:
/// The current branch decides which `branch.<name>.*` keys apply,
/// and a detached `HEAD` has no upstream.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function readHead(layout: GitLayout): Head  // throws: ask Git
/// ```
pub fn read_head(layout: &GitLayout) -> Result<Head, String> {
    // Typed binding: `path: PathBuf`.
    let path: PathBuf = layout.git_dir.join("HEAD");
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Borrow `&path`: lends the value without copying or giving it away.
    // Typed binding: `text: String`.
    let text: String = read_text(&path)?;
    // Typed binding: `trimmed: &str`.
    let trimmed: &str = text.trim();
    // `Some`: the present or the absent value of an `Option`.
    // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
    if let Some(name) = trimmed.strip_prefix("ref:") {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        return Ok(Head::Branch(name.trim().to_string()));
    }
    if is_object_name(trimmed) {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        return Ok(Head::Detached(trimmed.to_string()));
    }
    // `Err(...)`: builds the success or the failure variant of a `Result`.
    // Macro `format!`: builds or prints text (or a list) from a template.
    return Err(format!("{} holds neither a reference nor an object name", path.display()));
}

/// What:
/// `fn check_ref_name(name: &str) -> Result<(), String>` refuses a reference name that could leave the repository directory.
/// Why:
/// Names come from configuration files;
/// a `..` component or an absolute name must never become a file path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkRefName(name: string): void  // throws on a name that could escape
/// ```
fn check_ref_name(name: &str) -> Result<(), String> {
    // `let mut`: a binding that may change.
    // Typed binding: `escapes: bool`.
    let mut escapes: bool = name.is_empty() || name.starts_with('/');
    for part in name.split('/') {
        if part.is_empty() || part == ".." || part == "." {
            escapes = true;
        }
    }
    if escapes {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        return Err(format!("{name:?} is not a reference name this reader will turn into a path"));
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    return Ok(());
}

/// What:
/// `pub fn lookup_ref(layout, name) -> Result<Option<String>, String>` is the object name a full reference holds:
/// the loose file `<common>/<name>` first,
/// then a line of `<common>/packed-refs`.
/// `Ok(None)` means the reference does not exist;
/// `Err` means it exists in a form this reader does not follow.
/// Why:
/// This is how Git's files backend resolves a reference,
/// and a symbolic reference (`ref: ...`) is left to Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function lookupRef(layout: GitLayout, name: string): string | undefined  // throws: ask Git
/// ```
pub fn lookup_ref(layout: &GitLayout, name: &str) -> Result<Option<String>, String> {
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    check_ref_name(name)?;
    // Typed binding: `loose: PathBuf`.
    let loose: PathBuf = layout.common_dir.join(name);
    // `match`: picks the arm whose pattern fits the value.
    // Borrow `&loose`: lends the value without copying or giving it away.
    match fs::read_to_string(&loose) {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        Ok(text) => {
            // Typed binding: `trimmed: &str`.
            let trimmed: &str = text.trim();
            if is_object_name(trimmed) {
                // `Ok(...)`: builds the success or the failure variant of a `Result`.
                // `Some`: the present or the absent value of an `Option`.
                // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
                return Ok(Some(trimmed.to_string()));
            }
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            // Macro `format!`: builds or prints text (or a list) from a template.
            return Err(format!("{} holds something other than an object name (a symbolic reference?)", loose.display()));
        }
        // A missing file, or a directory where a file was looked for, means there is no loose reference of that name.
        Err(error) if error.kind() == std::io::ErrorKind::NotFound || loose.is_dir() => {}
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        Err(error) => return Err(format!("{} cannot be read: {error}", loose.display())),
    }
    // Typed binding: `packed_path: PathBuf`.
    let packed_path: PathBuf = layout.common_dir.join("packed-refs");
    // `match`: picks the arm whose pattern fits the value.
    // Borrow `&packed_path`: lends the value without copying or giving it away.
    // Typed binding: `packed: String`.
    let packed: String = match fs::read_to_string(&packed_path) {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        Ok(found) => found,
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // `None`: the present or the absent value of an `Option`.
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        Err(error) => return Err(format!("{} cannot be read: {error}", packed_path.display())),
    };
    for line in packed.lines() {
        // Header lines start with `#`; a line starting with `^` is the peeled object of the tag above it.
        if line.starts_with('#') || line.starts_with('^') {
            continue;
        }
        // `Some`: the present or the absent value of an `Option`.
        // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
        if let Some((object, reference)) = line.split_once(' ')
            && reference == name
            && is_object_name(object)
        {
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            // `Some`: the present or the absent value of an `Option`.
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            return Ok(Some(object.to_string()));
        }
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // `None`: the present or the absent value of an `Option`.
    return Ok(None);
}

/// What:
/// `pub fn lookup_short(layout, name) -> Result<Option<String>, String>` resolves a short name such as `origin/main` the way
/// `git rev-parse` does for a name that is not a full reference:
/// the first of `refs/<name>`,
/// `refs/tags/<name>`,
/// `refs/heads/<name>`,
/// `refs/remotes/<name>` that exists.
/// Why:
/// A tag or branch that shadows the remote-tracking branch decides the answer in Git;
/// this reader answers only when the name
/// is unshadowed,
/// and a shadowing reference makes the answer Git's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function lookupShort(layout: GitLayout, name: string): string | undefined  // throws: ask Git
/// ```
pub fn lookup_short(layout: &GitLayout, name: &str) -> Result<Option<String>, String> {
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    check_ref_name(name)?;
    // A file named like the short name directly in the repository directory would be a root reference.
    if layout.common_dir.join(name).exists() {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        return Err(format!("{name:?} also names a file in the repository directory"));
    }
    for prefix in ["refs/", "refs/tags/", "refs/heads/"] {
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Borrow `&format`: lends the value without copying or giving it away.
        // Macro `format!`: builds or prints text (or a list) from a template.
        if lookup_ref(layout, &format!("{prefix}{name}"))?.is_some() {
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            // Macro `format!`: builds or prints text (or a list) from a template.
            return Err(format!("{name:?} is ambiguous: {prefix}{name} exists"));
        }
    }
    // Borrow `&format`: lends the value without copying or giving it away.
    // Macro `format!`: builds or prints text (or a list) from a template.
    return lookup_ref(layout, &format!("refs/remotes/{name}"));
}

/// What:
/// `fn map_refspec(spec: &str, merge: &str) -> Result<Option<String>, String>` applies one `remote.<name>.fetch` refspec to a
/// branch's `merge` reference and returns the local reference it is stored in,
/// if the spec covers it.
/// Why:
/// `+refs/heads/*:refs/remotes/origin/*` maps `refs/heads/main` to `refs/remotes/origin/main`;
/// this is how Git finds the
/// remote-tracking branch of `@{upstream}`.
/// A negative refspec (`^...`) is Git's to apply.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function mapRefspec(spec: string, merge: string): string | undefined  // throws on a negative refspec
/// ```
fn map_refspec(spec: &str, merge: &str) -> Result<Option<String>, String> {
    // What:     `.unwrap_or(x)`, `.unwrap_or_default()` and `.unwrap_or_else(f)` take the value out of an `Option`/`Result` or use a fallback (a given value, the type's empty value, or the result of a function). `.unwrap()`/`.expect()` instead stop the program on absence.
    // Why:      A missing value or a failure becomes a deliberate default here instead of an error.
    // Gotcha:   `unwrap_or*` on a `Result` silently drops its error value.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // value ?? fallback
    // ```
    // Typed binding: `without_force: &str`.
    let without_force: &str = spec.trim().strip_prefix('+').unwrap_or(spec.trim());
    if without_force.starts_with('^') {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        return Err("a negative fetch refspec is applied by Git".to_string());
    }
    // `match`: picks the arm whose pattern fits the value.
    let (source, destination) = match without_force.split_once(':') {
        // `Some`: the present or the absent value of an `Option`.
        Some(pair) => pair,
        // A refspec without a destination stores nothing.
        None => return Ok(None),
    };
    // `match`: picks the arm whose pattern fits the value.
    match (source.split_once('*'), destination.split_once('*')) {
        // `Some`: the present or the absent value of an `Option`.
        (Some((prefix, suffix)), Some((target_prefix, target_suffix))) => {
            // `Some`: the present or the absent value of an `Option`.
            // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
            if let Some(rest) = merge.strip_prefix(prefix)
                // `Some`: the present or the absent value of an `Option`.
                && let Some(middle) = rest.strip_suffix(suffix)
            {
                // `Ok(...)`: builds the success or the failure variant of a `Result`.
                // `Some`: the present or the absent value of an `Option`.
                // Macro `format!`: builds or prints text (or a list) from a template.
                return Ok(Some(format!("{target_prefix}{middle}{target_suffix}")));
            }
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            // `None`: the present or the absent value of an `Option`.
            return Ok(None);
        }
        // `None`: the present or the absent value of an `Option`.
        (None, None) => {
            if source == merge && !destination.is_empty() {
                // `Ok(...)`: builds the success or the failure variant of a `Result`.
                // `Some`: the present or the absent value of an `Option`.
                // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
                return Ok(Some(destination.to_string()));
            }
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            // `None`: the present or the absent value of an `Option`.
            return Ok(None);
        }
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        _ => return Err("a fetch refspec has a pattern on one side only".to_string()),
    }
}

/// What:
/// `pub fn tracking_ref(config, branch) -> Result<Option<String>, String>` is the local reference that `@{upstream}` of a
/// branch names,
/// from `branch.<name>.remote`,
/// `branch.<name>.merge` and `remote.<remote>.fetch`.
/// `Ok(None)` is "no upstream".
/// Why:
/// With the remote `.` the upstream is another local branch;
/// otherwise the first fetch refspec that covers the merge
/// reference gives the remote-tracking name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function trackingRef(config: ParsedConfig, branch: string): string | undefined  // throws: ask Git
/// ```
pub fn tracking_ref(config: &ParsedConfig, branch: &str) -> Result<Option<String>, String> {
    // `Some`: the present or the absent value of an `Option`.
    // `match`: picks the arm whose pattern fits the value.
    // Typed binding: `remote: &str`.
    let remote: &str = match config.last("branch", Some(branch), "remote") {
        // `Some`: the present or the absent value of an `Option`.
        Some(found) => found,
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // `None`: the present or the absent value of an `Option`.
        None => return Ok(None),
    };
    // `Some`: the present or the absent value of an `Option`.
    // `match`: picks the arm whose pattern fits the value.
    // Typed binding: `merge: &str`.
    let merge: &str = match config.last("branch", Some(branch), "merge") {
        // `Some`: the present or the absent value of an `Option`.
        Some(found) => found,
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // `None`: the present or the absent value of an `Option`.
        None => return Ok(None),
    };
    if !merge.starts_with("refs/") {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        return Err(format!("branch.{branch}.merge is {merge:?}, not a full reference name"));
    }
    if remote == "." {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // `Some`: the present or the absent value of an `Option`.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        return Ok(Some(merge.to_string()));
    }
    // `Some`: the present or the absent value of an `Option`.
    // Typed binding: `specs: Vec<&str>`.
    let specs: Vec<&str> = config.values("remote", Some(remote), "fetch");
    // A negative refspec can remove a match made by another one, so any of them makes the answer Git's.
    for spec in &specs {
        if spec.trim().starts_with('^') {
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            return Err("a negative fetch refspec is applied by Git".to_string());
        }
    }
    for spec in specs {
        // `Some`: the present or the absent value of an `Option`.
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
        if let Some(mapped) = map_refspec(spec, merge)? {
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            // `Some`: the present or the absent value of an `Option`.
            return Ok(Some(mapped));
        }
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // `None`: the present or the absent value of an `Option`.
    return Ok(None);
}
