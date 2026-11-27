//! What:
//!  The handler's configuration file:
//!  whether it is root's,
//!  whether only root may write it,
//!  and whether its text parsed.
//! Why:
//!  The dataset names a root program acts on come from this file,
//!  so an account that can write it chooses where root acts,
//!  and a file that does not parse leaves every name to the design's fallback
//!  without saying so.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = checkHandlerConfiguration(facts); // pass | finding('owner'|'mode'|'parse') | unknown
//! ```

/// One check's answer,
/// and the three constructors every check builds it from.
use super::check_outcome::{
    CheckOutcome,
    finding,
    pass,
    unknown,
};
/// What one look at a path answered,
/// and the owner every installed file has.
use super::handler_checks_paths_and_modes::{
    PathFacts,
    PathState,
    ROOT_OWNER,
};
//region The configuration

/// What:
///  The handler's configuration file and whether its text parsed.
/// Why:
///  A configuration another account can rewrite steers a root program,
///  and one that does not parse names no dataset at all,
///  so ownership and the parse answer are two separate findings.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HandlerConfigurationFacts = { file: PathFacts; parse_error?: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HandlerConfigurationFacts {
    /// The configuration file.
    pub file: PathFacts,
    /// Why the text did not parse,
    /// or nothing when it did.
    pub parse_error: Option<String>,
}

/// What:
///  Whether the handler's configuration is a root-owned file this build reads.
/// Why:
///  The dataset names a root program acts on come from this file,
///  so an account that can write it chooses where root acts,
///  and a file that does not parse leaves every name to the design's fallback
///  without saying so.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = check_handler_configuration(facts); // pass | finding('owner'|'mode'|'parse') | unknown
/// ```
pub fn check_handler_configuration(facts: &HandlerConfigurationFacts) -> CheckOutcome {
    let path: String = facts.file.path.display().to_string();
    return match &facts.file.state {
        PathState::File {
            owner,
            mode,
        } => {
            if *owner != ROOT_OWNER {
                return finding(
                    "owner",
                    format!(
                        "{path} is owned by user {owner}, and a configuration only root writes steers a root program"
                    ),
                );
            }
            if mode & 0o022 != 0 {
                return finding(
                    "mode",
                    format!(
                        "{path} is mode {mode:04o}, which the file's owner or group could write; only root may write it"
                    ),
                );
            }
            return match &facts.parse_error {
                Some(why) => finding(
                    "parse",
                    format!("{path} does not parse ({why}), so its dataset names are not read"),
                ),
                None => pass(format!("{path} is root's, mode {mode:04o}, and parses")),
            };
        }
        PathState::Absent => pass(format!(
            "{path} is not installed, and the design's fallback finds the checkout's datasets from the mount table"
        )),
        PathState::WrongKind {
            found,
        } => finding(
            "configuration",
            format!("{path} is not a regular file ({found})"),
        ),
        PathState::Directory {
            ..
        } => finding(
            "configuration",
            format!("{path} is a directory rather than a file"),
        ),
        PathState::Unreadable {
            why,
        } => unknown(format!(
            "{path} could not be looked at ({why}), so its ownership and text are unproved"
        )),
    };
}

//endregion
