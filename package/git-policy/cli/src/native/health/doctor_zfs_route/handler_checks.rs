//! What:
//!  The one executable the route's root modes run as,
//!  its installation,
//!  its root-owned configuration,
//!  the root-owned readiness and result folders,
//!  ZED and service wiring,
//!  the latest readiness record and the clones whose requests no receipt
//!  answers.
//! Why:
//!  The project decided on 2026-11-23 that the ZFS route's root modes become
//!  modes of the one wrapper executable,
//!  `monochromatic-git-policy-cli`,
//!  and that the second program `mount-git-worktree-clone-datasets` stops being
//!  a program of its own,
//!  so these checks name one executable and the root modes it must answer,
//!  never a second binary under `/usr/libexec`
//!  (`doc/decision/cli-git-zfs-one-program-only-the-root-modes-join-the-wrapper.md`,
//!  written by the round that records the owner's answer;
//!  how root reaches the modes is being designed in its own round).
//!  The install paths here are placeholders named as such:
//!  one edit to the two constants and the facts changes them when that design
//!  lands.
//!  The route's privileged half is a root-owned executable ZED starts,
//!  a configuration and two result folders root owns,
//!  and a caller waits for a readiness record and a receipt rather than for a
//!  present executable
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "What doctor checks and prints";
//!  "Add and Git registration",
//!  step 1;
//!  "Requests,
//! results and ordering").
//!  `doctor` starts nothing and asks the service manager nothing,
//!  so active state is left unknown and named,
//!  while the installed files,
//!  the enablement links and the records on disk are read and judged.
//!  Every fact is handed in,
//!  so a control writes the files and modes it chose in its own directory.
//!
//!  The checks are split across siblings because this file is over the line
//!  limit:
//!  this one only names what the module holds,
//!  `handler_checks_paths_and_modes.rs` holds the paths,
//!  owners,
//!  modes and the look at one path,
//!  `handler_checks_installation.rs` judges the installed pair and the root
//!  modes,
//!  `handler_checks_configuration.rs` judges the configuration file,
//!  `handler_checks_folder_modes.rs` judges the two root-owned folders,
//!  `handler_checks_zed_wiring.rs` judges ZED's unit and enablement,
//!  `handler_checks_readiness.rs` judges the readiness record,
//!  and `handler_checks_unresolved_requests.rs` names the requests no receipt
//!  answers.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = check_handler_configuration({ path: PathFacts, parse: Ok }); // pass | finding('owner'|'mode'|'parse') | unknown
//! ```

/// One check's answer,
/// which a control reads through `super::`.
#[cfg(test)]
use super::check_outcome::CheckOutcome;
/// The handler's configuration file.
pub use super::handler_checks_configuration::{
    HandlerConfigurationFacts,
    check_handler_configuration,
};
/// The two root-owned folders.
pub use super::handler_checks_folder_modes::{
    FolderFacts,
    check_handler_folders,
};
/// The installed halves and the root modes the one executable must answer.
pub use super::handler_checks_installation::{
    HandlerInstallationFacts,
    check_handler_installation,
    check_modes,
    check_root_modes,
};
/// What one look at a path answered,
/// and the paths,
/// owners and modes every check compares against.
pub use super::handler_checks_paths_and_modes::{
    EXECUTABLE_BIT,
    PathFacts,
    PathState,
    READINESS_DIRECTORY_MODE,
    REQUIRED_ROOT_MODES,
    RESULTS_DIRECTORY_MODE,
    ROOT_MODE_EXECUTABLE_PATH,
    ROOT_OWNER,
    ZEDLET_EVENT_PATH,
};
/// The latest readiness record.
pub use super::handler_checks_readiness::{
    ReadinessFacts,
    check_readiness,
};
/// The clone requests no receipt of this boot answers.
pub use super::handler_checks_unresolved_requests::{
    RequestFacts,
    check_unresolved_requests,
};
/// ZED's unit,
/// its enablement and the other zedlets an enablement runs.
pub use super::handler_checks_zed_wiring::{
    HandlerWiringFacts,
    check_zed_wiring,
};

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "handler_checks_tests.rs"]
mod tests;
