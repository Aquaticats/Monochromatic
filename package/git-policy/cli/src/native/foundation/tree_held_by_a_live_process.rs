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
///  A tree whose path is in hand only with the claim that keeps a sweep off it.
/// Why:
///  A maker that is told its tree is being removed has one thing it can do,
///  which is to leave the tree and make another:
///  a path that came back without a claim is a path a command would build in while a sweep removed it,
///  and `ClaimOnAMadeTree` gives no path back on that answer,
///  so using one afterwards is not something a caller can write.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class AClaimedTree { #tree; #claim; path(); intoParts() }
/// ```
#[derive(Debug)]
pub struct AClaimedTree {
    /// Where the tree is.
    tree: std::path::PathBuf,
    /// The claim that keeps every sweep off it while this value lives.
    claim: HeldTree,
}

impl AClaimedTree {
    /// Where the tree is.
    pub fn path(&self) -> &Path {
        return self.tree.as_path();
    }

    /// The tree and the claim that keeps it,
    /// for a value that renews the one and keeps the other.
    pub fn into_parts(self) -> (std::path::PathBuf, HeldTree) {
        return (self.tree, self.claim);
    }
}

/// What:
///  What a claim on a tree made a moment ago came to.
/// Why:
///  Three answers and three things to do:
///  a claimed tree is one this command may build in,
///  a tree a sweep took is one it must leave and make another instead,
///  and a volume that cannot answer whether a tree is held leaves it to its modification time,
///  which is how every tree was kept before the claim existed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ClaimOnAMadeTree = { claimed: AClaimedTree } | { theVolumeCannotSay: Path }
/// ```
#[derive(Debug)]
pub enum ClaimOnAMadeTree {
    /// The tree is this holder's until the claim goes,
    ///  and a sweep that arrives meanwhile is refused.
    Claimed(AClaimedTree),
    /// The volume cannot say who holds it,
    ///  so the tree is kept by its modification time alone,
    ///  as every tree was before the claim existed.
    TheVolumeCannotSay(std::path::PathBuf),
}

/// What:
///  A tree a sweep had already taken to remove,
///  so nothing of it is a maker's to use.
/// Why:
///  It carries no path:
///  the whole of this answer is that the tree is going,
///  and a caller that could still name it would be able to build in a tree a removal is emptying.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class TreeIsGoing {} // no path, by construction
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TreeIsGoing;

/// What:
///  Takes this maker's claim on a tree it has just made,
///  or says the tree is being removed and hands back nothing.
/// Why:
///  A tree is a maker's from the instant it takes the claim,
///  and a sweep that took the claim first is emptying it:
///  the maker must leave that tree and make another, which is why the path is consumed here
///  and only given back with a claim (`AClaimedTree`) or as a tree the volume cannot answer about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// claimAMadeTree(tree) // claimed | the volume cannot say | throws TreeIsGoing
/// ```
pub fn claim_a_made_tree(tree: std::path::PathBuf) -> Result<ClaimOnAMadeTree, TreeIsGoing> {
    return match hold(tree.as_path()) {
        ClaimTaken::Yes(claim) => Ok(ClaimOnAMadeTree::Claimed(AClaimedTree { tree, claim })),
        ClaimTaken::BeingTaken => Err(TreeIsGoing),
        ClaimTaken::CannotBeAsked => Ok(ClaimOnAMadeTree::TheVolumeCannotSay(tree)),
    };
}

/// What came of a sweep's attempt to remove one tree.
/// Why:
///  A sweep has three answers to give its caller:
///  the removal ran (and the removal says whether it finished),
///  a live holder had the tree,
///  and nothing was removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SweptRemoval = { removed: Removal } | { heldByALiveProcess: true }
/// ```
#[derive(Debug)]
pub enum SweptRemoval {
    /// The claim was taken and the removal ran;
    ///  the removal says whether the tree is gone.
    Removed(crate::scratch_entry_removal::Removal),
    /// A live holder has the tree,
    ///  so nothing was removed and the next sweep asks again.
    HeldByALiveProcess,
}

