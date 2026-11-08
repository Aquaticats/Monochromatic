//! What:
//! Which system configuration file the selected Git reads:
//! found by asking that Git,
//! and cached under the repository root by the Git program's identity.
//! Why:
//! Split out of `git_base.rs`,
//! which resolves the base revision of a trust check,
//! to keep that file under the code-line limit.
//! The items are re-exported from `git_base`,
//! so their paths there are unchanged.

/// The Git call the lookup asks through.
use crate::git_base::run_git;
/// Error text built only on failure,
/// from a `format_args!` value.
use crate::lazy_error_text::OrText;
/// What:
/// `use std::path::Path;` imports `Path` from the standard library so this file can name it unqualified.
/// Why:
/// `Path` is used by `git_base_lock_reader`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Path } from 'std/path';
/// ```
use std::cell::Cell;
/// What:
/// `use std::path::Path;` imports
/// `Path`
/// so this file can name them unqualified.
/// Why:
/// `Path` is the borrowed view of a file path (sibling:
/// `PathBuf`,
/// the owned one);
/// the repository root is lent to the functions.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Path } from 'std/path';
/// ```
use std::path::{Path, PathBuf};

/// What:
/// `use crate::git_refs::NativeEnvironment;` imports the description of where the user configuration lives and whether the environment forbids reading the files.
/// Why:
/// Tests pass a disposable one;
/// the shadow passes the process's own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { NativeEnvironment } from 'crate/git_refs';
/// ```
use crate::git_refs::NativeEnvironment;

/// The file in which the system configuration path a Git program reported is remembered,
/// under the repository root.
pub(crate) const SYSTEM_CONFIG_CACHE: &str = ".cache/cargo-shadow/git-system-config";

/// What:
/// `fn find_program(program: &Path, search: &str, working_directory: &Path) -> Option<PathBuf>` finds the executable that `execvp`
/// would run for a program name.
/// Why:
/// The first regular file is not the first executable,
/// and a relative name or search entry is relative to the directory the Git
/// runs in,
/// which is the repository (round 6,
/// 5a F5).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function findProgram(program: string, search: string, workingDirectory: string): string | undefined
/// ```
pub(crate) fn find_program(program: &Path, search: &str, working_directory: &Path) -> Option<PathBuf> {
    if program.components().count() > 1 {
        let anchored: PathBuf = working_directory.join(program);
        return Some(anchored);
    }
    for directory in search.split(':') {
        let candidate: PathBuf = working_directory.join(if directory.is_empty() { "." } else { directory }).join(program);
        if crate::control_group::is_executable_file(&candidate) {
            return Some(candidate);
        }
    }
    return None;
}

/// What:
/// `fn seen_variable(environment: &NativeEnvironment, name: &str) -> String` returns the value that the compiled Git would read for a variable:
/// the last one set on the command,
/// else the caller's when the run keeps the caller's environment,
/// else an empty string.
/// Why:
/// A function replaces the closure that captured the environment.
pub(crate) fn seen_variable(environment: &NativeEnvironment, name: &str) -> String {
    for (key, value) in environment.git.environment.iter().rev() {
        if key == name {
            return value.clone();
        }
    }
    if environment.git.clean_environment {
        return String::new();
    }
    return std::env::var(name).unwrap_or_default();
}

