//! What:
//!  The words and the names the index lock uses:
//!  the path of a lock,
//!  how often a holder's line is repeated,
//!  what the wait tells the clients,
//!  why a wait gave up,
//!  and the code a storage error stands for.
//! Why:
//!  The wait the landing makes,
//! the pass that reclaims a lock whose holder is gone and the pass that reports a refused one all read these,
//! and none of them reads the taking of the lock for a path or a line of words.
//!  Held apart,
//! the pass that reports a refused lock no longer names the file that takes one.
//!  The path and the words are the same for every one of them,
//! so a second copy would drift from the first.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! lockPathOf(indexPath); noticeEvery(); announce(note); storageFailure(error)
//! ```

use crate::service_protocol::FailureCode;
use std::path::{
    Path,
    PathBuf,
};
use std::time::Duration;

/// How often the holder's notice is repeated while the wait lasts.
///  Exempt from a citation under `LQM` (class:
/// notice pacing):
/// it times a line the person reads or a log threshold,
/// and
///  nothing waits on it beyond the wait that already exists.
/// Classed once in `doc/planning/cli-git-shipped-limits-
///  audit.md`,
/// "Exempt limits";
/// the coordinator's pick under the owner's delegation,
/// open to the owner's veto.
pub const NOTICE_EVERY: Duration = Duration::from_secs(10);

/// The environment variable that sets how often the holder's line is repeated,
/// in whole seconds.
pub const NOTICE_VARIABLE: &str = "CLI_GIT_WAIT_NOTICE_SECONDS";

/// How often the holder's line is repeated:
/// the variable when it holds a positive number of seconds,
/// ten seconds otherwise.
pub(crate) fn notice_every() -> Duration {
    return notice_interval(std::env::var(NOTICE_VARIABLE).ok().as_deref());
}

/// The interval a setting of `CLI_GIT_WAIT_NOTICE_SECONDS` stands for:
/// a positive whole number of seconds,
/// else ten seconds.
/// Why:
///   Zero would repeat the holder's line without a pause.
pub(crate) fn notice_interval(setting: Option<&str>) -> Duration {
    // A setting that is missing, is not a number or is zero leaves the default.
    let seconds: Option<u64> = match setting {
        Some(text) => text.trim().parse::<u64>().ok(),
        None => None,
    };
    return match seconds {
        Some(found) if found > 0 => Duration::from_secs(found),
        _ => NOTICE_EVERY,
    };
}

/// What the wait tells the clients of the batch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Announce {
    /// A line for the person's terminal:
    /// who holds the lock and how long the wait lasts.
    Waiting(String),
    /// Who held the lock when the wait ran out,
    /// in words that fit "... is using this repository's index".
    Holder(String),
}

/// What:
///  Where the wait for the index lock tells the clients of the batch what it waits for.
///  A `trait` is a named set of methods that different types can each supply,
/// like a TypeScript `interface`.
/// Why:
///   The wait and the landing take the announcer as a value,
/// so a control can stand in for it;
/// the state the announcement needs
///  (the streams of the batch) is a small struct that implements this.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface Announcer { announce(note: Announce): void; clientIsGone(place: number): boolean }
/// ```
pub trait Announcer {
    /// Tell the clients `note`.
    fn announce(
        &self,
        note: Announce,
    );

    /// What:
    ///  Whether the client of the request at `place` of the batch is gone,
    ///  so its commit is not made.
    /// Why:
    ///   The landing asks this where it can still stop without moving the branch,
    /// and a client that is gone cannot be told anything,
    ///  so its commit exists for nobody (`service_land_batch_body.rs`).
    ///  A caller that lands without clients answers `false`.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// interface Announcer { announce(note: Announce): void; clientIsGone(place: number): boolean }
    /// ```
    fn client_is_gone(
        &self,
        _place: usize,
    ) -> bool {
        return false;
    }
}

/// Why the lock was not taken.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LockFailure {
    /// The code the clients are answered with.
    pub code: FailureCode,
}

/// The path of the lock of an index.
pub fn lock_path_of(index: &Path) -> PathBuf {
    let mut name = index.as_os_str().to_os_string();
    name.push(".lock");
    return PathBuf::from(name);
}

/// The failure a storage error stands for.
pub fn storage_failure(error: &std::io::Error) -> FailureCode {
    return match error.raw_os_error() {
        Some(code) if code == 28 || code == 122 => FailureCode::RepositoryFull,
        Some(code) if code == libc::EROFS => FailureCode::RepositoryReadOnly,
        Some(code) if code == libc::EACCES => FailureCode::PermissionDenied,
        _ => FailureCode::Unavailable,
    };
}
