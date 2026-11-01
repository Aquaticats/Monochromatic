//! What:
//!  Native `git commit` in the shadow repository against the private commit index,
//!       the prepared commit read back from the shadow,
//!       and `prepared.json`.
//! Why:
//!  Git owns the hooks, the editor, the templates, the message cleanup and the signing;
//!  this module only points Git at the shadow, the private index and the hooks directory.
//!  The hooks directory and the silenced hook events come from the hook dispatcher,
//!  which this lane does not build: they arrive through [`PreparationHooks`].
//!  A failing preparation removes the shadow so no shadow outlives the failure
//!  (`src/policy-engine/commit-preparation-native.ts`, `src/shadow-repository/shadow-repository.ts`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await runNativePreparation({ gitPath, cwd, globalArgs, commitArgs, shadowPath, worktreeRoot, hooksDirectory, commitIndexPath });
//! ```

/// Private file write for the journal record.
use super::private_storage::write_exclusive_synced;
/// Journal record shape and its encoder.
use super::transaction_journal::{AddedPath, PreparedRecord};
use super::transaction_journal_encode::encode_prepared;
/// Invocation capture facts the shadow was built from.
use super::capture_invocation::InvocationCapture;
/// Shadow removal, so a failed preparation leaves no shadow.
use super::shadow_git::remove_shadow_repository;
/// The shadow request and the real-Git command builder.
use super::shadow_git::shadow_request;
use super::forwarding::git_command;
use super::signal_relay::{SignalGuard, is_guard_active_refusal};
/// Context, one required Git start and failure text.
use super::transaction_git::{GitContext, GitFailure, GitOutput, run_git_checked};
/// `OsString` is owned operating-system text.
use std::ffi::OsString;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};

/// Variables native Git must not inherit, so every object it writes lands in the shadow store.
pub const NATIVE_UNSET_VARIABLES: [&str; 4] = ["GIT_DIR", "GIT_WORK_TREE", "GIT_COMMON_DIR", "GIT_OBJECT_DIRECTORY"];

/// Name of the journal record a preparation writes.
pub const PREPARED_FILENAME: &str = "prepared.json";

/// What:
///  The hook dispatcher facts preparation needs:
///       the directory holding the dispatcher shim,
///       and the events whose hooks the shim runs instead of Git.
/// Why:
///   The dispatcher is built by another lane.
///  This seam keeps the preparation code independent of that lane,
///  and the incumbent's silenced events are supplied by the caller, not duplicated here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// { hooksDirectory: string; silencedEvents: readonly string[] } // from computeHookDispatchPlan and writeHookShim
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparationHooks {
    /// Directory Git reads hooks from, passed as `core.hooksPath`.
    pub directory: PathBuf,
    /// Events whose native hook is disabled with `hook.<event>.enabled=false`.
    pub silenced_events: Vec<String>,
}

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
    /// Hook dispatcher facts.
    pub hooks: &'request PreparationHooks,
    /// Private commit index.
    pub commit_index: &'request Path,
    /// Kept global options of the invocation.
    pub global_args: &'request [String],
    /// Commit arguments without pathspecs or `--only`.
    pub commit_args: &'request [String],
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
    /// Native `git commit` exited nonzero; its code is the invocation's code.
    NativeExit {
        /// Git's exit code.
        exit_code: i32,
    },
    /// A Git step around the commit failed.
    Git(GitFailure),
    /// `prepared.json` could not be written.
    Write(String),
}

