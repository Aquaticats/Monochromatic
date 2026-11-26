//! What:
//!  The readiness record one boot's handler publishes:
//!  the boot,
//!  the pool,
//!  the checkout and the sweep,
//!  the words each state is written as,
//!  and the decision that admits an attach only when this boot's pool and
//!  checkout are ready.
//! Why:
//!  A key-load event can arrive before the checkout is mounted,
//!  and with no waiter and no timer the handler must refuse rather than attach
//!  under an unready provider
//!  (`doc/planning/cli-git-zfs-worktree-design-proposal.md`,
//!  "ZED reconciliation and mount safety",
//!  "Requests,
//! results and ordering";
//!  `doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 7).
//!  The record is a cache and not authority to act:
//!  every reader compares it with the live boot,
//!  pool and checkout before an attach,
//!  and a record from another boot or another pool admits nothing.
//!  The sweep is recorded for an operator,
//!  and it gates no attach,
//!  because one running sweep does not make the pool or the checkout unready.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const admission = admitAttach(record, { boot, pool, checkout });
//! ```

use crate::worktree_clone_mount_protocol::clone_mount_property_names::NONCE_HEX_DIGITS;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;

/// The record's format version.
pub const READINESS_VERSION: &str = "v1";

/// The record's kind,
/// which tells it from a receipt.
pub const READINESS_KIND: &str = "clone-mount-readiness";

/// The directory one boot's readiness lives in.
pub const READINESS_DIRECTORY: &str = "/run/git-worktree-clone-mount-readiness";

/// The record's file name inside that directory.
pub const READINESS_NAME: &str = "result";

/// The directory's mode.
pub const READINESS_DIRECTORY_MODE: u32 = 0o755;

/// The record's mode.
pub const READINESS_MODE: u32 = 0o644;

/// The record's keys in the order the file holds them.
pub const READINESS_KEYS: [&str; 7] = ["v1", "kind", "boot", "pool", "checkout", "sweep", "time"];

/// The bound one pool name may use.
pub const MAXIMUM_POOL_NAME_BYTES: usize = 255;

/// The bound one checkout path may use,
/// which is the kernel's `PATH_MAX` without its terminator.
pub const MAXIMUM_CHECKOUT_PATH_BYTES: usize = 4095;

/// The bound the kernel's own boot identifier uses,
/// a canonical UUID's 36 characters.
pub const MAXIMUM_BOOT_BYTES: usize = 36;

/// The bound one ready state word may use.
pub const MAXIMUM_READY_STATE_BYTES: usize = 9;

/// The bound one sweep state word may use.
pub const MAXIMUM_SWEEP_STATE_BYTES: usize = 8;

/// The bound one sweep nonce may use.
pub const MAXIMUM_SWEEP_NONCE_BYTES: usize = NONCE_HEX_DIGITS;

/// The bound one time may use:
/// the date,
/// the second,
/// nine fraction digits and the offset.
pub const MAXIMUM_TIME_BYTES: usize = 35;

/// What:
///  The largest readiness record a reader accepts,
///  from the bound of every field the writer fills and the line each field is
///  written on.
/// Why:
///   A file bound typed in beside field bounds can fall below them,
///  and then a record the writer can write is one the reader refuses,
///  so no caller sees this boot ready and nothing says why.
///  Derived this way,
///  every record the writer builds fits,
///  and `publish` proves the reader reads it back.
///  It refuses a record it cannot read back,
///  so nothing is filed for a caller to wait out.
pub const MAXIMUM_READINESS_BYTES: usize = READINESS_KEYS[0].len()
    + READINESS_KEYS[1].len()
    + 1
    + READINESS_KIND.len()
    + READINESS_KEYS[2].len()
    + 1
    + MAXIMUM_BOOT_BYTES
    + READINESS_KEYS[3].len()
    + 1
    + MAXIMUM_READY_STATE_BYTES
    + 1
    + MAXIMUM_POOL_NAME_BYTES
    + READINESS_KEYS[4].len()
    + 1
    + MAXIMUM_READY_STATE_BYTES
    + 1
    + MAXIMUM_CHECKOUT_PATH_BYTES
    + READINESS_KEYS[5].len()
    + 1
    + MAXIMUM_SWEEP_STATE_BYTES
    + 1
    + MAXIMUM_SWEEP_NONCE_BYTES
    + READINESS_KEYS[6].len()
    + 1
    + MAXIMUM_TIME_BYTES
    + READINESS_KEYS.len();

