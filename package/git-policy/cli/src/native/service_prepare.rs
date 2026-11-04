//! What:
//!  The preparation of a commit that goes through the resident service: the private index, the selection UI, the policies,
//!  the settled changes, and the facts the landing needs, for every command line.
//! Why:
//!  The service lands what is prepared here, so the preparation covers every shape of commit: a branch, a detached head or a
//!  branch that does not exist yet, an ordinary commit or the conclusion of a merge, a cherry-pick or a revert, a selection
//!  by path, by `--include`, by `--patch`/`--interactive` or by a pathspec file, and a policy that rewrote a file. Whatever
//!  Git alone can answer (an unmerged index, nothing to commit) is marked for Git to answer in the scratch repository, so its
//!  words and its exit status are the person's. Nothing here locks anything, and nothing is registered
//!  (`doc/planning/cli-git-resident-service.md`, "Landing every command line").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const prepared = await prepare(invocation, checks, request, scratchName, attempt);
//! ```

use super::capture_invocation::{InvocationCapture, base_revision, capture_invocation_base, capture_invocation_layout};
use super::private_index_request::{CaptureIndexes, indexed_request};
use super::commit_conclusion::{events_with_changed_summary, events_with_summary};
use super::commit_convergence::{CommitConvergence, CommitConvergenceInput, converge_commit_policies};
use super::commit_facts::PrivateIndexFacts;
use super::commit_index::{
    IndexSelection, initialize_commit_index, list_changed_index_paths, list_unmerged_index_paths, prepare_interactive_selection,
    write_private_tree,
};
use super::commit_no_change::{NO_CHANGE_CODE, NO_CHANGE_CORE_ID, no_change_message};
use super::commit_staged_deletions::{STAGED_DELETION_CODE, STAGED_DELETION_CORE_ID, staged_deletion_message, staged_deletions_present_on_disk};
use super::commit_transaction::{
    CommitInvocation, Selection, TransactionResult, TransactionStop, base_object, decide_selection, failed, private_checks, settled_passes, staged_at_base,
};
use super::commit_worktree_records::selected_worktree_records;
use super::conclusion_cleanup::{CONCLUSION_STATE_FILES, SEQUENCER_DIRECTORY};
use super::config_snapshot::ConfigSnapshot;
use super::lfs_pre_push::uses_lfs;
use super::policy_checks::ShippedChecks;
use super::policy_engine::{StageRequest, pass_exit_code};
use super::policy_events::PolicyEvent;
use super::policy_pass::{PassResult, run_policy_pass};
use super::private_index_stage::PrivateStage;
use super::private_storage::{create_private_directory, remove_tree, write_private_file};
use super::repository_facts::RepositoryFacts;
use super::service_land_follow::{IndexPaths, update_index};
use super::service_pruned::reach_service_phase;
use super::transaction_phase::Phase;
use super::service_protocol::{ALLOWED_MODES, Change, Cleanup, MAX_CHANGES, is_object_name, is_plain_path};
use super::merge_second_pass::Merged as MergedFile;
use super::service_replay::{MergeSite, Merged, ReplayConflict, Retry, already_committed_result, capture_stamp, first_landed, merge_captured, word_merge_notice};
use super::external_hooks::warn_external_hooks;
use super::traditional_hook_warning::warn_traditional_hooks;
use super::transaction_git::{GitContext, run_git_checked};
use super::transaction_journal::{AddedPath, Base, Conclusion, RefFormat, SymbolicHead, TransactionMode};
use super::worktree_completion::install_added_worktree_files;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// A scratch directory for the private indexes and Git's scratch repository, removed when it goes out of scope.
pub struct Scratch(pub PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _removed = remove_tree(self.0.as_path());
    }
}

/// What a preparation came to.
pub enum Prepared {
    /// The policies stopped the commit, or the preparation itself found the commit's ending (a staged deletion that is
    ///  still on disk, a normalization that removed every change).
    Ended(TransactionResult),
    /// The change is ready to land.
    Ready(Box<Ready>),
    /// A retry could not merge the captured change onto the branch; nothing was changed.
    Conflict(Box<ReplayConflict>),
}

