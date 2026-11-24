//! What:
//!  Controls for the release:
//!  the verified lock released through the held descriptor,
//!  a cut-short lock released as well,
//!  and the refusals:
//!  a proof for another slot,
//!  a proof for another clone,
//!  no lock standing,
//!  and a lock that is a person's.
//! Why:
//!  The release is the one call that removes a lock the wrapper wrote,
//!  so a proof of another clone or another slot must release nothing,
//!  and a person's lock must survive every call.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(releaseTheManagedLock(registration, path, slot, facts, proof)).toEqual({ registration: path });
//! ```

use super::{ReleaseRefusal, release_the_managed_lock};
use crate::worktree_clone_managed_lock::test_fixtures::{Fixture, RecordSpec};
use crate::worktree_clone_managed_lock::{CloneDestructionProof, TheLiveCloneAnswer, prove_the_recorded_clone_is_gone, read_the_recorded_facts};
use std::path::PathBuf;

/// The fixture's nonce.
const NONCE: &str = "9f2c41d7b0a34e6c8f5d1a2b3c4d5e6f";

/// What:
///  A finished slot with the lock the caller plants,
///  the record read,
///  and the registration held open,
///  which is the shape the settle's third proof leaves.
/// Why:
///  The release takes the descriptor the settle holds,
///  so a control that passed a path would not be testing the release.
struct Settling {
    fixture: Fixture,
    registration: PathBuf,
    held: std::fs::File,
    facts: crate::worktree_clone_managed_lock::RecordedLockFacts,
    proof: CloneDestructionProof,
}

/// A fixture with one lock and a proof that its clone is gone.
fn settling(label: &str, lock: &[u8]) -> Settling {
    let fixture: Fixture = Fixture::new(label);
    let registration: PathBuf = fixture.write_registration("locked-slot");
    fixture.write_pointer(registration.as_path());
    let mut spec: RecordSpec = RecordSpec::for_nonce(NONCE);
    spec.slot_identity = Some(fixture.slot_identity());
    spec.registration = Some(registration.clone());
    fixture.write_record(&spec);
    fixture.write_lock(registration.as_path(), lock);
    let facts = read_the_recorded_facts(fixture.common_dir.as_path(), fixture.slot.as_str()).expect("the record reads");
    let proof: CloneDestructionProof = prove_the_recorded_clone_is_gone(fixture.slot.as_str(), &facts.clone, &TheLiveCloneAnswer::Absent).expect("the clone is gone");
    let held: std::fs::File = std::fs::File::open(registration.as_path()).expect("the registration is held open");
    return Settling { fixture, registration, held, facts, proof };
}

/// The verified lock is removed through the held descriptor,
/// and the outcome names the registration it went from.
#[test]
fn the_verified_lock_is_released() {
    let settling: Settling = settling("release-verified", settling_sentence().as_slice());
    let outcome = release_the_managed_lock(&settling.held, settling.registration.as_path(), settling.fixture.slot.as_str(), &settling.facts, &settling.proof).expect("the lock is released");
    assert_eq!(outcome.registration, settling.registration);
    assert!(!settling.registration.join("locked").exists(), "the lock is gone");
    assert!(settling.registration.join("gitdir").exists(), "the registration stays, because the recipe removes it next");
}

/// A cut-short lock is released as the removal's own state,
/// which is why the removal completes it before the proof is used.
#[test]
fn the_cut_short_lock_is_released() {
    let sentence: Vec<u8> = settling_sentence();
    let covering: usize = crate::worktree_clone_binding_record::LOCK_WORDS_PREFIX.len() + 32 + 1;
    let settling: Settling = settling("release-cut-short", &sentence[..covering]);
    assert!(release_the_managed_lock(&settling.held, settling.registration.as_path(), settling.fixture.slot.as_str(), &settling.facts, &settling.proof).is_ok());
    assert!(!settling.registration.join("locked").exists());
}

/// A proof made for another slot releases nothing.
#[test]
fn a_proof_for_another_slot_releases_nothing() {
    let settling: Settling = settling("release-other-slot", settling_sentence().as_slice());
    let refused = release_the_managed_lock(&settling.held, settling.registration.as_path(), "another-slot", &settling.facts, &settling.proof)
        .expect_err("another slot's proof releases nothing");
    assert!(matches!(refused, ReleaseRefusal::TheProofIsForAnotherSlot { .. }), "{refused:?}");
    assert!(settling.registration.join("locked").exists());
}

/// A proof made for another clone releases nothing,
/// even under the same slot,
/// which is what stops a stale plan settling a live clone's lock.
#[test]
fn a_proof_for_another_clone_releases_nothing() {
    let settling: Settling = settling("release-other-clone", settling_sentence().as_slice());
    let elsewhere = crate::worktree_clone_managed_lock::RecordedClone { dataset: String::from("pool/clones/another"), guid: settling.facts.clone.guid };
    let other: CloneDestructionProof = prove_the_recorded_clone_is_gone(settling.fixture.slot.as_str(), &elsewhere, &TheLiveCloneAnswer::Absent).expect("the other clone is gone too");
    let refused = release_the_managed_lock(&settling.held, settling.registration.as_path(), settling.fixture.slot.as_str(), &settling.facts, &other)
        .expect_err("another clone's proof releases nothing");
    assert!(matches!(refused, ReleaseRefusal::TheProofIsForAnotherClone { .. }), "{refused:?}");
    assert!(settling.registration.join("locked").exists());
}

/// No lock standing refuses,
/// because the settle writes a lost lock again before it releases one.
#[test]
fn a_settle_with_no_lock_refuses() {
    let settling: Settling = settling("release-no-lock", b"");
    std::fs::remove_file(settling.registration.join("locked")).expect("the lock goes");
    let refused = release_the_managed_lock(&settling.held, settling.registration.as_path(), settling.fixture.slot.as_str(), &settling.facts, &settling.proof)
        .expect_err("nothing stands to release");
    assert_eq!(refused, ReleaseRefusal::NoLockStands);
}

/// A person's lock survives the release,
/// because the wrapper never releases words its add did not write.
#[test]
fn a_persons_lock_survives_the_release() {
    let settling: Settling = settling("release-persons-lock", b"held by a person\n");
    let refused = release_the_managed_lock(&settling.held, settling.registration.as_path(), settling.fixture.slot.as_str(), &settling.facts, &settling.proof)
        .expect_err("a person's lock is not this add's");
    assert!(matches!(refused, ReleaseRefusal::NotTheAddsLock { .. }), "{refused:?}");
    assert_eq!(std::fs::read(settling.registration.join("locked")).expect("the lock reads"), b"held by a person\n");
}

/// The sentence the add writes for the fixture's nonce.
fn settling_sentence() -> Vec<u8> {
    let nonce = crate::worktree_clone_mount_protocol::RequestNonce::parse(NONCE).expect("the fixture nonce reads");
    return crate::worktree_clone_binding_record::lock_words::lock_file_bytes(&nonce);
}
