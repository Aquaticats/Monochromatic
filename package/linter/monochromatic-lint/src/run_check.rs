//! What:
//!  Check one exact snapshot of a host file:
//!  its own rules,
//!  then every embedded virtual file.
//! Why:
//!  Processors are always on.
//!  Each virtual file is matched against configuration by its own
//! logical path,
//!  and every finding and fix is mapped back to the host before it leaves this module,
//! so the fix loop and the output only ever see host coordinates.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // class HostChecker implements SourceChecker { check(source) { return [...root(source), ...virtualFiles(source)]; } }
//! ```

/// Import the configuration outcome for virtual paths and the finding and fix models.
/// Import Markdown parsing,
///  typed selection and dispatch.
/// Import the processor seam;
///  virtual findings arrive or are projected in host coordinates.
/// Import Rust syntax dispatch,
///  the semantic engine and typed selection.
use crate::{
    config_match::FileConfiguration,
    diagnostic::Diagnostic,
    directive_apply::{DirectiveContext, apply_directives},
    edits::FixError,
    fix_loop::SourceChecker,
    markdown_dispatch::check_markdown_rules,
    markdown_lfs_context::LfsImageContext,
    markdown_rule_settings::{MarkdownRuleSettings, markdown_rule_settings},
    markdown_source::MarkdownSource,
    processors::{ProcessorLanguage, VirtualSource, extract},
    run_check_shell::shell_findings,
    run_failure::{host_span, processing_failure, processor_failure},
    run_lfs::LfsRepos,
    run_paths::{Language, logical_path},
    run_plan::{FilePlan, RootRules},
    rust_dispatch::check_syntax_rules,
    rust_file_engine::RustFileEngine,
    rust_rule_settings::{RustRuleSettings, rust_rule_settings},
    rust_source::RustSource,
};
/// Import the JSONC value that carries a virtual file's resolved rules.
use monochromatic_jsonc_edit::JsoncValue;
/// Import owned native paths for virtual logical names.
use std::path::PathBuf;

/// What:
///  A per-file checking session the fix loop can call once per source snapshot.
/// Why:
///  The plan is fixed for the file;
///  only the source text changes between fix passes,
///  and
/// virtual files are re-extracted from each new snapshot.
/// `'run` is a lifetime:
///  these borrowed values must outlive the checker,
///  which the caller guarantees.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class HostChecker { constructor(plan: FilePlan, lfs: LfsRepos, engine?: RustFileEngine) }
/// ```
pub struct HostChecker<'run> {
    /// Immutable facts about the file under check.
    plan: &'run FilePlan,
    /// Shared LFS repository facts for this run.
    lfs: &'run LfsRepos,
    /// The semantic engine,
    ///  present only on the thread that owns it.
    engine: Option<&'run mut RustFileEngine>,
    /// Debug explanations collected while checking,
    ///  such as a fix that could not be mapped.
    pub notes: Vec<String>,
}

