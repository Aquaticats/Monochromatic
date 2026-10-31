//! What:
//!  Controls proving the configuration generator reaches accepted settings.
//! Why:
//!  The differential check is only evidence if generated documents really parse
//!      and really vary;
//!  these controls count what the generator reaches and run the
//!      invariants on fixed rejection cases.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // for (const data of samples) checkGeneratedConfig(generatedConfig(data));
//! ```

/// Import the generator and invariants under control.
use super::{
    GeneratedConfig, canonical_document, check_config_source, check_generated_config,
    generated_config,
};
use git_policy_cli::config_parse::parse_config;
use git_policy_cli::config_schema::CliGitConfig;
use git_policy_cli::policy_registry::{PolicyId, Severity};
use monochromatic_jsonc_edit::emit_jsonc_value;

/// Every single-byte fill and a spread of mixed inputs produce documents that parse to their expectation.
#[test]
fn generated_documents_parse_to_their_expectation() {
    let mut non_default: usize = 0;
    let mut with_exclusions: usize = 0;
    let mut scanner_baseline_off: usize = 0;
    let mut tuned: usize = 0;
    let mut severities_off: usize = 0;
    let mut with_one_rules_file: usize = 0;
    let mut with_several_rules_files: usize = 0;
    let mut with_optional_rules_file: usize = 0;
    let mut with_punctuated_rules_file: usize = 0;
    for byte in 0..=u8::MAX {
        for data in [
            vec![byte; 24],
            vec![
                byte,
                byte.wrapping_mul(7),
                byte.wrapping_add(91),
                255,
                3,
                253,
                127,
                63,
                255,
                255,
                byte,
                byte,
            ],
            vec![
                255, 255, 255, 255, 255, 255, 255, 255, 255, byte, byte, byte, 0, 0, 0, 0, byte, 1,
                2, 3, 4, 5, 6,
            ],
            vec![byte],
            Vec::<u8>::new(),
        ] {
            let generated: GeneratedConfig = generated_config(data.as_slice());
            check_generated_config(&generated);
            if generated.expected != CliGitConfig::defaults() {
                non_default += 1;
            }
            if !generated
                .expected
                .policies
                .markdown_autofix
                .exclude
                .is_empty()
            {
                with_exclusions += 1;
            }
            if !generated.expected.policies.forbidden_strings.builtin_rules {
                scanner_baseline_off += 1;
            }
            if generated.expected.concurrency != CliGitConfig::defaults().concurrency {
                tuned += 1;
            }
            if let Some(files) = &generated.expected.policies.forbidden_strings.rules_files {
                if files.len() == 1 && !files[0].optional {
                    with_one_rules_file += 1;
                }
                if files.len() > 1 {
                    with_several_rules_files += 1;
                }
                if files.iter().any(|file| return file.optional) {
                    with_optional_rules_file += 1;
                }
                if files.iter().any(|file| return file.path.contains(',') || file.path.contains('?')) {
                    with_punctuated_rules_file += 1;
                }
            }
            if generated
                .expected
                .policies
                .setting(PolicyId::RequireRoot)
                .severity
                == Severity::Off
            {
                severities_off += 1;
            }
        }
    }
    for (name, count) in [
        ("non-default documents", non_default),
        ("documents with exclusions", with_exclusions),
        (
            "documents disabling the scanner baseline",
            scanner_baseline_off,
        ),
        ("documents tuning concurrency", tuned),
        ("documents turning a built-in off", severities_off),
        ("documents with one required rules file", with_one_rules_file),
        ("documents with several rules files", with_several_rules_files),
        ("documents with an optional rules file", with_optional_rules_file),
        ("documents with a comma or question mark in a rules path", with_punctuated_rules_file),
    ] {
        assert!(count > 10, "{name} reached only {count} times");
    }
}

/// The canonical document of the defaults states every policy and parses back to the same values.
#[test]
fn canonical_document_round_trips() {
    let source: String = emit_jsonc_value(&canonical_document(&CliGitConfig::defaults()));
    let restated: CliGitConfig = parse_config(source.as_str()).expect("canonical defaults");
    assert_eq!(restated.concurrency, CliGitConfig::defaults().concurrency);
    assert!(
        source.contains("\"security/forbidden-strings\""),
        "{source}"
    );
    assert!(source.contains("\"reserveAfterLostRaces\""), "{source}");
    check_config_source(source.as_str());
}

/// Rejections with hostile key text still render as one decodable event line.
#[test]
fn rejection_invariants_hold_for_hostile_sources() {
    for source in [
        "",
        "{",
        "[]",
        "{ \"plugins\": {} }",
        "{ \"a\\\"}\\n{\\\"x\": 1 }",
        "{ \"policies\": { \"line\\nbreak\\u0000\\u001f\": \"off\" } }",
        "{ \"policies\": { \"\\ud800\": \"off\" } }",
        "{ \"policies\": { \"final-newline\": \"\\u2028\\\\\" } }",
        "{ \"landing\": { \"reserveAfterLostRaces\": 1e400 } }",
        "{ \"policies\": { \"markdown/autofix\": [\"warn\", { \"rules\": [\"\\t\"] }] } }",
        "{ \"hooks\": { \"concurrentCommits\": true }, \"hooks\": {} }",
    ] {
        assert!(parse_config(source).is_err(), "{source}");
        check_config_source(source);
    }
}
