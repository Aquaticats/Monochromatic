//! What:
//!  The claim chain:
//! how one reclaimer at a time is chosen to remove a stale index lock,
//! using only files placed whole beside the
//!  lock and the judgement of whether their makers are gone.
//! Every reclaim goes through it,
//! on every file system.
//! Why:
//!  A removal by name removes whatever the name holds by then,
//! so two reclaimers of one stale file must not both go on to unlink
//!  it (the second would remove the live lock a landing placed after the first).
//! A `flock` on the stale file would do where the
//!  file system gives one,
//! but a reclaimer on that path and one on the claim path would not see each other,
//! and a lock manager
//!  can answer one call and refuse the next;
//! one mechanism for all reclaimers closes that.
//! The exclusion is a numbered chain of
//!  claim files named from the stale lock's marker.
//! A reclaimer places its claim at the lowest slot whose claims below it are
//!  all made by gone processes,
//! and holds only when it finds every slot below its own still there and gone.
//! A dead claimer's
//!  claim is never removed while the stale file stands at its name,
//! because removing a claim by name would be the same race one
//!  level down;
//! claims are added,
//! a reclaimer removes its own live claim only when it backs off,
//! and every claim is removed
//!  after the stale file left its name,
//! when nothing destructive is left to guard
//!  (`doc/planning/cli-git-one-reclaimer-per-index.md`,
//! "The claim chain").
//! A claim whose maker is on another machine is neither
//!  passed over nor waited for as a running one:
//! it stays and the person is told.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! takeClaim(platform, lock, staleFile, marked) // 'held' | 'busy' | { foreign: { file, marked } } | { unavailable: error }
//! ```

use crate::landing_state_carried_by_its_task::{LandingState, on_the_landing_state};
use crate::service_index_marker::{Marked, holder_is_elsewhere, holder_is_gone, holder_is_running, marker_text, parse_marker};
use crate::service_index_placement::{Platform, place_marked};
use crate::service_index_staging::stage_marker;
use std::io::Read;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

/// What every claim file's name starts with (after the lock's directory).
pub const CLAIM_INFIX: &str = "cli-reclaim-";

/// The longest claim this reader looks at,
/// as long as the longest marker it reads from a lock.
const CLAIM_READ_LIMIT: u64 = 512;

/// The mode of a claim file:
/// readable by every account that can reach the repository's directory,
/// since a reclaimer that cannot
///  read a claim takes its maker to run.
const CLAIM_MODE: u32 = 0o644;

/// What taking the claim came to.
pub enum Claim {
    /// This reclaimer holds the chain until the value is cleared or dropped.
    Held(ClaimHold),
    /// A claimer that runs (or cannot be judged) is ahead,
    /// or a claim could not be read:
    /// not this reclaimer's turn.
    Busy,
    /// A claim in the way was made by a service on another machine:
    /// it is not proven gone,
    /// so it stays and is named.
    Foreign { file: PathBuf, marked: Marked },
    /// A claim in the way cannot be judged (its maker is in another PID namespace,
    /// or it cannot be read):
    /// it stays and is named.
    Unjudged { file: PathBuf, marked: Option<Marked> },
    /// A claim could not be placed or this process cannot name itself:
    /// nothing is removed,
    /// and the caller says so.
    Unavailable(std::io::Error),
}

/// What a claim file at a slot is.
#[derive(Debug, Eq, PartialEq)]
enum Slot {
    /// No such file.
    Free,
    /// A claim whose maker is gone,
    /// or a file that holds no whole marker (a power loss left it,
    /// or a stranger placed it;
    /// a
    ///  running claimer's file is placed whole):
    /// passed over,
    /// never removed while the stale file stands.
    Dead,
    /// A claim whose maker is proven to run.
    Live,
    /// A claim whose maker cannot be judged here (another PID namespace,
    /// a `/proc` that does not agree),
    /// or a claim that cannot be
    ///  read at all (then without a marker):
    /// neither gone nor proven running,
    /// and not a contention that passes.
    Unjudged(Option<Marked>),
    /// A claim whose maker is on another machine,
    /// or on one this machine cannot be compared with.
    Elsewhere(Marked),
}

