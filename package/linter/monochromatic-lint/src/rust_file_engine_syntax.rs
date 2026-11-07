//! What:
//!  The Rust-input router of a build without the `semantic` feature:
//!  syntax checks run,
//! and a selected semantic rule is an explicit failure.
//! Why:
//!  The semantic rule needs the rust-analyzer backend,
//! whose code and start-up work a caller such as the Git
//!  policy wrapper does not want in its executable.
//!  A rule that cannot run must fail loudly instead of passing the file unchecked.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // class RustFileEngine { check(...) { if (settings.explicitTypes) throw new SemanticError(...); return syntaxRules(...); } }
//! ```

/// Import actual rule results,
///  typed selection and syntax-only execution.
use crate::diagnostic::Diagnostic;
/// Import the syntax-rule dispatcher that runs the enabled checks on one Rust source.
use crate::rust_dispatch::check_syntax_rules;
/// Import the preparation choice,
/// which the callers name even when no workspace is ever loaded.
use crate::rust_manifest::WorkspacePreparation;
/// Import the resolved settings that tell the dispatcher which rules are enabled.
use crate::rust_rule_settings::RustRuleSettings;
/// Import the typed failure of a rule that cannot run.
use crate::rust_semantic_error::SemanticError;
/// Import the parsed Rust source model that each check reads.
use crate::rust_source::RustSource;
/// Import the path type that names the checked file.
use std::path::Path;

/// What:
///  The invocation-owned router,
///  without any workspace cache because no workspace can load.
/// Why:
///  Callers hold one and pass it down exactly as with the semantic build.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class RustFileEngine {}
/// ```
pub struct RustFileEngine;

/// Dispatch exact snapshots without a semantic backend.
impl RustFileEngine {
    /// Construct the router;
    ///  the arguments exist so callers compile unchanged against either build.
    pub fn new(_preparation: WorkspacePreparation, _progress: fn(String)) -> RustFileEngine {
        return RustFileEngine;
    }

    /// No workspace is ever initialized in this build.
    pub fn workspace_count(&self) -> usize {
        return 0;
    }

    /// Check supplied source bytes with the syntax rules;
    ///  a selected semantic rule fails with the way forward.
    pub fn check(
        &mut self,
        _path: &Path,
        source: String,
        filename: String,
        settings: RustRuleSettings,
    ) -> Result<Vec<Diagnostic>, SemanticError> {
        if settings.explicit_types.is_some() {
            return Err(SemanticError::new(
                "This build of monochromatic-lint has no semantic engine, so rust/require-explicit-types cannot run. Use the monochromatic-lint executable, or configure this rule off.",
            ));
        }
        let context: RustSource = RustSource::new(filename, source);
        return Ok(check_syntax_rules(&context, &settings));
    }
}
