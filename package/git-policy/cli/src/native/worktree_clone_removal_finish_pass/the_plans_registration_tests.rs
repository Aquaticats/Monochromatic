//! What:
//!  The controls of the plan's registration:
//!  the directory its own name holds,
//!  the three identities it can answer,
//!  and the settle the pass asks for once the absence is proved.
//! Why:
//!  A registration of another identity is another removal's,
//!  so the pass settles only its own,
//!  and the settle is reached only with a proof that names the plan's slot and clone.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(theRegistrationStandingFor(commonDir, plan)).toBe('this-plans');
//! ```

use crate::worktree_clone_removal_finish_pass::pass_fixtures::{
    ASettle,
    SLOT,
    a_finished_removal,
    a_plan_for,
    the_home,
    the_registrations_identity,
};
use crate::worktree_clone_removal_finish_pass::the_plans_registration::{
    RegistrationStanding,
    settle_the_plans_registration,
    the_registration_standing_for,
};
use crate::worktree_clone_removal_proof_and_plan::plan_record::DestructionPlan;
use crate::worktree_clone_removal_proof_and_plan::plan_words::HeadWord;
use crate::worktree_clone_removal_proof_and_plan::removal_fixtures::RemovalFixture;
use crate::worktree_clone_removal_proof_and_plan::removal_stop::RemovalStop;

#[test]
fn the_directory_the_plans_own_name_holds_is_the_plans_registration() {
    let (fixture, plan) = a_finished_removal("finish-pass-registration-this");
    assert_eq!(
        the_registration_standing_for(fixture.common(), &plan),
        RegistrationStanding::ThisPlans,
        "the fixture's registration answers the plan's own identity"
    );
}

#[test]
fn a_directory_of_another_identity_is_another_removals() {
    let (fixture, plan) = a_finished_removal("finish-pass-registration-other");
    std::fs::remove_dir_all(fixture.registration.as_path()).expect("the plan's registration went");
    std::fs::create_dir_all(fixture.registration.as_path())
        .expect("a later add made another registration here");
    assert_eq!(
        the_registration_standing_for(fixture.common(), &plan),
        RegistrationStanding::Anothers,
        "a directory of another identity is another removal's"
    );
}

#[test]
fn a_registration_that_is_not_there_is_an_absence() {
    let (fixture, plan) = a_finished_removal("finish-pass-registration-absent");
    std::fs::remove_dir_all(fixture.registration.as_path())
        .expect("the registration was settled already");
    assert_eq!(
        the_registration_standing_for(fixture.common(), &plan),
        RegistrationStanding::Absent
    );
}

#[test]
fn a_plan_that_names_no_registration_name_is_unreadable_rather_than_absent() {
    let (fixture, mut plan) = a_finished_removal("finish-pass-registration-empty");
    plan.facts.registration_id = Vec::new();
    let standing: RegistrationStanding = the_registration_standing_for(fixture.common(), &plan);
    assert!(
        matches!(standing, RegistrationStanding::Unreadable { .. }),
        "an empty name proves nothing, and this answer was {standing:?}"
    );
}

#[test]
fn the_settle_is_asked_once_the_absence_is_proved_and_carries_the_plans_own_identity() {
    let (fixture, plan) = a_finished_removal("finish-pass-registration-settle");
    let settle: ASettle = ASettle::succeeding();
    settle_the_plans_registration(fixture.common(), SLOT, &plan, &settle)
        .expect("the settle answers");
    assert_eq!(settle.calls.get(), 1, "the settle was asked once");
    assert_eq!(
        settle.slots.borrow().as_slice(),
        &[String::from(SLOT)],
        "the settle is asked about the plan's own slot"
    );
    assert_eq!(
        the_registrations_identity(&fixture),
        plan.facts.admin,
        "the plan records the registration's own identity"
    );
}

#[test]
fn a_slot_whose_binding_is_gone_is_no_settle_at_all() {
    let fixture: RemovalFixture = RemovalFixture::healthy("finish-pass-registration-no-binding");
    let plan: DestructionPlan = a_plan_for(&fixture, HeadWord::Gone);
    std::fs::remove_file(the_home(&fixture).join(format!("{SLOT}.binding")).as_path())
        .expect("the binding went");
    let settle: ASettle = ASettle::succeeding();
    let stop: RemovalStop = settle_the_plans_registration(fixture.common(), SLOT, &plan, &settle)
        .expect_err("a settle without its facts refuses");
    assert!(
        matches!(stop, RemovalStop::TheSettleRefused { .. }),
        "the stop names the settle, and this one was {stop:?}"
    );
    assert_eq!(
        settle.calls.get(),
        0,
        "no settle is asked without the facts it settles"
    );
}

#[test]
fn the_registration_lives_under_the_common_directory_and_not_beside_it() {
    let (a_repository, _plan) = a_finished_removal("finish-pass-registration-folders");
    assert!(
        a_repository
            .registration
            .starts_with(a_repository.common_dir.as_path()),
        "the registration lives under the common directory's own `worktrees` folder"
    );
}
