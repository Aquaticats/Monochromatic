//! What:
//!  A true `git merge <branch>` completed in isolation when the index holds staged changes that make native Git
//!  refuse it:
//! the merge is computed from objects,
//! committed,
//! and applied with Git's own two-way tree update,
//! which
//!  keeps every entry and every file the merge does not change.
//! Why:
//!  Several sessions share one checkout.
//!  Native Git refuses every merge that creates a commit while the index differs from `HEAD` at all (measured in
//!  `doc/planning/cli-git-merge-with-staged-index.md`),
//! although the merge may touch none of the staged paths.
//!  A fast-forward is not refused for unrelated paths,
//! because it is a two-way update that keeps entries it does not
//!  write;
//! a merge commit whose first parent is the current commit is exactly such an update.
//!  So the merge tree comes from `git merge-tree --write-tree`,
//! the commit from `git commit-tree`,
//! and the checkout
//!  moves with `git merge --ff-only <commit>`,
//! which refuses when the branch is no longer where the merge was computed,
//! after a check that no changed path is staged
//!  or modified by someone else.
//!  An intent record,
//! written before Git starts,
//! lets recovery finish or discard a merge whose process died.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = await runIsolatedMerge({ command, identity, environment }); // 'forward' | { code, stdout, stderr }
//! ```

/// Names and sets of unsaved work.
use crate::auto_push_dirty_paths::{diff_paths, overlapping_paths, status_paths};
/// Reading an environment value.
use crate::child_environment::environment_value;
/// Diagnostics for the log.
use crate::diagnostic_log::debug;
/// The failure rendering for state that cannot be read.
use crate::diagnostics::{EngineFailureCode, render_engine_failure_with_hint};
/// The merge messages.
use crate::merge_isolation_message::{clean_message, default_merge_message, describe_merge_source};
/// The word-level second pass over the conflicts of the merge tree,
/// and the attribute tree it reads.
use crate::merge_second_pass::Merged;
/// Recovery of records.
use crate::merge_isolation_recovery::recover_interrupted_merges;
/// The person's own work in the paths the merge changes,
/// and the durable ref that keeps it.
use crate::merge_isolation_carry::{capture_work, keep_capture};
/// The directory a merge would replace with a file,
/// and the sentence that stops the merge instead.
use crate::merge_isolation_a_directory_the_merge_replaces::{a_directory_the_merge_would_replace, the_line};
/// A named pipe,
/// a socket or a device at a path the merge writes or deletes,
/// and the sentence that stops the merge instead.
use crate::merge_isolation_a_special_file_the_merge_changes::{a_special_file_the_merge_changes, the_line as a_special_file_line};
/// The paths a merge changes whose edit Git's status does not show.
use crate::merge_isolation_marks_the_status_hides::paths_the_status_hides;
use crate::merge_isolation_carry_parts::Capture;
/// The editor a merge opens for its message when the checkout holds the person's own work.
use crate::merge_isolation_editor::{EditorPlace, EditorRefusal, asked_once};
/// The command-line reading.
use crate::merge_options::{FastForwardChoice, MergeShape, ParsedMerge, parse_merge_shape};
/// The place recovery runs in.
use crate::merge_isolation_recovery::RecoveryPlace;
/// Real Git.
use crate::transaction_git::{GitContext, GitOutput};
/// The repository shape.
use crate::worktree_identity::WorktreeIdentity;
/// Owned operating-system text.
pub use crate::merge_isolation_run_types::{IsolatedMerge, IsolatedMergeInput};
use crate::merge_isolation_run_types::{Bail, MAX_ROUNDS, MergeConfig, Plan, TAG, IN_PROGRESS_MARKERS, refused};
use crate::merge_isolation_git_reads::{Session, editor_would_open, stdin_and_stdout_are_one_terminal, lines_of, read_config, read_fetch_head};
use crate::merge_isolation_apply::{apply, commit_merge, merge_tree, with_second_pass_report};
/// The notices of a round that is made again.
use std::io::Write;
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{Path, PathBuf};

