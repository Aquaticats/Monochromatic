//! What:
//!  The writing half of an isolated merge:
//! the merge tree,
//! the merge commit,
//! and the two-way update that applies it to the checkout.
//! Why:
//!  `merge_isolation.rs` decides and loops over rounds;
//! the steps that write objects and move the checkout are a file of their own that it imports and that imports nothing from it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = await apply(input, session, plan); // commit-tree, then merge --ff-only
//! ```

/// Reading an environment value.
use crate::child_environment::environment_with_overlay;
/// Diagnostics for the log.
use crate::diagnostic_log::debug;
/// Where Git's words about the wrapper's own fast-forward go now that the run carries no `--quiet`.
use crate::inner_git_words::record_streams;
/// The failure rendering for state that cannot be read.
use crate::diagnostics::{EngineFailureCode, render_engine_failure_with_hint};
/// The word-level second pass over the conflicts of the merge tree,
/// and the attribute tree it reads.
use crate::merge_second_pass::{ContextRunner, GitReply, GitRunner, Merged, config_options, conflicts_of_merge_tree};
use crate::journald_native::CLIENT_IDENTIFIER;
use crate::merge_second_pass_tree::{AttributeProbe, announce, attribute_tree_variables, resolve_tree};
use crate::path_bytes::path_message_text;
/// The intent record.
use crate::merge_isolation_journal::{MergeIntent, remove_merge_intent, write_merge_intent};
/// Recovery of records.
use crate::merge_isolation_recovery::{
    MergeRecoveryAction, settle_failed_merge,
};
/// The refusal findings.
use crate::merge_isolation_report::{conflict_finding, not_applied_finding};
/// The command-line reading.
use crate::merge_options::{MergeShape, Signing};
/// The place recovery runs in.
use crate::merge_isolation_recovery::RecoveryPlace;
/// Marker phases for the crash controls.
use crate::transaction_phase::{Phase, reach_transaction_phase};
/// Real Git.
use crate::transaction_git::{GitContext, GitOutput};
/// The owner of the intent record,
/// and its identifier.
use crate::transaction_owner::current_transaction_owner;
/// Unique identifiers and the clock.
use crate::iso_time::now_iso;
use crate::random_id::random_uuid;
/// Waiting for any program's hold on the index lock before the checkout changes.
use crate::index_holder_wait::wait_for_index_release_here;
/// Owned operating-system text.
use crate::merge_isolation_git_reads::Session;
use crate::merge_isolation_run_types::{Bail, IsolatedMerge, IsolatedMergeInput, MergeConfig, Plan, TAG, refused};
use std::ffi::OsString;
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{Path, PathBuf};

/// What:
///  The finished merge with the lines that list the files the second pass merged by word,
/// ahead of Git's own words.
/// Why:
///   The journal entries and the lines are produced only now,
/// after the merge is in the checkout,
/// so neither says a merge
///  happened that was refused or undone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function withSecondPassReport(done: IsolatedMerge, merged: Merged[]): IsolatedMerge;
/// ```
pub fn with_second_pass_report(done: IsolatedMerge, merged: &[Merged]) -> IsolatedMerge {
    match done {
        IsolatedMerge::Finished { code: 0, stdout, stderr } if !merged.is_empty() => {
            let mut report: String = String::new();
            for line in announce("monochromatic-git-policy-cli: ", (CLIENT_IDENTIFIER, "merge"), merged) {
                report.push_str(line.as_str());
                report.push('\n');
            }
            return IsolatedMerge::Finished { code: 0, stdout, stderr: report + stderr.as_str() };
        }
        other => return other,
    }
}

