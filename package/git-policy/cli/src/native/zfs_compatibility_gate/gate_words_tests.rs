//! What:
//!  The words a compatible verdict and a refusal print:
//! the release sentence appears only
//!  when the string differs from the tested text,
//! and a refusal names the step,
//!  what stays disabled and offers no bypass.
//! Why:
//!  The release string is reporting and never a rule,
//! so a test asserts the
//!  sentence both ways round,
//! and the refusal's words are what an operator acts on.

use super::gate_verdict::{Facts, Verdict};
use super::gate_words::{TESTED_MODULE_VERSION, compatible_line, refusal_line, release_sentence, verdict_line};
use monochromatic_zfs_request_to_openzfs_kernel_device::{CompatibilityRefusal as Refusal, CompatibilityStep as Step};

/// The facts of one compatible verdict.
fn facts(version: Option<&str>, absent: Vec<&'static str>) -> Facts {
    return Facts {
        target: "x86_64",
        version: version.map(str::to_string),
        absent,
        base_btf_bytes: 6_579_400,
        module_btf_bytes: 570_234,
        header_bytes: (32, 32),
    };
}

#[test]
fn the_tested_release_text_gets_no_sentence() {
    assert!(release_sentence(Some(TESTED_MODULE_VERSION)).is_none());
    let words: String = compatible_line(&facts(Some(TESTED_MODULE_VERSION), Vec::new()));
    assert!(!words.contains("a different text was read"), "{words}");
    assert!(words.contains("x86_64"), "{words}");
    assert!(words.contains("6579400"), "{words}");
}

#[test]
fn another_release_text_gets_the_sentence_quoting_both() {
    let sentence: String = release_sentence(Some("2.4.4-1_g1a2b3c4")).expect("the text differs");
    assert!(sentence.contains("2.4.4-1_g1a2b3c4"), "{sentence}");
    assert!(sentence.contains(TESTED_MODULE_VERSION), "{sentence}");
    assert!(sentence.contains("not a different version"), "{sentence}");
    let words: String = compatible_line(&facts(Some("2.2.2-0ubuntu9.5"), Vec::new()));
    assert!(words.contains("2.2.2-0ubuntu9.5"), "{words}");
}

#[test]
fn a_module_with_no_identity_attribute_says_the_check_is_weaker() {
    let words: String = compatible_line(&facts(None, vec!["version", "build ID"]));
    assert!(words.contains("no version, build ID"), "{words}");
    assert!(words.contains("weaker"), "{words}");
}

#[test]
fn a_refusal_names_the_step_the_disabled_requests_and_no_bypass() {
    let refusal: Refusal = Refusal {
        step: Step::Member,
        file: Some(std::path::PathBuf::from("/sys/kernel/btf/zfs")),
        detail: String::from("zfs_cmd.zc_objset_stats.dds_flags is missing"),
    };
    let words: String = refusal_line(&refusal);
    assert!(words.contains("member refused whilst reading /sys/kernel/btf/zfs"), "{words}");
    assert!(words.contains("dds_flags"), "{words}");
    assert!(words.contains("reads included"), "{words}");
    assert!(words.contains("registrations and durable recovery state"), "{words}");
    assert!(words.contains("reflink or ordinary Git route"), "{words}");
    assert!(words.contains("No flag and no environment"), "{words}");
}

#[test]
fn one_verdict_becomes_its_own_line() {
    let compatible: Verdict = Verdict::Compatible(facts(Some(TESTED_MODULE_VERSION), Vec::new()));
    assert!(verdict_line(&compatible).contains("is compatible"));
    let refused: Verdict = Verdict::Refused(Refusal {
        step: Step::Device,
        file: None,
        detail: String::from("it is absent"),
    });
    assert!(verdict_line(&refused).contains("gate refused"));
}
