//! What:
//!  The account's effective delegation over the route's datasets,
//!  including the grants an ancestor carries,
//!  the grants a creation-time entry puts on every dataset the account
//!  creates below it,
//!  the members of a named set the account can come to hold,
//!  and the absence of the lineage-changing grants.
//! Why:
//!  The removal threat boundary rests on the account's grants being the ones
//!  the project decided:
//!  `snapshot` local to the checkout dataset,
//!  `clone` on its descendants,
//!  `create,mount` local to the clone parent,
//!  `mount,userprop,destroy` on the clone parent's descendants,
//!  and `mount,destroy` on the checkout's descendants for origin cleanup,
//!  with `mountpoint`,
//!  `canmount`,
//!  `setuid`,
//!  `devices`,
//!  `rename`,
//!  `promote` and `receive` absent
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Scoped delegation" and
//!  "What doctor checks and prints";
//!  `doc/decision/cli-git-zfs-worktree-design.md`,
//!  "Delegated destroy").
//!  Effective delegation is what an ancestor's descendant grant gives as well
//!  as what the dataset's own record holds,
//!  so a grant the route needs but that stands on an ancestor is reported as
//!  too broad rather than counted as satisfied in the decided shape.
//!  A creation-time grant is effective on datasets that do not exist yet:
//!  the kernel copies it onto every dataset its creator makes below the
//!  dataset that carries it,
//!  as a local grant to that creator
//!  (`module/zfs/dsl_dataset.c`,
//!  `dsl_dataset_create_sync`,
//!  calls `dsl_deleg_set_create_perms` in
//!  `module/zfs/dsl_deleg.c`),
//!  and the account creates the route's clones below the container,
//!  so a forbidden permission there is one the account comes to hold.
//!  A named set's members stand under the set's own who key in the reply,
//!  so a forbidden permission inside a set the account can hold is named
//!  with the `zfs unallow -s` command that removes it.
//!  The kernel's own reply is the one source,
//!  and `delegation_reply.rs` reads it into the grants this check judges:
//!  `ZFS_IOC_GET_FSACL` returns every dataset on the way up that carries a
//!  delegation,
//!  as `{dataset: {who: {name: boolean}}}`
//!  (OpenZFS `zfs-2.4.4`,
//!  `module/zfs/dsl_deleg.c`,
//!  `dsl_deleg_get`,
//!  and `module/zcommon/zfs_deleg.c`,
//!  `zfs_deleg_whokey`).
//!  The grants are a value handed in,
//!  so a control builds the reply it chose and no pool is read.
//!  The check is split across siblings because this file is over the line
//!  limit:
//!  this one judges the whole delegation and answers the permissions it does
//!  not grant,
//!  `delegation_checks_permissions.rs` holds the permissions the design fixes,
//!  `delegation_checks_grant_commands.rs` tells an exact grant from a wider
//!  one and writes the commands that move a grant,
//!  `delegation_checks_narrowing.rs` builds the lines for the wider grants,
//!  and `delegation_checks_forbidden_grants.rs` names the lineage-changing
//!  grants.
//!
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const grants = grantsOf(fsaclReply); // [{ carriedBy, permission, who, inheritance }]
//! const outcome = checkDelegation(facts); // pass | finding('permission'|'grant'|'scope') | unknown
//! ```

