//! What:
//!  The durable record of the object one stage's creation made:
//! the container's path,
//! its identity,
//! and the instant the wrapper last wrote into it.
//! Why:
//!  A claim names a stage container before the container exists,
//! so the claim alone proves nothing about what stands at that name when a
//!  later command finds it.
//! The wrapper reads the identity of the object it made as soon as it exists,
//! and keeps the instant
//!  after which nothing of its own is written into that object,
//! so a later command can tell the object this wrapper made from a directory another party put at the same name,
//!  and the entries the wrapper wrote from one a person added after the wrapper died.
//! Only a stage whose record proves both is removed by the reaping
//! (`worktree_copy_stage_claim_reaping.rs`);
//! a stage whose record is absent,
//! damaged,
//! or does not match is
//!  left alone and reported,
//! which is the conservative answer for a folder that holds a copy of a checkout's ignored state.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await recordStageMadeObject({ commonDir, container, object }); // as soon as the container exists
//! const made = await readStageMadeObject(claim); // undefined for none, a damaged record included
//! await removeStageMadeObject(claim);
//! ```

use crate::fs_beneath::ObjectId;
use crate::path_bytes_in_records::{hex_of_path, path_of_hex};
use crate::worktree_copy_error::WorktreeCopyError;
use crate::worktree_copy_journal::{ensure_journal_root, journal_root, write_atomic_unsynced};
use crate::worktree_copy_journal_removal::remove_file_if_present;
use crate::worktree_copy_selection::STAGE_PREFIX;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// What:
///  The suffix of a stage's record of the object its creation made.
/// Why:
///   A claim ends in `.stage` and a destination record in `.add`;
/// a different suffix keeps this record from ever being read as either.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const MADE_SUFFIX = '.made';
/// ```
pub(crate) const MADE_SUFFIX: &str = ".made";

/// The line that holds the container's bytes.
const STAGE_LINE_PREFIX: &str = "stage=";

/// The line that holds the object's identity.
const OBJECT_LINE_PREFIX: &str = "object=";

/// The line that holds the instant the wrapper last wrote into the container.
const WRITTEN_LINE_PREFIX: &str = "written=";

/// What:
///  The object one stage's creation made,
/// as the record holds it.
/// Why:
///   The reaping compares the object's identity with what stands at the container's name,
/// and asks the
///  accounting (`worktree_copy_stage_entries_the_record_accounts_for.rs`) what the folder holds,
/// counting every entry born after `written` as one the record cannot account for.
/// The identity is absent only for the record a command writes before it creates the container:
/// a kill in the
///  instant between that creation and the record of the object it made leaves a record that names the
///  container and the instant, and the folder is then judged by its shape (it is a snapshot) and by what
///  was born after the instant.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type MadeObject = { container: string; object?: ObjectId; written: number };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MadeObject {
    /// The stage container the wrapper made.
    pub container: PathBuf,
    /// The identity of that container at the moment it was recorded,
    /// or nothing while the record names only the creation the command is about to make.
    pub object: Option<ObjectId>,
    /// The instant after which nothing of the wrapper's was written into the container.
    pub written: SystemTime,
}

/// What:
///  The record's path for a claim file,
/// which keeps the claim's own name and changes only its suffix.
/// Why:
///   The claim file and this record are one stage's two names,
/// so a reader that starts from either finds the other without a
///  second rule about which stage container names are reserved.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const madeOfAClaim = (claim: string) => claim.replace(/\.stage$/, MADE_SUFFIX);
/// ```
pub fn made_record_beside_claim(claim: &Path) -> PathBuf {
    return claim.with_extension(&MADE_SUFFIX[1..]);
}

