//! The five feature paths against the scripted server:
//!  targets,
//!  hover,
//!  hints,
//!  diagnostics,
//!  and
//! the request states (unsupported,
//!  failed,
//!  superseded,
//!  empty,
//!  timeout,
//!  stale).
//!
//! What:
//!  `#[path = "..."]` names the file a module is read from;
//!  this module's parts live under `tests/language/requests/`.
//! Why:
//!  One test crate links the application library once and shares the source text
//!       every request is asked about without unused-code warnings.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! import * as fixtures from './language/requests/fixtures';
//! ```

/// What a reply may say:
/// targets,
/// hover,
/// failures and refusals.
#[path = "requests/reply_validation.rs"]
mod reply_validation;

/// The states a request passes through,
/// and replies it drops.
#[path = "requests/request_lifecycle.rs"]
mod request_lifecycle;

/// Hints and diagnostics through a reload.
#[path = "requests/hints_and_diagnostics.rs"]
mod hints_and_diagnostics;

/// Line 0 has an accented letter and an astral character before offset 14;
///  line 2 starts with both.
const SOURCE: &str = "caf\u{e9} \u{1F600} hello world\n\n\u{e9}\u{1F600}cdef tail\n";
