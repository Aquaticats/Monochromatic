//! What:
//!  One source of the kernel's mount table:
//!  the release source that keeps `/proc/self/mountinfo` open for its poll,
//!  and the seam a control answers with a table it built.
//! Why:
//!  The kernel raises its namespace event after a mount,
//!  an unmount,
//!  a move and a remount or `mount_setattr` that changes flags,
//!  so a poll before every answer is what tells the reader its copy may be
//!  wrong;
//!  a poll that cannot be made counts as a change,
//!  and a control's table is what proves both fail-open paths without a mount.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! interface MountTableSource { changed(): boolean; read(): MountFacts[] }
//! const source = LiveMountTableSource.open();
//! ```

use crate::diagnostic_log::debug;
use crate::fs_mount_options::MountFacts;
use std::os::fd::AsRawFd;
use std::path::Path;

/// Module tag for diagnostics.
pub(crate) const TAG: &str = "sweepStandingMounts";

/// What:
///  One source of the kernel's mount table and the signal that the table
///  changed.
/// Why:
///  A control needs to answer with a table it built,
///  and the release source is the kernel's own file:
///  opening it makes the kernel remember the namespace's event number,
///  and `poll` reports a later event as `POLLPRI` or `POLLERR`.
///  The descriptor is opened before the first table read,
///  so a change between the open and the read fires the next poll rather than
///  being missed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface MountTableSource { changed(): boolean; read(): MountFacts[] }
/// ```
pub(crate) trait MountTableSource {
    /// Whether this namespace's mount table changed since the last poll.
    fn changed(&self) -> bool;

    /// The whole table,
    /// bounded as the route reads it.
    fn read(&self) -> Result<Vec<MountFacts>, String>;
}

/// What:
///  The release source:
///  `/proc/self/mountinfo`,
///  held open for its poll and read by path when it fires.
/// Why:
///  The kernel's `mounts_poll` sets `POLLPRI|POLLERR` when the namespace's
///  event counter differs from the one the open remembered,
///  and `fs/namespace.c` raises that counter after a mount,
///  an unmount,
///  a move and a `mount_setattr` or remount that changes flags,
///  so a poll before every answer keeps the copy right about a mount,
///  an unmount,
///  a move and a flag change.
///  A file that cannot be opened,
///  and a poll that refuses or that names no descriptor the kernel knows,
///  each count as a change,
///  so the sweep reads the table again for the next answer instead of trusting
///  a copy nothing watches
///  (the table's own read still tries by path).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const source = LiveMountTableSource.open();
/// ```
pub(crate) struct LiveMountTableSource {
    /// The descriptor the kernel's poll watches.
    descriptor: Option<std::fs::File>,
    /// A control's own poll answer,
    /// which wins over the descriptor so the two fail-open paths are provable.
    #[cfg(test)]
    answered_for_the_control: Option<Option<libc::c_short>>,
}

impl LiveMountTableSource {
    /// Open the descriptor before the first read,
    /// so a change after this call fires the next poll.
    pub(crate) fn open() -> Self {
        let path: &Path = Path::new(crate::fs_mount_options::MOUNTINFO_PATH);
        let descriptor: Option<std::fs::File> = match std::fs::File::open(path) {
            Ok(file) => Some(file),
            Err(error) => {
                debug(
                    TAG,
                    format!("the mount table {path:?} could not be opened for its poll: {error}")
                        .as_str(),
                );
                None
            }
        };
        return Self {
            descriptor,
            #[cfg(test)]
            answered_for_the_control: None,
        };
    }
}

impl MountTableSource for LiveMountTableSource {
    fn changed(&self) -> bool {
        return match self.poll_answer() {
            // A poll this build cannot make is not evidence that the table stood still: the answer may have changed, so
            // the caller reads the table again rather than trusting a copy nothing watches.
            None => true,
            Some(revents) => revents & (libc::POLLPRI | libc::POLLERR | libc::POLLNVAL) != 0,
        };
    }

    fn read(&self) -> Result<Vec<MountFacts>, String> {
        let path: &Path = Path::new(crate::fs_mount_options::MOUNTINFO_PATH);
        let bytes: Vec<u8> =
            crate::plain_file::read_plain_file(path, crate::plain_file::MOUNT_TABLE_LIMIT)
                .map_err(|error| {
                    return format!("the mount table {path:?} could not be read: {error}");
                })?;
        return Ok(crate::fs_mount_options::parse_mount_table(bytes.as_slice()));
    }
}

impl LiveMountTableSource {
    /// What one poll of the table's descriptor answered,
    /// or nothing when no poll could be made.
    /// Why:
    ///  A missing descriptor,
    ///  a poll that refused,
    ///  a descriptor the kernel no longer knows
    ///  and a control's own answer all have to reach the same rule,
    ///  so the rule lives here and the caller treats "no answer" as a change.
    fn poll_answer(&self) -> Option<libc::c_short> {
        #[cfg(test)]
        if let Some(answer) = self.answered_for_the_control {
            return answer;
        }
        let Some(file) = &self.descriptor else {
            return None;
        };
        let mut watched: libc::pollfd = libc::pollfd {
            fd: file.as_raw_fd(),
            events: libc::POLLPRI | libc::POLLERR,
            revents: 0,
        };
        // SAFETY: `watched` is one initialised `pollfd`, the call writes only its `revents` field, and the timeout is
        // zero so it never waits.
        let ready: libc::c_int = unsafe { libc::poll(&mut watched, 1, 0) };
        if ready < 0 {
            debug(
                TAG,
                format!(
                    "the mount table's descriptor could not be polled: {}",
                    std::io::Error::last_os_error()
                )
                .as_str(),
            );
            return None;
        }
        if ready == 0 {
            return Some(0);
        }
        return Some(watched.revents);
    }

    /// A source with no descriptor,
    /// for the control that the answer is then "changed".
    #[cfg(test)]
    pub(crate) fn without_a_descriptor() -> Self {
        return Self {
            descriptor: None,
            answered_for_the_control: None,
        };
    }

    /// A source whose poll answers the control's value,
    /// `None` being a poll that refused.
    #[cfg(test)]
    pub(crate) fn whose_poll_answers(answer: Option<libc::c_short>) -> Self {
        return Self {
            descriptor: None,
            answered_for_the_control: Some(answer),
        };
    }
}