/// What:
/// `pub(crate) fn system_config_of_git(environment, root, starts) -> Result<PathBuf, String>` returns the system configuration file
/// the Git that will answer reads,
/// as that Git reports it (`git var GIT_CONFIG_SYSTEM`).
/// Why:
/// Review finding (4a 4):
/// an assumed `/etc/gitconfig` is wrong whenever the selected Git is built for another prefix,
/// in both
/// directions;
/// the Git that answers knows.
/// The answer is cached under the repository root,
/// keyed by the Git program's canonical path,
/// size and modification time,
/// so the
/// question costs one Git start per Git program and not one per call.
/// A Git that cannot say makes the reader decline.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function systemConfigOfGit(environment: NativeEnvironment, root: string, starts: { count: number }): string  // throws: ask Git
/// ```
pub(crate) fn system_config_of_git(environment: &NativeEnvironment, root: &Path, starts: &Cell<u32>) -> Result<PathBuf, String> {
    // Typed binding: `search: String`.
    // The variable set on the command wins; without one, a run that keeps the caller's environment reads the caller's PATH.
    let mut search: String = String::new();
    let mut set_on_command: bool = false;
    for (name, value) in environment.git.environment.iter() {
        if name == "PATH" {
            search = value.clone();
            set_on_command = true;
            break;
        }
    }
    if !set_on_command && !environment.git.clean_environment {
        search = std::env::var("PATH").unwrap_or_default();
    }
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Borrow `&environment.git.program`: lends the value without copying or giving it away.
    // `.ok()`/`.ok_or()`: converts between `Result` and `Option`.
    // Typed binding: `program: PathBuf`.
    // Macro `format!`: builds or prints text (or a list) from a template.
    // What:     `PathBuf` is an owned file path and `&Path` the borrowed view of one (the same pair as `String` and `&str`).
    // Why:      Joined or stored paths are owned; paths passed in are borrowed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: string
    // ```
    let program: PathBuf = find_program(&environment.git.program, &search, root).or_text(format_args!("{} was not found on the search path, so its system configuration is unknown", environment.git.program.display()))?;
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Closure `|error|`: an anonymous function, like a TS arrow function.
    // Borrow `&program`: lends the value without copying or giving it away.
    // `.map_err(...)`: converts the callee's error into this function's error type.
    // Typed binding: `canonical: PathBuf`.
    // Macro `format!`: builds or prints text (or a list) from a template.
    let canonical: PathBuf = match std::fs::canonicalize(&program) {
        Ok(found) => found,
        Err(error) => return Err(format!("{} cannot be resolved: {error}", program.display())),
    };
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Closure `|error|`: an anonymous function, like a TS arrow function.
    // Borrow `&canonical`: lends the value without copying or giving it away.
    // `.map_err(...)`: converts the callee's error into this function's error type.
    // Macro `format!`: builds or prints text (or a list) from a template.
    let metadata = match std::fs::metadata(&canonical) {
        Ok(found) => found,
        Err(error) => return Err(format!("{} cannot be inspected: {error}", canonical.display())),
    };
    // Closure `|time|`: an anonymous function, like a TS arrow function.
    // `.unwrap_or*`: takes the value out, or the fallback when it is absent or failed.
    // `.ok()`/`.ok_or()`: converts between `Result` and `Option`.
    // Adapter `.and_then(`: transforms or selects items with a closure.
    // Typed binding: `modified_nanos: u128`.
    let modified_nanos: u128 = match metadata.modified() {
        Ok(time) => match time.duration_since(std::time::UNIX_EPOCH) {
            Ok(elapsed) => elapsed.as_nanos(),
            Err(_) => 0,
        },
        Err(_) => 0,
    };
    // Typed binding: `key: String`.
    // Macro `format!`: builds or prints text (or a list) from a template.
    // An answer is reused only for a compiled program: a script may forward to any Git, chosen by files, variables or the
    // directory, so nothing about the script says which Git answers (round 6, 5a F5).
    let compiled: bool = {
        let mut magic: [u8; 4] = [0; 4];
        let read_magic: bool = match std::fs::File::open(&canonical) {
            Ok(mut file) => std::io::Read::read_exact(&mut file, &mut magic).is_ok(),
            Err(_) => false,
        };
        read_magic && magic == *b"\x7fELF"
    };
    // What the compiled program reads to find its system configuration: its own file and the two variables that move or remove it.
    let key: String = {
        format!(
            "{}\t{}\t{}\t{}\t{modified_nanos}\t{}\t{}\t{}\t{}",
            canonical.display(),
            std::os::unix::fs::MetadataExt::dev(&metadata),
            std::os::unix::fs::MetadataExt::ino(&metadata),
            metadata.len(),
            std::os::unix::fs::MetadataExt::ctime_nsec(&metadata) + std::os::unix::fs::MetadataExt::ctime(&metadata) * 1_000_000_000,
            seen_variable(environment, "GIT_CONFIG_SYSTEM"),
            seen_variable(environment, "GIT_CONFIG_NOSYSTEM"),
            seen_variable(environment, "GIT_EXEC_PATH")
        )
    };
    // Typed binding: `cache: PathBuf`.
    let cache: PathBuf = root.join(SYSTEM_CONFIG_CACHE);
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
    // Borrow `&cache`: lends the value without copying or giving it away.
    if compiled && let Ok(text) = std::fs::read_to_string(&cache) {
        // `let mut`: a binding that may change.
        let mut lines = text.lines();
        // `Some`: the present or the absent value of an `Option`.
        if lines.next() == Some(key.as_str())
            // `Some`: the present or the absent value of an `Option`.
            && let Some(path) = lines.next()
            && !path.is_empty()
        {
            // `Ok(...)`: builds the success or the failure variant of a `Result`.
            return Ok(PathBuf::from(path));
        }
    }
    starts.set(starts.get() + 1);
    // `None`: the present or the absent value of an `Option`.
    // `?`: returns the failure of the call before it to the caller, otherwise unwraps the value.
    // Borrow `&[`: lends the value without copying or giving it away.
    let asked = run_git(environment, root, &["var", "GIT_CONFIG_SYSTEM"], None)?;
    // Borrow `&asked.stdout`: lends the value without copying or giving it away.
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Typed binding: `reported: String`.
    let reported: String = String::from_utf8_lossy(&asked.stdout).trim().to_string();
    if !asked.status.success() || reported.is_empty() {
        // `Err(...)`: builds the success or the failure variant of a `Result`.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        return Err("the Git that answers does not report its system configuration, so it is not assumed".to_string());
    }
    // `Some`: the present or the absent value of an `Option`.
    // `if let`/`let else`: continues only when the value fits the pattern, binding its parts.
    if let Some(parent) = cache.parent() {
        // What:     `let _ = expr;` runs the expression and discards its result, including a failure.
        // Why:      The outcome does not matter here (a best-effort write or cleanup), and the compiler would otherwise warn about an unused `Result`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // void expr;   // or try { expr } catch {}
        // ```
        let _ = crate::journal::create_directories_synced(parent);
    }
    // Borrow `&cache`: lends the value without copying or giving it away.
    // Macro `format!`: builds or prints text (or a list) from a template.
    // `let _ =`: discards the result on purpose.
    if compiled {
        let _ = crate::journal::atomic_write(&cache, &format!("{key}\n{reported}\n"));
    }
    // `Ok(...)`: builds the success or the failure variant of a `Result`.
    return Ok(PathBuf::from(reported));
}
