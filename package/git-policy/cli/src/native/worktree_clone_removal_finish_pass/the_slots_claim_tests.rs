//! What:
//!  The controls of the pass's claim:
//!  a free slot taken and given back,
//!  a live owner ending the attempt,
//!  and the generation file's own name.
//! Why:
//!  The claim is what keeps two commands off one slot,
//!  and the name it is taken under is the slot and never the clone's dataset name.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(takeTheSlotsClaim(commonDir, slot)).toEqual({ kind: 'held', claim });
//! ```

use crate::process::process_identity::current_birth_identity;
use crate::single_owner_claim::plant_claim_for_tests;
use crate::testing::test_support::fixture;
use crate::worktree_clone_binding_record::registration_home::{claim_prefix, home_folder};
use crate::worktree_clone_removal_finish_pass::the_slots_claim::{TheClaim, give_the_claim_back, take_the_slots_claim};
use std::path::PathBuf;

/// The slot every control here claims.
const SLOT: &str = "fix-flaky-test";

/// One common directory whose wrapper home stands,
/// which is the state a plan's folder implies.
fn a_repository(name: &str) -> PathBuf {
    let common: PathBuf = fixture(name);
    std::fs::create_dir_all(home_folder(common.as_path()).as_path()).expect("the wrapper's own home");
    return common;
}

#[test]
fn a_free_slot_is_taken_and_given_back_under_the_slots_own_name() {
    let common: PathBuf = a_repository("finish-pass-claim-free");
    let mut claim = match take_the_slots_claim(common.as_path(), SLOT) {
        TheClaim::Held(held) => held,
        TheClaim::Busy(owner) => panic!("a free slot is taken, and this attempt answered busy with {}", owner.pid),
        TheClaim::Refused { said } => panic!("a free slot is taken, and this attempt refused: {said}"),
    };
    let generation: PathBuf = home_folder(common.as_path()).join("fix-flaky-test.claim.1");
    assert!(generation.exists(), "the claim file is filed under the slot's name: {}", generation.display());
    give_the_claim_back(&mut claim).expect("the claim is given back");
    let text: String = std::fs::read_to_string(generation.as_path()).expect("the released claim reads");
    assert!(text.contains("released"), "the released claim says so: {text}");
}

#[test]
fn a_live_owner_ends_the_attempt_and_names_nothing_taken() {
    let common: PathBuf = a_repository("finish-pass-claim-busy");
    let home: PathBuf = home_folder(common.as_path());
    let birth: String = current_birth_identity().expect("this process's own birth identity reads");
    plant_claim_for_tests(home.as_path(), claim_prefix(SLOT).as_os_str(), 1, i64::from(std::process::id()), birth.as_str());
    let answer: TheClaim = take_the_slots_claim(common.as_path(), SLOT);
    let TheClaim::Busy(owner) = answer else { panic!("a live owner ends the attempt") };
    assert_eq!(owner.pid, i64::from(std::process::id()), "the busy answer names the live owner");
    assert_eq!(owner.path, home.join(format!("{SLOT}.claim.1")), "and the file it holds");
}

#[test]
fn a_claim_is_written_into_the_wrapper_own_home_and_not_the_repository() {
    let common: PathBuf = a_repository("finish-pass-claim-home");
    let home: PathBuf = home_folder(common.as_path());
    let taken: TheClaim = take_the_slots_claim(common.as_path(), SLOT);
    assert!(matches!(taken, TheClaim::Held(_)), "a claim in the wrapper's own home is taken");
    let names: Vec<String> = std::fs::read_dir(home.as_path()).expect("the claim folder lists").flatten().map(|entry| return entry.file_name().to_string_lossy().into_owned()).collect();
    assert_eq!(names, vec![String::from(format!("{SLOT}.claim.1"))], "the folder holds the claim and nothing else");
    assert!(!common.join(format!("{SLOT}.claim.1")).exists(), "no claim file is written beside the repository's own records");
}

#[test]
fn a_slot_whose_claim_folder_was_removed_answers_a_refusal_and_not_a_held_claim() {
    let common: PathBuf = fixture("finish-pass-claim-no-folder");
    let answer: TheClaim = take_the_slots_claim(common.as_path(), SLOT);
    let TheClaim::Refused { said } = answer else { panic!("a claim folder that is not there proves nothing about the slot") };
    assert!(said.contains("cannot list the claims"), "the refusal says what the claim call answered: {said}");
}
