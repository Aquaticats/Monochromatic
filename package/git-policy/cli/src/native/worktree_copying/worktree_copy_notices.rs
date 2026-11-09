//! What:
//!  Whose destination a creating command's copy may write into:
//!  the answer,
//!  how it is read from the reservation,
//!  and the line that says
//!  so when the destination is not this command's.
//! Why:
//!   The copy writes the source's ignored state into a worktree,
//! and only the command that made the destination
//!  may do it:
//! another command's registration stands at the same path whenever two adds name
//!  one destination,
//! and Git's own status does not say whose it is.
//! The reservation decides it before any
//!  Git runs,
//! and its identity says whether that directory is still the one at the path.
//! A worktree that receives no
//!  copy must say so,
//! and the person needs the two facts that decide what to do:
//! which command made
//!  the destination,
//! and the commands that show or remove a worktree.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const ownership = ownershipOf(reserved); stderr.write(notThisCommandsDestinationNotice({ destination, ownership }));
//! ```

use crate::json_record::quote;
use crate::worktree_copy_error::WorktreeCopyError;
use crate::worktree_copy_selection::shown_repository_path;
use crate::worktree_subvolume::Reservation;
use std::path::Path;

/// What:
///  Whose destination a creating command is looking at,
/// when it is about to copy.
/// Why:
///   The copy writes into the registration a worktree's own ignored state,
/// and only the command that made the destination may do it:
/// another
///  command's registration stands at the same path whenever two adds name one destination,
/// and Git's own
///  status does not say whose it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DestinationOwnership = 'mine' | 'anothers' | 'made again' | 'git failed';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DestinationOwnership {
    /// The destination is the empty directory this command made before real Git,
    /// still the object it made.
    Mine,
    /// The destination this command made is gone and something else stands at its path.
    MadeAgain,
    /// This command's own Git did not succeed,
    /// so the registration it found is not the one its Git made.
    GitFailed,
}

/// What:
///  Whose destination this command is looking at.
/// Why:
///   The reservation is read first because it is the only fact that was decided before any Git ran;
/// a destination that
///  was never reserved (no copy was planned) is not this command's either,
/// and the caller has already decided not to
///  copy for the commands that come this far without one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function ownershipOf(reserved: Reservation, destination: string): DestinationOwnership;
/// ```
pub(crate) fn ownership_of(reserved: &Reservation, git_succeeded: bool, destination: &Path) -> DestinationOwnership {
    // Only a Git that returned success registered the worktree this command asked for: a refusal means another command's
    // registration stands at the path, and a failure after registering means the checkout may be half done, so neither is
    // one this command fills.
    if !git_succeeded {
        return DestinationOwnership::GitFailed;
    }
    return match reserved {
        // A destination this command reserved is still its own while the object at the path is the one it made; a
        // destination it did not reserve is this command's only because its own Git filled it, which the success above
        // establishes.
        Reservation::Mine(made) => match made.identity {
            Some(identity) if crate::worktree_add_binding::path_identity(made.path.as_path()) == Some(identity) => {
                DestinationOwnership::Mine
            }
            _ => DestinationOwnership::MadeAgain,
        },
        // A destination this command did not make is its own only because its own Git filled it, which the success above
        // establishes: the object that stood at the path before the reservation must still be the one the binding names.
        // A different object is another command's replacement, which this command's ignored state is not written into.
        Reservation::Anothers(identity) => match identity {
            Some(seen) if crate::worktree_add_binding::path_identity(destination) != Some(*seen) => {
                DestinationOwnership::MadeAgain
            }
            _ => DestinationOwnership::Mine,
        },
        Reservation::NotAttempted => DestinationOwnership::Mine,
    };
}


