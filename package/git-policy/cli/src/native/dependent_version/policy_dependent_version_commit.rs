//! What:
//!  The work the commit path of `mono/dependent-version-bump` does around the plan:
//!  the notes
//!       it leaves,
//!  the real-index entries it reads,
//!  and the correction it applies for one bump.
//! Why:
//!  A commit adds the dependents' manifests to itself
//!  (owner answer of 2026-11-10,
//!  "A commit adds the dependents'
//!       manifests and updates them in the worktree when that copy still matches `HEAD`"),
//!  so it needs the
//!      checks a direct fix makes,
//!  and a reporting channel of its own for the dependents it may
//!      not touch:
//!  a note never stops the command.
//!  Moving them here keeps the check under the
//!      folder's line limit without hiding them in it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const left = await committedCorrection({ bump, selected, workspace, root, real });
//! ```

/// The modes and object names a correction keeps and compares.
use crate::candidate_object::CandidateMode;
/// The planner's modes,
///  failures and messages.
use crate::dependent_version_content::{TrackedMode, display_path};
/// The planner's view of the candidate state.
use crate::dependent_version_lifecycle::LifecycleWorkspace;
/// The planner's bump of one manifest.
use crate::dependent_version_plan::ManifestBump;
/// The planner's findings.
use crate::dependent_version_policy::DependentFinding;
/// The candidate and version types.
use crate::candidate_version::CandidateVersion;
/// The event pathname type a note carries.
use crate::event_path::EventPath;
/// The finding,
///  note and outcome types of a check.
use crate::policy_engine::{PolicyNote, PolicyOutcome};
/// The precondition for a path a lifecycle did not select,
///  its reasons and its messages.
use crate::policy_added_path::{
    ADDED_PATH_STAGED_REASON, ADDED_PATH_UNSTAGED_REASON, CommitAddedPath, Entry, commit_added_path_message,
    commit_refusal, real_index_entries,
};
/// The lifecycle's correction.
use crate::policy_content::Correction;
/// The facts interface the real index is listed through.
use crate::repository_facts::RepositoryFacts;
/// `BTreeMap<K, V>` is a map kept sorted by key.
use std::collections::BTreeMap;
/// Borrowed filesystem paths.
use std::path::Path;
/// `Rc<T>` is a shared,
///  read-only handle.
use std::rc::Rc;

/// What:
///  The outcome of a check that decided nothing and says why,
///  without stopping the
///       command.
/// Why:
///   On the commit path a planner that cannot decide must cost the commit its ripple and
///       never its landing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const noted = (message) => ({ kind: 'findings-with-notes', findings: [], notes: [{ message }] });
/// ```
pub fn noted(message: &str) -> PolicyOutcome {
    return PolicyOutcome::FindingsWithNotes {
        findings: Vec::new(),
        notes: vec![note_of(message, None, Some(crate::policy_events::LEFT_ALONE_HINT))],
    };
}

/// What:
///  One note for the person,
///  about `path` when one is named,
///  under `hint`.
/// Why:
///   A note always names its file in the event's own field,
///  never inside the message,
///  and states what the person can do about it:
/// a copy somebody holds and a read that failed need different words.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const noteOf = (message, path, hint) => ({ message, path: path && eventPath(path), hint });
/// ```
pub fn note_of(message: &str, path: Option<&[u8]>, hint: Option<&'static str>) -> PolicyNote {
    return PolicyNote {
        message: String::from(message),
        path: path.map(EventPath::from_git_bytes),
        hint,
    };
}

/// What:
/// Why a commit did not add one dependent's manifest to itself.
/// Why:
///   A copy that holds work the commit does not carry,
///  a copy that could not be read,
///  and a real
///       index that could not be listed are all reasons to leave the file exactly as it is,
///  and
///       all three are reported.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LeftBehind = { kind: 'holds'; reason: string } | { kind: 'unreadable'; message: string } | { kind: 'unlisted'; message: string };
/// ```
pub enum LeftBehind {
    /// A copy of the manifest holds what the commit does not carry.
    Holds(&'static str),
    /// A copy of the manifest could not be read,
    ///  so nothing is known about it.
    Unreadable(String),
    /// The real index could not be listed,
    ///  so nothing is known about the copy either.
    Unlisted(String),
}

/// What:
///  The note for a dependent the commit left out,
///  naming the file,
///  the version it keeps
///  and the reason.
/// Why:
///   The person must know which dependent stayed behind and what to do about it,
///  and
///       the two ways that happened need different words:
/// a copy that holds somebody's work asks for that
///       file to be committed or restored,
///  while a real index that could not be listed asks for the
///       read to answer first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const leftBehind = (bump, left) => noteOf(hintOf(left), messageOf(left), bump.path);
/// ```
pub fn left_behind(bump: &ManifestBump, left: &LeftBehind) -> PolicyNote {
    let shown: String = display_path(bump.path.as_slice());
    let message: String = match left {
        LeftBehind::Holds(reason) => commit_added_path_message(
            shown.as_str(),
            bump.planned.name.as_str(),
            bump.planned.bump.from.as_str(),
            bump.planned.bump.to.as_str(),
            reason,
        ),
        LeftBehind::Unreadable(error) => format!(
            "mono/dependent-version-bump left {} at {} and out of this commit: a copy of {shown} could not be read ({error}), \
             so nothing was written over it. Check that the file can be read, then commit again.",
            bump.planned.name, bump.planned.bump.from
        ),
        LeftBehind::Unlisted(error) => format!(
            "mono/dependent-version-bump left {} at {} and out of this commit: the real index could not be listed \
             ({error}), so whether {shown} still holds HEAD's version is unknown. Run git ls-files --stage in that \
             repository to see why Git cannot answer, then commit again.",
            bump.planned.name, bump.planned.bump.from
        ),
    };
    let hint: Option<&'static str> = match left {
        LeftBehind::Unlisted(_) => Some(crate::policy_events::UNPROVEN_RIPPLE_HINT),
        LeftBehind::Holds(_) | LeftBehind::Unreadable(_) => Some(crate::policy_events::LEFT_ALONE_HINT),
    };
    return note_of(message.as_str(), Some(bump.path.as_slice()), hint);
}

