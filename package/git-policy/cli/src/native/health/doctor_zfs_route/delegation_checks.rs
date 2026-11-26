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
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const grants = grantsOf(fsaclReply); // [{ carriedBy, permission, who, inheritance }]
//! const outcome = checkDelegation(facts); // pass | finding('permission'|'grant'|'scope') | unknown
//! ```

use super::check_outcome::{
    CheckOutcome,
    finding,
    pass,
    unknown,
};
use super::delegation_reply::{
    DelegationFacts,
    Grant,
    HeldSet,
    Inheritance,
    Who,
    grants_the_account,
    held_sets,
    names_a_set,
    set_memberships,
};

//region The permission names the design fixes

/// The permissions the route's account needs,
/// with where and how widely each is granted.
///
/// Why:
///  The design decides one scope per permission,
///  and a grant of the right permission at a wider scope is effective but not
///  the decided shape,
///  so both are written here rather than inferred from a name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const REQUIRED = [{ place: 'checkout', permission: 'snapshot' }, ...];
/// ```
pub const REQUIRED_GRANTS: [RequiredGrant; 9] = [
    RequiredGrant {
        place: GrantPlace::Checkout,
        permission: "snapshot",
    },
    RequiredGrant {
        place: GrantPlace::CheckoutDescendants,
        permission: "clone",
    },
    RequiredGrant {
        place: GrantPlace::CheckoutDescendants,
        permission: "mount",
    },
    RequiredGrant {
        place: GrantPlace::CheckoutDescendants,
        permission: "destroy",
    },
    RequiredGrant {
        place: GrantPlace::Container,
        permission: "create",
    },
    RequiredGrant {
        place: GrantPlace::Container,
        permission: "mount",
    },
    RequiredGrant {
        place: GrantPlace::ContainerDescendants,
        permission: "mount",
    },
    RequiredGrant {
        place: GrantPlace::ContainerDescendants,
        permission: "userprop",
    },
    RequiredGrant {
        place: GrantPlace::ContainerDescendants,
        permission: "destroy",
    },
];

/// The permissions whose presence would let the account change the lineage the
/// removal proof rests on,
/// or change what a mount exposes.
///
/// Why:
///  The design names every one of these as a grant the route must not have,
///  and `allow` is added here because it lets the account hand its own grants
///  to another account or to everyone,
///  which would put the power to change the clones' lineage in hands other
///  than the account's and root's
///  (`doc/decision/cli-git-zfs-delegation-forbids-allow.md`;
///  OpenZFS `zfs-2.4.4`,
///  `man/man8/zfs-allow.8`,
///  records that `allow` "Must also have the permission that is being
///  allowed",
///  so it grants nothing the account does not already hold).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const FORBIDDEN = ['mountpoint', 'canmount', 'setuid', 'devices', 'rename', 'promote', 'receive', 'receive:append', 'allow'];
/// ```
pub const FORBIDDEN_PERMISSIONS: [&str; 9] = [
    "mountpoint",
    "canmount",
    "setuid",
    "devices",
    "rename",
    "promote",
    "receive",
    "receive:append",
    "allow",
];

//endregion

//region The decided grants and where they belong

/// What:
///  Where the design puts one required permission.
/// Why:
///  The checkout and the container are siblings,
///  so a grant on one never reaches the other,
///  and the local and descendant halves of one dataset are two allow calls.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type GrantPlace = 'checkout' | 'checkout-descendants' | 'container' | 'container-descendants';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GrantPlace {
    /// The checkout dataset itself.
    Checkout,
    /// Every descendant of the checkout dataset.
    CheckoutDescendants,
    /// The clone container itself.
    Container,
    /// Every descendant of the clone container,
    /// which is every clone.
    ContainerDescendants,
}

/// One permission the route needs,
/// and where the design grants it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RequiredGrant = { place: GrantPlace; permission: string };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RequiredGrant {
    /// The dataset or descendants the permission is granted on.
    pub place: GrantPlace,
    /// The permission's name.
    pub permission: &'static str,
}

//endregion