/// What:
///  Complete or refuse one merge,
/// or hand the command back to native Git.
/// Why:
///   The wrapper acts only on the exact failing case:
/// a supported merge command,
/// a staged index,
/// a true merge.
///  Everything else is native Git's,
/// including its own errors.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function runIsolatedMerge(input: IsolatedMergeInput): Promise<IsolatedMerge>;
/// ```
pub fn run_isolated_merge(input: &IsolatedMergeInput<'_>) -> IsolatedMerge {
    // What the person wrote in the editor, kept across the rounds a moved branch makes again, so their message is
    //  never asked for twice.
    let mut edited: Option<Vec<u8>> = None;
    match attempt(input, &mut edited) {
        Ok(done) | Err(Bail::Done(done)) => return done,
        Err(Bail::Forward(notice)) => return IsolatedMerge::Forward { notice },
        // `attempt` makes the rounds itself and never lets one escape; native Git is the safe answer to a bug here.
        Err(Bail::Again(reason)) => {
            debug(TAG, format!("a round escaped the attempt ({reason}); native Git runs the merge").as_str());
            return IsolatedMerge::Forward { notice: None };
        }
    }
}

/// What:
///  The whole attempt,
/// with `Bail` standing for every early ending.
/// Why:
///   Early endings are many and mostly mean "forward";
/// the question mark keeps the sequence readable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function attempt(input): Promise<IsolatedMerge>; // throws Bail
/// ```
fn attempt(input: &IsolatedMergeInput<'_>, edited: &mut Option<Vec<u8>>) -> Result<IsolatedMerge, Bail> {
    for round in 1..=MAX_ROUNDS {
        match attempt_round(input, edited) {
            Err(Bail::Again(reason)) => {
                debug(TAG, format!("round {round} of {MAX_ROUNDS}: {reason}").as_str());
                let _noted = std::io::stderr().write_all(format!("monochromatic-git-policy-cli: git merge: {reason}; merging again on the current branch.\n").as_bytes());
            }
            finished => return finished,
        }
    }
    return Err(refused(
        2,
        format!(
            "monochromatic-git-policy-cli: git merge: the branch moved, a file the merge carries was saved again, or the index lock was \
             taken in {MAX_ROUNDS} merges in a row, so the wrapper stopped; nothing of yours was changed. Run the merge again.\n"
        ),
    ));
}