impl std::fmt::Display for PreparationError {
    /// Writes the wording of the failure.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return match self {
            Self::NativeExit { exit_code } => {
                write!(formatter, "git commit exited {exit_code} during private preparation; nothing landed.")
            }
            Self::Git(error) => write!(formatter, "{error}"),
            Self::Write(text) => formatter.write_str(text.as_str()),
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
///   The shadow repository, the work tree, the hooks directory and the silenced events
///  come before the `commit` subcommand, and the kept global options come first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// [...globalArgs, `--git-dir=${shadowPath}`, `--work-tree=${worktreeRoot}`, '-c', `core.hooksPath=${hooksDirectory}`, ...silenced, 'commit', ...commitArgs]
/// ```
fn native_commit_arguments(request: &PreparationRequest) -> Vec<String> {
    let mut arguments: Vec<String> = request.global_args.to_vec();
    arguments.push(format!("--git-dir={}", request.shadow.display()));
    arguments.push(format!("--work-tree={}", request.capture.layout.repository_root.display()));
    arguments.push(String::from("-c"));
    arguments.push(format!("core.hooksPath={}", request.hooks.directory.display()));
    for event in &request.hooks.silenced_events {
        arguments.push(String::from("-c"));
        arguments.push(format!("hook.{event}.enabled=false"));
    }
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
///   The editor and hook output reach the user unchanged,
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
    // A signal sent to the wrapper while Git runs reaches Git when a process sent it; Git's ending decides the status.
    let guard: SignalGuard = install_guard_when_free().map_err(|error| {
        return PreparationError::Git(GitFailure(format!("installing the signal guard failed: {error}")));
    })?;
    let outcome = guard.run_child(&mut command).map_err(|error| {
        return PreparationError::Git(GitFailure(format!("git commit could not start: {error}")));
    })?;
    return Ok(super::forwarding::exit_code(outcome));
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

/// What:
///  Writes `prepared.json` exclusively and makes it durable.
/// Why:
///   Recovery reads the prepared record to decide what a dead transaction had prepared.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeJournalRecord({ directory, filename: PREPARED_FILENAME, record });
/// ```
pub fn write_prepared_record(directory: &Path, record: &PreparedRecord) -> Result<(), PreparationError> {
    return write_exclusive_synced(
        directory.join(PREPARED_FILENAME).as_path(),
        encode_prepared(record).as_bytes(),
    )
    .map_err(|error| { return PreparationError::Write(format!("writing {PREPARED_FILENAME} failed: {error}")); });
}

/// What:
///  The facts `prepared.json` records beyond what the shadow and the index already answer.
/// Why:
///   The landing decides what to replay from these lists, and the policy pass that fills them is a later slice.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// { committedPaths, addedPaths, selectedWorktreePaths }
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PreparedPaths {
    /// Selected and added paths the commit carries.
    pub committed_paths: Vec<Vec<u8>>,
    /// Added paths with the blobs their worktree copies receive after landing.
    pub added_paths: Vec<AddedPath>,
    /// Selected corrections and hook changes that the worktree receives after landing.
    pub selected_worktree_paths: Vec<AddedPath>,
}

/// What:
///  Runs native preparation end to end:
///       native commit, the prepared commit read back, and `prepared.json`.
/// Why:
///   A failure removes the shadow before it returns, so no shadow outlives a failed preparation,
///  and the exit code of a failed native commit is the invocation's exit code.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await runNativePreparation(...); const prepared = await readPreparedCommit(...); await writeJournalRecord(...);
/// ```
pub fn prepare_native_commit(
    context: &GitContext,
    cwd: &Path,
    request: &PreparationRequest,
    journal_directory: &Path,
    intended_tree_oid: &str,
    paths: &PreparedPaths,
) -> Result<PreparedCommit, PreparationError> {
    let exit_code: i32 = match run_native_commit(context, cwd, request) {
        Ok(code) => code,
        Err(error) => {
            remove_failed_shadow(request.shadow);
            return Err(error);
        }
    };
    if exit_code != 0 {
        remove_failed_shadow(request.shadow);
        return Err(PreparationError::NativeExit { exit_code });
    }
    let prepared: PreparedCommit = match read_prepared_commit(context, request.shadow) {
        Ok(commit) => commit,
        Err(error) => {
            remove_failed_shadow(request.shadow);
            return Err(PreparationError::Git(error));
        }
    };
    write_prepared_record(
        journal_directory,
        &PreparedRecord {
            shadow_path: request.shadow.display().to_string(),
            prepared_oid: prepared.oid.clone(),
            signed: prepared.signed,
            intended_tree_oid: intended_tree_oid.to_string(),
            committed_paths: paths.committed_paths.clone(),
            added_paths: paths.added_paths.clone(),
            selected_worktree_paths: paths.selected_worktree_paths.clone(),
        },
    )?;
    return Ok(prepared);
}

/// What:
///  Removes the shadow of a failed preparation.
/// Why:
///   The original failure is the answer, so a removal failure is ignored here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await removeShadowRepository(shadowPath);
/// ```
fn remove_failed_shadow(shadow: &Path) {
    let _ = remove_shadow_repository(shadow);
}

/// Native preparation controls stay out of the release executable.
#[cfg(test)]
#[path = "commit_preparation_tests.rs"]
mod tests;
