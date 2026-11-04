//! What:
//!  Who a marker's holder is:
//! the identity this process writes into a marker,
//! and the three answers a reader gives about a holder named in one:
//! gone,
//! running,
//! or unknown.
//! Why:
//!  A marker names a process by its PID and start tick,
//! which mean something only in the boot and the PID namespace they were read in.
//! The identity therefore ends in the boot ID and the PID namespace,
//! a writer whose `/proc` numbers processes differently from the process itself writes none,
//! and a reader that cannot compare like with like answers unknown,
//! which keeps the lock and proves nothing,
//! instead of reading a number of another namespace as a local process and calling a live service gone.
//! The module is split from `service_index_marker.rs`,
//! which re-exports the public items here under their old names.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const identity = ownIdentity(); const state = holderState(holder, readerView); // 'gone' | 'running' | 'unknown'
//! ```

use crate::claim_time::current_boot_id;
use crate::process_identity::{ProcessIdentityError, current_birth_identity, process_birth_identity};
use crate::service_index_marker::{Marked, marker_of};
use std::path::Path;
use std::sync::OnceLock;

/// What separates the parts of the identity in a marker:
/// the birth identity,
/// the boot ID,
/// and the PID namespace.
pub const BOOT_SEPARATOR: char = '#';

/// What a reader makes of the holder a marker names.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HolderState {
    /// No such process,
    /// a process born since,
    /// or a boot that has ended.
    Gone,
    /// The process runs now,
    /// in the same boot and the same PID namespace.
    Running,
    /// The reader cannot compare:
    /// another PID namespace,
    /// a `/proc` that does not agree with the reader's own PID,
    /// no boot ID to compare,
    /// or a record that cannot be read.
    Unknown,
}

/// What a reader can say about its own numbering of processes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReaderView<'a> {
    /// The boot ID of the host now,
    /// empty where it cannot be read.
    pub boot: &'a str,
    /// The PID namespace of the reader,
    /// where it can be read.
    pub scope: Option<&'a str>,
    /// The machine ID of the reader's host (32 lowercase hexadecimal digits, not all zero), where it can be read.
    pub machine: Option<&'a str>,
    /// Whether the reader's `/proc` numbers the reader as the operating system does.
    pub procfs_agrees: bool,
}

/// The number inside the `pid:[...]` a `/proc/self/ns/pid` link reads,
/// when the text is exactly that with digits only.
/// Why:
///   The scope is written into every marker and compared by readers,
/// so a text that is not a namespace number is no scope.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// /^pid:\[(\d+)\]$/.exec(text)?.[1]
/// ```
pub fn scope_of_link(text: &str) -> Option<String> {
    let number: &str = text.strip_prefix("pid:[")?.strip_suffix(']')?;
    return (!number.is_empty() && number.as_bytes().iter().all(u8::is_ascii_digit)).then_some(number.to_string());
}

/// The PID namespace of this process as `/proc/self/ns/pid` names it (the number inside `pid:[...]`),
/// once.
pub fn current_pid_scope() -> Option<&'static str> {
    static SCOPE: OnceLock<Option<String>> = OnceLock::new();
    return SCOPE
        .get_or_init(|| {
            let target = std::fs::read_link("/proc/self/ns/pid").ok()?;
            return scope_of_link(target.to_string_lossy().as_ref());
        })
        .as_deref();
}

/// Whether the record `read` returned from `/proc/self/stat` numbers the process `own` as the operating system does:
/// a record that could not be read,
/// one that names no PID and one that names another PID do not.
pub fn procfs_agrees_with(read: std::io::Result<Vec<u8>>, own: u32) -> bool {
    let Ok(stat) = read else { return false };
    return check_procfs_view(stat.as_slice(), own).is_ok();
}

/// Whether this process's `/proc` numbers it as the operating system does,
/// once.
fn current_procfs_agrees() -> bool {
    static AGREES: OnceLock<bool> = OnceLock::new();
    return *AGREES.get_or_init(read_procfs_agreement);
}

