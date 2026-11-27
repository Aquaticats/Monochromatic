//! What:
//!  The plain sentence that tells a person why a `git worktree add` was not made as a snapshot,
//! and the places it goes:
//!  one line on standard error unless `--quiet` was given,
//! one warning entry in the journal,
//! and the debug log that already
//!  holds the detail.
//! Why:
//!  A snapshot of the main checkout takes seconds and Git's checkout with the ignored-state copy takes minutes.
//! The choice
//!  between them was made silently,
//! so two slow adds could not be explained afterwards.
//! The sentence names the cause in the
//!  person's words (a change,
//! a running Git command,
//! a stale index,
//! a missing subvolume) and what makes the next add quick,
//!  without the names of the ladder's steps.
//! A filesystem that is not btrfs gets no sentence:
//! nothing the person does there
//!  would make the add a snapshot,
//! so the line would only repeat itself.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const route = explainRefusal({ context, identity, refusal });
//! route?.tell(plan); // stderr unless quiet, and the journal
//! ```

use crate::fs_kind::{
    FilesystemKind,
    filesystem_kind,
    is_subvolume_root,
};
use crate::transaction_git::GitContext;
use crate::worktree_identity::WorktreeIdentity;
use crate::worktree_snapshot_source::{
    Refusal,
    RefusalKind,
    plain,
};
use std::path::Path;

/// The sentence for a source that cannot be shown unchanged.
pub(crate) use crate::worktree_slow_route_unclean::{
    explain_unclean,
    status_failed,
};
/// The value a person reads,
/// and the two leads a line starts with.
pub use crate::worktree_slow_route_words::{
    DECLINED_LEAD,
    LEAD,
    SlowRoute,
};
/// What makes the next add quick,
/// and how the source is called in a sentence.
pub(crate) use crate::worktree_slow_route_words::{
    THEN_QUICK,
    subject,
};

/// What:
///  The sentence for a source the expensive checks refused,
/// or nothing when no sentence would help.
/// Why:
///   The kind of the refusal chooses between the unclean sentence,
/// which reads Git's status,
/// and the plain statement of an
///  unsupported feature.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const route = explainRefusal({ context, identity, refusal });
/// ```
pub fn explain_refusal(
    context: &GitContext,
    identity: &WorktreeIdentity,
    refusal: &Refusal,
) -> Option<SlowRoute> {
    let root: &Path = match identity {
        WorktreeIdentity::MainWorktree {
            worktree_root,
            ..
        }
        | WorktreeIdentity::LinkedWorktree {
            worktree_root,
            ..
        } => worktree_root.as_path(),
        WorktreeIdentity::BareRepository {
            ..
        }
        | WorktreeIdentity::OutsideWorktree => return None,
    };
    return Some(match refusal.kind {
        RefusalKind::Unclean(kind) => explain_unclean(
            context,
            root,
            subject(identity),
            kind,
            refusal.reason.as_str(),
        ),
        RefusalKind::Unsupported => explain_unsupported(refusal.reason.as_str()),
        RefusalKind::StatusFailed => {
            status_failed(subject(identity), root, refusal.reason.as_str())
        }
    });
}

/// What:
///  The sentence for a feature of the source that a snapshot does not reproduce.
/// Why:
///   These reasons are already plain statements ("core.sparseCheckout is set");
/// the sentence adds nothing to them,
/// and names
///  no remedy because the person's choice of feature is not the wrapper's to undo.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const route = explainUnsupported('core.sparseCheckout is set');
/// ```
pub fn explain_unsupported(reason: &str) -> SlowRoute {
    return SlowRoute {
        lead: LEAD,
        word: "unsupported",
        detail: reason.to_string(),
        sentence: format!("{}.", plain(reason)),
    };
}

