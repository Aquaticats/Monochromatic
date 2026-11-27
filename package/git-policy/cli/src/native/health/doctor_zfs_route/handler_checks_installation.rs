//! What:
//!  The two installed halves of one build against root ownership and an
//!  executable bit,
//!  and the root modes the one executable must answer.
//! Why:
//!  A zedlet that is not executable never delivers an event nobody notices,
//!  a file another account can rewrite is no handler,
//!  and a mode word the build's grammar lost is root work nothing can run,
//!  so each of those is a finding with the file or the word that fails.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = checkHandlerInstallation(facts); // pass | finding | unknown
//! const modes = checkRootModes(); // pass | finding('mode')
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
/// and the paths and modes the route fixes.
use super::handler_checks_paths_and_modes::{
    EXECUTABLE_BIT,
    PathFacts,
    PathState,
    REQUIRED_ROOT_MODES,
    ROOT_OWNER,
};
/// The root mode grammar the words are read by.
use crate::worktree_clone_mount_handler::handler_mode_arguments::parse;
/// The name ZED starts the sweep under,
/// which the sweep's finding names.
use crate::worktree_clone_mount_handler::root_mode_names::ZEDLET_START_NAME;
/// The arguments one mode word is read as.
use std::ffi::OsString;
//region The installation

/// What:
///  The two installed halves of one build:
///  the root-owned wrapper executable a caller and the boot unit start, and the
///  ZED history event that starts its sweep mode.
/// Why:
///  ZED runs the event file as an executable,
///  so both are files,
///  both are root's,
///  and both carry an executable bit;
///  a zedlet that is not executable never delivers an event nobody notices.
///  The executable's path is a placeholder until the design that moves the
///  handler's modes into the wrapper names it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HandlerInstallationFacts = { executable: PathFacts; zedlet: PathFacts };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HandlerInstallationFacts {
    /// The root-owned wrapper executable root's modes run as.
    pub executable: PathFacts,
    /// The ZED history event file that starts its sweep mode.
    pub zedlet: PathFacts,
}

/// One installed file against root ownership and its executable bit.
fn installed_file_outcome(facts: &PathFacts) -> Result<String, CheckOutcome> {
    let path: String = facts.path.display().to_string();
    return match &facts.state {
        PathState::File {
            owner,
            mode,
        } => {
            if *owner != ROOT_OWNER {
                return Err(finding(
                    "owner",
                    format!(
                        "{path} is owned by user {owner}, and every file the route installs is root's; a file another account can rewrite is no handler"
                    ),
                ));
            }
            if mode & EXECUTABLE_BIT == 0 {
                return Err(finding(
                    "mode",
                    format!(
                        "{path} is mode {mode:04o} without an executable bit, and this file is started as a program"
                    ),
                ));
            }
            Ok(format!("{path} is root's and executable"))
        }
        PathState::Absent => Err(finding(
            "installation",
            format!("{path} is not installed, and the route attaches nothing without it"),
        )),
        PathState::WrongKind {
            found,
        } => Err(finding(
            "installation",
            format!("{path} is not a regular file ({found})"),
        )),
        PathState::Directory {
            ..
        } => Err(finding(
            "installation",
            format!("{path} is a directory rather than a file"),
        )),
        PathState::Unreadable {
            why,
        } => Err(unknown(format!(
            "{path} could not be looked at ({why}), so its ownership and mode are unproved"
        ))),
    };
}

/// What:
///  Whether both installed halves are root's and executable.
/// Why:
///  One missing half disables a different part of the route:
///  without the executable no caller or boot unit starts a root mode,
///  and without the zedlet no ZFS event reaches the sweep.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = check_handler_installation(facts); // pass | finding('owner'|'mode'|'installation') | unknown
/// ```
pub fn check_handler_installation(facts: &HandlerInstallationFacts) -> CheckOutcome {
    let executable: String = match installed_file_outcome(&facts.executable) {
        Ok(detail) => detail,
        Err(outcome) => return outcome,
    };
    let zedlet: String = match installed_file_outcome(&facts.zedlet) {
        Ok(detail) => detail,
        Err(outcome) => return outcome,
    };
    return pass(format!("{executable}; {zedlet}"));
}

/// What:
///  Whether the one executable answers every root mode this route needs.
/// Why:
///  A mode that is not in the build's grammar is a mode the root-owned file
///  cannot run,
///  so a build that lost the sweep,
///  the boot reconciliation,
///  the shutdown unmount or the adoption answers a finding naming the word,
///  and the check reads the one grammar rather than a second list a person must
///  keep in step.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = check_root_modes(); // pass | finding('mode')
/// ```
pub fn check_root_modes() -> CheckOutcome {
    return check_modes(REQUIRED_ROOT_MODES);
}

/// What:
///  The same check over a list of mode words,
///  so a control can hand in a word this build does not answer.
/// Why:
///  A check that read the build's own constant could only ever pass,
///  and the control that proves it judges a word needs one that can fail.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = check_modes(['not-a-mode']); // finding('mode')
/// ```
pub fn check_modes(words: &[&str]) -> CheckOutcome {
    let mut missing: Vec<String> = Vec::new();
    for word in words {
        let arguments: Vec<OsString> = mode_arguments(word);
        if parse(arguments.as_slice()).is_err() {
            missing.push(if word.is_empty() { format!("the sweep, which ZED starts under the link name {ZEDLET_START_NAME} with no argument") } else { format!("`{word}`") });
        }
    }
    if !missing.is_empty() {
        return finding(
            "mode",
            format!(
                "the one executable answers no mode for {}; the route's root work cannot run",
                missing.join(", ")
            ),
        );
    }
    return pass(format!(
        "the one executable answers all {} root modes this route needs",
        words.len()
    ));
}

/// The arguments one required mode word is read as.
/// The words `monochromatic-git-policy-cli zfs-root` are stripped by the
/// dispatch before this grammar sees them, so a mode word and its options are
/// what the grammar is asked for.
fn mode_arguments(word: &str) -> Vec<OsString> {
    if word != "adopt-worktree-parent" {
        return if word.is_empty() {
            Vec::new()
        } else {
            vec![OsString::from(word)]
        };
    }
    return vec![
        OsString::from(word),
        OsString::from("--checkout"),
        OsString::from("/home/user/Monochromatic"),
        OsString::from("--device"),
        OsString::from("1"),
        OsString::from("--inode"),
        OsString::from("2"),
    ];
}

//endregion
