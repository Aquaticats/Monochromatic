//! What:
//!  The decision behind automatic maintenance after a landing:
//!  parsing `git config --null --get-regexp` records,
//!  and choosing whether `git maintenance run --auto` runs and whether it detaches.
//! Why:
//!  Native `git commit` runs this step after updating `HEAD`;
//!  a landing replaces that commit with plumbing,
//!  so the pack count grows by one per commit unless this step runs.
//!  The rules mirror Git's `prepare_auto_maintenance` in `run-command.c`,
//!  including JavaScript number semantics for values the incumbent reads with `Number(...)`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const args = autoMaintenanceArgs(parseConfigRecords(output));
//! ```

/// `BTreeMap` keeps config keys sorted; its `insert` keeps the last value of a repeated key.
use std::collections::BTreeMap;
/// `fmt` formats the error for diagnostics.
use std::fmt;

/// The config key pattern the decision reads, as `git config --get-regexp` matches it.
///  Keys are canonicalized as Git prints them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MAINTENANCE_CONFIG_PATTERN = '^(maintenance\\.auto|maintenance\\.autodetach|gc\\.auto|gc\\.autodetach)$';
/// ```
pub const MAINTENANCE_CONFIG_PATTERN: &str =
    r"^(maintenance\.auto|maintenance\.autodetach|gc\.auto|gc\.autodetach)$";

/// What:
///  Why automatic maintenance could not be decided or run.
/// Why:
///   Each variant is a failure the caller can name;
///  a start failure is kept as text so the error stays comparable in tests.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class AutoMaintenanceError extends Error {}
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MaintenanceError {
    /// Git's config output for the maintenance keys is not UTF-8.
    ConfigNotUtf8,
    /// Real Git could not be started; the text names the command and the operating-system reason.
    Start(String),
}

/// Display text for a failed maintenance decision or run.
impl fmt::Display for MaintenanceError {
    /// What:
    ///  Human-readable text for the variant.
    /// Why:
    ///   Diagnostics print the error without a debug format.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// toString(): string
    /// ```
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConfigNotUtf8 => {
                return formatter.write_str("git config output for maintenance is not UTF-8");
            }
            Self::Start(reason) => return formatter.write_str(reason.as_str()),
        }
    }
}

/// The error trait lets callers box this error.
impl std::error::Error for MaintenanceError {}

/// What:
///  The arguments automatic maintenance runs, or the fact that configuration turned it off.
/// Why:
///   The incumbent's disabled marker is a symbol;
///  an enum keeps the two outcomes distinct without a sentinel value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MaintenanceDecision = typeof AUTO_MAINTENANCE_DISABLED | readonly string[];
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MaintenanceDecision {
    /// `maintenance.auto`, or a non-positive `gc.auto` when it is unset, turns maintenance off.
    Disabled,
    /// Run `git maintenance run --auto --quiet` with the listed arguments.
    Run(Vec<&'static str>),
}

/// What:
///  Parse `git config --type=bool-or-int --null --get-regexp` output into canonical key to value.
///  A record without a newline is a key with no value, which Git reads as `true`.
/// Why:
///   Git prints one NUL-terminated record per key occurrence;
///  the last occurrence wins, as Git's single-value lookup does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseConfigRecords(output: string): Record<string, string>;
/// ```
pub fn parse_config_records(output: &[u8]) -> Result<BTreeMap<String, String>, MaintenanceError> {
    // Git's config output is decoded strictly, as the incumbent's `TextDecoder` with `fatal`.
    let text: &str = std::str::from_utf8(output).map_err(|_| return MaintenanceError::ConfigNotUtf8)?;
    // `mut` lets each record overwrite an earlier value of the same key.
    let mut parsed: BTreeMap<String, String> = BTreeMap::new();
    for record in text.split('\0') {
        if record.is_empty() {
            continue;
        }
        match record.split_once('\n') {
            Some((key, value)) => {
                parsed.insert(String::from(key), String::from(value));
            }
            None => {
                parsed.insert(String::from(record), String::from("true"));
            }
        }
    }
    return Ok(parsed);
}

/// What:
///  Interpret a `bool-or-int` value as Git's boolean:
///  `true` and `false` by name, otherwise any non-zero number.
/// Why:
///   The incumbent's `Number(value) !== 0` reads a value that is neither as true,
///  because `NaN !== 0`;
///  the same rule applies here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function boolOrIntTrue(value: string): boolean { return value === 'true' || (value !== 'false' && Number(value) !== 0); }
/// ```
fn bool_or_int_true(value: &str) -> bool {
    if value == "true" {
        return true;
    }
    if value == "false" {
        return false;
    }
    return javascript_number(value) != 0.0;
}

/// What:
///  Read a config value as JavaScript's `Number(...)` does:
///  an empty or blank value is zero, and text that is not a number is `NaN`.
/// Why:
///   The incumbent compares `Number(gc.auto)` against `0`,
///  so `NaN` makes both comparisons false.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Number(value)
/// ```
fn javascript_number(value: &str) -> f64 {
    let trimmed: &str = value.trim();
    if trimmed.is_empty() {
        return 0.0;
    }
    return trimmed.parse::<f64>().unwrap_or(f64::NAN);
}

/// What:
///  The arguments native `git commit` would give `git maintenance run`,
///  or `Disabled` when configuration turns automatic maintenance off.
/// Why:
///   `maintenance.auto` takes precedence over `gc.auto`;
///  the detach setting follows the same precedence.
///  Detaching is the default, as in Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// autoMaintenanceArgs({}); // ['maintenance', 'run', '--auto', '--quiet', '--detach']
/// ```
pub fn auto_maintenance_arguments(config: &BTreeMap<String, String>) -> MaintenanceDecision {
    // `get` returns the value of a key that Git printed, or nothing when it was unset.
    let maintenance_auto: Option<&String> = config.get("maintenance.auto");
    let gc_auto: Option<&String> = config.get("gc.auto");
    // Whether maintenance runs: an explicit `maintenance.auto` decides alone, otherwise `gc.auto`.
    let enabled: bool = match maintenance_auto {
        Some(value) => bool_or_int_true(value.as_str()),
        None => match gc_auto {
            None => true,
            Some(value) => javascript_number(value.as_str()) > 0.0,
        },
    };
    if !enabled {
        return MaintenanceDecision::Disabled;
    }
    // The first configured detach setting; unset means detach.
    let detach_setting: Option<&String> = config
        .get("maintenance.autodetach")
        .or_else(|| return config.get("gc.autodetach"));
    let detach: bool = match detach_setting {
        None => true,
        Some(value) => bool_or_int_true(value.as_str()),
    };
    let flag: &'static str = if detach { "--detach" } else { "--no-detach" };
    return MaintenanceDecision::Run(vec!["maintenance", "run", "--auto", "--quiet", flag]);
}

/// Decision controls live beside the decision.
#[cfg(test)]
#[path = "maintenance_decision_tests.rs"]
mod tests;
