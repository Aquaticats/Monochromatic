//! What:
//!  The kernel's delegation reply and the grant model it becomes:
//!  who a grant names,
//!  how widely it reaches,
//!  and the dataset that carries it.
//! Why:
//!  `ZFS_IOC_GET_FSACL` is the one source of the account's effective
//!  delegation,
//!  and it answers `{dataset: {who: {name: boolean}}}`
//!  (OpenZFS `zfs-2.4.4`,
//!  `module/zfs/dsl_deleg.c`,
//!  `dsl_deleg_get`,
//!  and `module/zcommon/zfs_deleg.c`,
//!  `zfs_deleg_whokey`).
//!  A who key's first character says which holder it names and its second
//!  whether the grant reaches descendants,
//!  so reading the key the way the kernel writes it is what keeps one
//!  account's grant from being reported as another's.
//!  The grants are a value handed in,
//!  so a control builds the reply it chose and no pool is read.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const grants = grantsOf(fsaclReply); // [{ carriedBy, permission, who, inheritance }]
//! ```

use monochromatic_zfs_request_to_openzfs_kernel_device::{NvData, NvList};

//region The grant model

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
    /// The create-time and named-set who keys have no inheritance (`-`).
    NotApplicable,
}

/// What:
///  Who a grant names.
/// Why:
///  The route cares about the account's own user and groups,
///  and a grant to everyone, to a named set or to whoever creates a dataset
///  below the grant's dataset reaches the account while being wider than the
///  decided shape.
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
    /// whose members the reply carries under this same who key.
    NamedSet(String),
    /// The create-time pseudo holder,
    /// to whom the kernel grants the entry's name as a local grant on every
    /// dataset that holder creates below the one carrying it.
    Create,
    /// A who key this reader does not represent.
    Unknown(String),
}

impl Who {
    /// The words a finding uses for this grant's holder,
    /// which for the holders `zfs allow` names are also its argument.
    pub fn describe(&self) -> String {
        return match self {
            Who::User(id) => format!("user {id}"),
            Who::Group(id) => format!("group {id}"),
            Who::Everyone => String::from("everyone"),
            Who::NamedSet(name) => format!("named set {name}"),
            Who::Create => String::from("whoever creates a dataset below it"),
            Who::Unknown(raw) => format!("the who key `{raw}`"),
        };
    }
}

/// One name one dataset grants to one holder.
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
    /// The name the entry grants:
    /// a permission's kernel name,
    /// or a named set's name when the entry hands a set over.
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
///  as `{dataset: {who: {name: boolean}}}`.
/// Why:
///  One reply names every dataset on the way up that carries a delegation,
///  so the ancestor grants the design asks about arrive with the dataset's own,
///  and a reply this reader cannot represent is reported rather than guessed
///  at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const grants = grantsOf(reply); // one Grant per name
/// ```
///
/// # Errors
/// A pair that is not a dataset's list of holders,
/// a holder's value that is not a list of names,
/// or an entry that is not a boolean.
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

/// Whether a grant names the account itself.
pub(super) fn grants_the_account(grant: &Grant, facts: &DelegationFacts) -> bool {
    return match &grant.who {
        Who::User(id) => *id == facts.account,
        Who::Group(id) => facts.groups.contains(id),
        Who::Everyone | Who::NamedSet(_) | Who::Create | Who::Unknown(_) => false,
    };
}

//endregion