/// The claim a reclaimer holds:
/// slots zero to its own are the chain it removes when the stale file has left its name.
pub struct ClaimHold {
    lock: PathBuf,
    key: String,
    own: usize,
    cleared: bool,
}

impl ClaimHold {
    /// Removes the claims at slots zero to this one:
    /// called once the stale file no longer stands at its name,
    /// when no
    ///  reclaimer's removal needs guarding any more.
    /// A claim that cannot be removed is reported and left.
    pub fn clear_all(mut self) {
        self.cleared = true;
        for slot in 0..=self.own {
            remove_claim(claim_path(&self.lock, self.key.as_str(), slot).as_path());
        }
    }
}

impl Drop for ClaimHold {
    /// A reclaimer that leaves with the stale file still at its name removes only its own claim,
    /// so no claim below it is
    ///  taken from a reclaimer that reads it as gone.
    fn drop(&mut self) {
        if !self.cleared {
            remove_claim(claim_path(&self.lock, self.key.as_str(), self.own).as_path());
        }
    }
}

/// What:
///  A byte of a process identity as it is written in a claim's name:
/// the bytes a file name cannot hold and the escape itself as
///  `%` and two hexadecimal digits,
/// every other byte as the character it is.
/// Why:
///  Two different identities never share a name,
/// and the name stays short.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const escapedByte = (byte: number) => [0x2f, 0, 0x25].includes(byte) ? '%' + byte.toString(16).padStart(2, '0') : String.fromCharCode(byte);
/// ```
fn escaped_byte(byte: u8) -> String {
    return if matches!(byte, b'/' | 0 | b'%') { format!("%{}", crate::hexadecimal_of_bytes::hexadecimal_of_byte(byte)) } else { String::from(char::from(byte)) };
}

/// What:
///  Remembers a claim file that may have been made,
/// in the list of the landing that made it or of the job that stands in for one,
/// for the next attempt of that landing to remove.
/// Why:
///  A claim whose placement is not known would block every reclaimer if it stayed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// abandoned.push([path, text]);
/// ```
fn remember_abandoned(claims: &mut Vec<(PathBuf, String)>, path: PathBuf, text: String) {
    claims.push((path, text));
}

/// What:
///  The name of the claim at `slot` for the stale lock with the identity `key`,
/// in the lock's directory.
/// Why:
///   A claim name of the lock's own name with the whole identity appended would pass the limit of a file name (255 bytes) beside
///  a lock with a long name,
/// and no claim could be placed.
/// The name is a fixed-length digest of the lock's name and the key (the
///  first sixteen bytes of SHA-256,
/// which the index writer already carries),
/// so two different locks of one directory and two
///  different stale files of one lock never share a name,
/// and every reclaimer derives it alike.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// join(dirname(lock), `cli-reclaim-${sha256(basename(lock) + '\0' + key).slice(0, 16).toString('hex')}-${slot}`)
/// ```
pub fn claim_path(lock: &Path, key: &str, slot: usize) -> PathBuf {
    use std::os::unix::ffi::OsStrExt;
    let mut input: Vec<u8> = match lock.file_name() {
        Some(name) => name.as_bytes().to_vec(),
        None => Vec::new(),
    };
    input.push(0);
    input.extend_from_slice(key.as_bytes());
    let digest: [u8; 32] = crate::subvolume_sha::sha256(input.as_slice());
    let hex: String = crate::hexadecimal_of_bytes::hexadecimal_of_bytes(&digest[..CLAIM_DIGEST_BYTES]);
    return lock.with_file_name(format!("{CLAIM_INFIX}{hex}-{slot}"));
}

/// How many bytes of the digest name a claim:
/// 128 bits,
/// far past any chance of two stale locks of one directory sharing a name.
const CLAIM_DIGEST_BYTES: usize = 16;

