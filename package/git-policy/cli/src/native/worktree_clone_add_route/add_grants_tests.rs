//! Controls for the preflight's delegation read:
//!  the decided shape passes,
//!  a missing permission the add itself uses refuses with the `zfs allow`
//!  command that adds it,
//!  a missing permission only a removal uses warns and the add goes on,
//!  a lineage-changing grant refuses with the `zfs unallow` command that removes
//!  it,
//!  and a delegation reached through a named set is refused as unproved.

use super::{THE_ADDS_OWN_STEPS, refuse_unless_the_decided_ones};
use crate::doctor_zfs_route::route_test_facts::{ACCOUNT, CHECKOUT_DATASET, CONTAINER_DATASET, grant, healthy_delegation};
use crate::doctor_zfs_route::{DelegationFacts, Inheritance, Who};

/// The dataset above both the checkout and the clone container,
/// which a fixture grants a permission on when the design scopes it lower.
const ANCESTOR_DATASET: &str = "zpcachyos/ROOT/cos/home";

/// Facts whose one required permission is gone,
/// named so a control reads as the change it makes.
fn without(permission: &str, place: &str, inheritance: Inheritance) -> DelegationFacts {
    let mut facts: DelegationFacts = healthy_delegation();
    facts.grants.retain(|allowed| return !(allowed.permission == permission && allowed.carried_by == place && allowed.inheritance == inheritance));
    return facts;
}

/// The nine decided grants pass, print nothing and are not refused.
#[test]
fn the_decided_delegation_passes() {
    let facts: DelegationFacts = healthy_delegation();
    let warnings: Vec<String> = refuse_unless_the_decided_ones(&facts).expect("the decided shape passes");
    assert!(warnings.is_empty(), "a whole delegation earns no line: {warnings:?}");
}

/// A permission the add itself uses, both `destroy` grants among them, refuses
/// and never warns.
#[test]
fn every_permission_the_add_uses_refuses_when_it_is_missing() {
    for (place, permission) in THE_ADDS_OWN_STEPS {
        let dataset: &str = match place {
            crate::doctor_zfs_route::GrantPlace::Checkout => CHECKOUT_DATASET,
            crate::doctor_zfs_route::GrantPlace::CheckoutDescendants => CHECKOUT_DATASET,
            crate::doctor_zfs_route::GrantPlace::Container => CONTAINER_DATASET,
            crate::doctor_zfs_route::GrantPlace::ContainerDescendants => CONTAINER_DATASET,
        };
        let inheritance: Inheritance = match place {
            crate::doctor_zfs_route::GrantPlace::Checkout | crate::doctor_zfs_route::GrantPlace::Container => Inheritance::Local,
            _descendants => Inheritance::Descendants,
        };
        let refusal: String = refuse_unless_the_decided_ones(&without(permission, dataset, inheritance)).expect_err(permission).message;
        assert!(refusal.contains(&format!("`{permission}`")), "{permission}: {refusal}");
        assert!(refusal.contains(&format!("zfs allow -{scope} -u {ACCOUNT} {permission} {dataset}", scope = if inheritance == Inheritance::Local { "l" } else { "d" })), "{permission}: {refusal}");
    }
}

/// The checkout-origin `mount` is a removal's, so the add warns once with the
/// `zfs allow` command and goes on.
#[test]
fn a_missing_permission_only_a_removal_uses_warns_and_the_add_goes_on() {
    let facts: DelegationFacts = without("mount", CHECKOUT_DATASET, Inheritance::Descendants);
    let warnings: Vec<String> = refuse_unless_the_decided_ones(&facts).expect("the add never mounts a checkout snapshot");
    assert_eq!(warnings.len(), 1, "one line, printed once: {warnings:?}");
    let line: &str = warnings[0].as_str();
    assert!(line.contains(&format!("zfs allow -d -u {ACCOUNT} mount {CHECKOUT_DATASET}")), "{line}");
    assert!(line.contains("this add will finish"), "{line}");
    assert!(line.contains("a removal cannot do its part"), "{line}");
    assert!(line.starts_with("monochromatic-git-policy-cli: "), "{line}");
}

/// A permission the add itself writes with is refused before the claim,
/// naming where it belongs and the `zfs allow` command that puts it there.
#[test]
fn a_missing_permission_refuses_with_the_allow_command() {
    let facts: DelegationFacts = without("snapshot", CHECKOUT_DATASET, Inheritance::Local);
    let refusal: String = refuse_unless_the_decided_ones(&facts).expect_err("the snapshot permission is gone").message;
    assert!(refusal.contains("`snapshot`"), "{refusal}");
    assert!(refusal.contains(&format!("zfs allow -l -u {ACCOUNT} snapshot {CHECKOUT_DATASET}")), "{refusal}");
    assert!(refusal.contains("then run the add again"), "{refusal}");
}

/// A grant that would let the account change the lineage the removal proof
/// rests on is refused,
/// naming the `zfs unallow` command that removes it.
#[test]
fn a_lineage_changing_grant_refuses_with_the_unallow_command() {
    let mut facts: DelegationFacts = healthy_delegation();
    facts.grants.push(grant(CONTAINER_DATASET, "promote", Who::User(ACCOUNT), Inheritance::Local));
    let refusal: String = refuse_unless_the_decided_ones(&facts).expect_err("the account may promote a clone").message;
    assert!(refusal.contains("`promote`"), "{refusal}");
    assert!(refusal.contains(&format!("zfs unallow -l -u {ACCOUNT} promote {CONTAINER_DATASET}")), "{refusal}");
}

/// A delegation that reaches the account through a named set refuses as
/// unproved,
/// because the kernel's reply does not expand the set's members.
#[test]
fn a_delegation_through_a_named_set_refuses_as_unproved() {
    let mut facts: DelegationFacts = healthy_delegation();
    facts.grants.push(grant(CONTAINER_DATASET, "userprop", Who::NamedSet(String::from("backup")), Inheritance::Descendants));
    let refusal: String = refuse_unless_the_decided_ones(&facts).expect_err("the set's members are not in the reply").message;
    assert!(refusal.contains("named set backup"), "{refusal}");
    assert!(refusal.contains("unproved"), "{refusal}");
}

/// A permission granted wider than the design scopes it refuses the add with
/// the two commands that narrow it,
/// because the refusal carries the `scope` finding's own line.
#[test]
fn a_wide_permission_refuses_with_the_two_narrowing_commands() {
    let mut facts: DelegationFacts = without("destroy", CONTAINER_DATASET, Inheritance::Descendants);
    facts.grants.push(grant(ANCESTOR_DATASET, "destroy", Who::User(ACCOUNT), Inheritance::Descendants));
    let refusal: String = refuse_unless_the_decided_ones(&facts).expect_err("a wider grant refuses").message;
    assert!(refusal.contains(&format!("zfs allow -d -u {ACCOUNT} destroy {CONTAINER_DATASET}")), "{refusal}");
    assert!(refusal.contains(&format!("zfs unallow -d -u {ACCOUNT} destroy {ANCESTOR_DATASET}")), "{refusal}");
    assert!(refusal.contains("then run the add again"), "{refusal}");
}
