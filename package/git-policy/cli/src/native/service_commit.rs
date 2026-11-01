//! What:
//!  The commit command's path through the resident service: prepare the commit in this process (private index,
//!  policies), send the prepared change to the service, and report what it landed.
//! Why:
//!  The service is the only writer, so a commit takes no lock and polls nothing: preparation and policies run here,
//!  outside the service's serialized step, and the service moves the branch once for a whole batch of commits
//!  (`doc/planning/cli-git-resident-service.md`, "Where preparation and policies run").
//!  Every command line lands through it. The lines `service_commit_args.rs` accepts are built here from the command line
//!  (message and identity options, sign-off, trailers, a reused or fixup message, `-a`, an index commit, `--amend`); every
//!  other line, and every commit that is not on an ordinary branch with an ordinary state, is built by real Git in a scratch
//!  repository and landed as Git made it. Which route a commit takes is decided by `service_route.rs`; what is prepared is
//!  `service_prepare.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const result = await tryServiceCommit(invocation, checks, request); // undefined only when no service can be reached
//! ```

use super::capture_order::indexed_request;
use super::commit_transaction::{CommitInvocation, TransactionResult, TransactionStop, failed};
use super::diagnostic_log::warn;
use super::policy_checks::ShippedChecks;
use super::policy_engine::StageRequest;
use super::random_id::random_uuid;
use super::repository_facts::RepositoryFacts;
use super::service_amend::{AmendSource, amended_author, read_amend_source};
use super::service_client::{Activation, COMMIT_WAIT, ClientConfig, START_WAIT, Submitted, submit};
use super::service_commit_args::{ServiceCommitArgs, opens_editor, parse_service_commit, with_fixed_message};
use super::service_handoff::ServiceHandoff;
use super::service_land::LANDING_ATTEMPTS;
use super::service_message::{author_overrides, build_message, identity};
use super::service_messages::{ClientFailure, render};
use super::service_native::{BuildInput, GitBuilt, build_with_git};
use super::service_prepare::{NativeInputs, PrepareInputs, Prepared, Ready, prepare};
use super::service_replay::{Captured, Retry, conflict_result, retry_of};
use super::service_protocol::CommitRequest;
use super::service_provision::provision;
use super::service_pruned::{content_was_removed, prepared_content_missing, reach_service_phase};
use super::transaction_phase::Phase;
use super::service_route::{Route, RouteInputs, choose_route, remembered_own_executable};
use super::transaction_git::run_git_checked;
use super::worktree_completion::install_added_worktree_files;
use std::ffi::OsString;
use std::io::{Read, Write};
use std::path::PathBuf;

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

/// The plain failure of a branch that moved under a commit that cannot be made again on another tip.
fn branch_moved(ready: &Ready) -> TransactionStop {
    let branch: String = ready.reference.trim_start_matches("refs/heads/").to_string();
    return TransactionStop::Message(render(&ClientFailure::BranchMoved { branch }));
}

/// The standard input of the command when a pathspec file is read from it, read once.
fn standard_input_for(invocation: &CommitInvocation<'_>) -> Result<Option<Vec<u8>>, TransactionStop> {
    let after: &[OsString] = invocation.arguments.get(invocation.subcommand_index + 1..).unwrap_or(&[]);
    let Some(value) = invocation.region.pathspec_from_file else { return Ok(None) };
    if super::command_options_query::value_bytes(after, value) != b"-" {
        return Ok(None);
    }
    let mut bytes: Vec<u8> = Vec::new();
    std::io::stdin().read_to_end(&mut bytes).map_err(|error| { return failed(format!("reading the pathspec file failed: {error}")); })?;
    return Ok(Some(bytes));
}

