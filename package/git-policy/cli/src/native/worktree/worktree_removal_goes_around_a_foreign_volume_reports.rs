//! What:
//!  The words a removal that went around a volume prints,
//!  and the line a later `git worktree add` adds to Git's own refusal.
//! Why:
//!  Each of these lines is read by a person who has just lost a worktree to a command,
//!  or who finds a path taken by a directory Git no longer lists.
//!  Every one of them names the volumes that stayed,
//!  what each is and where it stands,
//!  and the commands that free the path,
//!  so no two of them can disagree about what was left.
//!  They are one subject because they are the same paragraph said on the routes a removal has:
//!  the person's own `git worktree remove`,
//!  its refusal,
//!  the pass that stopped,
//!  the landed sweep that removed a worktree nobody asked about,
//!  and the add that comes next.
//!  This file is one part of the module whose name it carries,
//!  split because that module was over the line limit:
//!  `worktree_removal_goes_around_a_foreign_volume.rs` holds the look for the volumes and the
//!  report of a removal that could not finish,
//!  `worktree_removal_goes_around_a_foreign_volume_records.rs` holds the record a later command shows,
//!  and this one holds the words a person reads.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const report = aroundRemovalReport('/repo/wt', left); // and the refusal, the sweep's line, the add's line
//! ```

use crate::shell_quote::shell_quote;
use crate::subvolume_remove_walk::Boundary;
use crate::worktree_removal_only_deletions::Reported;
use std::path::Path;

/// The prefix every line of this program carries.
pub(crate) const PREFIX: &str = "monochromatic-git-policy-cli: ";

/// What:
///  The words that give one volume and its path,
///  as a line of a list.
/// Why:
///   Every report names each volume the same way,
///  so a person who read one recognises the next.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = `  - ${what} at ${path}`;
/// ```
fn item(found: &Boundary) -> String {
    return format!(
        "  - {what} at {path}",
        what = found.what,
        path = found.path.display()
    );
}

/// What all of them are,
/// one line each.
pub(crate) fn items(left: &[Boundary]) -> String {
    let mut lines: Vec<String> = Vec::with_capacity(left.len());
    for found in left {
        lines.push(item(found));
    }
    return lines.join("\n");
}

/// The shell-quoted spelling of a path,
/// for a command a person copies.
fn quoted(path: &Path) -> String {
    return shell_quote(path.as_os_str().as_encoded_bytes());
}

/// What:
///  What frees the path once the person moves what was left out:
///  the `rmdir`s that take the empty directories,
///  and the one command that releases a volume of each kind.
/// Why:
///   A report that named only what was left would leave the person with a path nothing can be added at,
/// and the ways differ:
///  a mount point is released with `umount`,
///  which keeps what is mounted there;
/// a subvolume that is another repository's worktree is removed by that repository's own
///  command,
///  which keeps its branch.
/// `rmdir` is named first because it is the step that needs no decision about what a volume holds:
///  the directories it takes hold nothing by then.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const freeing = freeingThePath('/repo/wt');
/// ```
pub(crate) fn freeing_the_path(path: &Path) -> String {
    return format!(
        "Move each one out of the directory it stands in. The directories that then hold nothing go from the deepest one up \
         with `rmdir`, which the kernel allows for an empty subvolume root too, and {quoted} is free for `git worktree add` \
         once they are gone. A mount point is released with `umount`, which keeps what is mounted there; a subvolume that is \
         another repository's worktree goes with `git worktree remove <path>` run in that repository, which keeps its branch \
         and refuses while anything there is uncommitted.",
        quoted = quoted(path)
    );
}

/// What:
///  The one report a person's own `git worktree remove` prints when the worktree went and something stayed.
/// Why:
///   The command did what it was asked:
/// the worktree's own files are gone and Git no longer lists it.
/// It has to say the rest in the same breath,
///  because the path is still taken and a person who reads only "removed" looks for a free path:
/// what was
///  left,
///  that nothing of it was moved or deleted,
///  and what frees the path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const report = aroundRemovalReport('/repo/wt', left);
/// ```
pub fn around_removal_report(
    target: &Path,
    left: &[Boundary],
) -> String {
    return format!(
        "{PREFIX}removed the worktree {target}, and Git no longer lists it, but what someone else put inside it is left where \
         it stands, so the path is still taken:\n{items}\nThe directories that lead to each of them are left too, and every \
         other entry of the worktree was removed. Nothing of what is left was moved, unmounted or deleted.\n{freeing}",
        target = target.display(),
        items = items(left),
        freeing = freeing_the_path(target)
    );
}

