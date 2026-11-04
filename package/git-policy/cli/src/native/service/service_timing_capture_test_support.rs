//! What:
//!  Test-only capture of the service's log entries: a control runs code and gets the entries it wrote on that thread, which are written nowhere else.
//! Why:
//!  Kept in a file of its own so the shipped `service_timing.rs` holds no inline test module (`rust/tests-in-sidecars`);
//!  it is declared there under `#[cfg(test)]`, so the release executable never contains it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // a module that exists only in test builds
//! export const capture = { active, record, entriesOf };
//! ```
use super::Priority;
use std::cell::RefCell;

/// One entry as written: its priority, its message with the time, and its fields.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Captured {
    /// How serious the entry is.
    pub priority: Priority,
    /// The message as the journal would show it.
    pub message: String,
    /// The fields beside the message.
    pub fields: Vec<(String, String)>,
}

thread_local! {
    static ENTRIES: RefCell<Option<Vec<Captured>>> = const { RefCell::new(None) };
}

/// Whether a control is capturing the entries of this thread.
pub fn active() -> bool {
    return ENTRIES.with(|entries| { return entries.borrow().is_some(); });
}

/// Notes an entry when a control is capturing on this thread; says whether it did.
pub fn record(priority: Priority, message: &str, fields: &[(&str, String)]) -> bool {
    return ENTRIES.with(|entries| {
        return entries.borrow_mut().as_mut().is_some_and(|taken| {
            taken.push(Captured { priority, message: message.to_string(), fields: fields.iter().map(|(name, value)| { return ((*name).to_string(), value.clone()); }).collect() });
            return true;
        });
    });
}

/// Runs `body` and gives its entries, which are not written anywhere.
pub fn entries_of(body: impl FnOnce()) -> Vec<Captured> {
    ENTRIES.with(|entries| { *entries.borrow_mut() = Some(Vec::new()); });
    body();
    return ENTRIES.with(|entries| { return entries.borrow_mut().take().unwrap_or_default(); });
}
