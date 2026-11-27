//! What:
//!  What the sweep and the boot reconciliation know before any attach:
//!  the pool the pinned parent lives in and whether it is imported,
//!  whether the checkout walks and whether its own mount is shared,
//!  the fields this boot's readiness record carries,
//!  and the two halves of the preparation that reads them.
//! Why:
//!  A failed prerequisite authorizes no attachment
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Boot and shutdown unit"),
//!  and every answer here is one of the record's own three states,
//!  read from the live kernel and the live folders each run,
//!  because the readiness record is a cache and not authority.
//!  The start probe's record is `sweep_probe_record.rs`'s,
//!  and what a run knows about the pool and the checkout is this file's,
//!  so each can be read whole on its own
//!  (rule `MXL`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const facts = bootFacts(configuration, reader, recordPath); // { boot, poolName, poolState, checkout, checkoutState }
//! ```

use super::sweep_checkout_mount::{
    CheckoutMount,
    CheckoutMounts,
};
use super::sweep_configuration::SweepConfiguration;
use super::sweep_dataset_reader::DatasetReader;
use crate::worktree_clone_mount::clone_slot_walk_refusals::WalkRefusalReason;
use crate::worktree_clone_mount::hold_the_worktrees_parent;
use crate::worktree_clone_mount_protocol::ReadyState;
use std::path::{
    Path,
    PathBuf,
};

/// This boot's identifier,
/// the start probe's record for it and the words its verdict answers with,
/// which a caller of these checks names by this file.
pub use super::sweep_probe_record::{
    BOOT_ID_PATH,
    ProbeRefusal,
    ProbeStanding,
    boot_identifier,
    probe_fingerprint_standing,
    probe_record_path,
    probe_standing,
    read_probe_standing,
};

/// What:
///  The pool one pinned parent lives in,
///  which is the first `/`-separated component of its name.
/// Why:
///  The readiness record names the pool beside the checkout,
///  and the pool is the parent's own first component rather than a second
///  configuration key that could disagree with it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pool = parent.split('/')[0];
/// ```
pub fn pool_name_of(parent: &str) -> &str {
    return parent.split('/').next().unwrap_or(parent);
}

/// What:
///  Whether the pool is imported and its copy of the parent readable.
/// Why:
///  One identity read of the pinned parent answers both:
///  a pool that is not imported cannot answer it,
///  and a parent the pool no longer holds is not ready either.
///  A read the route cannot interpret is `unknown`,
///  which admits nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const state = poolState(reader, parent); // ready | not-ready | unknown
/// ```
pub fn pool_state(
    reader: &dyn DatasetReader,
    parent: &str,
) -> ReadyState {
    return match reader.identity(parent) {
        Ok(_identity) => ReadyState::Ready,
        // The kernel's own `ENOENT` is the one answer that says the pool or the parent is gone rather than unreadable,
        // so the state is read from the error number and never from the words the binding happened to print.
        Err(failure) if failure.errno == Some(libc::ENOENT) => ReadyState::NotReady,
        Err(_failure) => ReadyState::Unknown,
    };
}

/// What:
///  Whether the checkout is there,
///  its `.claude/worktrees` folder walkable,
///  and its own mount not shared.
/// Why:
///  The walk the attach starts with is the only proof that the checkout is
///  mounted and its parent folder reachable,
///  and the design's measured move refused `EINVAL` while the checkout was
///  shared and passed once only it was private
///  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
///  "Mounts held by other namespaces"),
///  so a shared checkout is an unverified immediate parent and admits nothing.
///  The state is the walk's own answer and the mount's own tag rather than a
///  path check.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const state = checkoutState('/home/user/Monochromatic', liveCheckoutMounts); // ready | not-ready | unknown
/// ```
pub fn checkout_state(
    checkout: &Path,
    mounts: &dyn CheckoutMounts,
) -> (ReadyState, String) {
    if let Err(refusal) = hold_the_worktrees_parent(checkout) {
        let said: String = refusal.message();
        let state: ReadyState = match refusal.reason {
            // A checkout that is not there, or that is reached through a link which is not its own canonical path, is
            // not mounted as the provider promises; every other refusal is a state this run cannot interpret.
            WalkRefusalReason::Missing
            | WalkRefusalReason::NotTheCanonicalPath
            | WalkRefusalReason::Checkout
            | WalkRefusalReason::NotADirectory => ReadyState::NotReady,
            WalkRefusalReason::LinkInTheWay
            | WalkRefusalReason::BoundaryCrossed
            | WalkRefusalReason::NotAPlainName
            | WalkRefusalReason::Unreadable
            | WalkRefusalReason::UnreadableFields => ReadyState::Unknown,
        };
        return (state, said);
    }
    let mount: CheckoutMount = match mounts.mount_holding(checkout) {
        Ok(mount) => mount,
        Err(refusal) => return (ReadyState::Unknown, refusal.message()),
    };
    if mount.is_shared() {
        return (
            ReadyState::NotReady,
            format!(
                "the checkout's own mount is `{}`, so it is shared and a mount made on it is delivered to another namespace; this route requires a not-shared immediate parent",
                mount.shared.as_deref().unwrap_or("shared")
            ),
        );
    }
    return (ReadyState::Ready, String::new());
}