//region The comparison

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
fn satisfaction<'a>(
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
enum Satisfaction<'a> {
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
fn unallow_command(
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

/// What:
///  One decided permission the pool reports more widely than the design
///  scopes it,
///  with the grant that makes it effective.
/// Why:
///  Narrowing one permission is two commands about one pair of datasets,
///  and one wider grant can serve several permissions,
///  so both halves are read from one pairing rather than from two lists.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Narrowing = { required: RequiredGrant; wide: Grant };
/// ```
struct Narrowing<'a> {
    /// The permission and the place the design grants it.
    required: &'a RequiredGrant,
    /// The ancestor's wider grant that reaches the place today.
    wide: &'a Grant,
}

/// Every decided permission that is effective only through an ancestor's
/// descendant grant.
fn narrowings<'a>(facts: &'a DelegationFacts) -> Vec<Narrowing<'a>> {
    let mut found: Vec<Narrowing<'a>> = Vec::new();
    for required in &REQUIRED_GRANTS {
        if let Satisfaction::Broad(wide) = satisfaction(required, facts) {
            found.push(Narrowing {
                required,
                wide,
            });
        }
    }
    return found;
}

/// What:
///  The lines in the order they were built and each printed once.
/// Why:
///  One ancestor grant can make two permissions effective,
///  and a second command to remove it would have nothing left to remove.
///  A reply read for two datasets carries a common ancestor's grant twice,
///  and one grant must not be reported or counted twice.
fn each_line_once(lines: &[String]) -> Vec<String> {
    let mut kept: Vec<String> = Vec::new();
    for line in lines {
        if !kept.contains(line) {
            kept.push(line.clone());
        }
    }
    return kept;
}

/// What:
///  The `scope` finding's line:
///  each permission the pool grants more widely than the design scopes it,
///  then every `zfs allow` that puts one where the design puts it,
///  then every `zfs unallow` that removes a wider grant.
/// Why:
///  A person reads the line to narrow the account's delegation,
///  and the decided grant has to be there before the wider one goes,
///  or the account is without the permission between the two commands.
///  One wider grant can serve more than one permission,
///  so the removal follows every grant that replaces it and prints once.
fn scope_detail(
    narrowings: &[Narrowing<'_>],
    facts: &DelegationFacts,
) -> String {
    let mut entries: Vec<String> = Vec::new();
    let mut allows: Vec<String> = Vec::new();
    let mut unallows: Vec<String> = Vec::new();
    for narrowing in narrowings {
        entries.push(format!(
            "`{}` reaches `{}` only through `{}`'s descendant grant",
            narrowing.required.permission,
            place_dataset(narrowing.required.place, facts),
            narrowing.wide.carried_by
        ));
        allows.push(allow_command(narrowing.required, facts));
        unallows.push(unallow_command(narrowing.wide, facts));
    }
    return format!(
        "{} decided permission(s) are effective but granted more widely than the design scopes them: {}. Grant each where the design puts it before the wider grant goes, so the account is never without one: {}. Then remove the wider grant(s): {}",
        narrowings.len(),
        entries.join("; "),
        each_line_once(&allows).join("; "),
        each_line_once(&unallows).join("; ")
    );
}

/// The grants of a route permission that reach the account through the
/// required places,
/// used to recognize a permission the check must look at.
fn is_route_permission(permission: &str) -> bool {
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
fn creation_time_forbidden(facts: &DelegationFacts) -> Vec<String> {
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
///  each naming the set, the holder and the command that takes the permission
///  out of the set.
/// Why:
///  A set's member is a grant the account holds through the holder that passes
///  the set on,
///  so it is judged exactly as a direct grant of the same permission.
fn held_set_forbidden(facts: &DelegationFacts) -> Vec<String> {
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
///  each naming the set, who holds it and the command that takes the
///  permission out of it.
/// Why:
///  The design grants each required permission to the account itself,
///  so a permission that reaches the account through a set is not the shape
///  the design decides,
///  and a set nobody in the reply holds still leaves the route's permissions
///  unproved because the check cannot say what a later grant of it would do.
fn unproved_named_sets(facts: &DelegationFacts) -> Vec<String> {
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
///  the answer unknown with the set, the permission and the command named,
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
///  builds, so the missing set is answered on its own
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
