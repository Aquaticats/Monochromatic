//! What:
//!  What one round of an isolated merge writes of its merge,
//!  once the round has judged the paths it changes:
//! the message the merge carries,
//! the capture of the person's own work,
//! the merge commit,
//! the update that puts it in the checkout,
//! and the lines that name the files the word pass merged.
//! Why:
//!   `merge_isolation_round.rs` reads the branch,
//! the person's paths and the merge tree,
//! and keeps its own file inside the code-line limit;
//! the steps that write the message,
//! the capture and the commit are a file of their own that it imports
//!  and that imports nothing from it.
//!  The words a merge whose editor left no message prints belong here too,
//! because nothing else reads them,
//! and so do the lines a finished merge prints about the files the word pass merged,
//! because nothing else prints them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const done = await writeTheRoundsMerge({ input, session, shape, config, ends, places, tree, description, overlap, opensEditor, edited, secondPassFiles });
//! ```

/// Diagnostics for the log.
use crate::diagnostic_log::debug;
/// The number this command is known by in the journal.
use crate::journald_native::CLIENT_IDENTIFIER;
/// The writing half of an isolated merge,
/// which moves the checkout.
use crate::merge_isolation_apply::apply;
/// The merge commit this round creates.
use crate::merge_isolation_apply_commit::commit_merge;
/// The person's own work in the paths the merge changes.
use crate::merge_isolation_carry::capture_work;
/// The record of that work in the paths the merge changes.
use crate::merge_isolation_carry_parts::Capture;
/// The ref of its own that keeps the capture's objects from `git gc`.
use crate::merge_isolation_carry_ref::keep_capture;
/// The editor a merge opens for its message when the checkout holds the person's own work.
use crate::merge_isolation_editor::{
    EditorPlace,
    EditorRefusal,
    asked_once,
};
/// The questions an isolated merge asks Git.
use crate::merge_isolation_git_reads::Session;
/// The merge messages.
use crate::merge_isolation_message::{
    clean_message,
    default_merge_message,
};
/// The values an isolated merge passes between its steps,
/// and the ending every early stop of one takes.
use crate::merge_isolation_run_types::{
    Bail,
    IsolatedMerge,
    IsolatedMergeInput,
    MergeConfig,
    Plan,
    TAG,
    refused,
};
/// The command-line reading.
use crate::merge_options::MergeShape;
/// The word-level second pass over the conflicts of the merge tree.
use crate::merge_second_pass::Merged;
/// The lines that name the files the word pass merged.
use crate::merge_second_pass_tree::announce;
/// Real Git.
use crate::transaction_git::GitContext;
/// `Path` is a borrowed filesystem path.
use std::path::Path;

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
pub fn with_second_pass_report(
    done: IsolatedMerge,
    merged: &[Merged],
) -> IsolatedMerge {
    match done {
        IsolatedMerge::Finished {
            code: 0,
            stdout,
            stderr,
        } if !merged.is_empty() => {
            let mut report: String = String::new();
            for line in announce(
                "monochromatic-git-policy-cli: ",
                (CLIENT_IDENTIFIER, "merge"),
                merged,
            ) {
                report.push_str(line.as_str());
                report.push('\n');
            }
            return IsolatedMerge::Finished {
                code: 0,
                stdout,
                stderr: report + stderr.as_str(),
            };
        }
        other => return other,
    }
}

/// What:
///  The message the merge carries,
/// the capture of the person's own work in the paths it changes,
/// the merge commit,
/// and the update that puts it in the checkout.
/// Why:
///   These are the writes of one round,
/// and the round that read the branch hands them what it read,
/// so no step here reads the repository again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function writeTheRoundsMerge(args): Promise<IsolatedMerge>; // throws Bail
/// ```
pub fn write_the_rounds_merge(
    input: &IsolatedMergeInput<'_>,
    session: &Session,
    shape: &MergeShape,
    config: &MergeConfig,
    ends: (&str, &str, &str),
    places: (&Path, &Path),
    tree: &str,
    description: Option<String>,
    overlap: &[Vec<u8>],
    opens_editor: bool,
    edited: &mut Option<Vec<u8>>,
    second_pass_files: &[Merged],
) -> Result<IsolatedMerge, Bail> {
    let (head, target, reference): (&str, &str, &str) = ends;
    let (git_dir, root): (&Path, &Path) = places;
    let written: Vec<u8> = match (&shape.message, description) {
        (Some(given), _) => given.clone(),
        (None, Some(line)) => default_merge_message(&session.context, line.as_str(), shape.log)?,
        (None, None) => return Err(Bail::Forward(None)),
    };
    // The message a merge that opens an editor is committed with is the person's: their editor reads the merge's own
    //  message in `<git-dir>/MERGE_MSG`, as native Git writes it, and its cleanup is the one an edited commit gets. The
    //  editor runs before the capture, so a message that is never written leaves no object and no ref behind.
    let message: Vec<u8> = if opens_editor {
        let place: EditorPlace<'_> = EditorPlace {
            session,
            git_dir,
        };
        match asked_once(edited, &place, written.as_slice()) {
            Ok(found) => found,
            Err(EditorRefusal::NoEditor) => return Err(Bail::Forward(None)),
            Err(refusal) => return Err(refused(1, editor_stopped(reference, &refusal))),
        }
    } else {
        clean_message(&session.context, written.as_slice())?
    };
    // The bytes are written and kept on a ref of their own before the record exists and before the first byte of the checkout
    //  changes, so a kill at any instant leaves them where recovery finds them. A capture that cannot be made is native Git's
    //  command, which refuses such a checkout with its own words and changes nothing.
    let carried: Option<Capture> = match capture_for(&session.context, root, overlap) {
        Ok(found) => found,
        Err(reason) => {
            debug(TAG, format!("native Git runs this merge: its paths hold work this command cannot carry: {reason}").as_str());
            return Err(Bail::Forward(None));
        }
    };
    let new_oid: String = match commit_merge(session, shape, config, (head, target, tree), message)
    {
        Ok(created) => created,
        Err(reason) => {
            return Err(refused(
                128,
                format!(
                    "monochromatic-git-policy-cli: git merge {reference}: the merge commit could not be created, and nothing was changed: {reason}\n"
                ),
            ));
        }
    };
    let plan: Plan<'_> = Plan {
        shape,
        config,
        git_dir,
        root,
        old_oid: head,
        new_oid: new_oid.as_str(),
        carried: carried.as_ref(),
    };
    let done: IsolatedMerge = apply(input, session, &plan)?;
    return Ok(with_second_pass_report(done, second_pass_files));
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
fn capture_for(
    context: &GitContext,
    root: &Path,
    paths: &[Vec<u8>],
) -> Result<Option<Capture>, String> {
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
fn editor_stopped(
    reference: &str,
    refusal: &EditorRefusal,
) -> String {
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
