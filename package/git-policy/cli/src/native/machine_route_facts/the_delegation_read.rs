//! What:
//!  The account the route serves,
//!  every group it belongs to,
//!  and every grant the kernel answers for the two datasets the design scopes
//!  a delegation over.
//! Why:
//!  The removal threat boundary rests on the account's grants being the nine the
//!  project decided and on no lineage-changing grant reaching it,
//!  and one reply walks a dataset's ancestors,
//!  so two replies cover both places the design grants a permission
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Scoped delegation";
//!  `crate::doctor_zfs_route::delegation_checks`).
//!  The account is this process's own,
//!  which is the one the kernel judges a delegation against and the one an add
//!  reads its grants for,
//!  so a report and a refusal cannot disagree about whose grants they are.
//!  A reply or a group list the kernel refused proves nothing about the
//!  delegation,
//!  and the check would otherwise report the decided permissions as missing,
//!  so a failed read answers that check an unknown naming what was refused.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const delegation = theDelegationFacts({ layout, pool, unread }); // { account, groups, checkoutDataset, containerDataset, grants }
//! ```

/// The pool the delegation replies come from.
use super::the_machines_pool_answers::TheMachinesPool;
/// The unknown a read that failed answers.
use super::the_unknown_a_failed_read_answers::the_unknown_of;
/// One check's stable name,
/// and the report line a failed read answers.
use crate::doctor_zfs_route::check_outcome::{
    Check,
    CheckReport,
};
/// Everything the delegation check reads.
use crate::doctor_zfs_route::delegation_reply::{
    DelegationFacts,
    Grant,
};
/// The account this process runs as,
/// and every group it belongs to,
/// read the way the add's own preflight reads them.
use crate::worktree_clone_add_route::add_grants::{
    groups_of_this_account,
    this_account,
};
/// The three names the route's layout holds.
use crate::worktree_clone_mount_handler::sweep_configuration::SweepConfiguration;

/// What:
///  The account's effective delegation over the checkout dataset and the clone
///  container,
///  with the account and its groups the kernel reports.
/// Why:
///  A grant to a group reaches the account only through a group it holds,
///  so the ids the kernel names are part of the facts and not a detail of
///  reading them,
///  and a group list nobody could read leaves every group's grant unproved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const facts = theDelegationFacts({ layout, pool, unread });
/// ```
pub(crate) fn the_delegation_facts(
    layout: &SweepConfiguration,
    pool: &dyn TheMachinesPool,
    unread: &mut Vec<CheckReport>,
) -> DelegationFacts {
    let checkout_dataset: String = String::from(layout.checkout_dataset.as_str());
    let container_dataset: String = String::from(layout.parent_dataset.as_str());
    let groups: Vec<u32> = match groups_of_this_account() {
        Ok(held) => held,
        Err(refusal) => {
            unread.push(the_unknown_of(Check::Delegation, refusal.message.as_str()));
            Vec::new()
        }
    };
    let mut grants: Vec<Grant> = Vec::new();
    for dataset in [checkout_dataset.as_str(), container_dataset.as_str()] {
        match pool.the_grants_at(dataset) {
            Ok(answered) => grants.extend(answered),
            Err(why) => {
                unread.push(the_unknown_of(Check::Delegation, why.as_str()));
            }
        }
    }
    return DelegationFacts {
        account: this_account(),
        groups,
        checkout_dataset,
        container_dataset,
        grants,
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "the_delegation_read_tests.rs"]
mod tests;
