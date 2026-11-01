//! What:
//!  Validated JSONC to typed Rust dispatch controls.
//! Why:
//!  Omission,
//!  explicit off and selected severity have different configuration meanings.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Resolve typed settings, then exercise the selected syntax checks through their consumer boundary.
//! ```

/// Import the real conversion,
///  shared diagnostics and syntax dispatcher.
use super::{LineBudget, RustRuleSettings, rust_rule_settings};
use crate::diagnostic::{Diagnostic, Severity};
use crate::rust_dispatch::check_syntax_rules;
use crate::rust_source::RustSource;
/// Import the repository's exact JSONC parser.
use monochromatic_jsonc_edit::{JsoncValue, parse_jsonc};

/// Convert one owned fixture without replacing configuration validation.
fn settings(source: &str) -> RustRuleSettings {
    let value: JsoncValue = parse_jsonc(source).expect("JSONC fixture");
    return rust_rule_settings(&value).expect("complete valid rule settings");
}

/// Absent and explicitly disabled rules do not initialize or run any check.
#[test]
fn disabled_and_missing_rules_remain_disabled() {
    assert_eq!(settings("{}"), RustRuleSettings::default());
    assert_eq!(
        settings(
            r#"{"rust/require-explicit-types":{"severity":"off"},"rust/require-rustdoc":{"severity":"off"},"rust/tests-in-sidecars":{"severity":"off"},"rust/max-lines":{"severity":"off"}}"#
        ),
        RustRuleSettings::default()
    );
    // This source would be reported by the line budget, the documentation rule and the sidecar rule.
    let source: RustSource = RustSource::new(
        String::from("input.rs"),
        String::from("#[test]\nfn stray() {}\n"),
    );
    assert!(check_syntax_rules(&source, &RustRuleSettings::default()).is_empty());
}

/// Selected severities and exact line limits reach the actual rules in stable order.
#[test]
fn selected_rules_receive_typed_options() {
    let selected: RustRuleSettings = settings(
        r#"{
        "rust/max-lines":{"severity":"warn","max":0},
        "rust/tests-in-sidecars":{"severity":"error"},
        "rust/require-rustdoc":{"severity":"off"},
        "rust/require-explicit-types":{"severity":"warn"}
    }"#,
    );
    assert_eq!(
        selected.max_lines,
        Some(LineBudget {
            severity: Severity::Warn,
            max: 0
        })
    );
    assert_eq!(selected.explicit_types, Some(Severity::Warn));
    assert_eq!(selected.rustdoc, None);
    assert_eq!(selected.tests_in_sidecars, Some(Severity::Error));
    let source: RustSource = RustSource::new(
        String::from("input.rs"),
        String::from("#[test]\nfn stray() {}\n"),
    );
    let findings: Vec<Diagnostic> = check_syntax_rules(&source, &selected);
    assert_eq!(findings.len(), 2);
    assert_eq!(findings[0].code, "rust/max-lines");
    assert_eq!(findings[0].severity, Severity::Warn);
    assert_eq!(findings[1].code, "rust/tests-in-sidecars");
    assert_eq!(findings[1].severity, Severity::Error);
    assert_eq!(
        settings(r#"{"rust/max-lines":{"severity":"error"}}"#).max_lines,
        Some(LineBudget {
            severity: Severity::Error,
            max: 300
        })
    );
}

/// The documentation selection reaches the actual documentation rule rather than just a settings field.
#[test]
fn selected_documentation_rule_executes() {
    let selected: RustRuleSettings = settings(r#"{"rust/require-rustdoc":{"severity":"error"}}"#);
    let source: RustSource = RustSource::new(String::from("input.rs"), String::new());
    let findings: Vec<Diagnostic> = check_syntax_rules(&source, &selected);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].code, "rust/require-rustdoc");
}