/// What:
///  The text of a record:
/// the container's bytes,
/// the object's identity when there is one,
/// and the instant,
/// one field per line.
/// Why:
///   A path is written as its bytes,
/// so a container whose name is not text is recorded whole,
/// and each line is
///  read by its own key rather than by its position.
/// The identity line is left out only where the command records the creation before the container exists.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${STAGE_LINE}${hex(container)}\n${object === undefined ? '' : `${OBJECT_LINE}${object}\n`}${WRITTEN_LINE}${seconds}.${nanos}\n`
/// ```
fn made_text(container: &Path, object: Option<ObjectId>, written: SystemTime) -> String {
    let since: Duration = written.duration_since(UNIX_EPOCH).unwrap_or(Duration::ZERO);
    let identity: String = match object {
        Some(named) => format!("{OBJECT_LINE_PREFIX}{}\n", named.encode()),
        None => String::new(),
    };
    return format!(
        "{STAGE_LINE_PREFIX}{}\n{identity}{WRITTEN_LINE_PREFIX}{}.{}\n",
        hex_of_path(container),
        since.as_secs(),
        since.subsec_nanos()
    );
}

/// What:
///  The record read as text,
/// or nothing when it is not a record this release writes.
/// Why:
///   A record that does not read proves nothing,
/// so anything unexpected is answered as "nothing proved"
/// instead of
///  failing a command that was only looking.
/// The identity line may be absent,
/// and no line may repeat.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const made = readMadeText(text); // undefined unless the lines parse and nothing else follows them
/// ```
fn read_made_text(text: &str) -> Option<MadeObject> {
    let mut lines: std::str::Lines<'_> = text.lines();
    let container: PathBuf = path_of_hex(lines.next()?.strip_prefix(STAGE_LINE_PREFIX)?)?;
    // The identity line is the one a record written before the container exists leaves out, so the line after the container
    // is the instant when it is not that line.
    let second: &str = lines.next()?;
    let (object, instant): (Option<ObjectId>, &str) = match second.strip_prefix(OBJECT_LINE_PREFIX) {
        Some(identity) => (Some(ObjectId::decode(identity)?), lines.next()?),
        None => (None, second),
    };
    let (seconds, nanos) = instant.strip_prefix(WRITTEN_LINE_PREFIX)?.split_once('.')?;
    let written: SystemTime = UNIX_EPOCH + Duration::new(seconds.parse().ok()?, nanos.parse().ok()?);
    // Nothing follows the instant: a line a reader does not know could be another field, and this record has none.
    if lines.next().is_some() {
        return None;
    }
    return Some(MadeObject { container, object, written });
}

/// What:
///  Reads the record of the object a claim's stage creation made,
/// or nothing when there is none to read.
/// Why:
///   The reaping asks one question of it:
/// does this record prove the object at the container's name is the one the wrapper made?
/// A record that cannot be read proves nothing and is never acted on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const made = await readStageMadeObject(claim); // undefined: absent, unreadable, or not a record of this shape
/// ```
pub fn read_made(claim: &Path) -> Option<MadeObject> {
    let text: String = std::fs::read_to_string(made_record_beside_claim(claim)).ok()?;
    return read_made_text(text.as_str());
}

/// What:
///  Records the object a stage's creation made,
/// at the instant it is called.
/// Why:
///   Called as soon as the object exists,
/// and called again after the wrapper's last write into it,
/// so the record's
///  instant is the cut-off the accounting compares against.
/// Neither call flushes anything:
/// the record is published whole by one rename,
/// a kill leaves the page cache
///  intact,
/// and a machine crash that loses it only makes a later command report the folder instead of removing
///  it,
/// which is the safe direction;
/// a flush on the ladder's path is what the swap of a subvolume makes a
///  commit of the whole filesystem,
/// which waits behind every other writer's data.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recordStageMadeObject({ commonDir, container, object }); // rename only, no flush
/// ```
pub fn record_made(common_dir: &Path, container: &Path, object: ObjectId) -> Result<(), WorktreeCopyError> {
    return record_made_at(common_dir, container, object, SystemTime::now());
}

