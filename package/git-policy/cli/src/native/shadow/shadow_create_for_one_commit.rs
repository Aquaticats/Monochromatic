//! What:
//!  The parts of a shadow that belong to one commit alone:
//! whether a finished tree may be handed to the commit,
//! whether its worktree holds a conclusion its commit concludes,
//! and the conclusion state and ref store the commit writes into the shadow once the tree is its own.
//! Why:
//!  A commit takes a finished tree,
//! a pooled member or makes its own,
//! and a member of the pool is made before any commit names a branch,
//! so what names this invocation's branch and base runs after the claim as it runs after a creation.
//! The checks that decide whether a drawn tree may be asked for are the commit's own,
//! not the service's.
//!  Kept apart from the creation so both stay under the file limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! mayBeHandedATree(template, carryConclusion); conclusionStatePresent(gitDir); carryTheCommitParts({ ... });
//! ```

/// Invocation capture.
use crate::capture_invocation::InvocationCapture;
/// The build phase's stretches,
/// which the carried parts are named under.
use crate::commit_build_part_durations::BuildPartClock;
/// The conclusion state the commit copies into its shadow.
use crate::conclusion_copy::copy_conclusion_state;
/// The facts a shadow is made from.
use crate::shadow_create_template::ShadowTemplate;
/// The shadow ref store's writer,
/// and the refs it is written from.
use crate::shadow_ref_write::write_shadow_refs;
use crate::shadow_refs::SnapshotSources;
/// Process context and the failure of a step.
use crate::transaction_git::{
    GitContext,
    GitFailure,
};
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// What:
///  Whether the worktree holds the state a conclusion leaves,
///  which is the state a commit may conclude.
/// Why:
///   The landing that concludes compares what Git removed in the shadow with the copy of those files,
///  and it reads the shadow at the path the commit's own scratch tree holds,
///  so the hand-out is bounded by the presence of the state itself and not by the caller's wish to carry it:
///  every explicit-path commit asks to carry it,
/// and almost none of them has any
///  (`doc/planning/git-policy-cli-shadow-pool-claim-rename-options.md`,
///  "The design the owner's decision made").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const conclusionStatePresent = (gitDir) => REMOVED_CONCLUSION_FILES.some((name) => exists(join(gitDir, name)));
/// ```
pub(super) fn conclusion_state_present(git_dir: &Path) -> bool {
    for name in crate::conclusion_cleanup::REMOVED_CONCLUSION_FILES {
        if git_dir.join(name).exists() {
            return true;
        }
    }
    return false;
}

/// What:
///  Whether this attempt may ask the service for a drawn shadow.
/// Why:
///   Three cases the hand-out cannot serve,
/// each of which keeps the commit on the path it knows:
///  a reftable repository's shadow is Git's own tables,
///  a linked worktree that keeps its own `info` builds its own directory,
///  and both are never drawn for,
/// so asking would be one round trip for nothing.
///  A commit that concludes a merge,
///  a cherry-pick or a revert copies the state that conclusion leaves and compares it with what Git removed in the
///  shadow,
///  reading the shadow at the path its own scratch tree holds,
///  so it takes a member or makes its creation until that comparison names the tree the request built in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const mayBeHandedATree = (template, carryConclusion) => template.refFormat !== 'reftable' && !keepsItsOwnInfo(template) && !carryConclusion;
/// ```
pub(crate) fn may_be_handed_a_tree(
    template: &ShadowTemplate,
    carry_conclusion: bool,
) -> bool {
    if carry_conclusion {
        return false;
    }
    if template.ref_format == crate::transaction_journal::RefFormat::Reftable {
        return false;
    }
    return !crate::shadow_links::keeps_its_own_info(
        template.common_dir.as_path(),
        template.git_dir.as_path(),
    )
    .unwrap_or(false);
}

/// What:
///  Adds what belongs to this commit alone:
/// the conclusion state it concludes and the ref store it points at.
/// Why:
///   A member of the pool is made before any commit names a branch,
/// so these two run after the claim as they run after a creation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await carryTheCommitParts({ context, cwd, capture, shadow, transactionDirectory, carryConclusion, clock });
/// ```
pub(super) fn carry_the_commit_parts(
    context: &GitContext,
    cwd: &Path,
    capture: &InvocationCapture,
    shadow: &Path,
    transaction_directory: &Path,
    carry_conclusion: bool,
    clock: &mut BuildPartClock,
) -> Result<(), GitFailure> {
    let layout = &capture.layout;
    clock.next("shadow conclusion");
    if carry_conclusion {
        copy_conclusion_state(
            context,
            cwd,
            layout.git_dir.as_path(),
            shadow,
            transaction_directory,
            layout.ref_format,
        )?;
    }
    clock.next("shadow refs");
    write_shadow_refs(
        context,
        cwd,
        shadow,
        &SnapshotSources::of(
            layout.git_dir.clone(),
            layout.common_dir.clone(),
            layout.ref_format,
            layout.object_format.as_str(),
        ),
        &layout.target_ref,
        &layout.symbolic_head,
        &capture.base,
    )?;
    return Ok(());
}
