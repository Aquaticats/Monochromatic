//! What:
//!  Controls for the record facts a lock's proof reads:
//!  the nonce,
//!  the slot,
//!  the registration,
//!  the two identities and the clone,
//!  and the refusals a record owes.
//! Why:
//!  The check order takes the nonce from a record whatever its phase,
//!  and one damage must not be read as another:
//!  a record written by another account,
//!  one another account could rewrite,
//!  one whose grammar is broken,
//!  and one whose nonce is absent are four different sentences.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(readTheRecordedFacts(commonDir, slot).nonce).toBe(nonce);
//! ```

use super::{
    RecordedFactsRefusal,
    read_the_recorded_facts,
};
use crate::worktree_clone_managed_lock::test_fixtures::{
    Fixture,
    RecordSpec,
};
use std::os::unix::fs::PermissionsExt as _;
use std::path::PathBuf;

/// The fixture's nonce.
const NONCE: &str = "9f2c41d7b0a34e6c8f5d1a2b3c4d5e6f";

/// The facts of a complete record are read,
/// and the registration and its identity travel with them.
#[test]
fn the_facts_of_a_complete_record_are_read() {
    let fixture: Fixture = Fixture::new("facts-complete");
    let registration: PathBuf = fixture.write_registration("entry");
    let mut spec: RecordSpec = RecordSpec::for_nonce(NONCE);
    spec.slot_identity = Some(fixture.slot_identity());
    spec.registration = Some(registration.clone());
    spec.registration_identity = Some(String::from("42:7:8"));
    fixture.write_record(&spec);
    let facts = read_the_recorded_facts(fixture.common_dir.as_path(), fixture.slot.as_str())
        .expect("the record reads");
    assert_eq!(facts.nonce.as_str(), NONCE);
    assert_eq!(facts.slot, fixture.slot_path);
    assert_eq!(facts.registration, Some(registration));
    assert_eq!(
        facts
            .registration_identity
            .map(|identity| return identity.encode()),
        Some(String::from("42:7:8"))
    );
    assert_eq!(facts.clone.dataset, spec.clone_dataset);
    assert_eq!(
        facts.clone.guid.map(|guid| return guid.file_name()),
        spec.clone_guid
    );
}

/// A record whose phase is ahead of the live facts still gives the nonce,
/// which is the limit the lock's proof rests on.
#[test]
fn a_record_ahead_of_the_live_facts_still_gives_the_nonce() {
    let fixture: Fixture = Fixture::new("facts-ahead");
    let mut spec: RecordSpec = RecordSpec::for_nonce(NONCE);
    spec.extra_lines = vec![
        String::from("phase=settling"),
        String::from("settleReason=mount-request"),
        String::from("unknownKey=whatever"),
    ];
    fixture.write_record(&spec);
    let facts = read_the_recorded_facts(fixture.common_dir.as_path(), fixture.slot.as_str())
        .expect("the record reads for its nonce");
    assert_eq!(facts.nonce.as_str(), NONCE);
    assert_eq!(facts.registration, None);
}

/// No record stands under the slot's name.
#[test]
fn an_absent_record_is_its_own_refusal() {
    let fixture: Fixture = Fixture::new("facts-absent");
    assert_eq!(
        read_the_recorded_facts(fixture.common_dir.as_path(), fixture.slot.as_str()).err(),
        Some(RecordedFactsRefusal::Absent)
    );
}

/// A record whose nonce line is gone cannot recompose any sentence.
#[test]
fn a_record_with_no_nonce_is_refused() {
    let fixture: Fixture = Fixture::new("facts-no-nonce");
    let mut spec: RecordSpec = RecordSpec::for_nonce(NONCE);
    spec.extra_lines = Vec::new();
    let path: PathBuf = fixture.write_record(&spec);
    let text: String = std::fs::read_to_string(path.as_path())
        .expect("the record reads")
        .lines()
        .filter(|line| return !line.starts_with("nonce="))
        .collect::<Vec<&str>>()
        .join("\n");
    std::fs::write(path.as_path(), text.as_bytes())
        .expect("the record is written without its nonce");
    assert_eq!(
        read_the_recorded_facts(fixture.common_dir.as_path(), fixture.slot.as_str()).err(),
        Some(RecordedFactsRefusal::MissingField {
            key: crate::worktree_clone_managed_lock::binding_record_fields::NONCE_KEY
        })
    );
}

