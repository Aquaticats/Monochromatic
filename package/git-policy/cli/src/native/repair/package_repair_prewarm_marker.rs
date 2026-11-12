//! What:
//!  The claim one filling leaves beside the names it takes:
//!  what a marker says,
//!  how one is read,
//!  and whose claim it can be.
//! Why:
//!  The staging copy,
//!  the clean's target directory and the marker are names this job's by convention,
//! and a directory at one of them is this job's only when a marker says so.
//! A marker states which repair began,
//! which build directory it staged,
//! and which add began it,
//! so an add believes one only when it can place it,
//! and a copy that carried a marker into another worktree cannot read it as a claim on that
//! worktree's names.
//! This file is that vocabulary alone:
//! the add's staging step and the service's filling decide with it
//! (`package_repair_prewarm_job.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // written whole by one exclusive write, read whole by one that cannot wait, placed by its own text
//! const claim = { repair: 'cargo-prewarm', to: buildDirectory, begunBy: `${pid}-${nanos.toString(16)}` };
//! ```
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// What:
///  The marker that says the staging copy and the clean's target beside it are this job's own.
/// Why:
///   The two names are this job's by convention,
/// and a directory at one of them is this job's only when a filling of it said so:
/// the marker is written before anything is cloned or cleaned,
/// so a filling that was killed leaves the proof its successor needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const markerOf = (staging) => `${staging}${MARKER_SUFFIX}`;
/// ```
pub(crate) fn marker_of(staging: &Path) -> PathBuf {
    let mut named: OsString = staging.as_os_str().to_os_string();
    named.push(MARKER_SUFFIX);
    return PathBuf::from(named);
}

/// What the marker of the staging name is suffixed with,
/// beside the staging directory.
const MARKER_SUFFIX: &str = ".owner";

/// What the marker's JSON names this repair as,
/// so a marker another repair wrote is never read as this job's claim.
const CARGO_PREWARM_REPAIR: &str = "cargo-prewarm";

/// What:
///  The lead every marker of this job starts with.
/// Why:
///   The name is a convention anyone can type,
/// so one read has to tell this job's claim from a person's note:
/// the lead says which program and which repair wrote the file,
/// and the JSON after it says which build directory it staged and which add began it.
/// A person who finds one reads what it claims in the file itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MARKER_LEAD = "monochromatic-git-policy-cli's Cargo prewarm job wrote this file. This line and the JSON after it say which repair began, for which build directory, and in which add: ";
/// ```
const MARKER_LEAD: &str = "monochromatic-git-policy-cli's Cargo prewarm job wrote this file. \
This line and the JSON after it say which repair began, for which build directory, and in which add: ";

/// What:
///  The most bytes a marker of this job can hold.
/// Why:
///   A marker names one build directory and one add's identity,
/// and the longest path the kernel accepts is half this,
/// so a larger file at that name is somebody's and is never read whole or removed.
const MARKER_LIMIT: u64 = 8192;

/// What:
///  What a marker of this job says,
/// so a reader can place it.
/// Why:
///   Three facts make a marker placeable:
/// which repair it began,
/// which build directory it staged,
/// and which add began it.
/// A file that says fewer is not this job's claim on any name,
/// and a claim on another build directory came here inside a copy of that worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TheMarker = { repair: string; to: string; begunBy: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TheMarker {
    /// The build directory whose staging name the marker claims.
    pub(crate) to: PathBuf,
    /// The add that began the filling, as its process and the moment it asked.
    pub(crate) begun_by: String,
}

/// What:
///  What stands at a marker name,
///  as far as one build directory can tell.
/// Why:
///   These are the answers an add or a filling acts on:
/// nothing,
/// a claim this job wrote,
/// whose own `to` says which build directory it is about,
/// and a file this job never wrote.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum TheMarkerThere {
    /// Nothing is at the name.
    Nothing,
    /// This job wrote it,
    /// and the claim says which build directory and which add.
    ThisJobs(TheMarker),
    /// Somebody else's file:
    /// a link,
    /// a pipe,
    /// a file too large for a marker,
    /// or text this job did not write.
    SomebodyElses,
}

/// What:
///  The bytes of one marker.
/// Why:
///   The claim is written whole by one exclusive write,
/// so a file at that name either holds this job's JSON or is somebody else's,
/// and a reader that knows the lead can place it without asking anything else.
///
/// # Errors
///
/// A build directory whose path is not UTF-8,
/// which no marker could name back.
fn marker_text(to: &Path, begun_by: &str) -> Result<Vec<u8>, String> {
    let Some(named) = to.to_str() else {
        return Err(format!("{} is not valid UTF-8, so no marker can name the build directory it takes", to.display()));
    };
    // The claim is built field by field, the way a record's payload is, so nothing here can panic on a shape the macro refuses.
    let mut fields: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
    fields.insert(String::from("repair"), serde_json::Value::from(CARGO_PREWARM_REPAIR));
    fields.insert(String::from("to"), serde_json::Value::from(named));
    fields.insert(String::from("begunBy"), serde_json::Value::from(begun_by));
    let claimed: String = serde_json::Value::Object(fields).to_string();
    let mut text: Vec<u8> = Vec::with_capacity(MARKER_LEAD.len() + claimed.len() + 1);
    text.extend_from_slice(MARKER_LEAD.as_bytes());
    text.extend_from_slice(claimed.as_bytes());
    text.push(b'\n');
    return Ok(text);
}

