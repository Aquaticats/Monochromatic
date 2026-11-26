//! What:
//!  The records a removal writes beside a worktree's registration,
//!  and the question they answer:
//!  which worktree the record was written for.
//! Why:
//!  A record outlives the worktree it names,
//!  and a person may add another worktree at the same path meanwhile
//!  (the README's advice is to remove one by hand and add it again),
//!  so the record cannot name its worktree by path alone.
//!  The device and the inode of the registration and of the worktree root are the kernel's answer to "which directory is this",
//!  and they are the answer this program's own writes never move.
//!  The second record names the worktree a landed sweep compared this one's ignored files with,
//!  which no command of the merge is there to name any more once the sweep has returned.
//!  Both live inside the registration,
//!  so a removal of the worktree takes them with the directory.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const identity = (target, registration) => `${dev}:${ino(registration)} ${dev}:${ino(target)}`;
//! await recordWhatTheServiceCompares(registration, target, comparedWith);
//! const why = whyThisIsAnotherWorktree(item, target, registration); // string | undefined
//! ```

use crate::private_storage::write_private_file_unsynced;
use crate::work_item::WorkItem;
use crate::worktree_registration_proof::read_record_here;
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt as _;
use std::path::{
    Path,
    PathBuf,
};

/// The file beside a worktree's registration that names the worktree a landed sweep compared its ignored files with.
///
/// It lives in the registration,
/// so a removal of the worktree takes it with the directory,
/// and a worktree that is kept keeps it for the next reading.
pub const COMPARED_WITH_RECORD_NAME: &str = "monochromatic-git-policy-cli-removal-compared-with";

/// The file beside a worktree's registration that names the worktree and the registration the record was written for.
///
/// It holds the device and the inode of each,
/// which is what tells the worktree the proof was made for from one a person
/// added at the same path while the record waited.
pub const IDENTITY_RECORD_NAME: &str = "monochromatic-git-policy-cli-removal-identity";

/// What:
///  The bytes one worktree removal is keyed by:
///  its path,
///  and the identity the filesystem gives the registration that is being removed.
/// Why:
///  A record's id is its idempotency key,
///  so two sweeps of one landing must write one record,
///  and two worktrees added one after another at the same path must not share one:
///  a `done` record is never removed,
///  and a second removal keyed by the path alone would be taken for the first
///  and never run.
///  A registration that cannot be read leaves the path alone,
///  which is the record this release wrote before the identity was added.
///  The bytes are the path and the registration's device,
/// inode and change time,
///  exactly as the release before this one computed them:
///  a record queued while that release ran is still the record this one finds under its id,
///  and a change of the name here would have left it queued for a worktree nothing removes.
///  What proves which worktree the record was written for is not this name:
///  it is the identity the hand-over writes beside the registration
///  (`identity_standing_there`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const removalKey = (target, registration) => concat(pathBytes(target), identityBytes(stat(registration)));
/// ```
pub(crate) fn removal_key(
    target: &Path,
    registration: &Path,
) -> Vec<u8> {
    let mut key: Vec<u8> = path_bytes(target);
    use std::os::unix::fs::MetadataExt as _;
    if let Ok(found) = std::fs::metadata(registration) {
        key.extend_from_slice(found.dev().to_le_bytes().as_slice());
        key.extend_from_slice(found.ino().to_le_bytes().as_slice());
        key.extend_from_slice(found.ctime().to_le_bytes().as_slice());
        key.extend_from_slice(found.ctime_nsec().to_le_bytes().as_slice());
    }
    return key;
}

/// What:
///  The name one worktree removal is recorded under,
///  which is also what a service tells the worktree the proof was made for from one added at the same path.
/// Why:
///  A record's id is its idempotency key,
///  and the service recomputes the name it was written under before it destroys anything:
///  a value that differs says a worktree a person added at the same path stands there,
///  and that one was never proven anything.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const named = (target, registration) => sha1(removalKey(target, registration)).hex();
/// ```
pub fn removal_named_by(
    target: &Path,
    registration: &Path,
) -> String {
    return crate::hexadecimal_of_bytes::hexadecimal_of_bytes(
        crate::subvolume_sha::sha1(removal_key(target, registration).as_slice()).as_slice(),
    );
}

/// What:
///  The two directories whose identity says which worktree a record was written for:
///  the registration beside it,
///  and the worktree root.
/// Why:
///  A path names whatever stands there when the service runs,
///  so the record has to name the worktree itself.
///  The device and the inode are the kernel's own answer to "which directory is this",
///  and they are the answer this program's own writes never move:
///  this program writes inside a registration's own Git directory and inside a worktree while a record waits,
///  which moves a change time and a modification time but never an inode.
///  A registration that a person removed and a worktree added again at the same path are two directories,
///  with two inodes,
///  however the filesystem may have numbered them before.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const identity = (target, registration) => `${dev}:${ino(registration)} ${dev}:${ino(target)}`;
/// ```
pub(crate) fn identity_standing_there(
    target: &Path,
    registration: &Path,
) -> String {
    use std::os::unix::fs::MetadataExt as _;
    let of = |path: &Path| -> String {
        return match std::fs::metadata(path) {
            Ok(found) => format!("{}:{}", found.dev(), found.ino()),
            Err(_unreadable) => String::from("nothing"),
        };
    };
    return format!("{} {}", of(registration), of(target));
}

