//! What:
//!  The fixed words of the add's binding record:
//!  the header line,
//!  the eight phases,
//!  the seven settle reasons,
//!  the three request forms,
//!  and the keys in the order the grammar writes them.
//! Why:
//!  A word a reader does not know is either a later grammar or damage,
//!  and the design refuses both rather than reading a record under a meaning it does not have
//!  (`doc/planning/cli-git-zfs-add-binding-record-design.md`,
//!  "The binding",
//!  and the project's choice 9 of 2026-11-23).
//!  One list keeps the writer,
//!  the reader and the tests spelling the same bytes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Phase = 'claimed' | 'slot' | 'registered' | 'snapshotted' | 'cloned' | 'mounted' | 'filled' | 'settling';
//! const PHASES: Phase[] = ['claimed', 'slot', 'registered', 'snapshotted', 'cloned', 'mounted', 'filled', 'settling'];
//! ```

/// The first line of the binding record,
/// naming the grammar's version.
pub const BINDING_HEADER: &str =
    "# monochromatic-git-policy-cli zfs worktree clone binding, version 1";

/// The first line of an inventory record,
/// naming the grammar's version.
pub const INVENTORY_HEADER: &str =
    "# monochromatic-git-policy-cli zfs worktree clone inventory, version 1";

/// Every key the binding grammar writes,
/// in the order it writes them.
///
/// Why:
///  The design fixes the order,
///  so a record whose keys arrive in another order or twice is damage,
///  and one constant gives the writer and the reader that order.
pub const BINDING_KEYS_IN_ORDER: &[&str] = &[
    "claim",
    "nonce",
    "request",
    "phase",
    "settleReason",
    "checkoutDataset",
    "checkoutDatasetGuid",
    "repository",
    "repositoryIdentity",
    "slot",
    "slotIdentity",
    "registration",
    "registrationIdentity",
    "pool",
    "poolGuid",
    "originSnapshot",
    "originSnapshotGuid",
    "originSnapshotCreatetxg",
    "originDataset",
    "originDatasetGuid",
    "cloneDataset",
    "cloneGuid",
    "cloneCreatetxg",
    "pointer",
];

/// The keys a record carries from its first write on,
/// whatever its phase.
///
/// Why:
///  The design's A2 write records these before any mutation beyond the claim,
///  so a record missing one of them was not written by the add's own order.
pub const KEYS_EVERY_PHASE_HOLDS: &[&str] = &[
    "claim",
    "nonce",
    "request",
    "phase",
    "checkoutDataset",
    "checkoutDatasetGuid",
    "repository",
    "repositoryIdentity",
    "slot",
    "pool",
    "poolGuid",
    "cloneDataset",
];

/// The keys a record holds from one phase on,
/// besides [`KEYS_EVERY_PHASE_HOLDS`].
pub const KEYS_FROM_EACH_PHASE: &[(&str, &[&str])] = &[
    ("originSnapshot", &["originSnapshot"]),
    ("originDataset", &["originDataset", "originDatasetGuid"]),
    ("slot", &["slotIdentity"]),
    (
        "registered",
        &["registration", "registrationIdentity", "pointer"],
    ),
    (
        "snapshotted",
        &["originSnapshotGuid", "originSnapshotCreatetxg"],
    ),
    ("cloned", &["cloneGuid", "cloneCreatetxg"]),
];

/// What:
///  How far the add got,
///  in the design's eight words.
/// Why:
///  The phase names the object the writer last confirmed,
///  and a reader keys its resume on it.
///  A phase ahead of the live facts refuses,
///  and never leads the truth.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Phase = 'claimed' | 'slot' | 'registered' | 'snapshotted' | 'cloned' | 'mounted' | 'filled' | 'settling';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Phase {
    /// The claim and the binding exist and nothing has mutated.
    Claimed,
    /// The empty slot exists and its identity is recorded.
    Slot,
    /// Git's registration exists,
    /// is locked,
    /// and is recorded.
    Registered,
    /// The checkout snapshot exists and its inventory is published.
    Snapshotted,
    /// The clone exists and its initial inventory is published.
    Cloned,
    /// The clone is mounted at the slot and the mount is verified.
    Mounted,
    /// The fill's exchange is done and the worktree is verified.
    Filled,
    /// The add's own fallback is under way.
    Settling,
}