/// What:
///  What Git says the worktree holds that a removal without `--force` would have to delete,
///  as the refusal of a removal beside a volume words it.
/// Why:
///   The report must not tell a person that nothing was removed when a tree in front of it is half gone,
/// and it must not offer a
///  remedy that cannot work:
/// what Git reports decides which of the three states the tree is in,
/// and each has its own
///  way on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type BesideAVolume = { reason: string; begun: boolean; reported: Reported };
/// ```
#[derive(Clone, Copy)]
pub struct BesideAVolume<'a> {
    /// What Git refuses to delete without `--force`,
    /// as the clean proof worded it.
    pub reason: &'a str,
    /// Whether a record of this program says an earlier removal of this very worktree emptied it and was cut
    /// short.
    pub begun: bool,
    /// What Git reports about the tree,
    /// leaving out what stands inside the volumes this removal keeps.
    pub reported: Reported,
}

/// What:
///  The refusal of a person's removal that would have to delete more than Git does without `--force`,
///  while a volume someone else put there stands below it.
/// Why:
///   A tree holding such a volume is never given to Git,
/// whose own removal would delete what is mounted at one,
///  so Git's own refusal cannot be the answer here.
/// The report says what this command did,
/// names the volumes once,
///  and words the way on by
/// what the tree in front of it is:
/// an earlier pass of this program that was cut short is finished by the same command once
///  its record proves it,
/// a tree whose entries are only missing is finished by `--force` because nothing else can
///  prove those deletions,
/// and a tree Git reports nothing about is no half-emptied tree at all:
/// it is refused
///  in words that say the proof of cleanliness could not settle it and what does end it.
///  It never claims to be Git's sentence,
/// because the reason it refuses may be one Git does not judge,
/// and it never says
///  nothing was removed when the tree shows otherwise,
/// nor that entries are missing when none is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const report = refusedBesideAVolumeReport({ target, operand: '../wt', left, beside });
/// ```
pub fn refused_beside_a_volume_report(
    target: &Path,
    operand: &str,
    left: &[Boundary],
    beside: BesideAVolume<'_>,
) -> String {
    let way_on: String = match (beside.begun, beside.reported) {
        // A record of this program proves the missing entries are its own, and the removal then finishes without a word
        // about them; this branch is unreachable while that record is read, and is worded for a record that went away
        // between the two readings.
        (true, Reported::OnlyDeletions) => String::from(
            "A record of this program says an earlier removal of this worktree was cut short, and the entries Git reports \
             missing are the ones it removed: running this removal again finishes it, and what is named above stays exactly \
             as it is.",
        ),
        (true, Reported::Nothing | Reported::Something) => format!(
            "A record of this program says an earlier removal of this worktree was cut short, and what stands there now is \
             more than the entries it removed: {reason}. `git worktree remove --force {operand}` finishes this removal, and \
             what is named above stays exactly as it is.",
            reason = beside.reason,
            operand = operand
        ),
        (false, Reported::OnlyDeletions) => format!(
            "Git reports nothing about the worktree but entries missing from it, apart from what stands inside the volumes \
             named above: {reason}. No record of this program says a removal of this worktree removed them, so nothing here \
             proves they are not yours: `git worktree remove --force {operand}` finishes this state, and what is named above \
             stays exactly as it is. Moving what is named above out would not finish it, because the missing entries would \
             still be missing.",
            reason = beside.reason,
            operand = operand
        ),
        // Git names nothing at all: no entry is missing, no change stands there, and the proof of cleanliness is what
        // refused. Nothing is half emptied, so nothing of the pass's wording applies.
        (false, Reported::Nothing) => format!(
            "Git reports no change in the worktree at all, and the proof of cleanliness this removal makes could not settle it: \
             {reason}. This is no state a pass of this program left: the tree holds what Git already holds, and only the \
             question of whether the files are the ones the index names is open. Nothing was removed, and what is named above \
             stays exactly as it is. `git worktree remove --force {operand}` removes every other entry of the worktree now; \
             to keep what Git would delete as well, move what is named above out and run this removal again, which then \
             hands the tree to Git.",
            reason = beside.reason,
            operand = operand
        ),
        (false, Reported::Something) => format!(
            "What Git refuses to delete without `--force` stands there too: {reason}. `git worktree remove --force {operand}` \
             removes every other entry of the worktree and leaves what is named above exactly as it is. To keep what Git would \
             delete as well, move that out first and run this removal again; to keep everything, move what is named above out \
             and run it again.",
            reason = beside.reason,
            operand = operand
        ),
    };
    return format!(
        "{PREFIX}this removal changed nothing. The worktree {target} holds what someone else put inside it, so no removal of \
         it may be handed to Git, whose own removal would delete what is mounted at a mount point and cannot destroy a subvolume \
         that contains one:\n{items}\n{way_on}",
        target = target.display(),
        items = items(left),
        way_on = way_on
    );
}

