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
pub(crate) fn ownership_of(reserved: &Reservation, git_succeeded: bool) -> DestinationOwnership {
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
        // A destination this command did not reserve is its own only because its own Git filled it, which the success
        // above establishes; a leftover empty directory from a killed command is filled the same way.
        Reservation::Anothers | Reservation::NotAttempted => DestinationOwnership::Mine,
    };
}

/// What:
///  The line for a destination this command will not copy into.
/// Why:
///   A worktree that receives no ignored-state copy must say so,
/// and the person needs the two facts that decide what to do:
/// which
///  command made the destination (another `git worktree add`,
/// or their own directory),
/// and the commands that
///  show or remove a worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: ${quote(destination)} received no ignored-state copy: ...`
/// ```
pub(crate) fn not_this_commands_destination_notice(destination: &Path, ownership: DestinationOwnership) -> String {
    let shown: String = quote(destination.to_string_lossy().as_ref());
    let why: &str = match ownership {
        DestinationOwnership::MadeAgain => "it is no longer the directory this command made, so the worktree that stands there is another command's",
        DestinationOwnership::GitFailed => "this command's own `git worktree add` did not succeed, so the ignored state was not copied into the worktree it may have registered; the record of this add stays and a later command reports it",
        DestinationOwnership::Mine => "it is not this command's to fill",
    };
    return format!(
        "monochromatic-git-policy-cli: {shown} received no ignored-state copy: {why}. A directory that was there before this command may be removed and the same line run again to have the copy made. `git worktree list` shows the worktrees of this repository.\n"
    );
}

/// What:
///  Whether the destination is absent,
/// or an empty directory holding nothing Git made.
/// Why:
///   A `git worktree add` that Git refused for a reason of its own leaves the empty directory this command reserved;
/// there is
///  then no worktree to explain a missing copy for and nothing to report.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function nothingButAReservation(destination: string): boolean; // absent, or an empty directory
/// ```
pub(crate) fn nothing_but_a_reservation(destination: &Path) -> bool {
    // `match` on the listing: an absent path and an empty directory both mean Git made nothing here.
    return match std::fs::read_dir(destination) {
        Ok(mut entries) => entries.next().is_none(),
        Err(_absent) => true,
    };
}
