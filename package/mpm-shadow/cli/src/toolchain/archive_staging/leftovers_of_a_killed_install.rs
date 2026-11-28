//! What:
//! The hidden siblings a killed install leaves beside a version directory,
//! which of them belong to an installer that no longer runs,
//! and what a later install does with them.
//! Why:
//! A process killed at any instruction is normal on this host,
//! and the publish step has an instant in which the version's name is absent
//! (issue 6600).
//! Nothing read those names before this file existed:
//! a finished copy stayed under a hidden name,
//! the copy that was there stayed under another,
//! and every start through the version reported a missing program until an install that could download ran again.
//! What a kill leaves at each step of an archive install,
//! and what removes it:
//! during the download,
//! the half file `<file>.download` in the downloads directory of that version,
//! which the next download of that version writes over,
//! and no hidden sibling;
//! during the unpacking,
//! before the completion marker,
//! an unfinished `.staging` sibling,
//! which the next install of that tool removes,
//! unpacking the archive again;
//! after the marker and before the version's name is given the copy,
//! a finished `.staging` sibling,
//! which the next install of that version publishes,
//! downloading nothing;
//! between the two renames a file system without the exchange needs,
//! the version's name absent,
//! the copy that was there at a `.replaced` sibling,
//! and the finished copy at a `.staging` sibling,
//! which the next install of that version publishes,
//! downloading nothing;
//! during the recovery itself,
//! after the finished copy was claimed under a `.recovered` sibling,
//! that sibling,
//! which the next install of that version publishes,
//! downloading nothing;
//! after that rename and before the `.replaced` sibling is removed,
//! a finished version directory,
//! and an unfinished `.replaced` sibling,
//! which the next install of that tool removes.
//!
//! The copy a `.staging` or `.recovered` sibling holds is published rather than the `.replaced` one put back:
//! the marker is written after the archive's SHA-256 matched the lock and after the program was found,
//! so that copy is a finished install,
//! while the copy at the version's name was unfinished,
//! which is why it was being replaced at all.
//! The marker records the tool,
//! the version and the archive's SHA-256,
//! and a copy is published only when those facts match the lock's row for the version.
//! A marker an earlier build wrote records no archive digest,
//! so it cannot vouch for the bytes and its copy is never published;
//! when such a copy sits under a name this build wrote and that installer has ended,
//! it is removed as an unverified leftover.
//! A name an earlier build wrote is never judged,
//! because it carries neither a boot nor a namespace,
//! so its copy stays where it is until a person removes it.
//! That copy is put back when the finished one is gone,
//! because an unfinished copy that holds the program is what the version had before the kill.
//!
//! A dead installer's copy is claimed before anything is published from it:
//! the claim is one rename to a name of this process's own,
//! so two installs recovering the same copy cannot both take it,
//! and the one that loses finds the name gone and publishes nothing.
//! Without the claim both would exchange the copy at the version's name in turn,
//! and the second would leave the unfinished copy there and the finished one deleted.
//!
//! A publish that fails keeps the copy it claimed:
//! the copy stays under this process's `.recovered` name,
//! which a later install in this process namespace finds dead and claims while the version's name holds no finished copy,
//! and removes as a second finished copy once that name holds one,
//! so no failure here costs the finished copy an offline machine needs.
//! The copy that was there is put back only when no installer that may still run holds a claim or a
//! staging name for that version:
//! such an installer is between its claim and its publish,
//! and a copy put at the version's name in that instant makes its rename fail.
//! An installer that may still run is one the system reports as running,
//! and one whose name belongs to this boot but that this reader cannot see:
//! another process namespace,
//! a record `/proc` hides,
//! and a name with no start time.
//! A name written by an earlier build carries neither a boot nor a namespace,
//! so nothing ever removes it,
//! and it does not hold the copy back:
//! waiting for it would keep the copy of a killed install aside for ever.
//!
//! Nothing here touches a sibling whose installer still runs,
//! or one whose installer cannot be judged
//! (`the_owner_of_a_hidden_sibling` says how that is told).
//! Nothing here removes a copy that may be the only one:
//! another version's hidden copy is removed only when it never finished,
//! and its completeness is judged by the marker alone,
//! since this run's lock does not say what program that version holds.
//! A finished copy of a version this run may not publish
//! (a newer finished version of a repository-published tool is installed,
//! and an older repository tool is never kept)
//! is left under its hidden name rather than published or removed:
//! a version directory of it would break that rule,
//! and the copy may be the only way that version comes back offline once the newer one is gone.
//! A finished copy the lock does not vouch for
//! (another tool,
//! another version,
//! another archive,
//! an unreadable marker,
//! or a marker an earlier build wrote)
//! is never published,
//! whatever it holds.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // leftovers-of-a-killed-install.ts
//! function recoverLeftovers(versionDir: string, firstBin: string, olderVersion: AnOlderVersion, expected: CopyRecord | undefined): Recovered;  // throws
//! type ARecoveryThatFailed = { error: ToolchainError; thePublishKeptItsCopy: boolean };
//! ```