/// What the first call of `current_procfs_agrees` remembers:
/// this process's own record,
/// read now.
fn read_procfs_agreement() -> bool {
    return procfs_agrees_with(std::fs::read("/proc/self/stat"), std::process::id());
}

impl ReaderView<'static> {
    /// What this process can say now.
    pub fn current() -> ReaderView<'static> {
        return ReaderView { boot: current_boot_id(), scope: current_pid_scope(), machine: None, procfs_agrees: current_procfs_agrees() };
    }
}

/// What:
///  The PID that a `/proc/self/stat` record names,
/// when it is one.
/// Why:
///   The record is the first thing a process reads of itself in `/proc`;
/// its first field is the PID as the mounted `/proc` numbers it.
/// The name of the process in the record may hold any byte,
/// so the record is read as bytes and only the leading digits are looked at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Number(stat.split(' ')[0])
/// ```
pub fn pid_of_self_stat(stat: &[u8]) -> Option<i64> {
    let digits: &[u8] = stat.split(|byte| { return *byte == b' '; }).next()?;
    return std::str::from_utf8(digits).ok()?.parse::<i64>().ok();
}

/// What:
///  This process's birth identity as a marker names it:
/// the identity,
/// the boot it was read in,
/// and the PID namespace.
/// Why:
///   A PID and a start tick since boot recur after a reboot and mean another process in another PID namespace,
/// so a marker is told by the boot ID and the namespace.
/// A `/proc` that belongs to another PID namespace numbers processes differently from the process itself,
/// and an identity read there names another process,
/// so a process whose `/proc` does not agree with its own PID takes no lock.
/// On Linux a process that cannot name its boot or its namespace takes no lock either:
/// a marker without them cannot be told from a reboot's collision.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${await birthIdentity(process.pid)}#${bootId}#${pidNamespace}`
/// ```
///
/// # Errors
///
/// The identity error when `/proc` is another namespace's,
/// the boot or the namespace is unknown,
/// or the identity cannot be read.
pub fn own_identity() -> Result<String, ProcessIdentityError> {
    let own: u32 = std::process::id();
    #[cfg(target_os = "linux")]
    let stat: Vec<u8> = std::fs::read("/proc/self/stat").map_err(|error| { return ProcessIdentityError::Io { pid: i64::from(own), error }; })?;
    #[cfg(not(target_os = "linux"))]
    let stat: Vec<u8> = format!("{own} ").into_bytes();
    return identity_in_view(stat.as_slice(), own, current_boot_id(), current_pid_scope());
}

/// `own_identity` for the process `own`,
/// whose `/proc/self/stat` reads `stat`,
/// on a host whose boot is `boot` and in the PID namespace `scope`.
///
/// # Errors
///
/// As `own_identity`;
/// `ProcMismatch` when `stat` names another PID than `own`,
/// `BootUnavailable` and `ScopeUnavailable` on Linux when the boot or the namespace is not known.
pub fn identity_in_view(stat: &[u8], own: u32, boot: &str, scope: Option<&str>) -> Result<String, ProcessIdentityError> {
    return identity_in_view_on(stat, own, boot, scope, None);
}

/// `identity_in_view` on the machine `machine`.
///
/// # Errors
///
/// As `identity_in_view`.
pub fn identity_in_view_on(stat: &[u8], own: u32, boot: &str, scope: Option<&str>, machine: Option<&str>) -> Result<String, ProcessIdentityError> {
    let _ = machine;
    check_procfs_view(stat, own)?;
    let birth: String = current_birth_identity()?;
    #[cfg(target_os = "linux")]
    {
        if boot.is_empty() {
            return Err(ProcessIdentityError::BootUnavailable);
        }
        if scope.is_none() {
            return Err(ProcessIdentityError::ScopeUnavailable);
        }
    }
    let mut identity: String = birth;
    for part in [Some(boot).filter(|text| { return !text.is_empty(); }), scope] {
        if let Some(text) = part {
            identity.push(BOOT_SEPARATOR);
            identity.push_str(text);
        }
    }
    return Ok(identity);
}

