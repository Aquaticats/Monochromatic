//! What:
//!  The schema golden test and the corpus check of the hand-written decoder against the committed schema.
//! Why:
//!  `file-enforcer.schema.json` is derived from the config types,
//! the decoder is hand-written,
//! and editors read the
//!  schema;
//! this test fails when the committed file is stale,
//! and names every sample where the two disagree.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! it('the committed schema equals the derived schema', () => { ... });
//! ```

use crate::config_decode::{ConfigScope, decode_config};
use crate::config_load::parse_config_text;
use crate::schema::{config_schema, config_schema_text};
use serde_json::Value;
use std::path::Path;

/// What:
///  Read a file relative to this package's directory.
/// Why:
///  The committed schema sits at the repository root,
/// the corpus beside this package.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readPackageFile(relative: string): string
/// ```
fn read_package_file(relative: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
    return std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
}

#[test]
fn the_committed_schema_equals_the_derived_schema() {
    let committed = read_package_file("../../../file-enforcer.schema.json");
    assert!(
        committed == config_schema_text(),
        "file-enforcer.schema.json is stale; run `mise run //package/file-enforcer/core-rust:schema:write` and commit it"
    );
}

#[test]
fn the_schema_declares_the_draft_the_validator_is_told_to_use() {
    let schema = config_schema();
    assert_eq!(schema["$schema"], "https://json-schema.org/draft/2020-12/schema");
}

/// What:
///  Whether the schema accepts a sample.
/// Why:
///  The corpus compares this with the decoder's verdict.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function schemaAccepts(validator: Validator, sample: unknown): boolean
/// ```
fn schema_accepts(validator: &jsonschema::Validator, sample: &Value) -> bool {
    return validator.is_valid(sample);
}

/// What:
///  Whether the decoder accepts a sample under the repository scope.
/// Why:
///  The corpus compares this with the schema's verdict.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function decoderAccepts(sample: unknown): boolean
/// ```
fn decoder_accepts(sample: &Value) -> bool {
    return decode_config(sample, ConfigScope::Repository, "corpus").is_ok();
}

/// What:
///  The members of one corpus section,
/// as (name,
/// sample) pairs.
/// Why:
///  Each section is an object of named samples.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function section(corpus: unknown, name: string): [string, unknown][]
/// ```
fn section<'a>(corpus: &'a Value, name: &str) -> Vec<(&'a String, &'a Value)> {
    let object = corpus[name].as_object().unwrap_or_else(|| panic!("the corpus has no section {name}"));
    return object.iter().collect();
}

#[test]
fn the_decoder_and_the_schema_agree_over_the_corpus() {
    let corpus = parse_config_text(&read_package_file("fixtures/config-corpus.jsonc"), "config-corpus.jsonc")
        .expect("the corpus is valid JSONC");
    let schema = config_schema();
    let validator = jsonschema::validator_for(&schema).expect("the derived schema compiles");
    let mut mismatches: Vec<String> = Vec::new();
    for (name, sample) in section(&corpus, "accept") {
        if !decoder_accepts(sample) {
            mismatches.push(format!("accept/{name}: the decoder rejects it"));
        }
        if !schema_accepts(&validator, sample) {
            mismatches.push(format!("accept/{name}: the schema rejects it"));
        }
    }
    for (name, sample) in section(&corpus, "reject") {
        if decoder_accepts(sample) {
            mismatches.push(format!("reject/{name}: the decoder accepts it"));
        }
        if schema_accepts(&validator, sample) {
            mismatches.push(format!("reject/{name}: the schema accepts it"));
        }
    }
    for (name, entry) in section(&corpus, "decoderStricter") {
        let sample = &entry["config"];
        if entry["reason"].as_str().is_none() {
            mismatches.push(format!("decoderStricter/{name}: names no reason"));
        }
        if decoder_accepts(sample) {
            mismatches.push(format!("decoderStricter/{name}: the decoder accepts it"));
        }
        if !schema_accepts(&validator, sample) {
            mismatches.push(format!("decoderStricter/{name}: the schema now rejects it; move it to reject"));
        }
    }
    assert!(mismatches.is_empty(), "decoder and schema disagree:\n{}", mismatches.join("\n"));
}

#[test]
fn the_per_user_kind_is_accepted_by_both_in_the_user_scope() {
    let text = r#"{"rules": [{
        "kind": "jetbrains-harper", "id": "h", "productPrefixes": ["IdeaIC"], "disabledLinters": ["A"],
        "excludePatterns": ["**/AGENTS.md"],
        "scopedServers": [{"id": "s", "name": "S", "fileNames": ["AGENTS.md"], "languageId": "markdown", "disabledLinters": []}]
    }]}"#;
    let sample: Value = serde_json::from_str(text).expect("the sample is valid JSON");
    let validator = jsonschema::validator_for(&config_schema()).expect("the derived schema compiles");
    assert!(schema_accepts(&validator, &sample));
    assert!(decode_config(&sample, ConfigScope::User, "corpus").is_ok());
    assert!(decode_config(&sample, ConfigScope::Repository, "corpus").is_err());
}
