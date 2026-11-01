//! What:
//!  Encoding and field-order controls for wrapper-made JSONL events.
//! Why:
//!  Every event must be one valid JSON object on one line,
//!  whatever a path or
//!      configuration key contains,
//!  with the field order existing consumers saw.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(renderEngineFailure(0, 'config-invalid', 'x')).toBe(JSON.stringify({ ... }) + '\n');
//! ```

/// Import the rendering under test.
use super::{
    EngineFailureCode, LEGACY_CONFIG_IGNORED_CODE, SCHEMA_VERSION, engine_failure_code_name,
    json_string, render_configuration_warning, render_engine_failure,
};

/// The engine failure line without its additive hint field, which the incumbent's shape does not have.
fn bare_engine_failure(sequence: u64, code: EngineFailureCode, message: &str) -> String {
    return crate::test_support::strip_hint_fields(render_engine_failure(sequence, code, message).as_str());
}

/// The configuration warning line without its additive hint field.
fn bare_configuration_warning(sequence: u64, code: &str, message: &str, path: &str) -> String {
    return crate::test_support::strip_hint_fields(render_configuration_warning(sequence, code, message, path).as_str());
}

/// Every code has the incumbent's wire spelling.
#[test]
fn failure_codes_have_stable_wire_spellings() {
    assert_eq!(SCHEMA_VERSION, 1);
    assert_eq!(LEGACY_CONFIG_IGNORED_CODE, "legacy-config-ignored");
    for (code, name) in [
        (EngineFailureCode::ConfigInvalid, "config-invalid"),
        (EngineFailureCode::CoreIncomplete, "core-incomplete"),
        (EngineFailureCode::ContentUnavailable, "content-unavailable"),
        (EngineFailureCode::PolicyIncomplete, "policy-incomplete"),
        (EngineFailureCode::PatchInvalid, "patch-invalid"),
        (EngineFailureCode::PatchConflict, "patch-conflict"),
        (EngineFailureCode::FixCycle, "fix-cycle"),
        (EngineFailureCode::FixPassLimit, "fix-pass-limit"),
        (EngineFailureCode::TransactionFailed, "transaction-failed"),
        (
            EngineFailureCode::IndexLockUnprovenOwner,
            "index-lock-unproven-owner",
        ),
    ] {
        assert_eq!(engine_failure_code_name(code), name);
    }
}

/// Strings are encoded exactly as `JSON.stringify` encodes them.
#[test]
fn json_strings_match_json_stringify() {
    for (input, expected) in [
        ("", r#""""#),
        ("plain", r#""plain""#),
        ("say \"hi\"", r#""say \"hi\"""#),
        ("back\\slash", r#""back\\slash""#),
        ("line\nfeed", r#""line\nfeed""#),
        ("carriage\rreturn", r#""carriage\rreturn""#),
        ("tab\tstop", r#""tab\tstop""#),
        ("\u{8}\u{c}", r#""\b\f""#),
        ("\u{0}\u{1}\u{1f}", r#""\u0000\u0001\u001f""#),
        ("\u{b}", r#""\u000b""#),
        // The first character that needs no escape, DEL, and non-ASCII text pass through.
        (" \u{7f}é\u{2028}\u{1F600}", "\" \u{7f}é\u{2028}\u{1F600}\""),
        ("</script>'", r#""</script>'""#),
        (
            "\"}\n{\"type\":\"finding\"",
            r#""\"}\n{\"type\":\"finding\"""#,
        ),
    ] {
        assert_eq!(json_string(input), expected, "{input:?}");
    }
}

/// An engine failure is one line with the incumbent field order.
#[test]
fn engine_failure_is_one_compact_line() {
    assert_eq!(
        bare_engine_failure(
            0,
            EngineFailureCode::ConfigInvalid,
            "Unknown configuration key: x."
        ),
        "{\"schemaVersion\":1,\"sequence\":0,\"type\":\"engine-failure\",\"code\":\"config-invalid\",\"message\":\"Unknown configuration key: x.\"}\n"
    );
    assert_eq!(
        bare_engine_failure(7, EngineFailureCode::TransactionFailed, ""),
        "{\"schemaVersion\":1,\"sequence\":7,\"type\":\"engine-failure\",\"code\":\"transaction-failed\",\"message\":\"\"}\n"
    );
    // A message that tries to end the string, the object and the line stays inside one string.
    let hostile: String = bare_engine_failure(
        18_446_744_073_709_551_615,
        EngineFailureCode::ConfigInvalid,
        "/repo/a\"}\n{\"type\":\"finding\"}\\",
    );
    assert_eq!(
        hostile,
        "{\"schemaVersion\":1,\"sequence\":18446744073709551615,\"type\":\"engine-failure\",\"code\":\"config-invalid\",\"message\":\"/repo/a\\\"}\\n{\\\"type\\\":\\\"finding\\\"}\\\\\"}\n"
    );
    assert_eq!(hostile.matches('\n').count(), 1);
    assert!(hostile.ends_with("}\n"));
}

/// A configuration warning is one line carrying code,
///  message and path,
///  each encoded.
#[test]
fn configuration_warning_is_one_compact_line() {
    assert_eq!(
        bare_configuration_warning(
            2,
            LEGACY_CONFIG_IGNORED_CODE,
            "Legacy configuration is ignored.",
            "/repo/cli-git.config.ts"
        ),
        "{\"schemaVersion\":1,\"sequence\":2,\"type\":\"configuration-warning\",\"code\":\"legacy-config-ignored\",\"message\":\"Legacy configuration is ignored.\",\"path\":\"/repo/cli-git.config.ts\"}\n"
    );
    let hostile: String = bare_configuration_warning(0, "a\"b", "m\nn", "p\\q\t");
    assert_eq!(
        hostile,
        "{\"schemaVersion\":1,\"sequence\":0,\"type\":\"configuration-warning\",\"code\":\"a\\\"b\",\"message\":\"m\\nn\",\"path\":\"p\\\\q\\t\"}\n"
    );
    assert_eq!(hostile.matches('\n').count(), 1);
}
