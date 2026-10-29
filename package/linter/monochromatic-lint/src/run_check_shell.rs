//! What:
//!  The `shell/` rules over one extracted workflow step, from resolved configuration.
//! Why:
//!  The host checker only needs the findings in the step's own coordinates,
//!  so reading the settings and the dialect lives apart from the host checker's bookkeeping.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function shellFindings(input: VirtualSource, rules: JsoncValue): Diagnostic[]; // throws string
//! ```

/// Import the finding model.
use crate::diagnostic::Diagnostic;
/// Import the virtual file the findings are about.
use crate::processors::VirtualSource;
/// Import the dialect named by the virtual file's suffix.
use crate::shell_dialect::ShellDialect;
/// Import the typed selection of the `shell/` rules.
use crate::shell_rule_settings::shell_rule_settings;
/// Import the rules.
use crate::shell_rules::check_shell;
/// Import the resolved rule settings of the virtual file.
use monochromatic_jsonc_edit::JsoncValue;

/// What:
///  Check one step under the rules its virtual path resolved to.
/// Why:
///  `Err` carries the reason the step's rules could not be determined,
///  which the host checker reports as a processing failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function shellFindings(input: VirtualSource, rules: JsoncValue): Diagnostic[]; // throws string
/// ```
pub(crate) fn shell_findings(
    input: &VirtualSource,
    rules: &JsoncValue,
) -> Result<Vec<Diagnostic>, String> {
    let settings = match shell_rule_settings(rules) {
        Ok(typed) => typed,
        Err(error) => return Err(error.to_string()),
    };
    let Some(dialect): Option<ShellDialect> = ShellDialect::from_filename(input.filename()) else {
        return Err(format!("it names no shell dialect: {}", input.filename()));
    };
    return Ok(check_shell(
        input.filename(),
        input.source(),
        dialect,
        &settings,
    ));
}
