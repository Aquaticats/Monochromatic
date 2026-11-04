//! What:
//!  The claim chain: how one reclaimer at a time is chosen to remove a stale index lock, using only files placed whole beside the
//!  lock and the judgement of whether their makers are gone. Every reclaim goes through it, on every file system.
//! Why:
//!  A removal by name removes whatever the name holds by then, so two reclaimers of one stale file must not both go on to unlink
//!  it (the second would remove the live lock a landing placed after the first). A `flock` on the stale file would do where the
//!  file system gives one, but a reclaimer on that path and one on the claim path would not see each other, and a lock manager
//!  can answer one call and refuse the next; one mechanism for all reclaimers closes that. The exclusion is a numbered chain of
//!  claim files named from the stale lock's marker. A reclaimer places its claim at the lowest slot whose claims below it are
//!  all made by gone processes, and holds only when it finds every slot below its own still there and gone. A dead claimer's
//!  claim is never removed while the stale file stands at its name, because removing a claim by name would be the same race one
//!  level down; claims are added, a reclaimer removes its own live claim only when it backs off, and every claim is removed
//!  after the stale file left its name, when nothing destructive is left to guard
//!  (`doc/planning/cli-git-one-reclaimer-per-index.md`, "The claim chain"). A claim whose maker is on another machine is neither
//!  passed over nor waited for as a running one: it stays and the person is told.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! takeClaim(platform, lock, staleFile, marked) // 'held' | 'busy' | { foreign: { file, marked } } | { unavailable: error }
//! ```

use crate::service_index_marker::{Marked, holder_is_elsewhere, holder_is_gone, marker_text, parse_marker};
use crate::service_index_placement::{Platform, place_marked};
use crate::service_index_staging::stage_marker;
use std::io::Read;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

/// What follows the lock's file name in the name of a claim file.
pub const CLAIM_INFIX: &str = ".cli-reclaim-";

/// The longest claim this reader looks at, as long as the longest marker it reads from a lock.
const CLAIM_READ_LIMIT: u64 = 512;

/// The mode of a claim file: readable by every account that can reach the repository's directory, since a reclaimer that cannot
///  read a claim takes its maker to run.
const CLAIM_MODE: u32 = 0o644;

/// What taking the claim came to.
pub enum Claim {
    /// This reclaimer holds the chain until the value is cleared or dropped.
    Held(ClaimHold),
    /// A claimer that runs (or cannot be judged) is ahead, or a claim could not be read: not this reclaimer's turn.
    Busy,
    /// A claim in the way was made by a service on another machine: it is not proven gone, so it stays and is named.
    Foreign { file: PathBuf, marked: Marked },
    /// A claim could not be placed or this process cannot name itself: nothing is removed, and the caller says so.
    Unavailable(std::io::Error),
}

/// What a claim file at a slot is.
#[derive(Debug, Eq, PartialEq)]
enum Slot {
    /// No such file.
    Free,
    /// A claim whose maker is gone, or a file that holds no whole marker (a power loss left it, or a stranger placed it; a
    ///  running claimer's file is placed whole): passed over, never removed while the stale file stands.
    Dead,
    /// A claim whose maker runs or cannot be judged, or a file that could not be read.
    Live,
    /// A claim whose maker is on another machine, or on one this machine cannot be compared with.
    Elsewhere(Marked),
}

/// The claim a reclaimer holds: slots zero to its own are the chain it removes when the stale file has left its name.
pub struct ClaimHold {
    lock: PathBuf,
    key: String,
    own: usize,
    cleared: bool,
}

impl ClaimHold {
    /// Removes the claims at slots zero to this one: called once the stale file no longer stands at its name, when no
    ///  reclaimer's removal needs guarding any more. A claim that cannot be removed is reported and left.
    pub fn clear_all(mut self) {
        self.cleared = true;
        for slot in 0..=self.own {
            remove_claim(claim_path(&self.lock, self.key.as_str(), slot).as_path());
        }
    }
}

impl Drop for ClaimHold {
    /// A reclaimer that leaves with the stale file still at its name removes only its own claim, so no claim below it is
    ///  taken from a reclaimer that reads it as gone.
    fn drop(&mut self) {
        if !self.cleared {
            remove_claim(claim_path(&self.lock, self.key.as_str(), self.own).as_path());
        }
    }
}

/// The name of the claim at `slot` for the stale file with the identity `key`, beside `lock`.
pub fn claim_path(lock: &Path, key: &str, slot: usize) -> PathBuf {
    let mut name = lock.as_os_str().to_os_string();
    name.push(format!("{CLAIM_INFIX}{key}-{slot}"));
    return PathBuf::from(name);
}

