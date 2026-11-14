//! What:
//!  Whether a tree is held by a live process,
//!  asked in a way no clock can answer wrongly:
//!  the claim a holder keeps on a tree's own directory,
//!  and the question a sweep asks of it.
//! Why:
//!  A sweep removes a tree nothing holds by the tree's modification time,
//!  and a wall clock that steps forward (the time synchronisation a machine does after a suspend and a resume)
//!  makes a tree written moments ago look hours old,
//!  so a sweep then deletes a tree a commit is building in, and that commit fails
//!  (`shadow_drawn.rs`,
//!  `nothing_holds`;
//!  `service_scratch_abandonment.rs`,
//!  `may_go`).
//!  A modification time cannot tell a live tree from an abandoned one across such a step, and no arithmetic on the wall
//!  clock can: the clock the mark was written with is not the clock the sweep reads.
//!  The claim is the exclusive `flock` a holder keeps on the tree's own directory.
//!  The kernel holds it, no clock moves it, and it is released the moment the holding process ends,
//!  however it ends,
//!  so a killed holder leaves nothing behind and nothing to collect.
//!  A tree nobody holds is still judged by its age:
//!  the modification time remains what says how long an unheld tree has stood,
//!  and a clock step in the other direction only makes an abandoned tree wait longer.
//!  A boot moves nothing either: no process survives it,
//!  so every tree is unheld after one and the age decides,
//!  which is right, because nothing can still be building in a tree when no process of that boot exists.
//!  What the claim cannot answer is a file system without `flock`,
//!  so the question has a third answer and each caller says what it does with it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const held = hold(tree);                       // the holder keeps it while it lives
//! const answer = whetherHeld(tree);              // 'a-process-holds-it' | 'no-process-holds-it' | 'cannot-be-asked'
//! ```

/// A path that may or may not exist.
use std::path::Path;

/// What:
///  The claim one live process keeps on a tree's own directory,
///  which a sweep may not remove the tree under however old the tree looks.
/// Why:
///   The claim is an exclusive `flock` on a descriptor of the tree's directory,
///  so it belongs to this process and to no clock,
///  and it ends when this value goes or when the process does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class HeldTree { #file; } // close() releases the claim
/// ```
#[derive(Debug)]
pub struct HeldTree {
    /// The descriptor the kernel's lock is held through.
    ///  Nothing reads it: holding it is the whole of the claim.
    _opened: std::fs::File,
}

/// What:
///  What a sweep learns when it asks whether a tree is held.
/// Why:
///  Three answers, not two:
///  a file system without `flock` cannot say whether a live process holds a tree,
///  and a caller that turned that into "nobody holds it" would let a clock step delete a live tree on such a volume,
///  while a caller that turned it into "somebody holds it" would never collect an abandoned tree there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WhetherHeld = 'a-process-holds-it' | 'no-process-holds-it' | 'cannot-be-asked';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WhetherHeld {
    /// A live process holds the tree,
    ///  so no age may remove it.
    ByALiveProcess,
    /// No process holds the tree,
    ///  so its age is what decides.
    ByNobody,
    /// The volume cannot be asked,
    ///  so a caller decides what it does without an answer.
    CannotBeAsked,
}

/// What:
///  Takes the claim on `tree`,
///  which is held until the value goes,
///  or answers nothing when the volume refuses the claim.
/// Why:
///  A holder that cannot claim its tree keeps it exactly as a holder did before this claim existed:
///  its modification time is renewed while it lives and the sweep reads that,
///  so no holder loses anything by a volume without `flock`.
///  The claim is never waited for: a tree a sweep is asking about at this instant is one this holder keeps without a
///  claim rather than one it waits on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const held = hold(tree); // undefined when no claim could be taken
/// ```
pub fn hold(tree: &Path) -> Option<HeldTree> {
    return match lock_without_waiting(tree) {
        Ok(opened) => Some(HeldTree { _opened: opened }),
        Err(error) => {
            debug_claim(tree, format!("this command holds it without a claim ({error})").as_str());
            None
        }
    };
}

/// What:
///  Whether a live process holds `tree` now.
/// Why:
///  A sweep asks this of a tree its age would remove, and nothing else:
///  a tree the age keeps needs no second witness,
///  so the question costs nothing on the ordinary sweep.
///  The question takes the claim for the instant it asks and gives it back at once,
///  so two sweeps at the same moment never both decide against a holder:
///  the second finds the first's question and leaves the tree,
///  and the tree goes at the next sweep.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// whetherHeld(tree) // 'a-process-holds-it' | 'no-process-holds-it' | 'cannot-be-asked'
/// ```
pub fn whether_held(tree: &Path) -> WhetherHeld {
    return match lock_without_waiting(tree) {
        // The claim was free, so no live process holds the tree; giving it back is the whole of the release.
        Ok(_free) => WhetherHeld::ByNobody,
        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => WhetherHeld::ByALiveProcess,
        Err(error) => {
            debug_claim(tree, format!("whether a live process holds it cannot be asked ({error})").as_str());
            WhetherHeld::CannotBeAsked
        }
    };
}

/// What:
///  The tree's directory, with the exclusive claim on it,
///  or why the claim could not be taken.
/// Why:
///  The claim is one `flock` of one descriptor,
///  and the one place it is taken keeps the two questions above from drifting apart.
///
/// Lock kept:
///  the exclusive `flock` of one tree's own directory,
///  taken without waiting,
///  by the process that builds in that tree,
///  so a sweep can ask a question no wall-clock step can answer wrongly.
///  It covers one tree and no other state,
///  it is never waited for,
///  and the kernel releases it when the holder ends,
///  so a killed holder leaves nothing held and nothing to collect.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const file = open(tree, O_RDONLY | O_NOFOLLOW | O_NONBLOCK); flock(file, LOCK_EX | LOCK_NB) === 0 ? file : throw errno
/// ```
fn lock_without_waiting(tree: &Path) -> std::io::Result<std::fs::File> {
    use std::os::fd::AsRawFd as _;
    use std::os::unix::fs::OpenOptionsExt as _;
    // The open must not wait either: an entry whose open waits for a writer is one a control stands in for the volume
    //  with, and a name that is a link is not this program's tree, so the claim refuses both rather than waiting or
    //  locking through them.
    let opened: std::fs::File = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(tree)?;
    // SAFETY: `flock` takes a descriptor of a file this call opened and a flag word, and touches no memory of this process.
    //  The descriptor stays open for as long as the `File` lives, which is at least until the lock is released.
    let taken: libc::c_int = unsafe { libc::flock(opened.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
    if taken != 0 {
        return Err(std::io::Error::last_os_error());
    }
    return Ok(opened);
}

/// What:
///  Says in the diagnostic log why a claim was not taken or not asked,
///  and nothing on the terminal.
/// Why:
///  A volume without `flock` is nothing the person can act on,
///  and the tree is kept by its modification time either way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// debug('treeClaim', `${words}: ${tree}`)
/// ```
fn debug_claim(tree: &Path, words: &str) {
    crate::diagnostic_log::debug("treeClaim", format!("{}: {words}", tree.display()).as_str());
}

/// The claim and the question are controlled in the test module.
#[cfg(test)]
#[path = "tree_held_by_a_live_process_tests.rs"]
mod tests;
