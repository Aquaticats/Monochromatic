//! What:
//!  Source revalidation and feature-export controls against disposable authoritative bytes.
//! Why:
//!  A stale or unreadable source cannot authorize publication,
//!  and fuzz-only exports need executed tests.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Compare a stored digest with matching, changed and removed source files; execute both codec verdicts.
//! ```

/// Import the real source revalidation gate and content digest,
///  not a test substitute.
use crate::rules_list::RulesEntry;
use crate::rules_snapshot::RulesSnapshot;
#[cfg(feature = "fuzzing")]
use super::path::source_digest;

/// Changed and missing source files fail the publication revalidation gate.
#[test]
fn source_revalidation_rejects_changed_or_missing_bytes() {
    // PathBuf owns a private fixture pathname derived from this process, never a real rules file.
    let directory = crate::test_scratch::scratch_root().join(format!("scanner-source-revalidation-{}", std::process::id()));
    std::fs::create_dir(&directory).expect("fresh fixture");
    let path = directory.join("rules");
    std::fs::write(&path, b"ORIGINAL_FIXTURE_LONG\n").expect("original fixture");
    let snapshot = RulesSnapshot::read(&[RulesEntry { path: path.clone(), optional: false }]).expect("read fixture");
    assert!(snapshot.still_current());
    std::fs::write(&path, b"CHANGED_FIXTURE_LONG\n").expect("changed fixture");
    assert!(!snapshot.still_current());
    std::fs::remove_file(&path).expect("remove fixture source");
    assert!(!snapshot.still_current());
    std::fs::remove_dir(&directory).expect("remove fixture directory");
}

/// An optional entry that appears after the read fails the gate, and one that stays absent passes it.
#[test]
fn source_revalidation_rejects_an_optional_file_that_appears() {
    let directory = crate::test_scratch::scratch_root().join(format!("scanner-source-appears-{}", std::process::id()));
    std::fs::create_dir(&directory).expect("fresh fixture");
    let required = directory.join("shared");
    let optional = directory.join("local");
    std::fs::write(&required, b"SHARED_FIXTURE_LONG\n").expect("shared fixture");
    let snapshot = RulesSnapshot::read(&[
        RulesEntry { path: required.clone(), optional: false },
        RulesEntry { path: optional.clone(), optional: true },
    ])
    .expect("read fixture");
    assert!(snapshot.still_current());
    std::fs::write(&optional, b"LOCAL_FIXTURE_LONG\n").expect("local fixture");
    assert!(!snapshot.still_current());
    std::fs::remove_file(&optional).expect("remove local");
    std::fs::remove_file(&required).expect("remove shared");
    std::fs::remove_dir(&directory).expect("remove fixture directory");
}

/// Fuzz exports must both accept a real artifact and reject malformed or source-mismatched bytes.
#[cfg(feature = "fuzzing")]
#[test]
fn exported_fuzz_codec_has_positive_and_negative_controls() {
    let source = b"CODEC_FIXTURE_LONG\n";
    let digest = source_digest(source).expect("fixture digest");
    let compiled = crate::runtime_matcher::RuntimeRules::compile("CODEC_FIXTURE_LONG\n").expect("fixture rules");
    let bytes = super::envelope::encode(&compiled, digest).expect("fixture artifact");
    assert!(super::decode_artifact_for_fuzzing(&bytes, source));
    assert!(!super::decode_artifact_for_fuzzing(b"invalid", source));
    assert!(!super::decode_artifact_for_fuzzing(&bytes, b"CHANGED_FIXTURE_LONG\n"));
    assert!(crate::fuzz_api::load_from_text("FUZZ_FIXTURE_LONG\n").is_ok());
    assert!(crate::fuzz_api::load_from_text("").is_err());
    assert!(crate::fuzz_api::scanner_from_text_for_fuzzing("FUZZ_FIXTURE_LONG\n").is_ok());
    assert!(crate::fuzz_api::scanner_from_text_for_fuzzing("/PRIVATE_PATTERN_LONG/g\n").is_err());
}
