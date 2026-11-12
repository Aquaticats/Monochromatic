//! What:
//!  The private name a take gives a file it moves aside:
//!  a key for the file's own name,
//!  a key for the merge that took it,
//!  and the process and the moment that made it;
//!  every name a directory holds for one path,
//!  and the keys of the merges whose records are still there.
//! Why:
//!  A take moves a file beside its path before the merge writes there,
//! and a command that stopped between the two leaves the
//!  file under that name,
//! so the next command puts it back.
//!  The name must say which path it belongs to and which merge took it,
//! because a
//!  private name no record claims is what a merge that already ended left behind,
//! and putting it back would bring back a file the
//!  person deleted
//!  (a cross-vendor review of the second round,
//! its third finding).
//!  A key is the FNV-1a hash of the text in 64 bits,
//! which every release of this program computes the
//!  same way,
//! so a private name one release left behind is found by the next,
//! and a file name at the file system's
//!  255-byte component limit still has a private name.
//! An upper-case spelling of one name is a different key,
//! as it is a
//!  different file.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const aside = asideName(location, transactionId); const live = liveMergeKeys(gitDir); asidesOf(location).filter((one) => one.merge && live.includes(one.merge));
//! ```

/// The records a command leaves behind.
use crate::merge_isolation_journal::list_merge_intents;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// The start of the private name a file is set aside under while a merge's content takes its place.
///  It sits in the file's own directory,
/// so the take is one rename on whatever file system the worktree is on,
/// and it
///  ends in no name Git reads.
pub(crate) const ASIDE_PREFIX: &str = ".cli-git-recovery";

/// The start of the private name the bytes of a merge's own write wait under before they take their path.
///  It sits in the file's own directory,
/// so the placing is one rename on whatever file system the worktree is on,
///  and it ends in
/// no name Git reads.
///  It carries the key of the merge that wrote it,
/// so a command that finds one a killed command left can say whose it
///  is and remove only the names nothing is waiting on.
pub(crate) const CARRY_PREFIX: &str = ".cli-git-carry";

/// Makes two names set aside by one process differ.
static ASIDE_COUNT: AtomicU64 = AtomicU64::new(0);

/// The base of the FNV-1a hash,
/// in 64 bits.
const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;

/// The prime of the FNV-1a hash,
/// in 64 bits.
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// The hex digits one key is written in,
/// which is 64 bits at four bits a digit.
const KEY_DIGITS: usize = 16;

/// The text of one file name.
fn name_of(found: &std::ffi::OsStr) -> String {
    return found.to_string_lossy().into_owned();
}

/// The text of a file's name,
/// which a private name of a take carries as a key.
pub(crate) fn name_text(location: &Path) -> String {
    return location.file_name().map(name_of).unwrap_or_default();
}

/// What:
///  A short,
///  stable key for one text,
///  which a private name of a take carries.
/// Why:
///   A file name may fill the file system's 255-byte component limit,
/// and a private name that embedded it could not exist,
///  so a valid tracked file made a saved merge unfinishable and its record then stopped every command
///  (a cross-vendor review of the second round).
/// A merge's identifier and the name of the file it took are both carried this way,
/// so neither can make
///  a private name too long.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function nameKey(text: string): string; // FNV-1a 64 of the UTF-8 bytes, 16 hex digits
/// ```
pub(crate) fn name_key(text: &str) -> String {
    let mut hash: u64 = FNV_OFFSET_BASIS;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    return format!("{hash:0width$x}", width = KEY_DIGITS);
}

