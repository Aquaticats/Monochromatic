//! What:
//!  Invariants for the unified linter's merge and configuration boundaries.
//! Why:
//!  Structured generation reaches valid inputs while raw JSONC mutation also exercises rejection paths.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Shared property assertions, called by both unit tests and fuzz targets.
//! ```

/// Counted Markdown/MDX rules and raw-source edit safety controls.
pub mod markdown;

/// Whole per-source path:
///  host rules,
///  processors,
///  nested doc tests,
///  projection and the fix loop.
pub mod orchestration;

/// Semantic explicit-type property controls.
pub mod explicit_types;
/// Fixed in-memory workspace initialization for semantic properties.
mod semantic_fixture;

/// Structured and raw-source invariants for the sidecar test rule.
pub mod rust_style;

/// Import owned JSONC values and the production operations under test.
use monochromatic_jsonc_edit::{JsoncEntry, JsoncKey, JsoncValue, emit_jsonc_value};
use monochromatic_lint::config_merge::merge_values;
use monochromatic_lint::configuration::parse_configuration;
/// Import decoded-key deduplication for an independent order invariant.
use std::collections::BTreeSet;

/// What:
///  Exercise merge laws over a structured document.
/// Why:
///  These laws do not reimplement recursive merge and cannot share its recursive grouping bug.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkMerge(document: JsoncValue): void;
/// ```
pub fn check_merge(document: &JsoncValue) {
    // Clone a snapshot before lending values to the production implementation.
    let snapshot = document.clone();
    let mut inputs = Vec::new();
    // Array roots provide arbitrary argument groups; record roots provide single-input cases.
    if let Some(elements) = document.elements() {
        inputs.extend(elements);
    } else {
        inputs.push(document);
    }
    let actual = merge_values(&inputs);
    assert_eq!(document, &snapshot, "merge changed input data");
    if inputs.is_empty() {
        assert!(actual.entries().expect("empty settings record").is_empty());
        return;
    }
    let first = inputs[0];
    let last = inputs[inputs.len() - 1];
    let mut mismatch = false;
    for input in &inputs {
        if std::mem::discriminant(&input.kind) != std::mem::discriminant(&first.kind) {
            mismatch = true;
        }
    }
    if mismatch || (first.entries().is_none() && first.elements().is_none()) {
        assert_eq!(
            &actual, last,
            "scalar or mixed-kind group must select its final value"
        );
    } else if first.elements().is_some() {
        let mut expected = Vec::new();
        for input in &inputs {
            expected.extend_from_slice(input.elements().expect("classified array"));
        }
        assert_eq!(
            actual.elements().expect("merged array"),
            expected.as_slice(),
            "arrays concatenate without recursively merging elements"
        );
    } else {
        let mut seen = BTreeSet::new();
        let mut expected_keys = Vec::new();
        for input in &inputs {
            for entry in input.entries().expect("classified record") {
                if seen.insert(entry.key.units.clone()) {
                    expected_keys.push(entry.key.units.clone());
                }
            }
        }
        let mut actual_keys = Vec::new();
        for entry in actual.entries().expect("merged record") {
            actual_keys.push(entry.key.units.clone());
        }
        assert_eq!(
            actual_keys, expected_keys,
            "record keys retain first-seen order"
        );
    }
    let repeated = merge_values(&[&actual]);
    assert_eq!(
        repeated, actual,
        "merging an already merged value alone must be stable"
    );
}

/// What:
///  Construct an owned record member for generated configurations.
/// Why:
///  The parser's constructors perform string escaping rather than an invented text interpolation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function entry(name: string, value: JsoncValue): JsoncEntry;
/// ```
fn entry(name: &str, value: JsoncValue) -> JsoncEntry {
    return JsoncEntry {
        key: JsoncKey::from_text(name),
        value,
    };
}

/// What:
///  Construct ordinary generated string data through the JSONC value model.
/// Why:
///  Final emission,
///  not the generator,
///  is responsible for JSON quoting.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function text(value: string): JsoncValue;
/// ```
fn text(value: &str) -> JsoncValue {
    return JsoncValue::text_from_units(value.encode_utf16().collect());
}

/// What:
///  Build a valid configuration from arbitrary bytes,
///  with bounded block count.
/// Why:
///  Raw bytes alone rarely reach successful rule validation and merging.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function generatedConfiguration(bytes: Uint8Array): JsoncValue;
/// ```
pub fn generated_configuration(bytes: &[u8]) -> JsoncValue {
    let mut blocks = Vec::new();
    for byte in bytes.iter().take(32) {
        let severity = ["off", "warn", "error"][usize::from(*byte % 3)];
        let mut options = Vec::new();
        options.push(entry("severity", text(severity)));
        let id;
        if *byte % 2 == 0 {
            id = "rust/max-lines";
            let number = JsoncValue::number_from_token(byte.to_string().as_str())
                .expect("byte is an integer token");
            options.push(entry("max", number));
        } else {
            id = "markdown/lfs-image-url";
            options.push(entry(
                "exclude",
                JsoncValue::array(vec![text(format!("path-{byte}/").as_str())]),
            ));
        }
        let rules = JsoncValue::record(vec![entry(id, JsoncValue::record(options))]);
        blocks.push(JsoncValue::record(vec![
            entry("files", JsoncValue::array(vec![text("**/*")])),
            entry("rules", rules),
        ]));
    }
    return JsoncValue::array(blocks);
}

/// What:
///  Exercise configuration validation and check that merging valid blocks remains valid.
/// Why:
///  Per-block validity must remain true after the selected rules are combined.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkConfiguration(source: string): void;
/// ```
pub fn check_configuration(source: &str) {
    let Ok(blocks) = parse_configuration(source) else {
        return;
    };
    let snapshot = blocks.clone();
    let mut inputs = Vec::new();
    for block in &blocks {
        if !block.global_ignore {
            inputs.push(&block.rules);
        }
    }
    let merged = merge_values(&inputs);
    assert_eq!(
        blocks, snapshot,
        "configuration merge changed parsed blocks"
    );
    let reconstructed = JsoncValue::array(vec![JsoncValue::record(vec![
        entry("files", JsoncValue::array(Vec::new())),
        entry("rules", merged),
    ])]);
    let rendered = emit_jsonc_value(&reconstructed);
    assert!(
        parse_configuration(rendered.as_str()).is_ok(),
        "merged valid rule settings became invalid: {rendered}"
    );
}

/// Generator controls prove every byte value reaches a valid configuration rather than only rejection paths.
#[test]
fn generated_cases_reach_valid_settings() {
    for byte in 0..=u8::MAX {
        let generated = generated_configuration(&[byte, byte.wrapping_add(1)]);
        let source = emit_jsonc_value(&generated);
        assert!(
            parse_configuration(source.as_str()).is_ok(),
            "invalid generated case: {source}"
        );
        check_configuration(source.as_str());
    }
}

/// Fixed mixed-kind and array cases exercise the same invariant helper as the fuzzer.
#[test]
fn merge_invariant_controls_reach_mixed_and_array_shapes() {
    for source in [
        "[null,[1],[2]]",
        "[{\"x\":[1]},{\"x\":false},{\"x\":[2]}]",
        "[[1],[2]]",
        "[]",
        "{\"a\":1}",
    ] {
        let parsed = monochromatic_jsonc_edit::parse_jsonc(source).expect("control parses");
        check_merge(&parsed);
    }
}