/// What:
///  The identity of the stale lock that names its claims:
/// the batch and process its marker names,
/// from the marker's own text.
/// Why:
///   Every reclaimer of one stale lock must derive the same name,
/// whichever way it reached the file:
/// the device number of one
///  remote tree can differ between two mounts of it,
/// but the marker is the file's own content.
/// The process the marker names is
///  gone,
/// and a gone process places no further lock with that marker,
/// so no later lock carries the key and a claim left under
///  it is inert.
/// The birth identity is written as it is,
/// with the bytes a file name cannot hold escaped,
/// so that two different
///  identities never share a name.
/// A marker that names no process (an earlier release's) is keyed on its batch number alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${seq.toString(16)}-${pid.toString(16)}-${escape(birth)}`
/// ```
pub fn claim_key(marked: &Marked) -> String {
    return match marked.holder.as_ref() {
        Some((pid, birth)) => {
            // Only the bytes a file name cannot hold (`/`, NUL) and the escape itself are written as `%` and two hexadecimal digits, so two
            //  different identities never share a name and the name stays short.
            let escaped: String = birth.bytes().map(escaped_byte).collect();
            format!("{:x}-{pid:x}-{escaped}", marked.seq)
        }
        // A marker that names no process is an earlier release's; its key comes from the marker alone too, because the device number
        //  of one remote tree differs between two mounts of it. A repeated batch number of such a marker would only meet a gone
        //  claim at a low slot, which is passed over.
        None => format!("{:x}-n", marked.seq),
    };
}

fn remove_claim(path: &Path) {
    match std::fs::remove_file(path) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => crate::service_error!("the claim file {} of a reclaim cannot be removed ({error}); it is inert, remove it by hand", path.display()),
    }
}

/// What the file at `path` is as a claim:
/// absent,
/// a gone maker's,
/// a running maker's,
/// or one that cannot be judged.
fn look_at(path: &Path) -> Slot {
    // No link is followed and no pipe waited on, so a stranger's file of this name cannot hang the reclaimer.
    let opened = std::fs::OpenOptions::new().read(true).custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC).open(path);
    let file = match opened {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Slot::Free,
        Err(error) => {
            crate::service_error!("the claim file {} of a reclaim cannot be opened ({error}), so its maker is not judged", path.display());
            return Slot::Unjudged(None);
        }
    };
    let mut text: String = String::new();
    match file.take(CLAIM_READ_LIMIT).read_to_string(&mut text) {
        Ok(_) => {}
        // Bytes that are not text are no marker: the file is a stranger's.
        Err(error) if error.kind() == std::io::ErrorKind::InvalidData => return Slot::Dead,
        Err(error) => {
            crate::service_error!("the claim file {} of a reclaim cannot be read ({error}), so its maker is not judged", path.display());
            return Slot::Unjudged(None);
        }
    }
    return match parse_marker(text.as_str()) {
        Some(marked) if holder_is_gone(marked.holder.as_ref()) => Slot::Dead,
        Some(marked) if holder_is_elsewhere(marked.holder.as_ref()) => Slot::Elsewhere(marked),
        Some(marked) if holder_is_running(marked.holder.as_ref()) => Slot::Live,
        Some(marked) => Slot::Unjudged(Some(marked)),
        None => Slot::Dead,
    };
}

/// What reading the claim file at `path` as text gave:
/// its text,
/// nothing because there is no such file,
/// or an error because it
///  could not be read (then whether it is this attempt's claim is not known).
fn text_of(path: &Path) -> std::io::Result<Option<String>> {
    let file = match std::fs::OpenOptions::new().read(true).custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC).open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    let mut text: String = String::new();
    return match file.take(CLAIM_READ_LIMIT).read_to_string(&mut text) {
        Ok(_) => Ok(Some(text)),
        // A file that is not text is not this attempt's claim, which is text.
        Err(error) if error.kind() == std::io::ErrorKind::InvalidData => Ok(None),
        Err(error) => Err(error),
    };
}

/// Keeps `claims` as the landing's abandoned ones.
fn keep_the_abandoned_claims(state: &LandingState, claims: Vec<(PathBuf, String)>) {
    *state.abandoned_claims.borrow_mut() = claims;
}

/// Takes the landing's abandoned claims out of its state.
fn take_abandoned_claims(state: &LandingState, (): ()) -> Vec<(PathBuf, String)> {
    return state.abandoned_claims.take();
}

