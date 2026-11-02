//! What:
//!  The marker the service writes into the `index.lock` it holds,
//! and the questions asked of a marker found there.
//! Why:
//!  A restart tells its own stale lock from a foreign one,
//! and a live holder from a dead one,
//! by the marker alone:
//! the batch,
//!  the holding process and that process's birth identity.
//! The module is split from `service_index_lock.rs`,
//! which re-exports the public items here under their old names.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const marker = markerText(seq); const reclaimable = isReclaimable(lockPath);
//! ```

use super::claim_time::current_boot_id;
use super::process_identity::{ProcessIdentityError, current_birth_identity, process_birth_identity};
use std::io::Read;
use std::path::Path;

/// The first bytes of a lock the service holds,
/// followed by the batch number,
/// the holding process and its birth identity.
pub const LOCK_MARKER: &str = "cli-git-service ";

/// The longest marker this reader looks at.
const MARKER_LIMIT: u64 = 512;

/// What a service marker names.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Marked {
    /// The batch the lock was taken for.
    pub seq: u64,
    /// The holding process and its birth identity;
    /// none in a marker a release before this one wrote.
    pub holder: Option<(i64, String)>,
}

/// What:
///  The marker this process writes for batch `seq`:
/// the batch,
/// the process and its birth identity,
/// one line.
/// Why:
///   A restart reclaims a marked lock only when that process is gone,
/// so a slow but live holder keeps its lock.
/// A marker without a birth identity reads as a dead instance's,
/// so a process that cannot name its own identity writes none:
/// it must not take the lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `cli-git-service ${seq} ${process.pid} ${birth}\n`
/// ```
///
/// # Errors
///
/// The identity error when this process's birth identity cannot be read.
pub fn marker_text(seq: u64, identity: &dyn Fn() -> Result<String, ProcessIdentityError>) -> Result<String, ProcessIdentityError> {
    let birth: String = identity()?;
    return Ok(format!("{LOCK_MARKER}{seq} {} {birth}\n", std::process::id()));
}

/// What:
///  The batch and holder a lock's text names,
/// or nothing when it is not a service marker.
/// Why:
///   A marker without a holder is one a release before this one wrote;
/// one with a malformed holder is not the service's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// parseMarker(text) // { seq, holder?: [pid, birth] } | undefined
/// ```
pub fn parse_marker(text: &str) -> Option<Marked> {
    let rest: &str = text.strip_prefix(LOCK_MARKER)?;
    let line: &str = rest.strip_suffix('\n').unwrap_or(rest);
    let mut parts = line.splitn(3, ' ');
    let seq: u64 = parts.next()?.parse::<u64>().ok()?;
    let holder: Option<(i64, String)> = match (parts.next(), parts.next()) {
        (None, None) => None,
        (Some(pid), Some(birth)) if !birth.is_empty() => Some((pid.parse::<i64>().ok()?, birth.to_string())),
        _ => return None,
    };
    return Some(Marked { seq, holder });
}

/// What separates a birth identity from the boot ID in a marker.
pub const BOOT_SEPARATOR: char = '#';

/// What:
///  The PID that a `/proc/self/stat` text names,
/// when it is one.
/// Why:
///   The record is the first thing a process reads of itself in `/proc`;
/// its first field is the PID as the mounted `/proc` numbers it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Number(stat.split(' ')[0])
/// ```
pub fn pid_of_self_stat(stat: &str) -> Option<i64> {
    return stat.split(' ').next()?.parse::<i64>().ok();
}

/// What:
///  This process's birth identity as a marker names it:
/// the identity and the boot it was read in.
/// Why:
///   A PID and a start tick since boot recur after a reboot,
/// so a marker from another boot is told by the boot ID.
/// A `/proc` that belongs to another PID namespace numbers processes differently from the process itself,
/// and an identity read there names another process,
/// so a process whose `/proc` does not agree with its own PID takes no lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${await birthIdentity(process.pid)}#${bootId}`
/// ```
///
/// # Errors
///
/// The identity error when `/proc` is another namespace's or the identity cannot be read.
pub fn own_identity() -> Result<String, ProcessIdentityError> {
    #[cfg(target_os = "linux")]
    {
        let own: u32 = std::process::id();
        let stat: String = std::fs::read_to_string("/proc/self/stat").map_err(|error| { return ProcessIdentityError::Io { pid: i64::from(own), error }; })?;
        let procfs: i64 = pid_of_self_stat(stat.as_str()).ok_or(ProcessIdentityError::MalformedStat { pid: i64::from(own) })?;
        if procfs != i64::from(own) {
            return Err(ProcessIdentityError::ProcMismatch { procfs, own });
        }
    }
    let birth: String = current_birth_identity()?;
    let boot: &str = current_boot_id();
    return Ok(if boot.is_empty() { birth } else { format!("{birth}{BOOT_SEPARATOR}{boot}") });
}

/// The birth identity and the boot ID a marker names;
/// a marker an earlier release wrote names no boot.
fn split_birth(birth: &str) -> (&str, Option<&str>) {
    return match birth.rsplit_once(BOOT_SEPARATOR) {
        Some((identity, boot)) => (identity, Some(boot)),
        None => (birth, None),
    };
}

/// Whether the boot a marker was written in is not the current one.
///  A marker without a boot, and a host that has none, never differ.
fn from_another_boot(recorded: Option<&str>) -> bool {
    let current: &str = current_boot_id();
    return recorded.is_some_and(|boot| { return !current.is_empty() && boot != current; });
}

/// Whether the process a marker names is gone:
/// the boot ended,
/// no process runs under that number,
/// or another one was born since.
/// A question that cannot be answered keeps the lock,
/// which is the safe side.
pub fn holder_is_gone(holder: Option<&(i64, String)>) -> bool {
    let Some((pid, birth)) = holder else { return true };
    let (identity, boot) = split_birth(birth.as_str());
    if from_another_boot(boot) {
        return true;
    }
    return match process_birth_identity(*pid) {
        Ok(None) => true,
        Ok(Some(now)) => now != identity,
        Err(_) => false,
    };
}

/// Whether the process a marker names is proven to run now:
/// the same boot,
/// and the process under that number is the one born then.
/// A question that cannot be answered proves nothing.
pub fn holder_is_running(holder: Option<&(i64, String)>) -> bool {
    let Some((pid, birth)) = holder else { return false };
    let (identity, boot) = split_birth(birth.as_str());
    return !from_another_boot(boot) && matches!(process_birth_identity(*pid), Ok(Some(now)) if now == identity);
}

/// The marker of the file at `path`,
/// when it holds one.
pub fn marker_of(path: &Path) -> Option<Marked> {
    let mut text: String = String::new();
    std::fs::File::open(path).ok()?.take(MARKER_LIMIT).read_to_string(&mut text).ok()?;
    return parse_marker(text.as_str());
}

/// Whether the lock at `lock` is the service's and its holder is gone,
/// so it may be removed.
pub fn is_reclaimable(lock: &Path) -> bool {
    return marker_of(lock).is_some_and(|marked| { return holder_is_gone(marked.holder.as_ref()); });
}
