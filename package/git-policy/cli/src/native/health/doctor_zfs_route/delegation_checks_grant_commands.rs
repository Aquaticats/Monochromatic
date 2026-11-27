//! What:
//!  The dataset one place names,
//!  the inheritance one place requires,
//!  how a required permission was met,
//!  and the `zfs allow` and `zfs unallow` commands that put a grant where the
//!  design puts it or take a wider one away.
//! Why:
//!  A permission that reaches the account through an ancestor's descendant
//!  grant is effective but not the decided shape,
//!  so the check has to tell an exact grant from a wider one,
//!  and each holder kind has its own removal form:
//!  a creation-time permission comes off with `-c`,
//!  a permission set's member with `-s`,
//!  and a user's,
//!  a group's or everyone's with the scope and holder it was granted with.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const satisfaction = satisfactionOf(required, facts); // 'exact' | { broad } | 'missing'
//! const allow = allowCommand(required, facts);
//! const unallow = unallowCommand(grant, facts);
//! ```

/// One required permission and the place the design grants it.
use super::delegation_checks_permissions::{
    GrantPlace,
    RequiredGrant,
};
/// The grants the kernel's reply became,
/// and the tests that read one.
use super::delegation_reply::{
    DelegationFacts,
    Grant,
    Inheritance,
    Who,
    grants_the_account,
};
/// Whether `ancestor` is a dataset above `dataset`.
fn is_ancestor_of(
    ancestor: &str,
    dataset: &str,
) -> bool {
    let Some(rest) = dataset.strip_prefix(ancestor) else {
        return false;
    };
    return rest.starts_with('/') && rest.len() > 1;
}

/// The dataset a place names.
pub fn place_dataset(
    place: GrantPlace,
    facts: &DelegationFacts,
) -> &str {
    return match place {
        GrantPlace::Checkout | GrantPlace::CheckoutDescendants => facts.checkout_dataset.as_str(),
        GrantPlace::Container | GrantPlace::ContainerDescendants => {
            facts.container_dataset.as_str()
        }
    };
}

/// The inheritance a place requires.
fn place_inheritance(place: GrantPlace) -> Inheritance {
    return match place {
        GrantPlace::Checkout | GrantPlace::Container => Inheritance::Local,
        GrantPlace::CheckoutDescendants | GrantPlace::ContainerDescendants => {
            Inheritance::Descendants
        }
    };
}

/// What:
///  How a required permission was met,
///  which tells a decided grant from a wider one that happens to reach it.
/// Why:
///  The design fixes the scope as well as the permission,
///  so "effective but granted on an ancestor" is a finding about the scope
///  rather than a pass.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Satisfaction = 'exact' | { broad: Grant } | 'missing';
/// ```
pub(super) fn satisfaction<'a>(
    required: &RequiredGrant,
    facts: &'a DelegationFacts,
) -> Satisfaction<'a> {
    let target: &str = place_dataset(required.place, facts);
    let want: Inheritance = place_inheritance(required.place);
    let mut broad: Option<&Grant> = None;
    for grant in &facts.grants {
        if grant.permission != required.permission || !grants_the_account(grant, facts) {
            continue;
        }
        if grant.carried_by == target && grant.inheritance == want {
            return Satisfaction::Exact;
        }
        if grant.inheritance == Inheritance::Descendants
            && is_ancestor_of(&grant.carried_by, target)
        {
            broad = Some(grant);
        }
    }
    return match broad {
        Some(grant) => Satisfaction::Broad(grant),
        None => Satisfaction::Missing,
    };
}

/// What how a required permission was met.
pub(super) enum Satisfaction<'a> {
    /// Granted where and as widely as the design says.
    Exact,
    /// Effective through an ancestor's descendant grant.
    Broad(&'a Grant),
    /// Not granted to the account at all.
    Missing,
}

/// The `zfs allow` command that gives one required permission in the decided
/// shape.
pub fn allow_command(
    required: &RequiredGrant,
    facts: &DelegationFacts,
) -> String {
    let scope: &str = match place_inheritance(required.place) {
        Inheritance::Local => "-l",
        Inheritance::Descendants => "-d",
        Inheritance::NotApplicable => "-l",
    };
    return format!(
        "zfs allow {scope} -u {} {} {}",
        facts.account,
        required.permission,
        place_dataset(required.place, facts)
    );
}

/// The `-l` or `-d` a `zfs unallow` of one grant carries.
fn scope_flag(inheritance: Inheritance) -> &'static str {
    return match inheritance {
        Inheritance::Local => "-l",
        Inheritance::Descendants => "-d",
        Inheritance::NotApplicable => "-l",
    };
}

/// What:
///  The `zfs unallow` command that removes one forbidden grant.
/// Why:
///  Each holder kind has its own form and only some take a scope:
///  a creation-time permission comes off with `-c`,
///  a permission set's member with `-s`,
///  and a user's, a group's or everyone's with the scope and holder they were
///  granted with
///  (OpenZFS `zfs-2.4.4`,
///  `man/man8/zfs-allow.8`,
///  "zfs unallow").
pub(super) fn unallow_command(
    grant: &Grant,
    facts: &DelegationFacts,
) -> String {
    return match &grant.who {
        Who::Create => format!("zfs unallow -c {} {}", grant.permission, grant.carried_by),
        Who::NamedSet(name) => format!(
            "zfs unallow -s {name} {} {}",
            grant.permission, grant.carried_by
        ),
        Who::User(id) => format!(
            "zfs unallow {} -u {id} {} {}",
            scope_flag(grant.inheritance),
            grant.permission,
            grant.carried_by
        ),
        Who::Group(id) => format!(
            "zfs unallow {} -g {id} {} {}",
            scope_flag(grant.inheritance),
            grant.permission,
            grant.carried_by
        ),
        Who::Everyone => format!(
            "zfs unallow {} -e {} {}",
            scope_flag(grant.inheritance),
            grant.permission,
            grant.carried_by
        ),
        Who::Unknown(_) => format!(
            "zfs unallow {} -u {} {} {}",
            scope_flag(grant.inheritance),
            facts.account,
            grant.permission,
            grant.carried_by
        ),
    };
}
