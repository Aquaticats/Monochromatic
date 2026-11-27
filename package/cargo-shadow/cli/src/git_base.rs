//! What:
//! Reads the lockfiles of a base revision with `git`,
//! for the frozen install under `trustLockfile`.
//! Why:
//! The base is the pushed range's previous commit or a pull request's base (`doc/decision/rust-dependency-catalog.md`,
//! amended by the owner on 2026-10-29).
//! CI names it in the environment variable below;
//! without it the base is `HEAD`,
//! so a
//! developer's uncommitted lockfile changes are the ones checked.
//! The old TypeScript `rust:verify` took the same revision as
//! `--base`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // git-base.ts
//! export async function createGitBaseLockReader({ root, base }): Promise<(crateDir: string) => string>
//! ```
//! The choice of the base,
//! and the `git` calls it needs,
//! live in `git_base_resolution.rs`;
//! this file keeps what the choice answers with and the reads of one commit.

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
use std::path::Path;
/// What:
/// `use std::process::Command;` imports the process builder.
/// Why:
/// `Command` describes the Git call that reads one lockfile and the one that answers whether a path is ignored.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { spawn } from 'node:child_process';
/// ```
use std::process::Command;
/// Imports the length of time a Git call may take.
use std::time::Duration;

/// Imports the bound and the failure kinds of the bounded child run that the Git reads here go through.
use crate::bounded_run::{
    Bound,
    RunError,
    output_within,
};
/// What:
/// `use crate::constants::LOCKFILE_NAME;` imports `LOCKFILE_NAME` from another module of this crate so this file can name it unqualified.
/// Why:
/// `LOCKFILE_NAME` is needed by the derive or macro expansions of this file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { LOCKFILE_NAME } from 'crate/constants';
/// ```
use crate::constants::LOCKFILE_NAME;
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

/// Which system configuration file the selected Git reads (split out of the git base module).
pub(crate) use crate::git_system_config_lookup::system_config_of_git;

/// How the base is chosen,
/// by the repository's files and then by `git` (split out of the git base module).
pub use crate::git_base_resolution::resolve_trust_base_with;

/// The bounded `git` call that the configuration lookup goes through (split out of the git base module).
pub(crate) use crate::git_base_resolution::run_git;

/// The environment variable that names the base revision.
pub const BASE_VARIABLE: &str = "CARGO_SHADOW_TRUST_BASE";

/// The base used when the variable is absent or empty.
pub const DEFAULT_BASE: &str = "HEAD";

/// How a trust base was chosen and what it resolved to.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustBase {
    /// How the base is spelled (`CARGO_SHADOW_TRUST_BASE`,
    /// `@{upstream}`,
    /// `origin/main`,
    /// `HEAD`).
    pub spelling: String,
    /// The commit it names,
    /// or why it names none.
    pub commit: Result<String, String>,
    /// How many Git processes were started to choose it (none when the files answered).
    pub git_starts: u32,
}

/// What:
/// `pub fn not_a_commit_message(base: &str) -> String` is the text that says a base names no commit.
/// Why:
/// Git and the file reader report the same failure,
/// so they use the same words.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const notACommitMessage = (base: string): string => `The verification base ${JSON.stringify(base)} is not a commit...`;
/// ```
pub fn not_a_commit_message(base: &str) -> String {
    // Macro `format!`: builds or prints text (or a list) from a template.
    return format!(
        "The verification base {base:?} is not a commit in this repository.\nSet {BASE_VARIABLE} to a revision that exists locally (in CI, fetch it first), or leave it unset to compare with the upstream branch (HEAD in CI)."
    );
}

/// What:
/// `pub fn resolve_trust_base(root, in_ci, explicit) -> TrustBase` chooses the revision whose lockfiles count as trusted,
/// reading the running process's environment.
/// Why:
/// The shadow calls this;
/// the tests call `resolve_trust_base_with` with a disposable environment.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function resolveTrustBase(root: string, inCi: boolean, explicit?: string): TrustBase
/// ```
pub fn resolve_trust_base(
    root: &Path,
    in_ci: bool,
    explicit: Option<&str>,
) -> TrustBase {
    // Borrow `&NativeEnvironment`: lends the value without copying or giving it away.
    return resolve_trust_base_with(root, in_ci, explicit, &NativeEnvironment::from_process());
}