/// What:
///  One round of the attempt:
/// everything from the recovery of dead merges to the application,
/// on the branch as it is now.
/// Why:
///   A round that finds the branch moved by the time it applies ends with the checkout as it was,
/// and the next round
///  starts from the new head.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function attemptRound(input): Promise<IsolatedMerge>; // throws Bail
/// ```
fn attempt_round(input: &IsolatedMergeInput<'_>, edited: &mut Option<Vec<u8>>) -> Result<IsolatedMerge, Bail> {
    if !input.command.is_builtin() || input.command.word() != b"merge" {
        return Err(Bail::Forward(None));
    }
    let (git_dir, root, common_dir): (&Path, &Path, &Path) = match input.identity {
        WorktreeIdentity::MainWorktree { git_dir, worktree_root, common_dir, .. }
        | WorktreeIdentity::LinkedWorktree { git_dir, worktree_root, common_dir, .. } => {
            (git_dir.as_path(), worktree_root.as_path(), common_dir.as_path())
        }
        _ => return Err(Bail::Forward(None)),
    };
    recover_dead_merges(input, git_dir, root)?;
    let shape: MergeShape = match parse_merge_shape(input.command.region()) {
        ParsedMerge::Supported(found) => found,
        ParsedMerge::Unsupported(reason) => {
            debug(TAG, format!("native Git runs this merge: {reason}").as_str());
            return Err(Bail::Forward(None));
        }
    };
    for name in IN_PROGRESS_MARKERS {
        if git_dir.join(name).exists() {
            return Err(Bail::Forward(None));
        }
    }
    let session: Session = Session {
        context: GitContext {
            real_git: input.real_git.to_path_buf(),
            overlay: input.overlay.to_vec(),
            global_prefix: input.command.arguments[..input.command.subcommand_index].to_vec(),
            // The merge message passes through `git stripspace` and `git fmt-merge-msg` as input.
            common_dir: Some(common_dir.to_path_buf()),
        },
    };
    let Some(reference) = shape.reference.to_str().map(String::from) else {
        return Err(Bail::Forward(None));
    };
    // Exit 1 means another session has staged work, 0 means the index equals `HEAD`; the merge is completed here in both
    //  cases, because native Git's merge stashes first (`git stash create`), dies when `index.lock` is held, and moves the
    //  branch only after it changed the checkout. What native Git does better stays its own: a conflict that the person
    //  resolves in the worktree, and an overlap with unsaved work, which it words itself.
    let staged: GitOutput = session.run(&["diff-index", "--cached", "--quiet", "HEAD", "--"])?;
    let staged_work: bool = match staged.code {
        Some(1) => true,
        Some(0) => false,
        _ => return Err(Bail::Forward(None)),
    };
    // The value as text, with bytes that are not text replaced; `.map` takes the two steps by path.
    let auto_edit: Option<String> = environment_value(input.environment, "GIT_MERGE_AUTOEDIT")
        .as_deref()
        .map(std::ffi::OsStr::to_string_lossy)
        .map(std::borrow::Cow::into_owned);
    // `-m` gives the message and `--no-edit` declines the editor; without either a terminal opens one. The decision is
    //  only taken here: a merge whose paths hold the person's own work is this command's, whether an editor opens or not.
    let opens_editor: bool = shape.message.is_none()
        && !shape.no_edit
        && editor_would_open(auto_edit.as_deref(), stdin_and_stdout_are_one_terminal()).unwrap_or(true);
    let config: MergeConfig = read_config(&session)?;
    if config.deferred {
        return Err(Bail::Forward(None));
    }
    // `FETCH_HEAD` is the merge of a pull: the one head the file lists for merging, and the file's lines name it in the message.
    let fetched: Option<(String, String)> = if reference == "FETCH_HEAD" {
        Some(read_fetch_head(git_dir)?)
    } else {
        None
    };
    let target_spec: String = match &fetched {
        Some((oid, _)) => format!("{oid}^{{commit}}"),
        None => format!("{reference}^{{commit}}"),
    };
    let both = session.run(&["rev-parse", "HEAD^{commit}", target_spec.as_str()])?;
    let ids: Vec<String> = lines_of(&both);
    if both.code != Some(0) || ids.len() != 2 {
        return Err(Bail::Forward(None));
    }
    let (head, target): (&str, &str) = (ids[0].as_str(), ids[1].as_str());
    let bases: Vec<String> = lines_of(&session.run(&["merge-base", "--all", head, target])?);
    let fast_forwardable: bool = bases.len() == 1 && bases[0] == head;
    if bases.len() == 1 && bases[0] == target {
        return Err(Bail::Forward(None));
    }
    if bases.is_empty() && !shape.allow_unrelated_histories {
        return Err(Bail::Forward(None));
    }
    let merge_commit_wanted: bool = match shape.fast_forward {
        FastForwardChoice::Never => true,
        FastForwardChoice::Allowed => false,
        FastForwardChoice::Default => config.fast_forward == Some(false),
    };
    // A path either side changed is a path a merge of the two writes, so these are the merge's own paths before its tree exists.
    let touched: Vec<Vec<u8>> = diff_paths(
        session
            .run(&["diff-tree", "-r", "--name-only", "-z", "--no-renames", head, target])?
            .stdout
            .as_slice(),
    );
    // A directory standing where the merge writes a file is the person's own work, and Git's update of the checkout deletes
    //  everything in it: measured, an untracked `a.txt/nested/file.txt` and a staged one both went with the directory, the
    //  staged one with its index entry. No layer of the carry can hold a directory, so the merge stops here, before the
    //  first byte of the worktree, the index or the branch changes (`SPEC.md`, "A directory that stands where the merge puts a file").
    if let Some(found) = a_directory_the_merge_would_replace(&session.context, root, head, target, touched.as_slice())
        .map_err(Bail::from)?
    {
        return Err(refused(1, the_line(reference.as_str(), &found)));
    }
    if config.fast_forward_only && shape.fast_forward == FastForwardChoice::Default {
        return Err(Bail::Forward(None));
    }
    if fast_forwardable && !merge_commit_wanted {
        return Err(Bail::Forward(None));
    }
    let description: Option<String> = if shape.message.is_some() {
        None
    } else if let Some((_, lines)) = fetched {
        Some(lines)
    } else {
        let full_name: String = session.text(&["rev-parse", "--symbolic-full-name", reference.as_str()])?;
        Some(describe_merge_source(reference.as_str(), full_name.as_str(), target).ok_or(Bail::Forward(None))?)
    };
    // The paths whose own work the merge would write over: everything Git's status reports, untracked files included, because
    //  a merge that adds a path an untracked file stands at is refused by Git too. They are read once here, before the merge
    //  tree exists. Whether the checkout holds any of the paths the merge touches is what decides that a conflict of the merge
    //  itself is resolved here rather than handed to native Git: its merge refuses such a checkout outright
    //  (`SPEC.md`, "Merges and commits never fail", "Carry edits forward").
    let mut dirty: Vec<Vec<u8>> = dirty_paths(&session)?;
    // Git's status reads an entry marked assume-unchanged or skip-worktree as one not to compare, so an edit there is work the
    //  person's, listed here so the capture carries it and Git's own update of the checkout never meets it unjudged.
    dirty.extend(paths_the_status_hides(&session.context, root, touched.as_slice())?);
    let carrying: bool = !overlapping_paths(dirty.as_slice(), touched.as_slice()).is_empty();
    // A merge that would open an editor is native Git's while the checkout holds nothing the merge would write over:
    //  it performs the merge itself and then opens the editor. Over the person's own work it cannot run at all, because
    //  it refuses such a checkout before the editor opens (`doc/planning/cli-git-merge-with-staged-index.md`), so the
    //  merge is completed here and the editor is opened here, on the message the merge would carry (`SPEC.md`, "A merge
    //  whose paths hold the person's own work carries it forward").
    if opens_editor && !staged_work && !carrying {
        return Err(Bail::Forward(None));
    }
    let (tree, second_pass_files): (String, Vec<Merged>) = merge_tree(
        &session,
        &shape,
        (head, target, reference.as_str()),
        (common_dir, root, input.environment),
        staged_work,
        carrying,
    )?;
    let incoming: Vec<Vec<u8>> = diff_paths(
        session
            .run(&["diff-tree", "-r", "--name-only", "-z", "--no-renames", head, tree.as_str()])?
            .stdout
            .as_slice(),
    );
    let overlap: Vec<Vec<u8>> = overlapping_paths(dirty.as_slice(), incoming.as_slice());
    // A named pipe, a socket or a device at a path this merge writes or deletes holds no bytes for the carry, and Git's update
    //  of the checkout would remove it without a copy; reading one could wait for a writer that never comes, so the merge stops
    //  before the first write and names what it kept (`SPEC.md`, "Merges and commits never fail").
    let special: Vec<Vec<u8>> = a_special_file_the_merge_changes(root, incoming.as_slice());
    if !special.is_empty() {
        return Err(refused(1, a_special_file_line(reference.as_str(), special.as_slice())));
    }
    let written: Vec<u8> = match (&shape.message, description) {
        (Some(given), _) => given.clone(),
        (None, Some(line)) => default_merge_message(&session.context, line.as_str(), shape.log)?,
        (None, None) => return Err(Bail::Forward(None)),
    };
    // The message a merge that opens an editor is committed with is the person's: their editor reads the merge's own
    //  message in `<git-dir>/MERGE_MSG`, as native Git writes it, and its cleanup is the one an edited commit gets. The
    //  editor runs before the capture, so a message that is never written leaves no object and no ref behind.
    let message: Vec<u8> = if opens_editor {
        let place: EditorPlace<'_> = EditorPlace { session: &session, git_dir };
        match asked_once(edited, &place, written.as_slice()) {
            Ok(found) => found,
            Err(EditorRefusal::NoEditor) => return Err(Bail::Forward(None)),
            Err(refusal) => return Err(refused(1, editor_stopped(reference.as_str(), &refusal))),
        }
    } else {
        clean_message(&session.context, written.as_slice())?
    };
    // The bytes are written and kept on a ref of their own before the record exists and before the first byte of the checkout
    //  changes, so a kill at any instant leaves them where recovery finds them. A capture that cannot be made is native Git's
    //  command, which refuses such a checkout with its own words and changes nothing.
    let carried: Option<Capture> = match capture_for(&session.context, root, overlap.as_slice()) {
        Ok(found) => found,
        Err(reason) => {
            debug(TAG, format!("native Git runs this merge: its paths hold work this command cannot carry: {reason}").as_str());
            return Err(Bail::Forward(None));
        }
    };
    let new_oid: String = match commit_merge(&session, &shape, &config, (head, target, tree.as_str()), message) {
        Ok(created) => created,
        Err(reason) => {
            return Err(refused(
                128,
                format!("monochromatic-git-policy-cli: git merge {reference}: the merge commit could not be created, and nothing was changed: {reason}\n"),
            ));
        }
    };
    let plan: Plan<'_> = Plan {
        shape: &shape,
        config: &config,
        git_dir,
        root,
        old_oid: head,
        new_oid: new_oid.as_str(),
        carried: carried.as_ref(),
    };
    let done: IsolatedMerge = apply(input, &session, &plan)?;
    return Ok(with_second_pass_report(done, second_pass_files.as_slice()));
}

