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

use crate::commit_timing::CommitTiming;
/// The marker lines a committed file still holds, and the line about an operation left open.
use crate::conflict_markers::{open_operation_line, paths_with_markers};
/// The kind of operation a commit concludes.
use crate::transaction_journal::Conclusion;
use crate::private_index_request::indexed_request;
use crate::commit_transaction::{CommitInvocation, TransactionResult, TransactionStop, failed};
use crate::diagnostic_log::warn;
use crate::policy_checks::ShippedChecks;
use crate::policy_engine::StageRequest;
use crate::random_id::random_uuid;
use crate::repository_facts::RepositoryFacts;
use crate::service_amend::{AmendSource, amended_author, read_amend_source};
use crate::scratch_location::SCRATCH_REFRESH_EVERY;
use crate::service_client::{Activation, ClientConfig, FreshScratch, START_WAIT, ScratchWatch, Submitted, manager_watch_for, submit_traced, watch_scratch};
use crate::landed_summary::{rebuilt_note, short_name, subject_of, summary_line};
use crate::service_commit_args::{ServiceCommitArgs, opens_editor, parse_service_commit, with_fixed_message};
use crate::service_handoff::ServiceHandoff;
use crate::service_land::LANDING_ATTEMPTS;
use crate::service_message::{author_overrides, build_message, identity};
use crate::service_messages::{ClientFailure, render};
use crate::service_native::{BuildInput, GitBuilt, build_with_git};
use crate::service_prepare::{NativeInputs, PrepareInputs, Prepared, Ready, prepare};
use crate::service_replay::{Captured, Retry, conflict_result, retry_of};
use crate::service_protocol::{CommitRequest, HEARTBEAT_SILENCE};
use crate::service_provision::{platform_without_manager, provision};
use crate::service_pruned::{content_was_removed, prepared_content_missing, reach_service_phase};
use crate::transaction_phase::Phase;
use crate::service_route::{Route, RouteInputs, choose_route, remembered_own_executable, worktree_git_dir};
use crate::transaction_git::{GitRequest, run_git, run_git_checked};
use crate::worktree_completion::install_added_worktree_files;
use std::ffi::OsString;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::time::Instant;

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

/// What:
///  The words of the command line after the subcommand:
/// the options and the paths of the command itself.
/// Why:
///   The subcommand's own name is not one of them,
/// and a command line that ends at the subcommand has none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// arguments.slice(subcommandIndex + 1)
/// ```
fn arguments_after_subcommand(arguments: &[OsString], subcommand_index: usize) -> &[OsString] {
    return arguments.get(subcommand_index + 1..).unwrap_or(&[]);
}

/// What:
///  How a branch is named to the person when the worktree switched under a commit:
/// a detached head by those words,
/// any other
///  reference by its branch name.
/// Why:
///   `HEAD` is not a branch name,
/// and the full reference of a branch is not what the person typed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// reference === 'HEAD' ? 'a detached head' : reference.replace(/^refs\/heads\//, '')
/// ```
fn switched_branch_name(reference: &str) -> String {
    return if reference == "HEAD" { String::from("a detached head") } else { reference.trim_start_matches("refs/heads/").to_string() };
}

/// The standard input of the command when a pathspec file is read from it, read once.
fn standard_input_for(invocation: &CommitInvocation<'_>) -> Result<Option<Vec<u8>>, TransactionStop> {
    let after: &[OsString] = arguments_after_subcommand(invocation.arguments, invocation.subcommand_index);
    let Some(value) = invocation.region.pathspec_from_file else { return Ok(None) };
    if crate::command_options_query::value_bytes(after, value) != b"-" {
        return Ok(None);
    }
    let mut bytes: Vec<u8> = Vec::new();
    std::io::stdin().read_to_end(&mut bytes).map_err(|error| { return failed(format!("reading the pathspec file failed: {error}")); })?;
    return Ok(Some(bytes));
}

/// What:
///  What a commit run outside any repository says: Git's own words and status for the same question.
/// Why:
///   No worktree means no service instance to send the commit to. Asking Git which repository the directory belongs to gives
///  the person the text and the exit status native `git commit` gives there, and that question can make no commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// repositoryNotFound(invocation) // Git's stderr, then its exit status
/// ```
fn repository_not_found(invocation: &CommitInvocation<'_>) -> TransactionStop {
    let asked = run_git(invocation.context, &GitRequest::new(invocation.cwd, &["rev-parse", "--git-dir"]));
    let output = match asked {
        Ok(found) => found,
        Err(error) => return failed(format!("asking Git which repository this directory belongs to failed: {error}")),
    };
    if output.succeeded() {
        return failed(String::from("this directory belongs to a repository whose Git directory could not be located"));
    }
    let _reported = std::io::stderr().write_all(output.stderr.as_slice());
    return TransactionStop::NativeExit(output.code.unwrap_or(1));
}

