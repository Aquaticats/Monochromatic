//! Controls for a refusal's sentence:
//!  every arm names what it found and a way forward,
//!  and none of them is empty.

use crate::worktree_clone_binding_record::*;

/// Every refusal names its subject and at least one way forward.
#[test]
fn every_refusal_sentence_names_its_subject_and_a_way_forward() {
    let refusals: [RecordRefusal; 8] = [
        RecordRefusal::TooLarge {
            found: 50_000,
            bound: 43_027,
        },
        RecordRefusal::NotUtf8 {
            line: 2,
        },
        RecordRefusal::Header {
            found: String::from("# other"),
        },
        RecordRefusal::UnknownKey {
            line: 4,
            key: String::from("tariff"),
        },
        RecordRefusal::Missing {
            key: String::from("nonce"),
            phase: String::from("claimed"),
        },
        RecordRefusal::AheadOfItsPhase {
            key: String::from("pointer"),
            phase: String::from("slot"),
        },
        RecordRefusal::EntryCount {
            stated: 2,
            found: 1,
        },
        RecordRefusal::Kind {
            found: String::from("clone"),
            expected: String::from("origin"),
        },
    ];
    for refusal in refusals {
        let message: String = refusal.message();
        assert!(message.contains("Nothing was changed"), "{}", message);
        assert!(message.contains("Ways forward:"), "{}", message);
    }
}

/// The size refusal prints both numbers it compared.
#[test]
fn the_size_refusal_prints_both_numbers() {
    let message: String = RecordRefusal::TooLarge {
        found: 50_000,
        bound: 43_027,
    }
    .message();
    assert!(message.contains("50000"));
    assert!(message.contains("43027"));
}
