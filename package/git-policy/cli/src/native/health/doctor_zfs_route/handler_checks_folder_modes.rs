//! What:
//!  The readiness directory and the results directory,
//!  against root ownership and the modes the design fixes.
//! Why:
//!  A waiting caller reads a readiness record and a receipt through these
//!  directories,
//!  so a folder this account cannot traverse silently starves every waiter,
//!  one another account can write lets that account answer for root,
//!  and a missing folder sends a waiting caller nowhere.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = checkHandlerFolders(facts); // pass | finding('owner'|'mode'|'folder') | unknown
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
/// and the owner and modes the design fixes.
use super::handler_checks_paths_and_modes::{
    PathFacts,
    PathState,
    READINESS_DIRECTORY_MODE,
    RESULTS_DIRECTORY_MODE,
    ROOT_OWNER,
};
//region The root-owned folders

/// What:
///  The readiness directory and the results directory,
///  with the modes the design fixes.
/// Why:
///  The wrapper reads a readiness record and a receipt through these
///  directories,
///  so the directories must be traversable by the account and writable by
///  root alone,
///  and a missing one sends a waiting caller nowhere.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FolderFacts = { readiness: PathFacts; results: PathFacts };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FolderFacts {
    /// The directory one boot's readiness lives in.
    pub readiness: PathFacts,
    /// The directory the handler files receipts in.
    pub results: PathFacts,
}

/// One root-owned folder that must be traversable by the account.
fn folder_outcome(
    facts: &PathFacts,
    expected_mode: u32,
) -> Result<String, CheckOutcome> {
    let path: String = facts.path.display().to_string();
    return match &facts.state {
        PathState::Directory {
            owner,
            mode,
        } => {
            if *owner != ROOT_OWNER {
                return Err(finding(
                    "owner",
                    format!(
                        "{path} is owned by user {owner}, and only root writes the state a waiting caller reads"
                    ),
                ));
            }
            if *mode != expected_mode {
                return Err(finding(
                    "mode",
                    format!(
                        "{path} is mode {mode:04o}, and the design fixes {expected_mode:04o} so the account may traverse it and only root may write it"
                    ),
                ));
            }
            Ok(format!("{path} is root's, mode {mode:04o}"))
        }
        PathState::Absent => Err(finding(
            "folder",
            format!("{path} is not there, so no record or receipt can be read through it"),
        )),
        PathState::WrongKind {
            found,
        } => Err(finding(
            "folder",
            format!("{path} is not a directory ({found})"),
        )),
        PathState::File {
            ..
        } => Err(finding(
            "folder",
            format!("{path} is a file rather than a directory"),
        )),
        PathState::Unreadable {
            why,
        } => Err(unknown(format!(
            "{path} could not be looked at ({why}), so its ownership and mode are unproved"
        ))),
    };
}

/// What:
///  Whether both root-owned folders are there with the design's modes.
/// Why:
///  A folder this account cannot traverse silently starves every waiter,
///  and one another account can write lets that account answer for root.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = check_handler_folders(facts); // pass | finding('owner'|'mode'|'folder') | unknown
/// ```
pub fn check_handler_folders(facts: &FolderFacts) -> CheckOutcome {
    let readiness: String = match folder_outcome(&facts.readiness, READINESS_DIRECTORY_MODE) {
        Ok(detail) => detail,
        Err(outcome) => return outcome,
    };
    let results: String = match folder_outcome(&facts.results, RESULTS_DIRECTORY_MODE) {
        Ok(detail) => detail,
        Err(outcome) => return outcome,
    };
    return pass(format!("{readiness}; {results}"));
}

//endregion
