//! What:
//!  Creates the shadow repository of one published transaction:
//!       an empty bare repository,
//!       its config and alternates,
//!       the linked shared entries,
//!       the conclusion state,
//!       and the ref snapshot.
//! Why:
//!  The shadow is a separate repository rather than a registered worktree,
//!  so `git worktree list` never shows it and preparation creates no ref in the real repository.
//!  Its object store receives the objects native `git commit` writes,
//!  and its alternates name the real object directory,
//!  so a real `git prune` cannot delete a pending commit.
//!  The private files it makes and the facts it is made from are in sibling modules,
//!  re-exported here under the paths they had,
//!  and the checks and parts that belong to one commit alone are in
//!  `shadow_create_for_one_commit.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await createShadowRepository({ gitPath, cwd, capture, transactionId, transactionDirectory });
//! ```

/// Invocation capture.
use crate::capture_invocation::InvocationCapture;
/// The build phase's stretches,
/// which the creation marks as it goes.
use crate::commit_build_part_durations::BuildPartClock;
/// The checks and parts that belong to one commit alone,
/// re-exported under the path they had in this file.
pub(crate) use crate::shadow_create_for_one_commit::may_be_handed_a_tree;
use crate::shadow_create_for_one_commit::{
    carry_the_commit_parts,
    conclusion_state_present,
};
/// The number of those starts,
/// and what a start the ceiling ended or the system refused was,
/// re-exported under the paths they had in this file;
/// only the controls of this module name them.
#[cfg(test)]
pub(crate) use crate::shadow_create_init::{
    SHADOW_CREATION_ATTEMPTS,
    start_was_cut_or_refused,
};
/// The private parent of a shadow.
use crate::shadow_create_private_storage::create_parent;
/// The private directory a shadow is made in,
/// re-exported under the path it had in this file.
pub use crate::shadow_create_private_storage::ensure_private_directory;
/// The facts a shadow is made from,
/// re-exported under the path it had in this file.
pub use crate::shadow_create_template::ShadowTemplate;
/// The creation of everything but the commit's own parts,
/// and the failure text of one step of it.
use crate::shadow_create_the_commit_free_parts::{
    create_shadow_template,
    step_failure,
};
/// Shadow path.
use crate::shadow_git::shadow_repository_path;
/// What a tree must be for this invocation,
/// which is the question a claim and a hand-out both ask.
use crate::shadow_pool_member::refusal;
/// Process context and the failure of a step.
use crate::transaction_git::{
    GitContext,
    GitFailure,
};
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  Creates the shadow repository of one published transaction.
/// Why:
///   Every step runs before any native Git command reads the shadow,
///  so the first read already sees the real config,
/// objects,
/// links and refs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await createShadowRepository({ gitPath, cwd, capture, transactionId, transactionDirectory });
/// ```
pub fn create_shadow_repository(
    context: &GitContext,
    cwd: &Path,
    capture: &InvocationCapture,
    transaction_id: &str,
    transaction_directory: &Path,
) -> Result<PathBuf, GitFailure> {
    let shadow: PathBuf =
        shadow_repository_path(capture.layout.common_dir.as_path(), transaction_id);
    // No Git child runs here, so the stretch is named for the record a commit writes.
    let mut clock: BuildPartClock = BuildPartClock::begin("shadow parent");
    step_failure(
        create_parent(shadow.as_path()),
        "creating",
        shadow.as_path(),
    )?;
    create_shadow_repository_at(
        context,
        cwd,
        capture,
        shadow.as_path(),
        transaction_directory,
        &mut clock,
    )?;
    return Ok(shadow);
}

/// What:
///  Creates a shadow repository at a given path,
/// whose parent directory exists.
/// Why:
///   The commit service builds the commits Git must build (an editor,
/// a signature) in a scratch repository inside its own
///  scratch directory,
/// away from the registry directory that recovery scans,
/// so no other process mistakes it for a dead
///  transaction's shadow.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await createShadowRepositoryAt({ gitPath, cwd, capture, shadowPath, transactionDirectory });
/// ```
pub fn create_shadow_repository_at(
    context: &GitContext,
    cwd: &Path,
    capture: &InvocationCapture,
    shadow: &Path,
    transaction_directory: &Path,
    clock: &mut BuildPartClock,
) -> Result<(), GitFailure> {
    return create_shadow_repository_carrying(
        context,
        cwd,
        capture,
        shadow,
        transaction_directory,
        true,
        clock,
    );
}

/// What:
///  `create_shadow_repository_at`,
/// with the merge,
/// cherry-pick and revert conclusion state of the worktree either copied into
///  the scratch repository or left out of it.
/// Why:
///   A commit of named paths while another session's merge is unfinished must build as an ordinary commit of those paths on
///  `HEAD` (real Git refuses it as a partial commit during a merge);
/// a scratch repository that holds no `MERGE_HEAD` builds
///  exactly that,
/// and the landing then leaves the worktree's own conclusion state alone because the scratch repository
///  never held it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await createShadowRepositoryCarrying({ gitPath, cwd, capture, shadowPath, transactionDirectory, carryConclusion });
/// ```
pub fn create_shadow_repository_carrying(
    context: &GitContext,
    cwd: &Path,
    capture: &InvocationCapture,
    shadow: &Path,
    transaction_directory: &Path,
    carry_conclusion: bool,
    clock: &mut BuildPartClock,
) -> Result<(), GitFailure> {
    let template: ShadowTemplate = ShadowTemplate::of(context, &capture.layout)?;
    create_shadow_template(context, &template, shadow, clock)?;
    return carry_the_commit_parts(
        context,
        cwd,
        capture,
        shadow,
        transaction_directory,
        carry_conclusion,
        clock,
    );
}

