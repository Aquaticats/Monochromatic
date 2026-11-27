//! What:
//! What the stop of a scope found,
//! in the words a refusal carries:
//! how long the stop waited,
//! whether the kernel still reported the scope populated,
//! and every member still listed,
//! in the scope's own group and in every group below it,
//! each with the group it stands in and the state of its threads.
//! Why:
//! Issue 6528:
//! the install step refused a command because a process cargo started "could not be stopped",
//! and nothing said whether that process had exited (one that waits to be reaped holds no descriptor and can write
//! nothing) or was alive (a writer the lockfiles cannot be trusted under).
//! A person reading the refusal must be able to tell those apart,
//! and a loaded host from a member that cannot be killed,
//! so the refusal says how long the stop waited and what it was waiting for.
//! The kernel's `populated` word counts the groups below the scope too,
//! so a scope whose own `cgroup.procs` is empty can still hold a live process
//! (`doc/troubleshooting/a-cgroup-does-not-count-a-member-that-has-exited.md`),
//! and the listing reads every group of the scope for that reason.
//! Split out of `control_group_stop.rs`,
//! which keeps the stop itself,
//! and of `control_group_stop_report_members.rs`,
//! which keeps the state of one member.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // control-group-stop-report.ts
//! export class StopReport { waited: number; empty: boolean; sentence(scope: string): string; headline(): string }
//! ```

/// What:
/// `mod members;` declares how one member stood,
/// the group below the scope it stands in,
/// and the read of its threads through a pidfd.
/// Why:
/// A file that holds both the reading and the words would pass the code-line limit,
/// and a control can drive the words with members it builds itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // an import from the sibling that reads a member
/// ```
#[path = "control_group_stop_report_members.rs"]
pub(crate) mod members;
/// What:
/// `use self::members::MemberStanding;` re-exports one member's state for the controls alone.
/// Why:
/// The report asks a member what it can write through the member itself,
/// so only a control names the state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export type { MemberStanding } from './control_group_stop_report_members.ts';
/// ```
#[cfg(test)]
pub(crate) use self::members::MemberStanding;
/// What:
/// `use self::members::{MemberListing, standing_of_within};` brings the member with its standing and the read of its
/// threads into this module,
/// and re-exports both so `report::MemberListing` and `report::standing_of_within` keep their paths.
/// Why:
/// The report builds its members from that read,
/// and its controls name both by those paths.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { MemberListing, standingOfWithin } from './control_group_stop_report_members.ts';
/// ```
pub(crate) use self::members::{
    MemberListing,
    standing_of_within,
};
/// What:
/// `use super::LISTING_BOUND;` imports how long reading the members may take.
/// Why:
/// A listing that hit that bound says so rather than reading as a complete one,
/// and the words a refusal carries name the bound it hit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { LISTING_BOUND } from './control_group_stop.ts';
/// ```
use super::LISTING_BOUND;
/// What:
/// `use std::path::Path;` imports the borrowed view of a path (its sibling `PathBuf` owns one).
/// Why:
/// The sentence names the scope's directory without keeping a copy of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // a path is a plain string in TS
/// ```
use std::path::Path;
/// What:
/// `use std::time::Duration;` imports a span of time (its sibling `Instant` is a point on the clock).
/// Why:
/// The time the stop waited is what the refusal must state,
/// and a point on the clock belongs to the read of a member in `members.rs`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // a number of milliseconds
/// ```
use std::time::Duration;

/// What the reading of the members found,
/// and the one order every listing names the members in:
/// the scope's own group first,
/// then each group below it depth first with the greatest name first,
/// and the processes of one group in ascending process id order.
pub(crate) enum Listing {
    /// Every group of the scope,
    /// and every group below it,
    /// was read:
    /// these are the members that were left.
    Read(
        /// The members the read found,
        /// in the order `Listing` states.
        Vec<MemberListing>,
    ),
    /// The listing's own bound passed before every member was read:
    /// these are the members it read,
    /// and the rest were not named.
    Cut(
        /// The members the read reached before its bound passed,
        /// in the order `Listing` states.
        Vec<MemberListing>,
    ),
    /// The scope's own directory was gone when its members were read,
    /// so nothing was there to name.
    ScopeGone,
    /// The members could not be read,
    /// with the members read before the read that failed,
    /// and the reason it failed.
    Unreadable(
        /// The members the read reached before it failed,
        /// in the order `Listing` states.
        Vec<MemberListing>,
        /// Why the rest could not be read.
        String,
    ),
}

/// What a refusal may say about what the listing left,
/// which is the one thing the opening clause adds to the sentence.
pub(crate) enum Leftover {
    /// A member that was read can still write.
    Writer,
    /// Every group was read and no member it listed can still write.
    NoWriter,
    /// Every group was read and none of them listed a member.
    Nothing,
    /// The scope's directory was gone when its members were read.
    ScopeGone,
    /// The members could not be read.
    Unreadable,
    /// The listing's own bound passed before every member was read.
    Cut,
}

