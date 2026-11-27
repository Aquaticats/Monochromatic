//! What:
//! The shapes one recovery of a killed install works in:
//! what a hidden sibling holds,
//! the hidden sibling itself,
//! the rule that decides whether a version may be kept,
//! what became of the finished copy,
//! and what the recovery answered.
//! Why:
//! The repair and the report both name these answers,
//! and they are the whole vocabulary a reader of a recovery's sentence needs;
//! keeping them beside the algorithm that computes them would push that file past the length a person reads at once.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // what-a-recovery-answers.ts
//! type Holds = 'unpacking' | 'replaced-copy' | 'recovered';
//! type Leftover = { path: string; version: string; holds: Holds; owner?: Owner };
//! type AnOlderVersion = 'may-stay' | { newer: string };
//! type FinishedCopy = 'nothing-to-publish' | { publishedHere: string } | 'published-by-another-install' | { newer: string };
//! type HoldingInstaller = { path: string; running: boolean; finishedCopy: boolean };
//! type RemovedLeftover = { path: string; heldAFinishedCopy: boolean };
//! type Recovered = { finishedCopy: FinishedCopy; restored: boolean; leftForALiveInstall?: HoldingInstaller; removed: RemovedLeftover[] };
//! function holdsOf(purpose: string): Holds | undefined;
//! function mayBePublished(holds: Holds): boolean;
//! ```

/// Imports what this module uses from `std::path`.
use std::path::PathBuf;

/// Imports the owner of a hidden sibling and its liveness question.
use super::super::the_owner_of_a_hidden_sibling::{
    Liveness,
    Owner,
};
/// Imports the three purposes a hidden sibling's name may carry.
use super::super::{
    RECOVERED_PURPOSE,
    REPLACED_PURPOSE,
    STAGING_PURPOSE,
};

/// What one hidden sibling holds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Holds {
    /// An install being unpacked into it,
    /// which may or may not have reached its completion marker.
    Unpacking,
    /// The copy that was at the version's name before a replace step moved it aside.
    ReplacedCopy,
    /// A finished copy this or another install claimed in order to publish it.
    Recovered,
}

/// One hidden sibling of a version directory.
#[derive(Clone, Debug)]
pub(crate) struct Leftover {
    /// The sibling itself.
    pub(crate) path: PathBuf,
    /// The version directory it belongs to.
    pub(crate) version: String,
    /// What it holds.
    pub(crate) holds: Holds,
    /// The installer that made it,
    /// or nothing when its name carries no start time,
    /// which an earlier release wrote.
    pub(crate) owner: Option<Owner>,
}

/// The methods of `Leftover`.
impl Leftover {
    /// What:
    /// What this reader can say about the installer that made the sibling.
    /// Why:
    /// A name written by an earlier release carries no owner to ask about,
    /// and a name whose owner cannot be read is never removed.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function liveness(leftover: Leftover): Liveness;
    /// ```
    pub(crate) fn liveness(&self) -> Liveness {
        return match &self.owner {
            Some(owner) => owner.liveness(),
            None => Liveness::Unknown,
        };
    }
}

/// What:
/// Whether a copy of the version this recovery is about may be kept at all.
/// Why:
/// The owner's rule is that an older repository-published tool is never kept once a newer one is installed,
/// and a recovery runs before the check that a newer version satisfies the lock,
/// so the recovery is where an older locked version would otherwise appear beside the newer one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AnOlderVersion = 'may-stay' | { neverKept: string };
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) enum AnOlderVersion {
    /// No newer finished version is installed,
    /// so this version may be published or put back.
    #[default]
    MayStay,
    /// A newer finished version is installed,
    /// so neither a finished copy nor the copy that was there may reach this
    /// version's name.
    NeverKept {
        /// The newer finished version that is installed.
        newer: String,
    },
}

/// What:
/// What became of the finished copy a killed install left.
/// Why:
/// The install report says what happened to it:
/// this run published it,
/// another install published the version meanwhile and this run dropped its own copy,
/// or the rule above kept it out of the version's name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FinishedCopy = 'nothing-to-publish' | { publishedHere: string } | 'published-by-another-install' | { olderVersionIsNeverKept: string };
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) enum FinishedCopy {
    /// There was no finished copy this run could publish.
    #[default]
    NothingToPublish,
    /// This run published it at the version's name,
    /// from the hidden name the copy stood at when this run read it.
    PublishedHere {
        /// The hidden name the finished copy was found at,
        /// which the report names.
        copy: PathBuf,
    },
    /// Another install published the version in the instant since this run looked,
    /// so this run dropped its copy.
    PublishedByAnotherInstall,
    /// A newer finished version of this repository-published tool is installed,
    /// so the finished copy stays hidden.
    AnOlderVersionIsNeverKept {
        /// The newer finished version that is installed.
        newer: String,
    },
}

