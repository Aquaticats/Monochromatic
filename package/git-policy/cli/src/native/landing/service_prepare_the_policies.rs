//! What:
//!  The policies a preparation runs over the commit's own index:
//!  the checks and their facts,
//!  the convergence when the selection leaves them free to change files,
//!  and the finding a refusal of the person's own selection ends as.
//! Why:
//!   A selection the person made leaves the policies read-only,
//!  so the commit is exactly what was chosen,
//!  while a commit of the whole index lets them correct the files and say how many passes that took;
//!  neither path writes the live index,
//!  and both hand back what the copies of the worktree records are made from.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const [private, pass, changedPasses, changedPaths] = thePolicies(invocation, checks, request, setup, candidatePaths);
//! ```

use super::service_prepare_what_it_reads::Preparation;
use crate::commit_convergence::{
    CommitConvergence,
    CommitConvergenceInput,
    converge_commit_policies,
};
use crate::commit_facts::PrivateIndexFacts;
use crate::commit_transaction::{
    CommitInvocation,
    TransactionStop,
    base_object,
    private_checks,
    settled_passes,
    staged_at_base,
};
use crate::policy_checks::ShippedChecks;
use crate::policy_engine::StageRequest;
use crate::policy_pass::{
    PassResult,
    run_policy_pass,
};
use crate::private_index_stage::PrivateStage;
use crate::repository_facts::RepositoryFacts;
use crate::transaction_git::GitContext;
use std::path::Path;

/// What:
///  Runs the policies over the commit's own index,
///  and answers with the checks the worktree records are read through,
///  what the pass came to,
///  how many correction passes changed files,
///  and the paths those passes changed.
/// Why:
///   The caller needs the checks' sanitized content for the worktree records,
///  and needs to know the moment a policy refused the commit,
///  which `pass_exit_code` says and the caller turns into an ending.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function thePolicies(invocation, checks, request, setup, candidatePaths): [ShippedChecks, PassResult, number, Uint8Array[]]
/// ```
pub(crate) fn the_policies<'a, F: RepositoryFacts>(
    invocation: &CommitInvocation<'_>,
    checks: &'a mut ShippedChecks<F>,
    request: &StageRequest,
    setup: &Preparation,
    candidate_paths: &[Vec<u8>],
) -> Result<
    (
        ShippedChecks<PrivateIndexFacts<&'a mut F>>,
        PassResult,
        u64,
        Vec<Vec<u8>>,
    ),
    TransactionStop,
> {
    let context: &GitContext = invocation.context;
    let cwd: &Path = invocation.cwd;
    let mut private: ShippedChecks<PrivateIndexFacts<&mut F>> = private_checks(
        checks,
        setup.indexes.commit_index.as_path(),
        setup.indexes.object_directory.as_path(),
        setup.capture.layout.real_index_path.as_path(),
        staged_at_base(base_object(&setup.capture)?, candidate_paths),
    );
    // A selection the person made leaves the policies read-only: they report, and the commit is exactly what was chosen.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const [pass, changedPasses, changedPaths] = setup.selection.readOnly ? [runPolicyPass(request, private), 0, []] : converge(...);
    // ```
    let (pass, changed_passes, changed_paths): (PassResult, u64, Vec<Vec<u8>>) =
        if setup.selection.read_only {
            private.can_apply_patches = false;
            (run_policy_pass(request, &mut private), 0, Vec::new())
        } else {
            let outcome: CommitConvergence = converge_commit_policies(CommitConvergenceInput {
                request,
                checks: &mut private,
                stage: PrivateStage {
                    context,
                    cwd,
                    index: setup.indexes.commit_index.as_path(),
                    object_directory: Some(setup.indexes.object_directory.as_path()),
                },
            });
            let passes: u64 = settled_passes(&outcome.convergence);
            (outcome.pass, passes, outcome.changed_paths)
        };
    // What: `(private, pass, changed_passes, changed_paths)` hands the caller the checks, the pass and what the passes changed.
    // Why: the worktree records are read out of the checks' content, and the exit code decides whether the commit goes on.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return [private, pass, changedPasses, changedPaths];
    // ```
    return Ok((private, pass, changed_passes, changed_paths));
}
