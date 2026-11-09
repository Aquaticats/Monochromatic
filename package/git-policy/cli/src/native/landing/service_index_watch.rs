//! What:
//!  The inotify watch on the directory of an index lock,
//! and the reading of the events it delivers.
//! Why:
//!  The service waits for a holder of `index.lock` without polling:
//! the watch wakes it when the lock is removed or moved away,
//! when the kernel reports an overflow that may have hidden that,
//! or when the watch itself ended (the directory was unmounted or removed).
//! Where no watch can be set (the platform has none,
//! the watch limit is reached,
//! or the file system gives none),
//! the waits look at the lock again after `POLL_PAUSE` instead of failing or waiting blind.
//! The module is split from `service_index_lock.rs`,
//! which re-exports everything here under its old names.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const watch = watchDirectory(dir); const stillWatching = watch.wait(timeout, 'index.lock');
//! ```

use std::path::Path;
use std::time::{Duration, Instant};

/// How long a wait that has no watch sleeps between two looks at the lock.
///  This is a poll pace,
/// not a limit:
/// it decides how soon a release is noticed,
/// never whether a wait ends.
/// The measured holds of
///  Git's own short commands in this repository were 15 to 100 ms (2026-11-03,
/// measured at `INDEX_ATTEMPTS` in `service_land_follow.rs`),
/// so fifty milliseconds
///  notices a typical release within one hold.
pub const POLL_PAUSE: Duration = Duration::from_millis(50);

