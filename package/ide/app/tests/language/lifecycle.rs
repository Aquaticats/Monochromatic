//! Server lifetime:
//!  crash and restart,
//!  file switches,
//!  files outside the project,
//!  launch refusal,
//! and shutdown without leftover processes.

/// The report the scripted server writes,
///  which names the process of every start.
use serde_json::Value;

/// What:
///  `#[path = "..."]` names the file a module is read from;
///  this module's parts live under `tests/language/lifecycle/`.
/// Why:
///  The test crate reaches this file by its own path,
///       so a submodule declared here is looked for beside it,
///  and one file holds one role.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export * from './lifecycle/crash_and_restart';
/// ```
/// A crash fails the pending request and the next open starts a new one.
#[path = "lifecycle/crash_and_restart.rs"]
mod crash_and_restart;
/// Another file goes to the same server;
/// a file outside the project starts nothing.
#[path = "lifecycle/file_switching.rs"]
mod file_switching;
/// A refused or unpreparable launch is reported and nothing is spawned.
#[path = "lifecycle/launch_refusal.rs"]
mod launch_refusal;
/// Dropping the handle leaves no child process behind.
#[path = "lifecycle/shutdown.rs"]
mod shutdown;

/// Process identifiers of every scripted server that started,
///  in order.
fn started(lines: &[Value]) -> Vec<u64> {
    let mut pids = Vec::new();
    for line in lines {
        if let Some(pid) = line["started"]["pid"].as_u64() {
            pids.push(pid);
        }
    }
    return pids;
}
