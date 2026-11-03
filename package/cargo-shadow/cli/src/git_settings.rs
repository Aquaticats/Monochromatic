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

/// Imports the path types.
use std::path::PathBuf;

/// Imports the configuration types and the lexer entry point.
use crate::git_config::{ConfigEntry, ParsedConfig, parse_config};
/// Imports the layout and environment types and the file readers.
use crate::git_refs::{GitLayout, NativeEnvironment, read_optional};
/// Imports the refspec check.
use crate::git_refspec::check_refspec;

/// The remote and branch keys that mean true when written without a value.
pub(crate) const BARE_KEYS: [&str; 7] = ["skipdefaultupdate", "skipfetchall", "prune", "prunetags", "mirror", "promisor", "rebase"];

/// What:
/// `fn check_remote_and_branch_settings(config: &ParsedConfig) -> Result<(), String>` checks every `remote.*` and `branch.*`
/// variable,
/// not only the ones the chosen branch uses.
/// Why:
/// Review finding (3a 6):
/// Git loads all remote and branch configuration when it resolves an upstream,
/// so one invalid refspec
/// or one required string written without a value anywhere makes Git fail where a narrower check would not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkRemoteAndBranchSettings(config: ParsedConfig): void  // throws: ask Git
/// ```
pub(crate) fn check_remote_and_branch_settings(config: &ParsedConfig) -> Result<(), String> {
    // Borrow `&config.entries`: lends the value without copying or giving it away.
    for entry in &config.entries {
        // `Some`: the present or the absent value of an `Option`.
        let Some(rest) = entry.name.strip_prefix("remote.").or_else(|| { return entry.name.strip_prefix("branch."); }) else {
            continue;
        };
        // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
        // Typed binding: `key: &str`.
        let key: &str = rest.rsplit('.').next().unwrap_or("");
        if entry.name.starts_with("remote.") && key == "fetch" {
            // `match`: picks the arm whose pattern fits the value.
            // Borrow `&entry.value`: lends the value without copying or giving it away.
            match &entry.value {
                // `Some`: the present or the absent value of an `Option`.
                // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
                Some(spec) => check_refspec(spec)?,
                // `Err(...)`: builds the success or the failure variant of a `Result`.
                // `None`: the present or the absent value of an `Option`.
                // Macro `format!`: builds or prints text (or a list) from a template.
                None => return Err(format!("{} is written without a value, which Git rejects", entry.name)),
            }
            continue;
        }
        // Borrow `&key`: lends the value without copying or giving it away.
        if entry.value.is_none() && !BARE_KEYS.contains(&key) {
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            // Macro `format!`: builds or prints text (or a list) from a template.
            return Err(format!("{} is written without a value, which Git rejects", entry.name));
        }
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    return Ok(());
}

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
pub fn read_config(layout: &GitLayout, environment: &NativeEnvironment) -> Result<ParsedConfig, String> {
    // Review finding (3a 7): a system configuration in a place another Git build reads is a file this reader does not follow.
    for other in &environment.other_system_configs {
        if other.exists() {
            // `Err(...)`: builds the success or the failure variant of a `Result`.
            // Macro `format!`: builds or prints text (or a list) from a template.
            return Err(format!("{} exists; a Git built for another prefix reads its system configuration there, so only Git can say what it holds", other.display()));
        }
    }
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
        let parsed: ParsedConfig = parse_config(&text).map_err(|message| { return format!("{}: {message}", file.display()); })?;
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
