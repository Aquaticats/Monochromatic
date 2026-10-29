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
use super::{source_path_still_matches, path::source_digest};

/// Changed and missing source files fail the publication revalidation gate.
#[test]
fn source_revalidation_rejects_changed_or_missing_bytes() {
    // PathBuf owns a private fixture pathname derived from this process, never a real rules file.
    let directory = std::env::temp_dir().join(format!("scanner-source-revalidation-{}", std::process::id()));
    std::fs::create_dir(&directory).expect("fresh fixture");
    let path = directory.join("rules");
    std::fs::write(&path, b"ORIGINAL_FIXTURE_LONG\n").expect("original fixture");
    let digest = source_digest(b"ORIGINAL_FIXTURE_LONG\n").expect("original digest");
    // Borrow path ownership and copy the digest for each independent check.
    assert!(source_path_still_matches(&path, digest));
    std::fs::write(&path, b"CHANGED_FIXTURE_LONG\n").expect("changed fixture");
    assert!(!source_path_still_matches(&path, digest));
    std::fs::remove_file(&path).expect("remove fixture source");
    assert!(!source_path_still_matches(&path, digest));
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
