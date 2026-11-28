//! What:
//!  This boot's readiness record:
//!  the withdrawal that removes it before anything is drained,
//! and the decision of what the record it withdrew proves this boot verified.
//! Why:
//!  The shutdown unit's `ExecStopPost` runs
//!  "including after a failed or interrupted start",
//!  so the pass may run in a boot whose handler published nothing,
//!  whose pool or checkout never became ready,
//!  or with a record an earlier boot left
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "Boot and shutdown unit").
//! A mount is unmounted only when this boot's own record says the pool and the
//!  checkout were ready and the configuration names the same two places:
//!  a record that says otherwise proves nothing,
//!  and a record that disagrees with the configuration is a state a person
//!  repairs rather than one a pass guesses at
//!  (`crate::worktree_clone_mount_protocol::clone_mount_readiness_admission`:
//!  a record from another boot,
//!  pool or checkout admits nothing).
//!  The withdrawal comes first and the judging second,
//!  because the design orders the withdrawal before in-flight slot work is
//!  drained and the judging is not work on a mount.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const readiness = withdrawBeforeDraining(places);
//! const verified = whatTheRecordVerified(readiness, configuration); // everything | nothing, and why
//! ```

use super::shutdown_unmount_places::ShutdownUnmountPlaces;
use crate::worktree_clone_mount_handler::sweep_configuration::SweepConfiguration;
use crate::worktree_clone_mount_handler::sweep_preconditions::pool_name_of;
use crate::worktree_clone_mount_protocol::clone_mount_readiness::{
    ReadinessRecord,
    ReadyState,
};
use crate::worktree_clone_mount_protocol::clone_mount_readiness_file_withdraw::{
    RecordStood,
    WithdrawRefusal,
    Withdrawal,
    withdraw_the_record,
};
use std::path::Path;

/// What:
///  This boot's identifier and what its readiness record's withdrawal found.
/// Why:
///   The boot is what tells this boot's record from an earlier one's,
///  and the withdrawal is the proof that no attach is admitted after it,
///  so both belong to the pass's answer whatever it then does
///  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
///  "Boot and shutdown unit").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type BootsReadiness = { boot: string; withdrawal };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BootsReadiness {
    /// The boot this pass runs in,
    /// as the kernel names it.
    pub(crate) boot: String,
    /// What stood under the readiness record's name,
    /// and the proof that nothing answers there now.
    pub(crate) withdrawal: Withdrawal,
}

/// What:
///  What one boot's record proves about the mounts standing now.
/// Why:
///   A missing or unreadable record is an answer rather than a failure:
///  the pass changes nothing,
///  and the line it answers says why,
///  which is what an operator needs on a machine whose boot pass never
///  finished
/// (`doc/planning/cli-git-zfs-checkout-boot-mount-provider-and-ordering-design.md`,
///  "A failed prerequisite authorizes no attachment").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Verified = { kind: 'everything' } | { kind: 'nothing'; said };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Verified {
    /// This boot verified the pool and the checkout,
    /// so the mounts at the configured slots are this boot's to unmount.
    Everything,
    /// This boot proved nothing,
    /// and this sentence says what was missing.
    Nothing {
        /// Why no mount was unmounted.
        said: String,
    },
}

/// What:
///  Read this boot's identifier and withdraw its readiness record.
/// Why:
///   The withdrawal is the pass's first act,
///  so no attach is admitted from the moment the pass starts,
///  and a refusal here is the only answer that stops the pass before it reads
///  anything else
///  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
///  "Boot and shutdown unit":
///  "Withdraw readiness and close admission to new ZED attaches before
///  draining in-flight slot operations").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const readiness = withdrawBeforeDraining(places); // throws when the record's folder is not the route's
/// ```
///
/// # Errors
/// The boot identifier cannot be read,
/// the record's directory is not the designed root-owned folder,
/// or a record still answers after the removal.
pub(crate) fn withdraw_before_draining(
    places: &ShutdownUnmountPlaces
) -> Result<BootsReadiness, String> {
    let boot: String = places.this_boot()?;
    let directory: &Path = places.readiness_directory();
    let withdrawal: Withdrawal =
        withdraw_the_record(directory, owner_of(places)).map_err(|refusal: WithdrawRefusal| {
            return format!(
                "{}Nothing was unmounted and no durable intent was changed",
                refusal.message()
            );
        })?;
    return Ok(BootsReadiness {
        boot,
        withdrawal,
    });
}

