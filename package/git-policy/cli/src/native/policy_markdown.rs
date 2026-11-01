//! What:
//!  The optional `markdown/autofix` policy:
//!  every Markdown candidate the lifecycle
//!       offers is passed through `monochromatic-lint --fix` with the policy's rules;
//!  a
//!       changed source is reported,
//!  and corrected on `git cli-git fix`,
//!  and each finding
//!       the fixes leave is reported as it is.
//! Why:
//!  The installed wrapper's `markdownLintPolicy` (`package/git-policy/markdown-lint/src`)
//!      decides the behaviour:
//!  deleted files,
//!  symbolic links and submodules are skipped,
//!      and so are excluded paths before any linter starts and contents that are not UTF-8;
//!      candidates are linted one at a time,
//!  in candidate order;
//!  the codes and words of
//!      both findings are kept.
//!  One thing differs by design:
//!  the extension test is the
//!      linter's own (`.md` and `.mdx`,
//!  exact case),
//!  so no file is sent that the linter
//!      refuses.
//!  The autofix finding keeps the installed wrapper's words,
//!  which name its
//!      former linter,
//!  so both wrappers report the same file the same way while both are
//!      in use.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const findings = await rewriteCandidates({ rules, exclude, repositoryRoot, canApplyPatches, candidates });
//! ```

/// Import the candidate's mode,
///  which decides whether it is a file of text.
use super::candidate_object::CandidateMode;
/// Import the candidate's change kinds.
use super::candidate_record::CandidateChange;
/// Import the candidate and version types.
use super::candidate_version::{Candidate, CandidateVersion};
/// Import the policy's validated options and the configuration spelling of a rule.
use super::config_schema::{MarkdownAutofixOptions, PolicyConfig, markdown_rule_name};
/// Import the engine-failure codes.
use super::diagnostics::EngineFailureCode;
/// Import the event form of a pathname.
use super::event_path::EventPath;
/// Import the conversion of Git's pathname bytes to a path.
use super::git_metadata::path_from_git_bytes;
/// Import the compiled `exclude` patterns.
use super::markdown_exclude::{ExcludeMatcher, compile_exclude};
/// Import what one usable linter run returns.
use super::markdown_linter_output::{LintRun, RemainingFinding};
/// Import the lifecycle's candidates and the correction a fix applies.
use super::policy_content::{ContentState, Correction, LifecycleContent};
/// Import the finding and outcome types of a check.
use super::policy_engine::{PolicyFinding, PolicyOutcome};
/// Import the lifecycle points;
///  only a direct fix corrects.
use super::policy_trigger::Trigger;
/// Import the facts interface for the repository's top level.
use super::repository_facts::RepositoryFacts;
/// Import the repository location answer.
use super::repository_location::RepositoryLocation;
/// Import the worktree's top level from Git's answer.
use super::worktree_identity::worktree_root;
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{Path, PathBuf};
/// `Rc<T>` is a shared,
///  read-only handle.
use std::rc::Rc;

/// The policy-local code of a candidate the fixes rewrite.
pub const MARKDOWN_AUTOFIX_CODE: &str = "markdown-autofix";

/// The policy-local code of a finding the fixes left.
pub const MARKDOWN_VIOLATION_CODE: &str = "markdown-violation";

/// What the person can do about a candidate the linter could not check.
pub const UNCHECKED_REMEDY: &str = "The Markdown candidate was not checked and nothing was changed; fix the cause, or pass --no-enforce-markdown/autofix to skip this policy for one command.";

/// What:
///  One candidate for the linter.
///  `'a` names how long the borrowed parts last.
/// Why:
///   The linter runs from the top level,
///  so `.lfsconfig`,
///  `.gitattributes` and linked
///       images resolve as they do for the person,
///  and lints `source` as the file at `path`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LintRequest = { topLevel: string; options: MarkdownAutofixOptions; path: Uint8Array; source: Uint8Array };
/// ```
#[derive(Clone, Copy, Debug)]
pub struct LintRequest<'a> {
    /// The repository's top level,
    ///  the linter's working directory.
    pub top_level: &'a Path,
    /// The policy's rules and exclude patterns.
    pub options: &'a MarkdownAutofixOptions,
    /// The candidate's pathname,
    ///  relative to the top level,
    ///  as Git's raw bytes.
    pub path: &'a [u8],
    /// The candidate's bytes,
    ///  already known to be UTF-8.
    pub source: &'a [u8],
}