/// What the stop of one scope found.
pub(crate) struct StopReport {
    /// How long the stop waited for the scope to become empty.
    waited: Duration,
    /// Whether the stop showed the scope empty:
    /// the kernel reported it unpopulated,
    /// or its absence was verified.
    empty: bool,
    /// The kernel's last `populated` word,
    /// or `None` when the group could not be read (it was removed).
    populated: Option<bool>,
    /// Whether the scope's directory was gone when the stop looked and its absence was verified.
    /// Why a flag of its own:
    /// an empty report earned that way has no `populated` word behind it,
    /// and the words must not say the kernel spoke.
    absent: bool,
    /// The members the listing found,
    /// or why it found none it could read.
    listing: Listing,
}

/// What:
/// `impl StopReport` builds the report and turns it into the words a refusal carries.
/// Why:
/// The stop's answer and the text a person reads must be built from the same facts,
/// or a message could disagree with the run that produced it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // a plain class with a constructor and a few methods
/// ```
impl StopReport {
    /// What:
    /// `pub(crate) fn new(...) -> StopReport` takes what the stop saw of a scope that was not verified absent.
    /// Why:
    /// `control_group_stop.rs` owns the loop and hands it these facts once.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// constructor(waited: number, empty: boolean, populated: boolean | undefined, listing: Listing)
    /// ```
    pub(crate) fn new(
        waited: Duration,
        empty: bool,
        populated: Option<bool>,
        listing: Listing,
    ) -> StopReport {
        return StopReport {
            waited,
            empty,
            populated,
            absent: false,
            listing,
        };
    }

    /// What:
    /// `pub(crate) fn absent(waited: Duration) -> StopReport` is the report of a scope whose absence was verified.
    /// Why:
    /// The same report type carries both answers,
    /// so no caller has to read two shapes for one question,
    /// and the words say the directory was gone instead of pretending the kernel reported it empty.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// static absent(waited: number): StopReport
    /// ```
    pub(crate) fn absent(waited: Duration) -> StopReport {
        return StopReport {
            waited,
            empty: true,
            populated: None,
            absent: true,
            listing: Listing::ScopeGone,
        };
    }

    /// What:
    /// `pub(crate) fn empty(&self) -> bool` says whether the stop showed the scope empty.
    /// Why:
    /// A caller must ask this and nothing else before it treats the run as settled.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// get isEmpty(): boolean { return this.empty }
    /// ```
    pub(crate) fn empty(&self) -> bool {
        return self.empty;
    }

    /// What:
    /// `pub(crate) fn leftover(&self) -> Leftover` is what the report may say about what was left.
    /// Why:
    /// Every refusal's opening clause is chosen from this one answer,
    /// so a refusal cannot claim a member it never read is harmless
    /// and cannot stay silent about a member it did read,
    /// whether that read was the whole of the scope or stopped at a group below it or at the listing's bound.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// leftover(): 'writer' | 'no-writer' | 'nothing' | 'scope-gone' | 'unreadable' | 'cut'
    /// ```
    pub(crate) fn leftover(&self) -> Leftover {
        // What:     `let read: &[MemberListing] = ...` is every member the listing read, whatever stopped it afterwards.
        // Why:      One member that can still write decides the opening clause, so it is chosen before the kind of ending.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const read = this.listing.members;
        // ```
        let read: &[MemberListing] = match &self.listing {
            Listing::Read(found) => found,
            Listing::Cut(found) => found,
            Listing::Unreadable(found, _reason) => found,
            Listing::ScopeGone => return Leftover::ScopeGone,
        };
        if read
            .iter()
            .any(|member| return member.standing.can_still_write())
        {
            return Leftover::Writer;
        }
        return match &self.listing {
            Listing::Read(found) if found.is_empty() => Leftover::Nothing,
            Listing::Read(_found) => Leftover::NoWriter,
            Listing::Cut(_found) => Leftover::Cut,
            Listing::Unreadable(_found, _reason) => Leftover::Unreadable,
            Listing::ScopeGone => Leftover::ScopeGone,
        };
    }