/// What:
///  The claims this landing placed whose placement could not be read back,
/// taken out of its state.
/// Why:
///   A job of the blocking pool cannot read the landing's state,
/// so the task that asks it for a reclaim carries these into the job and keeps them when it answers
/// (`service_index_lock.rs`,
/// `TheLockWaitsMemory`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const abandoned = abandonedHere()
/// ```
pub(crate) fn abandoned_here() -> Vec<(PathBuf, String)> {
    return on_the_landing_state((), take_abandoned_claims);
}

/// Keeps `claims` as the ones this landing may still have made.
pub(crate) fn keep_abandoned(claims: Vec<(PathBuf, String)>) {
    on_the_landing_state(claims, keep_the_abandoned_claims);
}

/// Removes the claims this landing abandoned
/// (the state `abandoned_claims` of `landing_state_carried_by_its_task.rs`,
/// or the list a job of the blocking pool carries for it:
/// claims whose placement answered with an error and which could not be read back to be recognised,
/// with the marker text each was written from;
/// per landing,
/// so another landing's attempt in the same process is never taken for an abandoned one,
/// the process number being the same for both):
/// a file that still holds exactly the text the landing wrote is the landing's own claim,
///  and goes;
/// one that holds other text,
/// or is gone,
/// was never the thread's claim or is no more,
/// and is forgotten;
/// one that still
///  cannot be read stays on the list for the next attempt.
fn resolve_abandoned(claims: &mut Vec<(PathBuf, String)>) {
    let pending: Vec<(PathBuf, String)> = std::mem::take(claims);
    for (path, text) in pending {
        match text_of(path.as_path()) {
            Ok(Some(found)) if found == text => remove_claim(path.as_path()),
            Ok(_) => {}
            Err(_) => remember_abandoned(claims, path, text),
        }
    }
}

/// A number no other attempt has:
/// 64 bits of the system's random source.
fn attempt_token() -> std::io::Result<u64> {
    let uuid: String = match crate::random_id::random_uuid() {
        Ok(found) => found,
        Err(error) => return Err(std::io::Error::other(error.0)),
    };
    let digits: String = uuid.chars().filter(char::is_ascii_hexdigit).take(16).collect();
    return u64::from_str_radix(digits.as_str(), 16).map_err(std::io::Error::other);
}

/// Whether every slot below `slot` still holds a gone maker's claim.
fn all_below_are_dead(lock: &Path, key: &str, slot: usize) -> bool {
    for below in 0..slot {
        if look_at(claim_path(lock, key, below).as_path()) != Slot::Dead {
            return false;
        }
    }
    return true;
}

/// What:
///  Takes the claim for the stale lock whose marker is `marked`,
/// which the caller has judged stale,
/// at the lowest slot not taken by a
///  claimer that runs;
/// `Busy` when a claimer that runs is ahead.
/// Why:
///   Two reclaimers of one stale file must not both unlink its name.
/// A claim is placed whole (`place_marked`),
/// so a placed
///  claim is never half written;
/// the maker's marker is judged as a lock holder's is,
/// so a dead claimer never blocks anyone.
///  After the claim is placed,
/// every slot below is read again,
/// because what a reclaimer read before placing may be arbitrarily
///  old:
/// a claim there that is missing or alive means another reclaimer holds or may hold,
/// and this one backs off.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// let slot = 0
/// while (!(await place(slot))) { if (await look(slot) !== 'dead') return 'busy'; slot++ }
/// return allBelowDead(slot) ? held : (remove(slot), 'busy')
/// ```
pub fn take_claim(platform: &Platform, lock: &Path, marked: &Marked) -> Claim {
    let mut abandoned: Vec<(PathBuf, String)> = abandoned_here();
    let claim: Claim = take_claim_carrying(platform, lock, marked, &mut abandoned);
    keep_abandoned(abandoned);
    return claim;
}

