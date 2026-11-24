//! Controls for the preflight's delegation read:
//!  the decided shape passes,
//!  a missing permission refuses with the `zfs allow` command that adds it,
//!  a lineage-changing grant refuses with the `zfs unallow` command that removes
//!  it,
//!  and a delegation reached through a named set is refused as unproved.

use super::refuse_unless_the_decided_ones;
use crate::doctor_zfs_route::route_test_facts::{ACCOUNT, CHECKOUT_DATASET, CONTAINER_DATASET, grant, healthy_delegation};
use crate::doctor_zfs_route::{DelegationFacts, Inheritance, Who};

/// Facts whose one required permission is gone,
/// named so a control reads as the change it makes.
fn without(permission: &str, place: &str, inheritance: Inheritance) -> DelegationFacts {
    let mut facts: DelegationFacts = healthy_delegation();
    facts.grants.retain(|allowed| return !(allowed.permission == permission && allowed.carried_by == place && allowed.inheritance == inheritance));
    return facts;
}

/// The nine decided grants pass and nothing is refused.
#[test]
fn the_decided_delegation_passes() {
    let facts: DelegationFacts = healthy_delegation();
    assert!(refuse_unless_the_decided_ones(&facts).is_ok());
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
