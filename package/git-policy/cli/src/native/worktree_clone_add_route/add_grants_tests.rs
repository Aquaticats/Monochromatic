//! Controls for the preflight's delegation read:
//!  the decided shape passes,
//!  a missing permission the add itself uses refuses with the `zfs allow`
//!  command that adds it,
//!  a missing permission only a removal uses warns and the add goes on,
//!  a lineage-changing grant refuses with the `zfs unallow` command that removes
//!  it,
//!  and a delegation reached through a named set is refused as unproved.

use super::{A_REMOVALS_STEPS, THE_ADDS_OWN_STEPS, refuse_unless_the_decided_ones};
use crate::doctor_zfs_route::route_test_facts::{ACCOUNT, CHECKOUT_DATASET, CONTAINER_DATASET, grant, healthy_delegation};
use crate::doctor_zfs_route::{DelegationFacts, GrantPlace, Inheritance, REQUIRED_GRANTS, Who};

/// The dataset above both the checkout and the clone container,
/// which a fixture grants a permission on when the design scopes it lower.
const ANCESTOR_DATASET: &str = "zpcachyos/ROOT/cos/home";

/// The six permissions choice 3C says the add's own steps use,
/// written from the design's "Scoped delegation" so a table that moves one of
/// them is read against the record rather than against itself.
const THE_ADDS_GROUP: [(GrantPlace, &str); 6] = [
    (GrantPlace::Checkout, "snapshot"),
    (GrantPlace::CheckoutDescendants, "clone"),
    (GrantPlace::Container, "create"),
    (GrantPlace::Container, "mount"),
    (GrantPlace::ContainerDescendants, "mount"),
    (GrantPlace::ContainerDescendants, "userprop"),
];

/// The three permissions choice 3C says only a removal uses,
/// which the add warns about and never refuses for while no step of its own
/// issues a `destroy`.
const A_REMOVALS_GROUP: [(GrantPlace, &str); 3] = [
    (GrantPlace::ContainerDescendants, "destroy"),
    (GrantPlace::CheckoutDescendants, "mount"),
    (GrantPlace::CheckoutDescendants, "destroy"),
];

/// Facts whose one required permission is gone,
/// named so a control reads as the change it makes.
fn without(permission: &str, place: &str, inheritance: Inheritance) -> DelegationFacts {
    let mut facts: DelegationFacts = healthy_delegation();
    facts.grants.retain(|allowed| return !(allowed.permission == permission && allowed.carried_by == place && allowed.inheritance == inheritance));
    return facts;
}

/// The dataset and inheritance a place names,
/// which a control removes the one grant of.
fn place_of(place: GrantPlace) -> (&'static str, Inheritance) {
    return match place {
        GrantPlace::Checkout => (CHECKOUT_DATASET, Inheritance::Local),
        GrantPlace::CheckoutDescendants => (CHECKOUT_DATASET, Inheritance::Descendants),
        GrantPlace::Container => (CONTAINER_DATASET, Inheritance::Local),
        GrantPlace::ContainerDescendants => (CONTAINER_DATASET, Inheritance::Descendants),
    };
}

/// The nine decided grants pass, print nothing and are not refused.
#[test]
fn the_decided_delegation_passes() {
    let facts: DelegationFacts = healthy_delegation();
    let warnings: Vec<String> = refuse_unless_the_decided_ones(&facts).expect("the decided shape passes");
    assert!(warnings.is_empty(), "a whole delegation earns no line: {warnings:?}");
}

/// The release's two tables are the record's six and three.
#[test]
fn the_split_is_the_records_six_and_three() {
    assert_eq!(THE_ADDS_OWN_STEPS.as_slice(), THE_ADDS_GROUP.as_slice(), "the add's own steps are the six the design's \"Scoped delegation\" gives the unprivileged lifecycle");
    assert_eq!(A_REMOVALS_STEPS.as_slice(), A_REMOVALS_GROUP.as_slice(), "a removal's steps are the three the add never runs, and both `destroy` grants stay here while the add's own failure path issues none");
}

/// A permission the add itself uses refuses
/// and never warns.
#[test]
fn every_permission_the_add_uses_refuses_when_it_is_missing() {
    for (place, permission) in THE_ADDS_GROUP {
        let (dataset, inheritance): (&str, Inheritance) = place_of(place);
        let refusal: String = refuse_unless_the_decided_ones(&without(permission, dataset, inheritance)).expect_err(permission).message;
        assert!(refusal.contains(&format!("`{permission}`")), "{permission}: {refusal}");
        assert!(refusal.contains(&format!("zfs allow -{scope} -u {ACCOUNT} {permission} {dataset}", scope = if inheritance == Inheritance::Local { "l" } else { "d" })), "{permission}: {refusal}");
    }
}