/// What:
///  Every path whose worktree or index differs from the branch,
/// untracked files included.
/// Why:
///   These are exactly the paths a merge that writes them would discard,
/// and the untracked ones are why native
///  Git refuses a merge that adds a path a file already stands at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const dirty = statusPaths(await git(['--no-optional-locks', 'status', '--porcelain=v1', '-z', '--untracked-files=all']));
/// ```
fn dirty_paths(session: &Session) -> Result<Vec<Vec<u8>>, Bail> {
    let status: GitOutput = session.run(&[
        "--no-optional-locks",
        "status",
        "--porcelain=v1",
        "-z",
        "--untracked-files=all",
    ])?;
    if !status.succeeded() {
        return Err(Bail::from(format!("git status failed: {}", status.error_text())));
    }
    return Ok(status_paths(status.stdout.as_slice()));
}

/// What:
///  The person's own work in `paths`,
/// captured as objects and kept on a ref of its own.
/// Why:
///   A ref written before the record and before the checkout changes is what makes the bytes survive a kill at any
///  instant (`SPEC.md`,
/// "Merges and commits never fail").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function captureFor(context, root, paths): Promise<Capture | null>;
/// ```
fn capture_for(context: &GitContext, root: &Path, paths: &[Vec<u8>]) -> Result<Option<Capture>, String> {
    if paths.is_empty() {
        return Ok(None);
    }
    let capture: Capture = capture_work(context, root, paths)?;
    return Ok(Some(keep_capture(context, root, &capture)?));
}

