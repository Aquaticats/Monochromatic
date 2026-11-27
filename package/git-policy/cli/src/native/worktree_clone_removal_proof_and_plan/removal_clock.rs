//! What:
//!  The three facts a removal's plan takes from the process rather than from a record:
//!  the run's own nonce,
//!  the process that writes the plan,
//!  and the moment the removal started.
//! Why:
//!  A control that planted these would be reading its own constants back,
//!  and the field's meaning is the process that wrote it,
//!  so the seam is one trait with one live answer,
//!  and no reader decides anything by the values it gives.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! interface RemovalClock { run(); owner(); started(); }
//! ```

use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use crate::worktree_clone_removal_proof_and_plan::plan_record::owner_text;
use crate::worktree_clone_removal_proof_and_plan::removal_stop::RemovalStop;
use crate::worktree_clone_removal_proof_and_plan::started_moment::now_with_offset;

/// What:
///  The three facts a plan's header takes from the process rather than from a record:
///  the run's nonce,
///  the writing process and the moment.
/// Why:
///  A control that planted these would be reading its own constants back,
///  and the field's meaning is the process that wrote it,
///  so the seam is this trait with one live answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface RemovalClock { run(); owner(); started(); }
/// ```
pub trait RemovalClock {
    /// The removal run's own 32-digit nonce.
    ///
    /// # Throws
    ///
    /// The random source failed.
    fn run(&self) -> Result<RequestNonce, RemovalStop>;

    /// The process that writes the plan,
    /// spelled `<pid>:linux:<start time>`.
    ///
    /// # Throws
    ///
    /// The process's own birth identity cannot be read.
    fn owner(&self) -> Result<String, RemovalStop>;

    /// The moment the removal started.
    fn started(&self) -> String;
}

/// What:
///  The release answer:
///  the operating system's clock,
///  this process and its birth identity.
/// Why:
///  The fields are facts a person reads,
///  and no reader decides anything by them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const clock = TheLiveClock;
/// ```
pub struct TheLiveClock;

impl RemovalClock for TheLiveClock {
    fn run(&self) -> Result<RequestNonce, RemovalStop> {
        return RequestNonce::fresh().map_err(|refusal| {
            return RemovalStop::NoRunNonce {
                reason: refusal.message(),
            };
        });
    }

    fn owner(&self) -> Result<String, RemovalStop> {
        let identity: String =
            crate::process::process_identity::current_birth_identity().map_err(|error| {
                return RemovalStop::NoOwnerIdentity {
                    reason: format!("{error:?}"),
                };
            })?;
        return Ok(owner_text(std::process::id(), identity.as_str()));
    }

    fn started(&self) -> String {
        return now_with_offset();
    }
}
