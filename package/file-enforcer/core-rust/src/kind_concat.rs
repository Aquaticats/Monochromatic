//! What:
//!  The `concat` rule kind: one output file whose content is its parts joined end to end.
//! Why:
//!  `CLAUDE.md` (a preamble file then `AGENTS.md`) and the root `LICENSE` (one copied file) are the same rule with
//!  different parts.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function concatRule(output: string, parts: ValueExpr[], mode?: number): Rule
//! ```

use crate::config_decode::parse_mode;
use crate::config_parts::ValueExpr;
use crate::diagnostic::Diagnostic;
use crate::rule::{Change, Rule, RulePlan};
use crate::value_eval::{Vars, evaluate_text};
use crate::view::View;

/// What:
///  A planned `concat` rule.
/// Why:
///  Holds the decoded fields; planning evaluates the parts through the view.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ConcatRule implements Rule { id: string; output: string; parts: ValueExpr[]; mode?: number }
/// ```
pub struct ConcatRule {
    /// Rule id.
    pub id: String,
    /// Output path.
    pub output: String,
    /// Parts in order.
    pub parts: Vec<ValueExpr>,
    /// Permission bits, when the rule owns them.
    pub mode: Option<u32>,
}

impl ConcatRule {
    /// What:
    ///  Build from decoded fields, converting the mode text.
    /// Why:
    ///  The registry calls this with the decoded config entry.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// new ConcatRule(id, output, parts, mode)
    /// ```
    pub fn new(id: &str, output: &str, parts: &[ValueExpr], mode: &Option<String>) -> ConcatRule {
        let parsed_mode = mode.as_deref().map(parse_mode);
        return ConcatRule { id: id.to_string(), output: output.to_string(), parts: parts.to_vec(), mode: parsed_mode };
    }
}

impl Rule for ConcatRule {
    /// The rule id.
    fn id(&self) -> &str {
        return &self.id;
    }

    /// The kind tag.
    fn kind(&self) -> &str {
        return "concat";
    }

    /// What:
    ///  Evaluate each part to text and write the joined bytes.
    /// Why:
    ///  A missing input file fails the run with the file named.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// plan(view: View): RulePlan
    /// ```
    fn plan(&self, view: &mut View) -> Result<RulePlan, Diagnostic> {
        let vars = Vars::new();
        let mut joined = String::new();
        for part in &self.parts {
            joined.push_str(&evaluate_text(view, part, &vars)?);
        }
        let change = Change::Write { path: self.output.clone(), content: joined.into_bytes(), mode: self.mode };
        return Ok(RulePlan { changes: vec![change], warnings: Vec::new() });
    }
}
