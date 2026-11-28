//! What:
//! The shapes one recovery of a killed install works in:
//! what a hidden sibling holds,
//! the hidden sibling itself,
//! the rule that decides whether a version may be kept,
//! what became of the finished copy,
//! what the recovery answered,
//! and what a recovery that failed answers.
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
//! type RemovedLeftover = { path: string; finishedCopy: RemovedFinishedCopy };
//! type RemovedFinishedCopy = 'no-finished-copy' | { secondCopy: string; programFound: boolean } | { notAtItsName: string; whyNotPublished: WhyNotPublished };
//! type WhyNotPublished = 'the-lock-does-not-record-its-archive' | { anOlderRepositoryToolIsNeverKept: string };
//! type Recovered = { finishedCopy: FinishedCopy; restored: boolean; leftForALiveInstall?: HoldingInstaller; removed: RemovedLeftover[] };
//! type ARecoveryThatFailed = { error: ToolchainError; thePublishKeptItsCopy: boolean };
//! function holdsOf(purpose: string): Holds | undefined;
//! function mayBePublished(holds: Holds): boolean;
//! ```

/// Imports what this module uses from `std::fmt`.
use std::fmt;
/// Imports what this module uses from `std::path`.
use std::path::PathBuf;

/// Imports the error type the recovery answers with.
use super::super::super::error::ToolchainError;

/// Imports the owner of a hidden sibling and its liveness question.
use super::super::the_owner_of_a_hidden_sibling::{
    Liveness,
    Owner,
};
/// Imports the three purposes a hidden sibling's name may carry,
/// and the sentence a failure that named a finished copy is followed by.
use super::super::{
    RECOVERED_PURPOSE,
    REPLACED_PURPOSE,
    STAGING_PURPOSE,
    WHAT_A_LATER_INSTALL_DOES_WITH_THE_COPY,
};

/// What:
/// Which copy a failed recovery's own message named,
/// or that it named none,
/// which is what the warning built around that message may say a later install does.
/// Why:
/// The warning follows the recovery's words with what this run then did,
/// and only a failure that named a finished copy can go on to say what a later install does with that copy:
/// the listing named nothing,
/// and the put-back named the copy that was there,
/// which is not a finished copy and whose own fate is where it waits.
/// Each value is held by a control:
/// the publish's own failure by the five that read the end of its warning,
/// the claim's by one that reads which copy its failure named
/// (`a_failed_claim_says_what_a_later_install_does_with_the_finished_copy_it_named`),
/// and the put-back's by one that reads what is said about the copy it named
/// (`a_failed_put_back_of_the_copy_that_was_there_claims_nothing_about_a_finished_copy`).
/// Neither of the last two is held past a failure:
/// that claim's and that put-back's renames fail only in a tool directory that refuses writes,
/// and an install into such a directory cannot create its own staging directory either,
/// so no state a test can build carries either failure into a warning a run prints.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WhatTheFailureNamed = 'nothing' | 'a-finished-copy' | 'the-copy-that-was-moved-aside';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WhatTheFailureNamed {
    /// The failure named no copy at all,
    /// which is the tool directory's listing that could not be read.
    Nothing,
    /// The failure named a finished copy:
    /// the one the publish kept,
    /// or the one a dead install left and the claim could not take.
    AFinishedCopy,
    /// The failure named the copy that was moved aside,
    /// which the put-back could not return to the version's name.
    TheCopyThatWasMovedAside,
}

/// What:
/// A recovery that failed:
/// the error a person reads,
/// and which copy that error named.
/// Why:
/// The warning an install makes of a failed recovery states what a later install does with the copy that failure named,
/// and the sentence depends on which copy that was:
/// a finished copy is published or removed,
/// while the copy that was moved aside waits to be put back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ARecoveryThatFailed = { error: ToolchainError; named: WhatTheFailureNamed };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ARecoveryThatFailed {
    /// The error the run prints.
    pub(crate) error: ToolchainError,
    /// Which copy that error named,
    /// which decides what the warning around it may say.
    pub(crate) named: WhatTheFailureNamed,
}