/// What:
///  Compute the merge from objects and return its tree and the files the word pass merged,
/// or refuse a conflict.
/// Why:
///   `merge-tree --write-tree` exits 1 for a conflicted merge and prints the tree with the conflicted files' stages;
/// nothing
///  is written to the worktree or the index.
/// The conflicts get the word-level second pass,
/// and the merge continues only when
///  every conflicted file resolved;
/// otherwise it is refused as before,
/// naming the paths.
/// The attributes of the first side's tip apply to the merge itself,
/// and the tree the merge produced to the second pass,
/// because no worktree is read.
/// A conflict with a clean index is not computed here at all:
/// native Git writes
///  the markers for the person,
/// and the second pass then runs on the stopped merge.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function mergeTree({ session, shape, ends, places, stagedWork }): Promise<{ tree: string; merged: Merged[] }>;
/// ```
pub fn merge_tree(
    session: &Session,
    shape: &MergeShape,
    ends: (&str, &str, &str),
    places: (&Path, &Path, &[(OsString, OsString)]),
    staged_work: bool,
) -> Result<(String, Vec<Merged>), Bail> {
    let (head, target, reference): (&str, &str, &str) = ends;
    let (common_dir, root, environment): (&Path, &Path, &[(OsString, OsString)]) = places;
    let mut arguments: Vec<&str> = vec!["merge-tree", "--write-tree", "-z", "--no-messages"];
    if shape.allow_unrelated_histories {
        arguments.push("--allow-unrelated-histories");
    }
    arguments.push(head);
    arguments.push(target);
    let probing: ContextRunner<'_> = ContextRunner { context: &session.context, cwd: Path::new("."), without_prefix: false, object_directory: None, environment: Vec::new() };
    let variables: Vec<(OsString, OsString)> = environment_with_overlay(environment, session.context.overlay.as_slice());
    let probe: AttributeProbe<'_> = AttributeProbe { directory: root, global_prefix: session.context.global_prefix.as_slice(), variables: variables.as_slice(), runner: &probing };
    let runner: ContextRunner<'_> = ContextRunner {
        context: &session.context,
        cwd: Path::new("."),
        without_prefix: false,
        object_directory: None,
        environment: attribute_tree_variables(head, &probe),
    };
    let merged: GitReply = runner.run(arguments.as_slice(), None, None)?;
    let (tree, conflicts) = conflicts_of_merge_tree(merged.stdout.as_slice());
    match merged.code {
        Some(0) if !tree.is_empty() => return Ok((tree, Vec::new())),
        Some(0) => return Err(Bail::from(String::from("merge-tree printed no tree"))),
        // A conflict with a clean index is left to native Git, which writes the markers for the person to resolve; the
        //  second pass then runs on the stopped merge in the worktree.
        Some(1) if !staged_work => return Err(Bail::Forward(None)),
        Some(1) => {
            // The second pass runs from the top of the worktree with the person's configuration options only.
            let top_context: GitContext = GitContext { global_prefix: config_options(&session.context.global_prefix), ..session.context.clone() };
            let top_runner: ContextRunner<'_> = ContextRunner { context: &top_context, cwd: root, ..runner };
            if let Some(resolved) = resolve_tree(&top_runner, common_dir, tree.as_str(), conflicts.as_slice(), &probe) {
                return Ok((resolved.tree, resolved.merged));
            }
            let mut paths: Vec<String> = Vec::new();
            for conflict in &conflicts {
                paths.push(path_message_text(conflict.path.as_slice()));
            }
            return Err(refused(2, conflict_finding(reference, paths.as_slice())));
        }
        _ => return Err(Bail::from(format!("git merge-tree failed: {}", String::from_utf8_lossy(&merged.stderr).trim()))),
    }
}

/// What:
///  Create the merge commit with its parents in Git's order,
/// signing it as the options and settings ask.
/// Why:
///   `commit-tree` signs only when given `-S` and never reads `commit.gpgSign`;
/// a merge on a signed branch must not
///  be the one unsigned commit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function commitMerge({ session, shape, config, parents, tree, message }): Promise<string>;
/// ```
pub fn commit_merge(
    session: &Session,
    shape: &MergeShape,
    config: &MergeConfig,
    ends: (&str, &str, &str),
    message: Vec<u8>,
) -> Result<String, String> {
    let (head, target, tree): (&str, &str, &str) = ends;
    let signing: Option<String> = match &shape.signing {
        Signing::Never => None,
        Signing::Sign(None) => Some(String::from("-S")),
        Signing::Sign(Some(key)) => Some(format!("-S{}", String::from_utf8_lossy(key))),
        Signing::Default => if config.sign_commits { Some(String::from("-S")) } else { None },
    };
    let mut arguments: Vec<&str> = vec!["commit-tree", tree, "-p", head, "-p", target];
    if let Some(flag) = signing.as_deref() {
        arguments.push(flag);
    }
    let created: GitOutput = session.run_with_input(arguments.as_slice(), message)?;
    if !created.succeeded() {
        return Err(created.error_text());
    }
    return Ok(String::from_utf8_lossy(&created.stdout).trim().to_owned());
}

