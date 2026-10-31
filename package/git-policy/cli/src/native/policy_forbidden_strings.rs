//! What:
//!  The optional `security/forbidden-strings` policy:
//!  scan every candidate's exact
//!       bytes and pathname through the linked scanner and report each match,
//!  redacted.
//! Why:
//!  The installed wrapper wrote candidate bytes to temporary files,
//!  started the
//!      scanner executable and parsed its standard error.
//!  Here the same scanner is a
//!      library:
//!  rules are loaded once per invocation,
//!  only when a candidate can be
//!      scanned,
//!  and each finding carries the scanner's masked display path and an
//!      opaque rule name,
//!  never the matched text.
//!  A scan that could not complete ends
//!      the policy with the code of its cause,
//!  never as a clean result.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const findings = await scanCandidates({ builtinRules, repositoryRoot, candidates });
//! ```

/// Import the candidates whose content a parent commit's tree already holds.
use super::candidate_known_content::KnownContent;
/// Import the immutable version type.
use super::candidate_version::CandidateVersion;
/// Import the validated options of this policy.
use super::config_schema::{ForbiddenStringsOptions, PolicyConfig};
/// Import the debug and warning diagnostics on standard error.
use super::diagnostic_log::{debug, warn};
/// Import the engine-failure codes.
use super::diagnostics::EngineFailureCode;
/// Import the event form of a pathname.
use super::event_path::EventPath;
/// Import the lifecycle's candidates.
use super::policy_content::{ContentState, LifecycleContent};
/// Import the finding and outcome types of a check.
use super::policy_engine::{PolicyFinding, PolicyOutcome};
/// Import the facts interface for the repository's top level.
use super::repository_facts::RepositoryFacts;
/// Import the repository location answer.
use super::repository_location::RepositoryLocation;
/// Import the scanner adapter and its failure.
use super::scanner_adapter::{CandidateScanner, RulesSource, ScannerError, ScannerFailure};
/// Import the code of each scanner and scan failure.
use super::scanner_failure_code::{
    finding_failure_code, scan_run_failure_code, scanner_failure_code,
};
/// Import the rules-file precedence and which candidates are scanned.
use super::scanner_selection::{RULES_VARIABLE, is_scannable, rules_candidate_paths, rules_source};
/// Import the worktree's top level from Git's answer.
use super::worktree_identity::worktree_root;
/// The scanner library's per-candidate result and per-match finding.
use forbidden_strings::{CandidateScan, RulesListError, ScanFinding};
/// What:
///  `OsString` is owned operating-system text of raw OS bytes (sibling `String`
///       must be UTF-8).
/// Why:
///   The variable's value is a path and need not be UTF-8.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // string, but byte-preserving.
/// ```
use std::ffi::OsString;
/// `PathBuf` is an owned filesystem path.
use std::path::PathBuf;
/// `Rc<T>` is a shared,
///  read-only handle.
use std::rc::Rc;

/// The policy-local code of a match.
pub const FORBIDDEN_STRING_CODE: &str = "forbidden-string";

/// The engine-failure message of a scan a matcher could not complete.
pub const ENGINE_ERROR_MESSAGE: &str = "The forbidden-strings scanner reported a matcher failure while scanning a candidate, so the scan is incomplete.";

/// The engine-failure message of a pathname the line-based scanner cannot inspect.
pub const LINE_BREAK_MESSAGE: &str = "The forbidden-strings scanner cannot inspect a candidate pathname that contains a line break, so the scan is incomplete.";

/// What:
///  Where the rules come from:
///  the policy's options and the value of
///       `FORBIDDEN_STRINGS_RULES` in the invocation's environment.
/// Why:
///   The lifecycle reads both once;
///  the check resolves them against the repository
///       root only when it scans.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ScannerSettings = { options: ForbiddenStringsOptions; rulesVariable?: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScannerSettings {
    /// The policy's validated options;
    ///  defaults until the configuration is loaded.
    pub options: ForbiddenStringsOptions,
    /// The value of `FORBIDDEN_STRINGS_RULES`,
    ///  when the variable is set.
    pub rules_variable: Option<OsString>,
}

