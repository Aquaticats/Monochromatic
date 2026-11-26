//! Controls for the removals themselves:
//!  the whole order removes the three records and the registration,
//!  the records-only order leaves the registration,
//!  a lost lock is written again and a cut-short one is completed,
//!  a person's lock refuses,
//!  and a name that still answers after its removal stops the run.

use crate::lock::single_owner_claim::{
    OwnerClaim,
    take_claim,
};
use crate::worktree_clone_binding_record::binding_words::Phase;
use crate::worktree_clone_binding_record::lock_words::lock_file_bytes;
use crate::worktree_clone_binding_record::registration_home::{
    claim_prefix,
    home_folder,
};
use crate::worktree_clone_record_settling::settling_fixtures::{
    SLOT,
    TheFixture,
    a_guid,
    binding_path,
    publish_the_record,
    the_fixture,
    write_the_pointer,
};
use crate::worktree_clone_record_settling::settling_record::read_the_record;
use crate::worktree_clone_record_settling::settling_refusal::SettlingStop;
use crate::worktree_clone_record_settling::settling_removals::{
    RemovalsGone,
    RemovalsToMake,
    make_the_removals,
};
use crate::worktree_clone_record_settling::settling_steps::{
    TheOrderFacts,
    WHEN_A_CHECKOUT_HAS_BEGUN,
    WHEN_NO_CHECKOUT_HAS_BEGUN,
    the_steps,
};

/// One fixture with a published record and this add's pointer,
/// and the claim a control holds while it removes.
fn ready(name: &str) -> (TheFixture, OwnerClaim) {
    let made: TheFixture = the_fixture(name);
    publish_the_record(&made, Phase::Filled);
    write_the_pointer(&made);
    let home: std::path::PathBuf = home_folder(made.common.as_path());
    let claim: OwnerClaim = match take_claim(home.as_path(), claim_prefix(SLOT).as_os_str())
        .expect("the claim is taken")
    {
        crate::lock::single_owner_claim::ClaimOutcome::Held(held) => held,
        crate::lock::single_owner_claim::ClaimOutcome::Busy(owner) => {
            panic!("the fixture's claim is busy with process {}", owner.pid)
        }
    };
    return (made, claim);
}

/// The names one run removes,
/// built from a fixture's record.
fn the_removals<'a>(
    made: &'a TheFixture,
    record: &'a crate::worktree_clone_record_settling::settling_record::TheRecord,
) -> RemovalsToMake<'a> {
    return RemovalsToMake {
        common_dir: made.common.as_path(),
        slot: SLOT,
        slot_path: made.slot_path.as_path(),
        registration: made.registration.as_path(),
        nonce: &record.nonce,
        clone_guid: a_guid(crate::worktree_clone_record_settling::settling_fixtures::CLONE_GUID),
    };
}

/// The whole order removes the pointer,
/// the registration and the three records,
/// and releases the claim.
#[test]
fn the_whole_order_removes_everything() {
    let (made, mut claim) = ready("settling-removals-whole");
    let record = read_the_record(made.common.as_path(), SLOT).expect("the record reads");
    let to: RemovalsToMake<'_> = the_removals(&made, &record);
    let steps = the_steps(TheOrderFacts {
        checkout_begun: false,
        plan_stands: false,
    });
    let gone: RemovalsGone =
        make_the_removals(&to, steps.as_slice(), &mut claim).expect("the removals are made");
    assert_eq!(gone.steps, steps);
    assert!(gone.lock_written, "the fixture's registration had no lock");
    assert!(gone.registration_removed);
    assert_eq!(
        gone.files,
        vec![
            String::from("topic.origin-inventory"),
            String::from("topic.clone-inventory"),
            String::from("topic.binding")
        ]
    );
    assert!(!binding_path(&made).exists());
    assert!(!made.registration.exists());
    assert!(!made.slot_path.join(".git").exists());
}