/// What:
///  Record the intent,
/// let Git move the checkout to the merge commit,
/// and report.
/// Why:
///   Git's fast-forward is the two-way update that keeps every other staged entry and unstaged change,
/// refuses an
///  untracked file in the way,
/// and moves the branch only after the checkout is updated.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function apply({ session, shape, locations, commits }): Promise<IsolatedMerge>;
/// ```
pub fn apply(input: &IsolatedMergeInput<'_>, session: &Session, plan: &Plan<'_>) -> Result<IsolatedMerge, Bail> {
    let (git_dir, root): (&Path, &Path) = (plan.git_dir, plan.root);
    let (old_oid, new_oid): (&str, &str) = (plan.old_oid, plan.new_oid);
    let shape: &MergeShape = plan.shape;
    let reference: String = shape.reference.to_string_lossy().into_owned();
    let target_ref: Option<String> = {
        let named: GitOutput = session.run(&["symbolic-ref", "--quiet", "HEAD"])?;
        if named.succeeded() { Some(String::from_utf8_lossy(&named.stdout).trim().to_owned()) } else { None }
    };
    // Git's fast-forward takes `index.lock` before it changes anything and dies when another program holds it. The merge waits
    //  for the holder here, bounded and naming it; a lock taken in the instant between is met by the round made again.
    wait_for_index_release_here(git_dir.join("index").as_path(), "merge");
    let transaction_id: String = match random_uuid() {
        Ok(id) => id,
        Err(error) => return Err(Bail::from(error.0)),
    };
    let owner = match current_transaction_owner(transaction_id.as_str(), now_iso().as_str()) {
        Ok(found) => found,
        Err(error) => return Err(Bail::from(error.to_string())),
    };
    let intent: MergeIntent = MergeIntent {
        owner,
        repository_root: root.to_path_buf(),
        target_ref,
        old_oid: String::from(old_oid),
        new_oid: String::from(new_oid),
    };
    let record: PathBuf = match write_merge_intent(git_dir, &intent) {
        Ok(written) => written,
        Err(error) => return Err(Bail::from(format!("the merge record could not be written: {error}"))),
    };
    reach_phase(Phase::MergeIntent)?;
    let mut overlay: Vec<(OsString, OsString)> = input.overlay.to_vec();
    overlay.push((OsString::from("GIT_REFLOG_ACTION"), OsString::from(format!("merge {reference}"))));
    let mover: Session = Session {
        context: GitContext { overlay, ..session.context.clone() },
    };
    // A start failure is handled like a refusal: the record settles whatever Git began.
    let moved: GitOutput = match mover.run(&["merge", "--ff-only", "--no-edit", new_oid]) {
        Ok(output) => output,
        Err(reason) => GitOutput { code: None, stdout: Vec::new(), stderr: reason.into_bytes() },
    };
    let place: RecoveryPlace = RecoveryPlace {
        git: GitContext { global_prefix: Vec::new(), ..session.context.clone() },
        cwd: root.to_path_buf(),
        common_dir: git_dir.to_path_buf(),
    };
    if !moved.succeeded() {
        let detail: String = moved.error_text();
        let lock_taken: bool = detail.contains("index.lock");
        return Err(match settle_failed_merge(&place, record.as_path(), &intent) {
            Ok(MergeRecoveryAction::BranchMoved) => Bail::Again(format!("{reference} was merged on a branch that moved meanwhile")),
            Ok(MergeRecoveryAction::Discarded) if lock_taken => Bail::Again(String::from("another program took the index lock first")),
            Ok(MergeRecoveryAction::Completed) => Bail::Done(IsolatedMerge::Finished {
                code: 0,
                stdout: String::new(),
                stderr: format!(
                    "monochromatic-git-policy-cli: git merge {reference}: Git reported an error ({detail}) after it had started, \
                     and the wrapper completed the merge.\n"
                ),
            }),
            Ok(_) => refused(
                moved.code.unwrap_or(2),
                format!("{}{}", String::from_utf8_lossy(&moved.stderr), not_applied_finding(reference.as_str(), detail.as_str())),
            ),
            Err(error) => refused(2, render_engine_failure_with_hint(0, EngineFailureCode::ContentUnavailable, error.0.as_str(), None)),
        });
    }
    reach_phase(Phase::MergeApplied)?;
    if let Err(error) = remove_merge_intent(record.as_path()) {
        debug(TAG, format!("the merge record could not be removed: {error}").as_str());
    }
    // The fast-forward carries no `--quiet`, and what Git says of it ("Updating a..b", "Fast-forward", its diffstat) is about
    //  the wrapper's own step, not about the merge the person asked for. The person's streams carry exactly what native Git's
    //  merge prints (the wrapper's lines below), so what Git wrote to standard output about the step is kept in the journal. What
    //  it wrote to standard error (a warning) still reaches the person, as it did before the option was dropped.
    record_streams(&["merge", "--ff-only", "--no-edit", new_oid], &moved.stdout, &[]);
    let mut stdout: String = String::new();
    if !shape.quiet {
        stdout.push_str("Merge made by the 'ort' strategy.\n");
        if shape.stat.unwrap_or(plan.config.stat)
            && let Ok(stat) = session.run(&["diff", "--stat", "--summary", old_oid, new_oid])
        {
            stdout.push_str(String::from_utf8_lossy(&stat.stdout).as_ref());
        }
    }
    return Ok(IsolatedMerge::Finished {
        code: 0,
        stdout,
        stderr: String::from_utf8_lossy(&moved.stderr).into_owned(),
    });
}

/// What:
///  Reach a marker phase,
/// failing the invocation when the marker variable is malformed.
/// Why:
///   The crash controls arm a phase by environment;
/// a malformed value must not be ignored.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await reachTransactionPhase({ phase });
/// ```
pub fn reach_phase(phase: Phase) -> Result<(), Bail> {
    return match reach_transaction_phase(phase, None) {
        Ok(()) => Ok(()),
        Err(error) => Err(refused(2, format!("monochromatic-git-policy-cli: {error}\n"))),
    };
}
