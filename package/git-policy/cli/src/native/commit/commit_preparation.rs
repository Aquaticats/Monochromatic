//! What:
//!  Native `git commit` in the shadow repository against the private commit index,
//!       the prepared commit read back from the shadow,
//!       and `prepared.json`.
//! Why:
//!  Git owns the hooks, the editor, the templates, the message cleanup and the signing;
//!  this module only points Git at the shadow and the private index.
//!  Every hook is off in the shadow (its config switches the commit events off and the hooks-off option comes
//!  first), so Git starts no external program for a hook and never starts the wrapper.
//!  A failing preparation removes the shadow so no shadow outlives the failure
//!  (`src/policy-engine/commit-preparation-native.ts`, `src/shadow-repository/shadow-repository.ts`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await runNativePreparation({ gitPath, cwd, globalArgs, commitArgs, shadowPath, hookPlan, commitIndexPath });
//! ```

/// Private file write for the journal record.
/// Journal record shape and its encoder.
/// Invocation capture facts the shadow was built from.
use crate::capture_invocation::InvocationCapture;
/// The hooks-off value Git is given again after the caller's options.
use crate::hooks_off::NO_HOOKS_DIRECTORY;
/// Shadow removal, so a failed preparation leaves no shadow.
/// The shadow request and the real-Git command builder.
use crate::shadow_git::shadow_request;
use crate::forwarding::git_command;
use crate::signal_relay::{SignalGuard, is_guard_active_refusal};
/// Context, one required Git start and failure text.
use crate::transaction_git::{GitContext, GitFailure, GitOutput, run_git_checked};
/// `OsString` is owned operating-system text.
use std::ffi::OsString;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::Path;

/// Variables native Git must not inherit, so every object it writes lands in the shadow store.
pub const NATIVE_UNSET_VARIABLES: [&str; 4] = ["GIT_DIR", "GIT_WORK_TREE", "GIT_COMMON_DIR", "GIT_OBJECT_DIRECTORY"];

/// What:
///  Everything one native preparation needs from its transaction.
/// Why:
///   The incumbent passes these as separate options, and grouping them keeps the call short.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// runNativePreparation({ gitPath, cwd, globalArgs, commitArgs, shadowPath, worktreeRoot, hooksDirectory, commitIndexPath });
/// ```
#[derive(Clone, Copy, Debug)]
pub struct PreparationRequest<'request> {
    /// Invocation capture the shadow was created from.
    pub capture: &'request InvocationCapture,
    /// Shadow repository of the transaction.
    pub shadow: &'request Path,
    /// Private commit index.
    pub commit_index: &'request Path,
    /// Kept global options of the invocation.
    pub global_args: &'request [String],
    /// Commit arguments without pathspecs or `--only`.
    pub commit_args: &'request [String],
    /// The object directory Git writes its new objects into; nothing leaves them in the shadow's own store.
    ///  The commit service names the real one, so the commit it lands is already where the branch can point at it.
    pub object_directory: Option<&'request Path>,
}

/// What:
///  Why a preparation stopped.
/// Why:
///   A nonzero native exit is the invocation's own exit code, while a start failure is an error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class NativeCommitFailedError extends Error { exitCode: number }
/// ```
#[derive(Debug)]
pub enum PreparationError {
    /// A Git step around the commit failed.
    Git(GitFailure),
}

impl std::fmt::Display for PreparationError {
    /// Writes the wording of the failure.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return match self {
            Self::Git(error) => write!(formatter, "{error}"),
        };
    }
}

/// What:
///  The prepared commit read back from the shadow `HEAD`.
/// Why:
///   Replay rebuilds every later commit from the exact object bytes,
///  and the tree and signature decide what landing may do.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PreparedCommit = { oid: string; treeOid: string; signed: boolean; raw: RawCommit };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedCommit {
    /// Prepared commit, the shadow `HEAD` value.
    pub oid: String,
    /// Prepared commit tree.
    pub tree_oid: String,
    /// Whether the header block carries a `gpgsig` or `gpgsig-sha256` header.
    pub signed: bool,
    /// Exact commit object bytes.
    pub raw: Vec<u8>,
}

/// What:
///  Builds the arguments of native `git commit` in the shadow.
/// Why:
///   The kept global options come first, then the shadow, then a hooks directory that cannot exist, which also beats a
///  `-c core.hooksPath` of the caller.
///  The shadow's own config switches every commit event off, so no external program runs for a hook.
///  No `--work-tree` is passed: the shadow's own config names the work tree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// [...globalArgs, `--git-dir=${shadowPath}`, '-c', `core.hooksPath=${NO_HOOKS}`, 'commit', ...commitArgs]
/// ```
fn native_commit_arguments(request: &PreparationRequest) -> Vec<String> {
    let mut arguments: Vec<String> = request.global_args.to_vec();
    arguments.push(format!("--git-dir={}", request.shadow.display()));
    arguments.push(String::from("-c"));
    arguments.push(format!("core.hooksPath={NO_HOOKS_DIRECTORY}"));
    arguments.push(String::from("commit"));
    arguments.extend(request.commit_args.iter().cloned());
    return arguments;
}

