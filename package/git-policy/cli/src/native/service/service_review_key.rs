//! What:
//!  The gateway credential of the standards review as it travels from the wrapper to the service: a value that prints as
//!  redacted, is read from the wrapper's own environment, and is carried only in the in-memory `work` message over the
//!  user-only socket.
//! Why:
//!  The credential must not reach the on-disk work item, a unit file or the manager's environment, and must not appear in
//!  a log line by accident. A type whose `Debug` output hides the value keeps it out of every `{:?}` the code or a test
//!  prints; reading it takes the explicit `expose`. Nothing here depends on the network client, so the wrapper's client
//!  path may use it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const key = reviewKeyFrom(process.env); // sent in the message, never written to disk
//! ```

/// The credential variable, and the stop-hook credential used when it is unset.
pub const KEY_VARIABLES: [&str; 2] = ["SHADOW_REVIEW_LLMGATEWAY_API_KEY", "STOP_REMINDER_LLMGATEWAY_API_KEY"];

/// What:
///  A non-empty credential. `Debug` prints a placeholder; only `expose` returns the text.
/// Why:
///   An empty value is the same as no value (the script fails with "no key" on `''`), so one type states both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ReviewKey { toString() { return '<redacted>' } }
/// ```
#[derive(Clone, Eq, PartialEq)]
pub struct ReviewKey(String);

/// Methods over a key.
impl ReviewKey {
    /// The key for this text, or nothing when the text is empty.
    pub fn new(text: &str) -> Option<ReviewKey> {
        if text.is_empty() {
            return None;
        }
        return Some(ReviewKey(text.to_string()));
    }

    /// The credential text, for the one place that sends it.
    pub fn expose(&self) -> &str {
        return self.0.as_str();
    }
}

/// Debug output never carries the value.
impl std::fmt::Debug for ReviewKey {
    /// Writes a placeholder.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return formatter.write_str("ReviewKey(<redacted>)");
    }
}

/// What:
///  The credential the script would use: the first variable that is set, even to an empty text, else the second.
/// Why:
///   `??` in the script falls back only for an unset variable, so an empty first value means "no key", not the second.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// process.env.SHADOW_REVIEW_LLMGATEWAY_API_KEY ?? process.env.STOP_REMINDER_LLMGATEWAY_API_KEY ?? ''
/// ```
pub fn key_from_environment(variable: &dyn Fn(&str) -> Option<String>) -> Option<ReviewKey> {
    let chosen: Option<String> = variable(KEY_VARIABLES[0]).or_else(|| { return variable(KEY_VARIABLES[1]); });
    return chosen.and_then(|text| { return ReviewKey::new(text.as_str()); });
}

/// Key controls stay out of the release executable.
#[cfg(test)]
#[path = "service_review_key_tests.rs"]
mod tests;
