//! What:
//!  The commit command's path through the resident service: prepare the commit in this process (private index,
//!  policies), send the prepared change to the service, and report what it landed.
//! Why:
//!  The service is the only writer, so a commit takes no lock and polls nothing: preparation and policies run here,
//!  outside the service's serialized step, and the service moves the branch once for a whole batch of commits
//!  (`doc/planning/cli-git-resident-service.md`, "Where preparation and policies run").
//!  It covers the commit lines `service_commit_args.rs` accepts (message and identity options, sign-off, trailers, a
//!  reused or fixup message, `-a`, an index commit, and `--amend`); any other command line, a commit in a merge, on an
//!  unborn branch or on a detached head, a policy that rewrites a file, and anything with nothing to commit return `None`
//!  so the transaction handles them as before. Which route a commit takes is decided by `service_route.rs`.
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
use super::diagnostic_log::warn;
use super::service_commit_args::{ServiceCommitArgs, opens_editor, parse_service_commit};
use super::service_native::{BuildInput, GitBuilt, build_with_git};
use super::service_amend::{AmendSource, amended_author, read_amend_source};
use super::service_message::{author_overrides, build_message, identity};
use super::service_land::LANDING_ATTEMPTS;
use super::service_messages::{ClientFailure, render};
use super::service_protocol::{ALLOWED_MODES, Change, CommitRequest, MAX_CHANGES, is_object_name, is_plain_path};
use super::service_handoff::ServiceHandoff;
use super::service_provision::provision;
use super::service_pruned::{PRUNE_RETRIES, content_was_removed, prepared_content_missing, reach_service_phase};
use super::transaction_phase::Phase;
use super::service_route::{Route, RouteInputs, choose_route, remembered_own_executable};
use super::config_snapshot::ConfigSnapshot;
use super::transaction_git::{GitContext, run_git_checked};
use super::transaction_journal::{Base, Conclusion, SymbolicHead, TransactionMode};
use std::ffi::OsString;
use std::io::Write;
use std::path::{Path, PathBuf};

/// What the service path returns: the transaction result for the lifecycle, and the handoff when the commit landed.
pub struct ServiceCommit {
    /// What the policies and the landing came to.
    pub result: TransactionResult,
    /// How to leave the after-commit work for the service; absent when nothing landed.
    pub handoff: Option<ServiceHandoff>,
}

/// How many times a commit Git built is built again on the new tip when the branch moves before it lands, before the
///  plain failure. A build is one signature, so the bound keeps a busy branch from keeping a person's command running.
pub const REBUILD_ATTEMPTS: usize = 5;

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
    /// The commit's scratch directory, kept until the commit is landed because Git's scratch repository lives in it.
    scratch: Scratch,
    /// Whether the repository's configuration asks Git to build the commit (a template, a signature, a clean-up mode).
    needs_native: bool,
    /// What real Git needs to build the commit, or what the embedded route needs to make its tree.
    inputs: NativeInputs,
}

