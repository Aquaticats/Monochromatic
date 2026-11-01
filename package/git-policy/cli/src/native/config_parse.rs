//! What:
//!  Turn `cli-git.config.jsonc` text into validated typed settings.
//! Why:
//!  Every unknown key,
//!  wrong type or ambiguous duplicate is rejected before any
//!      policy or transaction reads a setting.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // parseConfig(source) returns CliGitConfig or throws ConfigError naming the key.
//! ```

/// What:
///  `use super::...` imports sibling modules of this crate.
/// Why:
///   Parsing combines the error type,
///  the policy section reader,
///  the typed
///       settings and the shared value readers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { ConfigError } from './config-error.ts';
/// ```
use super::config_error::ConfigError;
use super::config_policies::parse_policies;
use super::config_schema::{CliGitConfig, ConcurrencyConfig};
use super::config_values::{boolean, member_keys, member_path, safe_integer, wrong_kind};
/// What:
///  Import the repository's JSONC parser and its value type.
/// Why:
///   cli-git reads its configuration through the shared JSONC package only.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { parseJsonc, type JsoncValue } from 'jsonc-edit';
/// ```
use monochromatic_jsonc_edit::{JsoncValue, parse_jsonc};

/// What:
///  Every accepted top-level key,
///  for the unknown-key diagnostic.
///       `&str` is borrowed text compiled into the executable (sibling:
///  owned `String`).
/// Why:
///   One list keeps the message and the dispatch from drifting apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ACCEPTED_KEYS = 'policies, hooks, indexLock, landing';
/// ```
const ACCEPTED_KEYS: &str = "policies, hooks, indexLock, landing";

/// What:
///  Explain a top-level key the schema does not accept.
/// Why:
///   `plugins` and `trust` existed in the executable TypeScript configuration;
///  their
///       authors need the reason and the remedy,
///  not a bare "unknown key".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function unknownTopLevelKey(key: string): ConfigError;
/// ```
fn unknown_top_level_key(key: &str) -> ConfigError {
    if key == "plugins" {
        return ConfigError::new(
            "Configuration key plugins is retired: policies are compiled into cli-git and \
             JSONC configuration cannot load plugin code. Remove the key and configure the \
             shipped policies under policies.",
        );
    }
    if key == "trust" {
        return ConfigError::new(
            "Configuration key trust is retired: JSONC configuration is data and needs no \
             code-execution approval. Remove the key.",
        );
    }
    // What: `format!` builds owned text; `.as_str()` lends it to the constructor.
    // Why:  The message names the offending key and every accepted one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return new ConfigError(`Unknown configuration key: ${key}. Accepted keys: ...`);
    // ```
    return ConfigError::new(
        format!("Unknown configuration key: {key}. Accepted keys: {ACCEPTED_KEYS}.").as_str(),
    );
}