/// What:
///  The real index's entries for the manifests this commit would add to itself,
///  or why the one
///       listing that reads them failed.
/// Why:
///   A commit never fails over the ripple,
///  so a listing that could not run has to reach the
///       report:
///  carried beside the entries it stands for,
///  it names every dependent it left
///       unproved,
///  where an empty map alone would read as an index that holds nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RealIndexListing = { kind: 'listed'; entries: Map<string, Entry> } | { kind: 'unlisted'; message: string };
/// ```
pub enum RealIndexListing {
    /// The stage-0 entry of each path the real index holds;
    ///  a path it does not hold is absent.
    Listed(BTreeMap<Vec<u8>, Entry>),
    /// The listing failed;
    ///  the message is Git's own.
    Unlisted(String),
}

/// What:
///  The real index's entries for the manifests this commit would add to itself,
///  by
///       pathname,
///  or why they could not be read.
///  Empty for every lifecycle that adds no path,
///  and for a
///       commit whose findings all name paths it selected.
/// Why:
///   One Git start lists every added dependent,
///  and only a commit that adds one reaches
///       it at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function addedEntries({ plansCommit, findings, version, facts, root, realIndex }): Promise<RealIndexListing>;
/// ```
pub fn added_entries<F: RepositoryFacts>(
    plans_commit: bool,
    findings: &[DependentFinding],
    version: &CandidateVersion,
    facts: &mut F,
    root: Option<&Path>,
    real_index: Option<&Path>,
) -> RealIndexListing {
    let mut paths: Vec<Vec<u8>> = Vec::new();
    if plans_commit {
        for finding in findings {
            if let Some(bump) = &finding.patch
                && version.candidate_at_path(&bump.path).is_none()
            {
                paths.push(bump.path.clone());
            }
        }
    }
    let (Some(top_level), Some(index)) = (root, real_index) else {
        return RealIndexListing::Listed(BTreeMap::new());
    };
    if paths.is_empty() {
        return RealIndexListing::Listed(BTreeMap::new());
    }
    return match real_index_entries(&facts.transaction_context(), top_level, index, paths.as_slice()) {
        Ok(entries) => RealIndexListing::Listed(entries),
        Err(message) => RealIndexListing::Unlisted(message),
    };
}

/// What:
///  The correction a commit applies for one planned bump,
///  or why it left the manifest
///       alone.
/// Why:
///   A manifest the commit already changes is corrected over its candidate bytes;
///  a
///       manifest it does not change is added only while every copy of it holds the file
///       the
///       bump was computed against.
///  A listing that never ran proves none of them,
///  so every
///       manifest it would have proved is left behind with that reason
///       (`RealIndexListing`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function committedCorrection({ bump, selected, workspace, root, real }): Promise<Correction | LeftBehind>;
/// ```
pub fn committed_correction(
    bump: &ManifestBump,
    selected: bool,
    workspace: &mut LifecycleWorkspace<'_>,
    root: Option<&Path>,
    real: &RealIndexListing,
) -> Result<Correction, LeftBehind> {
    let mode: CandidateMode = if bump.mode == TrackedMode::Executable {
        CandidateMode::Executable
    } else {
        CandidateMode::Regular
    };
    let correction: Correction = Correction {
        path: bump.path.clone(),
        mode,
        before: Rc::from(bump.original.as_slice()),
        after: Rc::from(bump.replacement.as_slice()),
    };
    if selected {
        return Ok(correction);
    }
    let entries: &BTreeMap<Vec<u8>, Entry> = match real {
        RealIndexListing::Listed(found) => found,
        RealIndexListing::Unlisted(message) => return Err(LeftBehind::Unlisted(message.clone())),
    };
    // The commit index holds the bytes the bump was computed against; the other three copies must hold the same file.
    let Some(top_level) = root else {
        return Err(LeftBehind::Holds(ADDED_PATH_UNSTAGED_REASON));
    };
    let commit: Entry = match workspace.index_entry(&bump.path) {
        Ok(Some(found)) => found,
        Ok(None) => return Err(LeftBehind::Holds(ADDED_PATH_STAGED_REASON)),
        Err(error) => return Err(LeftBehind::Unreadable(error.reason)),
    };
    let head: Option<Entry> = match workspace.head_entry(&bump.path) {
        Ok(found) => found,
        Err(error) => return Err(LeftBehind::Unreadable(error.reason)),
    };
    let added: CommitAddedPath<'_> = CommitAddedPath {
        path: bump.path.as_slice(),
        head,
        commit,
        real: entries.get(&bump.path).cloned(),
        original: bump.original.as_slice(),
        root: top_level,
    };
    return match commit_refusal(&added) {
        Ok(None) => Ok(correction),
        Ok(Some(reason)) => Err(LeftBehind::Holds(reason)),
        Err(error) => Err(LeftBehind::Unreadable(error.to_string())),
    };
}
