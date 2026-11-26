//! What:
//!  What a control may ask about one watched command:
//!  whether the monitor still watches it,
//!  how many looks found a task of it waiting for the disk,
//!  and how many looks the monitor took of it at all.
//! Why:
//!   A control that waits for the monitor to have judged its command a number of times counts the looks instead of the
//!   clock,
//!   and a control that wants a slow volume to have been in the way reads the disk-wait count after the command ended.
//!  Both figures are kept after a command ends for exactly that second reader,
//!  so neither goes with the entry the monitor removes when the command is over.
//!  The registry those figures live in is `binary_supervisor_registry.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const looksOf = (pid: number) => registry.looks.get(pid) ?? 0;
//! ```
use super::supervisor_registry::with_registry;

/// Whether the monitor still watches the command `pid`.
pub fn is_watching(pid: u32) -> bool {
    let Ok(watched) = i32::try_from(pid) else {
        return false;
    };
    return with_registry(|registry| {
        return registry.watched.contains_key(&watched);
    });
}

/// How many looks of the monitor found a task of the command `pid` waiting for the disk:
/// a control that wants a slow
///  volume to have been in the way reads it after the command ended.
pub fn disk_wait_looks(pid: u32) -> u32 {
    let Ok(watched) = i32::try_from(pid) else {
        return 0;
    };
    return with_registry(|registry| {
        return registry.disk_wait_looks.get(&watched).copied().unwrap_or(0);
    });
}

/// How many looks the monitor has taken of the command `pid`:
/// a control that waits for the monitor to have judged a command
///  a number of times counts them instead of the clock.
pub fn looks_of(pid: u32) -> u32 {
    let Ok(watched) = i32::try_from(pid) else {
        return 0;
    };
    return with_registry(|registry| {
        return registry.looks.get(&watched).copied().unwrap_or(0);
    });
}