/// What:
///  Install the signal guard, waiting while another thread of this process holds it.
/// Why:
///   One wrapper process runs one Git child at a time, so the wait never happens there; library tests run several
///  preparations in threads of one process, and the guard is process-wide.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const guard = installSignalGuard();
/// ```
pub fn install_guard_when_free() -> std::io::Result<SignalGuard> {
    return retry_while_active(SignalGuard::install, GUARD_WAIT_LIMIT, GUARD_WAIT_DELAY);
}

/// The most times the guard installation is retried while another thread holds the guard.
const GUARD_WAIT_LIMIT: u32 = 1_200;

/// The pause between two tries of the guard installation.
const GUARD_WAIT_DELAY: std::time::Duration = std::time::Duration::from_millis(50);

/// What:
///  Run `install` again while it fails only because another holder is active, up to `limit` more times.
/// Why:
///   Any other failure, and a holder that never lets go, end the wait with the failure itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (let attempt = 0; ; attempt += 1) { try { return install(); } catch (e) { if (!isActive(e) || attempt >= limit) throw e; await wait(delay); } }
/// ```
pub fn retry_while_active<Value>(
    mut install: impl FnMut() -> std::io::Result<Value>,
    limit: u32,
    delay: std::time::Duration,
) -> std::io::Result<Value> {
    let mut attempts: u32 = 0;
    loop {
        match install() {
            Ok(value) => return Ok(value),
            Err(error) if is_guard_active_refusal(&error) && attempts < limit => {
                attempts += 1;
                std::thread::sleep(delay);
            }
            Err(error) => return Err(error),
        }
    }
}

/// What:
///  Runs native `git commit` in the shadow with the terminal attached and returns its exit code.
/// Why:
///   The editor output reaches the user unchanged,
///  so the child inherits standard streams.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runTransactionGit({ gitPath, cwd, args, indexPath, unsetEnvironment, stdio: 'inherit', allowFailure: true });
/// ```
pub fn run_native_commit(
    context: &GitContext,
    cwd: &Path,
    request: &PreparationRequest,
) -> Result<i32, PreparationError> {
    let arguments: Vec<OsString> = native_commit_arguments(request).iter().map(OsString::from).collect();
    let mut command = git_command(context.real_git.as_path(), arguments.as_slice(), context.overlay.as_slice());
    command
        .current_dir(cwd)
        .env("GIT_INDEX_FILE", request.commit_index);
    for name in NATIVE_UNSET_VARIABLES {
        command.env_remove(name);
    }
    if let Some(directory) = request.object_directory {
        command.env("GIT_OBJECT_DIRECTORY", directory);
    }
    return run_guarded_child(&mut command);
}

/// What:
///  Run the native commit under the signal guard and return its exit code.
/// Why:
///   A signal sent to the wrapper while Git runs reaches Git when a process sent it; Git's ending decides the status.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// using guard = installSignalGuard(); return exitCode(await guard.runChild(command));
/// ```
fn run_guarded_child(command: &mut std::process::Command) -> Result<i32, PreparationError> {
    let guard: SignalGuard = install_guard_when_free().map_err(|error| {
        return PreparationError::Git(GitFailure(format!("installing the signal guard failed: {error}")));
    })?;
    let outcome = guard.run_child(command).map_err(|error| {
        return PreparationError::Git(GitFailure(format!("git commit could not start: {error}")));
    })?;
    return Ok(crate::forwarding::exit_code(outcome));
}

/// What:
///  Reads the prepared commit from the shadow `HEAD`:
///  its object, its tree and whether it is signed.
/// Why:
///   The header block ends at the first empty line; a signature header in it marks the commit signed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await readPreparedCommit({ gitPath, shadowPath });
/// ```
pub fn read_prepared_commit(context: &GitContext, shadow: &Path) -> Result<PreparedCommit, GitFailure> {
    let oid_output: GitOutput = run_git_checked(
        context,
        &shadow_request(shadow, shadow, &["rev-parse", "--verify", "HEAD^{commit}"]),
    )?;
    let oid: String = String::from_utf8_lossy(oid_output.stdout.as_slice()).trim().to_string();
    let raw: Vec<u8> = run_git_checked(
        context,
        &shadow_request(shadow, shadow, &["cat-file", "commit", oid.as_str()]),
    )?
    .stdout;
    let header_end: usize = raw
        .windows(2)
        .position(|window| { return window == b"\n\n"; })
        .unwrap_or(raw.len());
    let header: String = String::from_utf8_lossy(&raw[..header_end]).into_owned();
    let tree_oid: String = header
        .lines()
        .find_map(|line| { return line.strip_prefix("tree "); })
        .unwrap_or("")
        .to_string();
    let signed: bool = header
        .lines()
        .any(|line| { return line.starts_with("gpgsig ") || line.starts_with("gpgsig-sha256 "); });
    return Ok(PreparedCommit {
        oid,
        tree_oid,
        signed,
        raw,
    });
}

/// Native preparation controls stay out of the release executable.
#[cfg(test)]
#[path = "commit_preparation_tests.rs"]
mod tests;