/// What:
///  The account the readiness directory must belong to,
///  which is the one the record's own reader proves.
/// Why:
///   The withdrawal and the read are one rule about one folder,
/// so the owner comes from the file object the places hold rather than from a
///  second constant that could drift from it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const owner = ownerOf(places); // 0 for the installed place
/// ```
fn owner_of(places: &ShutdownUnmountPlaces) -> u32 {
    return places.readiness.owner();
}

/// What:
///  What this boot's record proves about the mounts standing now.
/// Why:
///   Only a record that names this boot and says the pool and the checkout
///  were ready authorizes the pass to unmount,
///  and a record whose names disagree with the configuration is a refusal
///  rather than an absence,
///  because the two say which place a person looks at
/// (`crate::worktree_clone_mount_protocol::clone_mount_readiness_admission`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const verified = whatTheRecordVerified(readiness, configuration); // throws when the two name different places
/// ```
///
/// # Errors
/// The record and the configuration name different pools or different
/// checkouts.
pub(crate) fn what_the_record_verified(
    readiness: &BootsReadiness,
    configuration: &SweepConfiguration,
) -> Result<Verified, String> {
    let record: &ReadinessRecord = match &readiness.withdrawal.stood {
        RecordStood::Nothing => {
            return Ok(Verified::Nothing {
                said: String::from(
                    "this boot published no readiness record, so nothing here says it verified the pool and the checkout",
                ),
            });
        }
        RecordStood::Unreadable {
            said,
        } => {
            return Ok(Verified::Nothing {
                said: format!(
                    "the readiness record this boot published is one this route's reader refuses ({said})"
                ),
            });
        }
        RecordStood::Record(record) => record,
    };
    if record.boot != readiness.boot {
        return Ok(Verified::Nothing {
            said: format!(
                "the readiness record standing belonged to boot {}, and this is boot {}",
                record.boot, readiness.boot
            ),
        });
    }
    let wanted_pool: &str = pool_name_of(configuration.parent_dataset.as_str());
    if record.pool_name != wanted_pool {
        return Err(format!(
            "this boot's readiness record names pool `{}` and the configuration names `{wanted_pool}` (its container is `{}`), so the two say which pool this pass may unmount in differently and nothing was unmounted. Ways forward: write the container this boot verified into the configuration the boot unit reads, or run the boot unit's `boot-reconcile` again, and read `journalctl -u reconcile-git-worktree-clone-mounts.service` for what that pass read",
            record.pool_name, configuration.parent_dataset
        ));
    }
    let wanted_checkout: String = configuration.checkout.display().to_string();
    if record.checkout_path != wanted_checkout {
        return Err(format!(
            "this boot's readiness record names checkout {} and the configuration names {wanted_checkout}, so the two say which place this boot verified differently and nothing was unmounted. Ways forward: write the checkout this boot verified into the configuration or run the boot unit's `boot-reconcile` again, and read `journalctl -u reconcile-git-worktree-clone-mounts.service` for what that pass read",
            record.checkout_path
        ));
    }
    if record.pool_state != ReadyState::Ready {
        return Ok(Verified::Nothing {
            said: format!(
                "this boot's readiness record says the pool was `{}` and not ready",
                record.pool_state.word()
            ),
        });
    }
    if record.checkout_state != ReadyState::Ready {
        return Ok(Verified::Nothing {
            said: format!(
                "this boot's readiness record says the checkout was `{}` and not ready",
                record.checkout_state.word()
            ),
        });
    }
    return Ok(Verified::Everything);
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "the_boots_readiness_tests.rs"]
mod tests;
