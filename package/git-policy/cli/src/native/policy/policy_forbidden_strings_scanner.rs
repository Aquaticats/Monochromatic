//! What:
//!  The scanner one invocation uses:
//!  where its rules come from,
//!  the one attempt to load them and the failure it remembers,
//!  and who writes the compiled rules into the shared cache.
//! Why:
//!  Rules are loaded once per invocation and only when a candidate can be scanned;
//!  a cache that already holds the compiled text costs no write,
//!  and a resident service can publish the artifact on its own time,
//!       so the command that needs the rules takes no write at all.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const scanner = unloadedScanner(); const loaded = scanner.load(rules, builtinRules, access);
//! ```

/// Import the validated options of this policy.
use crate::config_schema::{
    ForbiddenStringsOptions,
    PolicyConfig,
};
/// Import the debug diagnostics on standard error.
use crate::diagnostic_log::debug;
/// Import the engine-failure codes.
use crate::diagnostics::EngineFailureCode;
/// Import the outcome type a load hands back on failure.
use crate::policy_engine::PolicyOutcome;
/// Import the failure outcome a load reports.
use crate::policy_forbidden_strings_scan_outcomes::failed;
/// Import the scanner adapter and its failure.
use crate::scanner_adapter::{
    CandidateScanner,
    RulesSource,
    ScannerError,
    compiled_rules_artifact_exists,
};
/// Import the code of each scanner failure.
use crate::scanner_failure_code::scanner_failure_code;
/// What a load may write into the shared compiled-rules cache.
use forbidden_strings::CacheAccess;
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
///  a machine with no service running publishes from this process,
/// as every command did before.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CachePublisher = { kind: 'service'; gitDir: string; repository: string } | { kind: 'this-process' };
/// ```
pub enum CachePublisher<'a> {
    /// The commit service of this worktree,
    /// asked to compile the rules and never waited for.
    Service {
        /// The worktree's own Git directory,
        ///  which names the service instance and its socket.
        git_dir: &'a std::path::Path,
        /// The worktree's top level,
        ///  whose configuration names the rules files.
        repository: &'a std::path::Path,
    },
    /// This process,
    /// which writes the artifact as the scanner does by default.
    ThisProcess,
}

/// What:
///  The access a load takes,
/// from whether the cache holds the text and whether someone else publishes.
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
pub fn cache_access(
    cached: bool,
    published_elsewhere: bool,
) -> CacheAccess {
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
    pub(crate) fn access(
        &self,
        rules: &RulesSource,
    ) -> CacheAccess {
        let Self::Service {
            git_dir,
            repository,
        } = self
        else {
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
    return ScannerState {
        loaded: None,
    };
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
    pub(crate) fn load(
        &mut self,
        rules: &RulesSource,
        builtin_rules: bool,
        access: CacheAccess,
    ) -> Result<&CandidateScanner, PolicyOutcome> {
        if self.loaded.is_none() {
            self.loaded = Some(CandidateScanner::load_writing_cache(
                rules,
                builtin_rules,
                access,
            ));
            // An absent optional file is routine on a fresh clone, so it is logged for diagnosis, not warned.
            if let Some(Ok(scanner)) = &self.loaded {
                for skipped in scanner.skipped_optional_rules() {
                    debug(
                        "checkForbiddenStrings",
                        format!(
                            "optional rules file is absent; skipped: {}",
                            skipped.display()
                        )
                        .as_str(),
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
