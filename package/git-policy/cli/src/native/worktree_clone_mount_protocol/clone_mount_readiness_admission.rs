//! What:
//!  Why an attach is not admitted,
//!  and the one comparison that decides it:
//!  this boot,
//!  this pool and this checkout,
//!  each ready.
//! Why:
//!  The readiness record is a cache and not authority to act,
//!  so an attach is admitted only when the record names the live boot,
//!  pool and checkout and both provider facts are ready
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "ZED reconciliation and mount safety",
//!  "Requests,
//! results and ordering").
//!  A key-load event before the checkout mounts therefore refuses,
//!  naming the checkout,
//!  with no waiter and no timer.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const admitted = admitAttach(record, { boot, pool, checkout });
//! ```

use crate::worktree_clone_mount_protocol::clone_mount_readiness::{
    ReadinessExpectation,
    ReadinessRecord,
    ReadyState,
};
use crate::worktree_clone_mount_protocol::clone_mount_receipts_folder_refusals::NO_DOCTOR_LINE_AND_NO_INSTALLED_UNIT;

/// The log tag of every decision this module takes.
const TAG: &str = "cloneMountReadinessAdmission";

/// What:
///  Why an attach is not admitted.
/// Why:
///   The design asks for the readiness refusal to name what is not ready,
///  and a key-load event before the checkout mounts must report the checkout
///  rather than a generic failure,
///  so each cause names its own field and value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AdmissionRefusal = { kind: 'other-boot' | 'other-pool' | 'other-checkout' | 'pool' | 'checkout'; ... };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdmissionRefusal {
    /// The record belongs to another boot.
    OtherBoot {
        /// The record's boot.
        recorded: String,
        /// This boot.
        expected: String,
    },
    /// The record describes another pool.
    OtherPool {
        /// The record's pool.
        recorded: String,
        /// The pool the caller named.
        expected: String,
    },
    /// The record describes another checkout.
    OtherCheckout {
        /// The record's checkout.
        recorded: String,
        /// The checkout the caller named.
        expected: String,
    },
    /// The pool is not ready.
    PoolNotReady {
        /// The pool.
        pool: String,
        /// The state the record carries.
        state: ReadyState,
    },
    /// The checkout is not ready.
    CheckoutNotReady {
        /// The checkout.
        checkout: String,
        /// The state the record carries.
        state: ReadyState,
    },
}

impl AdmissionRefusal {
    /// The refusal in words,
    /// naming the field and the way forward.
    pub fn message(&self) -> String {
        let reason: String = match self {
            AdmissionRefusal::OtherBoot {
                recorded,
                expected,
            } => format!(
                "the readiness record belongs to boot {recorded} and this is boot {expected}, so it says nothing about this boot"
            ),
            AdmissionRefusal::OtherPool {
                recorded,
                expected,
            } => {
                format!(
                    "the readiness record was written for pool `{recorded}` and the caller named pool `{expected}`"
                )
            }
            AdmissionRefusal::OtherCheckout {
                recorded,
                expected,
            } => {
                format!(
                    "the readiness record was written for checkout `{recorded}` and the caller named `{expected}`"
                )
            }
            AdmissionRefusal::PoolNotReady {
                pool,
                state,
            } => format!(
                "pool `{pool}` is `{}` in the readiness record",
                state.word()
            ),
            AdmissionRefusal::CheckoutNotReady {
                checkout,
                state,
            } => {
                format!(
                    "checkout `{checkout}` is `{}` in the readiness record",
                    state.word()
                )
            }
        };
        return format!(
            "the clone mount is not admitted: {reason}. No clone was attached, mounted or unmounted, and no durable intent was changed. Ways forward: wait for the boot unit's `boot-reconcile` to verify the provider and publish this boot's readiness, then let the next trigger (a ZED event, the checkout provider's post-key-load hook, or the caller's own sweep nudge) run the sweep. {NO_DOCTOR_LINE_AND_NO_INSTALLED_UNIT}."
        );
    }
}

/// What:
///  Whether this boot's record admits an attach for the given pool and
///  checkout.
/// Why:
///   The comparison is one place,
///  so a bookkeeping record and a live caller cannot decide differently,
///  and the refusal names the first field that is not ready.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const admission = admitAttach(record, expectation); // { ready } | { refusal }
/// ```
pub fn admit_attach(
    record: &ReadinessRecord,
    expectation: &ReadinessExpectation<'_>,
) -> Result<(), AdmissionRefusal> {
    let admitted: Result<(), AdmissionRefusal> = compare(record, expectation);
    match &admitted {
        Ok(()) => {
            crate::diagnostic_log::debug(TAG, "admitted an attach on this boot's readiness record")
        }
        Err(refusal) => crate::diagnostic_log::warn(TAG, refusal.message().as_str()),
    }
    return admitted;
}

/// The comparison itself,
/// which logs nothing and decides everything.
fn compare(
    record: &ReadinessRecord,
    expectation: &ReadinessExpectation<'_>,
) -> Result<(), AdmissionRefusal> {
    if record.boot != expectation.boot {
        return Err(AdmissionRefusal::OtherBoot {
            recorded: record.boot.clone(),
            expected: String::from(expectation.boot),
        });
    }
    if record.pool_name != expectation.pool {
        return Err(AdmissionRefusal::OtherPool {
            recorded: record.pool_name.clone(),
            expected: String::from(expectation.pool),
        });
    }
    if record.pool_state != ReadyState::Ready {
        return Err(AdmissionRefusal::PoolNotReady {
            pool: record.pool_name.clone(),
            state: record.pool_state,
        });
    }
    if record.checkout_path != expectation.checkout {
        return Err(AdmissionRefusal::OtherCheckout {
            recorded: record.checkout_path.clone(),
            expected: String::from(expectation.checkout),
        });
    }
    if record.checkout_state != ReadyState::Ready {
        return Err(AdmissionRefusal::CheckoutNotReady {
            checkout: record.checkout_path.clone(),
            state: record.checkout_state,
        });
    }
    return Ok(());
}
