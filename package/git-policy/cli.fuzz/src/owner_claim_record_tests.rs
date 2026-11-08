//! What:
//! Controls proving the claim-text generator reaches every outcome,
//! and that the decoder's invariants hold on every generated and hand-written claim.
//! Why:
//! An invariant that no input reaches proves nothing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(outcomesReachedBy(generatedClaimText)).toEqual(['released', 'owned', 'refused']);
//! ```

/// Import the generators and invariants under control.
use super::{check_claim_text, check_generated_claim_text, generated_claim_text};
/// The decoder and the record it returns.
use git_policy_cli::single_owner_claim::{ClaimRecord, decode};
/// The folder names a claim file's first line is spelled with.
use git_policy_cli::wrapper_names_old_and_new::DATA_FOLDER;
/// The generated inputs the owner-record and index-lock controls already share.
use crate::owners::tests::inputs;

/// The first line of a claim file written under one folder name.
fn header_of(folder: &str) -> String {
    return format!("{folder} owner-claim v1");
}

/// Generated claim files are released,
/// owned with a process id alone,
/// reached through either folder spelling,
/// and refused,
/// and every input holds the invariants.
#[test]
fn claim_texts_reach_every_outcome() {
    let mut released: usize = 0;
    let mut owned: usize = 0;
    let mut earlier: usize = 0;
    let mut refused: usize = 0;
    for data in inputs() {
        check_generated_claim_text(data.as_slice());
        let (text, expected) = generated_claim_text(data.as_slice());
        match expected {
            Some(ClaimRecord::Released) => released += 1,
            Some(ClaimRecord::Owned { .. }) => owned += 1,
            None => refused += 1,
        }
        if text.starts_with(&header_of(DATA_FOLDER.earlier)) && decode(text.as_str()).is_some() {
            earlier += 1;
        }
    }
    assert!(released > 100, "{released}");
    assert!(owned > 100, "{owned}");
    assert!(refused > 100, "{refused}");
    assert!(earlier > 10, "{earlier}");
}

/// Known spellings hold the invariants and mean what the command's own tests say they mean.
#[test]
fn known_claim_spellings_hold_the_invariants() {
    let current: String = header_of(DATA_FOLDER.current);
    let earlier: String = header_of(DATA_FOLDER.earlier);
    let owned: String = format!("{current}\npid=42\nbirth=linux:99\nnamespace=pid:[4026531836]\n");
    assert_eq!(
        decode(owned.as_str()),
        Some(ClaimRecord::Owned {
            pid: 42,
            birth: String::from("linux:99"),
            namespace: Some(String::from("pid:[4026531836]")),
        })
    );
    let cases: Vec<(String, Option<ClaimRecord>)> = vec![
        (format!("{current}\nreleased\n"), Some(ClaimRecord::Released)),
        (format!("{earlier}\nreleased\n"), Some(ClaimRecord::Released)),
        (format!("{current}\nreleased\npid=42\n"), Some(ClaimRecord::Released)),
        (owned.clone(), decode(owned.as_str())),
        (format!("{current}\npid=007\nbirth=t\nnamespace=-\n"), Some(ClaimRecord::Owned {
            pid: 7,
            birth: String::from("t"),
            namespace: None,
        })),
        (format!("{current}\npid=0\nbirth=t\nnamespace=-\n"), None),
        (format!("{current}\npid=-1\nbirth=t\nnamespace=-\n"), None),
        (format!("{current}\npid=42\nbirth=\nnamespace=-\n"), None),
        (format!("{current}\npid=42\nbirth=t\n"), None),
        (format!("{current}\npid=42\n"), None),
        (format!("{current}\nreleased \n"), None),
        (format!("{current}\npid\n"), None),
        (format!("{}\nreleased\n", current.to_uppercase()), None),
        (String::from("owner-claim v1\nreleased\n"), None),
        (String::new(), None),
        (format!("{current}\npid=9223372036854775808\nbirth=t\nnamespace=-\n"), None),
    ];
    for (text, expected) in cases {
        assert_eq!(decode(text.as_str()), expected, "{text:?}");
        check_claim_text(text.as_str());
    }
}

/// The `-` namespace marker reads as no namespace,
/// and no generated claim may expect one named `-`.
/// The input is the artifact the campaign found after 6,938 executions (`campaign-Xw7gB3`),
/// kept as
/// `seed/owner_claim_record/marker-namespace`.
#[test]
fn a_generated_namespace_of_the_marker_is_no_namespace() {
    let found: [u8; 55] = [
        0x6f, 0x63, 0x68, 0x6f, 0x6f, 0x6e, 0x72, 0x6f, 0xef, 0xef, 0xef, 0xef, 0xef, 0xef, 0xef, 0xef, 0xef, 0xee,
        0xef, 0xef, 0xef, 0xef, 0x70, 0x69, 0x64, 0x3d, 0x39, 0x32, 0x32, 0x33, 0x33, 0x37, 0x32, 0x30, 0x70, 0x69,
        0x64, 0x3d, 0x39, 0x30, 0x30, 0x39, 0x31, 0x37, 0x39, 0x32, 0x35, 0x34, 0x37, 0x34, 0x30, 0x39, 0x39, 0x31,
        0x0a,
    ];
    check_generated_claim_text(found.as_slice());
    let (text, expected) = generated_claim_text(found.as_slice());
    assert_eq!(decode(text.as_str()), expected, "{text:?}");
    let Some(ClaimRecord::Owned { namespace, .. }) = expected else { panic!("{expected:?}") };
    assert_eq!(namespace, None, "{text:?}");
    assert!(text.ends_with("namespace=-\ngarbage\n"), "{text:?}");
}

/// A generated namespace of the released marker reads as no namespace on every generated claim.
#[test]
fn generated_namespaces_are_never_the_released_marker() {
    for data in inputs() {
        let (_, expected) = generated_claim_text(data.as_slice());
        let Some(ClaimRecord::Owned { namespace, .. }) = expected else { continue };
        assert_ne!(namespace.as_deref(), Some("-"));
    }
}

/// A claim whose text is the record's own text gives that record back,
/// and a hand-written file whose field ends in a carriage return is read without the carriage return.
#[test]
fn restated_claims_give_their_record_back() {
    let current: String = header_of(DATA_FOLDER.current);
    for pid in [1_i64, 7, 42, 9_007_199_254_740_991] {
        for birth in ["t", "t a", "t\"q", "t\u{e9}", "t\u{1f600}"] {
            for namespace in [None, Some(String::new()), Some(String::from("pid:[1]"))] {
                let text: String = format!(
                    "{current}\npid={pid}\nbirth={birth}\nnamespace={}\n",
                    namespace.as_deref().unwrap_or("-")
                );
                let expected: ClaimRecord = ClaimRecord::Owned {
                    pid,
                    birth: String::from(birth),
                    namespace,
                };
                assert_eq!(decode(text.as_str()), Some(expected), "{text:?}");
                check_claim_text(text.as_str());
            }
        }
    }
    let carried: String = format!("{current}\npid=42\nbirth=t\r\nnamespace=-\n");
    let found: ClaimRecord = decode(carried.as_str()).expect("a claim whose birth ends in a carriage return");
    let ClaimRecord::Owned { birth, .. } = &found else { panic!("{found:?}") };
    assert!(!birth.ends_with('\r'), "{birth:?}");
    check_claim_text(carried.as_str());
}
