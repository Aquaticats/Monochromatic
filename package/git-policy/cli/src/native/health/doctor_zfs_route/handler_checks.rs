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
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = check_handler_configuration({ path: PathFacts, parse: Ok }); // pass | finding('owner'|'mode'|'parse') | unknown
//! ```

use super::check_outcome::{
    CheckOutcome,
    finding,
    pass,
    unknown,
};
use crate::worktree_clone_mount_handler::handler_mode_arguments::parse;
use crate::worktree_clone_mount_handler::root_mode_names::ZEDLET_START_NAME;
use crate::worktree_clone_mount_protocol::ReadyState;
use std::ffi::OsString;
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

//region ZED and the service wiring

/// What:
///  The ZFS event daemon's unit file and enablement link,
///  and the other zedlets an enablement of ZED would also run.
/// Why:
///  The handler's zedlet runs only inside ZED,
///  so ZED's own unit and its enablement are the route's wiring,
///  and a person turning ZED on must first see the other zedlets it would
///  start,
///  including syslog,
///  notification,
///  enclosure and scrub effects
///  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
///  "What doctor checks and prints").
///  Whether the unit is active is not asked of the service manager,
///  because `doctor` neither starts nor signals anything,
///  so that state is named as unread.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HandlerWiringFacts = { zed_unit: PathFacts; zed_enabled: boolean; other_zedlets: string[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HandlerWiringFacts {
    /// The ZFS event daemon's unit file.
    pub zed_unit: PathFacts,
    /// Whether a wants link enables it.
    pub zed_enabled: bool,
    /// Every other zedlet the installed directories hold.
    pub other_zedlets: Vec<String>,
}

/// What:
///  Whether ZED is installed and enabled,
///  and which other zedlets an enablement would start.
/// Why:
///  A route that needs an event and a daemon nobody enabled fails only when a
///  clone is made,
///  and the other zedlets are a person's decision before that enablement.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = check_zed_wiring(facts); // pass | finding('unit'|'enablement') | unknown
/// ```
pub fn check_zed_wiring(facts: &HandlerWiringFacts) -> CheckOutcome {
    let path: String = facts.zed_unit.path.display().to_string();
    return match &facts.zed_unit.state {
        PathState::Unreadable {
            why,
        } => unknown(format!(
            "{path} could not be looked at ({why}), so ZED's installation is unproved"
        )),
        PathState::Absent => finding(
            "unit",
            format!("{path} is not installed, so no ZFS event reaches the handler's zedlet"),
        ),
        PathState::WrongKind {
            found,
        } => finding("unit", format!("{path} is not a unit file ({found})")),
        PathState::File {
            ..
        }
        | PathState::Directory {
            ..
        } => {
            if !facts.zed_enabled {
                return finding(
                    "enablement",
                    format!(
                        "{path} is installed and no wants link enables it, so the route's zedlet never runs; enabling it would also run the other zedlets this report lists: {}",
                        others(&facts.other_zedlets)
                    ),
                );
            }
            pass(format!(
                "{path} is installed and enabled; whether it is active is not asked of the service manager, and the other zedlets an enablement runs are: {}",
                others(&facts.other_zedlets)
            ))
        }
    };
}

/// The other zedlets in one sentence,
/// or the fact that there are none.
fn others(zedlets: &[String]) -> String {
    if zedlets.is_empty() {
        return String::from("none besides the route's own");
    }
    return zedlets.join(", ");
}

//endregion

//region The readiness record

/// What:
///  The latest readiness record's fields,
///  or why none was read.
/// Why:
///  A record from another boot or another pool admits nothing,
///  and the checks need each state beside the boot the record names,
///  so the whole record travels rather than a boolean.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ReadinessFacts = { kind: 'read'; boot; pool; checkout; sweep } | { kind: 'absent'; why } | { kind: 'unreadable'; why };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReadinessFacts {
    /// The record parsed.
    Read {
        /// The boot the record belongs to.
        boot: String,
        /// Whether the pool was ready.
        pool: ReadyState,
        /// Whether the checkout was ready.
        checkout: ReadyState,
        /// What the sweep was doing,
        /// which gates no attach.
        sweep: String,
    },
    /// No record is filed,
    /// or it was filed for another boot.
    Absent {
        /// What the read answered.
        why: String,
    },
    /// The record could not be read.
    Unreadable {
        /// What the read answered.
        why: String,
    },
}