/// What:
///  The line for a destination this command will not copy into,
/// where `record_stays` says whether this command keeps a record of it.
/// Why:
///   A worktree that receives no ignored-state copy must say so,
/// and the person needs the two facts that decide what to do:
/// which
///  command made the destination (another `git worktree add`,
/// or their own directory),
/// and the commands that
///  show or remove a worktree.
/// A refused add whose destination carries another command's proof settles its own record,
/// so
///  promising that a later command reports one would send the person looking for a record nothing keeps.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: ${quote(destination)} received no ignored-state copy: ...`
/// ```
pub(crate) fn not_this_commands_destination_notice(destination: &Path, ownership: DestinationOwnership, record_stays: bool) -> String {
    let shown: String = quote(destination.to_string_lossy().as_ref());
    let why: &str = match ownership {
        DestinationOwnership::MadeAgain => "the worktree this command registered is no longer at that path, so the ignored state was not copied into it and the record of this add stays; `git worktree list` shows where it is now",
        DestinationOwnership::GitFailed if record_stays => "this command's own `git worktree add` did not succeed, so the ignored state was not copied into the worktree it may have registered; the record of this add stays and a later command reports it",
        DestinationOwnership::GitFailed => "this command's own `git worktree add` did not succeed, and the worktree at that path was made by another command, so nothing was copied into it and this command keeps no record of it",
        DestinationOwnership::Mine => "it is not this command's to fill",
    };
    return format!(
        "monochromatic-git-policy-cli: {shown} received no ignored-state copy: {why}. A directory that was there before this command may be removed and the same line run again to have the copy made. `git worktree list` shows the worktrees of this repository.\n"
    );
}

/// What:
///  Whether the destination is absent,
/// or an empty directory holding nothing Git made,
/// in the cases where this command can
///  tell.
/// Why:
///   A `git worktree add` that Git refused for a reason of its own leaves the empty directory this command reserved;
/// there is
///  then no worktree to explain a missing copy for and nothing to report.
/// Only an absent directory is
///  evidence that nothing is there:
/// any other refusal,
/// a destination this command may not read included,
/// means it cannot say what
///  the directory holds,
/// so the answer is that there may be a worktree,
/// and its unfinished record stays.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function nothingButAReservation(destination: string): boolean; // absent, or an empty directory it may read
/// ```
pub(crate) fn nothing_but_a_reservation(destination: &Path) -> bool {
    // `match` on the listing: only absence and an empty directory mean Git made nothing here.
    return match std::fs::read_dir(destination) {
        Ok(mut entries) => entries.next().is_none(),
        Err(error) => error.kind() == std::io::ErrorKind::NotFound,
    };
}

/// What:
///  The refusal of a copy whose destination is no longer the registration it was made for.
/// Why:
///   Nothing in a worktree is overwritten or removed,
/// so the copy stops before the first entry,
/// and the person gets both registrations and the commands that show one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// new WorktreeCopyError(`monochromatic-git-policy-cli: the ignored state for ${destination} was not copied: ${reason}. ...`)
/// ```
pub(crate) fn registration_changed(destination: &Path, reason: &str) -> WorktreeCopyError {
    let shown: String = quote(destination.to_string_lossy().as_ref());
    let message: String = format!(
        "monochromatic-git-policy-cli: the ignored state for {shown} was not copied: {reason}. Nothing in that worktree was changed; `git worktree list` shows what is registered there now, and a `git worktree add` into the same path copies the ignored state afresh."
    );
    return crate::worktree_copy_error::failure_with_cause(message.as_str(), None);
}

/// How many unreadable paths the notice names before it says how many more there are.
///  Exempt from a citation under `LQM` (class:
/// display cap):
/// it limits how much text a message,
/// log field or record
///  names,
/// and no command result or duration depends on it.
/// Classed once in `doc/planning/cli-git-shipped-limits-
///  audit.md`,
/// "Exempt limits";
/// the coordinator's pick under the owner's delegation,
/// open to the owner's veto.
const NOTICE_PATHS: usize = 3;