/// The records-only order leaves the registration and the pointer,
/// and writes no lock.
#[test]
fn the_records_only_order_leaves_the_registration() {
    let (made, mut claim) = ready("settling-removals-records-only");
    let record = read_the_record(made.common.as_path(), SLOT).expect("the record reads");
    let to: RemovalsToMake<'_> = the_removals(&made, &record);
    let gone: RemovalsGone = make_the_removals(&to, WHEN_A_CHECKOUT_HAS_BEGUN, &mut claim)
        .expect("the removals are made");
    assert!(!gone.lock_written, "an ordinary worktree is never locked");
    assert!(!gone.registration_removed);
    assert!(made.registration.exists());
    assert!(made.slot_path.join(".git").exists());
    assert!(!binding_path(&made).exists());
}

/// A lock that is a proper prefix of this add's sentence is completed by one write.
#[test]
fn a_cut_short_lock_is_completed() {
    let (made, mut claim) = ready("settling-removals-cut-short");
    let record = read_the_record(made.common.as_path(), SLOT).expect("the record reads");
    let whole: Vec<u8> = lock_file_bytes(&record.nonce);
    std::fs::write(made.registration.join("locked"), &whole[..whole.len() - 4])
        .expect("the cut-short lock writes");
    let to: RemovalsToMake<'_> = the_removals(&made, &record);
    let gone: RemovalsGone = make_the_removals(&to, WHEN_NO_CHECKOUT_HAS_BEGUN, &mut claim)
        .expect("the removals are made");
    assert!(gone.lock_written);
    assert!(
        gone.plan_removed,
        "the whole order names the plan even when no file stands under its GUID"
    );
}

/// A person's own words in the lock refuse,
/// and nothing is removed.
#[test]
fn a_persons_lock_refuses() {
    let (made, mut claim) = ready("settling-removals-persons-lock");
    let record = read_the_record(made.common.as_path(), SLOT).expect("the record reads");
    std::fs::write(
        made.registration.join("locked"),
        b"a person's own reason for locking this worktree\n",
    )
    .expect("the lock writes");
    let to: RemovalsToMake<'_> = the_removals(&made, &record);
    let refused = make_the_removals(&to, WHEN_NO_CHECKOUT_HAS_BEGUN, &mut claim)
        .expect_err("a person's lock refuses");
    assert!(
        matches!(refused, SettlingStop::TheLockIsNotThisAdds { .. }),
        "{refused:?}"
    );
    assert!(
        refused.sentence().contains("git worktree unlock"),
        "{}",
        refused.sentence()
    );
    assert!(binding_path(&made).exists(), "nothing was removed");
    assert!(made.registration.exists());
}

/// An empty lock file is the reasonless lock Git's own `worktree lock` leaves,
/// which a removal refuses until `git worktree unlock` (the lock design's S3).
#[test]
fn an_empty_lock_refuses_as_a_persons() {
    let (made, mut claim) = ready("settling-removals-empty-lock");
    let record = read_the_record(made.common.as_path(), SLOT).expect("the record reads");
    std::fs::write(made.registration.join("locked"), b"").expect("the empty lock writes");
    let to: RemovalsToMake<'_> = the_removals(&made, &record);
    let refused = make_the_removals(&to, WHEN_NO_CHECKOUT_HAS_BEGUN, &mut claim)
        .expect_err("an empty lock is a person's");
    assert!(
        matches!(refused, SettlingStop::TheLockIsNotThisAdds { .. }),
        "{refused:?}"
    );
    assert!(binding_path(&made).exists(), "nothing was removed");
}

/// A record that still answers after its removal stops the run there.
#[test]
fn a_record_that_still_answers_stops_the_run() {
    let (made, mut claim) = ready("settling-removals-still-answers");
    let record = read_the_record(made.common.as_path(), SLOT).expect("the record reads");
    // A directory at the binding's name cannot be removed as a file, so the removal's own answer is the stop.
    std::fs::remove_file(binding_path(&made).as_path()).expect("the binding goes");
    std::fs::create_dir(binding_path(&made).as_path()).expect("a folder takes the name");
    let to: RemovalsToMake<'_> = the_removals(&made, &record);
    let refused = make_the_removals(&to, WHEN_A_CHECKOUT_HAS_BEGUN, &mut claim)
        .expect_err("a name that stays refuses");
    assert!(
        matches!(refused, SettlingStop::TheRecordCouldNotGo { .. }),
        "{refused:?}"
    );
}
