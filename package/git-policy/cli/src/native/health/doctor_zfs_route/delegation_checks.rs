//! What:
//!  The account's effective delegation over the route's datasets,
//!  including the grants an ancestor carries,
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
//!  The kernel's own reply is the one source:
//!  `ZFS_IOC_GET_FSACL` returns every dataset on the way up that carries a
//!  delegation,
//!  as `{dataset: {who: {permission: boolean}}}`
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

use super::check_outcome::{CheckOutcome, finding, pass, unknown};
use monochromatic_zfs_request_to_openzfs_kernel_device::{NvData, NvList};

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
    RequiredGrant { place: GrantPlace::Checkout, permission: "snapshot" },
    RequiredGrant { place: GrantPlace::CheckoutDescendants, permission: "clone" },
    RequiredGrant { place: GrantPlace::CheckoutDescendants, permission: "mount" },
    RequiredGrant { place: GrantPlace::CheckoutDescendants, permission: "destroy" },
    RequiredGrant { place: GrantPlace::Container, permission: "create" },
    RequiredGrant { place: GrantPlace::Container, permission: "mount" },
    RequiredGrant { place: GrantPlace::ContainerDescendants, permission: "mount" },
    RequiredGrant { place: GrantPlace::ContainerDescendants, permission: "userprop" },
    RequiredGrant { place: GrantPlace::ContainerDescendants, permission: "destroy" },
];

/// The permissions whose presence would let the account change the lineage the
/// removal proof rests on,
/// or change what a mount exposes.
///
/// Why:
///  The design names every one of these as a grant the route must not have,
///  and `allow` is added here because it would let the account grant itself
///  any of the others,
///  which is a decision of this build rather than of the design.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const FORBIDDEN = ['mountpoint', 'canmount', 'setuid', 'devices', 'rename', 'promote', 'receive', 'receive:append', 'allow'];
/// ```
pub const FORBIDDEN_PERMISSIONS: [&str; 9] = ["mountpoint", "canmount", "setuid", "devices", "rename", "promote", "receive", "receive:append", "allow"];

//endregion

//region The grant model

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

/// What:
///  How widely a grant reaches:
///  this dataset only,
///  or every descendant as well.
/// Why:
///  The kernel keeps the two apart in the who key's second character,
///  and the design's scopes turn on that difference.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Inheritance = 'local' | 'descendants' | 'not-applicable';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Inheritance {
    /// This dataset only (`l`).
    Local,
    /// This dataset and every descendant (`d`).
    Descendants,
    /// The create-time pseudo who has no inheritance (`-`).
    NotApplicable,
}

/// What:
///  Who a grant names.
/// Why:
///  The route cares about the account's own user and groups,
///  and a grant to everyone or through a named set is effective while being
///  wider than the decided shape or not readable at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Who = { kind: 'user' | 'group'; id: number } | { kind: 'everyone' } | { kind: 'named-set'; name } | { kind: 'create' } | { kind: 'unknown'; raw };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Who {
    /// A user by the account's id.
    User(u32),
    /// A group by its id.
    Group(u32),
    /// Everyone (`everyone` in `zfs allow`).
    Everyone,
    /// A named permission set,
    /// whose members this read does not show.
    NamedSet(String),
    /// The create-time pseudo permission.
    Create,
    /// A who key this reader does not represent.
    Unknown(String),
}

impl Who {
    /// The words a finding uses for this grant's holder,
    /// which are also the `zfs allow` argument.
    pub fn describe(&self) -> String {
        return match self {
            Who::User(id) => format!("user {id}"),
            Who::Group(id) => format!("group {id}"),
            Who::Everyone => String::from("everyone"),
            Who::NamedSet(name) => format!("named set {name}"),
            Who::Create => String::from("the create-time who"),
            Who::Unknown(raw) => format!("the who key `{raw}`"),
        };
    }
}

/// One permission one dataset grants to one holder.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Grant = { carriedBy: string; permission: string; who: Who; inheritance: Inheritance };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Grant {
    /// The dataset whose delegation record holds the grant,
    /// which is the ancestor that makes it effective.
    pub carried_by: String,
    /// The permission's kernel name.
    pub permission: String,
    /// Who it is granted to.
    pub who: Who,
    /// How widely it reaches.
    pub inheritance: Inheritance,
}