/// What:
///  What one run knows about this boot before it attaches anything.
/// Why:
///  The readiness record's fields are these facts,
///  and the sweep's own admission is the same two states,
///  so one value carries them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type BootFacts = { boot; poolName; poolState; checkout; checkoutState };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BootFacts {
    /// This boot's identifier.
    pub boot: String,
    /// The pool's name.
    pub pool_name: String,
    /// Whether the pool is ready.
    pub pool_state: ReadyState,
    /// The checkout's path.
    pub checkout: PathBuf,
    /// Whether the checkout is ready.
    pub checkout_state: ReadyState,
    /// What the checkout's check answered,
    /// which the refusal quotes when the checkout is not ready.
    pub checkout_said: String,
}

impl BootFacts {
    /// What:
    ///  Refuse an attach unless both the pool and the checkout are ready.
    /// Why:
    ///  A failed prerequisite authorizes no attachment,
    ///  and `unknown` admits nothing.
    ///
    /// # Errors
    /// The first field that is not ready.
    pub fn admit(&self) -> Result<(), ProbeRefusal> {
        if self.pool_state != ReadyState::Ready {
            return Err(ProbeRefusal::PoolNotReady {
                pool: self.pool_name.clone(),
                state: self.pool_state,
            });
        }
        if self.checkout_state != ReadyState::Ready {
            return Err(ProbeRefusal::CheckoutNotReady {
                checkout: self.checkout.clone(),
                state: self.checkout_state,
                said: self.checkout_said.clone(),
            });
        }
        return Ok(());
    }
}

/// What:
///  What one run knows before it opens the device:
///  this boot,
///  the probe's answer for it,
///  and whether the checkout is walkable.
/// Why:
///  Every check here reads a file or walks a folder,
///  so a run that fails one of them makes no raw request at all,
///  and a boot with no probe record never opens the pool.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PreparedBoot = { boot; checkout; checkoutState; standing };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedBoot {
    /// This boot's identifier.
    pub boot: String,
    /// The checkout's path.
    pub checkout: PathBuf,
    /// Whether the checkout is walkable.
    pub checkout_state: ReadyState,
    /// What the checkout's check answered.
    pub checkout_said: String,
    /// The probe's standing for this boot.
    pub standing: ProbeStanding,
}

/// What:
///  Read this boot's identifier,
///  the start probe's record for it,
///  and whether the checkout walks,
///  before anything is opened.
/// Why:
///  A boot with no passing probe record attaches nothing,
///  and that decision costs the pool nothing to make.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const prepared = prepare(configuration, probeRecordPath);
/// ```
///
/// # Errors
/// The probe's record,
/// the boot identifier,
/// or nothing at all when every check passed.
pub fn prepare(
    configuration: &SweepConfiguration,
    probe_record: &Path,
    mounts: &dyn CheckoutMounts,
) -> Result<PreparedBoot, ProbeRefusal> {
    let standing: ProbeStanding = read_probe_standing(probe_record)?;
    let boot: String = boot_identifier()?;
    let (checkout_state, checkout_said): (ReadyState, String) =
        checkout_state(configuration.checkout.as_path(), mounts);
    return Ok(PreparedBoot {
        boot,
        checkout: configuration.checkout.clone(),
        checkout_state,
        checkout_said,
        standing,
    });
}

/// What:
///  Fill in the pool's state and refuse the run when a prerequisite is not ready.
/// Why:
///  The pool is read through the device,
///  so this half is what needs the open handle,
///  and the checkout's own state joins it here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const facts = complete(prepared, reader, parent);
/// ```
///
/// # Errors
/// The pool or the checkout is not ready.
pub fn complete(
    prepared: PreparedBoot,
    reader: &dyn DatasetReader,
    parent: &str,
) -> Result<BootFacts, ProbeRefusal> {
    let facts: BootFacts = BootFacts {
        boot: prepared.boot,
        pool_name: String::from(pool_name_of(parent)),
        pool_state: pool_state(reader, parent),
        checkout: prepared.checkout,
        checkout_state: prepared.checkout_state,
        checkout_said: prepared.checkout_said,
    };
    facts.admit()?;
    return Ok(facts);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "sweep_preconditions_tests.rs"]
mod tests;
