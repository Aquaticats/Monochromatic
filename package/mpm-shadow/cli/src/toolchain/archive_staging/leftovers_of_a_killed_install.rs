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
//! when its installer has ended and it belongs to the version this run installs,
//! it is removed as an unverified leftover.
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
//! Nothing here touches a sibling whose installer still runs,
//! or one whose installer cannot be judged
//! (`the_owner_of_a_hidden_sibling` says how that is told).
//! Nothing here removes a copy that may be the only one:
//! another version's hidden copy is removed only when it never finished,
//! and its completeness is judged by the marker alone,
//! since this run's lock does not say what program that version holds.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // leftovers-of-a-killed-install.ts
//! type Holds = 'unpacking' | 'replaced-copy' | 'recovered';
//! type Recovered = { published: boolean; restored: boolean; removed: string[] };
//! function recoverLeftovers(versionDir: string, firstBin: string): Recovered;  // throws
//! ```

/// Imports `fs` from `std`.
use std::fs;
/// Imports `io` from `std` for the error kind of a directory that is not there,
/// and of a rename whose name was taken first.
use std::io;
/// Imports what this module uses from `std::path`.
use std::path::{
    Path,
    PathBuf,
};

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
/// Imports the hidden names,
/// the three purposes and the publish step from the module above this one.
use super::{
    RECOVERED_PURPOSE,
    REPLACED_PURPOSE,
    STAGING_PURPOSE,
    discard_path,
    has_complete_marker,
    hidden_sibling,
    publish_staged,
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
/// What one recovery of a tool's version directory found and did.
/// Why:
/// The install report says whether a finished copy became the version
/// or the copy that was there came back,
/// and which hidden siblings went.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Recovered = { published: boolean; restored: boolean; removed: string[] };
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct Recovered {
    /// Whether a finished copy a killed install left became the version directory.
    pub(crate) published: bool,
    /// Whether the copy a killed install moved aside was put back.
    pub(crate) restored: bool,
    /// The hidden siblings this run removed,
    /// which are gone when it returns.
    pub(crate) removed: Vec<PathBuf>,
}

/// What a purpose in a hidden sibling's name says about what it holds,
/// or nothing when it names none of the three.
fn holds_of(purpose: &str) -> Option<Holds> {
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
fn may_be_published(holds: Holds) -> bool {
    return matches!(holds, Holds::Unpacking | Holds::Recovered);
}

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
                "cannot list {} for the leftovers of a killed install: {error}",
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
/// Takes a finished copy a dead install left under a name of this process's own,
/// so no other install publishes from the same directory.
/// Why:
/// The rename is the claim,
/// and it is atomic:
/// two installs that recover the same copy cannot both win,
/// and the one that loses finds the name gone and publishes nothing.
/// A copy this process already claimed is taken as it is,
/// because a second install of the same tool in one process is the same install.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function claimTheFinishedCopy(versionDir: string, candidates: Leftover[], firstBin: string): string | undefined
/// ```
fn claim_the_finished_copy(
    version_dir: &Path,
    of_this_version: &[&Leftover],
    first_bin: &str,
    expected: Option<&CopyRecord>,
) -> Result<Option<PathBuf>, ToolchainError> {
    let claim: PathBuf = hidden_sibling(version_dir, RECOVERED_PURPOSE, Owner::here());
    if fs::symlink_metadata(&claim).is_ok() {
        return Ok(Some(claim));
    }
    let candidate = of_this_version.iter().copied().find(|leftover| {
        return may_be_published(leftover.holds)
            && is_finished(&leftover.path, first_bin)
            && expected.is_some_and(|record| return marker_vouches(&leftover.path, record));
    });
    let Some(finished) = candidate else {
        return Ok(None);
    };
    return match fs::rename(&finished.path, &claim) {
        Ok(()) => Ok(Some(claim)),
        // Another install claimed it first, so its copy is the one that will be published.
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(ToolchainError::failure(format!(
            "cannot claim the finished install at {} that a killed install left: {error}",
            finished.path.display()
        ))),
    };
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
            "cannot put back the copy at {} that a killed install had moved aside: {error}",
            replaced.display()
        )));
    }
    return Ok(true);
}

/// What:
/// Makes the version directory what a killed install was about to make it,
/// then removes the hidden siblings of every installer that is known to have ended.
/// Why:
/// A later install finishes what a kill cut short instead of downloading the archive again:
/// the copy a `.staging` or `.recovered` sibling holds is complete,
/// checked against the lock's SHA-256 and marked before it was to be published,
/// so publishing it is the finished install the killed run would have published.
/// A version whose finished copy is gone gets the copy that was there back.
/// Nothing of an installer that still runs,
/// and nothing of one whose installer cannot be judged,
/// is touched,
/// so a second install working in the same directory keeps its own copies.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function recoverLeftovers(versionDir: string, firstBin: string): Recovered
/// ```
pub(crate) fn recover_leftovers(
    version_dir: &Path,
    first_bin: &str,
    expected: Option<&CopyRecord>,
) -> Result<Recovered, ToolchainError> {
    let Some(directory) = version_dir.parent() else {
        return Ok(Recovered::default());
    };
    let siblings: Vec<Leftover> = hidden_siblings(directory)?;
    let gone: Vec<&Leftover> = siblings
        .iter()
        .filter(|sibling| return is_gone(sibling))
        .collect();
    let version: String = file_name_text(version_dir);
    let mut outcome: Recovered = Recovered::default();
    if !is_finished(version_dir, first_bin) {
        let of_this_version: Vec<&Leftover> = gone
            .iter()
            .copied()
            .filter(|sibling| return sibling.version == version)
            .collect();
        if let Some(claim) =
            claim_the_finished_copy(version_dir, &of_this_version, first_bin, expected)?
        {
            publish_staged(&claim, version_dir, first_bin)?;
            outcome.published = true;
        } else if fs::symlink_metadata(version_dir).is_err() {
            let moved_aside = of_this_version
                .iter()
                .copied()
                .find(|sibling| return sibling.holds == Holds::ReplacedCopy);
            if let Some(copy_that_was_there) = moved_aside {
                outcome.restored = put_back_the_copy(version_dir, &copy_that_was_there.path)?;
            }
        }
    }
    for sibling in &gone {
        if sibling.version != version && has_complete_marker(&sibling.path) {
            // Another version's finished copy may be the only one that version has,
            // and publishing it is that version's own install to do.
            continue;
        }
        if fs::symlink_metadata(&sibling.path).is_err() {
            continue;
        }
        discard_path(&sibling.path);
        if fs::symlink_metadata(&sibling.path).is_err() {
            outcome.removed.push(sibling.path.clone());
        }
    }
    return Ok(outcome);
}