/// What:
/// The hidden name that holds the copy a killed install moved aside,
/// because nothing this run read shows that its own installer ended.
/// Why:
/// A person reading the report needs the path whose installer waits,
/// whether the system reports that installer running,
/// and whether the name holds a finished copy,
/// because that copy is all the version has when the installer died where this reader cannot see it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type HoldingInstaller = { path: string; running: boolean; finishedCopy: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct HoldingInstaller {
    /// The hidden name whose installer may still run.
    pub(crate) path: PathBuf,
    /// Whether the system reports that installer running,
    /// rather than this reader failing to judge it.
    pub(crate) running: bool,
    /// Whether the hidden name holds a finished copy of the version.
    pub(crate) finished_copy: bool,
}

/// What:
/// One hidden sibling this run removed,
/// with whether it held a finished copy when it went.
/// Why:
/// The report says that a finished copy went with a removed directory,
/// so a person knows the version comes back only from a download or another copy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RemovedLeftover = { path: string; heldAFinishedCopy: boolean };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RemovedLeftover {
    /// The hidden sibling that is gone when the recovery returns.
    pub(crate) path: PathBuf,
    /// Whether that directory held a finished copy of its version.
    pub(crate) held_a_finished_copy: bool,
}

/// What:
/// What one recovery of a tool's version directory found and did.
/// Why:
/// The install report says what became of the finished copy,
/// whether the copy that was there came back,
/// which hidden name holds the wait,
/// and which hidden siblings went with whether a finished copy went with them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Recovered = { finishedCopy: FinishedCopy; restored: boolean; leftForALiveInstall?: HoldingInstaller; removed: RemovedLeftover[] };
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct Recovered {
    /// What became of the finished copy a killed install left.
    pub(crate) finished_copy: FinishedCopy,
    /// Whether the copy a killed install moved aside was put back at the version's name.
    pub(crate) restored: bool,
    /// The hidden name that holds the copy a killed install moved aside in place,
    /// because nothing this run read shows that its installer ended.
    pub(crate) left_for_a_live_install: Option<HoldingInstaller>,
    /// The hidden siblings this run removed,
    /// which are gone when it returns.
    pub(crate) removed: Vec<RemovedLeftover>,
}

/// The methods of `Recovered`.
impl Recovered {
    /// What:
    /// Whether this run left a finished copy at the version's name:
    /// it published one,
    /// another install had published it,
    /// or the copy that was there came back.
    /// Why:
    /// The report counts a tool as installed when its version's name holds a finished copy after the run,
    /// whichever of those put it there.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function installedHereOrMeanwhile(recovered: Recovered): boolean {
    ///   return recovered.finishedCopy === 'published-here' || recovered.finishedCopy === 'published-by-another-install' || recovered.restored;
    /// }
    /// ```
    pub(crate) fn installed_here_or_meanwhile(&self) -> bool {
        // What: `matches!` asks whether the enum value is one of the variants listed, without binding what it holds.
        // Why: the two published variants both leave the version's name finished, and `restored` leaves the copy that
        //  was there.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return ['published-here', 'published-by-another-install'].includes(recovered.finishedCopy) || recovered.restored;
        // ```
        return matches!(
            self.finished_copy,
            FinishedCopy::PublishedHere {
                copy: _,
            } | FinishedCopy::PublishedByAnotherInstall
        ) || self.restored;
    }

    /// What:
    /// Whether this run gave the version's name a copy,
    /// by publishing one or by putting the copy that was there back.
    /// Why:
    /// The rule that no older repository tool is kept settles after exactly those two cases,
    /// and a copy that another install published there is that install's to settle.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function putACopyAtTheVersionName(recovered: Recovered): boolean {
    ///   return recovered.finishedCopy === 'published-here' || recovered.restored;
    /// }
    /// ```
    pub(crate) fn put_a_copy_at_the_version_name(&self) -> bool {
        return matches!(
            self.finished_copy,
            FinishedCopy::PublishedHere {
                copy: _,
            }
        ) || self.restored;
    }
}

/// What a purpose in a hidden sibling's name says about what it holds,
/// or nothing when it names none of the three.
pub(crate) fn holds_of(purpose: &str) -> Option<Holds> {
    if purpose == STAGING_PURPOSE {
        return Some(Holds::Unpacking);
    }
    if purpose == REPLACED_PURPOSE {
        return Some(Holds::ReplacedCopy);
    }
    if purpose == RECOVERED_PURPOSE {
        return Some(Holds::Recovered);
    }
    return None;
}

/// Whether a hidden copy of this kind may hold a finished install that a later install publishes.
pub(crate) fn may_be_published(holds: Holds) -> bool {
    return matches!(holds, Holds::Unpacking | Holds::Recovered);
}
