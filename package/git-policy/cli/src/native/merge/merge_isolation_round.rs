//! What:
//!  One round of an isolated merge:
//! the command it is asked to complete,
//! the branch as it is now,
//! the settings and the paths it changes,
//! the checks that stop it before anything is written,
//! the merge tree over those paths,
//! and the writes it hands on.
//! Why:
//!   `merge_isolation.rs` holds the entry every merge comes through and the loop that makes a round
//!  again when the branch moved;
//! a round is a job of its own,
//! and it is a file of its own so both stay inside the code-line limit.
//!  Every check of this round stops the merge before the first byte is written,
//! and the writes themselves are the last step it hands on.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = await attemptRound({ input, edited }); // throws Bail
//! ```

/// Names and sets of unsaved work.
use crate::auto_push_dirty_paths::{
    diff_paths,
    overlapping_paths,
};
/// Reading an environment value.
use crate::child_environment::environment_value;
/// Diagnostics for the log.
use crate::diagnostic_log::debug;
/// The directory a merge would replace with a file,
/// and the sentence that stops the merge instead.
use crate::merge_isolation_a_directory_the_merge_replaces::{
    a_directory_the_merge_would_replace,
    the_line,
};
/// A named pipe,
/// a socket or a device at a path the merge writes or deletes,
/// and the sentence that stops the merge instead.
use crate::merge_isolation_a_special_file_the_merge_changes::{
    a_special_file_the_merge_changes,
    the_line as a_special_file_line,
};
/// The merge tree of an isolated merge,
/// and the word-level second pass over its conflicts.
use crate::merge_isolation_apply::merge_tree;
/// The paths a merge that writes or deletes them would discard.
use crate::merge_isolation_dirty_paths::dirty_paths;
/// The questions an isolated merge asks Git and the terminal.
use crate::merge_isolation_git_reads::{
    Session,
    editor_would_open,
    lines_of,
    read_config,
    read_fetch_head,
    stdin_and_stdout_are_one_terminal,
};
/// The paths a merge changes whose edit Git's status does not show.
use crate::merge_isolation_marks_the_status_hides::paths_the_status_hides;
/// The merge messages.
use crate::merge_isolation_message::describe_merge_source;
/// Recovery of the merges a dead process left,
/// before this merge reads the checkout.
use crate::merge_isolation_recovery_before_the_merge::recover_dead_merges;
/// What one round writes of its merge once its paths are judged.
use crate::merge_isolation_round_writes::write_the_rounds_merge;
/// The values an isolated merge passes between its steps,
/// the command's own ending,
/// and the markers of other operations in progress.
use crate::merge_isolation_run_types::{
    Bail,
    IN_PROGRESS_MARKERS,
    IsolatedMerge,
    IsolatedMergeInput,
    MergeConfig,
    TAG,
    refused,
};
/// The command-line reading.
use crate::merge_options::{
    FastForwardChoice,
    MergeShape,
    ParsedMerge,
    parse_merge_shape,
};
/// The word-level second pass over the conflicts of the merge tree,
/// and the files it merged.
use crate::merge_second_pass::Merged;
/// Real Git.
use crate::transaction_git::{
    GitContext,
    GitOutput,
};
/// The repository shape.
use crate::worktree_identity::WorktreeIdentity;
/// `Path` is a borrowed filesystem path.
use std::path::Path;
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
pub(crate) fn attempt_round(
    input: &IsolatedMergeInput<'_>,
    edited: &mut Option<Vec<u8>>,
) -> Result<IsolatedMerge, Bail> {
    if !input.command.is_builtin() || input.command.word() != b"merge" {
        return Err(Bail::Forward(None));
    }
    let (git_dir, root, common_dir): (&Path, &Path, &Path) = match input.identity {
        WorktreeIdentity::MainWorktree {
            git_dir,
            worktree_root,
            common_dir,
            ..
        }
        | WorktreeIdentity::LinkedWorktree {
            git_dir,
            worktree_root,
            common_dir,
            ..
        } => (
            git_dir.as_path(),
            worktree_root.as_path(),
            common_dir.as_path(),
        ),
        _ => return Err(Bail::Forward(None)),
    };
    recover_dead_merges(input, git_dir, root)?;
    let shape: MergeShape = match parse_merge_shape(input.command.region()) {
        ParsedMerge::Supported(found) => found,
        ParsedMerge::Unsupported(reason) => {
            debug(
                TAG,
                format!("native Git runs this merge: {reason}").as_str(),
            );
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
        && editor_would_open(auto_edit.as_deref(), stdin_and_stdout_are_one_terminal())
            .unwrap_or(true);
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
            .run(&[
                "diff-tree",
                "-r",
                "--name-only",
                "-z",
                "--no-renames",
                head,
                target,
            ])?
            .stdout
            .as_slice(),
    );
    // A directory standing where the merge writes a file is the person's own work, and Git's update of the checkout deletes
    //  everything in it: measured, an untracked `a.txt/nested/file.txt` and a staged one both went with the directory, the
    //  staged one with its index entry. No layer of the carry can hold a directory, so the merge stops here, before the
    //  first byte of the worktree, the index or the branch changes (`SPEC.md`, "A directory that stands where the merge puts a file").
    if let Some(found) = a_directory_the_merge_would_replace(
        &session.context,
        root,
        head,
        target,
        touched.as_slice(),
    )
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
        let full_name: String =
            session.text(&["rev-parse", "--symbolic-full-name", reference.as_str()])?;
        Some(
            describe_merge_source(reference.as_str(), full_name.as_str(), target)
                .ok_or(Bail::Forward(None))?,
        )
    };
    // The paths whose own work the merge would write over: everything Git's status reports, untracked files included, because
    //  a merge that adds a path an untracked file stands at is refused by Git too, and the ignored untracked files of the
    //  person's, which no status lists and which the merge overwrites without a word. They are read once here, before the merge
    //  tree exists. Whether the checkout holds any of the paths the merge touches is what decides that a conflict of the merge
    //  itself is resolved here rather than handed to native Git: its merge refuses such a checkout outright
    //  (`SPEC.md`, "Merges and commits never fail", "Carry edits forward").
    let mut dirty: Vec<Vec<u8>> = dirty_paths(&session, root, touched.as_slice())?;
    // Git's status reads an entry marked assume-unchanged or skip-worktree as one not to compare, so an edit there is work the
    //  person's, listed here so the capture carries it and Git's own update of the checkout never meets it unjudged.
    dirty.extend(paths_the_status_hides(
        &session.context,
        root,
        touched.as_slice(),
    )?);
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
            .run(&[
                "diff-tree",
                "-r",
                "--name-only",
                "-z",
                "--no-renames",
                head,
                tree.as_str(),
            ])?
            .stdout
            .as_slice(),
    );
    let overlap: Vec<Vec<u8>> = overlapping_paths(dirty.as_slice(), incoming.as_slice());
    // A named pipe, a socket or a device at a path this merge writes or deletes holds no bytes for the carry, and Git's update
    //  of the checkout would remove it without a copy; reading one could wait for a writer that never comes, so the merge stops
    //  before the first write and names what it kept (`SPEC.md`, "Merges and commits never fail").
    let special: Vec<Vec<u8>> = a_special_file_the_merge_changes(root, incoming.as_slice());
    if !special.is_empty() {
        return Err(refused(
            1,
            a_special_file_line(reference.as_str(), special.as_slice()),
        ));
    }
    return write_the_rounds_merge(
        input,
        &session,
        &shape,
        &config,
        (head, target, reference.as_str()),
        (git_dir, root),
        tree.as_str(),
        description,
        overlap.as_slice(),
        opens_editor,
        edited,
        second_pass_files.as_slice(),
    );
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
