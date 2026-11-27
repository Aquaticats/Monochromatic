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

/// Import the candidates whose content a parent commit's tree already holds.
use crate::candidate_known_content::KnownContent;
/// Import the immutable version type.
use crate::candidate_version::CandidateVersion;
/// Import the warning diagnostics on standard error.
use crate::diagnostic_log::warn;
/// Import the engine-failure codes.
use crate::diagnostics::EngineFailureCode;
/// Import the lifecycle's candidates.
use crate::policy_content::{
    ContentState,
    LifecycleContent,
};
/// Import the finding and outcome types of a check.
use crate::policy_engine::{
    PolicyFinding,
    PolicyOutcome,
};
/// Import what one candidate's scan becomes:
///  the failure outcome,
///  each match's finding,
///  and the record of a candidate a parent commit already holds.
use crate::policy_forbidden_strings_scan_outcomes::{
    failed,
    incomplete_scan,
    log_known_content,
    match_finding,
};
/// Import the facts interface for the repository's top level.
use crate::repository_facts::RepositoryFacts;
/// Import the repository location answer.
use crate::repository_location::RepositoryLocation;
/// Import the scanner adapter and its failure.
use crate::scanner_adapter::{
    CandidateScanner,
    RulesSource,
    ScannerFailure,
};
/// Import the code of each scan failure.
use crate::scanner_failure_code::{
    scan_run_failure_code,
    scanner_failure_code,
};
/// Import the rules-file precedence and which candidates are scanned.
use crate::scanner_selection::{
    RULES_VARIABLE,
    is_scannable,
    rules_candidate_paths,
    rules_source,
};
/// Import the worktree's top level from Git's answer.
use crate::worktree_identity::{
    git_directory,
    worktree_root,
};
/// The scanner library's per-candidate result,
///  its rules-list failure,
///  and what a load may write into the shared compiled-rules cache.
use forbidden_strings::{
    CacheAccess,
    CandidateScan,
    RulesListError,
};
/// `Path`/`PathBuf` are borrowed/owned filesystem paths of raw OS bytes.
use std::path::{
    Path,
    PathBuf,
};
/// `Rc<T>` is a shared,
///  read-only handle.
use std::rc::Rc;

/// The codes and messages of a match and of an incomplete scan,
///  which the sibling file defines and every reader of this policy names.
pub use crate::policy_forbidden_strings_scan_outcomes::{
    ENGINE_ERROR_MESSAGE,
    FORBIDDEN_STRING_CODE,
    LINE_BREAK_MESSAGE,
};
/// The scanner's settings and state,
///  which the sibling file defines and the check hands on.
pub use crate::policy_forbidden_strings_scanner::{
    CachePublisher,
    ScannerSettings,
    ScannerState,
    cache_access,
    default_scanner_settings,
    unloaded_scanner,
};

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
    let configured: Result<RulesSource, RulesListError> = rules_source(
        settings.options.rules_files.as_deref(),
        None,
        repository_root,
    );
    let Ok(without_variable) = configured else {
        return false;
    };
    return without_variable == *selected;
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
    let publisher: CachePublisher<'_> = match (
        service_reads_the_same_rules(settings, root.as_path(), &rules),
        service_git_dir.as_deref(),
    ) {
        (true, Some(git_dir)) => CachePublisher::Service {
            git_dir,
            repository: root.as_path(),
        },
        _ => CachePublisher::ThisProcess,
    };
    let access: CacheAccess = publisher.access(&rules);
    let loaded: &CandidateScanner =
        match scanner.load(&rules, settings.options.builtin_rules, access) {
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