/// What:
///  The sentence for a clone rung that ended,
///  the conversion fence's decline included.
/// Why:
///   The add is whole,
/// made by Git's own checkout;
/// the sentence names the input and says the add ended after it had read the source,
/// then gives the check's own words,
/// which name the path or the rules and,
/// for the two classes a person can act on,
/// the remedy:
/// `git status` once in the source refreshes its record of the file times,
/// and committing the source's
/// `.gitattributes` change makes the rules the add reads match the ones Git writes with.
/// It names no snapshot and no rung:
/// a person reading it ran an add that ended,
/// and the words are plain.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const route = explainCloneFailure('a line-ending rule ... so this rung declines');
/// ```
pub fn explain_clone_failure(reason: &str) -> SlowRoute {
    return SlowRoute {
        lead: LEAD,
        word: "clone-ended",
        detail: reason.to_string(),
        sentence: format!(
            "the add ended after it had read the source ({}), so Git's own checkout wrote the worktree.",
            plain(reason)
        ),
    };
}

/// What:
///  The sentence for a rung the conversion fence declined before anything was copied.
/// Why:
///   A declined add is no failure:
/// the fence read the attributes,
/// decided that writing that many paths through Git would cost more than Git's checkout,
///  and ended the rung;
/// the sentence is the fence's own words,
///  which name the count and what it read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const route = explainDecline('a line-ending rule ... so this rung declines');
/// ```
pub fn explain_decline(reason: &str) -> SlowRoute {
    return SlowRoute {
        lead: DECLINED_LEAD,
        word: "declines",
        detail: reason.to_string(),
        sentence: format!("{}.", plain(reason)),
    };
}

/// What:
///  The sentence for a snapshot that started and could not be finished.
/// Why:
///   The add is whole,
/// made by Git's checkout;
/// the sentence says why the quick way ended,
/// in the check's own words.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const route = explainFailure('the stage could not be swapped into place');
/// ```
pub fn explain_failure(reason: &str) -> SlowRoute {
    return SlowRoute {
        lead: LEAD,
        word: "snapshot-failed",
        detail: reason.to_string(),
        sentence: format!(
            "the snapshot could not be finished ({}), so Git's own checkout made the worktree.",
            plain(reason)
        ),
    };
}

/// What:
///  The sentence for the cheap refusals (the source is no subvolume,
/// or the two sides are not on one filesystem),
/// or nothing
///  where the filesystem is not btrfs.
/// Why:
///   On a btrfs source that is no subvolume the person can convert it;
/// on another filesystem nothing makes the add a
///  snapshot,
/// and a line on every add would be noise.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const route = explainCheapRefusal({ identity, plan, reason });
/// ```
pub fn explain_cheap_refusal(
    identity: &WorktreeIdentity,
    reason: &str,
) -> Option<SlowRoute> {
    let root: &Path = match identity {
        WorktreeIdentity::MainWorktree {
            worktree_root,
            ..
        }
        | WorktreeIdentity::LinkedWorktree {
            worktree_root,
            ..
        } => worktree_root.as_path(),
        WorktreeIdentity::BareRepository {
            ..
        }
        | WorktreeIdentity::OutsideWorktree => return None,
    };
    if filesystem_kind(root) != Ok(FilesystemKind::Btrfs) {
        return None;
    }
    let subject: &str = subject(identity);
    // The main checkout is migrated with the repository's other worktrees in view; a linked worktree is converted alone.
    let command: &str = if matches!(identity, WorktreeIdentity::MainWorktree { .. }) {
        "migrate-to-subvolume"
    } else {
        "convert-worktree-to-subvolume"
    };
    if !is_subvolume_root(root).unwrap_or(false) {
        return Some(SlowRoute {
            lead: LEAD,
            word: "not-subvolume",
            detail: reason.to_string(),
            sentence: format!(
                "{subject} is not a btrfs subvolume. Run `git monochromatic-git-policy-cli {command}` in it, {THEN_QUICK}."
            ),
        });
    }
    return Some(SlowRoute {
        lead: LEAD,
        word: "other-filesystem",
        detail: reason.to_string(),
        sentence: format!(
            "the new worktree is not on the same btrfs filesystem as {subject}. Put it on the same one, {THEN_QUICK}."
        ),
    });
}

/// Explanation controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_slow_route_tests.rs"]
mod tests;
