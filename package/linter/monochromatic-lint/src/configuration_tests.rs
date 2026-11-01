//! What:
//!  Consumer-level JSONC schema regressions.
//! Why:
//!  Invalid or ambiguous documents must fail before their rule settings reach merging.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! describe('configuration schema', () => { /* accepted and rejected documents */ });
//! ```

/// Import the production parser and exact-number option validator.
use super::parse_configuration;
use crate::configuration_rules::line_limit;
use monochromatic_jsonc_edit::parse_jsonc;

/// Comments and trailing commas retain ordered blocks and typed pattern values.
#[test]
fn comments_and_trailing_commas_preserve_block_order() {
    let blocks = parse_configuration(
        r#"[
      // Global exclusions
      { "name": "ignored", "ignores": ["generated/"], },
      { "name": "source", "files": ["**/*.rs"], "ignores": [],
        "rules": { "rust/max-lines": { "severity": "error", "max": 3e2 }, }, },
    ]"#,
    )
    .expect("valid JSONC configuration");
    assert_eq!(blocks.len(), 2);
    assert_eq!(blocks[0].name.as_deref(), Some("ignored"));
    assert!(blocks[0].global_ignore);
    assert!(!blocks[1].global_ignore);
    assert_eq!(blocks[1].files, ["**/*.rs"]);
}

/// Empty arrays and empty files lists remain valid,
///  but empty blocks do not.
#[test]
fn empty_configuration_and_nonmatching_blocks_are_valid() {
    assert!(parse_configuration("[]").expect("empty array").is_empty());
    let blocks = parse_configuration("[{\"files\":[],\"rules\":{}}]").expect("empty match set");
    assert_eq!(blocks.len(), 1);
    assert!(blocks[0].files.is_empty());
    assert!(!blocks[0].global_ignore);
}

/// Every schema rejection is paired with an affected-field diagnostic.
#[test]
fn malformed_shapes_are_rejected() {
    let cases = [
        ("{}", "ordered array"),
        ("[false]", "block must be an object"),
        ("[{}]", "need files"),
        ("[{\"name\":\"only-name\"}]", "need files"),
        ("[{\"ignores\":[],\"rules\":{}}]", "need files"),
        ("[{\"files\":42}]", "array of strings"),
        ("[{\"files\":[false]}]", "must be a string"),
        ("[{\"files\":[],\"name\":true}]", "must be a string"),
        ("[{\"files\":[],\"rules\":[]}]", "rules must be an object"),
        (
            "[{\"files\":[],\"plugins\":[]}]",
            "Unknown configuration block field",
        ),
        (
            "[{\"files\":[],\"command\":[\"program\"]}]",
            "Unknown configuration block field",
        ),
        (
            "[{\"files\":[],\"rules\":{\"unknown/rule\":{}}}]",
            "Unknown built-in rule",
        ),
        // The removed anonymous-function rule is rejected by name, exactly like an unregistered one.
        (
            "[{\"files\":[],\"rules\":{\"rust/no-anonymous-functions\":{\"severity\":\"off\"}}}]",
            "Unknown built-in rule rust/no-anonymous-functions",
        ),
        (
            "[{\"files\":[],\"rules\":{\"rust/max-lines\":\"off\"}}]",
            "settings must be an object",
        ),
        (
            "[{\"files\":[],\"rules\":{\"rust/max-lines\":{\"severity\":\"warning\"}}}]",
            "severity must be",
        ),
        (
            "[{\"files\":[],\"rules\":{\"rust/max-lines\":{\"severity\":1}}}]",
            "must be a string",
        ),
        (
            "[{\"files\":[],\"rules\":{\"rust/require-rustdoc\":{\"max\":2}}}]",
            "unknown option",
        ),
        (
            "[{\"files\":[],\"rules\":{\"markdown/lfs-image-url\":{\"exclude\":true}}}]",
            "array of strings",
        ),
        // These distinguish the rule-id AND option-name guard from an incorrect OR.
        (
            "[{\"files\":[],\"rules\":{\"rust/require-rustdoc\":{\"exclude\":[]}}}]",
            "unknown option",
        ),
        (
            "[{\"files\":[],\"rules\":{\"markdown/lfs-image-url\":{\"unexpected\":[]}}}]",
            "unknown option",
        ),
    ];
    for (source, expected) in cases {
        let error = parse_configuration(source).expect_err("schema must reject invalid source");
        assert!(
            error.message.contains(expected),
            "{source}: {}",
            error.message
        );
    }
}