/// The shapes a recovery works in,
/// which the repair and the report both name.
mod what_a_recovery_answers;

/// The step that takes a finished copy a dead install left,
/// so only one install publishes from it.
mod the_claim_of_a_finished_copy;

/// Imports the claim from its own file,
/// which the repair makes before it publishes anything.
use self::the_claim_of_a_finished_copy::claim_the_finished_copy;

/// Re-exports what one hidden sibling holds,
/// the sibling itself,
/// the rule that decides whether a version may be kept,
/// what became of the finished copy,
/// the hidden name that holds the copy a kill moved aside,
/// the hidden siblings this run removed,
/// what the recovery answered,
/// what a recovery that failed answers,
/// and the two questions the repair asks of those shapes.
pub(crate) use self::what_a_recovery_answers::{
    ARecoveryThatFailed,
    AnOlderVersion,
    FinishedCopy,
    HoldingInstaller,
    Holds,
    Leftover,
    Recovered,
    RemovedFinishedCopy,
    RemovedLeftover,
    WhyNotPublished,
    holds_of,
    may_be_published,
};

/// Re-exports which copy a failed recovery named,
/// which the module above re-exports for the controls that read it.
#[cfg(test)]
pub(crate) use self::what_a_recovery_answers::WhatTheFailureNamed;

/// Imports `fs` from `std`.
use std::fs;
/// Imports `io` from `std` for the error kind of a directory that is not there,
/// and of a rename whose name was taken first.
use std::io;
/// Imports what this module uses from `std::path`.
use std::path::Path;

/// Imports `ToolchainError` from the toolchain error module.
use super::super::error::ToolchainError;
/// Imports `file_name_text` from the toolchain helpers.
use super::super::util::file_name_text;
/// Imports the question that tells a finished install from an unfinished one.
use super::is_finished;
/// Imports the owner of a hidden sibling,
/// its liveness question,
/// and the check on the boot field of a name.
use super::the_owner_of_a_hidden_sibling::{
    Liveness,
    Owner,
    is_boot_id,
};
/// Imports the marker reads that tell a copy the lock vouches for from one it does not.
use super::{
    CopyRecord,
    marker_vouches,
};
/// Imports the hidden names and the publish step from the module above this one.
use super::{
    Published,
    discard_path,
    has_complete_marker,
    publish_staged,
};

