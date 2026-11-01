//! What:
//!  Rejection controls for the structure of a configuration document.
//! Why:
//!  Non-object documents,
//!  unknown and retired keys,
//!  duplicates and nulls must each
//!      fail with a message naming the key.
//!  Acceptance controls live in
//!      `config_parse_acceptance_tests.rs`,
//!  invalid-value controls in
//!      `config_parse_value_tests.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(() => parseConfig('{"plugin":1}')).toThrow(/Unknown configuration key: plugin/);
//! ```

/// Import the parser under test.
use super::parse_config;

/// What:
///  Return the rejection message of one document;
///  an accepted document fails the test.
/// Why:
///  `pub(super)` lets the sibling invalid-value controls reuse it instead of copying it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function rejection(source: string): string { /* message of the thrown error */ }
/// ```
pub(super) fn rejection(source: &str) -> String {
    return parse_config(source).expect_err(source).message;
}

/// Assert that every listed document is rejected with a message containing the fragment.
pub(super) fn assert_rejections(cases: &[(&str, &str)]) {
    for (source, fragment) in cases {
        let message: String = rejection(source);
        assert!(message.contains(fragment), "{source}\n  got: {message}");
    }
}

/// Documents that are not one JSONC object are rejected before any key is read.
#[test]
fn non_object_documents_and_syntax_errors_are_rejected() {
    assert_rejections(&[
        (
            "[]",
            "The configuration document must be an object, found an array.",
        ),
        (
            "[{ \"policies\": {} }]",
            "The configuration document must be an object, found an array.",
        ),
        // The shared parser accepts only an object or array as a document root.
        (
            "\"policies\"",
            "JSONC syntax error at byte 0: JSONC document root must be an object or array",
        ),
        ("1", "JSONC syntax error at byte 0"),
        ("true", "JSONC syntax error at byte 0"),
        ("null", "JSONC syntax error at byte 0"),
        ("", "JSONC syntax error at byte 0"),
        ("{", "JSONC syntax error at byte"),
        ("{ \"policies\": }", "JSONC syntax error at byte"),
        ("{} {}", "JSONC syntax error at byte"),
        ("export default {};", "JSONC syntax error at byte 0"),
    ]);
}

/// Unknown and retired top-level keys are named,
///  including near misses and executable-era keys.
#[test]
fn unknown_and_retired_top_level_keys_are_named() {
    assert_rejections(&[
        (
            r#"{ "plugin": {} }"#,
            "Unknown configuration key: plugin. Accepted keys: policies, hooks, indexLock, landing.",
        ),
        (
            r#"{ "Policies": {} }"#,
            "Unknown configuration key: Policies.",
        ),
        (
            r#"{ "policies ": {} }"#,
            "Unknown configuration key: policies .",
        ),
        (r#"{ "": {} }"#, "Unknown configuration key: ."),
        (
            r#"{ "__proto__": {} }"#,
            "Unknown configuration key: __proto__.",
        ),
        (
            r#"{ "$schema": "x" }"#,
            "Unknown configuration key: $schema.",
        ),
        (
            r#"{ "imports": ["./x.mjs"] }"#,
            "Unknown configuration key: imports.",
        ),
        (
            r#"{ "index-lock": {} }"#,
            "Unknown configuration key: index-lock.",
        ),
        (
            r#"{ "plugins": { "security": "x" } }"#,
            "Configuration key plugins is retired",
        ),
        (
            r#"{ "trust": { "children": true } }"#,
            "Configuration key trust is retired",
        ),
        (
            r#"{ "policies": {}, "unknown": 1 }"#,
            "Unknown configuration key: unknown.",
        ),
    ]);
}

/// A key defined twice,
///  literally or through an escape alias,
///  is rejected at every level.
#[test]
fn ambiguous_duplicate_settings_are_rejected() {
    assert_rejections(&[
        (
            r#"{ "policies": {}, "policies": {} }"#,
            "Configuration key policies is defined more than once",
        ),
        (
            r#"{ "hooks": {}, "hooks": { "concurrentCommits": true } }"#,
            "Configuration key hooks is defined more than once",
        ),
        (
            r#"{ "policies": { "final-newline": "off", "final-newline": "error" } }"#,
            "Configuration key policies.final-newline is defined more than once",
        ),
        (
            r#"{ "policies": { "final-newline": "off", "final-newline": "off" } }"#,
            "Configuration key policies.final-newline is defined more than once",
        ),
        (
            r#"{ "hooks": { "concurrentCommits": true, "concurrentCommits": true } }"#,
            "Configuration key hooks.concurrentCommits is defined more than once",
        ),
        (
            r#"{ "indexLock": { "unprovenOwnerTimeoutMs": 1, "unprovenOwnerTimeoutMs": 2 } }"#,
            "Configuration key indexLock.unprovenOwnerTimeoutMs is defined more than once",
        ),
        (
            r#"{ "policies": { "security/forbidden-strings": ["error", { "builtinRules": true, "builtinRules": false }] } }"#,
            "Configuration key policies.security/forbidden-strings[1].builtinRules is defined more than once",
        ),
        (
            r#"{ "policies": { "markdown/autofix": ["error", { "exclude": [], "exclude": [] }] } }"#,
            "Configuration key policies.markdown/autofix[1].exclude is defined more than once",
        ),
    ]);
}

/// Null is rejected wherever a setting is expected.
#[test]
fn null_is_rejected_at_every_level() {
    assert_rejections(&[
        (
            r#"{ "policies": null }"#,
            "Configuration key policies must not be null",
        ),
        (
            r#"{ "hooks": null }"#,
            "Configuration key hooks must not be null",
        ),
        (
            r#"{ "indexLock": null }"#,
            "Configuration key indexLock must not be null",
        ),
        (
            r#"{ "landing": null }"#,
            "Configuration key landing must not be null",
        ),
        (
            r#"{ "hooks": { "concurrentCommits": null } }"#,
            "Configuration key hooks.concurrentCommits must not be null",
        ),
        (
            r#"{ "indexLock": { "unprovenOwnerTimeoutMs": null } }"#,
            "Configuration key indexLock.unprovenOwnerTimeoutMs must not be null",
        ),
        (
            r#"{ "landing": { "reserveAfterLostRaces": null } }"#,
            "Configuration key landing.reserveAfterLostRaces must not be null",
        ),
        (
            r#"{ "policies": { "require-root": null } }"#,
            "Configuration key policies.require-root must not be null",
        ),
        (
            r#"{ "policies": { "markdown/autofix": [null, {}] } }"#,
            "Configuration key policies.markdown/autofix[0] must not be null",
        ),
        (
            r#"{ "policies": { "markdown/autofix": ["warn", null] } }"#,
            "Configuration key policies.markdown/autofix[1] must not be null",
        ),
        (
            r#"{ "policies": { "markdown/autofix": ["warn", { "rules": null }] } }"#,
            "Configuration key policies.markdown/autofix[1].rules must not be null",
        ),
        (
            r#"{ "policies": { "markdown/autofix": ["warn", { "exclude": [null] }] } }"#,
            "Configuration key policies.markdown/autofix[1].exclude[0] must not be null",
        ),
        (
            r#"{ "policies": { "security/forbidden-strings": ["warn", { "builtinRules": null }] } }"#,
            "Configuration key policies.security/forbidden-strings[1].builtinRules must not be null",
        ),
    ]);
}
