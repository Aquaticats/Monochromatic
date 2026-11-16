//! What:
//!  Ending a commit service,
//!  through a handle the kernel ties to the one process the note names,
//!  and only for a service this command may end.
//! Why:
//!   Reading a process's birth identity and sending its signal are two operations,
//! and the kernel may give the number to another process between them,
//! so a signal by number after a proof still lands on whatever holds the number when the call runs
//! (the review of `567ae37198`, finding 4).
//!  A pidfd is made while the number is proven to name the note's process,
//! and the signal travels through that descriptor,
//! which the kernel keeps pointed at that process for good;
//! a number that was reused is a number the descriptor never named.
//!  The second question is whose service this is:
//! a manager's fallback, having seen no greeting, used to end whichever service the shared note named,
//! which can be another command's service that binds between the two starts
//! (the review of `567ae37198`, finding 3).
//!  So a caller that ends a service only because it started one says which process it started,
//! and a service the note names that is not that process is left alone.
//!  A caller whose ending is the person's own command or a removed worktree names no process,
//! and ends the service the note proves, which is the service of that worktree.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const ended = endTheService(dir, instance, { only: leader }); // 'sent' | 'gone' | 'unproven' | 'not-ours'
//! ```

#[cfg(target_os = "linux")]
use crate::service_server_pid_note_of_an_instance::{ServerProcess, path as pid_note, read as read_note};
#[cfg(not(target_os = "linux"))]
use crate::service_server_pid_note_of_an_instance::{ServerProcess, is_still_the_process, path as pid_note, read as read_note};
use std::path::Path;

/// What:
///  What ending one service came to.
/// Why:
///   Four answers must stay apart:
/// the signal was taken;
/// the process was already gone, so nothing was sent;
/// nothing could be proven, so nothing was sent and nothing is known;
/// and the service is not the one this command started, so it was left alone.
///  A caller that records a stop records it only for `Sent`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Ended = 'sent' | 'gone' | 'unproven' | 'not-ours';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Ended {
    /// The kernel took the signal for the process the note names.
    Sent,
    /// No process the note names is there any more, so nothing was sent.
    Gone,
    /// Nothing was sent, and nothing is known about the service:
    /// the note could not be read as one this program wrote,
    /// the kernel refused a handle,
    /// or the platform has no handle to give.
    Unproven,
    /// The note names a service this command did not start, which is left alone.
    NotOurs,
}

/// What:
///  Ends the service the instance's own note names,
///  when `only` is nothing or is that service's number.
/// Why:
///   A service the note proves is the service of that worktree,
/// and a caller that started one itself passes the number it started,
/// so another command's service is never the one ended
/// (the review of `567ae37198`, findings 3 and 4).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// endTheService(dir, instance, leader) // 'sent' when the note names leader and the signal is taken
/// ```
pub fn end_the_service(directory: &Path, instance: &str, only: Option<i32>) -> Ended {
    let Some(service): Option<ServerProcess> = read_note(pid_note(directory, instance).as_path()) else { return Ended::Gone };
    if let Some(mine) = only
        && mine != service.pid
    {
        return Ended::NotOurs;
    }
    return end_through_a_handle(&service);
}

/// What:
///  Ends the service the note names,
///  which is what a person's stop and a removed worktree's sweep do.
/// Why:
///   Both act on the service of this worktree,
/// and the note's birth identity is what says the service is that one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// endTheServiceTheNoteNames(dir, instance) // true when the service is not there any more
/// ```
pub fn end_the_service_the_note_names(directory: &Path, instance: &str) -> bool {
    return matches!(end_the_service(directory, instance, None), Ended::Sent | Ended::Gone);
}

/// What:
///  The process a note names, as the tree reader names a member:
///  its number and the start tick its birth identity holds.
/// Why:
///   The descriptor is made for a member, and the member's start time is the fact the kernel's own record holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// memberOf(note) // { pid: note.pid, started: Number(note.identity.slice('linux:'.length)) }
/// ```
#[cfg(target_os = "linux")]
fn member_of_a_note(service: &ServerProcess) -> Option<crate::process_tree::TreeMember> {
    let started: u64 = service.identity.strip_prefix(crate::process_identity::LINUX_IDENTITY_PREFIX)?.parse().ok()?;
    return Some(crate::process_tree::TreeMember { pid: service.pid, started });
}

/// What:
///  The signal, sent through the descriptor the kernel made for the note's process.
/// Why:
///   The descriptor is the only handle a number cannot outlive:
/// it names one process, and `pidfd_send_signal` reaches that process or fails
/// (the review of `567ae37198`, finding 4).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pinned = PinnedMember.open({ pid, started }); pinned.kind === 'held' ? pinned.member.signal('SIGKILL') : ...
/// ```
#[cfg(target_os = "linux")]
fn end_through_a_handle(service: &ServerProcess) -> Ended {
    let Some(member) = member_of_a_note(service) else { return Ended::Unproven };
    return match crate::member_signalled_through_its_pidfd::PinnedMember::open(&member) {
        crate::member_signalled_through_its_pidfd::Pinned::Gone => Ended::Gone,
        crate::member_signalled_through_its_pidfd::Pinned::Unproven => Ended::Unproven,
        crate::member_signalled_through_its_pidfd::Pinned::Held(pinned) => match pinned.signal(libc::SIGKILL) {
            crate::member_signalled_through_its_pidfd::Delivered::Sent => Ended::Sent,
            crate::member_signalled_through_its_pidfd::Delivered::Gone => Ended::Gone,
            crate::member_signalled_through_its_pidfd::Delivered::Unproven => Ended::Unproven,
        },
    };
}

/// What:
///  The platform's own way to end the process the note proves,
///  which is the number and a birth check, and the interval between them that only Linux closes.
/// Why:
///   A platform without pidfds has no handle to tie the signal to the process,
/// and this program ships on Linux, where the descriptor is used.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (!stillTheProcess(note)) return 'gone'; kill(note.pid, 'SIGKILL'); return 'sent';
/// ```
#[cfg(not(target_os = "linux"))]
fn end_through_a_handle(service: &ServerProcess) -> Ended {
    if !is_still_the_process(service) {
        return Ended::Gone;
    }
    // SAFETY: a positive number and one signal, and no memory touched; the birth identity was read just above.
    let _signalled: libc::c_int = unsafe { libc::kill(service.pid, libc::SIGKILL) };
    return Ended::Sent;
}

/// Controls of the door stay out of the release executable.
#[cfg(test)]
#[path = "service_ended_through_a_pinned_handle_tests.rs"]
mod tests;
