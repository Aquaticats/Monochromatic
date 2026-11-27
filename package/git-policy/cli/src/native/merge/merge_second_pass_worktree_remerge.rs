//! What:
//!  The awaited re-merge of the conflicts a stopped operation created:
//!  every conflict is merged,
//!  the merged bytes are written as blobs,
//!  and only the files no other command moved are staged as resolved and written to the worktree.
//! Why:
//!   The pass awaits every Git start it makes (`merge_second_pass_git_runner.rs`),
//!  so a person's command reaches it through one entry of the runtime,
//!  and the reconcile pass,
//!  which runs as a task,
//!  needs no entry at all.
//!  The order of the steps and every reason a path is left are the synchronous face's:
//!  the merge is computed first,
//!  the index is read again after it,
//!  and a path another command moved is named and never replaced.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const parts = await secondPassParts({ runner, conflicts, source, layout, held, stderr }); // { files, moved, unreadable } | undefined
//! ```

use crate::diagnostic_log::{
    debug,
    warn,
};
use crate::merge_second_pass::{
    AttributeSource,
    Conflict,
    ContextRunner,
    Left,
    Merged,
    Verdict,
    second_pass,
    write_blob,
};
use crate::merge_second_pass_worktree_install::install;
use crate::merge_second_pass_worktree_unchanged::{
    Before,
    Held,
    Kept,
    still_the_same,
};
use crate::merge_second_pass_worktree_watch::index_entries;
use crate::merge_second_pass_worktree_words::{
    moved_words,
    unreadable_words,
};
use crate::path_bytes::path_message_text;
use crate::repo_read_layout::RepoLayout;
use std::collections::BTreeMap;
use std::io::Write;

/// Tag used for the diagnostics of this part of the pass:
/// the same words the rest of the second pass logs under,
/// so one search finds every line about it.
const TAG: &str = "wordMergeWorktree";
/// What the awaited part of a forwarded pass came to.
/// Why:
///   The files it installed and the paths it left are what the report names,
/// and `None` says the conflicts stay as Git left them,
/// with the reason already given.
pub(super) struct SecondPassParts {
    /// The files staged as resolved and written to the worktree.
    pub(super) files: Vec<Merged>,
    /// The paths another command moved while the pass computed its merge,
    /// in words.
    pub(super) moved: Vec<String>,
    /// The paths nobody could compare,
    /// in words.
    pub(super) unreadable: Vec<String>,
}

/// What:
///  Re-merge the conflicts this command created and write the files whose every part resolves,
/// awaiting every Git start the pass makes.
/// Why:
///   The pass awaits its starts (`merge_second_pass_git_runner.rs`),
/// so a person's command reaches it through one entry of the runtime,
/// and the reconcile's pass,
/// which runs as a task,
/// needs no entry at all.
/// The order of the steps and every reason a path is left are the synchronous face's:
/// the merge is computed first,
/// the index is read again after it,
/// and a path another command moved is named and never replaced.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const parts = await secondPassParts({ runner, conflicts, source, layout, held, stderr }); // { files, moved, unreadable } | undefined
/// ```
pub(super) async fn second_pass_parts(
    runner: &ContextRunner<'_>,
    conflicts: &[Conflict],
    source: AttributeSource<'_>,
    layout: &RepoLayout,
    held: &BTreeMap<Vec<u8>, Held>,
    stderr: &mut dyn Write,
) -> Option<SecondPassParts> {
    let verdicts: Vec<Verdict> = match second_pass(runner, conflicts, source).await {
        Ok(found) => found,
        Err(reason) => {
            warn(TAG, format!("the second merge pass could not run, so the conflicts stay as Git left them: {reason}").as_str());
            return None;
        }
    };
    let mut merged: Vec<Merged> = Vec::new();
    for verdict in verdicts {
        match verdict {
            Verdict::Merged(file) => merged.push(*file),
            Verdict::Left {
                path,
                why,
            } => debug(
                TAG,
                format!(
                    "{} stays conflicted: {}",
                    path_message_text(path.as_slice()),
                    left_words(&why)
                )
                .as_str(),
            ),
        }
    }
    if merged.is_empty() {
        return None;
    }
    let mut oids: Vec<String> = Vec::new();
    for file in &merged {
        match write_blob(runner, file.bytes.as_slice()).await {
            Ok(oid) => oids.push(oid),
            Err(reason) => {
                warn(TAG, format!("the merged blob of {} could not be written, so the conflicts stay: {reason}", path_message_text(file.path.as_slice())).as_str());
                return None;
            }
        }
    }
    // The index is read again after the merge was computed: a path another command resolved and staged holds no stages of the merge's.
    let Some(entries_now) = index_entries(layout) else {
        warn(
            TAG,
            "the index cannot be read again before the merged files are installed, so the conflicts stay as they are",
        );
        return None;
    };
    let before: Before<'_> = Before {
        worktree_root: layout.worktree_root.as_path(),
        entries: entries_now.as_slice(),
        held,
    };
    let kept: Kept = still_the_same(&before, merged, oids);
    let mut moved_names: Vec<String> = Vec::new();
    for path in &kept.moved {
        moved_names.push(path_message_text(path.as_slice()));
    }
    let mut unreadable_names: Vec<String> = Vec::new();
    for path in &kept.unreadable {
        unreadable_names.push(path_message_text(path.as_slice()));
    }
    if kept.files.is_empty() {
        warn(TAG, format!("every merged file moved or could not be compared while the pass computed its merge, so they stay as they are now: {}{}", moved_names.join(", "), unreadable_names.join(", ")).as_str());
        // The person must still be told which file the pass did not touch, and which of the two reasons it was.
        let mut alone: String = String::new();
        if !moved_names.is_empty() {
            alone.push_str(moved_words(moved_names.as_slice()).as_str());
        }
        if !unreadable_names.is_empty() {
            alone.push_str(unreadable_words(unreadable_names.as_slice()).as_str());
        }
        let told: String = format!("{}\n", alone.trim_end());
        if let Err(error) = stderr.write_all(told.as_bytes()) {
            debug(TAG, format!("standard error did not take the line about the files the pass left alone: {error}").as_str());
        }
        return None;
    }
    let files: Vec<Merged> = kept.files;
    let file_oids: Vec<String> = kept.oids;
    if let Err(reason) = install(runner, files.as_slice(), file_oids.as_slice()).await {
        warn(TAG, format!("the merged files could not be installed, so the conflicts stay as Git left them: {reason}").as_str());
        return None;
    }
    return Some(SecondPassParts {
        files,
        moved: moved_names,
        unreadable: unreadable_names,
    });
}
/// Why a path stayed conflicted,
/// in words for the diagnostic log.
fn left_words(why: &Left) -> String {
    return format!("{why:?}");
}
