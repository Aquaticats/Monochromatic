//! What:
//!  The identity of a process the hang guard of the binary tests may signal:
//! a process ID held with its start time and,
//! where
//!  the kernel gives one,
//! a pidfd,
//! and the ending of a whole tree through those identities.
//! Why:
//!  A process ID is reused once its process is reaped,
//! so a signal sent to a number the monitor read a moment ago can end an
//!  unrelated process of the same user,
//! and `kill(-pid)` has the same flaw for a group.
//! A pidfd names one process for good:
//!  `pidfd_send_signal` on it reaches that process or fails,
//! and never reaches a successor.
//! A pin is made only after the
//!  start time read before the pidfd was opened equals the one read after,
//! which shows that the number named one process
//!  throughout.
//! Where the kernel has no pidfd the pin falls back to reading the start time again right before each signal,
//!  and the window that leaves is the one a pidfd closes.
//!  A tree is ended by stopping every member first (a stopped process forks no more),
//! looking again for children it had
//!  started,
//! and only then killing,
//! so no member escapes between the look and the signal.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const pin = Pin.open(pid, started); // undefined unless pid still names the process that started at `started`
//! pin?.signal('SIGKILL');             // pidfd_send_signal: never reaches a successor of the process
//! ```

use super::supervisor_proc::{children, stat_of};
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};

/// What a signal came to.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Delivered {
    /// The process received the signal.
    Sent,
    /// The process is gone,
    /// or is not the one that was pinned,
    /// so nothing was sent.
    Gone,
}

/// One process,
/// pinned.
#[derive(Debug)]
pub struct Pin {
    /// The process ID the pin was made for.
    pid: i32,
    /// When the process started,
    /// in clock ticks after boot.
    started: u64,
    /// The pidfd,
    /// where the kernel has pidfds.
    descriptor: Option<OwnedFd>,
}

impl Pin {
    /// Pin the process `pid` if it is the one that started at `started`;
    /// nothing when it is gone or another process now.
    pub fn open(pid: i32, started: u64) -> Option<Pin> {
        if stat_of(pid)?.started != started {
            return None;
        }
        // SAFETY:
        // `pidfd_open` takes two integers and returns a descriptor or -1;
        // nothing of this process's memory is touched.
        let raw: libc::c_long = unsafe { libc::syscall(libc::SYS_pidfd_open, libc::c_long::from(pid), 0 as libc::c_long) };
        if raw < 0 {
            let error: std::io::Error = std::io::Error::last_os_error();
            // The process went between the two looks,
            // or the number is not a process.
            if error.raw_os_error() == Some(libc::ESRCH) {
                return None;
            }
            // No pidfd on this kernel or under this sandbox:
            // the pin signals by number after reading the start time again.
            return Some(Pin { pid, started, descriptor: None });
        }
        // SAFETY:
        // `raw` is a descriptor `pidfd_open` just made for this process alone,
        // and nothing else owns it.
        let descriptor: OwnedFd = unsafe { OwnedFd::from_raw_fd(i32::try_from(raw).ok()?) };
        // The number named the process before and after the descriptor was made,
        // so the descriptor is that process's.
        if stat_of(pid)?.started != started {
            return None;
        }
        return Some(Pin { pid, started, descriptor: Some(descriptor) });
    }

    /// Pin `child`,
    /// found in the `children` list of the pinned `parent`:
    /// it must name `parent` as its parent before and after
    ///  the pin,
    /// and the parent must still be the process that was pinned.
    pub fn open_child(parent: &Pin, child: i32) -> Option<Pin> {
        let stat = stat_of(child)?;
        if stat.parent != parent.pid {
            return None;
        }
        let pinned: Pin = Pin::open(child, stat.started)?;
        if stat_of(child)?.parent != parent.pid || parent.is_gone() {
            return None;
        }
        return Some(pinned);
    }

    /// The process ID.
    pub fn pid(&self) -> i32 {
        return self.pid;
    }

    /// When the process started.
    pub fn started(&self) -> u64 {
        return self.started;
    }

    /// Whether the process has exited (a zombie counts) or is another process now.
    pub fn is_gone(&self) -> bool {
        if let Some(descriptor) = &self.descriptor {
            let mut ask = libc::pollfd { fd: descriptor.as_raw_fd(), events: libc::POLLIN, revents: 0 };
            // SAFETY:
            // `ask` is one valid `pollfd` and the count says so;
            // a zero timeout makes the call return at once.
            let ready: libc::c_int = unsafe { libc::poll(&raw mut ask, 1, 0) };
            return ready > 0;
        }
        return stat_of(self.pid).is_none_or(|stat| { return stat.started != self.started || stat.state == 'Z'; });
    }

    /// Send `signal` to the pinned process,
    /// and to no other.
    pub fn signal(&self, signal: libc::c_int) -> Delivered {
        if let Some(descriptor) = &self.descriptor {
            // SAFETY:
            // `pidfd_send_signal` takes a descriptor,
            // a signal number,
            // a null `siginfo` and flags;
            // no memory is touched.
            let result: libc::c_long = unsafe {
                libc::syscall(libc::SYS_pidfd_send_signal, libc::c_long::from(descriptor.as_raw_fd()), libc::c_long::from(signal), 0 as libc::c_long, 0 as libc::c_long)
            };
            return if result == 0 { Delivered::Sent } else { Delivered::Gone };
        }
        // Without a pidfd the start time is read again right before the signal:
        // the window that remains is the pidfd's to close.
        if stat_of(self.pid).is_none_or(|stat| { return stat.started != self.started; }) {
            return Delivered::Gone;
        }
        // SAFETY:
        // `kill` takes two integers and touches no memory of this process.
        let result: libc::c_int = unsafe { libc::kill(self.pid, signal) };
        return if result == 0 { Delivered::Sent } else { Delivered::Gone };
    }
}

/// How many times the tree is searched for members that started after the last look before the kill goes out.
const SEARCH_PASSES: usize = 16;

/// End the tree below the pinned `root`:
/// stop every member,
/// search again for children the stopped members had started,
/// and kill
///  them all.
/// Only pinned processes are signalled,
/// so a process ID that was reused meanwhile is never reached.
pub fn end_tree(root: &Pin) {
    let mut members: Vec<Pin> = Vec::new();
    root.signal(libc::SIGSTOP);
    for _ in 0..SEARCH_PASSES {
        let mut found: Vec<Pin> = Vec::new();
        let parents: Vec<&Pin> = std::iter::once(root).chain(members.iter()).collect();
        for parent in parents {
            for child in children(parent.pid()) {
                if child == root.pid() || members.iter().chain(found.iter()).any(|known| { return known.pid() == child; }) {
                    continue;
                }
                if let Some(pin) = Pin::open_child(parent, child) {
                    pin.signal(libc::SIGSTOP);
                    found.push(pin);
                }
            }
        }
        if found.is_empty() {
            break;
        }
        members.extend(found);
    }
    for member in members.iter().rev() {
        member.signal(libc::SIGKILL);
    }
    root.signal(libc::SIGKILL);
}
