//! What:
//!  Controls of the effective delegation:
//!  the nine decided grants,
//!  a missing grant,
//!  a grant wider than the design scopes it and the two commands that narrow it,
//!  a lineage-changing grant,
//!  a permission given to everyone,
//!  a creation-time grant of a forbidden permission,
//!  a forbidden permission inside a named set the account can hold,
//!  a forbidden permission reached through a set inside a set,
//!  a grant both replies carry,
//!  a set the reply defines no member of,
//!  and a route permission reached through a named set.
//! Why:
//!  The removal's threat boundary is this set,
//!  so a control changes one grant of the decided shape and reads the answer
//!  and the command the report prints.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(check_delegation(withGrant('rename', Inheritance.Local))).toMatchObject({ kind: 'finding', field: 'permission' });
//! ```

use super::check_delegation;
use crate::health::doctor_zfs_route::delegation_reply::{DelegationFacts, Grant, Inheritance, Who, grants_of};
use crate::health::doctor_zfs_route::route_test_facts::{ACCOUNT, ACCOUNT_GROUP, CHECKOUT_DATASET, CONTAINER_DATASET, grant, healthy_delegation, healthy_grants};
use monochromatic_zfs_request_to_openzfs_kernel_device::NvList;

/// The dataset above both the checkout and the clone container,
/// which a control grants a permission on when the design scopes it lower.
const ANCESTOR_DATASET: &str = "zpcachyos/ROOT/cos/home";

/// The decided facts with one grant added.
fn with_grant(granted: Grant) -> DelegationFacts {
    let mut facts: DelegationFacts = healthy_delegation();
    facts.grants.push(granted);
    return facts;
}

/// The decided facts with one grant removed.
fn without_grant(permission: &str, place: &str, inheritance: Inheritance) -> DelegationFacts {
    let mut facts: DelegationFacts = healthy_delegation();
    facts.grants.retain(|grant| return !(grant.permission == permission && grant.carried_by == place && grant.inheritance == inheritance));
    return facts;
}

//region The kernel's reply

/// The reply's `{dataset: {who: {permission: boolean}}}` shape becomes one
/// grant per permission.
#[test]
fn the_reply_shape_becomes_one_grant_per_permission() {
    let mut permissions: NvList = NvList::new();
    permissions.add_bool_flag("destroy");
    permissions.add_boolean("mount", true);
    permissions.add_boolean("rename", false);
    let mut holders: NvList = NvList::new();
    holders.add_nvlist("ud$1000", permissions);
    let mut reply: NvList = NvList::new();
    reply.add_nvlist(CHECKOUT_DATASET, holders);
    let grants: Vec<Grant> = grants_of(&reply).expect("the reply is the kernel's shape");
    assert_eq!(grants.len(), 2, "{grants:?}");
    assert_eq!(grants[0].permission, "destroy");
    assert_eq!(grants[0].who, Who::User(ACCOUNT));
    assert_eq!(grants[0].inheritance, Inheritance::Descendants);
    assert_eq!(grants[1].permission, "mount");
}

/// A who key the kernel writes is read for its type and inheritance.
#[test]
fn a_who_key_is_read_for_its_type_and_inheritance() {
    let mut permissions: NvList = NvList::new();
    permissions.add_bool_flag("snapshot");
    let mut holders: NvList = NvList::new();
    holders.add_nvlist("gl$1000", permissions.clone());
    holders.add_nvlist("ed$", permissions.clone());
    holders.add_nvlist("s-$backup", permissions);
    let mut reply: NvList = NvList::new();
    reply.add_nvlist(CHECKOUT_DATASET, holders);
    let grants: Vec<Grant> = grants_of(&reply).expect("the reply is the kernel's shape");
    assert_eq!(grants[0].who, Who::Group(ACCOUNT));
    assert_eq!(grants[0].inheritance, Inheritance::Local);
    assert_eq!(grants[1].who, Who::Everyone);
    assert_eq!(grants[2].who, Who::NamedSet(String::from("backup")));
}

