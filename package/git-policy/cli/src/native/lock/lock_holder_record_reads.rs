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
//!  The spelling, the parts and the judgement are the commit service's own
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
        let Some(rest) = line.strip_prefix(SELF_LINE).or_else(|| { return line.strip_prefix(CHILD_LINE); }) else {
            continue;
        };
        let mut parts = rest.splitn(2, ' ');
        let (Some(pid), Some(identity)) = (parts.next(), parts.next()) else {
            continue;
        };
        let (Ok(pid), false) = (pid.parse::<i64>(), identity.is_empty()) else {
            continue;
        };
        holders.push((pid, identity.to_owned()));
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
        if entry.file_type().is_ok_and(|kind| { return kind.is_file(); }) {
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
///  The identity a record of this repository names for `pid`,
///  and the file it is in.
/// Why:
///   Two records may name one number after a reuse;
///  the identity then decides.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const [file, identity] = recordedIdentity(commonDir, pid) ?? [];
/// ```
pub fn recorded_identity(common_dir: &Path, pid: i64) -> Option<String> {
    for (_, holders) in every_holder(common_dir) {
        for (holder, identity) in holders {
            if holder == pid {
                return Some(identity);
            }
        }
    }
    return None;
}

/// What:
///  The identity a record names for `pid`,
///  when this wrapper's own account proves that holder gone.
/// Why:
///   This is the whole proof of `abandoned_lock_proving.rs`:
///  a record of this repository,
///  of this boot,
///  this PID namespace and this machine,
///  whose birth identity no running process has.
///  An identity of another boot is gone as well,
///  because a boot that ended ends every process in it;
///  a record of another namespace or another machine answers unknown and is never read as gone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const gone = holderState([pid, recordedIdentity(commonDir, pid)]) === 'gone';
/// ```
pub fn proven_gone(common_dir: &Path, pid: i64) -> Option<String> {
    let identity: String = recorded_identity(common_dir, pid)?;
    let holder: (i64, String) = (pid, identity);
    if holder_state(Some(&holder)) != HolderState::Gone {
        return None;
    }
    return Some(holder.1);
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