/// What real Git needs from the preparation to build the commit in a scratch repository.
pub struct NativeInputs {
    /// The invocation capture the scratch repository is made from.
    pub capture: InvocationCapture,
    /// The arguments after the policies' fixed transforms.
    pub arguments: Vec<OsString>,
    /// The positional pathspecs of the command line, which the private index already holds.
    pub pathspecs: Vec<String>,
    /// Whether the line commits named paths or the index.
    pub mode: TransactionMode,
    /// Whether `-a` staged tracked changes into the private index.
    pub staged_all: bool,
    /// Whether the private index already holds a selection by `--include`, `--patch`, `--interactive`.
    pub selected_privately: bool,
    /// The private index the policies settled.
    pub commit_index: PathBuf,
}

/// Everything a landing needs, from one preparation.
pub struct Ready {
    /// The worktree root.
    pub repository: PathBuf,
    /// The common Git directory.
    pub common_dir: PathBuf,
    /// The branch, or `HEAD` for a detached head.
    pub reference: String,
    /// The commit prepared against; the all-zero name for a branch that does not exist yet.
    pub base: String,
    /// The changes between the base and the private index, when the embedded route can carry them; nothing when a name is
    ///  not text, a mode is not an ordinary file, a symbolic link or an executable, or there are too many.
    pub changes: Option<Vec<Change>>,
    /// The policies' events, with the fix summary when a correction changed a file.
    pub events: Vec<PolicyEvent>,
    /// Whether the repository uses Git LFS.
    pub uses_lfs: bool,
    /// The commit's scratch directory, kept until the commit is landed because Git's scratch repository lives in it.
    pub scratch: Scratch,
    /// Whether only Git can build this commit: the configuration asks for something the service does not embed, or the
    ///  head, the branch or the state is not an ordinary one, or Git alone can say what the line means.
    pub needs_native: bool,
    /// What real Git needs to build the commit, or what the embedded route needs to make its tree.
    pub inputs: NativeInputs,
    /// The files a policy rewrote that the worktree receives once the commit has landed.
    pub worktree_records: Vec<AddedPath>,
    /// The conclusion state the landing removes, for the commit that concludes a merge, a cherry-pick or a revert.
    pub cleanup: Option<Cleanup>,
    /// Whether the person chose what to commit interactively, so that a moved branch cannot be answered by choosing again.
    pub interactive: bool,
    /// Whether a moved branch makes another build on the new tip meaningless (a conclusion, a branch that did not exist).
    pub fixed_parents: bool,
    /// When the worktree's bytes were captured (`service_replay::capture_stamp`); a retry keeps the first capture's stamp.
    pub captured_at: u64,
    /// The paths this commit commits that the unfinished merge or pick left in conflict. Their worktree bytes, conflict markers
    ///  included, are what is committed; the person is warned once the commit has landed (`conflicted_path_warning`).
    pub committed_conflicted: Vec<Vec<u8>>,
    /// The operation a commit of named paths leaves in progress: the merge or the cherry-pick whose state the commit does not
    ///  conclude. The person is told once the commit has landed.
    pub leaves_open: Option<Conclusion>,
}

/// The all-zero object name of the repository's hash length, standing for a branch that does not exist yet.
pub fn zero_object(empty_tree: &str) -> String {
    return "0".repeat(empty_tree.len());
}

/// The changes between the base and the private index: path, mode and object, deletions as no object. Nothing when the
///  embedded route cannot carry them.
pub fn changes_of(context: &GitContext, cwd: &Path, indexes: &CaptureIndexes, base: &str) -> Result<Option<Vec<Change>>, TransactionStop> {
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
        if changes.len() > MAX_CHANGES {
            return Ok(None);
        }
    }
    return Ok(Some(changes));
}

/// Whether the repository's configuration makes native `git commit` do something the service does not: sign, apply a
///  template or a clean-up mode, use another message encoding or another comment character, or change trailers.
pub fn config_needs_native(snapshot: Option<&ConfigSnapshot>) -> bool {
    const KEYS: [&str; 7] = ["commit.template", "commit.cleanup", "i18n.commitencoding", "core.commentchar", "core.commentstring", "commit.signoff", "commit.verbose"];
    let Some(found) = snapshot else { return true };
    return found.entries().iter().any(|entry| {
        let value: String = entry.value.as_deref().map(|bytes| { return String::from_utf8_lossy(bytes).to_lowercase(); }).unwrap_or_default();
        let signs: bool = entry.key == "commit.gpgsign" && !["false", "no", "off", "0"].contains(&value.as_str());
        return signs || KEYS.contains(&entry.key.as_str()) || entry.key.starts_with("trailer.");
    });
}

