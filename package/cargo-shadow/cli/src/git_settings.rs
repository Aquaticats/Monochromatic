//! What:
//! Reads the Git configuration files in Git's order and refuses the settings only Git can apply.
//! Why:
//! `git_refs.rs` holds the layout,
//! `HEAD` and references;
//! the configuration reading was split out to keep each file under the line limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // git-settings.ts
//! export function readConfig(layout: GitLayout, env: NativeEnvironment): ParsedConfig  // throws: ask Git
//! ```

/// Error text built only on failure,
/// from a `format_args!` value.
use crate::lazy_error_text::ExplainedError;
/// Imports the path types.
use std::cell::Cell;
// What:     `use path::Name;` imports names from another module or crate so they can be used unqualified in this file. `::` separates the path segments; `{A, B}` imports several names at once.
// Why:      The code below names these items directly instead of spelling their full paths each time.
//
// In TS you'd write (pseudocode):
// ```ts
// import { Name } from './module.ts';
// ```
// What:     `PathBuf` is an owned file path and `&Path` the borrowed view of one (the same pair as `String` and `&str`).
// Why:      Joined or stored paths are owned; paths passed in are borrowed.
//
// In TS you'd write (pseudocode):
// ```ts
// // TS has one type for this: string
// ```
/// Imports `std::path::{Path, PathBuf}` for this file.
use std::path::{Path, PathBuf};
// Imports `crate::git_base::system_config_of_git`.
/// Imports `crate::git_base::system_config_of_git` for this file.
use crate::git_base::system_config_of_git;

/// Imports the configuration types and the lexer entry point.
use crate::git_config::{ConfigEntry, ParsedConfig, parse_bool, parse_config};
/// Imports the layout and environment types and the file readers.
use crate::git_refs::{GitLayout, NativeEnvironment, read_optional};
/// Imports the refspec check.
use crate::git_refspec::check_refspec;

/// What:
/// `fn is_extension(entry: &&ConfigEntry) -> bool` says whether a variable is a repository extension (`extensions.*`).
/// Why:
/// It names the test that finds a setting only Git applies.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isExtension = (entry: ConfigEntry): boolean => entry.name.startsWith('extensions.');
/// ```
pub(crate) fn is_extension(entry: &&ConfigEntry) -> bool {
    return entry.name.starts_with("extensions.");
}

/// The `remote.<name>.<key>` keys whose value is a plain string.
const REMOTE_STRING_KEYS: [&str; 9] = ["url", "pushurl", "tagopt", "proxy", "proxyauthmethod", "vcs", "receivepack", "uploadpack", "partialclonefilter"];

/// The `remote.<name>.<key>` keys whose value is a Git boolean.
const REMOTE_BOOLEAN_KEYS: [&str; 6] = ["mirror", "skipdefaultupdate", "skipfetchall", "prune", "prunetags", "promisor"];

/// The `branch.<name>.<key>` keys whose value is a plain string.
const BRANCH_STRING_KEYS: [&str; 5] = ["remote", "merge", "pushremote", "description", "mergeoptions"];

/// The words `branch.<name>.rebase` takes besides a boolean.
const REBASE_WORDS: [&str; 3] = ["merges", "interactive", "i"];

/// What:
/// `fn rejected_entry(name: &str, why: &str) -> String` words the refusal of one configuration entry:
/// the entry's name,
/// the reason,
/// and that Git rejects it or this reader cannot judge it.
/// Why:
/// A function replaces the closure that captured the entry.
fn rejected_entry(name: &str, why: &str) -> String {
    return format!("{name} {why}, which Git rejects or this reader cannot judge");
}