#[cfg(test)]
thread_local! {
    /// How many watches the current thread holds open,
    /// so a control can tell that a watch was dropped without reading descriptor numbers another thread may reuse.
    static LIVE_ON_THIS_THREAD: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

/// How many watches the current thread holds open.
#[cfg(test)]
pub fn live_watches_on_this_thread() -> usize {
    return LIVE_ON_THIS_THREAD.with(std::cell::Cell::get);
}

/// What:
///  How many waits on a watch ended at the timer they asked for,
/// for the controls;
/// nothing in the release executable.
/// Why:
///   A control of "the lock's removal woke the wait, and not the reminder's timer" otherwise reads the host's
/// speed,
/// while the wait itself knows which of the two ended it.
#[cfg(test)]
pub static WAKES_BY_THE_TIMER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// Records for the controls that a wait on a watch ended at its timer.
#[cfg(test)]
fn note_a_wake_at_the_timer() {
    WAKES_BY_THE_TIMER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
}

/// Does nothing in the release executable.
#[cfg(not(test))]
fn note_a_wake_at_the_timer() {}

/// An inotify watch on the directory of the lock;
/// nothing on platforms without inotify.
pub struct Watch {
    #[cfg(target_os = "linux")]
    descriptor: i32,
}

/// What:
///  A native-endian 32-bit field of an inotify event,
/// `offset` bytes into the event that starts at `at`.
/// Why:
///  `parse_events` reads the mask and the name length of each event of a buffer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const fieldAt = (buffer: Buffer, at: number, offset: number) => buffer.readUInt32LE(at + offset);
/// ```
#[cfg(target_os = "linux")]
fn field_at(buffer: &[u8], at: usize, offset: usize) -> u32 {
    let mut bytes: [u8; 4] = [0; 4];
    bytes.copy_from_slice(&buffer[at + offset..at + offset + 4]);
    return u32::from_ne_bytes(bytes);
}

/// What:
///  Whether a byte is the NUL that ends the name of an inotify event.
/// Why:
///  `parse_events` cuts the name at the first NUL.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isZeroByte = (byte: number) => byte === 0;
/// ```
#[cfg(target_os = "linux")]
fn is_zero_byte(byte: &u8) -> bool {
    return *byte == 0;
}

/// What:
///  Whether an inotify event says the watch is over:
/// the file system was unmounted,
/// the watch was removed,
/// or the watched file moved.
/// Why:
///  `events_end_watch` asks it of every event of a buffer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const endsTheWatch = ([mask]) => (mask & (IN_UNMOUNT | IN_IGNORED | IN_MOVE_SELF)) !== 0;
/// ```
#[cfg(target_os = "linux")]
fn ends_the_watch(event: &(u32, &[u8])) -> bool {
    return event.0 & (libc::IN_UNMOUNT | libc::IN_IGNORED | libc::IN_MOVE_SELF) != 0;
}

/// What:
///  The events of an inotify read as their masks and names.
/// Why:
///   Bytes are parsed,
/// not cast,
/// so an unaligned buffer is read as safely as an aligned one;
/// a record cut off by the end of the buffer ends the list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// parseEvents(buffer) // [{ mask, name }]
/// ```
#[cfg(target_os = "linux")]
fn parse_events(buffer: &[u8]) -> Vec<(u32, &[u8])> {
    let header: usize = std::mem::size_of::<libc::inotify_event>();
    let mut events: Vec<(u32, &[u8])> = Vec::new();
    let mut at: usize = 0;
    while at + header <= buffer.len() {
        let length: usize = usize::try_from(field_at(buffer, at, 12)).unwrap_or(usize::MAX);
        let start: usize = at + header;
        let end: usize = start.saturating_add(length).min(buffer.len());
        let name: &[u8] = buffer[start..end].split(is_zero_byte).next().unwrap_or(&[]);
        events.push((field_at(buffer, at, 4), name));
        at = start.saturating_add(length);
    }
    return events;
}

/// What:
///  Whether a buffer of inotify events reports that the watch itself ended:
/// its directory was unmounted (`IN_UNMOUNT`),
/// the kernel dropped the watch (`IN_IGNORED`,
/// which also follows the directory's removal),
/// or the directory was renamed (`IN_MOVE_SELF`).
/// Why:
///   None of these names a file.
/// A watch that has ended reports nothing more,
/// and one on a directory that was renamed aside stays on the directory at its new name,
/// so a wait that kept polling either would sleep to its timer instead of looking at the lock at the old path again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// events.some((event) => event.unmount || event.ignored || event.moveSelf)
/// ```
#[cfg(target_os = "linux")]
pub fn events_end_watch(buffer: &[u8]) -> bool {
    return parse_events(buffer).iter().any(ends_the_watch);
}

/// What:
///  Whether a buffer of inotify events names `wanted`,
/// or reports an overflow that may have hidden it,
/// or the end of the watch.
/// Why:
///   The watch covers the whole Git directory,
/// where Git and the service create and remove other files all the time;
/// only the lock's own removal is a reason to try again,
/// so a wait is not woken by its own staging file or a neighbour's.
/// An overflow may have hidden that removal,
/// and a watch that ended can report none,
/// so those wake the caller to look.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// events.some((event) => event.overflow || event.unmount || event.ignored || event.name === wanted)
/// ```
#[cfg(target_os = "linux")]
pub fn events_name(buffer: &[u8], wanted: &[u8]) -> bool {
    if events_end_watch(buffer) {
        return true;
    }
    for (mask, name) in parse_events(buffer) {
        if mask & libc::IN_Q_OVERFLOW != 0 || name == wanted {
            return true;
        }
    }
    return false;
}

#[cfg(target_os = "linux")]
impl Watch {
    pub fn on(directory: &Path) -> Option<Watch> {
        use std::os::unix::ffi::OsStrExt;
        let path = std::ffi::CString::new(directory.as_os_str().as_bytes()).ok()?;
        // SAFETY: plain system calls with valid arguments; the descriptor is closed in `Drop`.
        let descriptor: i32 = unsafe { libc::inotify_init1(libc::IN_CLOEXEC) };
        if descriptor < 0 {
            return None;
        }
        // SAFETY: `path` is a valid NUL-terminated string for the call.
        let added: i32 = unsafe { libc::inotify_add_watch(descriptor, path.as_ptr(), libc::IN_DELETE | libc::IN_MOVED_FROM | libc::IN_MOVE_SELF) };
        if added < 0 {
            // SAFETY: the descriptor was just opened and nothing else owns it.
            unsafe { libc::close(descriptor) };
            return None;
        }
        #[cfg(test)]
        LIVE_ON_THIS_THREAD.with(|count| { count.set(count.get() + 1); });
        return Some(Watch { descriptor });
    }

    /// Blocks until the file `name` in the directory was removed or moved away,
    /// or `timeout` passed;
    /// either way the caller looks again.
    /// Events for other files are drained and the wait goes on.
    /// False when the watch has ended (the directory was unmounted,
    /// removed or moved),
    /// so the caller sets a new one before it waits again.
    pub fn wait(&self, timeout: Duration, name: &std::ffi::OsStr) -> bool {
        use std::os::unix::ffi::OsStrExt;
        let deadline: Instant = Instant::now() + timeout;
        loop {
            let left: Duration = deadline.saturating_duration_since(Instant::now());
            if left.is_zero() {
                note_a_wake_at_the_timer();
                return true;
            }
            let mut entry = libc::pollfd { fd: self.descriptor, events: libc::POLLIN, revents: 0 };
            let milliseconds: i32 = i32::try_from(left.as_millis()).unwrap_or(i32::MAX).max(1);
            // SAFETY: `entry` is valid for the one descriptor named by the count.
            let ready: i32 = unsafe { libc::poll(&raw mut entry, 1, milliseconds) };
            if ready <= 0 {
                note_a_wake_at_the_timer();
                return true;
            }
            let mut buffer: [u8; 4096] = [0; 4096];
            // SAFETY: the buffer is valid for its length.
            let read: isize = unsafe { libc::read(self.descriptor, buffer.as_mut_ptr().cast(), buffer.len()) };
            let Ok(count) = usize::try_from(read) else {
                note_a_wake_at_the_timer();
                return true;
            };
            if count == 0 {
                note_a_wake_at_the_timer();
                return true;
            }
            if events_name(&buffer[..count], name.as_bytes()) {
                return !events_end_watch(&buffer[..count]);
            }
        }
    }
}

#[cfg(target_os = "linux")]
impl Drop for Watch {
    fn drop(&mut self) {
        #[cfg(test)]
        LIVE_ON_THIS_THREAD.with(|count| { count.set(count.get().saturating_sub(1)); });
        // SAFETY: the descriptor belongs to this watch and is closed once.
        unsafe { libc::close(self.descriptor) };
    }
}

#[cfg(not(target_os = "linux"))]
impl Watch {
    pub fn on(_directory: &Path) -> Option<Watch> {
        return None;
    }

    pub fn wait(&self, _timeout: Duration, _name: &std::ffi::OsStr) -> bool {
        return true;
    }
}