/// What:
///  Lands the commit through the service when the platform can reach one, for every command line.
/// Why:
///   Returns `None` only when no service can be reached on this machine, before anything was changed, so the earlier
///  landing runs as it did; every other ending is the commit's.
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
    let after: &[OsString] = invocation.arguments.get(invocation.subcommand_index + 1..).unwrap_or(&[]);
    let id: String = random_uuid().map_err(|error| { return failed(format!("an identity for this commit could not be made: {error:?}")); })?;
    let standard_input: Option<Vec<u8>> = standard_input_for(invocation)?;
    let mut provisioned: bool = false;
    // Whether an editor may have run, which decides what a moved branch costs the person: a message to keep or a signature to redo.
    let editor_possible: bool = opens_editor(after);
    let config: ClientConfig = ClientConfig { address, activation: Activation::Managed, start_wait: START_WAIT, commit_wait: COMMIT_WAIT };
    let scratch_name: String = format!("cli-git-service-{id}");
    // The service builds the commits it can from the command line; real Git builds the rest in a scratch repository. Which
    //  it is gets decided once the preparation has passed every cheap decision (head, configuration, index, policies), so a
    //  commit the service never takes starts no Git for a message or an identity.
    let mut embedded: Option<Embedded> = None;
    let mut decided: bool = false;
    // The message the person typed once the editor has run, kept in a file so a commit moved by a tip that changed is built again from it.
    let mut fixed_message: Option<PathBuf> = None;
    // The message of the command as the last preparation settled it, kept for the person when the commit cannot be made.
    let mut last_message: Option<String> = None;
    // What the previous preparation captured when the service sent the commit back because the branch moved: the next
    //  preparation merges it onto the branch as it is and never reads the worktree again.
    let mut retry: Option<Captured> = None;
    for attempt in 0..REBUILD_ATTEMPTS {
        let inputs: PrepareInputs<'_> = PrepareInputs {
            scratch_name: scratch_name.as_str(),
            allow_empty: invocation.region.allow_empty,
            standard_input: standard_input.as_deref(),
            first: attempt == 0,
            replay: retry.as_ref().map(|captured| { return Retry { captured, attempt: u64::try_from(attempt).unwrap_or(u64::MAX) }; }),
        };
        let ready: Box<Ready> = match prepare(invocation, checks, request, &inputs)? {
            Prepared::Conflict(conflict) => {
                // The message is kept so the person can commit again once their files agree with the commit that landed first.
                let saved: PathBuf = conflict.git_dir.join(format!("cli-git-saved-message-{}", id.get(..8).unwrap_or(id.as_str())));
                let kept: Option<&PathBuf> = last_message.as_ref().and_then(|text| {
                    return match std::fs::write(&saved, text.as_str()) {
                        Ok(()) => Some(&saved),
                        Err(error) => {
                            warn("serviceCommit", format!("saving the message failed: {error}").as_str());
                            None
                        }
                    };
                });
                let result: TransactionResult = conflict_result(&conflict, kept.map(PathBuf::as_path));
                return Ok(Some(ServiceCommit { result, handoff: None }));
            }
            Prepared::Ended(result) => {
                // The policies could not read an object this commit had just written: something removed it meanwhile.
                //  The captured bytes are gone, and the worktree is not read again to replace them.
                if content_was_removed(result.events.as_slice()) {
                    return Err(TransactionStop::Message(render(&ClientFailure::ContentPruned)));
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
        let native_inputs: &NativeInputs = &ready.inputs;
        // An amend the service builds is made from the commit `HEAD` named when the message was read; if the branch left it
        //  since, the amend is not made. Its tree is the private index's, and native Git refuses an amend that leaves the
        //  commit empty against its first parent (a merge excepted) unless `--allow-empty` says it is meant, which Git then
        //  says itself.
        let mut tree: Option<String> = None;
        if !ready.needs_native
            && let Some(source) = embedded.as_ref().and_then(|made| { return made.source.as_ref(); })
        {
            if source.base != ready.base {
                return Err(branch_moved(&ready));
            }
            let written = run_git_checked(
                invocation.context,
                &indexed_request(
                    invocation.cwd,
                    &["write-tree"],
                    Some(native_inputs.commit_index.as_path()),
                    Some(native_inputs.capture.layout.object_directory.as_path()),
                ),
            )?;
            let made: String = String::from_utf8_lossy(written.stdout.as_slice()).trim().to_string();
            let against: &str = source.parent_tree.as_deref().unwrap_or(native_inputs.capture.layout.empty_tree_oid.as_str());
            if source.parents.len() < 2 && !invocation.region.allow_empty && made == against {
                embedded = None;
            } else {
                tree = Some(made);
            }
        }
        let native: bool = ready.needs_native || embedded.is_none();
        let mut typed: Option<String> = None;
        let (sent, first_parent): (CommitRequest, String) = match embedded.as_ref().filter(|_| { return !native; }) {
            None => {
                let layout_prefix: usize = super::global_arguments::global_layout(native_inputs.arguments.as_slice()).prefix_len;
                let rebuilt_arguments: Option<Vec<OsString>> = fixed_message.as_deref().and_then(|file| { return with_fixed_message(native_inputs.arguments.as_slice(), layout_prefix, file); });
                let built: GitBuilt = build_with_git(&BuildInput {
                    context: invocation.context,
                    cwd: invocation.cwd,
                    capture: &native_inputs.capture,
                    scratch: ready.scratch.0.as_path(),
                    commit_index: native_inputs.commit_index.as_path(),
                    arguments: rebuilt_arguments.as_deref().unwrap_or(native_inputs.arguments.as_slice()),
                    pathspecs: native_inputs.pathspecs.as_slice(),
                    mode: native_inputs.mode,
                    staged_all: native_inputs.staged_all,
                    selected_privately: native_inputs.selected_privately,
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
                    // The service names the index entries from the commit itself, so no list of changes travels.
                    changes: Vec::new(),
                    // The commit is Git's whole: message, identities and tree are in it.
                    message: String::new(),
                    author: None,
                    committer: String::new(),
                    amend: invocation.region.amend,
                    tree: None,
                    commit: Some(built.commit),
                    cleanup: ready.cleanup.clone(),
                    index: native_inputs.capture.layout.real_index_path.clone(),
                    captured: ready.captured_at,
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
                    changes: ready.changes.clone().unwrap_or_default(),
                    message: made.message.clone(),
                    author: Some(made.author.clone()),
                    committer: made.committer.clone(),
                    amend: tree.is_some(),
                    tree: tree.clone(),
                    commit: None,
                    cleanup: None,
                    index: native_inputs.capture.layout.real_index_path.clone(),
                    captured: ready.captured_at,
                };
                // The new commit's first parent: the tip it was made on, or for an amend the replaced commit's own parent.
                let parent: String = match made.source.as_ref() {
                    Some(replaced) => replaced.parents.first().cloned().unwrap_or_default(),
                    None => ready.base.clone(),
                };
                (sent, parent)
            }
        };
        last_message = typed.clone().or_else(|| { return embedded.as_ref().filter(|_| { return !native; }).map(|made| { return made.message.clone(); }); });
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
                if let Some(file) = fixed_message.as_ref() {
                    // The commit was made from the saved message; the file has done its work.
                    let _removed = std::fs::remove_file(file);
                }
                if let Some(made) = embedded.as_ref().filter(|made| { return !native && !made.quiet; }) {
                    let subject: &str = made.message.lines().next().unwrap_or("");
                    let short: &str = commit.get(..7).unwrap_or(commit.as_str());
                    let branch: &str = ready.reference.trim_start_matches("refs/heads/");
                    print_summary(format!("[{branch} {short}] {subject}").as_str());
                }
                if native && attempt > 0 {
                    // Git printed no summary for the build that landed, and the first one named a commit that did not land.
                    let subject: &str = typed.as_deref().and_then(|text| { return text.lines().next(); }).unwrap_or("");
                    let short: &str = commit.get(..7).unwrap_or(commit.as_str());
                    let branch: &str = ready.reference.trim_start_matches("refs/heads/");
                    print_summary(format!("[{branch} {short}] {subject}").as_str());
                }
                // The corrections a policy made reach the worktree once the commit has landed, as the earlier landing did.
                install_added_worktree_files(
                    invocation.context,
                    invocation.cwd,
                    ready.repository.as_path(),
                    ready.worktree_records.as_slice(),
                    Some(native_inputs.capture.layout.object_directory.as_path()),
                )
                .map_err(|error| { return TransactionStop::Message(error.0); })?;
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
            Ok(Submitted::PrepareAgain { landed, .. }) => {
                if native || sent.amend {
                    // An amend, and a commit Git built, are never sent for another tip; the service answers a moved branch as a failure.
                    return Err(branch_moved(&ready));
                }
                if attempt + 1 == LANDING_ATTEMPTS {
                    let branch: String = ready.reference.trim_start_matches("refs/heads/").to_string();
                    return Err(TransactionStop::Message(render(&ClientFailure::BranchKeepsChanging { branch, times: LANDING_ATTEMPTS })));
                }
                // What this preparation captured is merged onto the branch by the next one.
                retry = Some(retry_of(invocation.context, invocation.cwd, &ready, landed)?);
            }
            Ok(Submitted::Moved) => {
                let branch: String = ready.reference.trim_start_matches("refs/heads/").to_string();
                // Nobody typed or chose anything, and the commit does not depend on the tip it was made on: Git builds it
                //  again on the new tip (and signs it again), up to the bound.
                if native && !sent.amend && !editor_possible && !ready.interactive && !ready.fixed_parents {
                    if attempt + 1 == REBUILD_ATTEMPTS {
                        return Err(TransactionStop::Message(render(&ClientFailure::BranchKeepsChanging { branch, times: REBUILD_ATTEMPTS })));
                    }
                    // Git builds it again from the tree this build captured, not from the worktree.
                    retry = Some(retry_of(invocation.context, invocation.cwd, &ready, Vec::new())?);
                    continue;
                }
                if native && editor_possible && let Some(text) = typed.as_deref() {
                    // The person typed this message: keep it where they can reuse it.
                    let file: PathBuf = native_inputs.capture.layout.git_dir.join(format!("cli-git-saved-message-{}", id.get(..8).unwrap_or(id.as_str())));
                    match std::fs::write(&file, text) {
                        Ok(()) => {
                            // After the editor the message is fixed: while the bound allows, Git builds the commit again on the new
                            //  tip from `-F <file>`, with no editor and no person, unless the line borrows its author or its parents.
                            let prefix: usize = super::global_arguments::global_layout(native_inputs.arguments.as_slice()).prefix_len;
                            let repeatable: bool = !sent.amend
                                && !ready.interactive
                                && !ready.fixed_parents
                                && with_fixed_message(native_inputs.arguments.as_slice(), prefix, file.as_path()).is_some();
                            if repeatable && attempt + 1 < REBUILD_ATTEMPTS {
                                fixed_message = Some(file);
                                retry = Some(retry_of(invocation.context, invocation.cwd, &ready, Vec::new())?);
                                continue;
                            }
                            let failure: ClientFailure = ClientFailure::BranchMovedMessageSaved { branch, file: file.display().to_string(), amend: sent.amend };
                            return Err(TransactionStop::Message(render(&failure)));
                        }
                        Err(error) => warn("serviceCommit", format!("saving the typed message failed: {error}").as_str()),
                    }
                }
                return Err(branch_moved(&ready));
            }
            Err(ClientFailure::Unavailable)
                if prepared_content_missing(
                    native_inputs.capture.layout.common_dir.as_path(),
                    native_inputs.capture.layout.object_directory.as_path(),
                    native_inputs.capture.layout.object_format.as_str(),
                    &sent,
                ) =>
            {
                // The service could not land the request because an object it names is gone from the repository, and the
                //  worktree is not read again to make it.
                return Err(TransactionStop::Message(render(&ClientFailure::ContentPruned)));
            }
            Err(failure) => return Err(TransactionStop::Message(render(&failure))),
        }
    }
    return Err(TransactionStop::Message(render(&ClientFailure::BranchKeepsChanging { branch: String::from("the branch"), times: REBUILD_ATTEMPTS })));
}

/// What:
///  Writes the summary line of a landed commit and ends the command as Git does when it cannot be written.
/// Why:
///   Git prints its summary after the branch moved and dies with `SIGPIPE` when the reader is gone and with a write error when
///  the device is full or closed, so a command whose output is broken ends the same way whichever program built the commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// printSummary(line)
/// ```
fn print_summary(line: &str) {
    let mut out = std::io::stdout();
    let written = writeln!(out, "{line}").and_then(|()| { return out.flush(); });
    let Err(error) = written else { return };
    if error.kind() == std::io::ErrorKind::BrokenPipe {
        // SAFETY: restoring the default action of a signal and raising it takes no pointers.
        unsafe {
            libc::signal(libc::SIGPIPE, libc::SIG_DFL);
            libc::raise(libc::SIGPIPE);
        }
    }
    let _reported = writeln!(std::io::stderr(), "fatal: write error: {error}");
    std::process::exit(128);
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