/// One check's answer,
/// and the three constructors every check builds it from.
use super::check_outcome::{
    CheckOutcome,
    finding,
    pass,
    unknown,
};
/// The lineage-changing grants and the unproved named sets.
use super::delegation_checks_forbidden_grants::{
    creation_time_forbidden,
    held_set_forbidden,
    is_route_permission,
    unproved_named_sets,
};
use super::delegation_checks_grant_commands::{
    Satisfaction,
    satisfaction,
    unallow_command,
};
/// How one permission was met,
/// and the commands that grant or remove one.
pub use super::delegation_checks_grant_commands::{
    allow_command,
    place_dataset,
};
/// The wider grants a person narrows,
/// and the line that names them.
use super::delegation_checks_narrowing::{
    Narrowing,
    each_line_once,
    narrowings,
    scope_detail,
};
/// The permissions the design requires,
/// forbids and names the place of.
pub use super::delegation_checks_permissions::{
    FORBIDDEN_PERMISSIONS,
    GrantPlace,
    REQUIRED_GRANTS,
    RequiredGrant,
};
/// The grants the kernel's reply became,
/// and the tests that read one.
use super::delegation_reply::{
    DelegationFacts,
    Who,
    grants_the_account,
    names_a_set,
};
/// What:
///  Whether the account's effective delegation is the decided one:
///  every required permission where and as widely as the design says,
///  no lineage-changing grant the account holds or comes to hold,
///  and no route permission granted to everyone or reached through a set.
/// Why:
///  The removal's threat boundary is exactly this set,
///  so the report names each grant that breaks it with the command that
///  removes it,
///  a permission granted wider than the design scopes it gets both the grant
///  in the decided place and the wider grant's removal,
///  and a route permission that reaches the account through a named set leaves
///  the answer unknown with the set,
/// the permission and the command named,
///  because the design grants each permission directly and this check does not
///  prove a delegation that arrives another way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = checkDelegation(facts); // pass | finding('permission'|'grant'|'scope'|'everyone') | unknown
/// ```
pub fn check_delegation(facts: &DelegationFacts) -> CheckOutcome {
    let mut forbidden: Vec<String> = facts
        .grants
        .iter()
        .filter(|grant| {
            return FORBIDDEN_PERMISSIONS.contains(&grant.permission.as_str())
                && (grants_the_account(grant, facts) || matches!(grant.who, Who::Everyone));
        })
        .map(|grant| {
            return format!(
                "`{}` on `{}` to {} ({})",
                grant.permission,
                grant.carried_by,
                grant.who.describe(),
                unallow_command(grant, facts)
            );
        })
        .collect();
    forbidden.extend(creation_time_forbidden(facts));
    forbidden.extend(held_set_forbidden(facts));
    forbidden = each_line_once(&forbidden);
    if !forbidden.is_empty() {
        return finding(
            "permission",
            format!(
                "the account holds or will come to hold {} lineage-changing grant(s) the route must not have: {}",
                forbidden.len(),
                forbidden.join("; ")
            ),
        );
    }
    let everyone: Vec<String> = facts
        .grants
        .iter()
        .filter(|grant| {
            return matches!(grant.who, Who::Everyone)
                && is_route_permission(&grant.permission)
                && !names_a_set(&grant.permission);
        })
        .map(|grant| return format!("`{}` on `{}`", grant.permission, grant.carried_by))
        .collect();
    if !everyone.is_empty() {
        return finding(
            "everyone",
            format!(
                "{} route permission(s) are granted to everyone, which reaches every account on this machine: {}",
                everyone.len(),
                everyone.join("; ")
            ),
        );
    }
    let named_sets: Vec<String> = unproved_named_sets(facts);
    if !named_sets.is_empty() {
        return unknown(format!(
            "{} route permission(s) stand in named sets rather than in the direct grants the design names, so the effective delegation is unproved: {}",
            named_sets.len(),
            named_sets.join("; ")
        ));
    }
    let missing: Vec<String> = REQUIRED_GRANTS
        .iter()
        .filter(|required| return matches!(satisfaction(required, facts), Satisfaction::Missing))
        .map(|required| {
            return format!(
                "`{}` on `{}` ({})",
                required.permission,
                place_dataset(required.place, facts),
                allow_command(required, facts)
            );
        })
        .collect();
    if !missing.is_empty() {
        return finding(
            "grant",
            format!(
                "{} decided permission(s) are not granted to the account: {}",
                missing.len(),
                missing.join("; ")
            ),
        );
    }
    let narrowings: Vec<Narrowing<'_>> = narrowings(facts);
    if !narrowings.is_empty() {
        return finding("scope", scope_detail(&narrowings, facts));
    }
    return pass(format!(
        "{} decided permission(s) are granted where the design puts them and no lineage-changing grant reaches the account",
        REQUIRED_GRANTS.len()
    ));
}

/// What:
///  The decided permissions this delegation does not grant the account,
///  in the check's own order.
/// Why:
///  The add's preflight splits its verdict by which step uses a permission,
///  and the split needs the values rather than the one sentence the check
///  builds,
/// so the missing set is answered on its own
///  (`doc/planning/cli-git-zfs-step-10-open-choices.md`,
///  choice 3C's cost).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const missing = missingPermissions(facts); // RequiredGrant[]
/// ```
pub fn missing_grants(facts: &DelegationFacts) -> Vec<RequiredGrant> {
    return REQUIRED_GRANTS
        .iter()
        .copied()
        .filter(|required| return matches!(satisfaction(required, facts), Satisfaction::Missing))
        .collect();
}

//endregion

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "delegation_checks_tests.rs"]
mod tests;