/// What:
///  Whether this boot's handler published a readiness record that admits an
///  attach.
/// Why:
///  A caller waits for this record rather than for an installed executable,
///  so a record from another boot or a pool or checkout that is not ready is a
///  finding,
///  and one state the handler could not read leaves the answer unknown
///  because the handler itself refused to guess.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = check_readiness({ kind: 'read', boot, pool, checkout, sweep }, liveBoot); // pass | finding('boot'|'pool'|'checkout') | unknown
/// ```
pub fn check_readiness(
    facts: &ReadinessFacts,
    live_boot: &str,
) -> CheckOutcome {
    return match facts {
        ReadinessFacts::Absent {
            why,
        } => finding(
            "readiness",
            format!(
                "no readiness record answers this boot ({why}), so a caller waits for nothing; the handler writes it on its first qualifying run"
            ),
        ),
        ReadinessFacts::Unreadable {
            why,
        } => unknown(format!("the readiness record could not be read ({why})")),
        ReadinessFacts::Read {
            boot,
            pool,
            checkout,
            sweep,
        } => {
            if boot != live_boot {
                return finding(
                    "boot",
                    format!(
                        "the readiness record belongs to boot {boot} and this boot is {live_boot}, so it admits no attach here"
                    ),
                );
            }
            if *pool == ReadyState::Unknown || *checkout == ReadyState::Unknown {
                let unknown_state: &str = if *pool == ReadyState::Unknown {
                    "pool"
                } else {
                    "checkout"
                };
                return unknown(format!(
                    "the handler could not read the {unknown_state}, and it records that as unknown rather than guessing; no attach is admitted on it"
                ));
            }
            if *pool != ReadyState::Ready {
                return finding(
                    "pool",
                    format!(
                        "the record says the pool is {}, and only a ready pool admits an attach",
                        pool.word()
                    ),
                );
            }
            if *checkout != ReadyState::Ready {
                return finding(
                    "checkout",
                    format!(
                        "the record says the checkout is {}, and only a ready checkout admits an attach",
                        checkout.word()
                    ),
                );
            }
            pass(format!(
                "this boot's record says the pool and checkout are ready (sweep: {sweep})"
            ))
        }
    };
}

//endregion

//region The unresolved requests

/// What:
///  One clone's durable request and the receipt that answers it,
///  where either is there.
/// Why:
///  A receipt names the nonce it answers and the boot it was filed in,
///  so a request with no matching receipt,
///  or one answered in another boot,
///  is the request nobody has done yet.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RequestFacts = { clone: string; intentNonce?: string; receiptNonce?: string; receiptBoot?: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RequestFacts {
    /// The clone's dataset name.
    pub clone: String,
    /// The nonce its local request names,
    /// or nothing when it asks for no work.
    pub intent_nonce: Option<String>,
    /// The nonce the newest receipt answers,
    /// or nothing when there is no receipt.
    pub receipt_nonce: Option<String>,
    /// The boot the newest receipt belongs to.
    pub receipt_boot: Option<String>,
}

/// What:
///  Every clone whose request no receipt of this boot answers.
/// Why:
///  A clone that asked to be mounted or unmounted and got no answer is the one
///  state a person can act on,
///  while a request answered in another boot is not answered here,
///  and the report names each clone rather than a count.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = check_unresolved_requests(requests, liveBoot); // pass | finding('request')
/// ```
pub fn check_unresolved_requests(
    requests: &[RequestFacts],
    live_boot: &str,
) -> CheckOutcome {
    let mut unresolved: Vec<String> = Vec::new();
    let mut asked: usize = 0;
    for request in requests {
        let Some(nonce) = &request.intent_nonce else {
            continue;
        };
        asked += 1;
        let answered: bool = request.receipt_nonce.as_ref() == Some(nonce)
            && request.receipt_boot.as_deref() == Some(live_boot);
        if !answered {
            unresolved.push(format!("`{}` asks under nonce {nonce}", request.clone));
        }
    }
    if !unresolved.is_empty() {
        return finding(
            "request",
            format!(
                "{} clone request(s) have no receipt from this boot: {}",
                unresolved.len(),
                unresolved.join(", ")
            ),
        );
    }
    return pass(format!(
        "{asked} clone request(s) are each answered by a receipt of this boot"
    ));
}

//endregion

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "handler_checks_tests.rs"]
mod tests;
