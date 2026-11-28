//! What:
//!  What one `doctor` check answered:
//!  a pass with what it read,
//!  a finding naming the failing field or path component,
//!  or unknown with the reason the check could not read or prove the fact;
//!  plus the stable name of every check and the line a report prints.
//! Why:
//!  The step says a check it cannot read or prove is reported as unknown,
//!  which keeps three states apart:
//!  a fact that holds,
//!  a fact that is wrong,
//!  and a fact nobody read.
//!  A finding names the field so a person learns what to look at,
//!  and setup findings go to standard error without changing `doctor`'s exit
//!  status
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "What doctor checks and prints").
//!  The names are stable words rather than numbers,
//!  so a log line from one release still says which check spoke in another.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type CheckOutcome = { kind: 'pass'; detail: string } | { kind: 'finding'; field: string; detail: string } | { kind: 'unknown'; reason: string };
//! const line = reportLine({ check: 'checkout-mount', outcome }); // 'zfs: checkout-mount: pass: ...'
//! ```

/// What:
///  The checks plan step 18 adds,
///  named so a report line and a control can both point at one check.
/// Why:
///  A check's name is its stable handle across releases,
///  and the enum keeps a caller from printing a name no check answers to.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Check = 'device' | 'abi' | 'pool' | ...;
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Check {
    /// `/dev/zfs` is there and the account may open it.
    Device,
    /// The loaded module's layout matches the request contract.
    Abi,
    /// The pool named by the checkout dataset answers.
    Pool,
    /// The checkout dataset is a mounted filesystem at the checkout path.
    CheckoutDataset,
    /// The clone container is a legacy filesystem with its decided flags.
    ContainerDataset,
    /// The checkout dataset's snapshot directory is hidden from a name.
    CheckoutSnapdir,
    /// The clone container's snapshot directory is hidden from a name,
    /// and the clones inherit it.
    ContainerSnapdir,
    /// Every dataset's encryption key is loaded where it has one.
    Key,
    /// Every clone carries the properties this route attaches it with.
    CloneProperties,
    /// The checkout's own mount is in the caller's table.
    CheckoutMount,
    /// The checkout's mount is not shared with another namespace.
    CheckoutPropagation,
    /// Every dataset mounted under the checkout path is reachable by path.
    NestedReachability,
    /// The recorded parent identity matches the live `.claude/worktrees`.
    ParentIdentity,
    /// A recorded slot identity matches its live slot.
    SlotIdentity,
    /// The store folder's entry count and the shape of every entry.
    StoreEntries,
    /// The two staging forms a killed writer leaves.
    StagingForms,
    /// A dot entry that matches neither staging form.
    UnknownDotEntry,
    /// The account's effective delegation and the forbidden grants' absence.
    Delegation,
    /// The one executable's installation and the ZED event that starts it.
    HandlerInstallation,
    /// The root modes the one executable answers.
    RootModes,
    /// The handler's root-owned configuration.
    HandlerConfiguration,
    /// The readiness,
    /// state and result folders.
    HandlerFolders,
    /// ZED and the boot unit the route needs.
    ZedWiring,
    /// The latest readiness record against this boot.
    Readiness,
    /// A clone whose request no receipt answers.
    UnresolvedRequests,
    /// The staging folder under the account's cache.
    StagingFolder,
    /// The staging bind's propagation.
    StagingBindPropagation,
    /// What one managed slot is:
    /// the clone mounted at it,
    /// unmounted from it,
    /// staged under the cache,
    /// or left by an add or a removal that stopped.
    CloneState,
    /// The Git registration one slot's record names.
    Registration,
    /// The managed Git lock one slot's registration holds.
    ManagedLock,
    /// The binding record one slot's add wrote.
    BindingRecord,
    /// The destruction plan one clone's removal left.
    RemovalPlan,
    /// The clone's own snapshots beside the plan's states.
    CloneSnapshots,
    /// The origin each clone names and its deferred-destroy mark.
    CloneOrigin,
    /// What one clone and its snapshots retain.
    CloneRetention,
    /// The blocked-removal state of one clone.
    BlockedRemoval,
    /// The namespaces that hold a clone.
    Holders,
    /// Step 17's cancel command.
    CancelCommand,
    /// Step 19's checkout boot protection and its immutable covered entry.
    BootProtection,
    /// Step 19's boot and shutdown units.
    ServiceUnit,
    /// Step 19's backup coverage.
    BackupCoverage,
    /// Step 20's Sanoid coverage and exclusions.
    Sanoid,
}

