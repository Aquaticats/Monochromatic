//! What:
//!  The external hooks a repository configures (`hook.<name>.event` with `hook.<name>.command`),
//! read from the
//!  configuration snapshot,
//! and the loud report that none of them runs.
//! Why:
//!  The wrapper supports no external hook program at all:
//! every real-Git start of the wrapper switches every hook
//!  event off (`hooks_off.rs`),
//! and the behaviors a hook used to provide are declared natively in
//!  `monochromatic-git-policy-cli.config.jsonc`.
//!  A configured hook that silently never ran would look like a broken check,
//! so a commit prints one
//!  `configuration-warning` event naming the keys and the events,
//! decided from the snapshot the commit already holds
//!  (no process start),
//! and `git monochromatic-git-policy-cli doctor` lists the same hooks.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const hooks = configuredHooks(snapshot); if (hooks.length > 0) stderr.write(renderConfigurationWarning(0, code, message, names));
//! ```

/// Debug logging of a stream that refused the warning.
use crate::diagnostic_log::debug;
/// The warning's JSONL rendering,
/// shared with the other configuration warnings.
use crate::diagnostics::render_configuration_warning;
/// The configuration the command read once.
use crate::config_snapshot::{ConfigSnapshot, Lookup};
/// The longest this optional line may take to reach the stream it is written to.
use crate::wait_ceilings::WARNING_WRITE_CEILING;
/// The job run on a thread of its own,
/// awaited no longer than that ceiling.
use crate::work_awaited_within_a_ceiling::work_within;
/// A stream the warning is written to.
use std::io::Write;

/// Code of the configuration warning for configured external hooks.
pub const EXTERNAL_HOOKS_UNSUPPORTED_CODE: &str = "external-hooks-unsupported";

/// Module tag for diagnostics.
const TAG: &str = "externalHooks";

/// The key parts around a hook name:
/// `hook.<name>.event` and `hook.<name>.command`.
const HOOK_PREFIX: &str = "hook.";

/// What:
///  One configured hook.
/// Why:
///   The report names the hook,
/// its events and whether it has a command,
/// so the author knows which keys to remove.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ConfiguredHook = { name: string; events: string[]; hasCommand: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfiguredHook {
    /// The hook's friendly name (the subsection of the keys).
    pub name: String,
    /// The events it asks for,
    /// in the order Git reads them;
    /// an empty value resets the list,
    /// as Git does.
    pub events: Vec<String>,
    /// Whether `hook.<name>.command` is set.
    pub has_command: bool,
}

/// What:
///  The text of a value as UTF-8,
/// lossily.
/// Why:
///   Event names and hook names are ASCII in every real configuration,
/// and the report only names them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// new TextDecoder().decode(bytes)
/// ```
fn lossy(value: Option<&Vec<u8>>) -> String {
    return match value {
        Some(bytes) => String::from_utf8_lossy(bytes.as_slice()).into_owned(),
        None => String::new(),
    };
}

/// What:
///  Whether the configuration the person wrote switches every hook of `event` off.
/// Why:
///   `hook.<event>.enabled=false` is Git's own switch for the whole event
/// (`git hook list` reports such a hook
///  as `event-disabled`),
/// so a hook whose events are all off asks for nothing.
///   Only the configuration files are read:
/// every real-Git start of the wrapper carries the same switch for every
///  event,
/// and a child of one inherits those values in the `command` scope,
/// so reading them here would let the
///  wrapper's own suppression hide a hook the person configured.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// snapshot.boolValueFromFiles(`hook.${event}.enabled`) === false
/// ```
fn event_is_off(snapshot: &ConfigSnapshot, event: &str) -> bool {
    return matches!(snapshot.bool_value_from_files(format!("hook.{event}.enabled").as_str()), Lookup::Found(false));
}

/// What:
///  Whether the configuration switches this hook off at the event level,
/// when it asks for at least one event.
/// Why:
///   A hook with no event is not this switch's business:
/// it asks for nothing whatever the events are set to,
/// and the
///  report already says so by naming it without an event.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// hook.events.length > 0 && hook.events.every((event) => eventIsOff(snapshot, event))
/// ```
fn every_event_is_off(snapshot: &ConfigSnapshot, hook: &ConfiguredHook) -> bool {
    if hook.events.is_empty() {
        return false;
    }
    for event in &hook.events {
        if !event_is_off(snapshot, event) {
            return false;
        }
    }
    return true;
}

