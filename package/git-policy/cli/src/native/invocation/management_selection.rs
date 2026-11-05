//! What:
//!  The `--policy` selection of a direct `git monochromatic-git-policy-cli check` or `fix`:
//!  the names turned into shipped policies,
//!  and the refusal of a selection that names a policy which is off.
//! Why:
//!  A mistyped policy ID must stop the command instead of selecting nothing.
//!  A policy that is off runs nothing,
//!  so a run limited to it would exit 0 with no output and read as "checked and clean".
//!  A CI step or a person who selected the policy by name must learn that it did not run,
//!  and which `monochromatic-git-policy-cli.config.jsonc` key turns it on.
//!  The commit,
//! add and push lifecycles are not direct runs and keep skipping policies that are off.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const ids = names.map(shippedPolicy); const off = ids.filter(id => config.severity(id) === 'off'); if (off.length) throw new ConfigError(...);
//! ```

/// Import the refusal's error type.
use crate::config_error::ConfigError;
/// Import the severities in effect.
use crate::config_schema::{PolicyConfig, PolicySetting};
/// Import the registry rows and the severity spellings.
use crate::policy_registry::{
    POLICY_REGISTRY, PolicyDescriptor, PolicyId, Severity, policy_by_name, policy_descriptor, severity_name,
};
/// Import the lifecycle points and which policy runs at which.
use crate::policy_trigger::{Trigger, policy_runs_on};

/// What:
///  Turn the selected policy names into policy identities.
///       `Result<Vec<PolicyId>, ConfigError>` is the identities,
///  or the error naming the
///       first name no shipped policy has.
/// Why:
///   `--policy` selects among shipped policies only;
///  a mistyped ID must stop the
///       command instead of selecting nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function selectedPolicies(names: string[]): PolicyId[]; // throws ConfigError on an unknown name
/// ```
pub fn selected_policies(names: &[String]) -> Result<Vec<PolicyId>, ConfigError> {
    // `Vec::<PolicyId>::new()` is an empty owned list; `mut` allows pushing.
    let mut selected: Vec<PolicyId> = Vec::<PolicyId>::new();
    // `for name in names` borrows each selected ID in order.
    for name in names {
        // `match` unpacks "a registry row or nothing".
        match policy_by_name(name.as_str()) {
            Some(descriptor) => selected.push(descriptor.id),
            None => {
                let mut known: String = String::new();
                for descriptor in POLICY_REGISTRY {
                    if !known.is_empty() {
                        known.push_str(", ");
                    }
                    known.push_str(descriptor.name);
                }
                // `Err(...)` is the failure variant carrying the error.
                return Err(ConfigError::new(
                    format!("Unknown policy ID: {name}. Shipped policies: {known}.").as_str(),
                ));
            }
        }
    }
    // `Ok(...)` is the success variant: every selected ID is shipped.
    return Ok(selected);
}

/// What:
///  The refusal text for one selected policy that is off.
///  `PolicySetting::explicit` tells a listed `"off"` from a policy the file does not name.
/// Why:
///   The two remedies differ (change a value,
/// or add a key),
///  and both are spelled with the exact key and a severity that turns the policy on:
///  the policy's own default.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const offMessage = (name: string, explicit: boolean, on: string): string => `...`;
/// ```
fn off_message(name: &str, explicit: bool, on: &str) -> String {
    let cause: String = if explicit {
        format!("monochromatic-git-policy-cli.config.jsonc sets policies.\"{name}\" to \"off\"")
    } else {
        format!("monochromatic-git-policy-cli.config.jsonc does not list \"{name}\", and an unlisted policy of this kind is off")
    };
    return format!(
        "Policy {name} is off, so --policy {name} would check nothing and exit clean: {cause}. \
         To turn it on, set \"policies\": {{ \"{name}\": \"{on}\" }} in monochromatic-git-policy-cli.config.jsonc and run the command again."
    );
}

/// What:
///  The refusal text for one selected policy that is on but does not run for this direct command.
///  `runs_for` is the direct commands that do run it,
/// in `check`,
/// `fix` order.
/// Why:
///   A run limited to it would check nothing and exit clean,
/// exactly as for a policy that is off.
/// The remedy is the command that does run it,
/// or,
/// for a policy that runs on neither,
/// the Git command it guards.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unrunMessage = (name: string, command: string, runsFor: string[]): string => `...`;
/// ```
fn unrun_message(name: &str, command: &str, runs_for: &[&str]) -> String {
    if runs_for.is_empty() {
        return format!(
            "Policy {name} does not run for the monochromatic-git-policy-cli check or fix commands, only when the Git command it guards is \
             forwarded, so --policy {name} would check nothing and exit clean. Run that Git command to see it."
        );
    }
    let mut commands: Vec<String> = Vec::new();
    for word in runs_for {
        commands.push(format!("monochromatic-git-policy-cli {word}"));
    }
    return format!(
        "Policy {name} does not run for monochromatic-git-policy-cli {command}, so --policy {name} would check nothing and exit clean. \
         It runs for {}.",
        commands.join(" or ")
    );
}

/// What:
///  The refusal for a direct command,
/// when any selected policy has severity `off`,
/// or is on but does not run for this command;
///  `None` when nothing is selected or every selected policy runs.
///  `&[PolicyId]` is the `--policy` selection,
/// `&PolicyConfig` the severities in effect,
/// and `trigger` is `DirectCheck` or `DirectFix`.
/// Why:
///   One message names every such policy,
/// in selection order,
/// so a repeated `--policy` is answered once and a run never silently drops a selected policy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function selectionError(selected: PolicyId[], policies: PolicyConfig, trigger: PolicyTrigger): ConfigError | undefined;
/// ```
pub fn selection_error(selected: &[PolicyId], policies: &PolicyConfig, trigger: Trigger) -> Option<ConfigError> {
    let mut messages: Vec<String> = Vec::new();
    // A policy named twice gets one sentence.
    let mut seen: Vec<PolicyId> = Vec::new();
    for id in selected {
        if seen.contains(id) {
            continue;
        }
        seen.push(*id);
        let setting: PolicySetting = policies.setting(*id);
        let descriptor: &PolicyDescriptor = policy_descriptor(*id);
        if setting.severity == Severity::Off {
            let on: &str = severity_name(descriptor.default_severity);
            messages.push(off_message(descriptor.name, setting.explicit, on));
        } else if !policy_runs_on(*id, trigger) {
            let mut runs_for: Vec<&str> = Vec::new();
            if policy_runs_on(*id, Trigger::DirectCheck) {
                runs_for.push("check");
            }
            if policy_runs_on(*id, Trigger::DirectFix) {
                runs_for.push("fix");
            }
            let command: &str = if trigger == Trigger::DirectFix { "fix" } else { "check" };
            messages.push(unrun_message(descriptor.name, command, runs_for.as_slice()));
        }
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