/// What:
///  The shadow one commit was given,
///  and the finished one the service drew ahead of it when that is where it came from.
/// Why:
///   A drawn tree is used where it stands and is not the commit's own,
///  so the value that keeps it renewed,
///  releases it,
///  and tells the request which tree to name travels with the path:
///  a caller that took the path alone would leave a tree nothing renews.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ShadowTree = { path: string; handed?: HandedShadow };
/// ```
pub struct ShadowTree {
    /// Where the shadow is.
    pub path: PathBuf,
    /// The drawn tree the service handed this commit,
    /// which is renewed while the commit lives and released when the service does not take it;
    /// nothing when the shadow is inside the commit's own scratch tree.
    pub handed: Option<crate::shadow_drawn::HandedShadow>,
}

/// What:
///  The shadow of one commit:
/// a finished one the service drew ahead and handed over,
/// a finished one this invocation can use from the pool,
/// or one made here when neither is there.
/// Why:
///   The creation is about 40 calls that create a directory entry on the volume the commit already waits on:
/// ten `mkdir`,
///  two file writes and three mode changes,
/// one link per shared entry,
/// one directory for the conclusion copies,
///  and the ref store.
/// A commit that takes a finished member makes one `rename` instead,
/// and a commit the pool cannot serve
///  makes the creation exactly as it did before
/// (`doc/planning/cli-git-commit-latency-under-many-agents.md`,
/// "What each part of a commit's private repository is for,
/// and how many commits need it").
///   The ref store and the conclusion state are always this commit's,
/// after a claim as well as after a creation:
/// they name the branch and base of this invocation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await takeOrCreateShadow({ context, cwd, capture, shadow, transactionDirectory, carryConclusion, clock });
/// ```
pub fn take_or_create_shadow(
    context: &GitContext,
    cwd: &Path,
    capture: &InvocationCapture,
    shadow: &Path,
    transaction_directory: &Path,
    carry_conclusion: bool,
    clock: &mut BuildPartClock,
    service: &crate::service_server_socket_binding::Bind,
) -> Result<ShadowTree, GitFailure> {
    let template: ShadowTemplate = ShadowTemplate::of(context, &capture.layout)?;
    // What: The commit asks whether the service drew a finished shadow ahead of it, and where it is.
    // Why:  The one write a claim makes is the call that waited seconds on this volume, and the service made it before
    //       the commit asked. Every check a claim runs is still run here, on the tree the commit was handed, and a tree
    //       this invocation refuses is handed back and the commit takes the pool's member or makes its own: nothing about
    //       the question can fail a commit.
    clock.next("shadow hand-out");
    let concludes: bool =
        carry_conclusion && conclusion_state_present(capture.layout.git_dir.as_path());
    let asked: Option<PathBuf> = match may_be_handed_a_tree(&template, concludes) {
        true => crate::service_client_shadow_hand_out::ask_for_a_drawn_shadow(
            service,
            &template,
            transaction_directory,
        ),
        false => None,
    };
    if let Some(drawn) = asked {
        match refusal(&template, drawn.as_path()) {
            Ok(None) => {
                let handed = crate::shadow_drawn::HandedShadow::hold(drawn.clone());
                carry_the_commit_parts(
                    context,
                    cwd,
                    capture,
                    drawn.as_path(),
                    transaction_directory,
                    carry_conclusion,
                    clock,
                )?;
                return Ok(ShadowTree {
                    path: drawn,
                    handed: Some(handed),
                });
            }
            Ok(Some(reason)) => crate::diagnostic_log::debug(
                "shadowDrawn",
                format!("refused the drawn shadow {}: {reason}", drawn.display()).as_str(),
            ),
            Err(error) => crate::diagnostic_log::debug(
                "shadowDrawn",
                format!("refused the drawn shadow {}: {error}", drawn.display()).as_str(),
            ),
        }
        // A tree this invocation cannot use is the service's own, and this commit is the last thing that held it.
        crate::service_scratch_release_removal::start(drawn);
    }
    clock.next("shadow pool");
    if crate::shadow_pool::take(&template, shadow) {
        carry_the_commit_parts(
            context,
            cwd,
            capture,
            shadow,
            transaction_directory,
            carry_conclusion,
            clock,
        )?;
        return Ok(ShadowTree {
            path: shadow.to_path_buf(),
            handed: None,
        });
    }
    clock.next("shadow skeleton");
    create_shadow_template(context, &template, shadow, clock)?;
    carry_the_commit_parts(
        context,
        cwd,
        capture,
        shadow,
        transaction_directory,
        carry_conclusion,
        clock,
    )?;
    return Ok(ShadowTree {
        path: shadow.to_path_buf(),
        handed: None,
    });
}

/// Shadow creation controls stay out of the release executable.
#[cfg(test)]
#[path = "shadow_create_tests.rs"]
mod tests;