/// What:
///  The pathspec file the private index reads, for a source named on the command line.
/// Why:
///   A regular file is read where it is named; standard input was read once, before the first preparation, and is written
///  into each preparation's scratch directory, because a second read would see nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// pathspecFileFor(scratch, cwd, source, stdinBytes)
/// ```
fn pathspec_file_for(scratch: &Path, cwd: &Path, source: Option<&str>, standard_input: Option<&[u8]>) -> std::io::Result<Option<PathBuf>> {
    let Some(name) = source else { return Ok(None) };
    if name != "-" {
        let path: PathBuf = PathBuf::from(name);
        return Ok(Some(if path.is_absolute() { path } else { cwd.join(path) }));
    }
    let snapshot: PathBuf = scratch.join(super::commit_index::PATHSPEC_INPUT_NAME);
    write_private_file(snapshot.as_path(), standard_input.unwrap_or_default())?;
    return Ok(Some(snapshot));
}

/// What the preparation reads that does not change between two preparations of one command.
pub struct PrepareInputs<'a> {
    /// The name of the commit's scratch directory under the Git directory.
    pub scratch_name: &'a str,
    /// Whether an empty commit is asked for.
    pub allow_empty: bool,
    /// The bytes of standard input when a pathspec file is read from it.
    pub standard_input: Option<&'a [u8]>,
    /// Whether this is the first preparation of the command, which says what a configuration warns about.
    pub first: bool,
    /// The change the first preparation captured, when the service sent the commit back because the branch moved: it is
    ///  merged onto the branch as it is now, and the worktree is not read again.
    pub replay: Option<Retry<'a>>,
}