/// What real Git needs from the preparation to build the commit in a scratch repository.
struct NativeInputs {
    capture: InvocationCapture,
    arguments: Vec<OsString>,
    pathspecs: Vec<String>,
    mode: TransactionMode,
    staged_all: bool,
    commit_index: PathBuf,
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
    if !matches!(capture.base, Base::Commit(_)) {
        return Ok(Prepared::Decline);
    }
    // Read from the snapshot the layout already holds, so the cheap declines above all come before anything starts Git for
    //  the message or the identities, and before a service is provisioned.
    let needs_native: bool = config_needs_native(capture.layout.config_snapshot.as_ref());
    // An amend still replaces the last commit when no path changed (a new message, a new committer).
    let amends: bool = invocation.region.amend;
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
    // A test removes the staged content here, between the writing and the reading by the policies.
    reach_service_phase(Phase::ServiceStaged)?;
    if !list_unmerged_index_paths(context, cwd, indexes.commit_index.as_path())?.is_empty() {
        return Ok(Prepared::Decline);
    }
    let candidate_paths: Vec<Vec<u8>> =
        list_changed_index_paths(context, cwd, indexes.commit_index.as_path(), base.as_str(), indexes.object_directory.as_path())?;
    if candidate_paths.is_empty() && !allow_empty && !amends {
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
    if changes.is_empty() && !allow_empty && !amends {
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
        needs_native,
        inputs: NativeInputs {
            arguments: outcome.pass.arguments.clone(),
            pathspecs: pathspecs.iter().map(|token| { return token.to_string_lossy().into_owned(); }).collect(),
            mode: selection.mode,
            staged_all: selection.staged_all,
            commit_index: indexes.commit_index.clone(),
            capture,
        },
        scratch,
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
    let config_home: Option<PathBuf> = path_variable("XDG_CONFIG_HOME");
    let home: Option<PathBuf> = path_variable("HOME");
    let bus_address: Option<String> = std::env::var("DBUS_SESSION_BUS_ADDRESS").ok();
    let route: Route = choose_route(&RouteInputs {
        runtime_dir: runtime.as_deref(),
        bus_address: bus_address.as_deref(),
        config_home: config_home.as_deref(),
        home: home.as_deref(),
        own_executable: remembered_own_executable(),
        real_git: invocation.context.real_git.as_path(),
    });
    let Route::Service { address, provision: provisioning } = route else { return Ok(None) };
    let region = invocation.region;
    if region.include || region.interactive || region.patch || region.dry_run || region.pathspec_from_file.is_some() {
        return Ok(None);
    }
    let after: &[OsString] = invocation.arguments.get(invocation.subcommand_index + 1..).unwrap_or(&[]);
    let id: String = random_uuid().map_err(|error| { return failed(format!("an identity for this commit could not be made: {error:?}")); })?;
    let allow_empty: bool = region.allow_empty;
    let mut provisioned: bool = false;
    // Whether an editor may have run, which decides what a moved branch costs the person: a message to keep or a signature to redo.
    let editor_possible: bool = opens_editor(after);
    let config: ClientConfig = ClientConfig { address, activation: Activation::Managed, start_wait: START_WAIT, commit_wait: COMMIT_WAIT };
    let scratch_name: String = format!("cli-git-service-{id}");
    // The service builds the commits it can from the command line; real Git builds the rest in a scratch repository. Which
    //  it is gets decided once the preparation has passed every cheap decline (head, configuration, index, policies), so a
    //  commit the service never takes starts no Git for a message or an identity.
    let mut embedded: Option<Embedded> = None;
    let mut decided: bool = false;
    // How often the content was found removed; the commit is prepared again for the first, and the second is said plainly.
    let mut removals: usize = 0;
    for attempt in 0..REBUILD_ATTEMPTS {
        let ready: Box<Ready> = match prepare(invocation, checks, request, scratch_name.as_str(), allow_empty)? {
            Prepared::Decline => return Ok(None),
            Prepared::Blocked(result) => {
                // The policies could not read an object this commit had just written: something removed it meanwhile.
                if content_was_removed(result.events.as_slice()) {
                    removals += 1;
                    if removals > PRUNE_RETRIES {
                        return Err(TransactionStop::Message(render(&ClientFailure::ContentPruned)));
                    }
                    continue;
                }
                return Ok(Some(ServiceCommit { result, handoff: None }));
            }
            Prepared::Ready(ready) => ready,
        };
        if !decided {
            decided = true;
            if !ready.needs_native
                && let Some(parsed) = parse_service_commit(after)
            {
                embedded = embedded_commit(invocation, parsed)?;
            }
        }
        let native: bool = embedded.is_none();
        let inputs: &NativeInputs = &ready.inputs;
        // An amend the service builds is made from the commit `HEAD` named when the message was read; if the branch left it
        //  since, the amend is not made. Its tree is the private index's, and native Git refuses an amend that leaves the
        //  commit empty against its first parent (a merge excepted) unless `--allow-empty` says it is meant.
        let mut tree: Option<String> = None;
        if let Some(source) = embedded.as_ref().and_then(|made| { return made.source.as_ref(); }) {
            if source.base != ready.base {
                let branch: String = ready.reference.trim_start_matches("refs/heads/").to_string();
                return Err(TransactionStop::Message(render(&ClientFailure::BranchMoved { branch })));
            }
            let written = run_git_checked(
                invocation.context,
                &indexed_request(
                    invocation.cwd,
                    &["write-tree"],
                    Some(inputs.commit_index.as_path()),
                    Some(inputs.capture.layout.object_directory.as_path()),
                ),
            )?;
            let made: String = String::from_utf8_lossy(written.stdout.as_slice()).trim().to_string();
            let against: &str = source.parent_tree.as_deref().unwrap_or(inputs.capture.layout.empty_tree_oid.as_str());
            if source.parents.len() < 2 && !allow_empty && made == against {
                return Ok(None);
            }
            tree = Some(made);
        }
        let mut typed: Option<String> = None;
        let (sent, first_parent): (CommitRequest, String) = match embedded.as_ref() {
            None => {
                let built: GitBuilt = build_with_git(&BuildInput {
                    context: invocation.context,
                    cwd: invocation.cwd,
                    capture: &inputs.capture,
                    scratch: ready.scratch.0.as_path(),
                    commit_index: inputs.commit_index.as_path(),
                    arguments: inputs.arguments.as_slice(),
                    pathspecs: inputs.pathspecs.as_slice(),
                    mode: inputs.mode,
                    staged_all: inputs.staged_all,
                    quiet: attempt > 0,
                })?;
                let parent: String = built.parents.first().cloned().unwrap_or_default();
                typed = Some(built.message.clone());
                let sent: CommitRequest = CommitRequest {
                    id: id.clone(),
                    repository: ready.repository.clone(),
                    common_dir: ready.common_dir.clone(),
                    reference: ready.reference.clone(),
                    base: ready.base.clone(),
                    changes: ready.changes.clone(),
                    // The commit is Git's whole: message, identities and tree are in it.
                    message: String::new(),
                    author: None,
                    committer: String::new(),
                    amend: invocation.region.amend,
                    tree: None,
                    commit: Some(built.commit),
                };
                (sent, parent)
            }
            Some(made) => {
                let sent: CommitRequest = CommitRequest {
                    id: id.clone(),
                    repository: ready.repository.clone(),
                    common_dir: ready.common_dir.clone(),
                    reference: ready.reference.clone(),
                    base: ready.base.clone(),
                    changes: ready.changes.clone(),
                    message: made.message.clone(),
                    author: Some(made.author.clone()),
                    committer: made.committer.clone(),
                    amend: tree.is_some(),
                    tree: tree.clone(),
                    commit: None,
                };
                // The new commit's first parent: the tip it was made on, or for an amend the replaced commit's own parent.
                let parent: String = match made.source.as_ref() {
                    Some(replaced) => replaced.parents.first().cloned().unwrap_or_default(),
                    None => ready.base.clone(),
                };
                (sent, parent)
            }
        };
        // A test removes the prepared content here, between the preparation and the landing.
        reach_service_phase(Phase::ServicePrepared)?;
        // The first commit that reaches the service sets up its socket activation, and only then: a commit that was
        //  declined, blocked by a policy, or abandoned in the editor never needs it, and the manager starts the service.
        if let Some(host) = provisioning.as_ref().filter(|_| { return !provisioned; }) {
            provision(host).map_err(|failure| { return TransactionStop::Message(render(&failure)); })?;
            provisioned = true;
        }
        match submit(&config, &sent, &mut std::io::stderr()) {
            Ok(Submitted::Landed { commit, .. }) => {
                if let Some(made) = embedded.as_ref().filter(|made| { return !native && !made.quiet; }) {
                    let subject: &str = made.message.lines().next().unwrap_or("");
                    let short: &str = commit.get(..7).unwrap_or(commit.as_str());
                    let branch: &str = ready.reference.trim_start_matches("refs/heads/");
                    let _printed = writeln!(std::io::stdout(), "[{branch} {short}] {subject}");
                }
                if native && attempt > 0 {
                    // Git printed no summary for the build that landed, and the first one named a commit that did not land.
                    let subject: &str = typed.as_deref().and_then(|text| { return text.lines().next(); }).unwrap_or("");
                    let short: &str = commit.get(..7).unwrap_or(commit.as_str());
                    let branch: &str = ready.reference.trim_start_matches("refs/heads/");
                    let _printed = writeln!(std::io::stdout(), "[{branch} {short}] {subject}");
                }
                let handoff: ServiceHandoff = ServiceHandoff {
                    config: config.clone(),
                    repository: ready.repository.clone(),
                    common_dir: ready.common_dir.clone(),
                    reference: ready.reference.clone(),
                    parent: first_parent,
                };
                let result: TransactionResult = TransactionResult { events: ready.events, exit_code: 0, landed_oid: Some(commit), uses_lfs: ready.uses_lfs };
                return Ok(Some(ServiceCommit { result, handoff: Some(handoff) }));
            }
            Ok(Submitted::PrepareAgain { .. }) => {
                if native || sent.amend {
                    // An amend, and a commit Git built, are never sent for another tip; the service answers a moved branch as a failure.
                    let branch: String = ready.reference.trim_start_matches("refs/heads/").to_string();
                    return Err(TransactionStop::Message(render(&ClientFailure::BranchMoved { branch })));
                }
                if attempt + 1 == LANDING_ATTEMPTS {
                    let branch: String = ready.reference.trim_start_matches("refs/heads/").to_string();
                    return Err(TransactionStop::Message(render(&ClientFailure::BranchKeepsChanging { branch, times: LANDING_ATTEMPTS })));
                }
            }
            Ok(Submitted::Moved) => {
                let branch: String = ready.reference.trim_start_matches("refs/heads/").to_string();
                if native && !sent.amend && !editor_possible {
                    // Nobody typed anything: Git builds it again on the new tip (and signs it again), up to the bound.
                    if attempt + 1 == REBUILD_ATTEMPTS {
                        return Err(TransactionStop::Message(render(&ClientFailure::BranchKeepsChanging { branch, times: REBUILD_ATTEMPTS })));
                    }
                    continue;
                }
                if native && editor_possible && let Some(text) = typed.as_deref() {
                    // The person typed this message: keep it where they can reuse it.
                    let file: PathBuf = inputs.capture.layout.git_dir.join(format!("cli-git-saved-message-{}", id.get(..8).unwrap_or(id.as_str())));
                    match std::fs::write(&file, text) {
                        Ok(()) => {
                            let failure: ClientFailure = ClientFailure::BranchMovedMessageSaved { branch, file: file.display().to_string(), amend: sent.amend };
                            return Err(TransactionStop::Message(render(&failure)));
                        }
                        Err(error) => warn("serviceCommit", format!("saving the typed message failed: {error}").as_str()),
                    }
                    let again: String = ready.reference.trim_start_matches("refs/heads/").to_string();
                    return Err(TransactionStop::Message(render(&ClientFailure::BranchMoved { branch: again })));
                }
                return Err(TransactionStop::Message(render(&ClientFailure::BranchMoved { branch })));
            }
            Err(ClientFailure::Unavailable)
                if prepared_content_missing(
                    inputs.capture.layout.common_dir.as_path(),
                    inputs.capture.layout.object_directory.as_path(),
                    inputs.capture.layout.object_format.as_str(),
                    &sent,
                ) =>
            {
                // The service could not land the request because an object it names is gone from the repository.
                removals += 1;
                if removals > PRUNE_RETRIES {
                    return Err(TransactionStop::Message(render(&ClientFailure::ContentPruned)));
                }
            }
            Err(failure) => return Err(TransactionStop::Message(render(&failure))),
        }
    }
    return Ok(None);
}

/// What the embedded route made of a command line: the message, the identities and, for an amend, the replaced commit.
struct Embedded {
    message: String,
    author: String,
    committer: String,
    source: Option<AmendSource>,
    quiet: bool,
}

/// What:
///  Builds the message and the identities of a command line the service covers, the way native Git would.
/// Why:
///   Returns nothing when Git alone can say what the line means (an unreadable message file, a date it does not read, no
///  identity configured, an empty message), and Git then builds the commit and gives the person its own words.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await embeddedCommit(invocation, parsed) // undefined when Git must build it
/// ```
fn embedded_commit(invocation: &CommitInvocation<'_>, parsed: ServiceCommitArgs) -> Result<Option<Embedded>, TransactionStop> {
    let Some(committer) = identity(invocation.context, invocation.cwd, "GIT_COMMITTER_IDENT", &[])? else { return Ok(None) };
    let source: Option<AmendSource> = if parsed.amend {
        let Some(read) = read_amend_source(invocation.context, invocation.cwd)? else { return Ok(None) };
        Some(read)
    } else {
        None
    };
    let previous: Option<&str> = source.as_ref().map(|found| { return found.message.as_str(); });
    let Some(built) = build_message(invocation, &parsed, committer.as_str(), previous)? else { return Ok(None) };
    let author: String = match built.reused_author {
        Some(reused) => reused,
        None => match source.as_ref() {
            Some(replaced) => {
                let Some(made) = amended_author(invocation.context, invocation.cwd, replaced, &parsed)? else { return Ok(None) };
                made
            }
            None => {
                let Some(made) = identity(invocation.context, invocation.cwd, "GIT_AUTHOR_IDENT", author_overrides(&parsed).as_slice())? else {
                    return Ok(None);
                };
                made
            }
        },
    };
    return Ok(Some(Embedded { message: built.message, author, committer, source, quiet: parsed.quiet }));
}

/// Service commit controls stay out of the release executable.
#[cfg(test)]
#[path = "service_commit_tests.rs"]
mod tests;