/// What:
///  Validate one concurrency section and apply its single optional key.
///       `Result<T, E>` is "value or error":
///  `Ok(...)` or `Err(...)`.
/// Why:
///   `hooks`,
///  `indexLock` and `landing` share one shape:
///  an object whose only
///       accepted key may be omitted to keep the default.
///  They tune hook serialization,
///       index-lock patience and landing starvation;
///  none can disable the transaction.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function applyConcurrency(base: ConcurrencyConfig, key: string, value: JsoncValue): ConcurrencyConfig;
/// ```
fn apply_concurrency(
    base: ConcurrencyConfig,
    key: &str,
    value: &JsoncValue,
) -> Result<ConcurrencyConfig, ConfigError> {
    // The single key each section accepts.
    let allowed: &str = if key == "hooks" {
        "concurrentCommits"
    } else if key == "indexLock" {
        "unprovenOwnerTimeoutMs"
    } else {
        "reserveAfterLostRaces"
    };
    // What: `let Some(x) = ... else { return ... };` unwraps the present case or exits.
    // Why:  A section that is not an object has no keys to read.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (!isRecord(value)) throw wrongKind(value, key, "an object");
    // ```
    let Some(entries) = value.entries() else {
        // `Err(...)` is the failure variant returned to the caller.
        return Err(wrong_kind(value, key, "an object"));
    };
    // A trailing `?` returns the reader error to our caller, or unwraps its value.
    let keys: Vec<String> = member_keys(entries, key)?;
    // `ConcurrencyConfig` is `Copy`, so this is an independent editable copy.
    let mut result: ConcurrencyConfig = base;
    // What: `.iter().enumerate()` yields `(index, item)` pairs; `usize` is the index type.
    // Why:  The decoded keys are parallel to `entries`, so the index reaches the value.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (const [index, member] of keys.entries()) { ... }
    // ```
    for (index, member) in keys.iter().enumerate() {
        let path: String = member_path(key, member.as_str());
        if member != allowed {
            return Err(ConfigError::new(
                format!(
                    "Unknown configuration key: {path}. The only accepted key under {key} is {allowed}."
                )
                .as_str(),
            ));
        }
        // `&entries[index].value` borrows the member value parallel to this key.
        let member_value: &JsoncValue = &entries[index].value;
        if key == "hooks" {
            result.hooks.concurrent_commits = boolean(member_value, path.as_str())?;
        } else if key == "indexLock" {
            result.index_lock.unproven_owner_timeout_ms =
                safe_integer(member_value, path.as_str(), 0)?;
        } else {
            result.landing.reserve_after_lost_races = safe_integer(member_value, path.as_str(), 1)?;
        }
    }
    // `Ok(result)` is the success variant carrying the tuned copy.
    return Ok(result);
}

/// What:
///  Parse and validate one complete configuration document.
/// Why:
///   This is the only way settings enter the native wrapper:
///  data in,
///  typed
///       settings or one actionable error out.
///  Nothing in the document is executed,
///       imported or resolved as a path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseConfig(source: string): CliGitConfig;
/// ```
pub fn parse_config(source: &str) -> Result<CliGitConfig, ConfigError> {
    // What: `match` on the parser's `Result` keeps the value or converts its error.
    //       `Ok(parsed) => parsed` unwraps success; `Err(error) => { ... }` handles failure.
    // Why:  A syntax failure must keep its byte offset so the author can find it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let document; try { document = parseJsonc(source); } catch (e) { throw new ConfigError(...); }
    // ```
    let document: JsoncValue = match parse_jsonc(source) {
        Ok(parsed) => parsed,
        Err(error) => {
            return Err(ConfigError::new(
                format!(
                    "JSONC syntax error at byte {}: {}",
                    error.offset, error.message
                )
                .as_str(),
            ));
        }
    };
    let Some(entries) = document.entries() else {
        // `&document` lends the value to the diagnostic builder.
        return Err(wrong_kind(&document, "", "an object"));
    };
    let keys: Vec<String> = member_keys(entries, "")?;
    let mut config: CliGitConfig = CliGitConfig::defaults();
    for (index, key) in keys.iter().enumerate() {
        // `&entries[index].value` borrows the member value parallel to this key.
        let value: &JsoncValue = &entries[index].value;
        if key == "policies" {
            config.policies = parse_policies(value)?;
        } else if key == "hooks" || key == "indexLock" || key == "landing" {
            config.concurrency = apply_concurrency(config.concurrency, key.as_str(), value)?;
        } else {
            return Err(unknown_top_level_key(key.as_str()));
        }
    }
    return Ok(config);
}

/// Acceptance controls stay out of the release executable.
#[cfg(test)]
#[path = "config_parse_acceptance_tests.rs"]
mod acceptance_tests;

/// Structural rejection controls and the shared rejection helpers.
#[cfg(test)]
#[path = "config_parse_tests.rs"]
mod tests;

/// Invalid-value rejection controls.
#[cfg(test)]
#[path = "config_parse_value_tests.rs"]
mod value_tests;