/// What:
/// `pub fn git_lock_reader_at(root, commit) -> LockReader` reads one crate root's lockfile text at a commit
/// (empty when the lockfile did not exist there).
/// Why:
/// The base is resolved once,
/// so every read names the same commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function gitLockReaderAt(root: string, commit: string): (crateDir: string) => string
/// ```
#[cfg_attr(
    dylint_lib = "monochromatic_lint_closure",
    expect(
        avoidable_closure,
        reason = "the returned reader keeps the repository root and the commit after this function returns"
    )
)]
pub fn git_lock_reader_at(
    root: &Path,
    commit: &str,
) -> Box<dyn Fn(&str) -> String> {
    let root_path = root.to_path_buf();
    // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
    // Typed binding: `commit_text: String`.
    let commit_text: String = commit.to_string();
    // Constructor `Box::new(`: an empty owned collection or a boxed value.
    return Box::new(move |crate_dir: &str| -> String {
        // Typed binding: `object: String`.
        // Macro `format!`: builds or prints text (or a list) from a template.
        let object: String = format!("{commit_text}:{crate_dir}/{LOCKFILE_NAME}");
        // `let mut`: a binding that may change.
        // Typed binding: `command: Command`.
        let mut command: Command = Command::new("git");
        // Borrow `&object`: lends the value without copying or giving it away.
        command.args(["show", &object]).current_dir(&root_path);
        // Typed binding: `bound: Bound`.
        let bound: Bound = Bound {
            // What:     `Duration` is a length of time and `Instant` a point on the monotonic clock (siblings: the wall-clock `SystemTime`).
            // Why:      Spacing between requests is a length of time, measured on a clock that never jumps back.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // // TS has one type for this: T[] / Set / Map / number
            // ```
            limit: Duration::from_secs(crate::git_refs::GIT_CALL_SECONDS),
            // Macro `format!`: builds or prints text (or a list) from a template.
            what: format!(
                "`git show {object}` (cargo-shadow reads the lockfile of the verification base)"
            ),
            // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
            rerun: crate::bounded_run::rerun_text(),
        };
        // `None`: the present or the absent value of an `Option`.
        // `match`: picks the arm whose pattern fits the value.
        // Borrow `&mut command`: lends the value without copying or giving it away.
        return match output_within(&mut command, None, &bound) {
            Ok(shown) if shown.status.success() => {
                format!("{}\n", String::from_utf8_lossy(&shown.stdout))
            }
            // A lockfile that cannot be read from the base is read as absent there, which checks more, never less.
            Err(
                error @ RunError::TimedOut {
                    ..
                },
            ) => {
                // Macro `eprintln!`: builds or prints text (or a list) from a template.
                eprintln!(
                    "cargo-shadow: warning: {error}; the lockfile is treated as absent at the base, so every version in it is checked"
                );
                // Constructor `String::new()`: an empty owned collection or a boxed value.
                String::new()
            }
            _ => String::new(),
        };
    });
}

/// What:
/// `pub fn is_git_ignored(root: &Path, relative: &str) -> bool` says whether git ignores a path of the repository at `root`
/// (`git check-ignore`,
/// exit status 0).
/// Why:
/// A crate in a directory that git ignores (a gate's disposable build context under `node_modules`,
/// a scratch copy) is not a lockfile
/// of the repository:
/// there is no committed lockfile to compare with,
/// so every locked version would count as newly locked and be held
/// to the trust policy of versions that nobody added.
/// Only a clear "ignored" answer counts;
/// any other answer (not ignored,
/// no
/// repository,
/// a failure,
/// a timeout) leaves the crate under the catalog's policy,
/// so a crate that is merely new is still checked.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function isGitIgnored(root: string, relative: string): boolean
/// ```
pub fn is_git_ignored(
    root: &Path,
    relative: &str,
) -> bool {
    // A plain child with a deadline, not a contained run: the question only reads, it writes nothing that needs a scope to stop, and
    // it must not start a scope (or take a fault meant for the first contained run) on every invocation of the shadow.
    let mut child = match Command::new("git")
        .args(["check-ignore", "--quiet", "--", relative])
        .current_dir(root)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
    {
        Ok(started) => started,
        Err(_) => return false,
    };
    let deadline: std::time::Instant =
        std::time::Instant::now() + Duration::from_secs(crate::git_refs::GIT_CALL_SECONDS);
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return status.code() == Some(0),
            Ok(None) => {}
            Err(_) => return false,
        }
        if std::time::Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return false;
        }
        std::thread::sleep(Duration::from_millis(2));
    }
}

/// What:
/// `pub fn explicit_base_from_environment() -> Option<String>` is the base named by `CARGO_SHADOW_TRUST_BASE`,
/// when it is set
/// and not blank.
/// Why:
/// CI sets it to the pushed range's previous commit or the pull request's base;
/// it wins over every default.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const explicitBaseFromEnvironment = (): string | undefined => process.env.CARGO_SHADOW_TRUST_BASE?.trim() || undefined;
/// ```
pub fn explicit_base_from_environment() -> Option<String> {
    // What:     `match value { pattern => result, ... }` runs the arm whose pattern fits the value and binds the parts the pattern names. The compiler checks that every case is covered.
    // Why:      It separates the success and failure (or present and absent) cases explicitly, with no fall-through.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // switch (kind) { case "a": ...; case "b": ...; }  // or a type-narrowing if/else
    // ```
    return match std::env::var(BASE_VARIABLE) {
        // `Ok(...)`: builds the success or the failure variant of a `Result`.
        // `Some`: the present or the absent value of an `Option`.
        // `.to_string()`/`.to_owned()`: copies the text into a new owned `String`.
        Ok(found) if !found.trim().is_empty() => Some(found.trim().to_string()),
        // `None`: the present or the absent value of an `Option`.
        _ => None,
    };
}