/// What:
/// The fields of a hidden sibling's name,
/// read from the end of the name,
/// or nothing when the name is not one this scheme writes.
/// Why:
/// The name is the whole record,
/// and a version directory's own name may hold dots,
/// so the purpose and the fields naming the installer are read from the right.
/// The newest shape carries the process id,
/// its start time,
/// the process namespace and the boot;
/// a name from the release before this one carries the process id and start time only,
/// and a name from the release before that carries the process id alone.
/// The two older shapes cannot be judged by this reader,
/// so they are still read,
/// named,
/// and left where they are.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseName(entry: string): [string, Holds, Owner | undefined] | undefined
/// ```
fn parse_name(entry: &str) -> Option<(String, Holds, Option<Owner>)> {
    let stem: &str = entry.strip_prefix('.')?;
    let parts: Vec<&str> = stem.split('.').collect();
    // What: the newest shape ends in the purpose, process id, start, namespace and boot.
    // Why: a name with all four facts is the only one this reader may judge against its own.
    if let [rest @ .., purpose, pid, started, namespace, boot] = parts.as_slice()
        && !rest.is_empty()
        && let Some(holds) = holds_of(purpose)
        && let Ok(number) = pid.parse::<u32>()
        && let Ok(moment) = started.parse::<u64>()
        && let Ok(space) = namespace.parse::<u64>()
        && is_boot_id(boot)
    {
        return Some((
            rest.join("."),
            holds,
            Some(Owner {
                pid: number,
                started: moment,
                boot: Some((*boot).to_string()),
                namespace: Some(space),
            }),
        ));
    }
    // What: the shape the release before this one wrote ends in the purpose, process id and start time.
    // Why: without the boot and namespace it cannot be judged, but it is still one of this tool's names.
    if let [rest @ .., purpose, pid, started] = parts.as_slice()
        && !rest.is_empty()
        && let Some(holds) = holds_of(purpose)
        && let Ok(number) = pid.parse::<u32>()
        && let Ok(moment) = started.parse::<u64>()
    {
        return Some((
            rest.join("."),
            holds,
            Some(Owner {
                pid: number,
                started: moment,
                boot: None,
                namespace: None,
            }),
        ));
    }
    // What: the oldest shape ends in the purpose and process id alone.
    // Why: it carries no fact this reader can judge, so it keeps an owner of nothing.
    if let [rest @ .., purpose, pid] = parts.as_slice()
        && !rest.is_empty()
        && let Some(holds) = holds_of(purpose)
        && pid.parse::<u32>().is_ok()
    {
        return Some((rest.join("."), holds, None));
    }
    return None;
}

/// What:
/// Every hidden sibling a tool's directory holds,
/// whatever version it belongs to.
/// Why:
/// One listing names them all,
/// so a recovery of one version still removes the remains of a kill in another,
/// which no later install of that other version may ever come to clear.
/// A directory that is not there holds none,
/// which is the state before the tool's first install.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function hiddenSiblings(directory: string): Leftover[]
/// ```
pub(crate) fn hidden_siblings(directory: &Path) -> Result<Vec<Leftover>, ToolchainError> {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => {
            return Err(ToolchainError::failure(format!(
                "cannot list {} for the hidden names an earlier install left: {error}",
                directory.display()
            )));
        }
    };
    let mut siblings: Vec<Leftover> = Vec::new();
    for entry in entries.flatten() {
        let name: String = entry.file_name().to_string_lossy().to_string();
        let Some((version, holds, owner)) = parse_name(&name) else {
            continue;
        };
        siblings.push(Leftover {
            path: directory.join(&name),
            version,
            holds,
            owner,
        });
    }
    return Ok(siblings);
}

/// Whether the installer that made this hidden sibling is known to have ended.
/// A sibling its installer cannot be judged against is never treated as gone.
fn is_gone(sibling: &Leftover) -> bool {
    return sibling.liveness() == Liveness::Gone;
}

/// What:
/// Whether the installer that made this hidden sibling may still be running,
/// whether the system reports it running or this reader cannot judge it.
/// Why:
/// A copy put at the version's name while an installer is between its claim and its publish makes that installer's
/// rename fail,
/// and the finished copy of an install that is not this build's is then lost rather than retried;
/// so a name this reader cannot judge is treated as a live installer would be.
/// The one name this leaves out is a name from an earlier build's shape,
/// which carries neither a boot nor a namespace:
/// no install ever removes such a name,
/// so waiting for it would keep the copy of a killed install aside for ever,
/// and putting the copy back costs nothing once a publish of this build replaces a name that appeared.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function mayStillBeRunning(sibling: Leftover): boolean {
///   const answer = sibling.liveness();
///   if (answer === 'running') return true;
///   if (answer === 'gone') return false;
///   return sibling.owner?.carriesThisBootsIdentifier() === true;
/// }
/// ```
fn may_still_be_running(sibling: &Leftover) -> bool {
    return match sibling.liveness() {
        Liveness::Running => true,
        Liveness::Gone => false,
        Liveness::Unknown => sibling
            .owner
            .as_ref()
            .is_some_and(|owner| return owner.carries_this_boots_identifier()),
    };
}