/// What:
///  The one notice line for ignored entries the user could not read and the copy therefore left out.
/// Why:
///   The new worktree lacks them,
/// so the person sees how many and which first ones,
/// without a flood for a large set.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: left out ${count} unreadable ignored entries from ${source}: ${first.join(', ')}${more}.\n`
/// ```
pub(crate) fn unreadable_notice(source_root: &Path, unreadable: &[Vec<u8>]) -> String {
    let mut named: Vec<String> = Vec::new();
    for path in unreadable.iter().take(NOTICE_PATHS) {
        named.push(quote(shown_repository_path(path.as_slice()).as_str()));
    }
    let rest: usize = unreadable.len().saturating_sub(NOTICE_PATHS);
    let more: String = if rest == 0 { String::new() } else { format!(" and {rest} more") };
    let noun: &str = if unreadable.len() == 1 { "entry" } else { "entries" };
    return format!(
        "monochromatic-git-policy-cli: left out {} unreadable ignored {noun} from {}: {}{more}.\n",
        unreadable.len(),
        quote(source_root.to_string_lossy().as_ref()),
        named.join(", ")
    );
}

/// What:
///  The one notice line for ignored entries that kept changing under the copy and were left where they are.
/// Why:
///   The new worktree does not hold them,
/// and their only copy is the source's own file,
/// which is
///  untouched;
/// the person sees how many and which first ones,
/// and knows why they are missing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: left out ${count} ignored ${noun} that kept changing while they were copied, from ${source}: ${first.join(', ')}${more}. They stay in the source worktree, unchanged.\n`
/// ```
pub(crate) fn unsettled_notice(source_root: &Path, unsettled: &[Vec<u8>]) -> String {
    let mut named: Vec<String> = Vec::new();
    for path in unsettled.iter().take(NOTICE_PATHS) {
        named.push(quote(shown_repository_path(path.as_slice()).as_str()));
    }
    let rest: usize = unsettled.len().saturating_sub(NOTICE_PATHS);
    let more: String = if rest == 0 { String::new() } else { format!(" and {rest} more") };
    let noun: &str = if unsettled.len() == 1 { "entry" } else { "entries" };
    return format!(
        "monochromatic-git-policy-cli: left {} ignored {noun} out of the new worktree because they kept changing while they were copied, from {}: {}{more}; they stay in that worktree, unchanged.\n",
        unsettled.len(),
        quote(source_root.to_string_lossy().as_ref()),
        named.join(", ")
    );
}

/// The line a successful `git worktree add` prints when its command line has a form whose destination is not read.
pub const UNKNOWN_DESTINATION_NOTICE: &str = "monochromatic-git-policy-cli: this `git worktree add` names its worktree in a form monochromatic-git-policy-cli does not read, so the worktree received no ignored-state copy. Name the path as a plain operand (`git worktree add [-b <branch>] <path> [<commit-ish>]`, with `-C <directory>` or `-c <name>=<value>` before `worktree`) to have it copied.\n";

/// What:
///  The line for a destination Git made whose registration could not be confirmed.
/// Why:
///   A worktree that gets no copy must say so;
/// here the pointer in its `.git` file and the registration it names do not
///  name each other,
/// so monochromatic-git-policy-cli cannot tell the worktree is the command's own and copies nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: ${quote(destination)} was made by git worktree add, but ...`
/// ```
pub(crate) fn unconfirmed_destination_notice(destination: &Path) -> String {
    return format!(
        "monochromatic-git-policy-cli: {} was made, but its registration could not be confirmed as this command's, so it received no ignored-state copy. `git worktree list` shows whether Git registered it.\n",
        quote(destination.to_string_lossy().as_ref())
    );
}



/// What:
///  The line for a destination another command registered while this one's real Git ran.
/// Why:
///   Git refuses this command's add because the destination exists,
/// and the worktree there is the other command's:
///  this command copies nothing into it,
///  and the person is told which process made it and how to see the worktrees of the repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: ${quote(destination)} was registered by another git worktree add (process ${pid}) ...`
/// ```
pub(crate) fn registered_by_another_notice(destination: &Path, other: &str) -> String {
    let shown: String = quote(destination.to_string_lossy().as_ref());
    let owner: String = match crate::worktree_copy_owner::CopyOwner::from_token(other) {
        Some(found) => format!("process {}", found.pid),
        None => String::from("another command"),
    };
    return format!(
        "monochromatic-git-policy-cli: {shown} was registered by another `git worktree add` ({owner}) while this one ran, so this command copied nothing into it. `git worktree list` shows the worktrees of this repository.\n"
    );
}
