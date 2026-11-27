//! What:
//!  The permissions the route's account needs,
//!  where the design grants each of them,
//!  and the permissions whose presence would let the account change the
//!  lineage the removal proof rests on.
//! Why:
//!  The design decides one scope per permission,
//!  so a grant of the right permission at a wider scope is effective but not
//!  the decided shape,
//!  and both halves of that judgement are written here rather than inferred
//!  from a permission's name.
//!  `allow` stands among the forbidden permissions because it lets the account
//!  hand its own grants to another account or to everyone.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const REQUIRED = [{ place: 'checkout', permission: 'snapshot' }, ...];
//! const FORBIDDEN = ['mountpoint', 'canmount', 'setuid', 'devices', 'rename',
//!   'promote', 'receive', 'receive:append', 'allow'];
//! ```
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
pub const REQUIRED_GRANTS: &[RequiredGrant] = &[
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
pub const FORBIDDEN_PERMISSIONS: &[&str] = &[
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