/// What:
///  What stands at a marker name,
/// read only from a name that holds a regular file,
/// through an open that cannot wait.
/// Why:
///   The name alone is a convention anyone can type,
/// and only the text this job writes is proof that it made the directories the name is derived from.
/// The marker is published whole by an exclusive write,
/// so a file at that name either holds that text or is somebody else's.
/// A named pipe at that name would hold a plain read for ever,
/// because opening one for reading waits for a writer that may never come,
/// and one foreign object at the name would then stop the service's filling with no end
/// (SPEC.md,
/// "Every wait has a bound").
/// `read_regular_file` opens without waiting and reads only what the descriptor proves is a file,
/// so a pipe,
/// a socket or a link is somebody else's and answers at once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const theMarkerThere = (marker) => placeThisJobsClaim(readRegularFile(marker, MARKER_LIMIT));
/// ```
pub(crate) fn the_marker_there(marker: &Path) -> TheMarkerThere {
    let text: Vec<u8> = match crate::private_storage::read_regular_file(marker, MARKER_LIMIT) {
        Ok(text) => text,
        Err(crate::private_storage::ReadRefusal::Missing) => return TheMarkerThere::Nothing,
        // A link, a pipe, a directory, or a file larger than any marker of this job: somebody's, and never removed.
        Err(_) => return TheMarkerThere::SomebodyElses,
    };
    let Some(claimed) = text.strip_prefix(MARKER_LEAD.as_bytes()) else { return TheMarkerThere::SomebodyElses };
    let Ok(parsed) = serde_json::from_slice::<serde_json::Value>(claimed) else { return TheMarkerThere::SomebodyElses };
    let (Some(repair), Some(to), Some(begun_by)) = (
        parsed.get("repair").and_then(serde_json::Value::as_str),
        parsed.get("to").and_then(serde_json::Value::as_str),
        parsed.get("begunBy").and_then(serde_json::Value::as_str),
    ) else {
        return TheMarkerThere::SomebodyElses;
    };
    if repair != CARGO_PREWARM_REPAIR || begun_by.is_empty() {
        return TheMarkerThere::SomebodyElses;
    }
    return TheMarkerThere::ThisJobs(TheMarker { to: PathBuf::from(to), begun_by: String::from(begun_by) });
}

/// What:
///  Whether the marker at this name claims this very build directory.
/// Why:
///   A marker another worktree's copy carried here names a build directory that is not this one,
/// so it proves nothing about the names beside this one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const markerClaims = (marker, destination) => theMarkerThere(marker).claim?.to === destination;
/// ```
pub(crate) fn marker_claims(marker: &Path, destination: &Path) -> bool {
    return matches!(the_marker_there(marker), TheMarkerThere::ThisJobs(claim) if claim.to == destination);
}

/// What:
///  This add's name for itself,
/// as its process and the moment it asked.
/// Why:
///   A marker says which add began the filling,
/// so an add reads a marker a copy carried from another worktree as one begun by a process that
/// is not this one,
/// and a kill at any instruction leaves the marker naming the add that will not finish it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const thisAddsNameForItself = () => `${process.pid}-${nanos().toString(16)}`;
/// ```
fn this_adds_name_for_itself() -> String {
    let since_the_epoch: std::time::Duration = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap_or_default();
    return format!("{}-{:x}", std::process::id(), since_the_epoch.as_nanos());
}

/// What:
///  Remove a marker this job wrote,
///  and say why when it cannot.
/// Why:
///   The caller asks this only after `the_marker_there` placed the file as this job's own,
/// so a person's note at the name is never removed here.
/// A marker that cannot be taken back leaves the names claimed by what this add cannot place,
/// so the state stays where it is and the add cleans it there.
///
/// # Errors
///
/// A marker that cannot be removed.
pub(crate) fn take_the_marker_back(marker: &Path) -> Result<(), String> {
    return match std::fs::remove_file(marker) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!(
            "{} could not be taken back ({error}), so the copied build state is left where it is; run `cargo clean --offline --package <member>` in the worktree instead",
            marker.display()
        )),
    };
}

/// What a run's exclusive write of one marker came to.
/// Why:
///   The marker is removed when the filling ends,
///   and removing the name is this run's only when the write that made it was this run's:
/// a name somebody else took is theirs,
/// whatever it holds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TheMarkersAnswer {
    /// This run created the marker,
    /// so it is this run's to remove.
    WrittenByThisRun,
    /// Another writer had the name already,
    /// so this run wrote nothing there.
    WrittenByAnother,
}

/// What:
///  Write the marker that claims the two names of one workspace,
///  naming the build directory it stages and the add that began it,
///  and say whether it is this run's.
/// Why:
///   An exclusive write makes two fillings of one workspace agree on one marker instead of replacing
/// each other's,
/// and the answer says which of them made it:
/// `AlreadyExists` means another command's file is at that name,
/// which may be a person's own note,
/// so this run neither treats it as its claim nor removes it later.
/// The marker names what it claims,
/// so a copy that carries it into another worktree cannot read it as a claim on the names beside
/// that copy's build directory.
///
/// # Errors
///
/// A marker that cannot be written,
/// and a build directory whose path no marker can name.
pub(crate) fn write_marker(marker: &Path, destination: &Path) -> Result<TheMarkersAnswer, String> {
    let text: Vec<u8> = marker_text(destination, this_adds_name_for_itself().as_str())?;
    return match crate::private_storage::write_private_file_unsynced(marker, text.as_slice()) {
        Ok(()) => Ok(TheMarkersAnswer::WrittenByThisRun),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Ok(TheMarkersAnswer::WrittenByAnother),
        Err(error) => Err(format!("{} could not be written, so nothing of this workspace was filled ({error})", marker.display())),
    };
}
