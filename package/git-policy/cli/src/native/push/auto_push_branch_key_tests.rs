//! Controls for the push coordination branch key codec.
//! The TypeScript corpus is `src/auto-push-branch-key.unit.test.ts`;
//! the incumbent fixture pairs come from its `encodeBranchKey` on Node 26.

use super::*;

/// Ref names covering slashes, case, the escape character, dots, and non-ASCII text.
const REF_NAMES: &[&str] = &[
    "refs/heads/main",
    "refs/heads/Main",
    "refs/heads/feat/ccc-push",
    "refs/heads/100%-done",
    "refs/heads/a.b_c-d",
    "refs/heads/über/zweig",
    "refs/heads/emoji-\u{1F600}",
];

/// Pairs of ref name and key written by the incumbent's `encodeBranchKey`.
const INCUMBENT_PAIRS: &[(&str, &str)] = &[
    ("refs/heads/main", "refs%2Fheads%2Fmain"),
    ("refs/heads/Main", "refs%2Fheads%2F%4Dain"),
    ("refs/heads/feat/ccc-push", "refs%2Fheads%2Ffeat%2Fccc-push"),
    ("refs/heads/100%-done", "refs%2Fheads%2F100%25-done"),
    ("refs/heads/a.b_c-d", "refs%2Fheads%2Fa.b_c-d"),
    ("refs/heads/über/zweig", "refs%2Fheads%2F%C3%BCber%2Fzweig"),
    (
        "refs/heads/emoji-\u{1F600}",
        "refs%2Fheads%2Femoji-%F0%9F%98%80",
    ),
    ("refs/heads/a~b^c:d", "refs%2Fheads%2Fa%7Eb%5Ec%3Ad"),
    ("refs/heads/x%2Fy", "refs%2Fheads%2Fx%252%46y"),
];

/// Every ref name round-trips through one flat, lowercase-safe file name.
#[test]
fn every_ref_name_round_trips_through_a_flat_key() {
    for ref_name in REF_NAMES {
        let key = encode_branch_key(ref_name);
        assert!(!key.contains('/'), "{key}");
        assert!(
            key.bytes().all(|byte| {
                return byte.is_ascii_lowercase()
                    || byte.is_ascii_digit()
                    || b"-_.%ABCDEF".contains(&byte);
            }),
            "{key}"
        );
        assert_eq!(decode_branch_key(&key).expect("decodes"), *ref_name);
    }
}

/// Slashes, uppercase letters and the escape character are escaped with uppercase hex.
#[test]
fn slashes_uppercase_and_the_escape_character_are_escaped() {
    assert_eq!(
        encode_branch_key("refs/heads/Main"),
        "refs%2Fheads%2F%4Dain"
    );
    assert_eq!(
        encode_branch_key("refs/heads/100%"),
        "refs%2Fheads%2F100%25"
    );
}

/// Refs differing only in case never share a key.
#[test]
fn refs_differing_only_in_case_have_distinct_keys() {
    assert_ne!(
        encode_branch_key("refs/heads/main").to_lowercase(),
        encode_branch_key("refs/heads/Main").to_lowercase()
    );
}

/// The incumbent's keys encode and decode identically natively.
#[test]
fn incumbent_keys_match_byte_for_byte_in_both_directions() {
    for (ref_name, key) in INCUMBENT_PAIRS {
        assert_eq!(encode_branch_key(ref_name), *key, "encode {ref_name}");
        assert_eq!(
            decode_branch_key(key).expect("decodes"),
            *ref_name,
            "decode {key}"
        );
    }
}

/// Every key the incumbent does not produce is refused.
#[test]
fn keys_the_encoder_never_produces_are_rejected() {
    for key in [
        "refs%2fheads",
        "refs%2",
        "refs%",
        "refs%61",
        "Refs",
        "refs/heads",
        "refs%FF",
    ] {
        assert!(decode_branch_key(key).is_err(), "accepted {key}");
    }
}

/// An escape of a verbatim byte and a raw non-verbatim character are both refused.
#[test]
fn an_escape_of_a_verbatim_byte_and_a_raw_character_are_refused() {
    assert!(decode_branch_key("%61").is_err());
    assert!(decode_branch_key("%2D").is_err());
    assert!(decode_branch_key("refs%2Fü").is_err());
    assert!(decode_branch_key("refs%2F!").is_err());
}

/// A refusal names the key, so the caller can report it.
#[test]
fn a_refusal_names_the_key() {
    let error = decode_branch_key("Refs").expect_err("refused");
    assert!(error.message.contains("Refs"), "{}", error.message);
    assert_eq!(error.to_string(), error.message);
}