/// What:
///  A private name beside `location`,
///  naming the file it holds and the merge that took it,
///  that no other process uses.
/// Why:
///   A kill between the rename of a take and the write that follows leaves the file under this name,
/// so the name
///  must say which path it belongs to:
/// the next recovery finds it there and puts it back
///  (`doc/planning/cli-git-merge-and-repair-route-findings-on-main.md`,
/// finding 4's take,
/// round two).
///  The merge's own key is what tells a name a live merge may put back from one a merge that ended left behind.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${dirname}/.cli-git-recovery-${nameKey(basename)}-${nameKey(transactionId)}-${pid}-${count}-${nanos}`;
/// ```
pub(crate) fn aside_name(location: &Path, transaction_id: &str) -> PathBuf {
    let count: u64 = ASIDE_COUNT.fetch_add(1, Ordering::Relaxed);
    let nanos: u128 = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos();
    return location.with_file_name(format!(
        "{ASIDE_PREFIX}-{}-{}-{}-{count}-{nanos:x}",
        name_key(name_text(location).as_str()),
        name_key(transaction_id),
        std::process::id()
    ));
}

/// One file a take moved aside.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Aside = { path: string; merge?: string };
/// ```
pub(crate) struct Aside {
    /// Where the file waits.
    pub(crate) path: PathBuf,
    /// The key of the merge that took it,
    /// or nothing for a name an earlier release wrote,
    /// which named no record at all.
    pub(crate) merge: Option<String>,
}

/// Whether one byte is one of the hex digits a key or a ran number is written in.
fn is_a_hex_digit(byte: u8) -> bool {
    return byte.is_ascii_hexdigit();
}

/// Whether one text is a key this file writes:
/// as many hex digits as one key has.
fn is_a_key(text: &str) -> bool {
    return text.len() == KEY_DIGITS && text.bytes().all(is_a_hex_digit);
}

/// Whether one text is the number of a moment,
/// which a private name ends in.
fn is_a_ran_number(text: &str) -> bool {
    return !text.is_empty() && text.bytes().all(is_a_hex_digit);
}

/// What:
///  The merge a private name of `location` was made by,
///  or nothing when the name is not a private name of that path.
/// Why:
///   The parts after the keys are the process,
/// the count and the moment the name was made,
/// and a name an earlier release wrote
///  carries no merge key at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theMergeOfAName(name: string, location: string): string | null | undefined; // null: no record named
/// ```
fn the_merge_of_a_name(name: &str, location: &Path) -> Option<Option<String>> {
    return the_merge_of_a_private_name(ASIDE_PREFIX, name, location);
}

/// What:
///  The merge a private name of `location` under `prefix` was made by,
///  or nothing when the name is not one of that kind for that path.
/// Why:
///   The two kinds of private name are one shape:
/// the file's own key,
/// then the merge's key,
///  then the
/// process,
/// the count and the moment,
/// so one reading serves both and the names cannot drift apart.
///  A name an earlier
/// release wrote carries no merge key and is answered as such.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theMergeOfAPrivateName(prefix: string, name: string, location: string): string | null | undefined;
/// ```
fn the_merge_of_a_private_name(kind: &str, name: &str, location: &Path) -> Option<Option<String>> {
    let start: String = format!("{kind}-{}-", name_key(name_text(location).as_str()));
    let rest: &str = name.strip_prefix(start.as_str())?;
    let parts: Vec<&str> = rest.split('-').collect();
    return match parts.as_slice() {
        // A name an earlier release wrote named no record, so nothing claims it.
        [_process, _count, nanos] if is_a_ran_number(nanos) => Some(None),
        [merge, _process, _count, nanos] if is_a_key(merge) && is_a_ran_number(nanos) => Some(Some(String::from(*merge))),
        _ => None,
    };
}