/// What:
///  Settings with the default options and no variable.
/// Why:
///   A lifecycle that loads no configuration and has no candidates never scans.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const defaultScannerSettings = (): ScannerSettings => ({ options: defaults.forbiddenStrings });
/// ```
pub fn default_scanner_settings() -> ScannerSettings {
    return ScannerSettings {
        options: PolicyConfig::defaults().forbidden_strings,
        rules_variable: None,
    };
}

/// What:
///  The scanner of one invocation,
///  loaded on first use,
///  or the remembered failure.
/// Why:
///   Rules are loaded once per invocation and only when a candidate needs scanning.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ScannerState { #loaded?: CandidateScanner | ScannerError }
/// ```
pub struct ScannerState {
    /// The loaded scanner,
    ///  or the failure of the one attempt to load it.
    loaded: Option<Result<CandidateScanner, ScannerError>>,
}

/// What:
///  A state that has loaded nothing.
/// Why:
///   Loading waits until a candidate can be scanned.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unloadedScanner = (): ScannerState => new ScannerState();
/// ```
pub fn unloaded_scanner() -> ScannerState {
    // `None` is the "absent" variant: nothing has been loaded.
    return ScannerState { loaded: None };
}

/// What:
///  The outcome of a check that could not finish.
///  `&str` is borrowed message text.
/// Why:
///   Every failure of this policy names its cause through its code.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const failed = (code, message): PolicyOutcome => ({ kind: 'failed', code, message });
/// ```
fn failed(code: EngineFailureCode, message: &str) -> PolicyOutcome {
    return PolicyOutcome::Failed {
        code,
        message: String::from(message),
        // `None`: a scan failure never names a pathname, which may hold a secret.
        path: None,
    };
}

/// What:
///  The failure a scan result carries,
///  or nothing when its findings are all matches.
///       `&CandidateScan` borrows one candidate's result.
/// Why:
///   The scanner reports a failed matcher and an uninspectable pathname inside an
///       otherwise successful result;
///  either makes the whole scan incomplete,
///  so it is
///       found before any match is reported.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function incompleteScan(scan: CandidateScan): PolicyOutcome | undefined;
/// ```
fn incomplete_scan(scan: &CandidateScan) -> Option<PolicyOutcome> {
    for finding in &scan.findings {
        // `if let Some(code) = ...` runs only for a finding that is a failure.
        if let Some(code) = finding_failure_code(finding) {
            let message: &str = if *finding == ScanFinding::PathnameLineBreak {
                LINE_BREAK_MESSAGE
            } else {
                ENGINE_ERROR_MESSAGE
            };
            return Some(failed(code, message));
        }
    }
    return None;
}

/// What:
///  The finding for one match in one scan,
///  or nothing for a finding that is not a match.
/// Why:
///   The words,
///  the code and the masked path are the installed wrapper's,
///  so a match
///       is reported the same way by both wrappers and never shows the matched text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const toFinding = (hit, displayPath): PolicyFinding => ({ code: 'forbidden-string', message, path: displayPath });
/// ```
fn match_finding(finding: &ScanFinding, display_path: &str) -> Option<PolicyFinding> {
    let message: String = match finding {
        ScanFinding::Content { line, rule } => {
            format!("Forbidden string matched at line {line} (rule {rule}).")
        }
        ScanFinding::Name { component, rule } => {
            format!("Forbidden string matched in pathname segment {component} (rule {rule}).")
        }
        ScanFinding::EngineError | ScanFinding::PathnameLineBreak => return None,
    };
    return Some(PolicyFinding {
        code: FORBIDDEN_STRING_CODE,
        message,
        // The scanner's masked display is text only: its bytes would undo the masking.
        path: Some(EventPath::display(display_path)),
        location: None,
        fix_available: false,
    });
}