/// What:
///  The identity of the stale lock that names its claims: the batch and process its marker names, from the marker's own text.
/// Why:
///   Every reclaimer of one stale lock must derive the same name, whichever way it reached the file: the device number of one
///  remote tree can differ between two mounts of it, but the marker is the file's own content. The process the marker names is
///  gone, and a gone process places no further lock with that marker, so no later lock carries the key and a claim left under
///  it is inert. The birth identity is written as it is, with the bytes a file name cannot hold escaped, so that two different identities never share a name. A marker
///  that names no process (an earlier release's) falls back to the device and inode of the open file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${seq.toString(16)}-${pid.toString(16)}-${escape(birth)}`
/// ```
pub fn claim_key(stale: &std::fs::File, marked: &Marked) -> std::io::Result<String> {
    return match marked.holder.as_ref() {
        Some((pid, birth)) => {
            // Only the bytes a file name cannot hold (`/`, NUL) and the escape itself are written as `%` and two hexadecimal digits, so two
            //  different identities never share a name and the name stays short.
            let escaped: String = birth.bytes().map(|byte| { return if matches!(byte, b'/' | 0 | b'%') { format!("%{byte:02x}") } else { String::from(char::from(byte)) }; }).collect();
            Ok(format!("{:x}-{pid:x}-{escaped}", marked.seq))
        }
        None => {
            let identity = stale.metadata()?;
            Ok(format!("{:x}-n-{:x}-{:x}", marked.seq, identity.dev(), identity.ino()))
        }
    };
}

fn remove_claim(path: &Path) {
    match std::fs::remove_file(path) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => crate::service_error!("the claim file {} of a reclaim cannot be removed ({error}); it is inert, remove it by hand", path.display()),
    }
}

/// What the file at `path` is as a claim: absent, a gone maker's, or a running (or unjudgeable, or unreadable) maker's.
fn look_at(path: &Path) -> Slot {
    // No link is followed and no pipe waited on, so a stranger's file of this name cannot hang the reclaimer.
    let opened = std::fs::OpenOptions::new().read(true).custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC).open(path);
    let file = match opened {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Slot::Free,
        Err(error) => {
            crate::service_error!("the claim file {} of a reclaim cannot be opened ({error}), so its maker is taken to run", path.display());
            return Slot::Live;
        }
    };
    let mut text: String = String::new();
    match file.take(CLAIM_READ_LIMIT).read_to_string(&mut text) {
        Ok(_) => {}
        // Bytes that are not text are no marker: the file is a stranger's.
        Err(error) if error.kind() == std::io::ErrorKind::InvalidData => return Slot::Dead,
        Err(error) => {
            crate::service_error!("the claim file {} of a reclaim cannot be read ({error}), so its maker is taken to run", path.display());
            return Slot::Live;
        }
    }
    return match parse_marker(text.as_str()) {
        Some(marked) if holder_is_gone(marked.holder.as_ref()) => Slot::Dead,
        Some(marked) if holder_is_elsewhere(marked.holder.as_ref()) => Slot::Elsewhere(marked),
        Some(_) => Slot::Live,
        None => Slot::Dead,
    };
}

/// The text of the claim file at `path`, when it can be read as text.
fn text_of(path: &Path) -> Option<String> {
    let file = std::fs::OpenOptions::new().read(true).custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC).open(path).ok()?;
    let mut text: String = String::new();
    file.take(CLAIM_READ_LIMIT).read_to_string(&mut text).ok()?;
    return Some(text);
}

/// A number no other attempt has: 64 bits of the system's random source.
fn attempt_token() -> std::io::Result<u64> {
    let uuid: String = crate::random_id::random_uuid().map_err(|error| { return std::io::Error::other(error.0); })?;
    let digits: String = uuid.chars().filter(char::is_ascii_hexdigit).take(16).collect();
    return u64::from_str_radix(digits.as_str(), 16).map_err(std::io::Error::other);
}

/// Whether every slot below `slot` still holds a gone maker's claim.
fn all_below_are_dead(lock: &Path, key: &str, slot: usize) -> bool {
    return (0..slot).all(|below| { return look_at(claim_path(lock, key, below).as_path()) == Slot::Dead; });
}

/// What:
///  Takes the claim for the stale file `stale`, which the caller has open and judged stale, at the lowest slot not taken by a
///  claimer that runs; `Busy` when a claimer that runs is ahead.
/// Why:
///   Two reclaimers of one stale file must not both unlink its name. A claim is placed whole (`place_marked`), so a placed
///  claim is never half written; the maker's marker is judged as a lock holder's is, so a dead claimer never blocks anyone.
///  After the claim is placed, every slot below is read again, because what a reclaimer read before placing may be arbitrarily
///  old: a claim there that is missing or alive means another reclaimer holds or may hold, and this one backs off.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// let slot = 0
/// while (!(await place(slot))) { if (await look(slot) !== 'dead') return 'busy'; slot++ }
/// return allBelowDead(slot) ? held : (remove(slot), 'busy')
/// ```
pub fn take_claim(platform: &Platform, lock: &Path, stale: &std::fs::File, marked: &Marked) -> Claim {
    // Lock kept: two reclaimers of one stale lock must not both unlink its name, and no system call unlinks a name only if it
    //  still names a given file. The claim is a file placed whole beside the lock, named from the stale lock's marker; the hold
    //  is a few system calls long, is one file per reclaimer (never the repository or the directory), and a dead reclaimer's
    //  claim is passed over, never waited for, because its maker's marker says it is gone.
    let key: String = match claim_key(stale, marked) {
        Ok(key) => key,
        Err(error) => return Claim::Unavailable(error),
    };
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
            Err(_) => text_of(path.as_path()).is_some_and(|text| { return text == marker; }),
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