impl Phase {
    /// The word the record carries.
    pub fn word(self) -> &'static str {
        return match self {
            Phase::Claimed => "claimed",
            Phase::Slot => "slot",
            Phase::Registered => "registered",
            Phase::Snapshotted => "snapshotted",
            Phase::Cloned => "cloned",
            Phase::Mounted => "mounted",
            Phase::Filled => "filled",
            Phase::Settling => "settling",
        };
    }

    /// How far along the add the phase stands,
    /// which orders two phases without comparing their words.
    pub fn order(self) -> u8 {
        return match self {
            Phase::Claimed => 0,
            Phase::Slot => 1,
            Phase::Registered => 2,
            Phase::Snapshotted => 3,
            Phase::Cloned => 4,
            Phase::Mounted => 5,
            Phase::Filled => 6,
            Phase::Settling => 7,
        };
    }

    /// The phase a stored word names.
    pub fn parse(word: &str) -> Option<Phase> {
        return match word {
            "claimed" => Some(Phase::Claimed),
            "slot" => Some(Phase::Slot),
            "registered" => Some(Phase::Registered),
            "snapshotted" => Some(Phase::Snapshotted),
            "cloned" => Some(Phase::Cloned),
            "mounted" => Some(Phase::Mounted),
            "filled" => Some(Phase::Filled),
            "settling" => Some(Phase::Settling),
            _ => None,
        };
    }
}

/// What:
///  Which step's object stopped answering,
///  carried only beside `phase=settling`.
/// Why:
///  A resumed add reads the reason to know which of the fallback's steps to skip,
///  and a person reads the same kind beside the phase.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SettleReason = 'proof' | 'enospc' | 'mount' | 'clone' | 'snapshot' | 'destroy' | 'checkout';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SettleReason {
    /// The frozen proof refused the clone's bytes.
    Proof,
    /// A step answered `ENOSPC`.
    Enospc,
    /// The mount did not answer.
    Mount,
    /// The clone did not answer.
    Clone,
    /// The snapshot did not answer.
    Snapshot,
    /// A destroy was refused.
    Destroy,
    /// Git's own checkout could not finish.
    Checkout,
}

impl SettleReason {
    /// The word the record carries.
    pub fn word(self) -> &'static str {
        return match self {
            SettleReason::Proof => "proof",
            SettleReason::Enospc => "enospc",
            SettleReason::Mount => "mount",
            SettleReason::Clone => "clone",
            SettleReason::Snapshot => "snapshot",
            SettleReason::Destroy => "destroy",
            SettleReason::Checkout => "checkout",
        };
    }

    /// The reason a stored word names.
    pub fn parse(word: &str) -> Option<SettleReason> {
        return match word {
            "proof" => Some(SettleReason::Proof),
            "enospc" => Some(SettleReason::Enospc),
            "mount" => Some(SettleReason::Mount),
            "clone" => Some(SettleReason::Clone),
            "snapshot" => Some(SettleReason::Snapshot),
            "destroy" => Some(SettleReason::Destroy),
            "checkout" => Some(SettleReason::Checkout),
            _ => None,
        };
    }
}

/// What:
///  The word the add's request starts with,
///  which says how Git was asked for the commit-ish.
/// Why:
///  A resumed add compares its own request with the recorded one,
///  so a record written for another request refuses rather than continuing as if it were the same.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RequestForm = 'branch' | 'commit' | 'detach';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RequestForm {
    /// `git worktree add -b <branch>`.
    Branch,
    /// `git worktree add <commit-ish>`.
    Commit,
    /// `git worktree add --detach [<commit-ish>]`.
    Detach,
}

impl RequestForm {
    /// The word the record carries.
    pub fn word(self) -> &'static str {
        return match self {
            RequestForm::Branch => "branch",
            RequestForm::Commit => "commit",
            RequestForm::Detach => "detach",
        };
    }

    /// The form a stored word names.
    pub fn parse(word: &str) -> Option<RequestForm> {
        return match word {
            "branch" => Some(RequestForm::Branch),
            "commit" => Some(RequestForm::Commit),
            "detach" => Some(RequestForm::Detach),
            _ => None,
        };
    }
}

/// The value a request with no argument carries,
/// which only `--detach` may hold.
pub const NO_REQUEST_ARGUMENT: &str = "-";

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "binding_words_tests.rs"]
mod tests;