impl Check {
    /// The check's stable name in a report line.
    ///
    /// # Examples
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const name = checkName(Check.CheckoutMount); // 'checkout-mount'
    /// ```
    pub fn name(self) -> &'static str {
        return match self {
            Check::Device => "device",
            Check::Abi => "abi",
            Check::Pool => "pool",
            Check::CheckoutDataset => "checkout-dataset",
            Check::ContainerDataset => "container-dataset",
            Check::CheckoutSnapdir => "checkout-snapdir",
            Check::ContainerSnapdir => "container-snapdir",
            Check::Key => "key",
            Check::CloneProperties => "clone-properties",
            Check::CheckoutMount => "checkout-mount",
            Check::CheckoutPropagation => "checkout-propagation",
            Check::NestedReachability => "nested-reachability",
            Check::ParentIdentity => "parent-identity",
            Check::SlotIdentity => "slot-identity",
            Check::StoreEntries => "store-entries",
            Check::StagingForms => "staging-forms",
            Check::UnknownDotEntry => "unknown-dot-entry",
            Check::Delegation => "delegation",
            Check::HandlerInstallation => "handler-installation",
            Check::RootModes => "root-modes",
            Check::HandlerConfiguration => "handler-configuration",
            Check::HandlerFolders => "handler-folders",
            Check::ZedWiring => "zed-wiring",
            Check::Readiness => "readiness",
            Check::UnresolvedRequests => "unresolved-requests",
            Check::StagingFolder => "staging-folder",
            Check::StagingBindPropagation => "staging-bind-propagation",
            Check::CloneState => "clone-state",
            Check::Registration => "registration",
            Check::ManagedLock => "managed-lock",
            Check::BindingRecord => "binding-record",
            Check::RemovalPlan => "removal-plan",
            Check::CloneSnapshots => "clone-snapshots",
            Check::CloneOrigin => "clone-origin",
            Check::CloneRetention => "clone-retention",
            Check::BlockedRemoval => "blocked-removal",
            Check::Holders => "holders",
            Check::CancelCommand => "cancel-command",
            Check::BootProtection => "boot-protection",
            Check::ServiceUnit => "service-unit",
            Check::BackupCoverage => "backup-coverage",
            Check::Sanoid => "sanoid",
        };
    }
}

/// What:
///  One check's answer.
/// Why:
///  A pass still carries what it read,
///  because "the checkout is mounted" is useful beside the command a person
///  may want next,
///  and an unknown always carries why,
///  because an unread fact without a reason sends a person nowhere.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CheckOutcome = { kind: 'pass'; detail: string } | { kind: 'finding'; field: string; detail: string } | { kind: 'unknown'; reason: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CheckOutcome {
    /// The check read the fact and it holds.
    Pass {
        /// What the check read.
        detail: String,
    },
    /// The check read the fact and it does not hold.
    Finding {
        /// The field or path component that fails.
        field: String,
        /// What the check read and what the decided state is.
        detail: String,
    },
    /// The check could not read or prove the fact.
    Unknown {
        /// Why the fact could not be read or proved.
        reason: String,
    },
}

/// A pass with what the check read.
pub fn pass(detail: impl Into<String>) -> CheckOutcome {
    return CheckOutcome::Pass {
        detail: detail.into(),
    };
}

/// A finding naming the failing field or path component.
pub fn finding(
    field: impl Into<String>,
    detail: impl Into<String>,
) -> CheckOutcome {
    return CheckOutcome::Finding {
        field: field.into(),
        detail: detail.into(),
    };
}

/// Unknown with the reason the check could not read or prove the fact.
pub fn unknown(reason: impl Into<String>) -> CheckOutcome {
    return CheckOutcome::Unknown {
        reason: reason.into(),
    };
}

/// What:
///  One check and its answer,
///  in the order a report prints them.
/// Why:
///  The wiring run collects every answer once and prints them in a stable
///  order,
///  so two runs of one machine are comparable and a control asserts one line
///  without naming the others.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CheckReport = { check: Check; outcome: CheckOutcome };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckReport {
    /// Which check answered.
    pub check: Check,
    /// What it answered.
    pub outcome: CheckOutcome,
}

impl CheckReport {
    /// The check and its answer.
    pub fn new(
        check: Check,
        outcome: CheckOutcome,
    ) -> Self {
        return CheckReport {
            check,
            outcome,
        };
    }

    /// Whether the answer is a pass.
    pub fn passed(&self) -> bool {
        return matches!(self.outcome, CheckOutcome::Pass { .. });
    }

    /// Whether the answer is a finding.
    pub fn found(&self) -> bool {
        return matches!(self.outcome, CheckOutcome::Finding { .. });
    }

    /// Whether the answer is unknown.
    pub fn unknown(&self) -> bool {
        return matches!(self.outcome, CheckOutcome::Unknown { .. });
    }

    /// What:
    ///  The line this report prints:
    ///  the section's prefix,
    ///  the check's name,
    ///  the state,
    ///  and the detail,
    ///  field or reason.
    /// Why:
    ///  One shape for every line lets a person and a control read a report
    ///  without parsing three grammars,
    ///  and the field sits where a finding's reader needs it first.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const line = `zfs: ${check.name()}: ${state}: ${detail}`;
    /// ```
    pub fn line(&self) -> String {
        let name: &'static str = self.check.name();
        return match &self.outcome {
            CheckOutcome::Pass {
                detail,
            } => format!("zfs: {name}: pass: {detail}"),
            CheckOutcome::Finding {
                field,
                detail,
            } => format!("zfs: {name}: finding: {field}: {detail}"),
            CheckOutcome::Unknown {
                reason,
            } => format!("zfs: {name}: unknown: {reason}"),
        };
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "check_outcome_tests.rs"]
mod tests;