/// What:
///  Whether one prerequisite is ready.
/// Why:
///   A boot's answer can be ready,
///  not ready,
///  or unknown,
///  and only ready admits an attach:
///  unknown is what the handler says when it could not read the fact,
///  and treating it as ready would attach under a provider nobody verified.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ReadyState = 'ready' | 'not-ready' | 'unknown';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReadyState {
    /// The prerequisite was verified ready.
    Ready,
    /// The prerequisite was verified not ready.
    NotReady,
    /// The handler could not tell.
    Unknown,
}

impl ReadyState {
    /// The word the record carries.
    pub fn word(self) -> &'static str {
        return match self {
            ReadyState::Ready => "ready",
            ReadyState::NotReady => "not-ready",
            ReadyState::Unknown => "unknown",
        };
    }

    /// Read one state word.
    pub fn from_word(word: &str) -> Option<Self> {
        return match word {
            "ready" => Some(ReadyState::Ready),
            "not-ready" => Some(ReadyState::NotReady),
            "unknown" => Some(ReadyState::Unknown),
            _ => None,
        };
    }
}

/// What:
///  What the sweep was doing when the record was written.
/// Why:
///   The design names the sweep as the record's fourth field,
///  and an operator needs to see a running or failed sweep;
///  no state of it makes the pool or the checkout unready.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SweepState = 'finished' | 'running' | 'failed' | 'never';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SweepState {
    /// A sweep finished.
    Finished,
    /// A sweep was running.
    Running,
    /// A sweep failed.
    Failed,
    /// No sweep has run in this boot.
    Never,
}

impl SweepState {
    /// The word the record carries.
    pub fn word(self) -> &'static str {
        return match self {
            SweepState::Finished => "finished",
            SweepState::Running => "running",
            SweepState::Failed => "failed",
            SweepState::Never => "never",
        };
    }

    /// Read one sweep state word.
    pub fn from_word(word: &str) -> Option<Self> {
        return match word {
            "finished" => Some(SweepState::Finished),
            "running" => Some(SweepState::Running),
            "failed" => Some(SweepState::Failed),
            "never" => Some(SweepState::Never),
            _ => None,
        };
    }
}

/// What:
///  This boot's pool,
///  checkout and sweep,
///  as one record.
/// Why:
///   The four fields are everything a reader compares and an operator reports,
///  so a record and its file hold the same values.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ReadinessRecord = { boot; poolName; poolState; checkoutPath; checkoutState; sweepNonce; sweepState; time };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReadinessRecord {
    /// The boot this record belongs to,
    /// as the kernel's own identifier.
    pub boot: String,
    /// The pool the handler verified.
    pub pool_name: String,
    /// Whether the pool is imported and ready.
    pub pool_state: ReadyState,
    /// The checkout the handler verified.
    pub checkout_path: String,
    /// Whether the checkout is mounted and ready.
    pub checkout_state: ReadyState,
    /// The sweep's nonce,
    /// where a sweep was involved.
    pub sweep_nonce: Option<RequestNonce>,
    /// What the sweep was doing.
    pub sweep_state: SweepState,
    /// When the record was published,
    /// as ISO 8601 with offset.
    pub time: String,
}

impl ReadinessRecord {
    /// The file's whole text,
    /// its final newline included.
    pub fn text(&self) -> String {
        let nonce: String = self
            .sweep_nonce
            .as_ref()
            .map_or(String::from("none"), |value| {
                return String::from(value.as_str());
            });
        let lines: [String; 7] = [
            String::from(READINESS_VERSION),
            format!("kind {READINESS_KIND}"),
            format!("boot {}", self.boot),
            format!("pool {} {}", self.pool_state.word(), self.pool_name),
            format!(
                "checkout {} {}",
                self.checkout_state.word(),
                self.checkout_path
            ),
            format!("sweep {} {nonce}", self.sweep_state.word()),
            format!("time {}", self.time),
        ];
        return format!("{}\n", lines.join("\n"));
    }
}

/// What:
///  What one reader knows about the live boot,
///  pool and checkout.
/// Why:
///   The record is a cache,
///  so every attach compares it with the provider's live facts,
///  and a record for another boot,
///  another pool or another checkout is stale rather than ready.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ReadinessExpectation = { boot; pool; checkout };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReadinessExpectation<'a> {
    /// This boot's kernel identifier.
    pub boot: &'a str,
    /// The pool the caller is about to attach in.
    pub pool: &'a str,
    /// The checkout the caller walks from.
    pub checkout: &'a str,
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "clone_mount_readiness_tests.rs"]
mod tests;