/// What:
/// `pub(crate) fn check_remote_and_branch_settings(config: &ParsedConfig) -> Result<(), String>` checks every `remote.*` and
/// `branch.*` variable in the configuration,
/// against the classes Git loads them as.
/// Why:
/// Review finding (4a 3):
/// Git loads all remote and branch configuration when it resolves an upstream,
/// so a refspec that is
/// invalid,
/// a boolean that is not one or a required string written without a value anywhere makes Git fail where a narrower check
/// would not.
/// A key this reader has no class for is Git's to judge.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkRemoteAndBranchSettings(config: ParsedConfig): void  // throws: ask Git
/// ```
pub(crate) fn check_remote_and_branch_settings(config: &ParsedConfig) -> Result<(), String> {
    for entry in &config.entries {
        // `Some`: the present or the absent value of an `Option`.
        // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
        // Borrow `&str`: lends the value without copying or giving it away.
        // What:     `String` is an owned, growable UTF-8 text; the sibling `&str` is a borrowed read-only view that owns nothing.
        // Why:      A returned value or a stored field must own its text, so `String` and not `&str`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // TS has one type for this: string
        // ```
        let (family, rest): (&str, &str) = if let Some(rest) = entry.name.strip_prefix("remote.") {
            ("remote", rest)
        // `Some`: the present or the absent value of an `Option`.
        // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
        } else if let Some(rest) = entry.name.strip_prefix("branch.") {
            ("branch", rest)
        } else {
            continue;
        };
        let key: &str = rest.rsplit('.').next().unwrap_or("");
        // Typed binding: `has_subsection: bool`.
        let has_subsection: bool = rest.contains('.');
        // Macro `format!`: builds or prints text (or a list) from a template.
        // Typed binding: `value: Option<&str>`.
        let value: Option<&str> = entry.value.as_deref();
        // `match`: picks the arm whose pattern fits the value.
        match (family, has_subsection) {
            // `match`: picks the arm whose pattern fits the value.
            ("remote", true) if key == "fetch" || key == "push" => match value {
                Some(spec) => check_refspec(spec)?,
                // `Err(...)`: builds the success or the failure variant of a `Result`.
                // `None`: the present or the absent value of an `Option`.
                None => return Err(rejected_entry(&entry.name, "is written without a value")),
            },
            // Borrow `&key`: lends the value without copying or giving it away.
            ("remote", true) if REMOTE_BOOLEAN_KEYS.contains(&key) => {
                // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
                // Closure `|_|`: an anonymous function, like a TS arrow function.
                // `.map_err(...)`: converts the callee's error into this function's error type.
                if parse_bool(value).is_err() {
                    return Err(rejected_entry(&entry.name, "is not a boolean"));
                }
            }
            // Borrow `&key`: lends the value without copying or giving it away.
            ("remote", true) if REMOTE_STRING_KEYS.contains(&key) => {
                if value.is_none() {
                    // `Err(...)`: builds the success or the failure variant of a `Result`.
                    return Err(rejected_entry(&entry.name, "is written without a value"));
                }
            }
            ("remote", false) if key == "pushdefault" => {
                if value.is_none() {
                    // `Err(...)`: builds the success or the failure variant of a `Result`.
                    return Err(rejected_entry(&entry.name, "is written without a value"));
                }
            }
            // Borrow `&key`: lends the value without copying or giving it away.
            ("branch", true) if BRANCH_STRING_KEYS.contains(&key) => {
                if value.is_none() {
                    // `Err(...)`: builds the success or the failure variant of a `Result`.
                    return Err(rejected_entry(&entry.name, "is written without a value"));
                }
            }
            ("branch", true) if key == "rebase" => {
                // `Some`: the present or the absent value of an `Option`.
                // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
                if let Some(text) = value
                    // `Some`: the present or the absent value of an `Option`.
                    && parse_bool(Some(text)).is_err()
                    // Borrow `&text`: lends the value without copying or giving it away.
                    && !REBASE_WORDS.contains(&text)
                {
                    // `Err(...)`: builds the success or the failure variant of a `Result`.
                    return Err(rejected_entry(&entry.name, "is neither a boolean nor a rebase mode"));
                }
            }
            // Borrow `&key`: lends the value without copying or giving it away.
            ("branch", false) if ["autosetupmerge", "autosetuprebase", "sort"].contains(&key) => {}
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            _ => return Err(rejected_entry(&entry.name, "is a setting this reader has no class for")),
        }
    }
    return Ok(());
}