/// A reply that is not the kernel's shape is a refusal naming the pair.
#[test]
fn a_reply_of_another_shape_is_a_refusal() {
    let mut reply: NvList = NvList::new();
    reply.add_str(CHECKOUT_DATASET, "not a holder list");
    let refusal: String = grants_of(&reply).expect_err("a string is not a holder list");
    assert!(refusal.contains(CHECKOUT_DATASET), "{refusal}");
}

//endregion

//region The decided shape

/// The nine decided grants pass.
#[test]
fn the_nine_decided_grants_pass() {
    let outcome: super::CheckOutcome = check_delegation(&healthy_delegation());
    let super::CheckOutcome::Pass { detail } = outcome else {
        panic!("the decided shape passes: {outcome:?}");
    };
    assert!(detail.contains('9'), "{detail}");
}

/// A missing decided grant is a finding naming the permission and the command.
#[test]
fn a_missing_grant_is_a_finding_with_the_command() {
    let facts: DelegationFacts = without_grant("userprop", CONTAINER_DATASET, Inheritance::Descendants);
    let super::CheckOutcome::Finding { field, detail } = check_delegation(&facts) else {
        panic!("a missing grant is a finding");
    };
    assert_eq!(field, "grant");
    assert!(detail.contains("userprop"), "{detail}");
    assert!(detail.contains("zfs allow -d -u 1000 userprop"), "{detail}");
}

/// A decided grant that reaches the dataset only through an ancestor is a
/// finding naming the scope and the two commands that narrow it:
/// the `zfs allow` where the design puts the permission,
/// then the `zfs unallow` of the ancestor's wider grant.
#[test]
fn a_grant_carried_by_an_ancestor_is_a_scope_finding_with_its_narrowing_pair() {
    let mut facts: DelegationFacts = without_grant("destroy", CONTAINER_DATASET, Inheritance::Descendants);
    facts.grants.push(grant(ANCESTOR_DATASET, "destroy", Who::User(ACCOUNT), Inheritance::Descendants));
    let super::CheckOutcome::Finding { field, detail } = check_delegation(&facts) else {
        panic!("a wider grant is a finding");
    };
    assert_eq!(field, "scope");
    assert!(detail.contains(CONTAINER_DATASET), "{detail}");
    assert!(detail.contains(ANCESTOR_DATASET), "{detail}");
    let narrow: String = format!("zfs allow -d -u {ACCOUNT} destroy {CONTAINER_DATASET}");
    let wide_removal: String = format!("zfs unallow -d -u {ACCOUNT} destroy {ANCESTOR_DATASET}");
    assert!(detail.contains(&narrow), "{detail}");
    assert!(detail.contains(&wide_removal), "{detail}");
    let narrow_at: usize = detail.find(&narrow).expect("the narrowed grant is printed");
    let wide_removal_at: usize = detail.find(&wide_removal).expect("the wider grant's removal is printed");
    assert!(narrow_at < wide_removal_at, "the grant comes before the wider one goes: {detail}");
}

