//! What:
//!  One failure or warning, and the JSON event that carries it.
//! Why:
//!  Every line file-enforcer writes is one JSON object (`schemaVersion`, `sequence`, `type`, `code`, `message`,
//!  `hint`), so every problem the engine finds is a `Diagnostic` first and an event only at the output boundary.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Diagnostic = { code: string; message: string; hint?: string; path?: string; rule?: string };
//! ```

use crate::event::Event;
use serde_json::Value;
use std::fmt;

/// What:
///  A problem found while loading, planning or publishing.
///  `String` is owned, growable text (sibling: `&str`, a borrowed read-only view);
///  `Option<String>` is "a `String` or nothing" (TS: `string | undefined`).
/// Why:
///  Plain owned fields let a diagnostic cross threads and the socket without borrowing from the run that made it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Diagnostic = { code: string; message: string; hint?: string; path?: string; rule?: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    /// Stable kebab-case handle consumers match on, for example `config-invalid`.
    pub code: String,
    /// Operation-focused explanation naming the affected input.
    pub message: String,
    /// What a person can do next, when the code alone says it.
    pub hint: Option<String>,
    /// Repo-relative path the problem is about, when there is one.
    pub path: Option<String>,
    /// Identifier of the rule that raised it, when a rule did.
    pub rule: Option<String>,
}

impl Diagnostic {
    /// What:
    ///  Build a diagnostic from a code and a message, with no hint, path or rule.
    /// Why:
    ///  Most failures start with only these two; the `with_*` methods add the rest.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function diagnostic(code: string, message: string): Diagnostic;
    /// ```
    pub fn new(code: &str, message: &str) -> Diagnostic {
        return Diagnostic {
            code: code.to_string(),
            message: message.to_string(),
            hint: None,
            path: None,
            rule: None,
        };
    }

    /// What:
    ///  Return a copy that carries the given hint.
    /// Why:
    ///  Builder style keeps construction a single expression; the original stays untouched.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// withHint(hint: string): Diagnostic;
    /// ```
    pub fn with_hint(mut self, hint: &str) -> Diagnostic {
        self.hint = Some(hint.to_string());
        return self;
    }

    /// What:
    ///  Return a copy that names the repo-relative path the problem is about.
    /// Why:
    ///  A diagnostic without its input is hard to act on.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// withPath(path: string): Diagnostic;
    /// ```
    pub fn with_path(mut self, path: &str) -> Diagnostic {
        self.path = Some(path.to_string());
        return self;
    }

    /// What:
    ///  Return a copy that names the rule that raised it.
    /// Why:
    ///  `explain` and the whole-run report group problems by rule.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// withRule(rule: string): Diagnostic;
    /// ```
    pub fn with_rule(mut self, rule: &str) -> Diagnostic {
        self.rule = Some(rule.to_string());
        return self;
    }

    /// What:
    ///  Turn this diagnostic into an event of the given type (`error` or `warning`).
    /// Why:
    ///  The output boundary only knows events.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// toEvent(type: string): Event;
    /// ```
    pub fn to_event(&self, event_type: &str) -> Event {
        let mut event = Event::new(event_type, &self.code, &self.message);
        event.hint = self.hint.clone();
        if let Some(path) = &self.path {
            event.set_field("path", Value::String(path.clone()));
        }
        if let Some(rule) = &self.rule {
            event.set_field("rule", Value::String(rule.clone()));
        }
        return event;
    }
}

impl fmt::Display for Diagnostic {
    /// What:
    ///  Print `code: message`, the shape a test failure shows.
    /// Why:
    ///  Lets `?` and `{}` formatting work on a diagnostic.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// toString(): string { return `${this.code}: ${this.message}`; }
    /// ```
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        return write!(formatter, "{}: {}", self.code, self.message);
    }
}

impl std::error::Error for Diagnostic {}