/// What:
///  Every configured hook that is not switched off,
/// by its own `hook.<name>.enabled` or by every event it asks for being off,
/// in the order the snapshot lists
///  them.
/// Why:
///   The hooks Git would have run:
/// a name with an event or a command,
/// minus the ones the author already disabled.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// configuredHooks(snapshot); // ConfiguredHook[]
/// ```
pub fn configured_hooks(snapshot: &ConfigSnapshot) -> Vec<ConfiguredHook> {
    let mut hooks: Vec<ConfiguredHook> = Vec::new();
    for entry in snapshot.entries() {
        let Some(rest) = entry.key.strip_prefix(HOOK_PREFIX) else {
            continue;
        };
        let (name, event_key): (&str, bool) = if let Some(name) = rest.strip_suffix(".event") {
            (name, true)
        } else if let Some(name) = rest.strip_suffix(".command") {
            (name, false)
        } else {
            continue;
        };
        if name.is_empty() {
            continue;
        }
        let mut known_position: Option<usize> = None;
        for (index, known) in hooks.iter().enumerate() {
            if known.name == name {
                known_position = Some(index);
                break;
            }
        }
        let position: usize = match known_position {
            Some(found) => found,
            None => {
                hooks.push(ConfiguredHook { name: String::from(name), events: Vec::new(), has_command: false });
                hooks.len() - 1
            }
        };
        if event_key {
            let value: String = lossy(entry.value.as_ref());
            if value.is_empty() {
                hooks[position].events.clear();
            } else {
                hooks[position].events.push(value);
            }
        } else {
            hooks[position].has_command = true;
        }
    }
    // A hook the author switched off by its own `hook.<name>.enabled`, and one whose every event is switched off, are
    //  dropped; the others keep their order. The two switches are read differently on purpose: the wrapper supplies
    //  `hook.<event>.enabled=false` for every event on every Git it starts, so the event switch is read from the
    //  files, while it writes no `hook.<name>.enabled` at all: that switch is read the way Git reads it, where the
    //  command scope comes last and wins and a caller's `-c` is their own choice.
    let mut enabled: Vec<ConfiguredHook> = Vec::new();
    for hook in hooks {
        if matches!(snapshot.bool_value(format!("hook.{}.enabled", hook.name).as_str()), Lookup::Found(false)) {
            continue;
        }
        if every_event_is_off(snapshot, &hook) {
            continue;
        }
        enabled.push(hook);
    }
    return enabled;
}

/// What:
///  The hook as one phrase:
/// `hook.<name> (events: a, b)`.
/// Why:
///   The report lists each hook by the key prefix the author would unset.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `hook.${name} (events: ${events.join(', ')})`
/// ```
pub fn describe_hook(hook: &ConfiguredHook) -> String {
    if hook.events.is_empty() {
        return format!("hook.{} (no event)", hook.name);
    }
    return format!("hook.{} (events: {})", hook.name, hook.events.join(", "));
}

/// What:
///  The sentence that explains what to do about configured hooks.
/// Why:
///   The explanation is shared by the commit warning and `git monochromatic-git-policy-cli doctor`,
/// so both say the same ways forward.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// explainExternalHooks(hooks);
/// ```
pub fn explain_external_hooks(hooks: &[ConfiguredHook]) -> String {
    let listed: Vec<String> = hooks.iter().map(describe_hook).collect();
    return format!(
        "This repository configures external hooks: {}. monochromatic-git-policy-cli supports no external hook program, so every Git it \
         starts has every hook event switched off and these never run. Ways forward: remove them with \
         `git config --unset-all hook.<name>.event` and `git config --unset-all hook.<name>.command` for each name \
         (the keys may live in a global or system file; `git config --show-origin --get-regexp '^hook[.]'` shows \
         where), or express what the hook checked as a monochromatic-git-policy-cli policy under policies in monochromatic-git-policy-cli.config.jsonc. monochromatic-git-policy-cli \
         runs no program named by any configuration, so no setting can bring a hook back. `git monochromatic-git-policy-cli doctor` lists \
         these hooks.",
        listed.join("; ")
    );
}

/// What:
///  The warning line for configured external hooks,
/// or nothing when the snapshot defines none.
/// Why:
///   The answer comes from the snapshot alone,
/// so a repository without hooks pays no process start and no file
///  system call.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// hooks.length === 0 ? undefined : renderConfigurationWarning(0, code, explain(hooks), names);
/// ```
pub fn external_hooks_warning(snapshot: Option<&ConfigSnapshot>) -> Option<String> {
    let hooks: Vec<ConfiguredHook> = configured_hooks(snapshot?);
    if hooks.is_empty() {
        return None;
    }
    let mut names: Vec<String> = Vec::new();
    for hook in &hooks {
        names.push(format!("hook.{}", hook.name));
    }
    return Some(render_configuration_warning(
        0,
        EXTERNAL_HOOKS_UNSUPPORTED_CODE,
        explain_external_hooks(hooks.as_slice()).as_str(),
        names.join(",").as_str(),
    ));
}

/// What:
///  Write the warning,
/// when there is one,
/// to `stream`.
/// Why:
///   The command's exit status already carries its result,
/// so a stream that refuses the line is logged and the
///  command goes on.
///  A stream whose reader has stopped draining waits in the kernel with no end,
/// and this line sits on
///  the commit's preparation path,
/// so the write runs on a thread of its own and the command waits for it no longer
///  than `WARNING_WRITE_CEILING`.
/// A line still waiting then is left behind and logged;
/// the commit goes on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = externalHooksWarning(snapshot); if (line !== undefined) writeWithin(WARNING_WRITE_CEILING, stream, line);
/// ```
pub fn warn_external_hooks(snapshot: Option<&ConfigSnapshot>, stream: impl Write + Send + 'static) {
    let Some(line) = external_hooks_warning(snapshot) else {
        return;
    };
    let delivered: Option<std::io::Result<()>> = work_within(WARNING_WRITE_CEILING, move || {
        let mut target = stream;
        // The stream is flushed only after the line was written, as `.and_then(...)` ran it.
        return match target.write_all(line.as_bytes()) {
            Ok(()) => target.flush(),
            Err(error) => Err(error),
        };
    });
    match delivered {
        Some(Err(error)) => debug(TAG, format!("the warning could not be written: {error}").as_str()),
        Some(Ok(())) => {}
        None => debug(TAG, format!("the warning was not written within {WARNING_WRITE_CEILING:?}").as_str()),
    }
}

/// Reading hooks from the snapshot and the warning are controlled in the test module.
#[cfg(test)]
#[path = "external_hooks_tests.rs"]
mod tests;