/// What:
///  Write what the service will compare before it destroys anything:
///  the identity of the two directories the record is about,
///  and the worktree the sweep compared this one's ignored files with.
/// Why:
///  Both are things the service cannot read for itself:
///  the worktree the proof was made from is named by no command of the merge once it has returned,
///  and the record has no field for either without changing the shape a service of an earlier release parses.
///  They live beside the registration,
///  which the removal deletes,
///  so they need no cleanup of their own,
///  and a file already holding the same bytes is left untouched:
///  a sweep that runs twice writes one record and one of each of these.
///  A hand-over whose writes fail does not happen,
///  and the sweep keeps the worktree with the reason the caller answers,
///  so no record of this release's shape is ever queued without them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recordWhatTheServiceCompares(registration, target, comparedWith);
/// ```
pub(crate) fn record_what_the_service_compares(
    registration: &Path,
    target: &Path,
    compared_with: Option<&Path>,
) -> Result<(), String> {
    write_beside_the_registration(
        registration,
        IDENTITY_RECORD_NAME,
        format!("{}\n", identity_standing_there(target, registration)).as_bytes(),
    )?;
    if let Some(source) = compared_with {
        write_beside_the_registration(
            registration,
            COMPARED_WITH_RECORD_NAME,
            format!("{}\n", source.display()).as_bytes(),
        )?;
    }
    return Ok(());
}

/// What:
///  Write one record beside a registration,
///  unless it already holds those bytes.
/// Why:
///  A sweep that runs twice writes the same facts,
///  and a file left as it is costs no write and no change of anything the record is keyed by.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeBeside(registration, name, bytes);
/// ```
fn write_beside_the_registration(
    registration: &Path,
    name: &str,
    bytes: &[u8],
) -> Result<(), String> {
    let record: PathBuf = registration.join(name);
    if let Ok(standing) =
        read_record_here(record.as_path(), "a record of what the service compares")
        && standing == bytes
    {
        return Ok(());
    }
    return match write_private_file_unsynced(record.as_path(), bytes) {
        Ok(()) => Ok(()),
        Err(error) => Err(format!("{} cannot be written ({error})", record.display())),
    };
}

/// What:
///  The identity the hand-over wrote for this record,
///  or nothing when the record was written by an earlier release.
/// Why:
///   A record with no such file is one this release did not write,
///  and the removal falls back to the name the record was keyed by.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const recorded = identityRecorded(registration); // string | undefined
/// ```
pub(crate) fn identity_recorded(registration: &Path) -> Option<String> {
    let bytes: Vec<u8> = read_record_here(
        registration.join(IDENTITY_RECORD_NAME).as_path(),
        "the identity a removal recorded for this worktree",
    )
    .ok()?;
    let text: String = String::from_utf8(bytes).ok()?.trim_end().to_owned();
    return (!text.is_empty()).then_some(text);
}

/// What:
///  The worktree a landed sweep compared this one's ignored files with,
///  as the sweep wrote it beside the registration.
/// Why:
///   A registration that holds no such record is one no sweep handed over with this release's shape,
///  and then nothing proves the ignored files are safe to delete.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const source = comparedWithOf(registration); // Path | undefined
/// ```
pub(crate) fn compared_with_of(registration: &Path) -> Option<PathBuf> {
    let bytes: Vec<u8> = read_record_here(
        registration.join(COMPARED_WITH_RECORD_NAME).as_path(),
        "the worktree a removal compared this one's ignored files with",
    )
    .ok()?;
    let named: &[u8] = bytes.trim_ascii_end();
    if named.is_empty() {
        return None;
    }
    let path: PathBuf = PathBuf::from(OsStr::from_bytes(named));
    return path.is_absolute().then_some(path);
}

/// What:
///  Why the worktree standing at the record's path is not the one the record was written for,
///  or nothing when it is.
/// Why:
///  A record outlives the worktree it names,
///  and the README's own advice,
///  `git worktree remove` by hand,
///  makes the sequence ordinary:
///  a launcher that restarts a session adds a worktree at the path the earlier run used.
///  The identity the hand-over wrote is compared with the one standing there now,
///  and a record this release did not write falls back to the name it was keyed by,
///  which the release before this one computed from the same path and registration.
///  A record that matches neither proves nothing about the worktree in front of it,
///  and the worktree then stays and is named.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const why = whyThisIsAnotherWorktree(item, target, registration); // string | undefined
/// ```
pub(crate) fn why_this_is_another_worktree(
    item: &WorkItem,
    target: &Path,
    registration: &Path,
) -> Option<String> {
    if let Some(recorded) = identity_recorded(registration) {
        let standing: String = identity_standing_there(target, registration);
        return (standing != recorded).then(|| {
            return format!(
                "the worktree the record was written for is gone, and another one stands at {} now; it was left exactly as it is",
                target.display()
            );
        });
    }
    if removal_named_by(target, registration) == item.oid {
        return None;
    }
    return Some(format!(
        "nothing proves the worktree standing at {} is the one this record was written for: the registration standing there carries no identity, and the name the record was keyed by does not match it, so it was left exactly as it is",
        target.display()
    ));
}

/// The bytes of a path,
/// for a control that checks the record names the same place twice.
pub fn path_bytes(path: &Path) -> Vec<u8> {
    return path.as_os_str().as_bytes().to_vec();
}
