//! Controls for the two removal orders:
//!  the whole order with no checkout begun,
//!  the records-only order with a checkout begun,
//!  and a plan's step left out when no plan stands.

use crate::worktree_clone_record_settling::settling_steps::{SettlingStep, TheOrderFacts, WHEN_A_CHECKOUT_HAS_BEGUN, WHEN_NO_CHECKOUT_HAS_BEGUN, the_steps};

/// The whole order is the design's,
/// and its last step is the claim.
#[test]
fn the_whole_order_is_the_designs() {
    assert_eq!(
        WHEN_NO_CHECKOUT_HAS_BEGUN,
        [
            SettlingStep::TheLostLock,
            SettlingStep::TheRegistrationRecipe,
            SettlingStep::ThePlan,
            SettlingStep::TheOriginInventory,
            SettlingStep::TheCloneInventory,
            SettlingStep::TheBinding,
            SettlingStep::TheClaim,
        ]
    );
    assert_eq!(the_steps(TheOrderFacts { checkout_begun: false, plan_stands: true }), WHEN_NO_CHECKOUT_HAS_BEGUN.to_vec());
}

/// The plan goes before both inventories,
/// the binding after them,
/// and the claim last,
/// which is the order the design states and a sibling quotes.
#[test]
fn the_plan_precedes_the_inventories_and_the_binding_precedes_the_claim() {
    let steps = the_steps(TheOrderFacts { checkout_begun: false, plan_stands: true });
    let at = |step: SettlingStep| return steps.iter().position(|held| return *held == step).expect("the step stands");
    assert!(at(SettlingStep::ThePlan) < at(SettlingStep::TheOriginInventory));
    assert!(at(SettlingStep::TheOriginInventory) < at(SettlingStep::TheCloneInventory));
    assert!(at(SettlingStep::TheCloneInventory) < at(SettlingStep::TheBinding));
    assert!(at(SettlingStep::TheBinding) < at(SettlingStep::TheClaim));
}

/// With a checkout begun the registration and the lock stay,
/// and the records still go in the design's order.
#[test]
fn a_checkout_leaves_the_registration_and_the_lock_alone() {
    assert_eq!(
        WHEN_A_CHECKOUT_HAS_BEGUN,
        [SettlingStep::ThePlan, SettlingStep::TheOriginInventory, SettlingStep::TheCloneInventory, SettlingStep::TheBinding, SettlingStep::TheClaim]
    );
    let steps = the_steps(TheOrderFacts { checkout_begun: true, plan_stands: true });
    assert!(!steps.contains(&SettlingStep::TheLostLock), "{steps:?}");
    assert!(!steps.contains(&SettlingStep::TheRegistrationRecipe), "{steps:?}");
}

/// A run with no plan standing leaves the plan's step out rather than making it a no-op.
#[test]
fn no_plan_leaves_its_step_out() {
    assert!(!the_steps(TheOrderFacts { checkout_begun: false, plan_stands: false }).contains(&SettlingStep::ThePlan));
    assert!(!the_steps(TheOrderFacts { checkout_begun: true, plan_stands: false }).contains(&SettlingStep::ThePlan));
}

/// Each step names itself in the words a line prints.
#[test]
fn every_step_names_itself() {
    for step in WHEN_NO_CHECKOUT_HAS_BEGUN {
        assert!(!step.words().is_empty());
    }
}