/// The methods of `ARecoveryThatFailed`.
impl ARecoveryThatFailed {
    /// What:
    /// A recovery that failed before it read the tool directory's hidden names,
    /// so its message names no copy.
    /// Why:
    /// The listing is the first step of a recovery,
    /// and a run whose listing failed read nothing about what the directory holds.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function nothingRead(error: ToolchainError): ARecoveryThatFailed { return { error, thePublishKeptItsCopy: false }; }
    /// ```
    pub(crate) fn nothing_read(error: ToolchainError) -> ARecoveryThatFailed {
        return ARecoveryThatFailed {
            error,
            named: WhatTheFailureNamed::Nothing,
        };
    }

    /// What:
    /// A recovery whose claim failed,
    /// whose message names the finished copy a dead install left at its hidden name.
    /// Why:
    /// That copy is still there and still finished,
    /// so a later install reads it as a second finished copy once the version is at its name,
    /// and this run's warning says so.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function aFinishedCopyItNamed(error: ToolchainError): ARecoveryThatFailed {
    ///   return { error, named: 'a-finished-copy' };
    /// }
    /// ```
    pub(crate) fn a_finished_copy_it_named(error: ToolchainError) -> ARecoveryThatFailed {
        return ARecoveryThatFailed {
            error,
            named: WhatTheFailureNamed::AFinishedCopy,
        };
    }

    /// What:
    /// A recovery whose put-back of the copy that was moved aside failed,
    /// whose message names that copy and says nothing was put back at the version's name.
    /// Why:
    /// That copy is the one that was there,
    /// not a finished copy,
    /// so the sentence about a finished copy would be false of it,
    /// and the warning says what this run then did and nothing about that copy.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function theCopyThatWasMovedAsideItNamed(error: ToolchainError): ARecoveryThatFailed {
    ///   return { error, named: 'the-copy-that-was-moved-aside' };
    /// }
    /// ```
    pub(crate) fn the_copy_that_was_moved_aside_it_named(
        error: ToolchainError
    ) -> ARecoveryThatFailed {
        return ARecoveryThatFailed {
            error,
            named: WhatTheFailureNamed::TheCopyThatWasMovedAside,
        };
    }

    /// What:
    /// The publish's own failure,
    /// whose message names the finished copy this run keeps.
    /// Why:
    /// This is the failure arm an install reaches on its own,
    /// and its warning states what a later install does with that copy.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function thePublishKeptItsCopy(error: ToolchainError): ARecoveryThatFailed {
    ///   return { error, named: 'a-finished-copy' };
    /// }
    /// ```
    pub(crate) fn the_publish_kept_its_copy(error: ToolchainError) -> ARecoveryThatFailed {
        return ARecoveryThatFailed {
            error,
            named: WhatTheFailureNamed::AFinishedCopy,
        };
    }

    /// What:
    /// What the warning says after "the install this run went on to make put <tool> <version> at its name".
    /// Why:
    /// The run did not fail for the recovery's failure,
    /// and which copy that failure named decides whether the warning can go on to say what a later install does with it.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function whatALaterInstallDoes(self: ARecoveryThatFailed): string {
    ///   const nothingFailed = 'so the run does not fail for it';
    ///   if (self.named !== 'a-finished-copy') return nothingFailed;
    ///   return `${nothingFailed}. ${WHAT_A_LATER_INSTALL_DOES_WITH_THE_COPY}`;
    /// }
    /// ```
    pub(crate) fn what_a_later_install_does(&self) -> String {
        let nothing_failed: &str = "so the run does not fail for it";
        return match self.named {
            // What: the two arms whose message names no finished copy.
            // Why: a later install's act on a copy the warning cannot name is nothing this run may state.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // if (self.named !== 'a-finished-copy') return nothingFailed;
            // ```
            WhatTheFailureNamed::Nothing | WhatTheFailureNamed::TheCopyThatWasMovedAside => {
                nothing_failed.to_string()
            }
            // What: the finished copy this run keeps, or the one a dead install left.
            // Why: the install that followed put the version at its name, so a later install removes that copy as a second one.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // return `${nothingFailed}. ${WHAT_A_LATER_INSTALL_DOES_WITH_THE_COPY}`;
            // ```
            WhatTheFailureNamed::AFinishedCopy => {
                format!("{nothing_failed}. {WHAT_A_LATER_INSTALL_DOES_WITH_THE_COPY}")
            }
        };
    }
}