/// Other valid language rules do not enable Rust rules;
///  invalid input is never silently ignored.
#[test]
fn language_selection_keeps_schema_failures_visible() {
    assert_eq!(
        settings(r#"{"markdown/single-h1":{"severity":"error"}}"#),
        RustRuleSettings::default()
    );
    for source in [
        r#"{"rust/require-explicit-types":{}}"#,
        r#"{"rust/require-rustdoc":{"severity":"warning"}}"#,
        r#"{"rust/require-rustdoc":{"severity":"off","max":1}}"#,
        r#"{"unknown/rule":{"severity":"off"}}"#,
        // A rule that was removed from the registry is as unknown as one never registered.
        r#"{"rust/no-anonymous-functions":{"severity":"off"}}"#,
        r#"{"rust/max-lines":{"severity":"error","severity":"off"}}"#,
        r#"{"rust/max-lines":{"severity":"error","max":null}}"#,
        "[]",
    ] {
        let value: JsoncValue = parse_jsonc(source).expect("syntactically valid fixture");
        assert!(rust_rule_settings(&value).is_err(), "{source}");
    }
}

/// The sidecar test rule is selected by its own id,
/// stays off when absent or off,
///  and runs through the syntax dispatcher with the selected severity.
#[test]
fn the_sidecar_test_rule_is_selected_by_its_id_and_dispatched() {
    assert_eq!(settings("{}").tests_in_sidecars, None);
    assert_eq!(
        settings(r#"{"rust/tests-in-sidecars":{"severity":"off"}}"#).tests_in_sidecars,
        None
    );
    let selected: RustRuleSettings = settings(r#"{"rust/tests-in-sidecars":{"severity":"warn"}}"#);
    assert_eq!(selected.tests_in_sidecars, Some(Severity::Warn));
    // Only this rule is selected, so no other check can contribute a finding.
    assert_eq!(selected.rustdoc, None);
    assert_eq!(selected.max_lines, None);
    let source: RustSource = RustSource::new(
        String::from("input.rs"),
        String::from("#[test]\nfn stray() {}\n"),
    );
    let findings: Vec<Diagnostic> = check_syntax_rules(&source, &selected);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].code, "rust/tests-in-sidecars");
    assert_eq!(findings[0].severity, Severity::Warn);
}

/// A configuration that still names the removed anonymous-function rule fails like any unknown rule,
///  naming that rule and the running binary,
///  whatever severity it asks for.
#[test]
fn a_removed_rule_name_is_reported_as_an_unknown_built_in_rule() {
    // Read the version from the manifest so a release bump cannot leave this assertion stale.
    let expected: String = format!(
        "Unknown built-in rule rust/no-anonymous-functions. (monochromatic-lint {})",
        env!("CARGO_PKG_VERSION")
    );
    for severity in ["off", "warn", "error"] {
        let source: String =
            format!(r#"{{"rust/no-anonymous-functions":{{"severity":"{severity}"}}}}"#);
        let value: JsoncValue = parse_jsonc(source.as_str()).expect("syntactically valid fixture");
        let error = rust_rule_settings(&value).expect_err("the removed rule is not registered");
        assert_eq!(error.message, expected.as_str(), "{severity}");
    }
}

/// The closure check carries its severity and library path,
/// and is disabled by absence and by off.
#[test]
fn the_closure_check_carries_its_library_and_severity() {
    use super::ClosureLint;
    let selected: RustRuleSettings = settings(
        r#"{"rust/avoidable-closure":{"severity":"warn","library":"package/linter/monochromatic-lint-closure"}}"#,
    );
    assert_eq!(
        selected.avoidable_closure,
        Some(ClosureLint {
            severity: Severity::Warn,
            library: String::from("package/linter/monochromatic-lint-closure")
        })
    );
    assert_eq!(
        settings(r#"{"rust/avoidable-closure":{"severity":"off","library":"lib"}}"#)
            .avoidable_closure,
        None
    );
    assert_eq!(
        settings(r#"{"rust/avoidable-closure":{"severity":"off"}}"#).avoidable_closure,
        None
    );
    assert_eq!(settings("{}").avoidable_closure, None);
}

/// A selected closure check without a library is refused,
/// and so are empty and non-text libraries.
#[test]
fn the_closure_check_needs_a_nonempty_text_library_when_selected() {
    for source in [
        r#"{"rust/avoidable-closure":{"severity":"warn"}}"#,
        r#"{"rust/avoidable-closure":{"severity":"error"}}"#,
        r#"{"rust/avoidable-closure":{"severity":"warn","library":""}}"#,
        r#"{"rust/avoidable-closure":{"severity":"warn","library":7}}"#,
        r#"{"rust/avoidable-closure":{"severity":"warn","library":["a"]}}"#,
        r#"{"rust/avoidable-closure":{"severity":"warn","library":"lib","max":3}}"#,
    ] {
        let value: JsoncValue = parse_jsonc(source).expect("JSONC fixture");
        assert!(rust_rule_settings(&value).is_err(), "{source} was accepted");
    }
}

/// The library option belongs to the closure check only.
#[test]
fn other_rules_refuse_the_library_option() {
    let value: JsoncValue =
        parse_jsonc(r#"{"rust/max-lines":{"severity":"warn","library":"lib"}}"#)
            .expect("JSONC fixture");
    assert!(rust_rule_settings(&value).is_err());
}
