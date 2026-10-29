//! What:
//!  Whether a commit event can have a user hook at all, so an event without one needs no dispatcher entry and no
//!  `post-commit` run.
//! Why:
//!  A dispatcher entry re-enters this executable, takes the hook lock and starts `git hook run`, and `post-commit`
//!  reads the author, takes the lock and starts `git hook run`; with no hook for the event each of those does
//!  nothing visible.
//!  The answer is conservative: it says "no hook" only when the resolved hooks directory has no entry of the
//!  event's name at all (a non-executable file or a dangling link counts, because Git prints a hint for a file it
//!  skips) and the configuration defines no hook by name.
//!  Any other shape, including an error other than "not found", says a hook may exist and keeps today's behavior.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const mayHaveHook = (plan, event) => plan.namedHooksPossible || entryMayExist(plan.hooksPath, event);
//! ```

/// Debug logging of the decision.
use super::diagnostic_log::debug;
/// The plan records the hooks directory and whether the configuration names hooks.
use super::hook_dispatch_plan::HookDispatchPlan;
/// `ErrorKind` tells "not found" from every other failure to look.
use std::io::ErrorKind;
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// What:
///  Whether `hooks_directory` may hold an entry named `event`, of any file type or mode.
/// Why:
///   Git looks the hook up by name, runs it when it is executable, and prints a hint when it exists but is not;
///  only a lookup that fails with "not found" proves nothing is there, and the other errors (no permission to
///  search the directory, a hooks path that is a file) keep the entry.
///  Windows adds executable suffixes to the name, so it never proves absence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// try { lstatSync(join(hooksDirectory, event)); return true; } catch (e) { return e.code !== 'ENOENT'; }
/// ```
pub fn hook_entry_may_exist(hooks_directory: &Path, event: &str) -> bool {
    if cfg!(windows) {
        return true;
    }
    return match std::fs::symlink_metadata(hooks_directory.join(event)) {
        Ok(_) => true,
        Err(error) if error.kind() == ErrorKind::NotFound => false,
        Err(error) => {
            debug(
                "hookEntryMayExist",
                format!("looking up {event} in {} failed: {error}", hooks_directory.display()).as_str(),
            );
            true
        }
    };
}

/// What:
///  Whether a user hook may run for `event` in the transaction the plan describes.
/// Why:
///   A hook defined by name in the configuration runs for its event whatever the hooks directory holds, so a
///  configuration that defines any keeps every event; otherwise the hooks directory decides, and it is looked
///  at when the answer is needed, not when the plan was made.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// plan.namedHooksPossible || hookEntryMayExist(plan.hooksPath, event);
/// ```
pub fn event_may_have_user_hook(plan: &HookDispatchPlan, event: &str) -> bool {
    return plan.named_hooks_possible || hook_entry_may_exist(Path::new(plan.hooks_path.as_str()), event);
}

/// Hook presence controls beside the decision.
#[cfg(test)]
#[path = "hook_presence_tests.rs"]
mod tests;