/// One ancestor grant can make two decided permissions effective,
/// so both narrow grants are printed and the wider grant is removed after them.
#[test]
fn one_wide_grant_serving_two_permissions_is_removed_once_after_both_grants() {
    let mut facts: DelegationFacts = healthy_delegation();
    facts.grants.retain(|grant| return !(grant.permission == "mount" && (grant.carried_by == CHECKOUT_DATASET || grant.carried_by == CONTAINER_DATASET) && grant.inheritance == Inheritance::Descendants));
    facts.grants.push(grant(ANCESTOR_DATASET, "mount", Who::User(ACCOUNT), Inheritance::Descendants));
    let super::CheckOutcome::Finding { field, detail } = check_delegation(&facts) else {
        panic!("a wider grant is a finding");
    };
    assert_eq!(field, "scope");
    let checkout_narrow: String = format!("zfs allow -d -u {ACCOUNT} mount {CHECKOUT_DATASET}");
    let container_narrow: String = format!("zfs allow -d -u {ACCOUNT} mount {CONTAINER_DATASET}");
    let wide_removal: String = format!("zfs unallow -d -u {ACCOUNT} mount {ANCESTOR_DATASET}");
    assert!(detail.contains(&checkout_narrow), "{detail}");
    assert!(detail.contains(&container_narrow), "{detail}");
    assert_eq!(detail.matches(&wide_removal).count(), 1, "the one wider grant is removed once: {detail}");
    let checkout_narrow_at: usize = detail.find(&checkout_narrow).expect("the checkout's narrowed grant is printed");
    let container_narrow_at: usize = detail.find(&container_narrow).expect("the container's narrowed grant is printed");
    let wide_removal_at: usize = detail.find(&wide_removal).expect("the wider grant's removal is printed");
    assert!(checkout_narrow_at < wide_removal_at && container_narrow_at < wide_removal_at, "both grants come before the wider one goes: {detail}");
}

/// Every forbidden permission is a finding naming it and the unallow command.
#[test]
fn a_forbidden_permission_is_a_finding_with_the_unallow_command() {
    for forbidden in ["mountpoint", "canmount", "setuid", "devices", "rename", "promote", "receive", "receive:append", "allow"] {
        let facts: DelegationFacts = with_grant(grant(CONTAINER_DATASET, forbidden, Who::User(ACCOUNT), Inheritance::Descendants));
        let super::CheckOutcome::Finding { field, detail } = check_delegation(&facts) else {
            panic!("{forbidden} is forbidden");
        };
        assert_eq!(field, "permission", "{forbidden}");
        assert!(detail.contains(forbidden), "{detail}");
        assert!(detail.contains("zfs unallow -d -u 1000"), "{detail}");
    }
}

/// A route permission given to everyone is a finding naming it,
/// even when the account's own grants are right.
#[test]
fn a_permission_given_to_everyone_is_a_finding() {
    let facts: DelegationFacts = with_grant(grant(CONTAINER_DATASET, "destroy", Who::Everyone, Inheritance::Descendants));
    let super::CheckOutcome::Finding { field, detail } = check_delegation(&facts) else {
        panic!("a grant to everyone is a finding");
    };
    assert_eq!(field, "everyone");
    assert!(detail.contains(CONTAINER_DATASET), "{detail}");
}

/// A route permission the account holds through a named set leaves the
/// effective delegation unproved,
/// and the reason names the set, the permission and the command that takes the
/// permission out of the set.
#[test]
fn a_named_set_the_account_holds_leaves_the_delegation_unknown() {
    let mut facts: DelegationFacts = healthy_delegation();
    facts.grants.push(grant(CONTAINER_DATASET, "@backup", Who::User(ACCOUNT), Inheritance::Local));
    facts.grants.push(grant(CONTAINER_DATASET, "destroy", Who::NamedSet(String::from("@backup")), Inheritance::NotApplicable));
    let super::CheckOutcome::Unknown { reason } = check_delegation(&facts) else {
        panic!("a held named set leaves the answer unknown");
    };
    assert!(reason.contains("@backup"), "{reason}");
    assert!(reason.contains("destroy"), "the permission is named: {reason}");
    assert!(reason.contains(&format!("zfs unallow -s @backup destroy {CONTAINER_DATASET}")), "the command that removes it is named: {reason}");
}

/// A set definition that no holder in the reply grants the account still
/// leaves the route's permissions unproved,
/// and the reason says that no holder grants it.
#[test]
fn a_named_set_no_holder_grants_leaves_the_route_permissions_unproved() {
    let facts: DelegationFacts = with_grant(grant(CONTAINER_DATASET, "destroy", Who::NamedSet(String::from("@backup")), Inheritance::NotApplicable));
    let super::CheckOutcome::Unknown { reason } = check_delegation(&facts) else {
        panic!("a set definition leaves the route's permissions unproved");
    };
    assert!(reason.contains("@backup"), "{reason}");
    assert!(reason.contains("no holder"), "{reason}");
}

