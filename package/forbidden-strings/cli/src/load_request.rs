//! What:
//!  Thread-isolated rule-load requests executed through a named unwind boundary.
//! Why:
//!  Loading may fail before a Scanner exists;
//!  no partial ruleset or panic payload may become a successful result.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Closest analogue: context.run(request, namedLoad), but this slot is per native thread and synchronous.
//! ```

/// Import the existing loader and its established error channel.
use crate::frx_load::{self, LoadedRules};
/// Imports the shipped baseline's precompiled engine bytes and rule names passed to every load.
use crate::{BUILTIN_NAMES, BUILTIN_PRECOMPILED};
/// Imports the loader's error channel and the macro building its fixed redacted messages.
use anyhow::{anyhow, Result};
/// Import a per-thread value slot.
use std::cell::Cell;
/// Imports the list entry whose owned copy a pending request keeps.
use crate::rules_list::RulesEntry;
/// Imports borrowed caller paths.
use std::path::Path;

/// Owned arguments crossing the no-capture callback boundary.
struct LoadRequest {
    /// Each entry's PathBuf owns native path bytes after the caller's borrowed values are released.
    entries: Vec<RulesEntry>,
    /// Whether the shipped baseline participates.
    builtin: bool,
    /// Whether the only entry is the implicit default file,
    ///  whose absence is not worth a log line.
    implicit: bool,
}

// What: thread_local! is the standard-library declaration for one Cell per native thread.
// Cell moves values in/out; unlike RefCell, no borrow remains active across the compiler call.
// Why: Named catch_unwind callbacks take no arguments, and a process-global request would race between concurrent callers.
//
// In TS you'd write (pseudocode):
// ```ts
// // A synchronous per-thread request slot, not one shared process variable or captured callback.
// ```
std::thread_local! {
    /// The pending request is taken before loading;
    ///  None is the normal idle state.
    static PENDING: Cell<Option<LoadRequest>> = const { Cell::new(None) };
}

/// Consume the request before invoking any compiler or logging callbacks,
///  permitting safe nested loads.
fn execute_pending() -> Result<LoadedRules> {
    // take leaves None in this thread's slot before loader work begins.
    let Some(request): Option<LoadRequest> = PENDING.take() else {
        return Err(anyhow!("Forbidden-strings rule loading had no pending request."));
    };
    return frx_load::load_listed(
        request.entries.as_slice(), request.builtin, request.implicit, BUILTIN_PRECOMPILED, BUILTIN_NAMES,
    );
}

/// Run one request without changing process-wide panic hooks or exposing panic payloads.
fn protected_request(request: LoadRequest, operation: fn() -> Result<LoadedRules>) -> Result<LoadedRules> {
    // Do not overwrite an unexpected request; restore its owner before returning a setup failure.
    if let Some(previous) = PENDING.replace(Some(request)) {
        PENDING.set(Some(previous));
        return Err(anyhow!("Forbidden-strings rule loading already has a pending request on this thread."));
    }
    // The function pointer captures nothing; the request slot is private to this native thread.
    let outcome = std::panic::catch_unwind(operation);
    // A fault before execute_pending takes the request must not contaminate the next load.
    let _unused: Option<LoadRequest> = PENDING.take();
    match outcome {
        Ok(result) => return result,
        Err(_payload) => return Err(anyhow!("Forbidden-strings rule loading panicked; no rule set was accepted.")),
    }
}

/// Load through the production compiler/cache with owned request arguments and a named callback.
pub(crate) fn load(path: &Path, builtin: bool, explicit: bool) -> Result<LoadedRules> {
    // The one-path contract is a one-entry list.
    let entry: RulesEntry = frx_load::single_entry(path, builtin, explicit);
    let request: LoadRequest = LoadRequest { entries: vec![entry], builtin, implicit: !explicit };
    return protected_request(request, execute_pending);
}

/// Load a rules list through the production compiler/cache with owned request arguments and a named callback.
pub(crate) fn load_entries(entries: &[RulesEntry], builtin: bool) -> Result<LoadedRules> {
    // Clone only the entries' ownership; rule source remains under the existing authoritative loader.
    let request: LoadRequest = LoadRequest { entries: entries.to_vec(), builtin, implicit: false };
    return protected_request(request, execute_pending);
}

/// Test-only injected callbacks never become production configuration or environment panic triggers.
#[cfg(test)]
#[path = "load_request_tests.rs"]
mod tests;

/// Nested callback and partial-construction controls remain confined to unit-test builds.
#[cfg(test)]
#[path = "load_request_nested_tests.rs"]
mod nested_tests;
