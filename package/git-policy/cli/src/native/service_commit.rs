//! What:
//!  The commit command's path through the resident service: prepare the commit in this process (private index,
//!  policies), send the prepared change to the service, and report what it landed.
//! Why:
//!  The service is the only writer, so a commit takes no lock and polls nothing: preparation and policies run here,
//!  outside the service's serialized step, and the service moves the branch once for a whole batch of commits
//!  (`doc/planning/cli-git-resident-service.md`, "Where preparation and policies run").
//!  This slice covers the plain scoped commit (`git commit -m <message> -- <paths>`); any other command line, a commit
//!  in a merge, on an unborn branch or on a detached head, a policy that rewrites a file, and anything with nothing to
//!  commit return `None` so native `git commit` handles them as before. It is on only when the person opts in.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const result = await tryServiceCommit(invocation, checks, request); // undefined when native git commit must run
//! ```

use super::capture_invocation::{InvocationCapture, base_revision, capture_invocation_base, capture_invocation_layout};
use super::capture_order::{CaptureIndexes, indexed_request};
use super::commit_convergence::{CommitConvergenceInput, converge_commit_policies};
use super::commit_facts::PrivateIndexFacts;
use super::commit_index::{IndexSelection, initialize_commit_index, list_changed_index_paths, list_unmerged_index_paths};
use super::commit_transaction::{CommitInvocation, TransactionResult, TransactionStop, base_object, decide_selection, failed, private_checks, staged_at_base};
use super::lfs_pre_push::uses_lfs;
use super::policy_checks::ShippedChecks;
use super::policy_engine::{StageRequest, pass_exit_code};
use super::policy_events::PolicyEvent;
use super::private_index_stage::PrivateStage;
use super::private_storage::{create_private_directory, remove_tree};
use super::random_id::random_uuid;
use super::repository_facts::RepositoryFacts;
use super::service_client::{Activation, COMMIT_WAIT, ClientConfig, START_WAIT, Submitted, submit};
use super::service_commit_args::parse_service_commit;
use super::service_message::{author_overrides, build_message, identity};
use super::service_land::LANDING_ATTEMPTS;
use super::service_messages::{ClientFailure, render};
use super::service_protocol::{ALLOWED_MODES, Change, CommitRequest, MAX_CHANGES, is_object_name, is_plain_path};
use super::service_handoff::ServiceHandoff;
use super::service_install::{SOCKET_UNIT_NAME, platform_without_manager, unit_directory};
use super::service_server::Bind;
use super::config_snapshot::ConfigSnapshot;
use super::transaction_git::{GitContext, run_git_checked};
use super::transaction_journal::{Base, Conclusion, SymbolicHead, TransactionMode};
use std::ffi::OsString;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Environment variable that steers landing scoped commits through the service:
///  unset uses the service when its activation is installed for the account, `0` never uses it, `1` always uses it
///  (and stops with the plain install message when it is not installed), and `abstract:<name>` names a service at an
///  abstract address (tests).
pub const SERVICE_VARIABLE: &str = "CLI_GIT_COMMIT_SERVICE";

/// The socket's place under the user's runtime directory.
pub const SOCKET_RELATIVE: &str = "cli-git/service.sock";

/// What:
///  Where the service listens and what the operating system can do about it, from the variable's value, the runtime
///  directory and the account's unit directory.
/// Why:
///   The default is on once the activation is installed. A machine without it keeps the commit path it had until that
///  path is retired; `1` asks for the service and so gets the plain stop when it cannot be reached. A socket that
///  exists counts as installed, whoever made it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// chooseService(value, runtimeDir, unitDir)
/// ```
pub fn choose_service(value: Option<&str>, runtime_dir: Option<&Path>, unit_dir: Option<&Path>) -> Option<(Bind, Activation)> {
    if value == Some("0") {
        return None;
    }
    if let Some(name) = value.and_then(|text| { return text.strip_prefix("abstract:"); }) {
        return if name.is_empty() { None } else { Some((Bind::Abstract(name.to_string()), Activation::Managed)) };
    }
    if value.is_some_and(|text| { return text != "1"; }) {
        return None;
    }
    let socket: PathBuf = runtime_dir?.join(SOCKET_RELATIVE);
    let asked: bool = value == Some("1");
    if !cfg!(target_os = "linux") {
        return if asked { Some((Bind::Path(socket), Activation::Unsupported(platform_without_manager()))) } else { None };
    }
    let installed: bool = socket.exists() || unit_dir.is_some_and(|directory| { return directory.join(SOCKET_UNIT_NAME).is_file(); });
    return match (installed, asked) {
        (true, _) => Some((Bind::Path(socket), Activation::Managed)),
        (false, true) => Some((Bind::Path(socket), Activation::NotInstalled)),
        (false, false) => None,
    };
}

