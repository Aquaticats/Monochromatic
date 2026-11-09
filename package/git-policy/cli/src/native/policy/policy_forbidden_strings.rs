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
use crate::candidate_known_content::KnownContent;
/// Import the immutable version type.
use crate::candidate_version::CandidateVersion;
/// Import the validated options of this policy.
use crate::config_schema::{ForbiddenStringsOptions, PolicyConfig};
/// Import the debug and warning diagnostics on standard error.
use crate::diagnostic_log::{debug, warn};
/// Import the engine-failure codes.
use crate::diagnostics::EngineFailureCode;
/// Import the event form of a pathname.
use crate::event_path::EventPath;
/// Import the lifecycle's candidates.
use crate::policy_content::{ContentState, LifecycleContent};
/// Import the finding and outcome types of a check.
use crate::policy_engine::{PolicyFinding, PolicyOutcome};
/// Import the facts interface for the repository's top level.
use crate::repository_facts::RepositoryFacts;
/// Import the repository location answer.
use crate::repository_location::RepositoryLocation;
/// Import the scanner adapter and its failure.
use crate::scanner_adapter::{
    CandidateScanner, RulesSource, ScannerError, ScannerFailure, compiled_rules_artifact_exists,
};
/// Import the code of each scanner and scan failure.
use crate::scanner_failure_code::{
    finding_failure_code, scan_run_failure_code, scanner_failure_code,
};
/// Import the rules-file precedence and which candidates are scanned.
use crate::scanner_selection::{RULES_VARIABLE, is_scannable, rules_candidate_paths, rules_source};
/// Import the worktree's top level from Git's answer.
use crate::worktree_identity::{git_directory, worktree_root};
/// The scanner library's per-candidate result and per-match finding,
///  and what a load may write into the shared compiled-rules cache.
use forbidden_strings::{CacheAccess, CandidateScan, RulesListError, ScanFinding};
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
/// `Path` is a borrowed filesystem path of raw OS bytes.
use std::path::{Path, PathBuf};
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
///  Who adds the compiled rules to the shared cache when a scan needs rules no cache holds.
/// Why:
///   The compiled artifact is derived data a machine pays for once per rules text and release,
///  and writing it costs the writer a device stall:
///  6.7 s and 3.0 s of `fsync` in one measured commit.
///  A resident service that already runs for this worktree compiles and publishes it on its own time,
///  so the command that needs the rules takes no write at all;
///  a machine with no service running publishes from this process, as every command did before.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CachePublisher = { kind: 'service'; gitDir: string; repository: string } | { kind: 'this-process' };
/// ```
pub enum CachePublisher<'a> {
    /// The commit service of this worktree, asked to compile the rules and never waited for.
    Service {
        /// The worktree's own Git directory,
        ///  which names the service instance and its socket.
        git_dir: &'a std::path::Path,
        /// The worktree's top level,
        ///  whose configuration names the rules files.
        repository: &'a std::path::Path,
    },
    /// This process, which writes the artifact as the scanner does by default.
    ThisProcess,
}

/// What:
///  The access a load takes, from whether the cache holds the text and whether someone else publishes.
///  `bool` is the answer to each.
/// Why:
///   A cache that already holds the compiled text costs no write either way,
///  and only a miss is the question here;
///  the two answers are kept apart so a control can pin each one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// cacheAccess(cached, publishedElsewhere) // cached || !publishedElsewhere ? readWrite : readOnly
/// ```
pub fn cache_access(cached: bool, published_elsewhere: bool) -> CacheAccess {
    if cached || !published_elsewhere {
        return CacheAccess::ReadWrite;
    }
    return CacheAccess::ReadOnly;
}

/// What:
///  The access this load takes.
/// Why:
///   The service is asked only when the cache holds nothing,
///  which is the only case a write question exists;
///  a hit is read and repaired here without a process walk or a message.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// publisher.access(rules) // cached ? readWrite : askTheService(rules)
/// ```
impl CachePublisher<'_> {
    /// The access a load of `rules` takes.
    fn access(&self, rules: &RulesSource) -> CacheAccess {
        let Self::Service { git_dir, repository } = self else {
            return CacheAccess::ReadWrite;
        };
        if compiled_rules_artifact_exists(rules) {
            return CacheAccess::ReadWrite;
        }
        return cache_access(
            false,
            crate::service_warm_request::ask_running_service_to_warm(git_dir, repository),
        );
    }
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
        access: CacheAccess,
    ) -> Result<&CandidateScanner, PolicyOutcome> {
        if self.loaded.is_none() {
            self.loaded = Some(CandidateScanner::load_writing_cache(rules, builtin_rules, access));
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
                    "monochromatic-git-policy-cli did not load the forbidden-strings rules; this is a defect in monochromatic-git-policy-cli.",
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
                "monochromatic-git-policy-cli could not find the repository's top level to resolve the forbidden-strings rules file.",
            ));
        }
    }
}

/// What:
///  Whether the service of this worktree would compile the very rules list this command selected.
///  `&ScannerSettings` borrows the options and the variable;
///  `&Path` is the repository's top level;
///  `&RulesSource` is the selection.
/// Why:
///   The service resolves the configuration's files only,
///  because its environment carries no `FORBIDDEN_STRINGS_RULES`;
///  a list the variable named is therefore another text,
///  and its artifact is not the one this command reads.
/// A configuration that names files wins over the variable,
///  so a repository with both still leaves the list to the service,
///  which is the usual setup here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// serviceReadsTheSameRules(settings, root, selected)
/// ```
fn service_reads_the_same_rules(
    settings: &ScannerSettings,
    repository_root: &Path,
    selected: &RulesSource,
) -> bool {
    let configured = rules_source(settings.options.rules_files.as_deref(), None, repository_root);
    return configured.is_ok_and(|without_variable| return without_variable == *selected);
}

/// What:
///  The worktree's own Git directory,
///  or nothing when the command runs outside one.
/// Why:
///   The Git directory names the commit service instance of this worktree,
///  which is the process asked to publish the compiled rules;
///  a command outside a worktree has none to ask and publishes itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function serviceGitDirectory(facts): Promise<string | undefined>;
/// ```
fn service_git_directory<F: RepositoryFacts>(facts: &mut F) -> Option<PathBuf> {
    let location: RepositoryLocation = facts.location().ok()?;
    return git_directory(&location.identity).map(Path::to_path_buf);
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
                    "monochromatic-git-policy-cli could not load the forbidden-strings rules, so no file was scanned: \
                     the value of {RULES_VARIABLE} is not a valid rules list: {error}."
                )
                .as_str(),
            );
        }
    };
    // What: the publisher decides the load's access, and the loader records what the scanner said.
    // Why:  The service reads this repository's configuration and its own environment carries no
    //       `FORBIDDEN_STRINGS_RULES`, so a list the variable named is not the list the service would compile:
    //       leaving that text to the service publishes another text's artifact and compiles this one on every command.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const publisher = serviceReadsTheSameRules(settings, root, rules) ? servicePublisher(gitDir, root) : thisProcess;
    // ```
    let service_git_dir: Option<PathBuf> = service_git_directory(facts);
    let publisher: CachePublisher<'_> = match (service_reads_the_same_rules(settings, root.as_path(), &rules), service_git_dir.as_deref()) {
        (true, Some(git_dir)) => CachePublisher::Service { git_dir, repository: root.as_path() },
        _ => CachePublisher::ThisProcess,
    };
    let access: CacheAccess = publisher.access(&rules);
    let loaded: &CandidateScanner = match scanner.load(&rules, settings.options.builtin_rules, access) {
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