/// What:
///  `record_made` with the instant given,
/// for a caller that measures its own cut-off.
/// Why:
///   The ladder records the object as soon as the snapshot exists and raises the cut-off once its last write is done;
/// a
///  control writes a record whose cut-off it chooses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recordStageMadeObjectAt({ commonDir, container, object, written });
/// ```
pub fn record_made_at(common_dir: &Path, container: &Path, object: ObjectId, written: SystemTime) -> Result<(), WorktreeCopyError> {
    return record_at(common_dir, container, Some(object), written);
}

/// What:
///  Records the creation a command is about to make,
/// before the container exists,
/// with the instant after which nothing of the wrapper's is written into it.
/// Why:
///   The identity of an object cannot be read before it exists,
/// and a kill in the instant between creating it and
///  recording its identity left a whole snapshot of the checkout that nothing could prove the wrapper
///  made (`worktree_copy_stage_reaping_proof_and_words.rs`).
/// This record names the container and the instant first,
/// so the next command judges such a
///  folder by its shape and by what was born after that instant,
/// and settles it instead of keeping it for
///  ever.
/// It flushes nothing,
/// for the reason [`record_made`] gives.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recordCreatingAt({ commonDir, container, written }); // no identity: the container does not exist yet
/// ```
pub fn record_creating_at(common_dir: &Path, container: &Path, written: SystemTime) -> Result<(), WorktreeCopyError> {
    return record_at(common_dir, container, None, written);
}

/// What:
///  `record_made_at` and `record_creating_at`:
/// the one write of a stage's record,
/// with the identity when the caller has one.
/// Why:
///   One writer keeps the record's shape in one place,
/// so a reader never meets a record the writer does not make.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recordAt({ commonDir, container, object, written });
/// ```
fn record_at(common_dir: &Path, container: &Path, object: Option<ObjectId>, written: SystemTime) -> Result<(), WorktreeCopyError> {
    let Some(path) = made_record_path(common_dir, container) else {
        // A stage name is reserved by the claim naming rule; a name outside it has no claim to record for.
        return Ok(());
    };
    ensure_journal_root(common_dir)?;
    return write_atomic_unsynced(path.as_path(), made_text(container, object, written).as_bytes(), "stage object record");
}

/// What:
///  The record's path for a container,
/// or nothing when the container's name is not one a claim reserves.
/// Why:
///   A record written beside a claim this wrapper never made would be a file the reaping never reads and nothing
///  removes;
/// the reserved prefix keeps every other stage name out of the journal root.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const path = madeRecordPath(commonDir, container); // join(root, `${stem}${MADE_SUFFIX}`)
/// ```
fn made_record_path(common_dir: &Path, container: &Path) -> Option<PathBuf> {
    let name: &str = container.file_name()?.to_str()?;
    let identifier: &str = name.strip_prefix(STAGE_PREFIX)?;
    if identifier.is_empty() {
        return None;
    }
    return Some(journal_root(common_dir).join(format!("{identifier}{MADE_SUFFIX}")));
}

/// What:
///  Removes the record of the object a claim's stage creation made,
/// accepting a record that is already gone.
/// Why:
///   A record is only worth keeping while the claim beside it stands,
/// so a release removes it first:
/// a command killed after
///  that leaves a claim whose stage it must report,
/// never a record nothing names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await removeStageMadeObject(claim);
/// ```
pub fn remove_made(claim: &Path) -> Result<(), WorktreeCopyError> {
    return remove_file_if_present(made_record_beside_claim(claim).as_path());
}

/// What:
///  Whether the object at the container's name is the one the record names,
/// a link at either name not followed.
/// Why:
///   The record proves what the wrapper made only while that object is still at the name;
/// another object there,
/// a
///  directory a person made,
/// or one moved in from elsewhere,
/// has another identity and is never removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// identityOfName(container) === made.object
/// ```
pub fn is_the_recorded_object(container: &Path, made: &MadeObject) -> bool {
    let Some(recorded) = made.object else { return false };
    return crate::worktree_add_binding::path_identity(container) == Some(recorded);
}