/// A permission only a removal uses warns with its `zfs allow` command and the
/// add goes on,
/// both `destroy` grants and the checkout-origin `mount` among them.
#[test]
fn every_permission_only_a_removal_uses_warns_and_the_add_goes_on() {
    for (place, permission) in A_REMOVALS_GROUP {
        let (dataset, inheritance): (&str, Inheritance) = place_of(place);
        let facts: DelegationFacts = without(permission, dataset, inheritance);
        let warnings: Vec<String> = refuse_unless_the_decided_ones(&facts).unwrap_or_else(|refusal| panic!("no add step uses `{permission}` on `{dataset}`, so the add must go on: {}", refusal.message));
        assert_eq!(warnings.len(), 1, "{permission}: one line, printed once: {warnings:?}");
        let line: &str = warnings[0].as_str();
        assert!(line.contains(&format!("zfs allow -d -u {ACCOUNT} {permission} {dataset}")), "{permission}: {line}");
        assert!(line.contains("this add will finish"), "{permission}: {line}");
        assert!(line.contains("a removal cannot do its part"), "{permission}: {line}");
        assert!(line.starts_with("monochromatic-git-policy-cli: "), "{permission}: {line}");
    }
}

/// Every missing permission of the removal's group prints its own `zfs allow`
/// in the one line,
/// so a machine missing all three reads three commands and runs them once.
#[test]
fn the_whole_removal_group_missing_prints_one_line_with_every_command() {
    let mut facts: DelegationFacts = healthy_delegation();
    for (place, permission) in A_REMOVALS_GROUP {
        let (dataset, inheritance): (&str, Inheritance) = place_of(place);
        facts.grants.retain(|allowed| return !(allowed.permission == permission && allowed.carried_by == dataset && allowed.inheritance == inheritance));
    }
    let warnings: Vec<String> = refuse_unless_the_decided_ones(&facts).expect("no add step uses a removal's permission");
    assert_eq!(warnings.len(), 1, "one line for the whole group: {warnings:?}");
    let line: &str = warnings[0].as_str();
    for (place, permission) in A_REMOVALS_GROUP {
        let (dataset, _inheritance): (&str, Inheritance) = place_of(place);
        assert!(line.contains(&format!("zfs allow -d -u {ACCOUNT} {permission} {dataset}")), "`{permission}` on `{dataset}` earns its command: {line}");
    }
    assert_eq!(line.matches("zfs allow").count(), A_REMOVALS_GROUP.len(), "each command prints once: {line}");
}

/// A removal's permission missing beside a wider grant still refuses with the
/// check's own two commands,
/// because the breadth is a finding whatever group the missing permission is in.
#[test]
fn a_wide_grant_refuses_even_beside_a_missing_removal_permission() {
    let mut facts: DelegationFacts = without("destroy", CONTAINER_DATASET, Inheritance::Descendants);
    facts.grants.push(grant(ANCESTOR_DATASET, "mount", Who::User(ACCOUNT), Inheritance::Descendants));
    facts.grants.retain(|allowed| return !(allowed.permission == "mount" && allowed.carried_by == CONTAINER_DATASET && allowed.inheritance == Inheritance::Descendants));
    let refusal: String = refuse_unless_the_decided_ones(&facts).expect_err("a wider grant refuses whatever group the missing permission is in").message;
    assert!(refusal.contains(&format!("zfs allow -d -u {ACCOUNT} mount {CONTAINER_DATASET}")), "the decided grant comes first: {refusal}");
    assert!(refusal.contains(&format!("zfs unallow -d -u {ACCOUNT} mount {ANCESTOR_DATASET}")), "the wider grant goes after it: {refusal}");
}

/// The two tables name every decided permission once,
/// so a permission neither table names is a table that drifted rather than a
/// permission that moved.
#[test]
fn the_two_tables_hold_each_decided_permission_once() {
    for required in REQUIRED_GRANTS {
        let in_the_adds: bool = THE_ADDS_OWN_STEPS.iter().any(|(place, permission)| return *place == required.place && *permission == required.permission);
        let in_a_removals: bool = A_REMOVALS_STEPS.iter().any(|(place, permission)| return *place == required.place && *permission == required.permission);
        assert!(
            in_the_adds != in_a_removals,
            "`{}` on {:?} is in {} of the two tables, so the split has drifted from the decided set",
            required.permission,
            required.place,
            if in_the_adds || in_a_removals { "both" } else { "neither" }
        );
    }
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