/// What the service path returns: the transaction result for the lifecycle, and the handoff when the commit landed.
pub struct ServiceCommit {
    /// What the policies and the landing came to.
    pub result: TransactionResult,
    /// How to leave the after-commit work for the service; absent when nothing landed.
    pub handoff: Option<ServiceHandoff>,
}

/// A scratch directory for the private indexes, removed when it goes out of scope.
struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _removed = remove_tree(self.0.as_path());
    }
}

/// What a preparation came to.
enum Prepared {
    /// Native `git commit` must handle this command.
    Decline,
    /// The policies stopped the commit.
    Blocked(TransactionResult),
    /// The change is ready to send.
    Ready(Box<Ready>),
}

struct Ready {
    repository: PathBuf,
    common_dir: PathBuf,
    reference: String,
    base: String,
    changes: Vec<Change>,
    events: Vec<PolicyEvent>,
    uses_lfs: bool,
}

/// The changes between the base and the private index: path, mode and object, deletions as no object.
fn changes_of(context: &GitContext, cwd: &Path, indexes: &CaptureIndexes, base: &str) -> Result<Option<Vec<Change>>, TransactionStop> {
    let arguments: Vec<String> = ["diff-index", "--cached", "-r", "--raw", "-z", "--no-renames", "--no-abbrev", base].map(String::from).to_vec();
    let output = run_git_checked(
        context,
        &indexed_request(cwd, &arguments, Some(indexes.commit_index.as_path()), Some(indexes.object_directory.as_path())),
    )?;
    let text: String = match String::from_utf8(output.stdout) {
        Ok(text) => text,
        Err(_) => return Ok(None),
    };
    let mut fields = text.split('\0').filter(|field| { return !field.is_empty(); });
    let mut changes: Vec<Change> = Vec::new();
    while let Some(header) = fields.next() {
        let Some(path) = fields.next() else { return Ok(None) };
        let parts: Vec<&str> = header.trim_start_matches(':').split(' ').collect();
        let [_, new_mode, _, new_name, status] = parts.as_slice() else { return Ok(None) };
        let deletion: bool = status.starts_with('D');
        let mode: u32 = if deletion { ALLOWED_MODES[0] } else { u32::from_str_radix(new_mode, 8).unwrap_or(0) };
        if !is_plain_path(path) || !ALLOWED_MODES.contains(&mode) || (!deletion && !is_object_name(new_name)) {
            return Ok(None);
        }
        changes.push(Change { path: path.to_string(), mode, blob: if deletion { None } else { Some((*new_name).to_string()) } });
    }
    return Ok(if changes.len() > MAX_CHANGES { None } else { Some(changes) });
}