/// What:
///  Whatever lints one candidate.
///  A `trait` is a named set of methods,
///  like a TS
///       `interface`;
///  `&mut self` lends it for writing,
///  because it may remember the program
///       and its configuration across candidates.
/// Why:
///   The executable starts the real linter;
///  tests answer from a script,
///  so every branch
///       of the policy is reached without a child process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface MarkdownLinter { lint(request: LintRequest): LintRun } // throws string
/// ```
pub trait MarkdownLinter {
    /// Lint one candidate:
    ///  the usable run,
    ///  or why there is none,
    ///  for the person.
    fn lint(&mut self, request: &LintRequest<'_>) -> Result<LintRun, String>;
}

/// What:
///  The policy's state for one invocation:
///  its options,
///  its linter and the compiled
///       exclude patterns.
///  `Box<dyn MarkdownLinter>` owns "any linter",
///  chosen at run time.
/// Why:
///   The lifecycle sets the options once;
///  the patterns compile on first use,
///  and a
///       failure to compile is remembered,
///  like the scanner's rules.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MarkdownState = { options: MarkdownAutofixOptions; linter: MarkdownLinter; exclude?: ExcludeMatcher | string };
/// ```
pub struct MarkdownState {
    /// The policy's validated options;
    ///  defaults until the configuration is loaded.
    pub options: MarkdownAutofixOptions,
    /// What lints a candidate.
    pub linter: Box<dyn MarkdownLinter>,
    /// The compiled exclude patterns,
    ///  or the reason they did not compile.
    exclude: Option<Result<ExcludeMatcher, String>>,
}

/// What:
///  A state over `linter` with the default options and nothing compiled.
/// Why:
///   Every lifecycle builds its checks the same way and then sets the options.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const markdownState = (linter: MarkdownLinter): MarkdownState => ({ options: defaults.markdownAutofix, linter });
/// ```
pub fn markdown_state(linter: Box<dyn MarkdownLinter>) -> MarkdownState {
    return MarkdownState {
        options: PolicyConfig::defaults().markdown_autofix,
        linter,
        exclude: None,
    };
}

/// What:
///  `impl MarkdownState { ... }` attaches the one question about exclusion.
/// Why:
///   The patterns compile at most once per invocation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class MarkdownState { excludes(path: Uint8Array): boolean }
/// ```
impl MarkdownState {
    /// What:
    ///  Whether the exclude patterns name `path`;
    ///  `Err` is the compile failure.
    /// Why:
    ///   An excluded candidate is left alone before its bytes are read.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// excludes(path: Uint8Array): boolean; // throws string
    /// ```
    fn excludes(&mut self, path: &[u8]) -> Result<bool, String> {
        if self.exclude.is_none() {
            self.exclude = Some(compile_exclude(self.options.exclude.as_slice()));
        }
        match &self.exclude {
            Some(Ok(matcher)) => return Ok(matcher.excludes(path)),
            Some(Err(reason)) => return Err(reason.clone()),
            None => {
                return Err(String::from(
                    "cli-git did not compile the markdown/autofix exclude patterns; this is a defect in cli-git.",
                ));
            }
        }
    }
}

/// What:
///  Whether a pathname ends in `.md` or `.mdx`,
///  with exactly that case,
///  by the
///       platform's rule for a file name's extension.
/// Why:
///   The linter decides the language with the same rule (`run_paths.rs`,
///       `language_of`) and refuses any other name read from standard input;
///  a name such
///       as `.md`,
///  with no extension under that rule,
///  is therefore not Markdown here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isMarkdownPath = (path: string) => ['md', 'mdx'].includes(extname(path).slice(1));
/// ```
pub fn is_markdown_path(path: &[u8]) -> bool {
    let Some(spelled) = path_from_git_bytes(path) else {
        return false;
    };
    // `.and_then(OsStr::to_str)` keeps only an extension that is text.
    match spelled.extension().and_then(std::ffi::OsStr::to_str) {
        Some("md") | Some("mdx") => return true,
        _ => return false,
    }
}

/// What:
///  Whether the policy inspects a candidate at all.
/// Why:
///   Deletions have no content;
///  symbolic links and submodules are not files of text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isEligible = (c) => c.change !== 'deleted' && (c.mode === 'regular' || c.mode === 'executable') && isMarkdownPath(c.path);
/// ```
fn is_eligible(candidate: &Candidate) -> bool {
    if candidate.change == CandidateChange::Deleted {
        return false;
    }
    if candidate.mode != CandidateMode::Regular && candidate.mode != CandidateMode::Executable {
        return false;
    }
    return is_markdown_path(candidate.path.as_slice());
}

/// What:
///  The outcome of a candidate the policy could not check,
///  naming that candidate.
/// Why:
///   The policy's own machinery failed,
///  which is `policy-incomplete`;
///  the event names
///       the file in its own field and the message says what to do.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unchecked = (path, reason): PolicyOutcome => ({ kind: 'failed', code: 'policy-incomplete', message: `${reason}. ${REMEDY}`, path });
/// ```
fn unchecked(path: Option<&[u8]>, reason: &str) -> PolicyOutcome {
    return PolicyOutcome::Failed {
        code: EngineFailureCode::PolicyIncomplete,
        message: format!("{reason}. {UNCHECKED_REMEDY}"),
        // `.map(EventPath::from_git_bytes)` renders the pathname when there is one.
        path: path.map(EventPath::from_git_bytes),
    };
}

/// What:
///  The repository's top level,
///  from Git's answer about where the command runs.
/// Why:
///   The linter runs there;
///  a missing answer is a repository fact that could not be read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function topLevel(facts): Promise<string>;
/// ```
fn top_level<F: RepositoryFacts>(facts: &mut F) -> Result<PathBuf, PolicyOutcome> {
    let location: RepositoryLocation = match facts.location() {
        Ok(found) => found,
        Err(message) => {
            return Err(PolicyOutcome::Failed {
                code: EngineFailureCode::ContentUnavailable,
                message,
                path: None,
            });
        }
    };
    match worktree_root(&location.identity) {
        Some(root) => return Ok(root.to_path_buf()),
        None => {
            return Err(PolicyOutcome::Failed {
                code: EngineFailureCode::ContentUnavailable,
                message: String::from(
                    "cli-git could not find the repository's top level, where monochromatic-lint must run.",
                ),
                path: None,
            });
        }
    }
}

/// What:
///  The rules of the options as configured,
///  joined by `, `.
/// Why:
///   The autofix finding lists them,
///  as the installed wrapper's did.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ruleList = (options) => options.rules.join(', ');
/// ```
fn rule_list(options: &MarkdownAutofixOptions) -> String {
    let mut names: Vec<String> = Vec::new();
    for rule in &options.rules {
        names.push(format!("markdown/{}", markdown_rule_name(*rule)));
    }
    return names.join(", ");
}

/// What:
///  The finding for one remaining linter finding,
///  in the installed wrapper's words:
///       `<rule> at <path>:<line>:<column>: <message>`.
/// Why:
///   The person sees which rule,
///  where,
///  and why,
///  without the linter's own output.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const violation = (path, d) => ({ code: 'markdown-violation', message: `${d.ruleId} at ${path}:${d.line}:${d.column}: ${d.message}`, path });
/// ```
fn violation(path: &EventPath, remaining: &RemainingFinding) -> PolicyFinding {
    return PolicyFinding {
        code: MARKDOWN_VIOLATION_CODE,
        message: format!(
            "{} at {}:{}:{}: {}",
            markdown_rule_name(remaining.rule),
            path.text(),
            remaining.line,
            remaining.column,
            remaining.message
        ),
        path: Some(path.clone()),
        location: None,
        fix_available: false,
    };
}

/// What:
///  The findings of one candidate the linter ran on,
///  and the correction a fix applies.
/// Why:
///   An unchanged source reports only what is left;
///  a changed one reports the
///       rewrite first,
///  with a correction on a direct fix only (`canApplyPatches`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function candidateFindings(content, candidate, bytes, run, options, trigger): PolicyFinding[];
/// ```
fn candidate_findings(
    content: &mut ContentState,
    candidate: &Candidate,
    bytes: Rc<[u8]>,
    run: LintRun,
    options: &MarkdownAutofixOptions,
    trigger: Trigger,
) -> Vec<PolicyFinding> {
    let path: EventPath = EventPath::from_git_bytes(candidate.path.as_slice());
    let mut findings: Vec<PolicyFinding> = Vec::new();
    if run.fixed.as_slice() != &*bytes {
        let corrects: bool = trigger == Trigger::DirectFix;
        findings.push(PolicyFinding {
            code: MARKDOWN_AUTOFIX_CODE,
            message: format!(
                "monochromatic-lint --fix ({}) rewrites {}.",
                rule_list(options),
                path.text()
            ),
            path: Some(path.clone()),
            location: None,
            fix_available: corrects,
        });
        if corrects {
            content.propose(Correction {
                path: candidate.path.clone(),
                mode: candidate.mode,
                before: bytes,
                after: Rc::from(run.fixed),
            });
        }
    }
    for remaining in &run.remaining {
        findings.push(violation(&path, remaining));
    }
    return findings;
}

/// What:
///  Check every candidate of the lifecycle.
///  `<F: RepositoryFacts>` accepts any facts
///       provider;
///  `&mut MarkdownState` lends the policy's linter and patterns.
/// Why:
///   See the module comment.
///  A policy with no rule selected has nothing to run;
///  the
///       first candidate the linter cannot check ends the policy with that candidate named.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function checkMarkdown(content, lifecycle, facts, state, trigger): Promise<PolicyOutcome>;
/// ```
pub fn check_markdown<F: RepositoryFacts>(
    content: &mut ContentState,
    lifecycle: &LifecycleContent,
    facts: &mut F,
    state: &mut MarkdownState,
    trigger: Trigger,
) -> PolicyOutcome {
    let version: Rc<CandidateVersion> = match content.version(lifecycle, facts) {
        Ok(Some(found)) => found,
        Ok(None) => return PolicyOutcome::Findings(Vec::new()),
        Err(outcome) => return outcome,
    };
    let mut findings: Vec<PolicyFinding> = Vec::new();
    if state.options.rules.is_empty() {
        return PolicyOutcome::Findings(findings);
    }
    for candidate in version.candidates() {
        if !is_eligible(candidate) {
            continue;
        }
        match state.excludes(candidate.path.as_slice()) {
            Ok(true) => continue,
            Ok(false) => {}
            Err(reason) => return unchecked(None, reason.as_str()),
        }
        let bytes: Rc<[u8]> = match content.bytes(candidate) {
            Ok(read) => read,
            Err(outcome) => return outcome,
        };
        // A candidate that is not UTF-8 is not text the linter can read; it is skipped.
        if std::str::from_utf8(&bytes).is_err() {
            continue;
        }
        // The facts provider remembers the location, so asking again starts no process.
        let top: PathBuf = match top_level(facts) {
            Ok(found) => found,
            Err(outcome) => return outcome,
        };
        let request: LintRequest<'_> = LintRequest {
            top_level: top.as_path(),
            options: &state.options,
            path: candidate.path.as_slice(),
            source: &bytes,
        };
        let run: LintRun = match state.linter.lint(&request) {
            Ok(usable) => usable,
            Err(reason) => return unchecked(Some(candidate.path.as_slice()), reason.as_str()),
        };
        findings.extend(candidate_findings(
            content,
            candidate,
            bytes,
            run,
            &state.options,
            trigger,
        ));
    }
    return PolicyOutcome::Findings(findings);
}

/// Eligibility,
///  exclusion,
///  finding and correction controls stay out of the release executable.
#[cfg(test)]
#[path = "policy_markdown_tests.rs"]
mod tests;