/// What:
/// The installer that may still run and the name it holds the version under,
/// or nothing when no such installer holds a claim or a staging name for this version.
/// Why:
/// Such an installer is between its claim and its publish,
/// and a copy another install puts at the version's name in that instant makes its rename fail;
/// so the copy a killed install moved aside waits rather than being put back now,
/// and the report names the hidden name that waits and whether it holds a finished copy.
/// A name whose installer cannot be placed in any boot does not hold this off:
/// blocking would keep the copy of a killed install aside for ever (an earlier release wrote names with no start time),
/// and putting the copy back costs nothing once a publish of this build replaces a name that appeared.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function aLiveInstallerHolds(version: string, siblings: Leftover[]): Leftover | undefined
/// ```
fn a_live_installer_holds<'a>(
    version: &str,
    siblings: &'a [Leftover],
) -> Option<&'a Leftover> {
    return siblings.iter().find(|sibling| {
        return sibling.version == version
            && may_be_published(sibling.holds)
            && may_still_be_running(sibling);
    });
}

/// What:
/// Whether this hidden sibling holds a finished copy that no later install of this version may come to publish:
/// another version's,
/// which that version's own install publishes,
/// or this version's when the rule forbids keeping it and a later install could still publish it.
/// Why:
/// A finished copy may be the only one there is,
/// so nothing here removes one a later install of that version can publish;
/// a copy the lock does not vouch for can never be published by any install,
/// so it is removed rather than left as a directory a person has to find.
/// A run that holds no row for the archive can vouch for nothing,
/// so it removes no finished copy either.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function aFinishedCopyNothingHerePublishes(version: string, olderVersion: AnOlderVersion, expected: CopyRecord | undefined, sibling: Leftover): boolean {
///   if (!hasCompleteMarker(sibling.path)) return false;
///   if (sibling.version !== version) return true;
///   if (expected === undefined) return true;
///   if (olderVersion === 'may-stay') return false;
///   return markerVouches(sibling.path, expected);
/// }
/// ```
fn a_finished_copy_nothing_here_publishes(
    version: &str,
    older_version: &AnOlderVersion,
    expected: Option<&CopyRecord>,
    sibling: &Leftover,
) -> bool {
    if !has_complete_marker(&sibling.path) {
        return false;
    }
    if sibling.version != version {
        return true;
    }
    let Some(record) = expected else {
        return true;
    };
    if matches!(older_version, AnOlderVersion::MayStay) {
        return false;
    }
    return marker_vouches(&sibling.path, record);
}

/// What:
/// Puts back the copy a killed install had moved aside,
/// for the case where the finished copy it was replacing that copy with is gone.
/// Why:
/// An unfinished copy that holds the program is what the version had before the kill,
/// and a version's name that holds nothing makes every start through it report a missing program.
/// The rename is also the claim here:
/// a second install that finds the name taken leaves the winner's copy alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function putBackTheCopy(versionDir: string, replaced: string): boolean  // throws
/// ```
fn put_back_the_copy(
    version_dir: &Path,
    replaced: &Path,
) -> Result<bool, ToolchainError> {
    if let Err(error) = fs::rename(replaced, version_dir) {
        if fs::symlink_metadata(version_dir).is_ok() {
            // Another install put a copy there first.
            return Ok(false);
        }
        return Err(ToolchainError::failure(format!(
            "cannot put back the copy at {} that an earlier install had moved aside: {error}; that copy is still there, and nothing was put back at {}",
            replaced.display(),
            version_dir.display()
        )));
    }
    return Ok(true);
}

/// What:
/// Why this run did not publish a finished copy of a version whose own name holds no finished copy.
/// Why:
/// Both answers are facts this run read:
/// a version that may not be kept beside a newer one never reached a publish at all,
/// and a version that may be kept was left for the claim above,
/// which takes only a copy whose marker records the archive this lock names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function whyThisRunDidNotPublishIt(olderVersion: AnOlderVersion): WhyNotPublished {
///   return olderVersion === 'may-stay'
///     ? 'the-lock-does-not-record-its-archive'
///     : { anOlderRepositoryToolIsNeverKept: olderVersion.newer };
/// }
/// ```
fn why_this_run_did_not_publish_it(older_version: &AnOlderVersion) -> WhyNotPublished {
    // What: `match` on the rule's two answers, which is a discriminated union in TS.
    // Why: only the older-version rule keeps the publish path from running at all, so only it names that rule.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (olderVersion !== 'may-stay') return { anOlderRepositoryToolIsNeverKept: olderVersion.newer };
    // return 'the-lock-does-not-record-its-archive';
    // ```
    return match older_version {
        AnOlderVersion::MayStay => WhyNotPublished::TheLockDoesNotRecordItsArchive,
        // What: `.clone()` copies the newer version's name into the answer, because the answer outlives this borrow.
        // Why: the sentence is built after this function returns.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return { anOlderRepositoryToolIsNeverKept: newer };
        // ```
        AnOlderVersion::NeverKept {
            newer,
        } => WhyNotPublished::AnOlderRepositoryToolIsNeverKept {
            newer: newer.clone(),
        },
    };
}