/// What:
/// `fn check_repository_settings(config: &ParsedConfig) -> Result<(), String>` refuses the settings that change what the
/// reader would have to read.
/// Why:
/// Any `extensions.*` variable (a different reference format,
/// object format,
/// worktree configuration or one this reader has never
/// heard of) and any repository format other than 0 or 1 are Git's to judge.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkRepositorySettings(config: ParsedConfig): void  // throws: ask Git
/// ```
pub(crate) fn check_repository_settings(config: &ParsedConfig) -> Result<(), String> {
    // `Some`: the present or the absent value of an `Option`.
    // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
    // Closure `|entry|`: an anonymous function, like a TS arrow function.
    // Iterator step (`.iter()`/`.into_iter()`/`.cloned()`): walks the collection by reference or copies the items.
    if let Some(entry) = config.entries.iter().find(is_extension) {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        return Err(format!("the configuration sets {}, a repository extension that only Git applies", entry.name));
    }
    // `Some`: the present or the absent value of an `Option`.
    // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
    for format in config.values("core", None, "repositoryformatversion") {
        // `match`: picks the arm whose pattern fits the value.
        match format {
            // `Some`: the present or the absent value of an `Option`.
            Some("0" | "1") => {}
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            // Macro `format!`: builds or prints text (or a list) from a template.
            other => return Err(format!("core.repositoryformatversion is {other:?}, which only Git judges")),
        }
    }
    return Ok(());
}

/// What:
/// `pub fn read_config(layout: &GitLayout, environment: &NativeEnvironment) -> Result<ParsedConfig, String>` reads the
/// configuration files in Git's order (system,
/// XDG or home,
/// home,
/// repository) and refuses what only Git can apply.
/// Why:
/// Later files override earlier ones;
/// an `include`,
/// an unreadable file or a refused setting makes the caller ask Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function readConfig(layout: GitLayout, env: NativeEnvironment): ParsedConfig  // throws: ask Git
/// ```
pub fn read_config(layout: &GitLayout, environment: &NativeEnvironment, root: &Path, starts: &Cell<u32>) -> Result<ParsedConfig, String> {
    // `match`: picks the arm whose pattern fits the value.
    // Borrow `&environment.system_config`: lends the value without copying or giving it away.
    // Typed binding: `system: PathBuf`.
    let system: PathBuf = match &environment.system_config {
        // `Some`: the present or the absent value of an `Option`.
        // What:     `.clone()` makes an independent copy of an owned value (a new `String`, `Vec`, ...), leaving the original usable.
        // Why:      The copy can be stored or moved elsewhere while the original stays in place.
        // Gotcha:   Cloning a large value copies all of it; it is not free like passing a reference.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // structuredClone(value)  // or [...list] / {...object}
        // ```
        Some(found) => found.clone(),
        // `None`: the present or the absent value of an `Option`.
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        None => system_config_of_git(environment, root, starts)?,
    };
    // What:     `let mut x` declares a variable that can be reassigned or changed; plain `let x` is a constant binding (TS `const`).
    // Why:      The value is built up or replaced step by step below.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let x = ...;   // plain Rust `let` is TS `const`
    // ```
    // Typed binding: `files: Vec<PathBuf>`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    // What:     `Vec<T>` is an owned growable list; the siblings are `&[T]`, a borrowed view of a list, and `[T; N]`, a fixed-size array.
    // Why:      The list is built or returned here, so `Vec<T>` and not a borrowed slice.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: T[] / Set / Map / number
    // ```
    let mut files: Vec<PathBuf> = vec![system];
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
        // `Some`: the present or the absent value of an `Option`.
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
        let Some(text) = read_optional(file)? else {
            continue;
        };
        // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
        // Closure `|message|`: an anonymous function, like a TS arrow function.
        // Borrow `&text`: lends the value without copying or giving it away.
        // `.map_err(...)`: converts the callee's error into this function's error type.
        // Typed binding: `parsed: ParsedConfig`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        let parsed: ParsedConfig = parse_config(&text).described(format_args!("{}", file.display()))?;
        merged.extend_with(parsed);
    }
    if merged.has_include {
        return Err("a configuration file has include or includeIf, which only Git follows".to_string());
    }
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Borrow `&merged`: lends the value without copying or giving it away.
    check_repository_settings(&merged)?;
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Borrow `&merged`: lends the value without copying or giving it away.
    check_remote_and_branch_settings(&merged)?;
    return Ok(merged);
}