    /// What:
    /// `pub(crate) fn headline(&self) -> String` is the opening clause of a refusal of a scope that did not read as empty.
    /// Why:
    /// The clause and the sentence after it are one message,
    /// and the clause must name only what the listing found (issue 6528's review,
    /// Medium 1).
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// headline(): string { ... }
    /// ```
    pub(crate) fn headline(&self) -> String {
        return match self.leftover() {
            Leftover::Writer => "a member of the scope could not be shown stopped".to_string(),
            Leftover::NoWriter => {
                "the scope did not read as empty, and no member it listed can still write".to_string()
            }
            Leftover::Nothing => {
                "the scope did not read as empty, and it listed no member of the scope or of any group below it"
                    .to_string()
            }
            Leftover::ScopeGone => {
                "the scope did not read as empty, and its directory was gone when its members were read"
                    .to_string()
            }
            Leftover::Unreadable => match &self.listing {
                Listing::Unreadable(read, _reason) if !read.is_empty() => {
                    "the scope did not read as empty, and its members could not all be read".to_string()
                }
                _other => "the scope did not read as empty, and its members could not be read".to_string(),
            },
            Leftover::Cut => format!(
                "the scope did not read as empty, and its members could not all be read within {LISTING_BOUND:?}"
            ),
        };
    }

    /// What:
    /// `pub(crate) fn sentence(&self, scope: &Path) -> String` is the report as one clause of a refusal.
    /// Why:
    /// The refusal of a run that cannot be settled must say how long it waited and what it was waiting for,
    /// in words that tell an exited member from a live one,
    /// and must not state a wait that was not the wait.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// sentence(scope: string): string { ... }
    /// ```
    pub(crate) fn sentence(
        &self,
        scope: &Path,
    ) -> String {
        if self.empty {
            if self.absent {
                return format!(
                    "the scope {} was gone when the stop looked, and its absence was verified",
                    scope.display()
                );
            }
            return format!(
                "the stop of {} waited {:?} and the kernel reported the scope empty",
                scope.display(),
                self.waited
            );
        }
        let population: &str = match self.populated {
            Some(true) => "the kernel last reported it populated",
            Some(false) => "the kernel last reported it empty",
            None => "its group could not be read",
        };
        return format!(
            "the stop of {} waited {:?} for it to be empty; {population}, and {}",
            scope.display(),
            self.waited,
            self.listing_text()
        );
    }

    /// What:
    /// `fn listing_text(&self) -> String` is what the listing found,
    /// in the words that follow the kernel's word.
    /// Why:
    /// Three different findings would otherwise read the same:
    /// no member was left to name,
    /// a listing could not be read,
    /// and a listing was cut at its bound.
    /// A scope the manager had already removed is the one case where nothing of the run is left to wait for,
    /// so that clause also says what became of the run's journal and what a run that cannot show what this one left prints next.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// listingText(): string { ... }
    /// ```
    fn listing_text(&self) -> String {
        return match &self.listing {
            Listing::Read(members) if members.is_empty() => {
                "it listed no member of the scope or of any group below it".to_string()
            }
            Listing::Read(members) => named_members(members),
            Listing::Cut(members) if members.is_empty() => format!(
                "the listing of its members was cut at its own bound of {LISTING_BOUND:?} before one was read"
            ),
            Listing::Cut(members) => format!(
                "the listing of its members was cut at its own bound of {LISTING_BOUND:?} after {}",
                named_members(members)
            ),
            Listing::ScopeGone => {
                "its directory was gone when its members were read, so none of its members was read (the manager removes the directory of a scope its last member has left, and this look did not verify that it still sees the parent as it did when the scope was planned, so the absence is not shown to be the manager's); the journal of this run was kept, and the next run of this workspace settles it by itself once it reads the recorded scope gone; a run that cannot show what this run left says so, names the journal, and prints the `cargo-shadow acknowledge <journal directory>` command to run once every process or worker of this run that can modify the workspace has stopped".to_string()
            }
            Listing::Unreadable(read, reason) if read.is_empty() => {
                format!("its members could not be read ({reason})")
            }
            // What: the members the read reached before it failed are named, so a live writer it already found is not lost.
            // Why:  "its members could not be read" with a member it did read would read as a scope that listed nothing.
            Listing::Unreadable(read, reason) => format!(
                "its members could not all be read ({reason}), and before that {}",
                named_members(read)
            ),
        };
    }
}

/// What:
/// `fn named_members(members: &[MemberListing]) -> String` is one clause naming every member the listing read.
/// Why:
/// The sentence and the cut case both need the list with the count that agrees with it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function namedMembers(members: MemberListing[]): string
/// ```
fn named_members(members: &[MemberListing]) -> String {
    let listed: Vec<String> = members
        .iter()
        .map(|member| {
            return member.text();
        })
        .collect();
    // What:     `if listed.len() == 1 { "member" } else { "members" }` picks the word that agrees with the count.
    // Why:      A refusal is read by a person, and "1 member(s)" reads like a placeholder nobody finished.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const noun = listed.length === 1 ? 'member' : 'members';
    // ```
    let noun: &str = if listed.len() == 1 {
        "member"
    } else {
        "members"
    };
    return format!("it listed {} {noun}: {}", listed.len(), listed.join(", "));
}
