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
use crate::package_repair_prewarm_a_held_name::{TheDirectory, a_name_only_this_run_holds};
/// The identity of the entry a marker's reader recorded, so a removal is bound to the entry this job read.
pub(crate) use crate::package_repair_prewarm_a_held_name::TheEntry;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// What the file this job writes inside a directory it makes is called:
/// the claim that travels with the entry itself, so a directory found there is this job's only when it
/// carries one naming this very build directory.
/// Why:
///   A marker beside the directory is a name another command can take;
/// the file inside it was written by the same call that made the directory,
/// so the entry carries its own ownership.
const A_CLAIM_INSIDE: &str = ".monochromatic-prewarm-of";

/// What:
///  The name of the claim file a directory of this job's carries.
/// Why:
///   The clone and the clean never look inside,
/// and a person who finds it reads the same lead the marker carries.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const aClaimInside = () => '.monochromatic-prewarm-of';
/// ```
pub(crate) fn a_claim_inside() -> &'static str {
    return A_CLAIM_INSIDE;
}

/// What:
///  This run's name for itself,
///  as its process and the moment it asked.
/// Why:
///   The marker and every claim this run writes name the same run,
///  so a claim inside a directory and the marker beside it are one run's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const aNameForThisRun = () => `${process.pid}-${nanos().toString(16)}`;
/// ```
pub(crate) fn a_name_for_this_run() -> String {
    return this_adds_name_for_itself();
}

/// What:
///  The claim that says the directory this file stands in was made by this run for this build directory.
/// Why:
///   It is written by the call that made the directory,
///  inside it,
///  so a directory that carries one is this job's even after a kill,
///  and a directory a person or another command made carries none.
///
/// # Errors
///
/// A build directory whose path is not UTF-8.
pub(crate) fn the_claim_text(to: &Path, begun_by: &str) -> Result<Vec<u8>, String> {
    return marker_text(to, begun_by, WhatTheMarkerRecords::TheStaging);
}

/// What:
///  What the claim file says a directory is,
///  when that file is this job's.
/// Why:
///   A removal of a directory is made only on a claim found inside it naming this very build directory,
/// so a directory whose claim is absent,
/// somebody else's,
/// or about another build directory is left where it is.
pub(crate) fn the_claim_a_directory_carries(directory: &Path) -> Option<TheMarker> {
    let found: Vec<u8> = match read_bounded(directory.join(A_CLAIM_INSIDE).as_path()) {
        Some((bytes, _)) => bytes,
        None => return None,
    };
    return match place_the_text(found.as_slice()) {
        TheMarkerThere::ThisJobs(claim) => Some(claim),
        _ => None,
    };
}

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
    /// The entry that was read, when the claim was made by reading a file,
    /// so a removal of the name is made only while that very entry stands there.
    pub(crate) read: Option<TheEntry>,
    /// Whether the run that wrote the marker also made the clean's target directory beside the staging
    /// name, which is what lets a later run remove that directory.
    /// A marker that does not say so proves nothing about a directory at the target name,
    /// and one written by a release that never wrote this field says nothing either.
    pub(crate) made_target: bool,
}

/// What a marker records about the names beside the build directory it claims.
/// Why:
///   A name is not proof that a thing is this job's,
///  so the marker is where a run writes down the directories it made:
///  the staging name always,
///  because the marker is written before anything is cloned into it,
///  and the clean's target directory only when the run that writes the marker made it.
///  The add writes a marker before its move aside and makes no target directory,
///  so its marker records the staging alone and a later filling cleans into a directory of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WhatTheMarkerRecords = 'theStaging' | 'theStagingAndItsTarget';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WhatTheMarkerRecords {
    /// The staging name alone.
    TheStaging,
    /// The staging name and the target directory beside it, both made by the run that wrote the marker.
    TheStagingAndItsTarget,
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
fn marker_text(to: &Path, begun_by: &str, records: WhatTheMarkerRecords) -> Result<Vec<u8>, String> {
    let Some(named) = to.to_str() else {
        return Err(format!("{} is not valid UTF-8, so no marker can name the build directory it takes", to.display()));
    };
    // The claim is built field by field, the way a record's payload is, so nothing here can panic on a shape the macro refuses.
    let mut fields: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
    fields.insert(String::from("repair"), serde_json::Value::from(CARGO_PREWARM_REPAIR));
    fields.insert(String::from("to"), serde_json::Value::from(named));
    fields.insert(String::from("begunBy"), serde_json::Value::from(begun_by));
    if records == WhatTheMarkerRecords::TheStagingAndItsTarget {
        fields.insert(String::from("madeTarget"), serde_json::Value::from(true));
    }
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
    let (text, read): (Vec<u8>, TheEntry) = match read_bounded(marker) {
        Some(found) => found,
        None => match entry_at(marker) {
            None => return TheMarkerThere::Nothing,
            // A link, a pipe, a directory, or a file larger than any marker of this job: somebody's, and never removed.
            Some(_) => return TheMarkerThere::SomebodyElses,
        },
    };
    return match place_the_text(text.as_slice()) {
        TheMarkerThere::ThisJobs(claim) => TheMarkerThere::ThisJobs(TheMarker { read: Some(read), ..claim }),
        other => other,
    };
}

/// What:
///  Whether a name is there at all, without following a link.
/// Why:
///   The read's refusals are told apart by what stands at the name afterwards,
///  and a name that no longer holds anything is the missing case.
fn entry_at(the_name: &Path) -> Option<TheEntry> {
    return TheEntry::at(the_name);
}