/// A creation-time grant is one the kernel copies onto every dataset its
/// creator makes below the dataset that carries it,
/// and the account creates the route's clones below the container,
/// so the grant is a finding naming the permission and the `zfs unallow -c`
/// command that clears it.
#[test]
fn a_creation_time_grant_of_a_forbidden_permission_is_a_finding() {
    for dataset in [CHECKOUT_DATASET, CONTAINER_DATASET] {
        let facts: DelegationFacts = with_grant(grant(dataset, "mountpoint", Who::Create, Inheritance::NotApplicable));
        let outcome: super::CheckOutcome = check_delegation(&facts);
        let super::CheckOutcome::Finding { field, detail } = outcome else {
            panic!("a creation-time `mountpoint` on `{dataset}` is a finding: {outcome:?}");
        };
        assert_eq!(field, "permission", "{dataset}");
        assert!(detail.contains("mountpoint"), "{detail}");
        assert!(detail.contains(&format!("zfs unallow -c mountpoint {dataset}")), "{detail}");
    }
}

/// A named set's members are in the reply under the set's own who key,
/// so a forbidden permission inside a set the account can hold is a finding
/// naming the set, the permission and the command that takes the permission
/// out of the set,
/// whichever holder hands the set over.
#[test]
fn a_forbidden_permission_in_a_named_set_the_account_can_hold_is_a_finding() {
    let holders: [(Who, Inheritance, &str); 4] = [
        (Who::User(ACCOUNT), Inheritance::Local, "the account's own grant"),
        (Who::Group(ACCOUNT_GROUP), Inheritance::Descendants, "a grant to one of the account's groups"),
        (Who::Everyone, Inheritance::Local, "a grant to everyone"),
        (Who::Create, Inheritance::NotApplicable, "a creation-time set grant"),
    ];
    for (holder, inheritance, words) in holders {
        let mut facts: DelegationFacts = healthy_delegation();
        facts.grants.push(grant(CONTAINER_DATASET, "@backup", holder, inheritance));
        facts.grants.push(grant(CONTAINER_DATASET, "mountpoint", Who::NamedSet(String::from("@backup")), Inheritance::NotApplicable));
        let outcome: super::CheckOutcome = check_delegation(&facts);
        let super::CheckOutcome::Finding { field, detail } = outcome else {
            panic!("{words} hands the account a set holding `mountpoint`: {outcome:?}");
        };
        assert_eq!(field, "permission", "{words}");
        assert!(detail.contains("@backup"), "{words}: {detail}");
        assert!(detail.contains("mountpoint"), "{words}: {detail}");
        assert!(detail.contains(&format!("zfs unallow -s @backup mountpoint {CONTAINER_DATASET}")), "{words}: {detail}");
    }
}

/// A set inside a held set is expanded as the kernel expands it before it
/// judges a permission,
/// so a forbidden permission two sets deep is still named with its command.
#[test]
fn a_forbidden_permission_reached_through_a_nested_set_is_a_finding() {
    let mut facts: DelegationFacts = healthy_delegation();
    facts.grants.push(grant(CONTAINER_DATASET, "@outer", Who::User(ACCOUNT), Inheritance::Local));
    facts.grants.push(grant(CONTAINER_DATASET, "@inner", Who::NamedSet(String::from("@outer")), Inheritance::NotApplicable));
    facts.grants.push(grant(CONTAINER_DATASET, "mountpoint", Who::NamedSet(String::from("@inner")), Inheritance::NotApplicable));
    let outcome: super::CheckOutcome = check_delegation(&facts);
    let super::CheckOutcome::Finding { field, detail } = outcome else {
        panic!("a set inside a held set hands `mountpoint` over: {outcome:?}");
    };
    assert_eq!(field, "permission");
    assert!(detail.contains("@inner"), "{detail}");
    assert!(detail.contains(&format!("zfs unallow -s @inner mountpoint {CONTAINER_DATASET}")), "{detail}");
}