/// Prepares the commit once against the branch as it is now.
pub fn prepare<F: RepositoryFacts>(
    invocation: &CommitInvocation<'_>,
    checks: &mut ShippedChecks<F>,
    request: &StageRequest,
    inputs: &PrepareInputs<'_>,
) -> Result<Prepared, TransactionStop> {
    let context: &GitContext = invocation.context;
    let cwd: &Path = invocation.cwd;
    let region = invocation.region;
    let layout = capture_invocation_layout(context, cwd, region.amend)?;
    if inputs.first {
        warn_traditional_hooks(layout.config_snapshot.as_ref(), &mut std::io::stderr());
        warn_external_hooks(layout.config_snapshot.as_ref(), &mut std::io::stderr());
    }
    // Named paths beside another session's unfinished merge or stopped cherry-pick land as an ordinary commit, one in conflict
    //  included: the owner's rule is "Committing a conflicted path is fine. Warn and commit anyway, even if conflict markers will
    //  be committed." The paths in conflict are listed here so the warning can name those the commit takes. `-a` stages the working
    //  copy of every tracked file, an unmerged one included, so it commits their markers too and is listed the same way.
    let names_paths: bool = !region.pathspecs.is_empty() || region.pathspec_from_file.is_some();
    let unmerged_before: Vec<Vec<u8>> = if region.all || (matches!(layout.conclusion, Conclusion::Merge | Conclusion::CherryPick) && names_paths) {
        list_unmerged_index_paths(context, cwd, layout.real_index_path.as_path())?
    } else {
        Vec::new()
    };
    let selection: Selection = decide_selection(region, layout.conclusion);
    let reference: String = match &layout.symbolic_head {
        SymbolicHead::Branch(name) => name.clone(),
        SymbolicHead::Detached => String::from("HEAD"),
    };
    let detached: bool = layout.symbolic_head == SymbolicHead::Detached;
    let named: bool = selection.mode == TransactionMode::ExplicitPath;
    let capture: InvocationCapture = capture_invocation_base(context, cwd, layout)?;
    let unborn: bool = matches!(capture.base, Base::Unborn);
    // Read from the snapshot the layout already holds, so nothing starts Git for the message or the identities before the
    //  cheap decisions above are made, and before a service is provisioned.
    let config_native: bool = config_needs_native(capture.layout.config_snapshot.as_ref());
    let base: String = base_revision(&capture);
    let announced_base: String = match &capture.base {
        Base::Commit(oid) => oid.clone(),
        Base::Unborn => zero_object(capture.layout.empty_tree_oid.as_str()),
    };
    let region_arguments: &[OsString] = &invocation.arguments[invocation.subcommand_index + 1..];
    let pathspecs: Vec<OsString> = region.pathspecs.iter().map(|at| { return region_arguments[*at].clone(); }).collect();
    let retry: Option<&Retry<'_>> = inputs.replay.as_ref();
    if named && retry.is_none() {
        // Before any private staging, so Git's "nothing to commit" or its force-add hint never stands in for the reason.
        let present: Vec<Vec<u8>> = staged_deletions_present_on_disk(
            context,
            cwd,
            capture.layout.repository_root.as_path(),
            capture.layout.real_index_path.as_path(),
            base.as_str(),
            pathspecs.as_slice(),
        )?;
        if !present.is_empty() {
            return Ok(Prepared::Ended(TransactionResult {
                events: vec![PolicyEvent::CoreFinding {
                    core_id: STAGED_DELETION_CORE_ID,
                    code: STAGED_DELETION_CODE,
                    message: staged_deletion_message(present.as_slice()),
                }],
                exit_code: 1,
                landed_oid: None,
                uses_lfs: false,
            }));
        }
    }
    let amends: bool = region.amend;
    let scratch: Scratch = Scratch(capture.layout.git_dir.join(inputs.scratch_name));
    create_private_directory(scratch.0.as_path()).map_err(|error| { return failed(format!("preparing the commit failed: {error}")); })?;
    let indexes: CaptureIndexes = CaptureIndexes {
        captured_index: scratch.0.join("captured.index"),
        commit_index: scratch.0.join("commit.index"),
        object_directory: capture.layout.object_directory.clone(),
    };
    let pathspec_source: Option<String> = region.pathspec_from_file.map(|value| {
        return String::from_utf8_lossy(super::command_options_query::value_bytes(region_arguments, value)).into_owned();
    });
    let pathspec_file: Option<PathBuf> = pathspec_file_for(scratch.0.as_path(), cwd, pathspec_source.as_deref(), inputs.standard_input)
        .map_err(|error| { return failed(format!("reading the pathspec file failed: {error}")); })?;
    let index_selection: IndexSelection<'_> = IndexSelection {
        mode: selection.mode,
        pathspecs: pathspecs.as_slice(),
        pathspec_file: pathspec_file.as_deref(),
        pathspec_file_nul: region.pathspec_file_nul,
        stage_into_index: region.include,
        stage_tracked_changes: selection.staged_all,
        base_revision: base.as_str(),
    };
    // The parts the word-level second pass merged when a retry combined the change with the branch, told with the commit.
    let mut word_merge_events: Option<PolicyEvent> = None;
    if let Some(again) = retry {
        // The change this command captured when it started, on the branch as it is now; the worktree is not read.
        let site: MergeSite<'_> = MergeSite {
            context,
            cwd,
            common_dir: capture.layout.common_dir.as_path(),
            object_directory: indexes.object_directory.as_path(),
            scratch: scratch.0.as_path(),
        };
        let (tree, combined): (String, Vec<MergedFile>) = match merge_captured(&site, again.captured, base.as_str(), again.attempt)? {
            Merged::Conflict(paths) => {
                let landed = first_landed(context, cwd, again.captured.base.as_str(), base.as_str(), paths.as_slice())?;
                return Ok(Prepared::Conflict(Box::new(ReplayConflict { paths, landed, git_dir: capture.layout.git_dir.clone() })));
            }
            Merged::Tree(tree) => (tree, Vec::new()),
            Merged::Combined { tree, files } => (tree, files),
        };
        run_git_checked(
            context,
            &indexed_request(cwd, &["read-tree", tree.as_str()], Some(indexes.commit_index.as_path()), Some(indexes.object_directory.as_path())),
        )?;
        // Every part the word pass merged is told to the person with the commit and sent to the journal.
        word_merge_events = word_merge_notice(combined.as_slice());
    } else {
        initialize_commit_index(context, cwd, capture.layout.real_index_path.as_path(), &indexes, &index_selection)
            .map_err(|error| { return failed(error.0); })?;
    }
    // The bytes are captured now; a retry keeps the stamp of the capture it replays.
    let captured_at: u64 = retry.map_or_else(capture_stamp, |again| { return again.captured.captured; });
    // A test removes the staged content here, between the writing and the reading by the policies.
    reach_service_phase(Phase::ServiceStaged)?;
    if (region.interactive || region.patch) && retry.is_none() {
        prepare_interactive_selection(context, cwd, indexes.commit_index.as_path(), indexes.object_directory.as_path(), region.patch, &index_selection)?;
    }
    let interactive: bool = region.interactive || region.patch;
    let reftable: bool = capture.layout.ref_format == RefFormat::Reftable;
    // Whatever state a merge or a pick left (also a squash message with no merge in progress) is removed as native Git removes it.
    let state_present: bool = selection.concludes_sequencer || CONCLUSION_STATE_FILES.iter().chain([SEQUENCER_DIRECTORY].iter()).any(|entry| { return capture.layout.git_dir.join(entry).exists(); });
    let cleanup: Option<Cleanup> = state_present.then(|| {
        return Cleanup { git_dir: capture.layout.git_dir.clone(), scratch: scratch.0.clone(), reftable };
    });
    let selected_privately: bool = selection.read_only;
    let make_inputs = |arguments: Vec<OsString>, captured: InvocationCapture| -> NativeInputs {
        return NativeInputs {
            arguments,
            pathspecs: pathspecs.iter().map(|token| { return token.to_string_lossy().into_owned(); }).collect(),
            mode: selection.mode,
            staged_all: selection.staged_all,
            selected_privately,
            commit_index: indexes.commit_index.clone(),
            capture: captured,
        };
    };
    let fixed_parents: bool = selection.concludes_sequencer || unborn;
    let by_git = |taken: InvocationCapture, events: Vec<PolicyEvent>, changes: Option<Vec<Change>>, records: Vec<AddedPath>, arguments: Vec<OsString>, scratch_dir: Scratch| -> Box<Ready> {
        return Box::new(Ready {
            repository: taken.layout.repository_root.clone(),
            common_dir: taken.layout.common_dir.clone(),
            reference: reference.clone(),
            base: announced_base.clone(),
            changes,
            events,
            uses_lfs: taken.layout.config_snapshot.as_ref().is_none_or(uses_lfs),
            scratch: scratch_dir,
            needs_native: true,
            inputs: make_inputs(arguments, taken),
            worktree_records: records,
            cleanup: cleanup.clone(),
            interactive,
            fixed_parents,
            captured_at,
            committed_conflicted: Vec::new(),
            leaves_open: None,
        });
    };
    // Files still unmerged when the commit names no paths: Git refuses here (`builtin/commit.c:348`, `refresh_cache_or_die`, which
    //  exits with "Committing is not possible because you have unmerged files"). The owner's rule is "Warn and finish the merge"
    //  (2026-11-03), so the working copy of each unmerged file is staged in the commit's private index, exactly as `git add` stages
    //  it, and the commit that follows is the one Git would have built for a resolved merge. The files are named in the warning.
    let unmerged_files: Vec<Vec<u8>> = list_unmerged_index_paths(context, cwd, indexes.commit_index.as_path())?;
    if !unmerged_files.is_empty() {
        let mut staging: Vec<OsString> = vec![OsString::from("add"), OsString::from("--all"), OsString::from("--")];
        for path in &unmerged_files {
            let mut spec: Vec<u8> = b":(top,literal)".to_vec();
            spec.extend_from_slice(path.as_slice());
            staging.push(<OsString as std::os::unix::ffi::OsStringExt>::from_vec(spec));
        }
        run_git_checked(context, &indexed_request(cwd, staging.as_slice(), Some(indexes.commit_index.as_path()), Some(indexes.object_directory.as_path())))?;
    }
    let candidate_paths: Vec<Vec<u8>> =
        list_changed_index_paths(context, cwd, indexes.commit_index.as_path(), base.as_str(), indexes.object_directory.as_path())?;
    // Nothing to commit is Git's to say, and an amend or an empty commit that was asked for is not nothing.
    if candidate_paths.is_empty() && !inputs.allow_empty && !amends {
        if retry.is_some() {
            // Commits that landed meanwhile hold everything this one changes.
            return Ok(Prepared::Ended(already_committed_result(reference.trim_start_matches("refs/heads/"))));
        }
        return Ok(Prepared::Ready(by_git(capture, Vec::new(), None, Vec::new(), invocation.arguments.to_vec(), scratch)));
    }
    let mut private: ShippedChecks<PrivateIndexFacts<&mut F>> = private_checks(
        checks,
        indexes.commit_index.as_path(),
        indexes.object_directory.as_path(),
        staged_at_base(base_object(&capture)?, candidate_paths.as_slice()),
    );
    // A selection the person made leaves the policies read-only: they report, and the commit is exactly what was chosen.
    let (pass, changed_passes, changed_paths): (PassResult, u64, Vec<Vec<u8>>) = if selection.read_only {
        private.can_apply_patches = false;
        (run_policy_pass(request, &mut private), 0, Vec::new())
    } else {
        let outcome: CommitConvergence = converge_commit_policies(CommitConvergenceInput {
            request,
            checks: &mut private,
            stage: PrivateStage { context, cwd, index: indexes.commit_index.as_path(), object_directory: Some(indexes.object_directory.as_path()) },
        });
        let passes: u64 = settled_passes(&outcome.convergence);
        (outcome.pass, passes, outcome.changed_paths)
    };
    let code: i32 = pass_exit_code(pass.events.as_slice(), pass.end);
    if code != 0 {
        return Ok(Prepared::Ended(TransactionResult { events: pass.events, exit_code: code, landed_oid: None, uses_lfs: false }));
    }
    let repository_root: &Path = capture.layout.repository_root.as_path();
    let install = private.content.install_changes();
    let records: Vec<AddedPath> = selected_worktree_records(context, cwd, repository_root, indexes.commit_index.as_path(), install.as_slice())?;
    // Corrections that removed every change leave nothing to commit: the copies are reconciled and the person is told.
    let eligible: bool = !install.is_empty() && !region.allow_empty && !amends && !selection.concludes_sequencer;
    if eligible && !unborn {
        let intended: String = write_private_tree(context, cwd, indexes.commit_index.as_path(), indexes.object_directory.as_path())?;
        let mut tree_request = indexed_request(cwd, &["rev-parse", "--verify", format!("{base}^{{tree}}").as_str()], None, None);
        tree_request.without_prefix = true;
        let base_tree: String = String::from_utf8_lossy(run_git_checked(context, &tree_request)?.stdout.as_slice()).trim().to_string();
        if base_tree == intended {
            return normalized(invocation, &capture, base.as_str(), &candidate_paths, records.as_slice(), &(pass.events, changed_passes, changed_paths));
        }
    }
    let mut events: Vec<PolicyEvent> = events_with_changed_summary(pass.events.clone(), changed_passes, changed_paths.as_slice());
    events.extend(word_merge_events);
    let changes: Option<Vec<Change>> = if config_native || unborn || detached || state_present { None } else { changes_of(context, cwd, &indexes, base.as_str())? };
    let forced: bool = config_native || unborn || detached || state_present || changes.is_none();
    // A commit of named paths leaves the merge or cherry-pick whose state it does not conclude in progress.
    let leaves_open: Option<Conclusion> = Some(capture.layout.conclusion).filter(|kind| { return named && matches!(kind, Conclusion::Merge | Conclusion::CherryPick); });
    let mut ready: Box<Ready> = by_git(capture, events, changes, records, pass.arguments.clone(), scratch);
    ready.needs_native = forced;
    // A path in conflict that the commit takes is committed with the bytes the worktree holds, markers included.
    ready.committed_conflicted = unmerged_before.into_iter().filter(|path| { return candidate_paths.contains(path); }).collect();
    ready.committed_conflicted.extend(unmerged_files);
    ready.leaves_open = leaves_open;
    return Ok(Prepared::Ready(ready));
}