/// What:
///  Everything the delegation check reads:
///  the account,
///  the two datasets,
///  and every grant the kernel answered for them.
/// Why:
///  The account's id and groups decide which grants are its own,
///  and the two dataset names decide where each required permission must
///  stand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DelegationFacts = { account: number; groups: number[]; checkoutDataset: string; containerDataset: string; grants: Grant[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DelegationFacts {
    /// The managed account's user id.
    pub account: u32,
    /// Every group the account belongs to.
    pub groups: Vec<u32>,
    /// The checkout dataset.
    pub checkout_dataset: String,
    /// The clone container.
    pub container_dataset: String,
    /// Every grant the kernel answered,
    /// from the reply for either dataset,
    /// which walks its own ancestors.
    pub grants: Vec<Grant>,
}

//endregion

//region Reading the kernel's reply

/// What:
///  One who key's type,
///  inheritance and id,
///  as `zfs_deleg_whokey` writes them.
/// Why:
///  The who key is the only place the kernel keeps who a grant names and how
///  widely it reaches,
///  so reading it wrongly would report another account's grant as this one's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const who = parseWho('ud$1000'); // { who: { kind: 'user', id: 1000 }, inheritance: 'descendants' }
/// ```
fn who_of(key: &str) -> (Who, Inheritance) {
    let mut characters = key.chars();
    let who_type: Option<char> = characters.next();
    let inherit: Option<char> = characters.next();
    let separator: Option<char> = characters.next();
    let inheritance: Inheritance = match inherit {
        Some('l') => Inheritance::Local,
        Some('d') => Inheritance::Descendants,
        Some('-') => Inheritance::NotApplicable,
        _ => Inheritance::NotApplicable,
    };
    let rest: String = characters.collect();
    if separator != Some('$') {
        return (Who::Unknown(String::from(key)), inheritance);
    }
    let who: Who = match (who_type, rest.as_str()) {
        (Some('u' | 'U'), id) => id.parse::<u32>().map_or_else(|_error| return Who::Unknown(String::from(key)), Who::User),
        (Some('g' | 'G'), id) => id.parse::<u32>().map_or_else(|_error| return Who::Unknown(String::from(key)), Who::Group),
        (Some('e' | 'E'), "") => Who::Everyone,
        (Some('c' | 'C'), "") => Who::Create,
        (Some('s' | 'S'), name) => Who::NamedSet(String::from(name)),
        _ => Who::Unknown(String::from(key)),
    };
    return (who, inheritance);
}

/// What:
///  Whether a permission entry says the grant is there.
/// Why:
///  The kernel writes a boolean flag for each permission,
///  and a value that says false is not a grant.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const granted = (data) => data.kind === 'flag' || (data.kind === 'boolean' && data.value);
/// ```
fn permission_is_granted(data: &NvData) -> bool {
    return match data {
        NvData::BooleanFlag => true,
        NvData::Boolean { value } => *value,
        _ => false,
    };
}

/// What:
///  Every grant the kernel's `ZFS_IOC_GET_FSACL` reply holds,
///  as `{dataset: {who: {permission: boolean}}}`.
/// Why:
///  One reply names every dataset on the way up that carries a delegation,
///  so the ancestor grants the design asks about arrive with the dataset's own,
///  and a reply this reader cannot represent is reported rather than guessed
///  at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const grants = grantsOf(reply); // one Grant per permission
/// ```
///
/// # Errors
/// A pair that is not a dataset's list of holders,
/// a holder's value that is not a list of permissions,
/// or a permission entry that is not a boolean.
pub fn grants_of(fsacl: &NvList) -> Result<Vec<Grant>, String> {
    let mut grants: Vec<Grant> = Vec::new();
    for dataset in &fsacl.pairs {
        let NvData::List { value: holders } = &dataset.data else {
            return Err(format!("`{}` is not a list of holders, which is what the kernel writes for a dataset's delegation", dataset.name));
        };
        for holder in &holders.pairs {
            let NvData::List { value: permissions } = &holder.data else {
                return Err(format!("`{}` below `{}` is not a list of permissions", holder.name, dataset.name));
            };
            let (who, inheritance) = who_of(&holder.name);
            for permission in &permissions.pairs {
                if !permission_is_granted(&permission.data) {
                    continue;
                }
                grants.push(Grant { carried_by: dataset.name.clone(), permission: permission.name.clone(), who: who.clone(), inheritance });
            }
        }
    }
    return Ok(grants);
}

