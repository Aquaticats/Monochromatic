//! What:
//!  The word-level second pass over the files a line merge could not settle,
//!  and the notice the person reads for the parts it merged.
//! Why:
//!   Git's line merge and the subsumption rules both had their say;
//!  what is left is conflicts of adjacent lines and of different words,
//!  which the owner decided (2026-11-03) the wrapper's own merges re-merge by word,
//!  all or none.
//!  A merge that leaves one path uncombined leaves them all,
//!  so a person never gets half of their change.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const combined = wordPass(site, tip, { tree, conflicts }, paths); // { combined: { tree, files } } | { conflict: paths }
//! ```

use super::service_replay_the_captured_change::{
    MergeSite,
    Merged,
};

use crate::child_environment::environment_with_overlay;
use crate::journald_native::CLIENT_IDENTIFIER;
use crate::merge_second_pass::{
    Conflict,
    ContextRunner,
    Merged as MergedFile,
};
use crate::merge_second_pass_tree::{
    AttributeProbe,
    announce,
    attribute_tree_variables_on_the_runtime,
    resolve_tree_on_the_runtime,
};
use crate::policy_events::PolicyEvent;
use crate::transaction_git::{
    GitRequest,
    run_git_checked,
};
use std::ffi::OsString;
use std::path::PathBuf;

/// What:
///  The event that tells the person which files the word pass merged for a replayed commit,
/// or nothing when none were.
/// Why:
///   The replayed commit carries the person's own message,
/// which is not changed;
/// the parts are told next to the commit's other
///  events,
/// and the journal gets one entry per file at the same time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// wordMergeNotice(files) // { type: 'word-merged', lines } | undefined
/// ```
pub fn word_merge_notice(files: &[MergedFile]) -> Option<PolicyEvent> {
    if files.is_empty() {
        return None;
    }
    return Some(PolicyEvent::WordMerged {
        lines: announce("", (CLIENT_IDENTIFIER, "commit-replay"), files),
    });
}

/// What:
///  The last resort for a merge that still conflicts:
/// the word-level second pass over its conflicted files.
/// Why:
///   Git's line merge and the subsumption rules have both had their say;
/// what is left is conflicts of adjacent lines and
///  of different words,
/// which the owner decided (2026-11-03) the wrapper's own merges re-merge by word,
/// all or none.
/// The
///  attributes of the landed tip apply,
/// because the merge reads no worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// wordPass(site, tip, { tree, conflicts }, paths) // { combined: { tree, files } } | { conflict: paths }
/// ```
/// What:
///  The top of the worktree Git printed,
/// with its line break removed and nothing else.
/// Why:
///   A directory may end in a space,
/// and trimming the line Git printed takes that space with it,
/// so every command of the second pass would
///  run in a directory that does not exist and the commit would be refused as an uncombinable change.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const top = stdout.toString().replace(/\r?\n$/, '');
/// ```
pub(crate) fn toplevel_of(stdout: &[u8]) -> PathBuf {
    let text: String = String::from_utf8_lossy(stdout).into_owned();
    // Exactly one terminator goes, so a directory whose name ends in a line break keeps it.
    let without_terminator: &str = match text.strip_suffix("\r\n") {
        Some(rest) => rest,
        None => text.strip_suffix('\n').unwrap_or(text.as_str()),
    };
    return PathBuf::from(without_terminator);
}

pub(crate) fn word_pass(
    site: &MergeSite<'_>,
    tip: &str,
    merge: (&str, &[Conflict]),
    paths: Vec<Vec<u8>>,
) -> Merged {
    let (tree, conflicts): (&str, &[Conflict]) = merge;
    // The commit may have been run from a subdirectory; the second pass reads paths from the top of the worktree.
    let mut top_request: GitRequest = GitRequest::new(site.cwd, &["rev-parse", "--show-toplevel"]);
    top_request.without_prefix = true;
    top_request.repository_selection = crate::repository_selection::from_global_options(
        site.cwd,
        site.context.global_prefix.as_slice(),
    );
    let top: PathBuf = match run_git_checked(site.context, &top_request) {
        Ok(found) => toplevel_of(found.stdout.as_slice()),
        Err(_) => return Merged::Conflict(paths),
    };
    let process_environment: Vec<(OsString, OsString)> = std::env::vars_os().collect();
    let variables: Vec<(OsString, OsString)> = environment_with_overlay(
        process_environment.as_slice(),
        site.context.overlay.as_slice(),
    );
    // The replay runs at the top of the worktree and keeps none of the caller's other options, so the repository the line named is
    //  carried here: a Git directory beside the work tree is found only when it is named.
    let repository: Vec<OsString> = crate::repository_selection::from_global_options(
        top.as_path(),
        site.context.global_prefix.as_slice(),
    );
    let probing: ContextRunner<'_> = ContextRunner {
        context: site.context,
        cwd: top.as_path(),
        without_prefix: true,
        repository: repository.clone(),
        object_directory: None,
        environment: Vec::new(),
    };
    let probe: AttributeProbe<'_> = AttributeProbe {
        directory: top.as_path(),
        global_prefix: &[],
        variables: variables.as_slice(),
        runner: &probing,
    };
    let runner: ContextRunner<'_> = ContextRunner {
        context: site.context,
        cwd: top.as_path(),
        without_prefix: true,
        repository,
        object_directory: Some(site.object_directory),
        environment: attribute_tree_variables_on_the_runtime(tip, &probe),
    };
    return match resolve_tree_on_the_runtime(&runner, site.common_dir, tree, conflicts, &probe) {
        Some(resolved) => Merged::Combined {
            tree: resolved.tree,
            files: resolved.merged,
        },
        None => Merged::Conflict(paths),
    };
}
