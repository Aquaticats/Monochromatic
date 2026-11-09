//! What:
//!  The non-blocking warning a wrapped commit prints when the configuration sets `core.hooksPath`.
//! Why:
//!  Traditional hooks (executable scripts in `.git/hooks` or in a `core.hooksPath` directory) are unsupported,
//! and
//!  their behavior with the wrapper is undefined.
//!  The commit path may not look for them:
//! a directory scan would cost file system calls on every commit.
//!  It already holds one free signal,
//! the configuration snapshot it read for other purposes,
//! so a configured
//!  `core.hooksPath` is the only trigger,
//! decided from memory alone.
//!  Finding the scripts themselves is the job of `git monochromatic-git-policy-cli doctor`.
//!  The line is a `configuration-warning` event,
//! which never blocks a command.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const hooksPath = snapshot?.lastValue('core.hookspath');
//! if (hooksPath !== undefined && !isWrapperOwned(hooksPath)) stderr.write(renderConfigurationWarning(0, code, message, hooksPath));
//! ```

/// Debug logging of a stream that refused the warning.
use crate::diagnostic_log::debug;
/// The warning's JSONL rendering,
/// shared with the other configuration warnings.
use crate::diagnostics::render_configuration_warning;
/// The configuration the commit read once.
use crate::config_snapshot::ConfigSnapshot;
/// The name of the absent hooks directory the wrapper's own overrides use.
use crate::hooks_off::NO_HOOKS_DIRECTORY;
/// The longest this optional line may take to reach the stream it is written to.
use crate::wait_ceilings::WARNING_WRITE_CEILING;
/// The job run on a thread of its own,
/// awaited no longer than that ceiling.
use crate::work_awaited_within_a_ceiling::work_within;
use crate::shadow_git::{SHADOW_NO_HOOKS_NAME, SHADOW_ROOT_NAME};
/// The spellings of the folder the wrapper keeps inside a Git directory.
use crate::wrapper_names_old_and_new::DATA_FOLDER;
/// How the wrapper names its transaction directories.
use crate::random_id::is_canonical_uuid;
/// A stream the warning is written to.
use std::io::Write;
/// `OsStr` is operating-system text;
/// `Path` is a borrowed filesystem path.
use std::ffi::OsStr;
use std::path::Path;

/// Code of the configuration warning for a configured `core.hooksPath`.
pub const TRADITIONAL_HOOKS_UNSUPPORTED_CODE: &str = "traditional-hooks-unsupported";

/// Canonical (lowercase) snapshot key of the setting that triggers the warning.
const HOOKS_PATH_KEY: &str = "core.hookspath";

/// The scope of a value that reaches Git as a `-c` option or through `GIT_CONFIG_PARAMETERS`,
/// which is
///  the only place the wrapper's own overrides come from.
const COMMAND_SCOPE: &str = "command";

/// Module tag for diagnostics.
const TAG: &str = "traditionalHookWarning";

