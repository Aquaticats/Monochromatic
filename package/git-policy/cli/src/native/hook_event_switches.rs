//! What:
//!  The user's `hook.<event>.enabled` switches for the commit events, read from the configuration snapshot or from one
//!  `git config` start.
//! Why:
//!  The hook plan needs to know which events the user disabled, so a disabled event takes no hook lock and starts no
//!  hook runner; Git itself applies the switch to the hooks it runs.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! SILENCED_HOOK_EVENTS.filter((event) => snapshot.boolValue(`hook.${event}.enabled`) === false);
//! ```

/// The one configuration read, and its lookup answers.
use super::config_snapshot::{ConfigSnapshot, Lookup};
/// The events whose switches are read, and the plan error a failed read becomes.
use super::hook_dispatch_plan::{HookPlanError, SILENCED_HOOK_EVENTS};
/// Config reads and the per-transaction Git context.
use super::transaction_git::{GitContext, GitRequest, run_git};
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// What:
///  The silenced events the user disabled with `hook.<event>.enabled`,
///  in the silenced list's order,
///  from one Git process.
/// Why:
///   The incumbent reads each event's switch with its own `git config --get`;
///  one `--get-regexp` over the same four keys answers all of them,
///  and `--type=bool` canonicalizes every spelling (`False`, `no`, `0`, a valueless key) per line.
///  Git prints repeated keys in precedence order with `-c` values last,
///  so the last value of a key is the one `--get` returns.
///  Exit code 1 means no event has a switch;
///  any other failure stops planning, as it does for a single read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lines = (await git(['config', '--type=bool', '--get-regexp', pattern])).split('\n');
/// SILENCED_HOOK_EVENTS.filter((event) => lastValue(lines, `hook.${event}.enabled`) === 'false');
/// ```
pub fn read_disabled_events(context: &GitContext, cwd: &Path) -> Result<Vec<String>, HookPlanError> {
    let pattern: String = format!("^hook\\.({})\\.enabled$", SILENCED_HOOK_EVENTS.join("|"));
    let arguments: Vec<String> = vec![
        String::from("config"),
        String::from("--type=bool"),
        String::from("--get-regexp"),
        pattern.clone(),
    ];
    let request: GitRequest = GitRequest::new(cwd, arguments.as_slice());
    let output = match run_git(context, &request) {
        Ok(finished) => finished,
        Err(error) => {
            return Err(HookPlanError::ConfigRead(format!(
                "git config --get-regexp {pattern} could not start: {error}"
            )));
        }
    };
    if output.code == Some(1) {
        return Ok(Vec::new());
    }
    if !output.succeeded() {
        return Err(HookPlanError::ConfigRead(format!(
            "git config --get-regexp {pattern} failed: {}",
            String::from_utf8_lossy(output.stderr.as_slice()).trim()
        )));
    }
    let Ok(text) = String::from_utf8(output.stdout) else {
        return Err(HookPlanError::ConfigRead(format!(
            "git config --get-regexp {pattern} printed bytes that are not UTF-8"
        )));
    };
    // Each line is `<key> <canonical bool>`; a later line replaces an earlier one for the same key.
    let mut switches: Vec<(&str, &str)> = Vec::new();
    for line in text.lines() {
        if let Some((key, value)) = line.rsplit_once(' ') {
            switches.retain(|(known, _)| return *known != key);
            switches.push((key, value));
        }
    }
    let mut disabled: Vec<String> = Vec::new();
    for event in SILENCED_HOOK_EVENTS {
        let key: String = format!("hook.{event}.enabled");
        let enabled: Option<&str> = switches
            .iter()
            .find(|(known, _)| return *known == key.as_str())
            .map(|(_, value)| return *value);
        if enabled == Some("false") {
            disabled.push(String::from(event));
        }
    }
    return Ok(disabled);
}

/// What:
///  The silenced events the user disabled with `hook.<event>.enabled`, from the configuration snapshot.
/// Why:
///   The snapshot answers exactly what the `--get-regexp` read answers when every entry of the four switches is a
///  boolean it models; one it does not model (a bad boolean, a unit suffix) is `None`,
///  and the caller starts Git, which reports the same error it always did.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// SILENCED_HOOK_EVENTS.filter((event) => snapshot.boolValue(`hook.${event}.enabled`) === false);
/// ```
pub fn snapshot_disabled_events(snapshot: &ConfigSnapshot) -> Option<Vec<String>> {
    let mut disabled: Vec<String> = Vec::new();
    for event in SILENCED_HOOK_EVENTS {
        match snapshot.bool_value(format!("hook.{event}.enabled").as_str()) {
            Lookup::Unset | Lookup::Found(true) => {}
            Lookup::Found(false) => disabled.push(String::from(event)),
            Lookup::Unhandled => return None,
        }
    }
    return Some(disabled);
}

/// The switches' spellings and the snapshot's answers are controlled in the test module.
#[cfg(test)]
#[path = "hook_event_switches_tests.rs"]
mod tests;