/// Prepares the commit once against the branch as it is now.
fn prepare<F: RepositoryFacts>(
    invocation: &CommitInvocation<'_>,
    checks: &mut ShippedChecks<F>,
    request: &StageRequest,
    scratch_name: &str,
    allow_empty: bool,
) -> Result<Prepared, TransactionStop> {
    let context: &GitContext = invocation.context;
    let cwd: &Path = invocation.cwd;
    let layout = capture_invocation_layout(context, cwd, false)?;
    if layout.conclusion != Conclusion::None {
        return Ok(Prepared::Decline);
    }
    let selection = decide_selection(invocation.region, layout.conclusion);
    let SymbolicHead::Branch(reference) = layout.symbolic_head.clone() else { return Ok(Prepared::Decline) };
    // An index commit takes the whole index (`-a` first stages every tracked change); a commit of named paths takes only
    //  those. A mix of the two, or anything that stages by hand, is the transaction's.
    let named: bool = selection.mode == TransactionMode::ExplicitPath;
    if selection.read_only || (selection.mode == TransactionMode::Index && selection.selects_paths) {
        return Ok(Prepared::Decline);
    }
    let capture: InvocationCapture = capture_invocation_base(context, cwd, layout)?;
    if !matches!(capture.base, Base::Commit(_)) || config_needs_native(capture.layout.config_snapshot.as_ref()) {
        return Ok(Prepared::Decline);
    }
    let base: String = base_revision(&capture);
    let scratch: Scratch = Scratch(capture.layout.git_dir.join(scratch_name));
    create_private_directory(scratch.0.as_path()).map_err(|error| { return failed(format!("preparing the commit failed: {error}")); })?;
    let indexes: CaptureIndexes = CaptureIndexes {
        captured_index: scratch.0.join("captured.index"),
        commit_index: scratch.0.join("commit.index"),
        object_directory: capture.layout.object_directory.clone(),
    };
    let region_arguments: &[OsString] = &invocation.arguments[invocation.subcommand_index + 1..];
    let pathspecs: Vec<OsString> = invocation.region.pathspecs.iter().map(|at| { return region_arguments[*at].clone(); }).collect();
    let index_selection: IndexSelection<'_> = IndexSelection {
        mode: selection.mode,
        pathspecs: if named { pathspecs.as_slice() } else { &[] },
        pathspec_file: None,
        pathspec_file_nul: false,
        stage_into_index: false,
        stage_tracked_changes: selection.staged_all,
        base_revision: base.as_str(),
    };
    initialize_commit_index(context, cwd, capture.layout.real_index_path.as_path(), &indexes, &index_selection)
        .map_err(|error| { return failed(error.0); })?;
    if !list_unmerged_index_paths(context, cwd, indexes.commit_index.as_path())?.is_empty() {
        return Ok(Prepared::Decline);
    }
    let candidate_paths: Vec<Vec<u8>> =
        list_changed_index_paths(context, cwd, indexes.commit_index.as_path(), base.as_str(), indexes.object_directory.as_path())?;
    if candidate_paths.is_empty() && !allow_empty {
        return Ok(Prepared::Decline);
    }
    let mut private: ShippedChecks<PrivateIndexFacts<&mut F>> = private_checks(
        checks,
        indexes.commit_index.as_path(),
        indexes.object_directory.as_path(),
        staged_at_base(base_object(&capture)?, candidate_paths.as_slice()),
    );
    let outcome = converge_commit_policies(CommitConvergenceInput {
        request,
        checks: &mut private,
        stage: PrivateStage {
            context,
            cwd,
            index: indexes.commit_index.as_path(),
            object_directory: Some(indexes.object_directory.as_path()),
        },
    });
    let code: i32 = pass_exit_code(outcome.pass.events.as_slice(), outcome.pass.end);
    if code != 0 {
        return Ok(Prepared::Blocked(TransactionResult { events: outcome.pass.events, exit_code: code, landed_oid: None, uses_lfs: false }));
    }
    if !outcome.changed_paths.is_empty() {
        // A policy rewrote a file: the worktree copy of that rewrite is native work.
        return Ok(Prepared::Decline);
    }
    let Some(changes) = changes_of(context, cwd, &indexes, base.as_str())? else { return Ok(Prepared::Decline) };
    if changes.is_empty() && !allow_empty {
        return Ok(Prepared::Decline);
    }
    return Ok(Prepared::Ready(Box::new(Ready {
        repository: capture.layout.repository_root.clone(),
        common_dir: capture.layout.common_dir.clone(),
        reference,
        base,
        changes,
        events: outcome.pass.events,
        uses_lfs: capture.layout.config_snapshot.as_ref().is_none_or(uses_lfs),
    })));
}

/// Whether the repository's configuration makes native `git commit` do something the service does not: sign, apply a
///  template or a clean-up mode, use another message encoding or another comment character, or change trailers.
fn config_needs_native(snapshot: Option<&ConfigSnapshot>) -> bool {
    const KEYS: [&str; 7] = ["commit.template", "commit.cleanup", "i18n.commitencoding", "core.commentchar", "core.commentstring", "commit.signoff", "commit.verbose"];
    let Some(found) = snapshot else { return true };
    return found.entries().iter().any(|entry| {
        let value: String = entry.value.as_deref().map(|bytes| { return String::from_utf8_lossy(bytes).to_lowercase(); }).unwrap_or_default();
        let signs: bool = entry.key == "commit.gpgsign" && !["false", "no", "off", "0"].contains(&value.as_str());
        return signs || KEYS.contains(&entry.key.as_str()) || entry.key.starts_with("trailer.");
    });
}

