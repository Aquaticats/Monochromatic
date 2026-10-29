//! What:
//!  The hooks directory of one commit transaction:
//!  its `plan.json` and one entry per preparation event.
//! Why:
//!  Git starts a hook through `core.hooksPath` by the entry's path, so the entry's name is the event
//!  and the executable it starts is the native wrapper.
//!  On Windows Git looks for `<event>.exe`, so entries are hard links or copies with that name.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await writeHookShim({ hooksDirectory, plan, nodePath: process.execPath });
//! ```

/// Debug logging of an omitted entry.
use super::diagnostic_log::debug;
/// Whether a user hook may exist for an event.
use super::hook_presence::event_may_have_user_hook;
/// Plan spelling shared with the reader.
use super::hook_dispatch_plan::{
    HookDispatchPlan, PLAN_FILENAME, PREPARATION_HOOK_EVENTS, encode_hook_dispatch_plan,
};
/// Private directory creation and exclusive private file writes.
use super::private_storage::{create_private_directory, write_private_file};
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};

/// What:
///  Why writing a hooks directory failed, naming the path.
/// Why:
///   The caller stops the transaction; a partial hooks directory is never reused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// throw new Error(`cannot write ${path}`);
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HookEntryError(pub String);

/// What:
///  One sentence naming the hook entry that failed.
/// Why:
///   The caller prints it as the diagnostic.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// error.message
/// ```
impl std::fmt::Display for HookEntryError {
    /// Writes the stored sentence.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(formatter, "{}", self.0);
    }
}

/// What:
///  The file name of one preparation event's entry.
/// Why:
///   Unix names the entry after the event; Windows adds `.exe`, which Git appends when it looks up a hook.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// process.platform === 'win32' ? `${event}.exe` : event;
/// ```
pub fn hook_entry_name(event: &str) -> String {
    if cfg!(windows) {
        return format!("{event}.exe");
    }
    return String::from(event);
}

/// What:
///  Write one entry that starts `executable` under its event's name.
/// Why:
///   Unix symbolic links keep the start name as `argv[0]`;
///  Windows entries are hard links, or copies where hard links are refused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeFile(join(hooksDirectory, event), hookEntryProgram({ nodePath, dispatchUrl, event }));
/// ```
fn write_entry(executable: &Path, entry: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(executable, entry)?;
    }
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(windows)]
    {
        if std::fs::hard_link(executable, entry).is_err() {
            std::fs::copy(executable, entry)?;
        }
    }
    return Ok(());
}

/// What:
///  Create a new hooks directory holding the plan and one entry per preparation event.
/// Why:
///   The incumbent's `writeHookShim`, with native entries instead of a Node program:
///  `post-commit` has no entry, so it never runs during preparation.
///  An event with no user hook has no entry either: Git then finds nothing to start for it, which is what the
///  entry would have done after re-entering this executable, taking the hook lock and running `git hook run`.
///  The directory must not exist, so nothing half-written is reused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeHookShim({ hooksDirectory, plan, nodePath });
/// ```
pub fn write_hook_dispatch(
    hooks_directory: &Path,
    plan: &HookDispatchPlan,
    executable: &Path,
) -> Result<(), HookEntryError> {
    create_private_directory(hooks_directory).map_err(|error| {
        return HookEntryError(format!(
            "cannot create the hook directory {}: {error}",
            hooks_directory.display()
        ))
    })?;
    let plan_path: PathBuf = hooks_directory.join(PLAN_FILENAME);
    let plan_bytes: String = format!("{}\n", encode_hook_dispatch_plan(plan));
    write_private_file(plan_path.as_path(), plan_bytes.as_bytes()).map_err(|error| {
        return HookEntryError(format!(
            "cannot write the hook plan {}: {error}",
            plan_path.display()
        ))
    })?;
    for event in PREPARATION_HOOK_EVENTS {
        if !event_may_have_user_hook(plan, event) {
            debug("writeHookDispatch", format!("no user hook for {event}; the entry is omitted").as_str());
            continue;
        }
        let entry: PathBuf = hooks_directory.join(hook_entry_name(event));
        write_entry(executable, entry.as_path()).map_err(|error| {
            return HookEntryError(format!(
                "cannot write the hook entry {}: {error}",
                entry.display()
            ))
        })?;
    }
    return Ok(());
}

#[cfg(test)]
#[path = "hook_dispatch_entries_tests.rs"]
mod tests;