/// What:
///  Every private name beside `location` under which the bytes of a merge's own write were waiting,
///  where the
///  merge that wrote it is not one whose record still governs the worktree,
///  or is the one this command is finishing.
/// Why:
///   A kill between writing those bytes and renaming them into place leaves the name behind,
/// and it is the one
///  thing a finished merge leaves in the worktree:
/// the file it held is made again by the next carry,
/// and the content is in
///  the object store either way,
/// so the name is a temporary nothing is waiting on.
///  A name whose merge still has a
/// record is left where it is,
/// because that merge may be one the record names and no command can
///  know whether it is running without reading the record:
/// only the merge whose record this command is finishing is removed,
///  and the ones whose records are gone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function staleCarryNamesOf(location, live: string[], finished: string): string[]; // carry names whose merge is finished or unnamed
/// ```
pub(crate) fn stale_carry_names_of(location: &Path, live: &[String], finished: &str) -> Vec<PathBuf> {
    let Some(directory) = location.parent() else {
        return Vec::new();
    };
    let Ok(listing) = std::fs::read_dir(directory) else {
        return Vec::new();
    };
    let mut found: Vec<PathBuf> = Vec::new();
    for entry in listing.flatten() {
        let name: String = name_of(entry.file_name().as_os_str());
        // A name of the file the merge writes, or of a path inside a directory it replaces, is still the file's own:
        //  `name_text` is the base name it was written for.
        let Some(Some(merge)) = the_merge_of_a_private_name(CARRY_PREFIX, name.as_str(), location) else { continue };
        // The private name carries a key, and the caller names the merge it is finishing by its identifier.
        if merge == name_key(finished) || !live.contains(&merge) {
            found.push(entry.path());
        }
    }
    found.sort();
    return found;
}

/// The order two files a take left are listed in,
/// which is the order of their private names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const byTheirPaths = (left: Aside, right: Aside) => left.path.toString().localeCompare(right.path.toString());
/// ```
fn by_their_paths(left: &Aside, right: &Aside) -> std::cmp::Ordering {
    return left.path.cmp(&right.path);
}

/// What:
///  Every file in `location`'s directory that a take of that path left.
/// Why:
///   A recovery that stopped for any reason can leave one,
/// and the next recovery must find it before it judges the path,
///  because the path is empty while the file waits beside it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (await readdir(dirname(location))).filter((name) => theMergeOfAName(name, location) !== undefined).map((name) => ({ path: join(dirname(location), name), merge: theMergeOfAName(name, location) }));
/// ```
pub(crate) fn asides_of(location: &Path) -> Vec<Aside> {
    let Some(directory) = location.parent() else {
        return Vec::new();
    };
    let Ok(listing) = std::fs::read_dir(directory) else {
        return Vec::new();
    };
    let mut found: Vec<Aside> = Vec::new();
    for entry in listing.flatten() {
        let name: String = name_of(entry.file_name().as_os_str());
        if let Some(merge) = the_merge_of_a_name(name.as_str(), location) {
            found.push(Aside { path: entry.path(), merge });
        }
    }
    found.sort_by(by_their_paths);
    return found;
}

/// What:
///  The key a merge's record carries,
///  which the private name of a copy the merge took holds.
/// Why:
///   A recovery judges a copy by the record its name names,
/// and the key has to be derived in one place
///  or the two readings name different merges.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const mergeKeyOfARecord = (record: string) => nameKey(stem(record));
/// ```
pub(crate) fn merge_key_of_a_record(record: &Path) -> String {
    return name_key(record.file_stem().map(name_of).unwrap_or_default().as_str());
}

/// What:
///  The keys of the merges whose records are still there,
///  which are the only ones that may put a file back.
/// Why:
///   A record that is gone names no checkout state,
/// so a private name it left belongs to a merge that already ended;
/// a record whose
///  read fails proves nothing either,
/// and then no merge may put anything back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function liveMergeKeys(gitDir: string): string[]; // listMergeIntents(gitDir).map((record) => nameKey(stem(record)))
/// ```
pub(crate) fn live_merge_keys(git_dir: &Path) -> Vec<String> {
    let records: Vec<PathBuf> = match list_merge_intents(git_dir) {
        Ok(found) => found,
        Err(_unread) => return Vec::new(),
    };
    let mut keys: Vec<String> = Vec::with_capacity(records.len());
    for record in records {
        keys.push(merge_key_of_a_record(record.as_path()));
    }
    return keys;
}
