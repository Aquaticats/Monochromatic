//! What:
//!  The check of one snapshot of a host file:
//!  its own rules,
//!  then every virtual file,
//!  then the host's inline directives.
//! Why:
//!  A child module of the host checker can use its private fields and methods,
//!  so this half of the checker lives in its own file under the line limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! checkSnapshot(source: string): Diagnostic[]
//! ```

/// Import the checker this module extends.
use super::HostChecker;
/// Import the finding model.
use crate::diagnostic::Diagnostic;
/// Import the directive filter and its context.
use crate::directive_apply::{DirectiveContext, apply_directives};
/// Import the processor seam.
use crate::processors::{VirtualSource, extract};
/// Import the processing-failure construction.
use crate::run_failure::processor_failure;
/// Import the plan records.
use crate::run_plan::{FilePlan, RootRules};

/// What:
///  The complete check of one snapshot,
///  which cannot itself fail.
/// Why:
///  Inability to check is carried as processing-failure findings,
///  never as an empty list,
///  so
/// lint mode reports it and fix mode refuses to publish edits.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// checkSnapshot(source: string): Diagnostic[]
/// ```
impl HostChecker<'_> {
    /// What:
    ///  Check the host's own rules,
    ///  then every virtual file extracted from this snapshot,
    ///  and apply the host's inline directives to all of those findings.
    /// Why:
    ///  Directives are read from the host only,
    ///  in the host's own comment syntax,
    ///  and filter findings in host coordinates:
    ///  the rules,
    ///  the semantic engine and the embedded-source processors are all covered by one filter,
    ///  and a suppressed finding never reaches the fix loop with its fix.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// checkSnapshot(source: string): Diagnostic[]
    /// ```
    pub fn check_snapshot(&mut self, source: &str) -> Vec<Diagnostic> {
        let findings: Vec<Diagnostic> = self.collect_findings(source);
        let context: DirectiveContext<'_> = DirectiveContext {
            language: self.plan.language,
            filename: self.plan.display.as_str(),
            source,
            options: self.plan.directives,
        };
        return apply_directives(&context, findings);
    }

    /// Check the host's own rules,
    ///  then every virtual file extracted from this snapshot,
    ///  before directives are applied.
    fn collect_findings(&mut self, source: &str) -> Vec<Diagnostic> {
        let mut findings: Vec<Diagnostic> = Vec::<Diagnostic>::new();
        let plan: &FilePlan = self.plan;
        let parsed: bool = match &plan.root {
            RootRules::None => true,
            RootRules::Rust { settings } => {
                self.check_rust_root(source, settings, &mut findings);
                true
            }
            RootRules::Markdown { settings } => {
                self.check_markdown_root(source, settings, &mut findings)
            }
        };
        if !parsed {
            return findings;
        }
        let inputs: Vec<VirtualSource> = match extract(
            plan.display.clone(),
            String::from(source),
            plan.language.processor(),
        ) {
            Ok(extracted) => extracted,
            Err(error) => {
                findings.push(processor_failure(plan.language, source, &error));
                return findings;
            }
        };
        for input in &inputs {
            self.check_virtual(input, source, &mut findings);
        }
        return findings;
    }
}