/// What:
/// Makes the version directory what a killed install was about to make it,
/// then removes the hidden siblings of every installer that is known to have ended.
/// Why:
/// A later install finishes what a kill cut short instead of downloading the archive again:
/// the copy a `.staging` or `.recovered` sibling holds is complete,
/// checked against the lock's SHA-256 and marked before it was to be published,
/// so publishing it is the finished install the killed run would have published,
/// and a publish that fails leaves it under a name the next install claims.
/// A version whose finished copy is gone gets the copy that was there back,
/// unless a live install is between its claim and its publish for that version.
/// A version the rule forbids keeping gets neither,
/// and its finished copy stays hidden where it is.
/// Nothing of an installer that still runs,
/// and nothing of one whose installer cannot be judged,
/// is touched,
/// so a second install working in the same directory keeps its own copies.
/// A finished copy is published only when `expected` records the tool,
/// the version and the archive digest its own marker records too,
/// so a copy the lock does not vouch for never becomes a version.
/// A failure answers whether it read the directory's hidden names,
/// because the listing is the one step before which no copy was read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function recoverLeftovers(versionDir: string, firstBin: string, olderVersion: AnOlderVersion, expected: CopyRecord | undefined): Recovered  // throws
/// ```
pub(crate) fn recover_leftovers(
    version_dir: &Path,
    first_bin: &str,
    older_version: AnOlderVersion,
    expected: Option<&CopyRecord>,
) -> Result<Recovered, ARecoveryThatFailed> {
    let Some(directory) = version_dir.parent() else {
        return Ok(Recovered::default());
    };
    // What: the listing is the first read of the recovery, so a failure here is one that read no hidden name.
    // Why: the warning an install makes of it may then claim nothing about a copy.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const siblings = hiddenSiblings(directory).mapErr(nothingRead);
    // ```
    let siblings: Vec<Leftover> = hidden_siblings(directory)
        .map_err(|error| return ARecoveryThatFailed::nothing_read(error))?;
    let gone: Vec<&Leftover> = siblings
        .iter()
        .filter(|sibling| return is_gone(sibling))
        .collect();
    let version: String = file_name_text(version_dir);
    let mut outcome: Recovered = Recovered::default();
    let of_this_version: Vec<&Leftover> = gone
        .iter()
        .copied()
        .filter(|sibling| return sibling.version == version)
        .collect();
    // What: whether this version may be kept at all, which the owner's rule about an older repository tool decides.
    // Why: a version that may not be kept gets no copy at its name, published or put back.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const mayStay: boolean = olderVersion === 'may-stay';
    // ```
    let may_stay: bool = matches!(older_version, AnOlderVersion::MayStay);
    if !may_stay {
        let finished_copy_here: bool = of_this_version.iter().any(|sibling| {
            return may_be_published(sibling.holds)
                && a_finished_copy_nothing_here_publishes(
                    &version,
                    &older_version,
                    expected,
                    sibling,
                );
        });
        if finished_copy_here
            && let AnOlderVersion::NeverKept {
                newer,
            } = &older_version
        {
            outcome.finished_copy = FinishedCopy::AnOlderVersionIsNeverKept {
                newer: newer.clone(),
            };
        }
    } else if !is_finished(version_dir, first_bin) {
        if let Some(claimed) =
            claim_the_finished_copy(version_dir, &of_this_version, first_bin, expected)
                .map_err(ARecoveryThatFailed::a_finished_copy_it_named)?
        {
            // What: the publish is the one step whose failure name the warning may follow with the copy's fate.
            // Why: it keeps the finished copy this run holds, and a run that then installs the version makes it a second one.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // outcome.finishedCopy = match publishStaged(claimed.claim, versionDir, firstBin) { ... };
            // ```
            outcome.finished_copy = match publish_staged(&claimed.claim, version_dir, first_bin)
                .map_err(ARecoveryThatFailed::the_publish_kept_its_copy)?
            {
                Published::Here => FinishedCopy::PublishedHere {
                    copy: claimed.left_at,
                },
                Published::AlreadyThere => FinishedCopy::PublishedByAnotherInstall,
            };
        } else if let Some(holder) = a_live_installer_holds(&version, &siblings) {
            // What: whether the copy that was moved aside is still there to wait.
            // Why: the report says a copy waits only when a copy really waits.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const theCopyMovedAsideWaits = ofThisVersion.some((sibling) => sibling.holds === 'replaced-copy');
            // ```
            let the_copy_moved_aside_waits: bool = of_this_version
                .iter()
                .any(|sibling| return sibling.holds == Holds::ReplacedCopy);
            if the_copy_moved_aside_waits {
                outcome.left_for_a_live_install = Some(HoldingInstaller {
                    path: holder.path.clone(),
                    running: holder.liveness() == Liveness::Running,
                    finished_copy: is_finished(&holder.path, first_bin),
                });
            }
        } else if fs::symlink_metadata(version_dir).is_err() {
            let moved_aside = of_this_version
                .iter()
                .copied()
                .find(|sibling| return sibling.holds == Holds::ReplacedCopy);
            if let Some(copy_that_was_there) = moved_aside {
                outcome.restored = put_back_the_copy(version_dir, &copy_that_was_there.path)
                    .map_err(ARecoveryThatFailed::the_copy_that_was_moved_aside_it_named)?;
            }
        }
    }
    for sibling in &gone {
        if a_finished_copy_nothing_here_publishes(&version, &older_version, expected, sibling) {
            continue;
        }
        if outcome.left_for_a_live_install.is_some()
            && sibling.version == version
            && sibling.holds == Holds::ReplacedCopy
        {
            continue;
        }
        if fs::symlink_metadata(&sibling.path).is_err() {
            continue;
        }
        // What: what this run reads about the finished copy that is about to go with the directory:
        //  whether the version's own name holds a finished copy, which makes the removed one a second one,
        //  and, when it holds none, why this run did not publish it.
        // Why: the report has to tell a second copy from the version's only copy here, and the directory is gone
        //  after this loop, so both questions are read first.
        //  Only this version's copies reach the question: the test above leaves every other version's finished copy
        //  where it is, so a finished copy removed here belongs to `version_dir`.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const finishedCopy = isFinished(sibling.path, firstBin)
        //   ? isFinished(versionDir, firstBin)
        //     ? { secondCopy: version, programFound: firstBin !== '' }
        //     : { notAtItsName: version, whyNotPublished: theRuleThatKeptItOut() }
        //   : 'no-finished-copy';
        // ```
        let finished_copy: RemovedFinishedCopy = if is_finished(&sibling.path, first_bin) {
            // What: `isFinished` asks the version's own name, which `version_dir` is.
            // Why: another version's finished copy never reaches this point, so no other directory is read.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // const versionIsFinished = isFinished(versionDir, firstBin);
            // ```
            let version_is_finished: bool = is_finished(version_dir, first_bin);
            if version_is_finished {
                // What: `isFinished` reads the marker, and the program only when a name for it is known.
                // Why: the sentence says which of those two reads it made, and a tool that declares no bin has none.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // const programFound = firstBin !== '';
                // ```
                let program_found: bool = !first_bin.is_empty();
                RemovedFinishedCopy::SecondCopy {
                    version: version.clone(),
                    program_found,
                }
            } else {
                RemovedFinishedCopy::NotAtItsName {
                    version: version.clone(),
                    why_not_published: why_this_run_did_not_publish_it(&older_version),
                }
            }
        } else {
            RemovedFinishedCopy::NoFinishedCopy
        };
        discard_path(&sibling.path);
        if fs::symlink_metadata(&sibling.path).is_err() {
            outcome.removed.push(RemovedLeftover {
                path: sibling.path.clone(),
                finished_copy,
            });
        }
    }
    return Ok(outcome);
}
