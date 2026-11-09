//! What:
//! Read the publishable package names from the generated registry configuration
//!       (`package/config/pnpr/config.yaml`).
//! Why:
//! Only published packages receive a dependent bump,
//! and the publish set is the list the
//!      registry configuration trusts each OIDC workload to publish.
//! The generator writes
//!      it four levels down,
//! as `auth.oidc[].workloads[].packages` (`package/config/pnpr/src/generated-config-render.ts`),
//! and the
//!      publish run reads it back from that same path with the `yaml` package
//! (`package/config/pnpr/src/publish-plan.ts`).
//! The same file also carries a `packages:`
//!      mapping under `registries`,
//! which grants a pattern access and lists no publishable name,
//!      so a reader that took the first `packages:` line in the text could read
//! the permissions
//!      instead of the trust list.
//! This module therefore descends the parsed document instead
//!      of scanning lines.
//!
//! No crate this executable links parses YAML,
//! and adding one is a dependency
//!      decision rather than a fix for this reader,
//! so `dependent_version_yaml` reads the one structure
//!      this file needs.
//! Every OIDC provider and every workload of it counts:
//! a package any
//!      trusted workload may publish is published,
//! so a narrower list added above the publish
//!      set cannot hide it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const names = readPnprPublishTarget(configText).packageNames; // the yaml package's parse, then auth.oidc[].workloads[].packages
//! ```

/// The block document reader and its problems.
use crate::dependent_version_yaml::{YamlNode, parse_yaml_document};

/// What:
/// The key of the OIDC providers in the configuration's top-level mapping.
/// Why:
///  The publish set lives below `auth`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const AUTH = 'auth';
/// ```
const AUTH: &str = "auth";

/// What:
/// The key of the providers under `auth`.
/// Why:
///  Each provider holds the workloads a CI identity may publish with.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const OIDC = 'oidc';
/// ```
const OIDC: &str = "oidc";

/// What:
/// The key of the workloads of one provider.
/// Why:
///  One workload binds an identity to a registry and a package list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const WORKLOADS = 'workloads';
/// ```
const WORKLOADS: &str = "workloads";

/// What:
/// The key of one workload's package list.
/// Why:
///  This list is the publish set;
/// the `packages` mapping under `registries` is a
///       permission pattern and is not read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const PACKAGES = 'packages';
/// ```
const PACKAGES: &str = "packages";

/// What:
/// Every package name the configuration trusts a workload to publish,
/// in configuration
///       order and without repeats.
/// `Err` is one clause naming what the reader could not read,
/// without a final period,
///       so the caller can put it in a diagnostic beside the file name.
/// Why:
///  A name that is not listed is never bumped,
/// and an empty or unreadable list is not
///       the same answer as "nothing to bump":
/// the caller has to be able to tell a clean
///       workspace from an unjudged one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function readPublishableNames(configText: string): string[];
/// ```
pub fn read_publishable_names(config_text: &str) -> Result<Vec<String>, String> {
    let document: YamlNode = match parse_yaml_document(config_text) {
        Ok(read) => read,
        Err(problem) => {
            return Err(format!("{} on line {}", problem.reason, problem.line));
        }
    };
    let Some(auth) = document.entry(AUTH) else {
        return Err(String::from("it has no `auth.oidc` list of providers"));
    };
    let Some(oidc) = auth.entry(OIDC) else {
        return Err(String::from("it has no `auth.oidc` list of providers"));
    };
    let Some(providers) = oidc.items() else {
        return Err(String::from("it has no `auth.oidc` list of providers"));
    };
    let mut names: Vec<String> = Vec::new();
    for (provider_index, provider) in providers.iter().enumerate() {
        let workloads: &[YamlNode] = match provider.entry(WORKLOADS) {
            None => continue,
            Some(listed) => match listed.items() {
                Some(items) => items,
                None => {
                    return Err(format!(
                        "`auth.oidc[{provider_index}].workloads` is not a list of workloads"
                    ));
                }
            },
        };
        for (workload_index, workload) in workloads.iter().enumerate() {
            let Some(listed) = workload.entry(PACKAGES) else {
                continue;
            };
            let Some(items) = listed.items() else {
                // A `packages:` with nothing below it lists no name;
                // anything else there is a shape this reader refuses.
                if listed.text() == Some("") {
                    continue;
                }
                return Err(format!(
                    "`auth.oidc[{provider_index}].workloads[{workload_index}].packages` \
                     is not a list of package names"
                ));
            };
            for item in items {
                let place: String = format!(
                    "`auth.oidc[{provider_index}].workloads[{workload_index}].packages`"
                );
                let Some(name) = item.text() else {
                    return Err(format!("{place} holds a value that is not a package name"));
                };
                if name.is_empty() {
                    return Err(format!("{place} holds an empty package name"));
                }
                if !already_listed(&names, name) {
                    names.push(String::from(name));
                }
            }
        }
    }
    if names.is_empty() {
        return Err(String::from(
            "no `auth.oidc` workload lists a package name",
        ));
    }
    return Ok(names);
}

/// What:
/// Whether the names read so far already hold `name`.
/// Why:
///  One package trusted by two workloads is one publishable package,
/// and the plan
///       reads the list as a set.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// names.includes(name);
/// ```
fn already_listed(names: &[String], name: &str) -> bool {
    for found in names {
        if found == name {
            return true;
        }
    }
    return false;
}

/// Publishable-name reading,
/// over the generated shape and the ones it refuses.
#[cfg(test)]
#[path = "dependent_version_publishable_tests.rs"]
mod tests;