/// The ending of a commit whose corrections removed every change: the index entries and the worktree copies of the
///  corrected files are reconciled with the base, and the command says so and exits 1.
fn normalized(
    invocation: &CommitInvocation<'_>,
    capture: &InvocationCapture,
    base: &str,
    committed: &[Vec<u8>],
    records: &[AddedPath],
    settled: &(Vec<PolicyEvent>, u64, Vec<Vec<u8>>),
) -> Result<Prepared, TransactionStop> {
    let (events, passes, changed_paths) = settled;
    update_index(invocation.context, capture.layout.repository_root.as_path(), capture.layout.real_index_path.as_path(), base, IndexPaths::Named(committed))
        .map_err(|code| { return failed(format!("the index could not be reconciled with the corrected files: {}", code.word())); })?;
    let completion = install_added_worktree_files(
        invocation.context,
        invocation.cwd,
        capture.layout.repository_root.as_path(),
        records,
        Some(capture.layout.object_directory.as_path()),
    )
    .map_err(|error| { return TransactionStop::Message(error.0); })?;
    let mut summary: Vec<PolicyEvent> = events_with_summary(events.clone(), *passes, changed_paths.as_slice());
    summary.push(PolicyEvent::CoreFinding {
        core_id: NO_CHANGE_CORE_ID,
        code: NO_CHANGE_CODE,
        message: no_change_message(completion.conflicted.as_slice()),
    });
    return Ok(Prepared::Ended(TransactionResult { events: summary, exit_code: 1, landed_oid: None, uses_lfs: false }));
}