//endregion

//region The comparison

/// Whether a grant names the account itself.
fn grants_the_account(grant: &Grant, facts: &DelegationFacts) -> bool {
    return match &grant.who {
        Who::User(id) => *id == facts.account,
        Who::Group(id) => facts.groups.contains(id),
        Who::Everyone | Who::NamedSet(_) | Who::Create | Who::Unknown(_) => false,
    };
}

/// Whether `ancestor` is a dataset above `dataset`.
fn is_ancestor_of(ancestor: &str, dataset: &str) -> bool {
    let Some(rest) = dataset.strip_prefix(ancestor) else {
        return false;
    };
    return rest.starts_with('/') && rest.len() > 1;
}

/// The dataset a place names.
fn place_dataset(place: GrantPlace, facts: &DelegationFacts) -> &str {
    return match place {
        GrantPlace::Checkout | GrantPlace::CheckoutDescendants => facts.checkout_dataset.as_str(),
        GrantPlace::Container | GrantPlace::ContainerDescendants => facts.container_dataset.as_str(),
    };
}

/// The inheritance a place requires.
fn place_inheritance(place: GrantPlace) -> Inheritance {
    return match place {
        GrantPlace::Checkout | GrantPlace::Container => Inheritance::Local,
        GrantPlace::CheckoutDescendants | GrantPlace::ContainerDescendants => Inheritance::Descendants,
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
fn satisfaction<'a>(required: &RequiredGrant, facts: &'a DelegationFacts) -> Satisfaction<'a> {
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
        if grant.inheritance == Inheritance::Descendants && is_ancestor_of(&grant.carried_by, target) {
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
fn allow_command(required: &RequiredGrant, facts: &DelegationFacts) -> String {
    let scope: &str = match place_inheritance(required.place) {
        Inheritance::Local => "-l",
        Inheritance::Descendants => "-d",
        Inheritance::NotApplicable => "-l",
    };
    return format!("zfs allow {scope} -u {} {} {}", facts.account, required.permission, place_dataset(required.place, facts));
}

/// The `zfs unallow` command that removes one forbidden grant.
fn unallow_command(grant: &Grant, facts: &DelegationFacts) -> String {
    let scope: &str = match grant.inheritance {
        Inheritance::Local => "-l",
        Inheritance::Descendants => "-d",
        Inheritance::NotApplicable => "-l",
    };
    let holder: String = match &grant.who {
        Who::User(id) => format!("-u {id}"),
        Who::Group(id) => format!("-g {id}"),
        Who::Everyone => String::from("-e"),
        Who::NamedSet(name) => format!("-s {name}"),
        Who::Create | Who::Unknown(_) => format!("-u {}", facts.account),
    };
    return format!("zfs unallow {scope} {holder} {} {}", grant.permission, grant.carried_by);
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
            found.push(Narrowing { required, wide });
        }
    }
    return found;
}