/// `take_claim` with the landing's abandoned claims carried rather than read from its state,
/// for the reclaim of a job of the blocking pool.
pub(crate) fn take_claim_carrying(platform: &Platform, lock: &Path, marked: &Marked, abandoned: &mut Vec<(PathBuf, String)>) -> Claim {
    // Lock kept: two reclaimers of one stale lock must not both unlink its name, and no system call unlinks a name only if it
    //  still names a given file. The claim is a file placed whole beside the lock, named from the stale lock's marker; the hold
    //  is a few system calls long, is one file per reclaimer (never the repository or the directory), and a dead reclaimer's
    //  claim is passed over, never waited for, because its maker's marker says it is gone.
    let key: String = claim_key(marked);
    resolve_abandoned(abandoned);
    // The batch field of a claim's marker is a token of this attempt alone: a placement whose answer was lost (an error after a
    //  link or rename took effect, as a network file system can answer) is told from another attempt's claim by the whole text.
    let token: u64 = match attempt_token() {
        Ok(token) => token,
        Err(error) => return Claim::Unavailable(error),
    };
    let marker: String = match marker_text(token, platform.identity) {
        Ok(text) => text,
        Err(error) => return Claim::Unavailable(std::io::Error::other(format!("this process has no birth identity to name in a claim ({error})"))),
    };
    let (staged, open) = match stage_marker(lock, token, marker.as_str()) {
        Ok(made) => made,
        Err(error) => return Claim::Unavailable(error),
    };
    // A claim is read by reclaimers of other accounts; a restrictive umask must not make a running claimer's file unreadable
    //  to them (an unreadable claim reads as running) or a dead claimer's claim a permanent obstacle.
    if let Err(error) = open.set_permissions(std::fs::Permissions::from_mode(CLAIM_MODE)) {
        let _gone = std::fs::remove_file(staged.as_path());
        return Claim::Unavailable(error);
    }
    let mut slot: usize = 0;
    let outcome: Claim = loop {
        let path: PathBuf = claim_path(lock, key.as_str(), slot);
        let placed: std::io::Result<()> = place_marked(platform, staged.as_path(), path.as_path());
        // A placement whose answer is an error may still have taken effect (a network file system can lose the answer of a
        //  link or rename, or answer a retried one with "exists"): a file at the slot that holds this attempt's own marker
        //  is this attempt's claim, which it owns and removes.
        let ours: bool = match &placed {
            Ok(()) => true,
            Err(_) => {
                // A control reaches here to make the claim unreadable at this moment.
                crate::service_fault::reach("reclaim-claim-unread");
                match text_of(path.as_path()) {
                    Ok(found) => matches!(found, Some(text) if text == marker),
                    Err(error) => {
                        // Whether the placement took effect is not known: the attempt remembers the claim it may have made, and
                        //  the next attempt of this thread removes it if it is there, so a claim of a running process is not
                        //  left to block every reclaimer.
                        crate::service_error!("the claim file {} of a reclaim cannot be read back ({error}); the next attempt resolves it", path.display());
                        remember_abandoned(abandoned, path.clone(), marker.clone());
                        break Claim::Busy;
                    }
                }
            }
        };
        if ours {
            // A control reaches here to act as another reclaimer would between the placement and the look below.
            crate::service_fault::reach("reclaim-claim-placed");
            let hold = ClaimHold { lock: lock.to_path_buf(), key: key.clone(), own: slot, cleared: false };
            if all_below_are_dead(lock, key.as_str(), slot) {
                break Claim::Held(hold);
            }
            // The slots below changed under this reclaimer: another holds or may hold, so it backs off, and dropping the
            //  hold removes its own claim.
            drop(hold);
            break Claim::Busy;
        }
        match placed {
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => match look_at(path.as_path()) {
                Slot::Dead => slot += 1,
                Slot::Live | Slot::Free => break Claim::Busy,
                Slot::Elsewhere(theirs) => break Claim::Foreign { file: path, marked: theirs },
                Slot::Unjudged(theirs) => break Claim::Unjudged { file: path, marked: theirs },
            },
            // An interrupted call, or one that found no memory, is not a refusal of the file system: asked again by the wait.
            Err(error) if matches!(error.kind(), std::io::ErrorKind::Interrupted | std::io::ErrorKind::WouldBlock | std::io::ErrorKind::OutOfMemory) => break Claim::Busy,
            Err(error) => break Claim::Unavailable(error),
            Ok(()) => unreachable!("a placement that succeeded is the attempt's own claim"),
        }
    };
    // The staging file is consumed by a placement; a back-off or a refusal leaves it to remove.
    let _gone = std::fs::remove_file(staged.as_path());
    return outcome;
}
