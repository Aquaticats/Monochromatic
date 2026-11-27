//! What:
//!  What the standards-review run was told:
//! the variables a person sets,
//! the log and the credential they name,
//! and where and how long the one call runs.
//! Why:
//!  Production passes the decisions endpoint and the script's quiet window,
//! and a control passes a loopback URL and a short window,
//! so the options are a value the caller hands over and never a read of this process's environment.
//! The log variable is read before the shared log's path is used,
//! and the second credential variable is read only when the first is unset,
//! as the script's `??` did.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // optionsFromEnvironment(commonDir, variables): { logPath, apiKey, endpoint, quietWindow, callCeiling, trustedPem }
//! ```

/// The quiet window the one call starts with,
/// which production passes unchanged.
use super::standards_review_client::SCRIPT_QUIET_WINDOW;
/// Which commits get an entry,
/// and the capped input the question is built from.
use super::standards_review_input::{
    ChangedPath,
    LandedCommit,
    shared_log_path,
};
/// The decisions endpoint.
use super::standards_review_request::ENDPOINT;
/// Filesystem paths.
use std::path::{
    Path,
    PathBuf,
};
/// How long a call may stay silent,
/// and the longest it may run in all.
use std::time::Duration;

/// The environment variable that points the log at a disposable fixture.
pub const LOG_VARIABLE: &str = "SHADOW_REVIEW_LOG";

/// The variable that replays a commit other than `HEAD`.
pub const SHA_VARIABLE: &str = "SHADOW_REVIEW_SHA";

/// The variable that asks for the full review route,
/// which this port does not run.
pub const ALLOW_VARIABLE: &str = "SHADOW_REVIEW_ALLOW";

/// The credential variable,
/// and the stop-hook credential used when it is unset.
pub const KEY_VARIABLES: &[&str] = crate::service_review_key::KEY_VARIABLES;

/// The failure line written when no credential is set.
pub const NO_KEY_MESSAGE: &str =
    "no SHADOW_REVIEW_LLMGATEWAY_API_KEY or STOP_REMINDER_LLMGATEWAY_API_KEY in the environment";

/// What:
///  What the run was told:
/// the log,
/// the credential,
/// and where and how long to call.
/// Why:
///   Production passes `ENDPOINT` and `SCRIPT_QUIET_WINDOW`;
/// tests pass a loopback URL and a short quiet window.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ReviewOptions = { logPath: string; apiKey?: string; endpoint: string; quietWindow: number };
/// ```
#[derive(Clone, Debug)]
pub struct ReviewOptions {
    /// Where entries are appended.
    pub log_path: PathBuf,
    /// The bearer credential;
    /// `None` writes the no-credential failure entry.
    pub api_key: Option<String>,
    /// The decisions endpoint.
    pub endpoint: String,
    /// How long the gateway call may stay silent before it is ended.
    pub quiet_window: Duration,
    /// The longest the gateway call runs in all,
    /// whatever it shows (`wait_ceilings.rs`,
    /// `REVIEW_CALL_CEILING`).
    pub call_ceiling: Duration,
    /// A PEM authority replacing the embedded roots (tests only).
    pub trusted_pem: Option<Vec<u8>>,
}

/// What:
///  Where the review's options read the variables of the environment.
///  A `trait` is a named set of methods that different types can each supply,
/// like a TypeScript `interface`.
/// Why:
///   The service's environment carries no secret:
/// the key comes in the wake message,
/// so the lookup holds the key it was given and
///  answers every other name from the environment.
/// A struct that holds the key implements this.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface EnvironmentVariables { variable(name: string): string | undefined }
/// ```
pub trait EnvironmentVariables {
    /// The value the name has,
    /// or nothing.
    fn variable(
        &self,
        name: &str,
    ) -> Option<String>;
}

/// What:
///  Where the review reads the patch and the files of a landed commit.
/// Why:
///   The read needs the Git context,
/// the directory,
/// the commit and the subject,
/// and tests fix the answer;
/// a struct that holds
///  them implements this.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface LandedCommitReader { read(files: ChangedPath[]): LandedCommit } // throws a reason
/// ```
pub trait LandedCommitReader {
    /// Read the commit as far as `files` (its code files) are concerned.
    fn read(
        &self,
        files: &[ChangedPath],
    ) -> Result<LandedCommit, String>;
}

/// What:
///  The options the script's environment variables give for a repository:
/// the live log under `common_dir` unless
///  `SHADOW_REVIEW_LOG` names another,
/// and the first credential variable that is set,
/// even to an empty text.
/// Why:
///   `??` in the script falls back only for an unset variable,
/// so an empty first key does not fall back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const logPath = process.env.SHADOW_REVIEW_LOG ?? `${commonDir}/monochromatic-git-policy-cli/standards-review-shadow.md`;
/// ```
pub fn options_from_environment(
    common_dir: &Path,
    variable: &dyn EnvironmentVariables,
) -> ReviewOptions {
    let log_path: PathBuf = match variable.variable(LOG_VARIABLE) {
        Some(configured) => PathBuf::from(configured),
        None => shared_log_path(common_dir),
    };
    // The second variable is read only when the first is unset, as `.or_else(...)` did.
    let api_key: Option<String> = match variable.variable(KEY_VARIABLES[0]) {
        Some(key) => Some(key),
        None => variable.variable(KEY_VARIABLES[1]),
    };
    return ReviewOptions {
        log_path,
        api_key,
        endpoint: String::from(ENDPOINT),
        quiet_window: SCRIPT_QUIET_WINDOW,
        call_ceiling: crate::wait_ceilings::REVIEW_CALL_CEILING,
        trusted_pem: None,
    };
}

/// What:
///  Whether the environment asks for the full-review route,
/// which this port leaves out.
/// Why:
///   The caller says so in a notice instead of silently running the cheap route.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// process.env.SHADOW_REVIEW_ALLOW === '1'
/// ```
pub fn full_review_requested(variable: &dyn Fn(&str) -> Option<String>) -> bool {
    return variable(ALLOW_VARIABLE).as_deref() == Some("1");
}
