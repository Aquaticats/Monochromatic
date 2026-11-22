//! What:
//!  The root handler's answer to "who holds this ZFS clone":
//!  the all-namespace scan,
//!  the holders it found,
//!  the managed attach a standing `unmount` intent is taking down,
//!  every read it could not complete,
//!  and the result that carries those under the dataset's GUID and the
//!  request's nonce,
//!  with its text,
//!  its parse and its reader's freshness check.
//! Why:
//!  "Unmounted" means absent in every mount namespace holding the clone's ZFS
//!  source,
//!  including binds and staging,
//!  and the removal workflow asks this question again immediately before every
//!  snapshot-destroy request and before the head destroy
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Mounts held by other namespaces";
//!  `doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 9,
//!  whose step 16 consumes the result before each destroy and whose step 18
//!  reports the holders `doctor` names).
//!  This folder only reads:
//!  no mount,
//!  unmount,
//!  destroy or record happens here,
//!  so a scan can run before any decision without changing what it decides.
//!  The grouping,
//!  the one table per namespace and the exact source match are
//!  `crate::mount_namespace_scan`'s,
//!  which the start probe reads too.
//!
//! Files in this folder,
//! each using only files listed before it:
//!
//! - `held_mounts.rs`:
//!   one mount of the dataset as a report names it.
//! - `namespace_holder.rs`:
//!   one namespace and the mounts of the dataset it holds.
//! - `incomplete_reads.rs`:
//!   every read the scan could not complete and what each cause means.
//! - `standing_unmount.rs`:
//!   the one managed attach a standing `unmount` intent sets aside,
//!   and the narrow rule that decides it.
//! - `holder_scan.rs`:
//!   the scan itself,
//!   its two bounds and the value it answers.
//! - `holder_scan_refusals.rs`:
//!   why a text is not a result.
//! - `holder_scan_text.rs`:
//!   the result's text,
//!   its escaping and the bound its reader shares.
//! - `holder_scan_parse.rs`:
//!   reading that text back.
//! - `holder_scan_acceptance.rs`:
//!   whether a result answers the request at hand.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const scan = scanHolders({ procRoot: '/proc', source: clone, ownNamespace, standingUnmount, dataset, nonce, time });
//! const read = parseHolderScan(scan.text()); read.accept({ dataset, nonce });
//! ```

/// One mount of the dataset as a report names it.
pub mod held_mounts;

/// One namespace and the mounts of the dataset it holds.
pub mod namespace_holder;

/// Every read the scan could not complete.
pub mod incomplete_reads;

/// The managed attach a standing `unmount` intent sets aside.
pub mod standing_unmount;

/// The scan,
/// its bounds and the value it answers.
pub mod holder_scan;

/// Why a text is not a result.
pub mod holder_scan_refusals;

/// The result's text and the bound its reader shares.
pub mod holder_scan_text;

/// Reading that text back.
pub mod holder_scan_parse;

/// Whether a result answers the request at hand.
pub mod holder_scan_acceptance;

/// One mount of the dataset.
pub use held_mounts::HeldMount;
/// One namespace and its mounts of the dataset.
pub use namespace_holder::NamespaceHolder;
/// Every read the scan could not complete.
pub use incomplete_reads::IncompleteRead;
/// The managed attach a standing `unmount` intent sets aside.
pub use standing_unmount::StandingUnmount;
/// The scan,
/// its request,
/// its bounds and its value.
pub use holder_scan::{HolderScan, HolderScanRequest, MAXIMUM_DETAIL_LINES, MAXIMUM_MOUNTS_PER_NAMESPACE, MAXIMUM_MOUNT_NAMESPACES, MAXIMUM_REASON_BYTES, scan_holders};
/// Why a text is not a result.
pub use holder_scan_refusals::HolderScanRefusal;
/// The result's text,
/// its escaping and its bound.
pub use holder_scan_text::{MAXIMUM_RESULT_BYTES, RESULT_KIND, RESULT_VERSION, escape_path, escape_text, unescape_path, unescape_text};
/// Reading that text back.
pub use holder_scan_parse::parse_holder_scan;
/// Whether a result answers the request at hand.
pub use holder_scan_acceptance::{ScanExpectation, StaleResult};