/// Whether the `/proc/self/stat` record `stat` numbers the process `own` the way the process itself is numbered.
///
/// # Errors
///
/// `ProcMismatch` naming both numbers when `/proc` belongs to another PID namespace,
/// and `MalformedStat` when the record names no PID.
pub fn check_procfs_view(stat: &[u8], own: u32) -> Result<(), ProcessIdentityError> {
    let procfs: i64 = pid_of_self_stat(stat).ok_or(ProcessIdentityError::MalformedStat { pid: i64::from(own) })?;
    if procfs != i64::from(own) {
        return Err(ProcessIdentityError::ProcMismatch { procfs, own });
    }
    return Ok(());
}

/// The birth identity,
/// the boot ID and the PID namespace a marker names;
/// a marker an earlier release wrote names no boot,
/// and one of the release before the namespace was recorded names no namespace.
fn split_birth(birth: &str) -> (&str, Option<&str>, Option<&str>) {
    let mut parts = birth.splitn(3, BOOT_SEPARATOR);
    let identity: &str = parts.next().unwrap_or(birth);
    return (identity, parts.next(), parts.next());
}

/// What:
///  What a reader with the view `view` makes of `holder`,
/// asking `ask` about the process.
/// Why:
///   A boot that has ended ends every process in it,
/// whatever the reader's numbering.
/// Beyond that the reader compares like with like or says unknown:
/// its `/proc` must agree with its own PID,
/// it must know its boot when the marker names one,
/// and its PID namespace must be the marker's when the marker names one.
/// A holder the marker does not name (an earlier release's marker) is gone,
/// as such markers always were.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// holderStateIn(holder, view, ask) // 'gone' | 'running' | 'unknown'
/// ```
pub fn holder_state_in(
    holder: Option<&(i64, String)>,
    view: &ReaderView,
    ask: &dyn Fn(i64) -> Result<Option<String>, ProcessIdentityError>,
) -> HolderState {
    let Some((pid, birth)) = holder else { return HolderState::Gone };
    let (identity, boot, scope) = split_birth(birth.as_str());
    if let Some(recorded) = boot
        && !view.boot.is_empty()
        && recorded != view.boot
    {
        return HolderState::Gone;
    }
    if !view.procfs_agrees || (boot.is_some() && view.boot.is_empty()) {
        return HolderState::Unknown;
    }
    if let Some(recorded) = scope
        && view.scope != Some(recorded)
    {
        return HolderState::Unknown;
    }
    return match ask(*pid) {
        Ok(None) => HolderState::Gone,
        Ok(Some(now)) if now == identity => HolderState::Running,
        Ok(Some(_)) => HolderState::Gone,
        Err(_) => HolderState::Unknown,
    };
}

/// What this process makes of `holder` now.
pub fn holder_state(holder: Option<&(i64, String)>) -> HolderState {
    return holder_state_in(holder, &ReaderView::current(), &process_birth_identity);
}

/// Whether the process a marker names is gone;
/// a question that cannot be answered keeps the lock,
/// which is the safe side.
pub fn holder_is_gone(holder: Option<&(i64, String)>) -> bool {
    return holder_state(holder) == HolderState::Gone;
}

/// Whether the process a marker names is proven to run now.
pub fn holder_is_running(holder: Option<&(i64, String)>) -> bool {
    return holder_state(holder) == HolderState::Running;
}

/// Whether the lock at `lock` carries the service's mark naming a holder that is gone:
/// a dead instance's leftover,
/// which the next landing removes.
/// A mark that names no holder (one an earlier release wrote) is not claimed here,
/// because its age is the only evidence a reader has of it.
pub fn mark_names_a_gone_holder(lock: &Path) -> bool {
    return marker_of(lock).is_some_and(|marked: Marked| { return marked.holder.is_some() && holder_is_gone(marked.holder.as_ref()); });
}
