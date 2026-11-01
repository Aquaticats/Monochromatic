//! What:
//!     Regular-expression abstract syntax:
//!  the node algebra and its smart constructors.
//! Why:
//!      This file is the Rust module that groups the ast implementation,
//!  so the
//!          compiler gives those items one namespace and sibling modules can import that name.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // module ast: see exported functions and types below.
//! ```

/// What:
///     The derivative-regex node type (`Node`) and its variants.
/// Why:
///      The package keeps that concept in a separate Rust file so this module can refer to
///          it by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./node";
/// ```
pub mod node;

/// What:
///     Normalizing constructors that keep the node algebra canonical.
/// Why:
///      The package keeps that concept in a separate Rust file so this module can refer to
///          it by name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./smart";
/// ```
pub mod smart;