/// What:
///  Lands the commit through the service, for every command line.
/// Why:
///   The service is the only writer of a commit. A machine where none can be reached or started ends the command with the
///  plain message of its platform, before anything was changed; every other ending is the commit's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await serviceCommit(invocation, checks, request, timing)
/// ```
pub fn service_commit<F: RepositoryFacts>(
    invocation: &CommitInvocation<'_>,
    checks: &mut ShippedChecks<F>,
    request: &StageRequest,
    timing: &mut CommitTiming,
) -> Result<ServiceCommit, TransactionStop> {
    let path_variable = |name: &str| -> Option<PathBuf> { return std::env::var_os(name).filter(|value| { return !value.is_empty(); }).map(PathBuf::from); };
    let runtime: Option<PathBuf> = path_variable("XDG_RUNTIME_DIR");
    let config_home: Option<PathBuf> = path_variable("XDG_CONFIG_HOME");
    let home: Option<PathBuf> = path_variable("HOME");
    let bus_address: Option<String> = std::env::var("DBUS_SESSION_BUS_ADDRESS").ok();
    // The worktree's own Git directory names the service instance, so no route exists outside a repository.
    let Some(git_dir) = worktree_git_dir(invocation.context, invocation.cwd) else { return Err(repository_not_found(invocation)) };
    let route: Option<Route> = choose_route(&RouteInputs {
        runtime_dir: runtime.as_deref(),
        bus_address: bus_address.as_deref(),
        config_home: config_home.as_deref(),
        home: home.as_deref(),
        own_executable: remembered_own_executable(),
        real_git: invocation.context.real_git.as_path(),
        git_dir: git_dir.as_path(),
    });
    let Some(Route { address, provision: provisioning }) = route else {
        return Err(TransactionStop::Message(render(&ClientFailure::NoActivation(platform_without_manager()))));
    };
    // From here on the commit is recorded; one that stopped above, on a machine with no way to start a service, took no time
    //  worth splitting and leaves no record (nothing it does changes the repository).
    timing.place_at(git_dir.as_path());
    let after: &[OsString] = arguments_after_subcommand(invocation.arguments, invocation.subcommand_index);
    let id: String = random_uuid().map_err(|error| { return failed(format!("an identity for this commit could not be made: {error:?}")); })?;
    // The identity is the key that joins this commit's timing record to the service's journal entry for it.
    timing.request = Some(id.clone());
    let standard_input: Option<Vec<u8>> = standard_input_for(invocation)?;
    let mut provisioned: bool = false;
    // Whether an editor may have run, which decides what a moved branch costs the person: a message to keep or a signature to redo.
    let editor_possible: bool = opens_editor(after);
    let config: ClientConfig = ClientConfig {
        address,
        activation: Activation::Managed,
        start_wait: START_WAIT,
        heartbeat_silence: HEARTBEAT_SILENCE,
        start_watch: provisioning.as_ref().map(manager_watch_for),
        keep_fresh: None, answer_ceiling: crate::wait_ceilings::ANSWER_CEILING, withdraw_ceiling: crate::wait_ceilings::WITHDRAW_CEILING, extra_wait: crate::wait_ceilings::SHORT_EXTRA_WAIT, start_ceiling: crate::wait_ceilings::START_CEILING,
    };
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
    // The branch (or `HEAD`) the first preparation found, which every later preparation must find again.
    let mut first_reference: Option<String> = None;
    // The commit's scratch directory, handed to the service once the first preparation made it: the service removes it when this
    //  command ends, and when this process is killed, because the connection closes either way. Without a service of this
    //  release to take it, the directory is removed here as before.
    let mut scratch_watch: Option<ScratchWatch> = None;
    // The commits Git built that the branch moving made useless: Git printed a summary for each, so the wrapper names them
    //  once the commit that did land is known.
    let mut unlanded: Vec<String> = Vec::new();
    for attempt in 0..REBUILD_ATTEMPTS {
        let inputs: PrepareInputs<'_> = PrepareInputs {
            scratch_name: scratch_name.as_str(),
            allow_empty: invocation.region.allow_empty,
            standard_input: standard_input.as_deref(),
            first: attempt == 0,
            replay: retry.as_ref().map(|captured| { return Retry { captured, attempt: u64::try_from(attempt).unwrap_or(u64::MAX) }; }),
        };
        let preparing: Instant = Instant::now();
        let prepared: Result<Prepared, TransactionStop> = prepare(invocation, checks, request, &inputs);
        timing.prepare += preparing.elapsed();
        timing.attempts += 1;
        let ready: Box<Ready> = match prepared? {
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
                return Ok(ServiceCommit { result, handoff: None });
            }
            Prepared::Ended(result) => {
                // The policies could not read an object this commit had just written: something removed it meanwhile.
                //  The captured bytes are gone, and the worktree is not read again to replace them.
                if content_was_removed(result.events.as_slice()) {
                    return Err(TransactionStop::Message(render(&ClientFailure::ContentPruned)));
                }
                return Ok(ServiceCommit { result, handoff: None });
            }
            Prepared::Ready(ready) => ready,
        };
        // The branch the first preparation found. A worktree that switched since (the service refuses the landing, and a
        //  new preparation would read the new branch) is a failure like native Git's, with the typed message kept in its file.
        let building: Instant = Instant::now();
        if timing.reference.is_none() {
            timing.reference = Some(ready.reference.clone());
        }
        let first_branch: &str = first_reference.get_or_insert_with(|| { return ready.reference.clone(); });
        if first_branch != ready.reference {
            let failure: ClientFailure = ClientFailure::BranchSwitched {
                branch: switched_branch_name(first_branch),
                now: switched_branch_name(ready.reference.as_str()),
                file: fixed_message.as_ref().map(|file| { return file.display().to_string(); }),
            };
            return Err(TransactionStop::Message(render(&failure)));
        }
        if scratch_watch.is_none() {
            scratch_watch = watch_scratch(&config, ready.inputs.capture.layout.git_dir.as_path(), ready.scratch.0.as_path());
        }
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
                let layout_prefix: usize = crate::global_arguments::global_layout(native_inputs.arguments.as_slice()).prefix_len;
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
                })?;
                let parent: String = built.parents.first().cloned().unwrap_or_default();
                typed = Some(built.message.clone());
                let sent: CommitRequest = CommitRequest {
                    id: id.clone(),
                    repository: ready.repository.clone(),
                    common_dir: ready.common_dir.clone(),
                    git_dir: native_inputs.capture.layout.git_dir.clone(),
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
                    git_dir: native_inputs.capture.layout.git_dir.clone(),
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
        timing.build += building.elapsed();
        reach_service_phase(Phase::ServicePrepared)?;
        // The first commit that reaches the service sets up its socket activation, and only then: a commit that was
        //  declined, blocked by a policy, or abandoned in the editor never needs it, and the manager starts the service.
        if let Some(host) = provisioning.as_ref().filter(|_| { return !provisioned; }) {
            let provisioning_began: Instant = Instant::now();
            let provided = provision(host);
            timing.provision += provisioning_began.elapsed();
            provided.map_err(|failure| { return TransactionStop::Message(render(&failure)); })?;
            provisioned = true;
        }
        // The commit's scratch directory is renewed while the commit waits, however long: the sweep of a later command would
        //  otherwise remove it from under a wait that outlasts its age.
        let waiting_config: ClientConfig = ClientConfig {
            keep_fresh: Some(FreshScratch { path: ready.scratch.0.clone(), every: SCRATCH_REFRESH_EVERY }),
            ..config.clone()
        };
        match submit_traced(&waiting_config, &sent, &mut std::io::stderr(), &mut timing.submit) {
            Ok(Submitted::Landed { commit, .. }) => {
                let after_landing: Instant = Instant::now();
                if let Some(file) = fixed_message.as_ref() {
                    // The commit was made from the saved message; the file has done its work.
                    let _removed = std::fs::remove_file(file);
                }
                // A commit the service built itself has had no `git commit` to print its summary, so one line is printed once the
                //  branch moved, and it names the commit that landed. A commit Git built was printed by Git, as Git prints it, with
                //  the options the person typed and no option of the wrapper's.
                if let Some(made) = embedded.as_ref().filter(|_| { return !native; }).filter(|made| { return !made.quiet; }) {
                    let name: String = short_name(
                        invocation.context,
                        invocation.cwd,
                        native_inputs.capture.layout.object_directory.as_path(),
                        native_inputs.capture.layout.config_snapshot.as_ref(),
                        commit.as_str(),
                    );
                    print_summary(summary_line(ready.reference.as_str(), first_parent.as_str(), name.as_str(), subject_of(made.message.as_str()).as_str()).as_str());
                }
                // A commit Git built again printed a summary for each build; one line names the builds that did not land and the one that did.
                if native && !unlanded.is_empty() {
                    let abbreviated = |full: &str| -> String {
                        return short_name(
                            invocation.context,
                            invocation.cwd,
                            native_inputs.capture.layout.object_directory.as_path(),
                            native_inputs.capture.layout.config_snapshot.as_ref(),
                            full,
                        );
                    };
                    let lost: Vec<String> = unlanded.iter().map(|full| { return abbreviated(full.as_str()); }).collect();
                    print_note(rebuilt_note(lost.as_slice(), abbreviated(commit.as_str()).as_str()).as_str());
                }
                // The owner: "Committing a conflicted path is fine. Warn and commit anyway, even if conflict markers will be committed."
                //  The claim is made for a path whose committed blob holds marker lines, read from the blob and the marker size
                //  the attributes give; a path whose blob holds none is resolved, and nothing is said about markers for it.
                for path in paths_with_markers(invocation.context, ready.repository.as_path(), commit.as_str(), ready.committed_conflicted.as_slice()) {
                    let name: String = String::from_utf8_lossy(path.as_slice()).into_owned();
                    // Terminal output error: a warning is advisory; a failed write must not fail a commit that landed.
                    let _warned = writeln!(
                        std::io::stderr(),
                        "cli-git: warning: {name} still has conflict markers, and they were committed. To fix it, remove the \
                         markers from {name} and make a follow-up commit: git commit -m \"<message>\" -- {name}"
                    );
                }
                // A commit of named paths leaves the merge or cherry-pick open, whether or not markers were committed.
                if let Some(kind) = ready.leaves_open {
                    let _noted = write!(std::io::stderr(), "{}", open_operation_line(kind == Conclusion::Merge));
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
                    git_dir: native_inputs.capture.layout.git_dir.clone(),
                    reference: ready.reference.clone(),
                    parent: first_parent,
                };
                let result: TransactionResult = TransactionResult { events: ready.events, exit_code: 0, landed_oid: Some(commit), uses_lfs: ready.uses_lfs };
                timing.after_landing += after_landing.elapsed();
                return Ok(ServiceCommit { result, handoff: Some(handoff) });
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
                    unlanded.extend(sent.commit.clone());
                    continue;
                }
                if native && editor_possible && let Some(text) = typed.as_deref() {
                    // The person typed this message: keep it where they can reuse it.
                    let file: PathBuf = native_inputs.capture.layout.git_dir.join(format!("cli-git-saved-message-{}", id.get(..8).unwrap_or(id.as_str())));
                    match std::fs::write(&file, text) {
                        Ok(()) => {
                            // After the editor the message is fixed: while the bound allows, Git builds the commit again on the new
                            //  tip from `-F <file>`, with no editor and no person, unless the line borrows its author or its parents.
                            let prefix: usize = crate::global_arguments::global_layout(native_inputs.arguments.as_slice()).prefix_len;
                            let repeatable: bool = !sent.amend
                                && !ready.interactive
                                && !ready.fixed_parents
                                && with_fixed_message(native_inputs.arguments.as_slice(), prefix, file.as_path()).is_some();
                            if repeatable && attempt + 1 < REBUILD_ATTEMPTS {
                                fixed_message = Some(file);
                                retry = Some(retry_of(invocation.context, invocation.cwd, &ready, Vec::new())?);
                                unlanded.extend(sent.commit.clone());
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

/// What:
///  Writes one note of the wrapper's to standard error.
/// Why:
///   The note is the wrapper's own words beside Git's output on standard output; a write that fails is logged and does not fail
///  the commit that has already landed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// printNote(line)
/// ```
fn print_note(line: &str) {
    if let Err(error) = writeln!(std::io::stderr(), "{line}") {
        warn("serviceCommit", format!("writing the note about the rebuilt commit failed: {error}").as_str());
    }
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
