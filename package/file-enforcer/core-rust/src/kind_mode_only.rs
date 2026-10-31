//! What:
//!  The `mode-only` rule kind:
//! keep a file's permission bits when the file exists.
//! Why:
//!  `forbidden-strings.append.local.txt` must stay 0600 when present,
//! and the tool must never create it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function modeOnlyRule(path: string, mode: number): Rule
//! ```

/// Imports `parse_mode` from `crate::config_decode`.
use crate::config_decode::parse_mode;
/// Imports `Diagnostic` from `crate::diagnostic`.
use crate::diagnostic::Diagnostic;
/// Imports `Change`,
/// `Rule`,
/// `RulePlan` from `crate::rule`.
use crate::rule::{Change, Rule, RulePlan};
/// Imports `View` from `crate::view`.
use crate::view::View;

/// What:
///  A planned `mode-only` rule.
/// Why:
///  Holds the path and the wanted bits.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ModeOnlyRule implements Rule { id: string; path: string; mode: number }
/// ```
pub struct ModeOnlyRule {
    /// Rule id.
    pub id: String,
    /// File path.
    pub path: String,
    /// Wanted permission bits.
    pub mode: u32,
}

/// What:
///   The constructor of the `mode-only` rule.
/// Why:
///   The registry builds the rule from a decoded config entry,
/// parsing the octal mode text once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// new ModeOnlyRule(id, path, mode)
/// ```
impl ModeOnlyRule {
    /// What:
    ///  Build from decoded fields.
    /// Why:
    ///  The registry calls this with the decoded config entry.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// new ModeOnlyRule(id, path, mode)
    /// ```
    pub fn new(id: &str, path: &str, mode: &str) -> ModeOnlyRule {
        return ModeOnlyRule { id: id.to_string(), path: path.to_string(), mode: parse_mode(mode) };
    }
}

/// What:
///   How the `mode-only` kind plugs into the engine:
/// its id,
/// its kind name and its plan.
/// Why:
///   The plan plans a mode change for an existing file and never creates the file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ModeOnlyRule implements Rule { id(); kind(); plan(view) }
/// ```
impl Rule for ModeOnlyRule {
    /// The rule id.
    fn id(&self) -> &str {
        return &self.id;
    }

    /// The kind tag.
    fn kind(&self) -> &str {
        return "mode-only";
    }

    /// What:
    ///  Plan a mode change when the file exists,
    /// nothing otherwise.
    /// Why:
    ///  Absence is fine:
    /// the file is created by a person,
    /// not by the tool.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// plan(view: View): RulePlan
    /// ```
    fn plan(&self, view: &mut View) -> Result<RulePlan, Diagnostic> {
        let mut plan = RulePlan::default();
        if view.mode_of(&self.path)?.is_some() {
            plan.changes.push(Change::SetMode { path: self.path.clone(), mode: self.mode });
        }
        return Ok(plan);
    }
}