/// What:
///  Construct the session and run each stage of a check.
/// Why:
///  Every stage appends host-coordinate findings to one list;
///  a stage that cannot run appends
/// a processing failure instead of returning early with a clean-looking result.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// new HostChecker(plan, lfs, engine).check(source)
/// ```
impl<'run> HostChecker<'run> {
    /// Bind a plan to the run's shared state.
    pub fn new(
        plan: &'run FilePlan,
        lfs: &'run LfsRepos,
        engine: Option<&'run mut RustFileEngine>,
    ) -> HostChecker<'run> {
        return HostChecker {
            plan,
            lfs,
            engine,
            notes: Vec::<String>::new(),
        };
    }

    /// What:
    ///  Run the host's Rust rules.
    /// Why:
    ///  Syntax rules need no workspace.
    ///  When the semantic rule is selected but cannot run,
    ///  the
    /// syntax rules still report and the unavailable coverage becomes a processing failure.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// private checkRustRoot(source, settings, findings): void
    /// ```
    fn check_rust_root(
        &mut self,
        source: &str,
        settings: &RustRuleSettings,
        findings: &mut Vec<Diagnostic>,
    ) {
        if settings.explicit_types.is_some() {
            let outcome: Result<Vec<Diagnostic>, String> = match self.engine.as_mut() {
                Some(engine) => match engine.check(
                    &self.plan.absolute,
                    String::from(source),
                    self.plan.display.clone(),
                    settings.clone(),
                ) {
                    Ok(found) => Ok(found),
                    Err(error) => Err(error.to_string()),
                },
                None => Err(String::from(
                    "rust/require-explicit-types was selected, but this file was not routed to the semantic engine.",
                )),
            };
            match outcome {
                Ok(found) => {
                    findings.extend(found);
                    return;
                }
                Err(message) => {
                    findings.push(processing_failure(
                        self.plan.display.as_str(),
                        host_span(Language::Rust, source, 0),
                        message,
                    ));
                }
            }
        }
        // `check_syntax_rules` reads only the three syntax selections and never the semantic one,
        // so the full settings pass through unchanged.
        let parsed: RustSource = RustSource::new(self.plan.display.clone(), String::from(source));
        findings.extend(check_syntax_rules(&parsed, settings));
    }

    /// What:
    ///  Prepare the LFS rule's context for a parsed document,
    ///  when that rule is selected.
    /// Why:
    ///  A repository whose facts cannot be read is a processing failure for this file;
    ///  the
    /// other Markdown rules still run.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// private lfsContext(document, settings, source, findings): LfsImageContext | undefined
    /// ```
    fn lfs_context(
        &self,
        document: &MarkdownSource,
        settings: &MarkdownRuleSettings,
        source: &str,
        findings: &mut Vec<Diagnostic>,
    ) -> Option<LfsImageContext> {
        let setting = settings.lfs_image_url.as_ref()?;
        match self.lfs.context_for(&self.plan.absolute, document, setting) {
            Ok(context) => return context,
            Err(error) => {
                findings.push(processing_failure(
                    self.plan.display.as_str(),
                    host_span(self.plan.language, source, 0),
                    format!("markdown/lfs-image-url could not check this file: {error}"),
                ));
                return None;
            }
        }
    }

    /// What:
    ///  Run the host's Markdown or MDX rules;
    ///  returns false when the host did not parse.
    /// Why:
    ///  A parse failure or MDX error is one processing finding;
    ///  extraction would only repeat it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// private checkMarkdownRoot(source, settings, findings): boolean
    /// ```
    fn check_markdown_root(
        &mut self,
        source: &str,
        settings: &MarkdownRuleSettings,
        findings: &mut Vec<Diagnostic>,
    ) -> bool {
        let mdx: bool = self.plan.language == Language::Mdx;
        let document: MarkdownSource =
            match MarkdownSource::new(self.plan.display.clone(), String::from(source), mdx) {
                Ok(parsed) => parsed,
                Err(error) => {
                    findings.push(processing_failure(
                        self.plan.display.as_str(),
                        host_span(self.plan.language, source, error.offset),
                        error.message,
                    ));
                    return false;
                }
            };
        let lfs: Option<LfsImageContext> = self.lfs_context(&document, settings, source, findings);
        findings.extend(check_markdown_rules(
            &document,
            settings,
            false,
            lfs.as_ref(),
        ));
        return true;
    }

    /// What:
    ///  Map one virtual Markdown finding to the host,
    ///  keeping it when only its fix cannot be mapped.
    /// Why:
    ///  A fix the processor refuses to project is unavailable,
    ///  not a reason to hide the finding;
    /// a finding whose position cannot be mapped at all is a processing failure.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// private project(input, finding, source, findings): void
    /// ```
    fn project(
        &mut self,
        input: &VirtualSource,
        finding: Diagnostic,
        source: &str,
        findings: &mut Vec<Diagnostic>,
    ) {
        let mut without_fix: Option<Diagnostic> = None;
        if finding.fix.is_some() {
            let mut copy: Diagnostic = finding.clone();
            copy.fix = None;
            without_fix = Some(copy);
        }
        let refusal = match input.project_diagnostic(finding) {
            Ok(Some(mapped)) => {
                findings.push(mapped);
                return;
            }
            Ok(None) => return,
            Err(error) => error,
        };
        let Some(retry): Option<Diagnostic> = without_fix else {
            findings.push(processor_failure(self.plan.language, source, &refusal));
            return;
        };
        match input.project_diagnostic(retry) {
            Ok(Some(mapped)) => {
                self.notes.push(format!(
                    "{}: reported {} without its fix: {}",
                    input.filename(),
                    mapped.code,
                    refusal.message
                ));
                findings.push(mapped);
            }
            Ok(None) => {}
            Err(error) => findings.push(processor_failure(self.plan.language, source, &error)),
        }
    }

    /// What:
    ///  Check one virtual Markdown file (a rustdoc comment) under its own resolved rules.
    /// Why:
    ///  Rustdoc is Markdown,
    ///  never MDX;
    ///  its unlabeled fences are doc tests.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// private checkVirtualMarkdown(input, rules, source, findings): void
    /// ```
    fn check_virtual_markdown(
        &mut self,
        input: &VirtualSource,
        settings: &MarkdownRuleSettings,
        source: &str,
        findings: &mut Vec<Diagnostic>,
    ) {
        let document: MarkdownSource = match MarkdownSource::new(
            String::from(input.filename()),
            String::from(input.source()),
            false,
        ) {
            Ok(parsed) => parsed,
            Err(error) => {
                let failure: Diagnostic = processing_failure(
                    input.filename(),
                    host_span(Language::Markdown, input.source(), error.offset),
                    error.message,
                );
                self.project(input, failure, source, findings);
                return;
            }
        };
        let lfs: Option<LfsImageContext> = self.lfs_context(&document, settings, source, findings);
        let virtual_findings: Vec<Diagnostic> =
            check_markdown_rules(&document, settings, input.is_rustdoc(), lfs.as_ref());
        for finding in virtual_findings {
            self.project(input, finding, source, findings);
        }
    }

    /// What:
    ///  Check one virtual file under the rules its logical path resolves to.
    /// Why:
    ///  `**/*.rs` reaches fenced Rust and `**/*.md` reaches rustdoc unless a block's `ignores`
    /// says otherwise;
    ///  an unconfigured or ignored virtual path is simply not checked.
    /// `Err` carries the reason the virtual file's rules could not be determined.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// private checkVirtualRules(input, source, findings): void // throws when rules cannot be resolved
    /// ```
    fn check_virtual_rules(
        &mut self,
        input: &VirtualSource,
        source: &str,
        findings: &mut Vec<Diagnostic>,
    ) -> Result<(), String> {
        let Some(suffix): Option<&str> = input.filename().strip_prefix(self.plan.display.as_str())
        else {
            return Err(String::from("its name is not under its host file's name"));
        };
        let logical: PathBuf = logical_path(&self.plan.relative, suffix);
        let rules: JsoncValue = match self.plan.config.resolve(&logical) {
            Ok(FileConfiguration::Configured { rules }) => rules,
            Ok(FileConfiguration::Ignored) | Ok(FileConfiguration::Unconfigured) => return Ok(()),
            Err(error) => return Err(error.to_string()),
        };
        if input.language() == ProcessorLanguage::Shell {
            // A workflow step's findings are in the step's coordinates; the processor maps them to the workflow.
            for finding in shell_findings(input, &rules)? {
                self.project(input, finding, source, findings);
            }
            return Ok(());
        }
        if input.language() == ProcessorLanguage::Rust {
            let settings: RustRuleSettings = match rust_rule_settings(&rules) {
                Ok(typed) => typed,
                Err(error) => return Err(error.to_string()),
            };
            match input.check_rust(settings) {
                Ok(found) => findings.extend(found),
                Err(error) => findings.push(processor_failure(self.plan.language, source, &error)),
            }
            return Ok(());
        }
        let settings: MarkdownRuleSettings = match markdown_rule_settings(&rules) {
            Ok(typed) => typed,
            Err(error) => return Err(error.to_string()),
        };
        self.check_virtual_markdown(input, &settings, source, findings);
        return Ok(());
    }

    /// What:
    ///  Check one virtual file,
    ///  turning an unresolvable rule selection into a processing failure.
    /// Why:
    ///  A virtual file whose rules cannot be determined was not checked,
    ///  and the run must say so.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// private checkVirtual(input, source, findings): void
    /// ```
    fn check_virtual(
        &mut self,
        input: &VirtualSource,
        source: &str,
        findings: &mut Vec<Diagnostic>,
    ) {
        if let Err(message) = self.check_virtual_rules(input, source, findings) {
            findings.push(processing_failure(
                self.plan.display.as_str(),
                host_span(self.plan.language, source, 0),
                format!("Cannot resolve rules for {}: {message}", input.filename()),
            ));
        }
    }
}

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

/// What:
///  The fix loop's view of a host file.
/// Why:
///  The loop's interface allows a checker to refuse;
///  this checker reports refusals as findings instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// check(source: string): Diagnostic[]
/// ```
impl SourceChecker for HostChecker<'_> {
    /// Wrap the infallible snapshot check in the loop's result type.
    fn check(&mut self, source: &str) -> Result<Vec<Diagnostic>, FixError> {
        return Ok(self.check_snapshot(source));
    }
}

/// Host and virtual checking controls stay outside release artifacts.
#[cfg(test)]
#[path = "run_check_tests.rs"]
mod tests;
