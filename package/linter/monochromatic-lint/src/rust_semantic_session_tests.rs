//! What:
//!  Session failure boundaries and named-callback unwind controls.
//! Why:
//!  Workspace problems and resolver panics must never become successful empty results.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Exercise successful, typed-error and panic outcomes through the same protected query boundary.
//! ```

/// Import private query protection and payload formatting.
use super::{panic_message, protect_query};
/// Import the shared result models.
use crate::diagnostic::Diagnostic;
use crate::rust_semantic_error::SemanticError;

/// Return a successful named query without a captured closure.
fn success() -> Result<Vec<Diagnostic>, SemanticError> {
    return Ok(Vec::<Diagnostic>::new());
}

/// Return an ordinary typed query failure.
fn failure() -> Result<Vec<Diagnostic>, SemanticError> {
    return Err(SemanticError::new("fixture query failed"));
}

/// Exercise the borrowed-text panic payload.
fn text_panic() -> Result<Vec<Diagnostic>, SemanticError> {
    panic!("fixture panic");
}

/// Exercise a non-text payload,
///  which still must produce a typed failure.
fn numeric_panic() -> Result<Vec<Diagnostic>, SemanticError> {
    std::panic::panic_any::<u32>(42_u32);
}

/// Both supported outcomes survive the protection wrapper without losing the typed error.
#[test]
fn named_query_outcomes_are_preserved() {
    assert_eq!(protect_query(success).expect("successful query"), []);
    let error: SemanticError = protect_query(failure).expect_err("typed failure");
    assert_eq!(error.message, "fixture query failed");
    assert_eq!(error.to_string(), error.message);
}

/// Panics produce failures explaining that the input was not verified.
#[test]
fn named_query_panics_cannot_masquerade_as_a_pass() {
    let text_error: SemanticError = protect_query(text_panic).expect_err("caught text panic");
    assert!(text_error.message.contains("fixture panic"));
    assert!(text_error.message.contains("not verified"));
    let number_error: SemanticError =
        protect_query(numeric_panic).expect_err("caught non-text panic");
    assert!(number_error.message.contains("non-text panic payload"));
    assert!(number_error.message.contains("not verified"));
}

/// Owned and borrowed panic strings retain their contents rather than a debug type identifier.
#[test]
fn panic_payload_text_is_preserved() {
    let owned: String = String::from("owned panic");
    let borrowed: &str = "borrowed panic";
    assert_eq!(panic_message(&owned), "owned panic");
    assert_eq!(panic_message(&borrowed), "borrowed panic");
}
