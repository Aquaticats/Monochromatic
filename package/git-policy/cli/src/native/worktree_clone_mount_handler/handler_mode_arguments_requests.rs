//! What:
//!  What one invocation asks this build to do:
//!  the mode,
//!  and the identities the four modes that take an argument are handed.
//! Why:
//!  A mode is chosen by a leading word and nothing else,
//!  and every argument a mode takes is an identity its proofs need,
//!  so the vocabulary lives here and the reading of the words lives in the
//!  module that opens the door.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type Mode = { kind: 'sweep' } | { kind: 'boot-reconcile' } | { kind: 'shutdown-unmount' } | { kind: 'adopt'; request: Adoption } | { kind: 'help' } | { kind: 'version' };
//! type Adoption = { checkout: string; device: number; inode: number };
//! ```

use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use std::path::PathBuf;

/// What:
///  The administrator's identification of the replacement folder an adoption judges.
/// Why:
///   The design requires the device and inode to be identified independently,
///  from the restored checkout or the controlled recreation,
///  and the mode compares the held folder against them before anything is recorded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Adoption = { checkout: '/home/user/Monochromatic', device: 66306, inode: 1234567 };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdoptRequest {
    /// The checkout whose `.claude/worktrees` folder is walked.
    pub checkout: PathBuf,
    /// The device the administrator read for the intended replacement.
    pub device: u64,
    /// The inode the administrator read for the intended replacement.
    pub inode: u64,
}

/// What:
///  The dataset one `holder-scan` invocation reads,
///  and the nonce it answers under when the caller names one.
/// Why:
///   The scan is about one dataset's mounts and nothing else:
///  it names every namespace that holds it and sets nothing aside,
///  so no path is read and none is guessed.
///  The nonce is the caller's when it names one,
///  because the wrapper asks for a fresh scan before each destroy and matches
///  the answer by the nonce it chose.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HolderScan = { dataset: 'zpcachyos/…/clones/one', nonce: '…' | undefined };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HolderScanInvocation {
    /// The dataset whose holders the scan reports.
    pub dataset: String,
    /// The nonce the answer is bound to,
    /// or nothing when the scan makes its own.
    pub nonce: Option<RequestNonce>,
}

/// What:
///  The checkout one `record-checkout` invocation records,
///  and the GUID it admits when a legitimate restore changed the record's.
/// Why:
///   The checkout is the folder whose dataset the route serves,
///  and the replacement is the kernel's own GUID and no other value,
///  so a command cannot admit a dataset without naming the number it read
///  (`doc/decision/cli-git-zfs-root-modes-the-eleven-choices-of-the-one-program-design.md`,
///  choice 9).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RecordCheckout = { checkout: '/home/user/Monochromatic', replace: '9876543210987654321' | undefined };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordCheckoutRequest {
    /// The checkout whose route facts are recorded.
    pub checkout: PathBuf,
    /// The kernel's GUID when this run replaces a record a restore moved.
    pub replace: Option<String>,
}

/// What:
///  The nested dataset one `mount-path-descendant` invocation mounts.
/// Why:
///   The dataset names the work and the record names the checkout it is placed
///  under,
///  so the mode takes no path:
///  a path a caller gave would be a mountpoint
///  nothing proved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MountPathDescendant = { dataset: 'zpcachyos/nosnap/vm-builder-output' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MountPathDescendantRequest {
    /// The nested dataset's full name.
    pub dataset: String,
}

/// What:
///  What one invocation asks this build to do.
/// Why:
///   A mode is chosen by a leading word and nothing else,
///  and the only arguments a mode takes are the identities its proofs need:
///  the sweep reads the durable intent,
///  an adoption takes the replacement's own identity,
///  `record-checkout` takes the checkout and the GUID a replace admits,
///  and `mount-path-descendant` takes the dataset it places under the recorded
///  checkout.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Mode = { kind: 'sweep' } | { kind: 'boot-reconcile' } | { kind: 'shutdown-unmount' } | { kind: 'adopt'; request: Adoption } | { kind: 'help' } | { kind: 'version' };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HandlerMode {
    /// No argument:
    /// the clone-mount sweep ZED's own start runs.
    AttachingSweep,
    /// `boot-reconcile`:
    /// the boot unit's reconciliation.
    BootReconcile,
    /// `start-probe`:
    /// the root start probe's arms and its record.
    StartProbe,
    /// `shutdown-unmount`:
    /// the shutdown unit's verified unmount.
    ShutdownUnmount,
    /// `holder-scan`:
    /// the all-namespace holder scan of one dataset.
    HolderScan(HolderScanInvocation),
    /// `adopt-worktree-parent`:
    /// the administrator's decision to record a replacement parent folder.
    AdoptWorktreeParent(AdoptRequest),
    /// `record-checkout`:
    /// the record of the checkout's path,
    /// dataset,
    /// GUID,
    /// container and pool.
    RecordCheckout(RecordCheckoutRequest),
    /// `mount-path-descendant`:
    /// one nested dataset mounted at its own mountpoint beneath the recorded
    /// checkout.
    MountPathDescendant(MountPathDescendantRequest),
    /// `--help`.
    Help,
    /// `--version`.
    Version,
}
