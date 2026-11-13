//! What:
//!  What the wrapper's records of the processes it started say about a lock's holder:
//!  the identity one record names for a process number,
//!  the proof that a holder is gone,
//!  and the removal of the records that explain nothing.
//! Why:
//!  A number alone proves nothing,
//!  so the proof is a record of this repository naming that number with a birth identity,
//!  and a process table that has no process with that identity.
//!  An identity of another boot is gone,
//!  because a boot that ended ended every process in it;
//!  an identity of another PID namespace or another machine answers unknown,
//!  and nothing is ever cleared on an unknown.
//!  The spelling,
//! the parts and the judgement are the commit service's own
//!  (`service_index_identity.rs`),
//!  so one rule decides both a service's marked lock and a record the wrapper wrote.
//!  The writing side is `lock_holder_records.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const identity = recordedIdentity(commonDir, pid); const gone = provenGone(commonDir, pid); sweep(commonDir, pidsInUse);
//! ```

/// The record files and their lines.
use crate::lock_holder_records::{CHILD_LINE, SELF_LINE, folder_of};
/// The one judgement of a holder the commit service's marker also uses.
use crate::process_identity::process_birth_identity;
use crate::service_index_identity::{HolderState, holder_state};
/// Reading a small file without following a link and without waiting on a pipe.
use crate::plain_file::{read_plain_file, SMALL_FILE_LIMIT};
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{Path, PathBuf};

/// How many record files one lookup reads,
/// so a folder a broken command filled cannot hold a command.
const MOST_RECORDS: usize = 512;

/// The longest record one read takes.
const RECORD_LIMIT: u64 = SMALL_FILE_LIMIT;

/// What:
///  The holders one record file names:
///  their numbers and the identities it wrote for them.
/// Why:
///   A record that cannot be read names nobody,
///  and a line that is not one of the two kinds is dropped,
///  so a file a person edited decides nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// parseRecord(text) // [[pid, identity], ...]
/// ```
fn holders_of(text: &str) -> Vec<(i64, String)> {
    let mut holders: Vec<(i64, String)> = Vec::new();
    for line in text.lines().skip(1) {
        let rest: &str = match line.strip_prefix(SELF_LINE) {
            Some(found) => found,
            None => line.strip_prefix(CHILD_LINE).unwrap_or(""),
        };
        if rest.is_empty() {
            continue;
        }
        let mut parts = rest.splitn(2, ' ');
        let (Some(pid), Some(identity)) = (parts.next(), parts.next()) else {
            continue;
        };
        let (Ok(number), false) = (pid.parse::<i64>(), identity.is_empty()) else {
            continue;
        };
        holders.push((number, identity.to_owned()));
    }
    return holders;
}

/// What:
///  Every holder every record of the repository names,
///  each with the file that names it,
///  in file-name order and at most `MOST_RECORDS` files.
/// Why:
///   A lookup and a sweep ask the same question of the same files,
///  so they read them once through one function.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const name of readdirSync(folder).sort()) { holders.push(...parseRecord(name)) }
/// ```
fn every_holder(common_dir: &Path) -> Vec<(PathBuf, Vec<(i64, String)>)> {
    let folder: PathBuf = folder_of(common_dir);
    let Ok(entries) = std::fs::read_dir(folder.as_path()) else {
        return Vec::new();
    };
    let mut names: Vec<std::ffi::OsString> = Vec::new();
    for entry in entries.flatten() {
        let kind: Option<std::fs::FileType> = entry.file_type().ok();
        if matches!(kind, Some(found) if found.is_file()) {
            names.push(entry.file_name());
        }
    }
    names.sort();
    let mut records: Vec<(PathBuf, Vec<(i64, String)>)> = Vec::new();
    for name in names.into_iter().take(MOST_RECORDS) {
        let path: PathBuf = folder.join(name);
        // A record is read as a plain file: a pipe planted at its name would hold the reader, and the bound above says what one
        //  read may take.
        let Ok(bytes) = read_plain_file(path.as_path(), RECORD_LIMIT) else {
            continue;
        };
        let text: std::borrow::Cow<'_, str> = String::from_utf8_lossy(bytes.as_slice());
        let holders: Vec<(i64, String)> = holders_of(text.as_ref());
        if !holders.is_empty() {
            records.push((path, holders));
        }
    }
    return records;
}

