//! What:
//!  The paths,
//!  owners and modes every handler check compares against,
//!  and what one look at a path answered.
//! Why:
//!  The route's privileged half is one root-owned executable,
//!  a configuration and two folders root owns,
//!  so the install paths,
//!  the mode bits and the owner are read once here and every check builds its
//!  finding from the same value.
//!  Presence,
//!  kind,
//!  owner and mode are four answers a person acts on differently,
//!  and a look this account was refused proves nothing about the path,
//!  so the look is kept apart from the judgement.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type PathState = { kind: 'file'; owner; mode } | { kind: 'directory'; owner; mode } | { kind: 'absent' } | { kind: 'wrong-kind'; found } | { kind: 'unreadable'; why };
//! type PathFacts = { path: string; state: PathState };
//! ```

/// The path a looked-at file or folder is named by.
use std::path::PathBuf;
/// What:
///  The one root-owned file root's modes run from,
///  read from the handler's own constant.
/// Why:
///  The owner answered that the release holds one program,
///  and the round that designed how root reaches its modes settled on one file
///  under the ZED history event's name
///  (`doc/decision/cli-git-zfs-root-modes-the-eleven-choices-of-the-one-program-design.md`,
///  choice 3).
///  A second copy of the path here would let a check and an install drift apart,
///  so the constant is the handler's and this is only the name the checks read.
pub const ROOT_MODE_EXECUTABLE_PATH: &str =
    crate::worktree_clone_mount_handler::handler_install_paths::ROOT_MODE_INSTALL_PATH;

/// What:
///  The ZED history event ZED starts the one file by.
/// Why:
///  ZED starts a `history_event-*` file and no other kind of entry,
///  and the event is a link to the file above,
///  so both spellings come from the handler's constants rather than a third
///  copy.
pub const ZEDLET_EVENT_PATH: &str =
    crate::worktree_clone_mount_handler::handler_install_paths::ZEDLET_INSTALL_PATH;

/// What:
///  The mode words the one executable must answer for this route's root work,
///  as the owner's answer of 2026-11-23 names them.
/// Why:
///  The record names the default sweep the ZED hook makes,
///  `boot-reconcile`,
///  `shutdown-unmount`,
///  `start-probe` and `adopt-worktree-parent`,
///  so a build that lost one of these answers a finding here.
///  The empty word is the sweep,
///  which is ZED's own start of the history event link with no argument.
///  The all-namespace holder scan,
///  the two modes the step 10 and step 19 answers add,
///  and the words that reach every mode are checked by the dispatch's own
///  controls,
///  so this list is the four the owner's answer of 2026-11-23 names.
pub const REQUIRED_ROOT_MODES: &[&str] = &[
    "",
    "adopt-worktree-parent",
    "boot-reconcile",
    "shutdown-unmount",
    "start-probe",
];

/// The owner every installed file and folder of the route must have.
pub const ROOT_OWNER: u32 = 0;

/// The mode the readiness directory must have,
/// so the wrapper can traverse it but not write it.
pub const READINESS_DIRECTORY_MODE: u32 = 0o755;

/// The mode the results directory must have,
/// from the receipts folder's own constant.
pub const RESULTS_DIRECTORY_MODE: u32 = crate::worktree_clone_mount_protocol::DIRECTORY_MODE;

/// The executable bit a handler build and its zedlet must have.
pub const EXECUTABLE_BIT: u32 = 0o111;

//region What a path is

/// What:
///  What one path the route depends on is,
///  as far as an account can see.
/// Why:
///  Presence,
///  kind,
///  owner and mode are four answers a person acts on differently,
///  and a look this account was refused proves nothing about the path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PathState = { kind: 'file'; owner; mode } | { kind: 'directory'; owner; mode } | { kind: 'absent' } | { kind: 'wrong-kind'; found } | { kind: 'unreadable'; why };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PathState {
    /// A regular file with its owner and mode.
    File {
        /// The user that owns it.
        owner: u32,
        /// Its permission bits,
        /// the file type's own bits excluded.
        mode: u32,
    },
    /// A directory with its owner and mode.
    Directory {
        /// The user that owns it.
        owner: u32,
        /// Its permission bits.
        mode: u32,
    },
    /// Nothing is at the path.
    Absent,
    /// Something else stands there.
    WrongKind {
        /// What the look found,
        /// in the system's words.
        found: String,
    },
    /// The look itself was refused.
    Unreadable {
        /// What the look answered.
        why: String,
    },
}

/// What:
///  One path and what the look at it answered.
/// Why:
///  A finding must name the file a person checks,
///  and the install paths are constants of the build rather than of the caller,
///  so the path travels with the state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PathFacts = { path: string; state: PathState };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PathFacts {
    /// The path the route installs or reads.
    pub path: PathBuf,
    /// What the look answered.
    pub state: PathState,
}

//endregion