/// What:
///  `impl ScannerState { ... }` attaches the one way to get the loaded scanner.
/// Why:
///   Loading and its remembered failure stay behind one owner.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ScannerState { load(rules, builtinRules): CandidateScanner }
/// ```
impl ScannerState {
    /// What:
    ///  The scanner,
    ///  loaded from `rules` on first use.
    ///  `Result<&CandidateScanner, ..>`
    ///       lends the loaded scanner,
    ///  or returns the outcome the policy reports.
    /// Why:
    ///   A second call in the same invocation reuses the loaded rules,
    ///  and a failure
    ///       is not retried.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// load(rules: RulesSource, builtinRules: boolean): CandidateScanner; // throws PolicyOutcome
    /// ```
    fn load(
        &mut self,
        rules: &RulesSource,
        builtin_rules: bool,
    ) -> Result<&CandidateScanner, PolicyOutcome> {
        if self.loaded.is_none() {
            self.loaded = Some(CandidateScanner::load(rules, builtin_rules));
            // An absent optional file is routine on a fresh clone, so it is logged for diagnosis, not warned.
            if let Some(Ok(scanner)) = &self.loaded {
                for skipped in scanner.skipped_optional_rules() {
                    debug(
                        "checkForbiddenStrings",
                        format!("optional rules file is absent; skipped: {}", skipped.display()).as_str(),
                    );
                }
            }
        }
        // `match` on the remembered attempt, lending the scanner when it loaded.
        match &self.loaded {
            Some(Ok(scanner)) => return Ok(scanner),
            Some(Err(error)) => {
                return Err(failed(
                    scanner_failure_code(error.failure),
                    error.message.as_str(),
                ));
            }
            None => {
                return Err(failed(
                    EngineFailureCode::PolicyIncomplete,
                    "cli-git did not load the forbidden-strings rules; this is a defect in cli-git.",
                ));
            }
        }
    }
}

/// What:
///  The repository's top level,
///  from Git's answer about where the command runs.
/// Why:
///   The rules file is named relative to it,
///  as the scanner child's working
///       directory was under the installed wrapper.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function repositoryRoot(facts): Promise<string>;
/// ```
fn repository_root<F: RepositoryFacts>(facts: &mut F) -> Result<PathBuf, PolicyOutcome> {
    let location: RepositoryLocation = match facts.location() {
        Ok(found) => found,
        Err(message) => {
            return Err(failed(
                EngineFailureCode::ContentUnavailable,
                message.as_str(),
            ));
        }
    };
    match worktree_root(&location.identity) {
        Some(root) => return Ok(root.to_path_buf()),
        None => {
            return Err(failed(
                EngineFailureCode::ContentUnavailable,
                "cli-git could not find the repository's top level to resolve the forbidden-strings rules file.",
            ));
        }
    }
}

/// What:
///  The candidates whose content a parent commit's tree already holds,
///  or none when that cannot be
///       determined.
///  `&mut ContentState` lends the prepared candidates;
///  `&[Vec<u8>]` are the rules files'
///       own pathnames.
/// Why:
///   Skipping content is only ever a saving.
///  When the parents cannot be listed,
///  every candidate is
///       scanned in full as it was before the rule existed,
///  and the warning says why,
///  so a Git failure
///       can make the policy slower or stricter and never more permissive.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function knownContentOrNone(content, lifecycle, rulesPath): Promise<KnownContent>;
/// ```
fn known_content_or_none(
    content: &mut ContentState,
    lifecycle: &LifecycleContent,
    rules_paths: &[Vec<u8>],
) -> KnownContent {
    // Only a lifecycle with candidates has a request to take the parents from.
    let LifecycleContent::Requested(request) = lifecycle else {
        return KnownContent::default();
    };
    match content.known_content(request, rules_paths) {
        Ok(found) => return found,
        Err(error) => {
            warn(
                "checkForbiddenStrings",
                format!(
                    "could not list the parent commits' files, so every candidate is scanned in full: {}",
                    error.message
                )
                .as_str(),
            );
            return KnownContent::default();
        }
    }
}

/// What:
///  Record each skipped candidate as a decision:
///  its masked pathname,
///  its blob and the parent commit that holds it.
///       `&[CandidateScan]` borrows the scan results,
///  whose identity is the candidate's position.
/// Why:
///   A skip changes what the policy reads,
///  so the inputs that justify it are kept for whoever checks a verdict later.
///       The pathname is the scanner's masked display,
///  never the raw pathname,
///  which may hold a secret.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function logKnownContent(scans: CandidateScan[], known: KnownContent): void;
/// ```
fn log_known_content(scans: &[CandidateScan], known: &KnownContent) {
    for scan in scans {
        if let Some(entry) = known.entry(scan.identity) {
            debug(
                "checkForbiddenStrings",
                entry.describe(scan.display_path.as_str()).as_str(),
            );
        }
    }
}