/// What:
///  Removes one tree under the removal claim,
///  which is held from before the removal starts until it is over.
/// Why:
///  A sweep that looked and let go would let a holder begin in the tree between its question and its removal,
///  and the removal would take a tree a command had claimed and written into
///  (`shadow_pool.rs`,
///  `sweep_abandoned`;
///  `shadow_drawn.rs`,
///  `sweep_drawn_folder`;
///  `scratch_location.rs`,
///  `sweep_stale`).
///  Every sweep removes through this one function, so a sweep cannot be written that looks and lets go:
///  the claim is taken here, no caller holds `take_to_remove` at all,
///  and a holder that arrives while the claim is taken is refused and gives its tree up
///  (`claim_a_made_tree`).
///  A volume that cannot answer whether the tree is held leaves the removal to the age that decided it,
///  as before the claim existed, and the log names it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// removeAsASweep(tree, (path, claim) => removeWithin(path, ceiling, claim)) // removed | heldByALiveProcess
/// ```
pub fn remove_as_a_sweep<F>(tree: &Path, remove: F) -> SweptRemoval
where
    F: FnOnce(&Path, Option<HeldTree>) -> crate::scratch_entry_removal::Removal,
{
    let claim: Option<HeldTree> = match take_to_remove(tree) {
        ClaimTaken::Yes(claim) => Some(claim),
        ClaimTaken::BeingTaken => return SweptRemoval::HeldByALiveProcess,
        ClaimTaken::CannotBeAsked => {
            debug_claim(tree, "the volume cannot say who holds it, so the removal goes by its age");
            None
        }
    };
    // What: The claim goes to the removal, which keeps it for as long as it is deleting.
    // Why:  A removal that outlived its ceiling goes on deleting on its own thread, and a claim dropped here would let a
    //       commit take the tree and build in it while the old removal emptied it
    //       (`scratch_entry_removal.rs`,
    //       `remove_within`).
    //       So the caller of this function never holds it after the removal started: either the removal ended and the
    //       claim is spent, or it is still running and the claim is its own.
    let outcome: crate::scratch_entry_removal::Removal = remove(tree, claim);
    return SweptRemoval::Removed(outcome);
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
fn hold(tree: &Path) -> ClaimTaken {
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
///  and holds it until the removal is over,
///  which is the removal's thread's end and not the caller's wait for it.
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
fn take_to_remove(tree: &Path) -> ClaimTaken {
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
///  No sweep asks this: every sweep removes through `remove_as_a_sweep`,
///  which takes the claim and keeps it through the removal,
///  so a look that let go was the defect a review found in the pool's sweep
///  and no sweep can make that mistake through this module.
///  What asks it is a control, which has to observe who holds a tree at an instant it cannot interrupt,
///  and a reader of the code, which needs the three answers named.
///  The question takes the claim for the instant it asks and gives it back at once.
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
    // What: The open and the `flock` run on a thread of their own, and this caller waits no longer than the ceiling.
    // Why:  Both are synchronous calls into the kernel, and a directory on a mount whose server stopped answering never
    //       returns either of them:
    //       `O_NONBLOCK` limits neither, and `LOCK_NB` only avoids sleeping for a competing lock.
    //       The package draws that line once, in `work_awaited_within_a_ceiling.rs`, and the claim goes through it,
    //       so a volume that never answers costs a command the ceiling and no more
    //       (`doc/planning/cli-git-no-unbounded-waits.md`).
    //       A job that gave no answer is a volume that cannot be asked, which is the third answer the claim already has.
    let wanted: std::path::PathBuf = tree.to_path_buf();
    let answered: crate::work_awaited_within_a_ceiling::TheAnswerOfAThreadOfWork<std::io::Result<std::fs::File>> =
        crate::work_awaited_within_a_ceiling::work_within_reporting(THE_CLAIM_IS_ASKED_FOR_AT_MOST, move || {
            return lock_now(wanted.as_path(), share);
        });
    return match answered {
        crate::work_awaited_within_a_ceiling::TheAnswerOfAThreadOfWork::Ran(outcome) => outcome,
        crate::work_awaited_within_a_ceiling::TheAnswerOfAThreadOfWork::NoThreadToStart => {
            Err(std::io::Error::other("the system refused a thread to ask whether this tree is held, so nothing was asked"))
        }
        crate::work_awaited_within_a_ceiling::TheAnswerOfAThreadOfWork::NoAnswer => Err(std::io::Error::new(
            std::io::ErrorKind::TimedOut,
            format!("whether this tree is held cannot be asked within {THE_CLAIM_IS_ASKED_FOR_AT_MOST:?}"),
        )),
    };
}

/// How long the claim may take to be answered, and why.
/// Why this figure:
///  The claim is one `open` and one `flock` of a directory that is already this program's own,
///  which is a few microseconds on a healthy volume and was written down as "never waited for beyond a holder's own fifty
///  milliseconds" beside the take itself;
///  a volume that has not answered in that time is one this command cannot wait for, and the tree is left to its age.
pub(crate) const THE_CLAIM_IS_ASKED_FOR_AT_MOST: std::time::Duration = std::time::Duration::from_millis(50);

/// What:
///  The open and the `flock` themselves,
///  on the thread `lock_without_waiting` gives them.
/// Why:
///  It is the call the kernel may hold; the caller's wait for it is bounded and this is what it is bounded around.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const file = open(tree, O_RDONLY | O_NOFOLLOW | O_NONBLOCK); flock(file, mode | LOCK_NB) === 0 ? file : throw errno
/// ```
fn lock_now(tree: &Path, share: Share) -> std::io::Result<std::fs::File> {
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