/// What:
///  The report of a removal that went around a volume and found it gone when its pass ended:
///  the worktree's entries went one by one,
///  the empty root went with one `rmdir`,
///  and nothing was handed to the commit service.
/// Why:
///   The line for a removal whose subvolume was destroyed says the destroy took it in one step and that the
///   commit service deletes the files after the command returns.
/// On this path neither is true:
/// the pass deleted the files before the command
///  returned,
/// no hand-over record was written,
/// and the root went with one `rmdir` because the volume that had kept it
///  standing was gone.
/// A person who read the destroy's words would look for a queued delete that does not exist,
/// and
///  for a worktree that is a plain directory they would also be told of a subvolume that never was.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const report = emptiedAroundReport('/repo/wt');
/// ```
pub fn emptied_around_report(target: &Path) -> String {
    return format!(
        "{PREFIX}removed the worktree {target}, and Git no longer lists it: this command deleted its entries one by one, the \
         subvolume or mount point that stood inside it had gone by the time that pass ended, and the root went with one \
         `rmdir`. Nothing was handed to the commit service, so no delete waits behind this command.",
        target = target.display()
    );
}

/// What:
///  The report of a removal that removed what it could and stopped,
///  so the worktree is partly emptied and still registered.
/// Why:
///   A step that fails halfway leaves files gone and a registration standing,
/// and a person told either "removed" or
///  "nothing was removed" would believe the wrong one.
/// The report says which happened,
/// what stopped it,
///  and what finishes it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const report = stoppedAroundReport(target, reason);
/// ```
pub fn stopped_around_report(
    target: &Path,
    reason: &str,
) -> String {
    return format!(
        "{PREFIX}removing the worktree {target} stopped before it finished: every entry it removed is gone, and the worktree is \
         still registered, so Git still lists it. What stopped it is: {reason}\nNothing that someone else put inside the \
         worktree was moved, unmounted or deleted. Fix what the reason names and run the same removal again; it goes on from \
         where this one stopped.",
        target = target.display(),
        reason = reason
    );
}

/// What:
///  The line a landed sweep prints for a worktree it removed and left something in.
/// Why:
///   The sweep removed a person's worktree without being asked,
/// so its line has to say that the worktree is gone from
///  Git's records,
///  what stayed where it stands,
///  and what frees the path:
/// a person who read only "removed" would try to
///  add a worktree at a path that is still taken.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = landedAroundLine('main', '/repo/wt', 'topic', left);
/// ```
pub fn landed_around_line(
    landed_branch: &str,
    worktree: &Path,
    branch: &str,
    left: &[Boundary],
) -> String {
    return format!(
        "{PREFIX}removed the worktree {worktree} by itself, because its branch {branch} is now on {landed}: every other entry of \
         it was removed and Git no longer lists it, and what someone else put inside it is left where it stands, so the path is \
         still taken:\n{items}\nThe branch stays. Nothing of what is left was moved, unmounted or deleted.\n{freeing}",
        worktree = worktree.display(),
        branch = branch,
        landed = landed_branch,
        items = items(left),
        freeing = freeing_the_path(worktree)
    );
}

/// What:
///  The line a `git worktree add` prints when the destination holds what a removal left,
///  or nothing when nothing is there.
/// Why:
///   Git's sentence for an occupied destination says only that the directory exists
/// (`fatal: <path> already exists`),
///  and the person who removed the worktree there was told a volume stayed.
/// The line names each volume and connects the two,
/// so that "already exists" is answered by what stands there,
///  and it comes from the same look the removal makes,
/// which is why the two cannot disagree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = addBesideAVolumeLine(destination, left);
/// ```
pub fn add_beside_a_volume_line(
    destination: &Path,
    left: &[Boundary],
) -> String {
    return format!(
        "{PREFIX}the directory {destination} is not empty because a removal of the worktree there had to leave what someone else \
         put inside it:\n{items}\nNothing of it was moved or deleted. {freeing}",
        destination = destination.display(),
        items = items(left),
        freeing = freeing_the_path(destination)
    );
}
