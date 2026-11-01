//! What:
//!  The lint diagnostic:
//!  a message,
//!  a help line,
//!  and the data they come from.
//! Why:
//!  rustc builds lint output from a value implementing `Diagnostic`;
//!  using a struct instead of the closure-taking `span_lint` helper keeps the lint's own source free of closures.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! class ClosureFound { constructor(public captures: string[], public placement: Placement) {} }
//! ```

/// Import the text builders.
use crate::message::{help_message, primary_message};
/// Import the placement categories.
use crate::placement::Placement;
/// Import the diagnostic builder types of the compiler.
use rustc_errors::{Diag, DiagCtxtHandle, Diagnostic, Level};

/// What:
///  Everything the message needs:
///  the captured names and where the closure sits.
/// Why:
///  `into_diag` consumes this value and turns it into rustc's output format.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ClosureFound = { captures: string[]; placement: Placement };
/// ```
pub struct ClosureFound {
    /// Names of the captured variables;
    /// empty when the closure captures nothing.
    pub captures: Vec<String>,
    /// Placement of the closure.
    pub placement: Placement,
}

/// Let rustc turn a `ClosureFound` into its diagnostic output.
impl<'a> Diagnostic<'a> for ClosureFound {
    /// What:
    ///  Build the compiler diagnostic:
    ///  primary message plus one help line.
    /// Why:
    ///  This is the trait method rustc calls when the lint fires and is not allowed.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function intoDiag(self: ClosureFound): Diag;
    /// ```
    fn into_diag(self, dcx: DiagCtxtHandle<'a>, level: Level) -> Diag<'a> {
        let mut diag: Diag<'a> = Diag::new(dcx, level, "");
        diag.primary_message(primary_message(&self.captures, self.placement));
        diag.help(help_message(&self.captures, self.placement));
        return diag;
    }
}
