//! What:
//!  The non-blocking warning a wrapped commit prints when the configuration sets `core.hooksPath`.
//! Why:
//!  Traditional hooks (executable scripts in `.git/hooks` or in a `core.hooksPath` directory) are unsupported, and
//!  their behavior with the wrapper is undefined.
//!  The commit path may not look for them: a directory scan would cost file system calls on every commit.
//!  It already holds one free signal, the configuration snapshot it read for other purposes, so a configured
//!  `core.hooksPath` is the only trigger, decided from memory alone.
//!  Finding the scripts themselves is the job of `git cli-git doctor`.
//!  The line is a `configuration-warning` event, which never blocks a command.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const hooksPath = snapshot?.lastValue('core.hookspath');
//! if (hooksPath !== undefined && !isWrapperOwned(hooksPath)) stderr.write(renderConfigurationWarning(0, code, message, hooksPath));
//! ```

/// Debug logging of a stream that refused the warning.
use super::diagnostic_log::debug;
/// The warning's JSONL rendering, shared with the other configuration warnings.
use super::diagnostics::render_configuration_warning;
/// The configuration the commit read once.
use super::config_snapshot::ConfigSnapshot;
/// The name of the absent hooks directory the wrapper's own overrides use.
use super::shadow_git::NO_HOOKS_DIRECTORY;
/// How the wrapper names its transaction directories.
use super::transaction_registry::{TRANSACTION_ROOT_NAME, is_transaction_id};
/// A stream the warning is written to.
use std::io::Write;
/// `OsStr` is operating-system text; `Path` is a borrowed filesystem path.
use std::ffi::OsStr;
use std::path::Path;

/// Code of the configuration warning for a configured `core.hooksPath`.
pub const TRADITIONAL_HOOKS_UNSUPPORTED_CODE: &str = "traditional-hooks-unsupported";

/// Canonical (lowercase) snapshot key of the setting that triggers the warning.
const HOOKS_PATH_KEY: &str = "core.hookspath";

/// Name of the dispatcher directory inside a transaction directory.
const DISPATCHER_DIRECTORY_NAME: &str = "hooks";

/// Module tag for diagnostics.
const TAG: &str = "traditionalHookWarning";

/// What:
///  Whether a `core.hooksPath` value is one of the wrapper's own per-transaction overrides.
/// Why:
///   The wrapper passes `core.hooksPath=<tx>/hooks` and `core.hooksPath=<tx>/no-hooks` to the Git processes it
///  starts, and those reach hooks the user configured as `-c` values in `GIT_CONFIG_PARAMETERS`.
///  A command such as `git commit` run by such a hook must not warn about a path the user never chose.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// /\/cli-git-transactions\/[0-9a-f-]{36}\/(hooks|no-hooks)$/.test(value)
/// ```
pub fn is_wrapper_hooks_path(value: &str) -> bool {
    let path: &Path = Path::new(value);
    let Some(name) = path.file_name().and_then(OsStr::to_str) else {
        return false;
    };
    if name != NO_HOOKS_DIRECTORY && name != DISPATCHER_DIRECTORY_NAME {
        return false;
    }
    let Some(transaction) = path.parent() else {
        return false;
    };
    let Some(identifier) = transaction.file_name().and_then(OsStr::to_str) else {
        return false;
    };
    if !is_transaction_id(identifier) {
        return false;
    }
    return transaction
        .parent()
        .and_then(Path::file_name)
        .is_some_and(|root| { return root == TRANSACTION_ROOT_NAME; });
}

/// What:
///  The `core.hooksPath` value the user configured, if the snapshot holds one.
/// Why:
///   The last entry of the key is the value Git uses; a valueless or empty entry names no directory, and the
///  wrapper's own overrides are not the user's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// snapshot.entries.findLast((entry) => entry.key === 'core.hookspath')?.value;
/// ```
pub fn configured_hooks_path(snapshot: &ConfigSnapshot) -> Option<String> {
    let entry = snapshot
        .entries()
        .iter()
        .rev()
        .find(|candidate| { return candidate.key == HOOKS_PATH_KEY; })?;
    let bytes: &[u8] = entry.value.as_deref()?;
    if bytes.is_empty() {
        return None;
    }
    let text: String = String::from_utf8_lossy(bytes).into_owned();
    if is_wrapper_hooks_path(text.as_str()) {
        return None;
    }
    return Some(text);
}

/// What:
///  The warning line for a configured `core.hooksPath`, or nothing when the snapshot sets none.
/// Why:
///   The answer comes from the snapshot alone, so a repository without the setting pays no process start and no
///  file system call; the directory is not looked at, because `git cli-git doctor` does that on request.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// hooksPath === undefined ? undefined : renderConfigurationWarning(0, code, message, hooksPath);
/// ```
pub fn traditional_hooks_warning(snapshot: Option<&ConfigSnapshot>) -> Option<String> {
    let value: String = configured_hooks_path(snapshot?)?;
    let message: String = format!(
        "`core.hooksPath` is set to {value}, so this repository may use traditional Git hooks (executable scripts in \
         a hooks directory). cli-git does not support traditional hooks and their behavior with it is undefined; \
         only hooks configured with `hook.<name>.event` and `hook.<name>.command` are supported. \
         Run `git cli-git doctor` to list the traditional hooks it can find."
    );
    return Some(render_configuration_warning(
        0,
        TRADITIONAL_HOOKS_UNSUPPORTED_CODE,
        message.as_str(),
        value.as_str(),
    ));
}

/// What:
///  Write the warning, when there is one, to `stream`.
/// Why:
///   The command's exit status already carries its result, so a stream that refuses the line is logged and the
///  command goes on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = traditionalHooksWarning(snapshot); if (line !== undefined) stream.write(line);
/// ```
pub fn warn_traditional_hooks(snapshot: Option<&ConfigSnapshot>, stream: &mut impl Write) {
    let Some(line) = traditional_hooks_warning(snapshot) else {
        return;
    };
    if let Err(error) = stream.write_all(line.as_bytes()).and_then(|()| { return stream.flush(); }) {
        debug(TAG, format!("the warning could not be written: {error}").as_str());
    }
}

/// Warning controls stay out of the release executable.
#[cfg(test)]
#[path = "traditional_hook_warning_tests.rs"]
mod tests;
