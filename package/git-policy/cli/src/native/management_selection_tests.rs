//! What:
//!  Controls for the refusal of a `--policy` selection that is off.
//! Why:
//!  The refusal is the only thing between "selected a policy that does not run" and a clean exit,
//!  so which settings trigger it and what the text names are pinned for every optional policy.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(offSelectionError(['mono/catalog-specifiers'], defaults)?.message).toContain('cli-git.config.jsonc');
//! ```

/// Import the module under test.
use super::off_selection_error;
use crate::config_schema::PolicyConfig;
use crate::policy_registry::{POLICY_REGISTRY, PolicyId, Severity, policy_descriptor};
use crate::policy_test_support::with_severity;

/// The message of the refusal,
/// or a failure when there is none.
fn refusal(selected: &[PolicyId], config: &PolicyConfig) -> String {
    return off_selection_error(selected, config)
        .expect("a refusal")
        .message;
}

/// Every optional policy is off when the file does not list it,
/// and the refusal names the policy,
/// the file and the
///  key that turns it on at the policy's own default severity.
#[test]
fn an_unlisted_optional_policy_is_refused_with_the_key_that_turns_it_on() {
    let defaults: PolicyConfig = PolicyConfig::defaults();
    let mut refused: usize = 0;
    for descriptor in POLICY_REGISTRY {
        if !descriptor.off_unless_listed {
            continue;
        }
        refused += 1;
        let message: String = refusal(&[descriptor.id], &defaults);
        let on: &str = if descriptor.default_severity == Severity::Warn {
            "warn"
        } else {
            "error"
        };
        assert!(
            message.starts_with(format!("Policy {} is off,", descriptor.name).as_str()),
            "{message}"
        );
        assert!(message.contains("does not list"), "{message}");
        assert!(message.contains("cli-git.config.jsonc"), "{message}");
        assert!(
            message.contains(format!("\"policies\": {{ \"{}\": \"{on}\" }}", descriptor.name).as_str()),
            "{message}"
        );
    }
    assert_eq!(refused, 5, "the four plugin policies and the catalog policy are optional");
}

/// A policy the file lists as `off` gets the remedy for a listed key,
/// not for a missing one.
#[test]
fn a_policy_listed_as_off_is_refused_with_its_listed_value() {
    let mut config: PolicyConfig =
        with_severity(&PolicyConfig::defaults(), PolicyId::CatalogSpecifiers, Severity::Off);
    for setting in &mut config.settings {
        setting.explicit = setting.id == PolicyId::CatalogSpecifiers;
    }
    let message: String = refusal(&[PolicyId::CatalogSpecifiers], &config);
    assert!(
        message.contains("sets policies.\"mono/catalog-specifiers\" to \"off\""),
        "{message}"
    );
    assert!(!message.contains("does not list"), "{message}");
}

/// Policies that run,
/// no selection,
/// and built-in policies (never off by default) are not refused.
#[test]
fn running_policies_and_an_empty_selection_are_not_refused() {
    let defaults: PolicyConfig = PolicyConfig::defaults();
    assert_eq!(off_selection_error(&[], &defaults), None);
    assert_eq!(
        off_selection_error(&[PolicyId::RequireRoot, PolicyId::FinalNewline], &defaults),
        None
    );
    for severity in [Severity::Warn, Severity::Error] {
        let on: PolicyConfig = with_severity(&defaults, PolicyId::CatalogSpecifiers, severity);
        assert_eq!(
            off_selection_error(&[PolicyId::CatalogSpecifiers], &on),
            None,
            "{severity:?}"
        );
    }
}

/// Every off policy of a selection is named once,
/// in selection order,
/// and a running one is left out.
#[test]
fn every_off_policy_of_a_selection_is_named_once() {
    let config: PolicyConfig =
        with_severity(&PolicyConfig::defaults(), PolicyId::ForbiddenStrings, Severity::Error);
    let message: String = refusal(
        &[
            PolicyId::CatalogSpecifiers,
            PolicyId::ForbiddenStrings,
            PolicyId::DependentVersionBump,
            PolicyId::CatalogSpecifiers,
        ],
        &config,
    );
    assert_eq!(
        message.matches("Policy mono/catalog-specifiers is off").count(),
        1,
        "{message}"
    );
    assert_eq!(
        message.matches("Policy mono/dependent-version-bump is off").count(),
        1,
        "{message}"
    );
    assert!(!message.contains("security/forbidden-strings is off"), "{message}");
    let first: usize = message.find("mono/catalog-specifiers is off").expect("first");
    let second: usize = message.find("mono/dependent-version-bump is off").expect("second");
    assert!(first < second, "{message}");
    assert_eq!(policy_descriptor(PolicyId::CatalogSpecifiers).name, "mono/catalog-specifiers");
}
