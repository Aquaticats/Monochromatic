//! What:
//!  Git's own trace of the LFS upload hook, read by the parent that started the push to learn whether the upload hook
//!  is what stopped it.
//! Why:
//!  A failing `pre-push` hook ends the push with status 1 and `error: failed to push some refs`, the same as a remote
//!  refusal, and that line is translated into the caller's language, so neither status nor text identifies the hook.
//!  Git's trace2 event stream does: with `GIT_TRACE2_EVENT` naming a private file, Git writes one `child_start` event
//!  (`child_class` `hook`, `hook_name` `pre-push`, the command string) and one `child_exit` event with the status for
//!  each hook it runs, in every locale. The file is written by Git alone, so output from a remote or from another
//!  hook cannot forge it; events of Git processes the hook starts carry a longer session ID and are ignored.
//!  The parent never starts itself or a shell for this: it sets one environment variable and reads one file.
//!  A caller who set `GIT_TRACE2_EVENT` keeps their target, and then no explanation is added.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const events = readFileSync(tracePath, 'utf8').split('\n').map(JSON.parse);
//! const failed = events.find((event) => event.event === 'child_exit' && hookIds.has(event.child_id) && event.code !== 0);
//! ```

/// The command string and the explanation texts of the upload hook.
use super::lfs_pre_push::{HOOK_COMMAND, failed_message, status_description};
/// The environment a Git child receives, and the private scratch directory for the trace file.
use super::child_environment::environment_value;
use super::diagnostic_log::debug;
use super::scratch_location::prepare_user_scratch;
/// Owned OS text for environment entries.
use std::ffi::OsString;
/// Bounded reading of the trace.
use std::io::Read;

/// Tag used for diagnostics from this module.
const TAG: &str = "cli-git";

/// The variable that names Git's trace2 event target.
const TRACE_VARIABLE: &str = "GIT_TRACE2_EVENT";

/// The most trace bytes read; a longer trace is read up to here and the rest is not looked at.
const MAX_TRACE_BYTES: u64 = 16 * 1024 * 1024;

/// What:
///  A private trace file for one push and the knowledge of what it recorded.
/// Why:
///   The file is removed when this value drops, and a file left by a killed process is swept with the rest of the
///  per-user scratch directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HookTrace = { path: string };
/// ```
pub struct HookTrace {
    /// The trace file, removed on drop.
    file: tempfile::NamedTempFile,
}

impl HookTrace {
    /// What:
    ///  Make the trace file, unless the caller already chose a trace2 event target or no private directory exists.
    /// Why:
    ///   An explanation is a courtesy; it must never replace the caller's own trace or fail the push.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// env.GIT_TRACE2_EVENT === undefined ? new HookTrace(await makePrivateFile()) : undefined
    /// ```
    pub fn start(environment: &[(OsString, OsString)]) -> Option<HookTrace> {
        if environment_value(environment, TRACE_VARIABLE).is_some() {
            debug(TAG, "GIT_TRACE2_EVENT is set by the caller; a failed LFS upload is not explained from it");
            return None;
        }
        let directory = match prepare_user_scratch(environment) {
            Ok(found) => found,
            Err(error) => {
                debug(TAG, format!("no private directory for the hook trace: {error}").as_str());
                return None;
            }
        };
        return match tempfile::NamedTempFile::new_in(directory) {
            Ok(file) => Some(HookTrace { file }),
            Err(error) => {
                debug(TAG, format!("the hook trace file could not be made: {error}").as_str());
                None
            }
        };
    }

    /// The environment entry that makes Git write its events to the file.
    pub fn entry(&self) -> (OsString, OsString) {
        return (OsString::from(TRACE_VARIABLE), self.file.path().as_os_str().to_owned());
    }

    /// What:
    ///  The explanation to print after a push that ended with a non-zero status, or nothing when the upload hook did
    ///  not fail.
    /// Why:
    ///   Only a hook that Git reports as failed is blamed; a remote refusal, a rule or a stop by deadline stays as
    ///  Git and the other parts report it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// failedStatus ? failedMessage(statusDescription(failedStatus)) : ''
    /// ```
    pub fn explanation(&self) -> String {
        let mut bytes: Vec<u8> = Vec::new();
        let read = std::fs::File::open(self.file.path()).and_then(|file| {
            return file.take(MAX_TRACE_BYTES).read_to_end(&mut bytes);
        });
        if let Err(error) = read {
            debug(TAG, format!("the hook trace could not be read: {error}").as_str());
            return String::new();
        }
        return match failed_hook_status(String::from_utf8_lossy(bytes.as_slice()).as_ref()) {
            Some(code) => failed_message(status_description(code).as_str()),
            None => String::new(),
        };
    }
}

/// What:
///  The status of the first upload hook that failed in a trace2 event stream.
/// Why:
///   Only events of the session that wrote the first line count, so the Git processes the hook starts (for example
///  `git-lfs` running `git`) cannot add a hook of their own; the command string must be the upload hook's, so another
///  `pre-push` hook of the caller is not blamed on LFS.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function failedHookStatus(trace: string): number | undefined;
/// ```
pub fn failed_hook_status(trace: &str) -> Option<i64> {
    let mut session: Option<String> = None;
    let mut hooks: Vec<i64> = Vec::new();
    for line in trace.lines() {
        let Ok(event) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        let Some(sid) = event.get("sid").and_then(serde_json::Value::as_str) else {
            continue;
        };
        let top: &str = session.get_or_insert_with(|| {
            return sid.to_owned();
        });
        if top != sid {
            continue;
        }
        let child_id: Option<i64> = event.get("child_id").and_then(serde_json::Value::as_i64);
        match (event.get("event").and_then(serde_json::Value::as_str), child_id) {
            (Some("child_start"), Some(id))
                if event.get("child_class").and_then(serde_json::Value::as_str) == Some("hook")
                    && event.get("hook_name").and_then(serde_json::Value::as_str) == Some("pre-push")
                    && event
                        .get("argv")
                        .and_then(|argv| {
                            return argv.get(0);
                        })
                        .and_then(serde_json::Value::as_str)
                        == Some(HOOK_COMMAND) =>
            {
                hooks.push(id);
            }
            (Some("child_exit"), Some(id)) if hooks.contains(&id) => {
                let code: i64 = event.get("code").and_then(serde_json::Value::as_i64).unwrap_or(0);
                if code != 0 {
                    return Some(code);
                }
            }
            _ => {}
        }
    }
    return None;
}

/// Trace reading is controlled in the test module.
#[cfg(test)]
#[path = "lfs_hook_trace_tests.rs"]
mod tests;