/// What:
///  Lands the commit through the service when the person opted in and the command line is one the service covers.
/// Why:
///   Returns `None` for everything else, before anything was changed, so native `git commit` runs unchanged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await tryServiceCommit(invocation, checks, request)
/// ```
pub fn try_service_commit<F: RepositoryFacts>(
    invocation: &CommitInvocation<'_>,
    checks: &mut ShippedChecks<F>,
    request: &StageRequest,
) -> Result<Option<ServiceCommit>, TransactionStop> {
    let path_variable = |name: &str| -> Option<PathBuf> { return std::env::var_os(name).filter(|value| { return !value.is_empty(); }).map(PathBuf::from); };
    let runtime: Option<PathBuf> = path_variable("XDG_RUNTIME_DIR");
    let units: Option<PathBuf> = unit_directory(path_variable("XDG_CONFIG_HOME").as_deref(), path_variable("HOME").as_deref());
    let value: Option<String> = std::env::var(SERVICE_VARIABLE).ok();
    let Some((address, activation)) = choose_service(value.as_deref(), runtime.as_deref(), units.as_deref()) else {
        return Ok(None);
    };
    let region = invocation.region;
    if region.include || region.interactive || region.patch || region.dry_run || region.amend || region.pathspec_from_file.is_some() {
        return Ok(None);
    }
    let after: &[OsString] = invocation.arguments.get(invocation.subcommand_index + 1..).unwrap_or(&[]);
    let Some(parsed) = parse_service_commit(after) else { return Ok(None) };
    let quiet: bool = parsed.quiet;
    let Some(committer) = identity(invocation.context, invocation.cwd, "GIT_COMMITTER_IDENT", &[])? else { return Ok(None) };
    let Some(built) = build_message(invocation, &parsed, committer.as_str())? else { return Ok(None) };
    let message: String = built.message;
    let id: String = random_uuid().map_err(|error| { return failed(format!("an identity for this commit could not be made: {error:?}")); })?;
    let author: String = match built.reused_author {
        Some(reused) => reused,
        None => {
            let Some(made) = identity(invocation.context, invocation.cwd, "GIT_AUTHOR_IDENT", author_overrides(&parsed).as_slice())? else {
                return Ok(None);
            };
            made
        }
    };
    let allow_empty: bool = region.allow_empty;
    let config: ClientConfig = ClientConfig { address, activation, start_wait: START_WAIT, commit_wait: COMMIT_WAIT };
    let scratch_name: String = format!("cli-git-service-{id}");
    for attempt in 0..LANDING_ATTEMPTS {
        let ready: Box<Ready> = match prepare(invocation, checks, request, scratch_name.as_str(), allow_empty)? {
            Prepared::Decline => return Ok(None),
            Prepared::Blocked(result) => return Ok(Some(ServiceCommit { result, handoff: None })),
            Prepared::Ready(ready) => ready,
        };
        let sent: CommitRequest = CommitRequest {
            id: id.clone(),
            repository: ready.repository.clone(),
            common_dir: ready.common_dir.clone(),
            reference: ready.reference.clone(),
            base: ready.base.clone(),
            changes: ready.changes.clone(),
            message: message.clone(),
            author: Some(author.clone()),
            committer: committer.clone(),
        };
        match submit(&config, &sent, &mut std::io::stderr()) {
            Ok(Submitted::Landed { commit, .. }) => {
                if !quiet {
                    let subject: &str = message.lines().next().unwrap_or("");
                    let short: &str = commit.get(..7).unwrap_or(commit.as_str());
                    let branch: &str = ready.reference.trim_start_matches("refs/heads/");
                    let _printed = writeln!(std::io::stdout(), "[{branch} {short}] {subject}");
                }
                let handoff: ServiceHandoff = ServiceHandoff {
                    config: config.clone(),
                    repository: ready.repository.clone(),
                    common_dir: ready.common_dir.clone(),
                    reference: ready.reference.clone(),
                    parent: ready.base.clone(),
                };
                let result: TransactionResult = TransactionResult { events: ready.events, exit_code: 0, landed_oid: Some(commit), uses_lfs: ready.uses_lfs };
                return Ok(Some(ServiceCommit { result, handoff: Some(handoff) }));
            }
            Ok(Submitted::PrepareAgain { .. }) => {
                if attempt + 1 == LANDING_ATTEMPTS {
                    let branch: String = ready.reference.trim_start_matches("refs/heads/").to_string();
                    return Err(TransactionStop::Message(render(&ClientFailure::BranchKeepsChanging { branch, times: LANDING_ATTEMPTS })));
                }
            }
            Err(failure) => return Err(TransactionStop::Message(render(&failure))),
        }
    }
    return Ok(None);
}

/// Service commit controls stay out of the release executable.
#[cfg(test)]
#[path = "service_commit_tests.rs"]
mod tests;