/// Duplicate decoding and null checks run throughout the tree before rule interpretation.
#[test]
fn ambiguous_and_null_values_are_rejected_before_merge() {
    let duplicate_sources = [
        r#"[{"files":[],"files":[]}]"#,
        r#"[{"files":[],"\u0066iles":[]}]"#,
        r#"[{"files":[],"rules":{"rust/max-lines":{"severity":"error","severity":"warn"}}}]"#,
        r#"[{"files":[],"rules":{"rust/max-lines":{"severity":"error"},"rust/max-lines":{"max":300}}}]"#,
    ];
    for source in duplicate_sources {
        let error = parse_configuration(source).expect_err("duplicate keys are rejected");
        assert!(
            error.message.contains("duplicate"),
            "{source}: {}",
            error.message
        );
    }
    for source in [
        "[null]",
        "[{\"files\":[null]}]",
        "[{\"files\":[],\"rules\":{\"rust/max-lines\":{\"max\":null}}}]",
    ] {
        let error = parse_configuration(source).expect_err("null is rejected");
        assert!(
            error.message.contains("null"),
            "{source}: {}",
            error.message
        );
    }
}

/// Invalid UTF-16 cannot become an identifier or filesystem pattern through replacement characters.
#[test]
fn unpaired_surrogates_are_rejected() {
    for source in [
        r#"[{"files":[],"name":"\ud800"}]"#,
        r#"[{"files":["\udc00"]}]"#,
        r#"[{"files":[],"\ud800":1}]"#,
    ] {
        let error = parse_configuration(source).expect_err("invalid textual field");
        assert!(error.message.contains("unpaired UTF-16 surrogate"));
    }
    let valid = parse_configuration(r#"[{"name":"\ud83d\ude80","files":["**/*.md"]}]"#)
        .expect("paired surrogate");
    assert_eq!(valid[0].name.as_deref(), Some("🚀"));
}

/// Exact-number conversion accepts integral spellings but never rounds or wraps.
#[test]
fn line_limits_use_exact_integer_values() {
    for token in ["300", "300.0", "3e2"] {
        let input = parse_jsonc(format!("[{token}]").as_str()).expect("numeric fixture");
        let value = &input.elements().expect("array")[0];
        assert_eq!(line_limit(value).expect("integral count"), 300);
    }
    for token in ["0", "-0", "0e100000"] {
        let input = parse_jsonc(format!("[{token}]").as_str()).expect("zero fixture");
        assert_eq!(
            line_limit(&input.elements().expect("array")[0]).expect("zero count"),
            0
        );
    }
    let maximum = usize::MAX.to_string();
    let maximum_source = parse_jsonc(format!("[{maximum}]").as_str()).expect("maximum fixture");
    assert_eq!(
        line_limit(&maximum_source.elements().expect("array")[0]).expect("maximum count"),
        usize::MAX
    );
    let overflow_source = parse_jsonc(format!("[{maximum}0]").as_str()).expect("overflow fixture");
    assert!(line_limit(&overflow_source.elements().expect("array")[0]).is_err());
    for token in [
        "-1",
        "0.5",
        "1e4000",
        "12345678901234567890123456789",
        "\"300\"",
        "true",
    ] {
        let input = parse_jsonc(format!("[{token}]").as_str()).expect("rejected value fixture");
        assert!(
            line_limit(&input.elements().expect("array")[0]).is_err(),
            "{token}"
        );
    }
}

/// Partial settings remain partial until all matching blocks have been merged.
#[test]
fn partial_rule_settings_and_valid_options_are_preserved() {
    let blocks = parse_configuration(r#"[
      {"files":["**/*.rs"],"rules":{"rust/max-lines":{"max":120}}},
      {"files":["**/*.rs"],"rules":{"rust/max-lines":{"severity":"warn"}}},
      {"files":["**/*.md"],"rules":{"markdown/lfs-image-url":{"severity":"off","exclude":["asset/"]}}}
    ]"#).expect("partial rule objects");
    assert_eq!(blocks.len(), 3);
    assert_eq!(blocks[0].rules.entries().expect("rules record").len(), 1);
}

/// Syntax errors retain the parser's source-byte diagnostic.
#[test]
fn syntax_errors_keep_byte_context() {
    let error = parse_configuration("[{").expect_err("incomplete document");
    assert!(error.message.contains("JSONC syntax error at byte"));
    assert_eq!(error.to_string(), error.message);
}

/// What:
///  The repository configuration names only rules this build registers.
/// Why:
///  A configured name the registry lacks is the `Unknown built-in rule` startup setup error,
///  and the repository configuration is the shipped consumer of this registry.
///  The check covers the source tree;
///  a stale installed binary with an older registry is a version-resolution problem,
///  recorded in `doc/troubleshooting/mise-lock-pins-repo-tool-version.md`.
#[test]
fn the_repository_configuration_only_names_registered_rules() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../monochromatic-lint.config.jsonc");
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    let blocks = parse_configuration(source.as_str())
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    assert!(!blocks.is_empty(), "{} selects no block", path.display());
    // Control: rewriting every rule namespace in the same document must be rejected by name,
    //  so the parse above cannot pass through a validator that checks nothing.
    let renamed = source
        .replace("\"rust/", "\"renamed/")
        .replace("\"markdown/", "\"renamed/");
    assert_ne!(renamed, source, "the control needs rule names to rewrite");
    let error = parse_configuration(renamed.as_str()).expect_err("unregistered names are rejected");
    assert!(
        error.message.contains("Unknown built-in rule renamed/"),
        "{}",
        error.message
    );
}
