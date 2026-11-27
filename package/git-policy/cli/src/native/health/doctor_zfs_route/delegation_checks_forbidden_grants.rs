//! What:
//!  The lineage-changing grants the account holds,
//!  the ones it would come to hold through a creation-time entry,
//!  the ones a named set it can hold carries,
//!  and the route permissions that stand in named sets rather than in a direct
//!  grant.
//! Why:
//!  The removal's threat boundary is exactly this set,
//!  so each grant is named with the command that removes it:
//!  a creation-time permission is copied onto every dataset its creator makes
//!  below the dataset that carries it,
//!  a set's member reaches whoever holds the set,
//!  and a permission granted through a set is not the direct grant the design
//!  names,
//!  which leaves the effective delegation unproved rather than passed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const forbidden = creationTimeForbidden(facts).concat(heldSetForbidden(facts));
//! const unproved = unprovedNamedSets(facts);
//! ```

/// The command that removes one grant.
use super::delegation_checks_grant_commands::unallow_command;
/// The lines in the order they were built and each printed once.
use super::delegation_checks_narrowing::each_line_once;
/// The permissions the design requires,
/// forbids and recognizes.
use super::delegation_checks_permissions::{
    FORBIDDEN_PERMISSIONS,
    REQUIRED_GRANTS,
};
/// The grants the kernel's reply became,
/// and the sets one reading of it found.
use super::delegation_reply::{
    DelegationFacts,
    HeldSet,
    Who,
    held_sets,
    names_a_set,
    set_memberships,
};
/// The grants of a route permission that reach the account through the
/// required places,
/// used to recognize a permission the check must look at.
pub(super) fn is_route_permission(permission: &str) -> bool {
    if FORBIDDEN_PERMISSIONS.contains(&permission) {
        return true;
    }
    return REQUIRED_GRANTS
        .iter()
        .any(|required| return required.permission == permission);
}

/// The create-time grants of a forbidden permission,
/// which the kernel copies onto every dataset their creator makes below the
/// dataset that carries them.
pub(super) fn creation_time_forbidden(facts: &DelegationFacts) -> Vec<String> {
    return facts
        .grants
        .iter()
        .filter(|grant| {
            return matches!(grant.who, Who::Create)
                && !names_a_set(&grant.permission)
                && FORBIDDEN_PERMISSIONS.contains(&grant.permission.as_str());
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
}

/// What:
///  The forbidden permissions a named set the account can come to hold carries,
///  each naming the set,
/// the holder and the command that takes the permission
///  out of the set.
/// Why:
///  A set's member is a grant the account holds through the holder that passes
///  the set on,
///  so it is judged exactly as a direct grant of the same permission.
pub(super) fn held_set_forbidden(facts: &DelegationFacts) -> Vec<String> {
    return set_memberships(facts)
        .into_iter()
        .filter(|membership| {
            return FORBIDDEN_PERMISSIONS.contains(&membership.member.permission.as_str());
        })
        .map(|membership| {
            return format!(
                "`{}` on `{}` in {}, held by {} ({})",
                membership.member.permission,
                membership.member.carried_by,
                membership.member.who.describe(),
                membership.held.held_by(),
                unallow_command(membership.member, facts)
            );
        })
        .collect();
}

/// What:
///  The route permissions that stand in a named set rather than in a direct
///  grant,
///  each naming the set,
/// who holds it and the command that takes the
///  permission out of it.
/// Why:
///  The design grants each required permission to the account itself,
///  so a permission that reaches the account through a set is not the shape
///  the design decides,
///  and a set nobody in the reply holds still leaves the route's permissions
///  unproved because the check cannot say what a later grant of it would do.
pub(super) fn unproved_named_sets(facts: &DelegationFacts) -> Vec<String> {
    let held: Vec<HeldSet<'_>> = held_sets(facts);
    let mut entries: Vec<String> = Vec::new();
    for definition in &facts.grants {
        let Who::NamedSet(name) = &definition.who else {
            continue;
        };
        if !is_route_permission(&definition.permission) || names_a_set(&definition.permission) {
            continue;
        }
        let holder: String = match held.iter().find(|set| return set.name == name.as_str()) {
            Some(set) => format!(", which {} holds", set.held_by()),
            None => String::from(", which no holder in this reply grants the account"),
        };
        entries.push(format!(
            "`{}` in {} on `{}`{holder} ({})",
            definition.permission,
            definition.who.describe(),
            definition.carried_by,
            unallow_command(definition, facts)
        ));
    }
    for set in &held {
        if !facts.grants.iter().any(
            |entry| return matches!(&entry.who, Who::NamedSet(name) if name.as_str() == set.name),
        ) {
            entries.push(format!(
                "named set {} which {} holds, and this reply defines no member of it",
                set.name,
                set.held_by()
            ));
        }
    }
    return each_line_once(&entries);
}