/// What:
///  Scan every candidate the scanner is given and report each match.
///       `&ScannerSettings` borrows where the rules come from;
///  `&mut ScannerState` lends
///       the invocation's scanner.
/// Why:
///   Deleted candidates,
///  the rule sources and the rules file itself are not scanned,
///       as the installed wrapper skips them.
///  When nothing is left,
///  no rules are loaded,
///       so a missing rules file stops nothing that would not have been scanned.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function checkForbiddenStrings(content, lifecycle, facts, settings, scanner): Promise<PolicyOutcome>;
/// ```
pub fn check_forbidden_strings<F: RepositoryFacts>(
    content: &mut ContentState,
    lifecycle: &LifecycleContent,
    facts: &mut F,
    settings: &ScannerSettings,
    scanner: &mut ScannerState,
) -> PolicyOutcome {
    let version: Rc<CandidateVersion> = match content.version(lifecycle, facts) {
        Ok(Some(found)) => found,
        // `Vec::new()` is the empty list: a lifecycle without candidates has nothing to report.
        Ok(None) => return PolicyOutcome::Findings(Vec::new()),
        Err(outcome) => return outcome,
    };
    let root: PathBuf = match repository_root(facts) {
        Ok(found) => found,
        Err(outcome) => return outcome,
    };
    // A malformed variable fails the policy only when something is scanned, like a missing rules file.
    let selected: Result<RulesSource, RulesListError> = rules_source(
        // `.as_deref()` lends the optional owned list as an optional borrowed slice.
        settings.options.rules_files.as_deref(),
        settings.rules_variable.as_deref(),
        root.as_path(),
    );
    let rules_paths: Vec<Vec<u8>> = match &selected {
        Ok(rules) => rules_candidate_paths(rules, root.as_path()),
        // `Vec::new()` is the empty list: no file is known to be a rules file.
        Err(_) => Vec::new(),
    };
    let mut any_scannable: bool = false;
    for candidate in version.candidates() {
        if is_scannable(candidate, rules_paths.as_slice()) {
            any_scannable = true;
            break;
        }
    }
    if !any_scannable {
        return PolicyOutcome::Findings(Vec::new());
    }
    let rules: RulesSource = match selected {
        Ok(found) => found,
        Err(error) => {
            return failed(
                scanner_failure_code(ScannerFailure::RulesNotLoaded),
                format!(
                    "cli-git could not load the forbidden-strings rules, so no file was scanned: \
                     the value of {RULES_VARIABLE} is not a valid rules list: {error}."
                )
                .as_str(),
            );
        }
    };
    let loaded: &CandidateScanner = match scanner.load(&rules, settings.options.builtin_rules) {
        Ok(found) => found,
        Err(outcome) => return outcome,
    };
    let known: KnownContent = known_content_or_none(content, lifecycle, rules_paths.as_slice());
    let scans: Vec<CandidateScan> =
        match content.scan_skipping(loaded, rules_paths.as_slice(), &known) {
            Ok(found) => found,
            // `.to_string()` renders the wrapped failure's message, which names no pathname.
            Err(error) => {
                return failed(scan_run_failure_code(&error), error.to_string().as_str());
            }
        };
    log_known_content(&scans, &known);
    // Every result is checked for a failure before any match is reported.
    for scan in &scans {
        if let Some(outcome) = incomplete_scan(scan) {
            return outcome;
        }
    }
    let mut findings: Vec<PolicyFinding> = Vec::new();
    for scan in &scans {
        for finding in &scan.findings {
            if let Some(reported) = match_finding(finding, scan.display_path.as_str()) {
                findings.push(reported);
            }
        }
    }
    return PolicyOutcome::Findings(findings);
}

/// Rules,
///  scan and redaction controls stay out of the release executable.
#[cfg(test)]
#[path = "policy_forbidden_strings_tests.rs"]
mod tests;