/// What:
///  Whether a `core.hooksPath` value is one of the wrapper's own hooks-off values.
/// Why:
///   Every real Git the wrapper starts carries `core.hooksPath=/dev/null/cli-git-no-hooks`,
/// and its shadow commands
///  carry `<git-common-dir>/monochromatic-git-policy-cli/shadow/<transaction>/no-hooks`;
/// a Git child that reads `GIT_CONFIG_PARAMETERS`
///  sees them as `-c` values.
///  A command such as `git commit` run by such a child must not warn about a path the user never chose.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// value === NO_HOOKS_DIRECTORY || /\/monochromatic-git-policy-cli\/shadow\/[0-9a-f-]{36}\/no-hooks$/.test(value)
/// ```
pub fn is_wrapper_hooks_path(scope: &str, value: &str) -> bool {
    // A value a configuration file holds is somebody's choice, whatever it spells; the wrapper writes no configuration,
    // so only a value that reached Git as a `-c` option or through the environment is its own.
    if scope != COMMAND_SCOPE {
        return false;
    }
    if value == NO_HOOKS_DIRECTORY {
        return true;
    }
    let path: &Path = Path::new(value);
    let Some(name) = path.file_name().and_then(OsStr::to_str) else {
        return false;
    };
    if name != SHADOW_NO_HOOKS_NAME {
        return false;
    }
    let Some(shadow) = path.parent() else {
        return false;
    };
    let Some(identifier) = shadow.file_name().and_then(OsStr::to_str) else {
        return false;
    };
    if !is_canonical_uuid(identifier) {
        return false;
    }
    let Some(root) = shadow.parent() else {
        return false;
    };
    if root.file_name() != Some(OsStr::new(SHADOW_ROOT_NAME)) {
        return false;
    }
    // The folder above `shadow` is the wrapper's own data folder inside the Git directory, whose name this build knows.
    // A path that merely ends in `shadow/<transaction>/no-hooks` belongs to whoever made it.
    return match root.parent().and_then(Path::file_name) {
        Some(folder) => folder == OsStr::new(DATA_FOLDER.current) || folder == OsStr::new(DATA_FOLDER.earlier),
        None => false,
    };
}

/// What:
///  The `core.hooksPath` value the user configured,
/// if the snapshot holds one.
/// Why:
///   The last entry of the key is the value Git uses;
/// a valueless or empty entry names no directory,
/// and the
///  wrapper's own overrides are not the user's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// snapshot.entries.findLast((entry) => entry.key === 'core.hookspath')?.value;
/// ```
pub fn configured_hooks_path(snapshot: &ConfigSnapshot) -> Option<String> {
    let mut last_entry = None;
    for candidate in snapshot.entries().iter().rev() {
        if candidate.key == HOOKS_PATH_KEY {
            last_entry = Some(candidate);
            break;
        }
    }
    let entry = last_entry?;
    let bytes: &[u8] = entry.value.as_deref()?;
    if bytes.is_empty() {
        return None;
    }
    let text: String = String::from_utf8_lossy(bytes).into_owned();
    if is_wrapper_hooks_path(entry.scope.as_str(), text.as_str()) {
        return None;
    }
    return Some(text);
}

/// What:
///  The warning line for a configured `core.hooksPath`,
/// or nothing when the snapshot sets none.
/// Why:
///   The answer comes from the snapshot alone,
/// so a repository without the setting pays no process start and no
///  file system call;
/// the directory is not looked at,
/// because `git monochromatic-git-policy-cli doctor` does that on request.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// hooksPath === undefined ? undefined : renderConfigurationWarning(0, code, message, hooksPath);
/// ```
pub fn traditional_hooks_warning(snapshot: Option<&ConfigSnapshot>) -> Option<String> {
    let value: String = configured_hooks_path(snapshot?)?;
    let message: String = format!(
        "`core.hooksPath` is set to {value}, so this repository may use traditional Git hooks (executable scripts in \
         a hooks directory). monochromatic-git-policy-cli supports no external hook program, traditional or configured: every Git it \
         starts has the hooks directory switched off, so these scripts never run. Express what a script checked as a \
         policy under policies in monochromatic-git-policy-cli.config.jsonc instead. \
         Run `git monochromatic-git-policy-cli doctor` to list the traditional hooks it can find."
    );
    return Some(render_configuration_warning(
        0,
        TRADITIONAL_HOOKS_UNSUPPORTED_CODE,
        message.as_str(),
        value.as_str(),
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
/// const line = traditionalHooksWarning(snapshot); if (line !== undefined) writeWithin(WARNING_WRITE_CEILING, stream, line);
/// ```
pub fn warn_traditional_hooks(snapshot: Option<&ConfigSnapshot>, stream: impl Write + Send + 'static) {
    let Some(line) = traditional_hooks_warning(snapshot) else {
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

/// Warning controls stay out of the release executable.
#[cfg(test)]
#[path = "traditional_hook_warning_tests.rs"]
mod tests;
