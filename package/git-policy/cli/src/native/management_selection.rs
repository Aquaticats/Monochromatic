//! What:
//!  The refusal of a direct `git cli-git check` or `fix` whose `--policy` selects a policy that is off.
//! Why:
//!  A policy that is off runs nothing, so a run limited to it would exit 0 with no output and read as
//!      "checked and clean".
//!  A CI step or a person who selected the policy by name must learn that it did not run,
//!      and which `cli-git.config.jsonc` key turns it on.
//!  The commit, add and push lifecycles are not direct runs and keep skipping policies that are off.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const off = selected.filter(id => config.severity(id) === 'off'); if (off.length) throw new ConfigError(...);
//! ```

/// Import the refusal's error type.
use super::config_error::ConfigError;
/// Import the severities in effect.
use super::config_schema::{PolicyConfig, PolicySetting};
/// Import the registry rows and the severity spellings.
use super::policy_registry::{PolicyDescriptor, PolicyId, Severity, policy_descriptor, severity_name};

/// What:
///  The refusal text for one selected policy that is off.
///  `PolicySetting::explicit` tells a listed `"off"` from a policy the file does not name.
/// Why:
///   The two remedies differ (change a value, or add a key),
///  and both are spelled with the exact key and a severity that turns the policy on:
///  the policy's own default.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const offMessage = (name: string, explicit: boolean, on: string): string => `...`;
/// ```
fn off_message(name: &str, explicit: bool, on: &str) -> String {
    let cause: String = if explicit {
        format!("cli-git.config.jsonc sets policies.\"{name}\" to \"off\"")
    } else {
        format!("cli-git.config.jsonc does not list \"{name}\", and an unlisted policy of this kind is off")
    };
    return format!(
        "Policy {name} is off, so --policy {name} would check nothing and exit clean: {cause}. \
         To turn it on, set \"policies\": {{ \"{name}\": \"{on}\" }} in cli-git.config.jsonc and run the command again."
    );
}

/// What:
///  The refusal for a direct command, when any selected policy has severity `off`;
///  `None` when nothing is selected or every selected policy runs.
///  `&[PolicyId]` is the `--policy` selection and `&PolicyConfig` the severities in effect.
/// Why:
///   One message names every off policy, in selection order, so a repeated `--policy` is answered once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function offSelectionError(selected: PolicyId[], policies: PolicyConfig): ConfigError | undefined;
/// ```
pub fn off_selection_error(selected: &[PolicyId], policies: &PolicyConfig) -> Option<ConfigError> {
    let mut messages: Vec<String> = Vec::new();
    // A policy named twice gets one sentence.
    let mut seen: Vec<PolicyId> = Vec::new();
    for id in selected {
        let setting: PolicySetting = policies.setting(*id);
        if setting.severity != Severity::Off || seen.contains(id) {
            continue;
        }
        seen.push(*id);
        let descriptor: &PolicyDescriptor = policy_descriptor(*id);
        let on: &str = severity_name(descriptor.default_severity);
        messages.push(off_message(descriptor.name, setting.explicit, on));
    }
    if messages.is_empty() {
        return None;
    }
    return Some(ConfigError::new(messages.join(" ").as_str()));
}

/// Refusal controls stay out of the release executable.
#[cfg(test)]
#[path = "management_selection_tests.rs"]
mod tests;