/// Implements `fmt::Display` for `ARecoveryThatFailed`.
impl fmt::Display for ARecoveryThatFailed {
    /// What:
    /// Prints the error alone,
    /// which is the sentence a warning carries.
    /// Why:
    /// A warning joins this to what the run then did,
    /// and which step failed is not part of what a person reads.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// toString(): string { return this.error.message; }
    /// ```
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        return write!(formatter, "{}", self.error.message);
    }
}

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
/// Why this run did not publish a finished copy that it removed:
/// the two facts of the publish rule that keep such a copy out of the version's name.
/// Why:
/// The report has to say why the copy went,
/// and both reasons are facts this run read:
/// one from the copy's completion marker and the lock's row,
/// the other from the finished version that is installed beside it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WhyNotPublished = 'the-lock-does-not-record-its-archive' | { anOlderRepositoryToolIsNeverKept: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum WhyNotPublished {
    /// The copy's completion marker does not record the archive this lock names for the version,
    /// so nothing here could vouch for its bytes.
    TheLockDoesNotRecordItsArchive,
    /// A newer finished version of this repository-published tool is installed,
    /// and an older repository tool is never kept.
    AnOlderRepositoryToolIsNeverKept {
        /// The newer finished version that is installed.
        newer: String,
    },
}

/// What:
/// What this run read about the finished copy that went with a hidden sibling it removed:
/// nothing at all,
/// a copy whose version's own name held a completion marker (and the program this run looks for),
/// or a copy whose version holds no finished copy at its name and that this run did not publish.
/// Why:
/// A removed finished copy may be a second one the version can spare or the one copy it had here,
/// and the run answers that by reading the version's own name,
/// never by calling the copy's version lost;
/// the words say which of those reads it made,
/// because a marker and a program found are not a working tool.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RemovedFinishedCopy = 'no-finished-copy' | { secondCopy: string; programFound: boolean } | { notAtItsName: string; whyNotPublished: WhyNotPublished };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum RemovedFinishedCopy {
    /// The hidden sibling held no finished copy,
    /// so nothing went with it.
    NoFinishedCopy,
    /// It held a finished copy of `version`,
    /// and a finished copy of that version is at its name.
    SecondCopy {
        /// The version the removed copy belonged to.
        version: String,
        /// Whether the run found the program it looks for at the version's name,
        /// rather than reading the completion marker alone,
        /// which is what it does for a tool that declares no bin.
        program_found: bool,
    },
    /// It held a finished copy of `version`,
    /// and no finished copy of that version is at its name.
    NotAtItsName {
        /// The version the removed copy belonged to.
        version: String,
        /// Why this run did not publish the copy.
        why_not_published: WhyNotPublished,
    },
}

/// What:
/// One hidden sibling this run removed,
/// with what it read about the finished copy that went with it.
/// Why:
/// The report says whether a finished copy went with a removed directory and what that cost,
/// so a person knows whether the version still has a copy or comes back only from a download.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RemovedLeftover = { path: string; finishedCopy: RemovedFinishedCopy };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RemovedLeftover {
    /// The hidden sibling that is gone when the recovery returns.
    pub(crate) path: PathBuf,
    /// What this run read about the finished copy the directory held when it went.
    pub(crate) finished_copy: RemovedFinishedCopy,
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