/// What:
///  Every identity the records of this repository name for `pid`,
///  in file-name order.
/// Why:
///   Two records may name one number:
///  the process that had it before,
///  and a later one after a reuse.
///  A caller deciding a lock judges them all,
///  because the first one found is not the only fact about the number.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const identities = recordedIdentities(commonDir, pid); // ['linux:1234#…', …]
/// ```
pub fn identities_recorded(common_dir: &Path, pid: i64) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for (_, holders) in every_holder(common_dir) {
        for (holder, identity) in holders {
            if holder == pid && !found.contains(&identity) {
                found.push(identity);
            }
        }
    }
    return found;
}

/// What:
///  The identity a record names for `pid`,
///  when the writer of a lock whose PID file names `pid` is proved gone.
/// Why:
///  This is the whole proof of `abandoned_lock_proving.rs`,
///  and it is a proof about the writer of the PID file,
/// not about the number:
///
///  - a live process with that number keeps the lock,
///    whatever the records say.
///    Its birth either is a recorded one (then it is the recorded process,
///    and it runs),
///    or it is not (then the number was reused,
///    and nothing tells the live Git that wrote the PID file from a later
///    program that merely has the number).
///    A process table read that fails answers the same way:
///    keep.
///  - with the number free,
///    every record naming it must judge `gone`:
///    a record of another boot is gone,
///    because a boot that ended ends every process in it;
///    a record of another PID namespace or another machine is unknown,
///    and one unknown record keeps the lock,
///    because a Git of that namespace or machine may be the writer.
///  - no record at all is unknown,
///    and keeps it.
///
///  A zombie holds no number here:
///  `process_birth_identity` answers nothing for a process in an exited state,
///  and a process that has exited cannot have written anything since.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const gone = pidIsFree(pid) && recordedIdentities(commonDir, pid).every((id) => holderState([pid, id]) === 'gone');
/// ```
pub fn proved_gone_writer(common_dir: &Path, pid: i64) -> Option<String> {
    let identities: Vec<String> = identities_recorded(common_dir, pid);
    if identities.is_empty() {
        return None;
    }
    // A number a process has now is no proof of anything: it may be the recorded process that runs, or a later program
    //  that reused the number and wrote this very PID file. Either way the lock stays.
    match process_birth_identity(pid) {
        Ok(None) => {}
        Ok(Some(_live)) => return None,
        Err(_unreadable) => return None,
    }
    // Every record for the number must be a process of this boot, this PID namespace and this machine that has ended.
    for identity in &identities {
        if holder_state(Some(&(pid, identity.clone()))) != HolderState::Gone {
            return None;
        }
    }
    return identities.into_iter().next();
}

/// What:
///  Remove every record whose holders are all gone
///  and whose numbers no standing lock of the repository names.
/// Why:
///   A record is the proof a lock's holder is gone,
///  so it stays exactly as long as it can explain a lock;
///  a record whose commands are over and whose numbers stand beside no lock is litter.
///  Nothing here waits and nothing is renamed over another file,
///  so two commands sweeping at once remove the same litter harmlessly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const [file, holders] of records) if (holders.every(dead) && !holders.some(inUse)) rm(file)
/// ```
pub fn sweep(common_dir: &Path, pids_in_use: &[i64]) {
    for (path, holders) in every_holder(common_dir) {
        let mut explainable: bool = false;
        for (pid, identity) in &holders {
            if pids_in_use.contains(pid) || holder_state(Some(&(*pid, identity.clone()))) != HolderState::Gone {
                explainable = true;
                break;
            }
        }
        if explainable {
            continue;
        }
        let _removed = std::fs::remove_file(path.as_path());
    }
}

/// What:
///  The words for one recorded holder that is gone,
///  in the person's terms.
/// Why:
///   The diagnostic names what was read and what it proved,
///  and nothing more.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `PID ${pid}, which the record this repository's commands wrote names with a birth identity this boot no longer has`
/// ```
pub fn gone_holder_words(pid: i64) -> String {
    return format!("PID {pid}, which a record this repository's own commands wrote names by a birth identity that this boot, this process table and this machine no longer have");
}