/// What:
///  The words for a merge whose editor left no message.
/// Why:
///   Everything the command would have changed is still as it was,
/// so the outcome is the person's message to rewrite,
///  and the sentence says both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function editorStopped(reference: string, refusal: EditorRefusal): string;
/// ```
fn editor_stopped(reference: &str, refusal: &EditorRefusal) -> String {
    let reason: &str = match refusal {
        EditorRefusal::Stopped(detail) => detail.as_str(),
        EditorRefusal::Empty => "the editor left an empty message",
        EditorRefusal::NoEditor => "Git resolved no editor for this environment",
    };
    return format!(
        "monochromatic-git-policy-cli: git merge {reference}: {reason}, so the merge was not made and nothing in the repository changed. \
         Run the merge again to write the message.\n"
    );
}

/// What:
///  Recover merges that died,
/// before this merge reads the checkout.
/// Why:
///   A merge killed between Git's checkout update and the branch move leaves the checkout half done,
/// and the
///  checks that follow would read it as another session's staged work.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recoverInterruptedMerges({ place, gitDir, stderr });
/// ```
fn recover_dead_merges(input: &IsolatedMergeInput<'_>, git_dir: &Path, root: &Path) -> Result<(), Bail> {
    let common_dir: PathBuf = match input.identity {
        WorktreeIdentity::MainWorktree { common_dir, .. } | WorktreeIdentity::LinkedWorktree { common_dir, .. } => {
            common_dir.clone()
        }
        _ => return Ok(()),
    };
    let place: RecoveryPlace = RecoveryPlace {
        git: GitContext {
            real_git: input.real_git.to_path_buf(),
            overlay: input.overlay.to_vec(),
            global_prefix: Vec::new(),
            common_dir: Some(common_dir.clone()),
        },
        cwd: root.to_path_buf(),
        common_dir,
        git_dir: git_dir.to_path_buf(),
    };
    // The notices of what recovery did reach the command's standard error: the branch may have moved onto a merge commit
    //  this command finished rather than the one it is about to compute (`doc/planning/cli-git-merge-and-repair-route-findings-on-main.md`, finding 1).
    match recover_interrupted_merges(&place, git_dir, &mut std::io::stderr()) {
        Ok(_) => return Ok(()),
        Err(error) => {
            return Err(refused(
                2,
                render_engine_failure_with_hint(0, EngineFailureCode::ContentUnavailable, error.0.as_str(), None),
            ));
        }
    }
}

/// Boolean settings,
/// the editor choice and the markers of other operations,
/// which need no repository.
#[cfg(test)]
#[path = "merge_isolation_tests.rs"]
mod tests;

/// The settings,
/// the staged-work check and the first decisions of an attempt,
/// each in a real repository.
#[cfg(test)]
#[path = "merge_isolation_repository_tests.rs"]
mod repository_tests;