/// A key that arrives twice is damage rather than a later grammar.
#[test]
fn a_repeated_key_is_refused() {
    let fixture: Fixture = Fixture::new("facts-repeated");
    let mut spec: RecordSpec = RecordSpec::for_nonce(NONCE);
    let repeated: String = format!("cloneDataset={}", spec.clone_dataset);
    spec.extra_lines = vec![repeated];
    fixture.write_record(&spec);
    assert!(matches!(
        read_the_recorded_facts(fixture.common_dir.as_path(), fixture.slot.as_str()),
        Err(RecordedFactsRefusal::NotThisGrammar { found, .. }) if found == "cloneDataset"
    ));
}

/// A record another account could rewrite is no fact about this add.
#[test]
fn a_record_another_account_could_rewrite_is_refused() {
    let fixture: Fixture = Fixture::new("facts-writable");
    let mut spec: RecordSpec = RecordSpec::for_nonce(NONCE);
    spec.slot_identity = Some(fixture.slot_identity());
    let path: PathBuf = fixture.write_record(&spec);
    std::fs::set_permissions(path.as_path(), std::fs::Permissions::from_mode(0o666))
        .expect("the mode is widened");
    assert_eq!(
        read_the_recorded_facts(fixture.common_dir.as_path(), fixture.slot.as_str()).err(),
        Some(RecordedFactsRefusal::WritableByOthers {
            mode: 0o666
        })
    );
}

/// A link at the record's name is refused,
/// and so is a nonce that is not thirty-two lowercase hexadecimal digits.
#[test]
fn a_link_and_a_bad_nonce_are_refused() {
    let fixture: Fixture = Fixture::new("facts-link-and-nonce");
    let mut spec: RecordSpec = RecordSpec::for_nonce(NONCE);
    spec.slot_identity = Some(fixture.slot_identity());
    let path: PathBuf = fixture.write_record(&spec);
    let elsewhere: PathBuf = fixture.root.join("elsewhere");
    std::fs::rename(path.as_path(), elsewhere.as_path()).expect("the record is moved aside");
    std::os::unix::fs::symlink(elsewhere.as_path(), path.as_path())
        .expect("a link takes the record's name");
    assert!(matches!(
        read_the_recorded_facts(fixture.common_dir.as_path(), fixture.slot.as_str()),
        Err(RecordedFactsRefusal::Refused(_cause))
    ));
    std::fs::remove_file(path.as_path()).expect("the link goes");
    let mut bad: RecordSpec = RecordSpec::for_nonce("not-a-nonce");
    bad.slot_identity = Some(fixture.slot_identity());
    fixture.write_record(&bad);
    assert_eq!(
        read_the_recorded_facts(fixture.common_dir.as_path(), fixture.slot.as_str()).err(),
        Some(RecordedFactsRefusal::Malformed {
            key: crate::worktree_clone_managed_lock::binding_record_fields::NONCE_KEY,
            found: String::from("not-a-nonce")
        })
    );
}

/// Every refusal reads as its own sentence naming the record.
#[test]
fn every_refusal_names_the_record() {
    let fixture: Fixture = Fixture::new("facts-words");
    let home: PathBuf = fixture.home();
    let refusals: [RecordedFactsRefusal; 6] = [
        RecordedFactsRefusal::Absent,
        RecordedFactsRefusal::Refused(crate::worktree_clone_managed_lock::ARefusedReading::ALink),
        RecordedFactsRefusal::AnotherAccount {
            owner: 1,
            reader: 2,
        },
        RecordedFactsRefusal::WritableByOthers {
            mode: 0o666,
        },
        RecordedFactsRefusal::NotThisGrammar {
            line: 2,
            found: String::from("x"),
        },
        RecordedFactsRefusal::MissingField {
            key: crate::worktree_clone_managed_lock::binding_record_fields::NONCE_KEY,
        },
    ];
    for refusal in refusals {
        let words: String = refusal.words(fixture.slot.as_str(), home.as_path());
        assert!(
            words.contains("locked-slot.binding"),
            "the refusal {refusal:?} names no record: {words}"
        );
    }
}
