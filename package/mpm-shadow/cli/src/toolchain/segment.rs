//! What:
//! Whether a name that comes from the plan,
//! the lock or a release tag can be used as one directory name under the toolchain root:
//! a tool id (after `install_dir_name` maps it),
//! a version,
//! a crate's package name and a bin name.
//! Why:
//! These names are joined onto a path,
//! and `remove_dir_all` runs on some of the joined paths.
//! A name such as `..` or `../x` would make the join leave the tool's directory,
//! so every boundary that accepts a name asks here first,
//! and `tool_install_dir` asks again before it builds a path.
//! The check validates and never rewrites:
//! the lock already holds versions such as `temurin-21.0.13+9.0.LTS` and `b9829`,
//! and renaming them would orphan the installs.
//!
//! The toolchain root also holds directories that belong to no tool:
//! `cargo` (the `cargo-binstall` install roots),
//! `downloads`,
//! `exposure` and `launchers` (the global scope's link directories) and the file `bin.config.json`.
//! A tool id that maps to one of them is refused.
//! The ids `bin` and `node` also name directories the toolchain uses (`bin` is where the `bin` program downloads to,
//! `node` is where the pnpm route looks for Node),
//! but they are the tools that own those directories and the shipped lock names both,
//! so they stay allowed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // segment.ts
//! export function nameProblem(text: string): string | undefined
//! export function checkToolId(id: string): void  // throws, naming the id
//! export function checkVersion(version: string): void  // throws, naming the version
//! export function checkBinName(name: string): void  // throws, naming the bin
//! export function checkDirectoryName(label: string, text: string): void  // throws
//! ```

/// Imports `CURRENT_LINK_NAME` from `super::cargo_installs`.
use super::cargo_installs::CURRENT_LINK_NAME;
/// Imports `install_dir_name` from `super::layout`.
use super::layout::install_dir_name;

/// The names under the toolchain root that belong to no tool,
/// so no tool id may map to one of them.
pub const RESERVED_TOOL_DIRECTORIES: [&str; 5] = ["cargo", "downloads", "exposure", "launchers", "bin.config.json"];

/// What:
/// Why `text` cannot be a single plain file or directory name,
/// or `None` when it can.
/// Why:
/// The empty name,
/// `.` and `..` name directories that already exist,
/// and a separator or a NUL character ends the segment early.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function nameProblem(text: string): string | undefined
/// ```
pub fn name_problem(text: &str) -> Option<&'static str> {
    if text.is_empty() {
        return Some("is empty");
    }
    if text == "." || text == ".." {
        return Some("is a reference to a directory (`.` or `..`), not a name");
    }
    if text.contains(['/', '\\', '\0']) {
        return Some("holds a path separator (`/` or `\\`) or a NUL character");
    }
    return None;
}

/// Why `text` cannot be the name of a directory the installer creates under a tool's directory,
/// or `None` when it can:
/// the problems of `name_problem`,
/// and a leading dot,
/// because the installer gives its own staging directories dot-names and the version scan skips dot-names.
pub fn directory_name_problem(text: &str) -> Option<&'static str> {
    if let Some(problem) = name_problem(text) {
        return Some(problem);
    }
    if text.starts_with('.') {
        return Some("starts with a dot, which the installer keeps for its own hidden staging directories");
    }
    return None;
}

/// What a person can do about a refused plan name (an id,
/// a bin name or a package name).
const PLAN_REMEDY: &str = "rename it in toolchain.plan.jsonc and run `mpm-shadow update` to write the lock again";

/// What a person can do about a refused version.
const VERSION_REMEDY: &str = "pin a version that is one plain name in the request, \
                              or correct the version in toolchain.lock.json if the lock was edited by hand";

/// The refusal text for a name that is not one plain directory name:
/// what it is called here (`label`),
/// the name itself,
/// the problem,
/// and what the person can do about it.
fn refusal(label: &str, text: &str, problem: &str, remedy: &str) -> String {
    return format!("{label} {text:?} {problem}, so it cannot be used as a directory name under the toolchain root; {remedy}");
}

/// What:
/// Checks a name that becomes a directory name as it is,
/// and returns the refusal text when it cannot.
/// Why:
/// The crate directory of the `cargo-binstall` route is built from the package name this way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function checkDirectoryName(label: string, text: string): void  // throws
/// ```
pub fn check_directory_name(label: &str, text: &str) -> Result<(), String> {
    if let Some(problem) = directory_name_problem(&install_dir_name(text)) {
        return Err(refusal(label, text, problem, PLAN_REMEDY));
    }
    return Ok(());
}

/// What:
/// Checks a tool id:
/// the directory name `install_dir_name` makes of it must be one plain name,
/// and not one of `RESERVED_TOOL_DIRECTORIES`.
/// Why:
/// `install_dir_name` already turns `/`, `\` and NUL into `-`,
/// so what remains to refuse is a dot-name and the names the toolchain root uses for other things.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function checkToolId(id: string): void  // throws
/// ```
pub fn check_tool_id(id: &str) -> Result<(), String> {
    let directory: String = install_dir_name(id);
    if let Some(problem) = directory_name_problem(&directory) {
        return Err(refusal("the tool id", id, &format!("becomes the directory name {directory:?}, which {problem}"), PLAN_REMEDY));
    }
    if RESERVED_TOOL_DIRECTORIES.contains(&directory.as_str()) {
        return Err(refusal(
            "the tool id",
            id,
            &format!("becomes the directory name {directory:?}, which the toolchain root already uses for something that is no tool"),
            PLAN_REMEDY,
        ));
    }
    return Ok(());
}

/// What:
/// Checks a version:
/// one plain name,
/// not `current` (the link that names the running version).
/// Why:
/// The version is the name of the directory the tool installs into.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function checkVersion(version: string): void  // throws
/// ```
pub fn check_version(version: &str) -> Result<(), String> {
    if let Some(problem) = directory_name_problem(version) {
        return Err(refusal("the version", version, problem, VERSION_REMEDY));
    }
    if version == CURRENT_LINK_NAME {
        return Err(refusal("the version", version, "is the name of the link that points at the running version", VERSION_REMEDY));
    }
    return Ok(());
}

/// What:
/// Checks a bin name:
/// one plain name.
/// Why:
/// The name is joined onto the exposure and launcher directories to make a link,
/// and searched for inside the install.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function checkBinName(name: string): void  // throws
/// ```
pub fn check_bin_name(name: &str) -> Result<(), String> {
    if let Some(problem) = name_problem(name) {
        return Err(refusal("the bin name", name, problem, PLAN_REMEDY));
    }
    return Ok(());
}
