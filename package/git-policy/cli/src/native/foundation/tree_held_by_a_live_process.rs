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
//!  The claim is a `flock` a holder keeps on the tree's own directory: shared by every holder, so two holders can hold
//!  one tree at once and a tree changes hands without an instant in which nobody holds it,
//!  and asked exclusively by a sweep, so any holder at all is an answer.
//!  The kernel holds it, no clock moves it, and it is released the moment the holding process ends,
//!  however it ends,
//!  so a killed holder leaves nothing behind and nothing to collect.
//!  A sweep that has decided to remove a tree takes the claim and keeps it through the removal,
//!  so no holder can begin in a tree that is going:
//!  a holder that cannot take the claim gives the tree up rather than build in it.
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
//! const held = hold(tree);                       // a holder's share of the claim, while it lives
//! const answer = whetherHeld(tree);              // 'a-process-holds-it' | 'no-process-holds-it' | 'cannot-be-asked'
//! ```

/// A path that may or may not exist.
use std::path::Path;

/// What:
///  The claim one live process keeps on a tree's own directory,
///  which a sweep may not remove the tree under however old the tree looks.
/// Why:
///   The claim is a shared `flock` on a descriptor of the tree's directory,
///  so it may be held by two holders at once, it belongs to this process and to no clock,
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
///  What an attempt to take a holder's share of the claim came to.
/// Why:
///  A caller has three things to do with the three answers and cannot do them with two:
///  a claim it took is one it keeps,
///  a tree a sweep holds exclusively is one being removed,
///  so the caller must give the tree up rather than build in it,
///  and a volume that cannot answer leaves the tree to its modification time as it was before the claim existed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ClaimTaken = 'yes' | 'being-taken' | 'cannot-be-asked';
/// ```
#[derive(Debug)]
pub enum ClaimTaken {
    /// The claim is this holder's until the value goes.
    Yes(HeldTree),
    /// A sweep holds it exclusively,
    ///  so it is being removed.
    BeingTaken,
    /// The volume cannot say whether the tree is held.
    CannotBeAsked,
}

/// What:
///  Takes this holder's share of the claim on `tree`,
///  or says why it could not.
/// Why:
///  The claim is shared, so another holder never stands in this one's way:
///  the only thing that refuses a holder is a sweep that has taken the claim exclusively,
///  and that is a sweep removing this very tree.
///  So a holder that is refused gives the tree up rather than build in a tree that is going,
///  and nothing waits at all: a tree being removed is no tree to keep a claim on.
///  A holder on a volume that cannot answer whether the tree is held keeps it as a holder did before this claim
///  existed: its modification time is renewed while it lives and the sweep reads that.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const held = hold(tree); // 'yes' | 'being-taken' | 'cannot-be-asked'
/// ```
pub fn hold(tree: &Path) -> ClaimTaken {
    return match lock_without_waiting(tree, Share::OfAHolder) {
        Ok(opened) => ClaimTaken::Yes(HeldTree { _opened: opened }),
        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
            debug_claim(tree, "a sweep holds it to remove it");
            ClaimTaken::BeingTaken
        }
        Err(error) => {
            debug_claim(tree, format!("the volume cannot say whether it is held ({error})").as_str());
            ClaimTaken::CannotBeAsked
        }
    };
}

/// What:
///  Takes the claim on `tree` for a sweep that means to remove it,
///  which is the whole of it and which every holder's share stands in the way of,
///  and holds it until the removal is over.
/// Why:
///  A sweep that asked "does anybody hold it" and then removed would let a holder begin in the tree between the question
///  and the removal, and the removal would take a tree a commit had claimed and written into.
///  Taking the claim makes the question and the removal one step:
///  no holder can begin while the claim is taken, and a holder that arrives at that instant is refused and gives the tree
///  up instead of building in it.
///  It is taken without waiting: a tree another sweep is removing is one this sweep leaves for the next.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const claim = takeToRemove(tree); // 'yes' | 'being-taken' | 'cannot-be-asked'
/// ```
pub fn take_to_remove(tree: &Path) -> ClaimTaken {
    return match lock_without_waiting(tree, Share::Alone) {
        Ok(opened) => ClaimTaken::Yes(HeldTree { _opened: opened }),
        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => ClaimTaken::BeingTaken,
        Err(error) => {
            debug_claim(tree, format!("the volume cannot say whether it is held ({error})").as_str());
            ClaimTaken::CannotBeAsked
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
    return match lock_without_waiting(tree, Share::Alone) {
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
///  The removal claim on `tree`, for a control that stands in for a sweep.
/// Why:
///  Every sweep takes a tree to remove it through `remove_as_a_sweep`,
///  so the take itself is this module's own;
///  a control has to hold the claim at an instant no sweep can be interrupted at,
///  and this is the only door left open for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// takeToRemoveForAControl(tree) // test builds only
/// ```
#[cfg(test)]
pub(crate) fn take_to_remove_for_a_control(tree: &Path) -> ClaimTaken {
    return take_to_remove(tree);
}

/// What:
///  How much of the claim a call wants:
///  one holder's share, which another holder may take at the same time,
///  or the whole of it, which only a sweep that means to remove the tree takes.
/// Why:
///   The two are one `flock` of one descriptor and differ in one flag,
///  and naming them keeps the flag words out of every call site,
///  so the one place the claim is taken is the one place its reason is written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Share = 'of-a-holder' | 'alone';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Share {
    /// One holder's share,
    /// which another holder can take at the same time.
    OfAHolder,
    /// The whole claim,
    /// which nobody else holds.
    Alone,
}

impl Share {
    /// What:
    ///  The kernel's flag for this share of the claim.
    /// Why:
    ///  The flag words stand here and beside the one take below, and nowhere else in this module.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// share === 'of-a-holder' ? LOCK_SH : LOCK_EX
    /// ```
    fn of_the_kernel(self) -> libc::c_int {
        return match self {
            // Lock kept: the share flag of one tree's own flock, whose reason stands beside the one take of it below.
            Share::OfAHolder => libc::LOCK_SH,
            Share::Alone => libc::LOCK_EX,
        };
    }
}

/// What:
///  The tree's directory, with the exclusive claim on it,
///  or why the claim could not be taken.
/// Why:
///  The claim is one `flock` of one descriptor,
///  and the one place it is taken keeps the two questions above from drifting apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const file = open(tree, O_RDONLY | O_NOFOLLOW | O_NONBLOCK); flock(file, mode | LOCK_NB) === 0 ? file : throw errno
/// ```
fn lock_without_waiting(tree: &Path, share: Share) -> std::io::Result<std::fs::File> {
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
    // Lock kept:
    //  the `flock` of one tree's own directory, shared by its holders and asked exclusively by a sweep,
    //  taken without waiting,
    //  by the processes that build in that tree and by the sweep that collects it,
    //  so a sweep can ask a question no wall-clock step can answer wrongly,
    //  two holders can hold one tree while it changes hands and there is no instant in which nobody holds it,
    //  and a sweep that takes it holds it through the removal so no holder begins in a tree that is going.
    //  It covers one tree and no other state,
    //  it is never waited for beyond a holder's own fifty milliseconds,
    //  and the kernel releases it when the holder ends,
    //  so a killed holder leaves nothing held and nothing to collect.
    let taken: libc::c_int = unsafe { libc::flock(opened.as_raw_fd(), share.of_the_kernel() | libc::LOCK_NB) };
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