/// What:
///  One bounded read of a file this job expects to be its own,
///  with the identity of the file that was read,
///  or nothing when the read was refused.
/// Why:
///   The identity is what binds a later removal of the name to the file this job read,
/// so another command's file renamed over the name afterwards is not unlinked.
fn read_bounded(the_name: &Path) -> Option<(Vec<u8>, TheEntry)> {
    use std::io::Read as _;
    use std::os::unix::fs::MetadataExt as _;
    let file: std::fs::File = crate::private_storage::open_regular_file(the_name).ok()?;
    // The handle's own metadata, so the identity and the size are the file that is read.
    let metadata: std::fs::Metadata = file.metadata().ok()?;
    if metadata.len() > MARKER_LIMIT {
        return None;
    }
    let mut text: Vec<u8> = Vec::new();
    file.take(MARKER_LIMIT + 1).read_to_end(&mut text).ok()?;
    if text.len() as u64 > MARKER_LIMIT {
        return None;
    }
    return Some((text, TheEntry { device: metadata.dev(), inode: metadata.ino() }));
}

/// What:
///  The claim some text makes,
///  or that it is not this job's.
/// Why:
///   A marker and a claim inside a directory are the same text,
/// so one reader places both.
fn place_the_text(text: &[u8]) -> TheMarkerThere {
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
    // A marker of a release that never wrote the record says nothing about the target directory, and a value that is
    //  not the boolean this job writes is somebody's edit rather than this job's claim on a directory.
    let made_target: bool = match parsed.get("madeTarget") {
        None => false,
        Some(serde_json::Value::Bool(true)) => true,
        Some(_) => return TheMarkerThere::SomebodyElses,
    };
    return TheMarkerThere::ThisJobs(TheMarker { read: None, to: PathBuf::from(to), begun_by: String::from(begun_by), made_target });
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
#[cfg(test)]
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
///  The name this run takes a marker aside to,
///  shaped like the staging names the exclude patterns already name.
/// Why:
///   A marker is `<staging>.owner`,
/// so a taken-aside marker is named from the staging name of this run's own,
/// and a name a kill leaves behind is a name the person's `git status` never shows.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const anOwnMarkerName = (marker) => markerOf(anOwnName(markerNameWithoutSuffix(marker)));
/// ```
fn a_name_only_this_run_holds_of_a_marker(marker: &Path) -> PathBuf {
    let Some(name) = marker.file_name() else { return a_name_only_this_run_holds(marker) };
    let text: std::borrow::Cow<'_, str> = name.to_string_lossy();
    let Some(staging_name) = text.strip_suffix(MARKER_SUFFIX) else { return a_name_only_this_run_holds(marker) };
    let staging: PathBuf = marker.with_file_name(staging_name);
    return marker_of(a_name_only_this_run_holds(staging.as_path()).as_path());
}

/// What:
///  Take a marker this job wrote back,
///  and say why when it cannot.
/// Why:
///   The caller asks this only after `the_marker_there` placed the file as this job's own,
/// so a person's note at the name is never removed here.
///   The entry is taken aside first, never replacing anything, to a name only this run holds,
/// and the identity is compared again at that name:
/// an entry another command renamed over the marker name after the read
/// is put back where its writer put it and left,
/// and the unlink reaches only the very file this run read.
///
/// # Errors
///
/// A marker that cannot be taken aside or removed.
pub(crate) fn take_the_marker_back(
    directory: &TheDirectory,
    marker: &Path,
    read: Option<TheEntry>,
    before_the_take_aside: fn() -> bool,
    before_the_unlink: fn() -> bool,
) -> Result<TheRemoval, String> {
    // The name is bound to the file this job read: an entry another command renamed over the name after that
    //  read is not this job's file, and it stays where its writer put it.
    let at: PathBuf = directory.along(marker);
    let Some(before) = TheEntry::at(at.as_path()) else { return Ok(TheRemoval::Left) };
    if let Some(was) = read
        && before != was
    {
        return Ok(TheRemoval::Left);
    }
    if before_the_take_aside() {
        return Err(String::from("the marker was left where it is for a control"));
    }
    let aside: PathBuf = a_name_only_this_run_holds_of_a_marker(marker);
    let Some(held) = directory.take_aside(marker, aside.as_path())? else { return Ok(TheRemoval::Left) };
    if TheEntry::at(held.at()) != Some(before) {
        directory.put_back(&held, marker)?;
        return Ok(TheRemoval::Left);
    }
    if before_the_unlink() {
        return Err(String::from("the marker was left where it is for a control"));
    }
    return match directory.remove_the_held_file(&held) {
        Ok(()) => Ok(TheRemoval::Removed),
        Err(reason) => Err(format!("{reason}, so the copied build state is left where it is; run `cargo clean --offline --package <member>` in the worktree instead")),
    };
}

/// What a removal came to.
/// Why:
///   An entry this job cannot prove is its own is left, and the filling says so:
/// a directory left behind costs disk, and a person's file removed is what the rules forbid.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TheRemoval = 'removed' | 'left';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TheRemoval {
    /// The entry was this job's and is gone.
    Removed,
    /// The entry is not provably this job's, and it stays.
    Left,
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
///  naming the build directory it stages, the add that began it and the directories it made,
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
pub(crate) fn write_marker(marker: &Path, destination: &Path, records: WhatTheMarkerRecords, begun_by: &str) -> Result<TheMarkersAnswer, String> {
    let text: Vec<u8> = marker_text(destination, begun_by, records)?;
    return match crate::private_storage::write_private_file_unsynced(marker, text.as_slice()) {
        Ok(()) => Ok(TheMarkersAnswer::WrittenByThisRun),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Ok(TheMarkersAnswer::WrittenByAnother),
        Err(error) => Err(format!("{} could not be written, so nothing of this workspace was filled ({error})", marker.display())),
    };
}