/// A grant both replies carry, because it stands on an ancestor of both
/// datasets, is reported and counted once.
#[test]
fn a_grant_both_replies_carry_is_a_finding_once() {
    let mut facts: DelegationFacts = healthy_delegation();
    facts.grants.push(grant(ANCESTOR_DATASET, "mountpoint", Who::User(ACCOUNT), Inheritance::Descendants));
    facts.grants.push(grant(ANCESTOR_DATASET, "mountpoint", Who::User(ACCOUNT), Inheritance::Descendants));
    let outcome: super::CheckOutcome = check_delegation(&facts);
    let super::CheckOutcome::Finding { field, detail } = outcome else {
        panic!("a forbidden grant on an ancestor is a finding: {outcome:?}");
    };
    assert_eq!(field, "permission");
    assert!(detail.contains("1 lineage-changing grant"), "{detail}");
    assert_eq!(detail.matches(&format!("zfs unallow -d -u {ACCOUNT} mountpoint {ANCESTOR_DATASET}")).count(), 1, "{detail}");
}

/// One set two entries hand the account is one set,
/// so its forbidden member is a finding printed once.
#[test]
fn a_set_the_account_holds_twice_is_expanded_once() {
    let mut facts: DelegationFacts = healthy_delegation();
    facts.grants.push(grant(CONTAINER_DATASET, "@backup", Who::User(ACCOUNT), Inheritance::Local));
    facts.grants.push(grant(CONTAINER_DATASET, "@backup", Who::Group(ACCOUNT_GROUP), Inheritance::Local));
    facts.grants.push(grant(CONTAINER_DATASET, "mountpoint", Who::NamedSet(String::from("@backup")), Inheritance::NotApplicable));
    let outcome: super::CheckOutcome = check_delegation(&facts);
    let super::CheckOutcome::Finding { field, detail } = outcome else {
        panic!("a set handed over twice still hands `mountpoint` over: {outcome:?}");
    };
    assert_eq!(field, "permission");
    assert_eq!(detail.matches(&format!("zfs unallow -s @backup mountpoint {CONTAINER_DATASET}")).count(), 1, "{detail}");
}

/// A set the account holds which the reply defines no member of leaves the
/// answer unknown,
/// naming the set and the holder whose grant cannot be judged.
#[test]
fn a_held_set_the_reply_does_not_define_is_unproved() {
    let facts: DelegationFacts = with_grant(grant(CONTAINER_DATASET, "@backup", Who::User(ACCOUNT), Inheritance::Local));
    let super::CheckOutcome::Unknown { reason } = check_delegation(&facts) else {
        panic!("a held set with no definition leaves the answer unknown");
    };
    assert!(reason.contains("@backup"), "{reason}");
    assert!(reason.contains("defines no member"), "{reason}");
    assert!(reason.contains("user 1000"), "{reason}");
}

/// The decided grants are the ones the design names,
/// so a change to the table turns this control red.
#[test]
fn the_decided_grants_are_the_ones_the_design_names() {
    let mut named: Vec<String> = healthy_grants().into_iter().map(|grant| return format!("{} {} {:?}", grant.carried_by, grant.permission, grant.inheritance)).collect();
    named.sort();
    assert_eq!(named.len(), 9, "{named:?}");
    assert!(named.contains(&format!("{CHECKOUT_DATASET} snapshot Local")), "{named:?}");
    assert!(named.contains(&format!("{CONTAINER_DATASET} userprop Descendants")), "{named:?}");
}

//endregion