/// What:
///  The commands in the order they were built and each printed once.
/// Why:
///  One ancestor grant can make two permissions effective,
///  and a second command to remove it would have nothing left to remove.
fn each_command_once(commands: &[String]) -> Vec<String> {
    let mut kept: Vec<String> = Vec::new();
    for command in commands {
        if !kept.contains(command) {
            kept.push(command.clone());
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
fn scope_detail(narrowings: &[Narrowing<'_>], facts: &DelegationFacts) -> String {
    let mut entries: Vec<String> = Vec::new();
    let mut allows: Vec<String> = Vec::new();
    let mut unallows: Vec<String> = Vec::new();
    for narrowing in narrowings {
        entries.push(format!("`{}` reaches `{}` only through `{}`'s descendant grant", narrowing.required.permission, place_dataset(narrowing.required.place, facts), narrowing.wide.carried_by));
        allows.push(allow_command(narrowing.required, facts));
        unallows.push(unallow_command(narrowing.wide, facts));
    }
    return format!(
        "{} decided permission(s) are effective but granted more widely than the design scopes them: {}. Grant each where the design puts it before the wider grant goes, so the account is never without one: {}. Then remove the wider grant(s): {}.",
        narrowings.len(),
        entries.join("; "),
        each_command_once(&allows).join("; "),
        each_command_once(&unallows).join("; ")
    );
}

/// The grants of a route permission that reach the account through the
/// required places,
/// used to recognize a permission the check must look at.
fn is_route_permission(permission: &str) -> bool {
    if FORBIDDEN_PERMISSIONS.contains(&permission) {
        return true;
    }
    return REQUIRED_GRANTS.iter().any(|required| return required.permission == permission);
}

/// What:
///  Whether the account's effective delegation is the decided one:
///  every required permission where and as widely as the design says,
///  no lineage-changing grant,
///  and no route permission granted to everyone.
/// Why:
///  The removal's threat boundary is exactly this set,
///  so the report names each grant that breaks it with the command that
///  removes it,
///  a permission granted wider than the design scopes it gets both the grant
///  in the decided place and the wider grant's removal,
///  and a named set the account belongs to leaves the answer unknown because
///  this read cannot expand it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = checkDelegation(facts); // pass | finding('permission'|'grant'|'scope'|'everyone') | unknown
/// ```
pub fn check_delegation(facts: &DelegationFacts) -> CheckOutcome {
    let forbidden: Vec<String> = facts
        .grants
        .iter()
        .filter(|grant| return FORBIDDEN_PERMISSIONS.contains(&grant.permission.as_str()) && (grants_the_account(grant, facts) || matches!(grant.who, Who::Everyone)))
        .map(|grant| return format!("`{}` on `{}` to {} ({})", grant.permission, grant.carried_by, grant.who.describe(), unallow_command(grant, facts)))
        .collect();
    if !forbidden.is_empty() {
        return finding("permission", format!("the account or everyone holds {} lineage-changing grant(s) the route must not have: {}", forbidden.len(), forbidden.join("; ")));
    }
    let everyone: Vec<String> = facts
        .grants
        .iter()
        .filter(|grant| return matches!(grant.who, Who::Everyone) && is_route_permission(&grant.permission))
        .map(|grant| return format!("`{}` on `{}`", grant.permission, grant.carried_by))
        .collect();
    if !everyone.is_empty() {
        return finding("everyone", format!("{} route permission(s) are granted to everyone, which reaches every account on this machine: {}", everyone.len(), everyone.join("; ")));
    }
    let named_sets: Vec<String> = facts
        .grants
        .iter()
        .filter(|grant| return matches!(grant.who, Who::NamedSet(_)) && is_route_permission(&grant.permission))
        .map(|grant| return format!("{} on `{}`", grant.who.describe(), grant.carried_by))
        .collect();
    if !named_sets.is_empty() {
        return unknown(format!("the route's permissions reach the account through {}, and the members of a named set are not in this reply, so the effective delegation is unproved", named_sets.join(", ")));
    }
    let missing: Vec<String> = REQUIRED_GRANTS
        .iter()
        .filter(|required| return matches!(satisfaction(required, facts), Satisfaction::Missing))
        .map(|required| return format!("`{}` on `{}` ({})", required.permission, place_dataset(required.place, facts), allow_command(required, facts)))
        .collect();
    if !missing.is_empty() {
        return finding("grant", format!("{} decided permission(s) are not granted to the account: {}", missing.len(), missing.join("; ")));
    }
    let narrowings: Vec<Narrowing<'_>> = narrowings(facts);
    if !narrowings.is_empty() {
        return finding("scope", scope_detail(&narrowings, facts));
    }
    return pass(format!("{} decided permission(s) are granted where the design puts them and no lineage-changing grant reaches the account", REQUIRED_GRANTS.len()));
}

//endregion

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "delegation_checks_tests.rs"]
mod tests;
