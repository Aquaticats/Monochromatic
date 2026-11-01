//! What:
//!  Typed failures before a Rust semantic check can produce trustworthy findings.
//! Why:
//!  Missing workspace membership and backend panics are not successful empty lint results.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! class SemanticError extends Error {}
//! ```

/// Owned explanation that survives releasing a workspace or parser.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SemanticError {
    /// Operation and remediation,
    ///  with the affected path supplied at the caller boundary.
    pub message: String,
}

/// Construct a typed semantic setup/execution failure.
impl SemanticError {
    /// Copy a borrowed message into the error's owned String rather than retaining an &str.
    pub fn new(message: &str) -> SemanticError {
        return SemanticError {
            message: String::from(message),
        };
    }
}

/// Render the owned explanation through Rust's ordinary error-formatting interface.
impl std::fmt::Display for SemanticError {
    /// Borrow the formatter while emitting the message.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return formatter.write_str(self.message.as_str());
    }
}

/// Integrate with typed application error handling.
impl std::error::Error for SemanticError {}
