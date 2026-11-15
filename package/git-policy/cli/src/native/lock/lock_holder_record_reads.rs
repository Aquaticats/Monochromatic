//! What:
//!  What the wrapper's records of the processes it started say about a lock's holder:
//!  the identity one record names for a process number,
//!  the proof that a holder is gone,
//!  who may remove a record and on what proof,
//!  and the folder that goes with the last record it held.
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
//!  Who may remove a record is the second half of that proof:
//!  a record file has one writer, the process its own line names,
//!  because the file is named by that process's number and identity and only that process appends to it.
//!  Only two things remove it, and both say what they know:
//!  the writer itself, at the end of its command, because it will append nothing more;
//!  and any other command's pass, only when the writer is proved gone (never when its state is unknown,
//!  and never for a record that names no writer),
//!  every process the record names is proved gone,
//!  and a walk of the lock files made after the record was read finds no lock naming any of them.
//!  A writer proved gone cannot append, so the reading is of a file nothing can change;
//!  a walk made after the reading sees every lock the reading could not know about.
//!  A record left a little longer loses nothing, and a record removed too early turns a lock that could be proved
//!  abandoned into a commit that fails.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const identity = recordedIdentity(commonDir, pid); const gone = provenGone(commonDir, pid);
//! const records = everyRecord(commonDir); /* walk the locks */ removeRecordsNoOneNeeds(commonDir, records, pidsInUse);
//! ```

/// The record files and their lines.
use crate::lock_holder_records::{CHILD_KIND, SELF_KIND, folder_of};
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
///  The lines one record file names:
///  whether each is the writer's own line,
///  the number,
///  and the identity.
/// Why:
///   A record that cannot be read names nobody,
///  and a line that is not one of the two kinds is dropped,
///  so a file a person edited decides nothing.
///  Both spellings a writer of this branch has used are read,
///  one space after the kind and two,
///  because a record of the two-space form is still on disk wherever
///  this branch has run and the writer wrote exactly that for a forwarded command's own line.
///  The kind is kept because a record's writer is the process its own line names,
///  and who may append to the file is the whole question of when it may be removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// parseRecord(text) // [{ own: true, pid, identity }, ...]
/// ```
fn lines_of(text: &str) -> Vec<(bool, i64, String)> {
    let mut lines: Vec<(bool, i64, String)> = Vec::new();
    for line in text.lines().skip(1) {
        // The kind and the rest of the line, with whatever spacing the writer used: the first form wrote two spaces
        //  after `self` (`self  <pid> <identity>`), and records of it are on disk wherever this branch has run.
        let Some((kind, rest)) = line.split_once(' ') else {
            continue;
        };
        if kind != SELF_KIND && kind != CHILD_KIND {
            continue;
        }
        let Some((pid, identity)) = rest.trim_start().split_once(' ') else {
            continue;
        };
        let (Ok(number), false) = (pid.trim().parse::<i64>(), identity.trim().is_empty()) else {
            continue;
        };
        lines.push((kind == SELF_KIND, number, identity.trim().to_owned()));
    }
    return lines;
}

/// What:
///  One record file:
///  where it is,
///  the process that wrote it (the own line it carries, when it carries one),
///  and every holder it names.
/// Why:
///   Who may remove a record is a question about its writer,
///  because that writer is the only process that ever appends to the file:
///  a record whose writer is proved gone cannot change any more,
///  and one whose writer may still run must stay.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Record = { path: string; writer?: [pid: number, identity: string]; holders: [number, string][] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Record {
    /// The record file.
    pub path: PathBuf,
    /// The writer's own line,
    ///  or nothing for a record an earlier release wrote without one.
    pub writer: Option<(i64, String)>,
    /// Every process the record names.
    pub holders: Vec<(i64, String)>,
}

/// What:
///  Every record the repository holds,
///  in file-name order and at most `MOST_RECORDS` files.
/// Why:
///   A lookup and a removal ask the same question of the same files,
///  so they read them once through one function.
///  A caller that removes records reads them first and walks the lock files after,
///  so the removal is decided on a reading of the records and a later sight of every lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const name of readdirSync(folder).sort()) { records.push(parseRecord(name)) }
/// ```
pub fn every_record(common_dir: &Path) -> Vec<Record> {
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
    let mut records: Vec<Record> = Vec::new();
    for name in names.into_iter().take(MOST_RECORDS) {
        let path: PathBuf = folder.join(name);
        if let Some(record) = record_at(path.as_path()) {
            records.push(record);
        }
    }
    return records;
}

/// What:
///  One record, read from its file:
///  the process its own line names,
///  and every holder it names.
///  `Option<Record>` is the record or nothing when the file names no holder.
/// Why:
///   The pass reads every record of a repository at once,
///  and the process that may remove its own record reads the very file it wrote,
///  so both get the same answer about who the writer is.
///  A record is read as a plain file: a pipe planted at its name would hold the reader,
///  and the bound says what one read may take.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// recordAt(path) // { path, writer, holders } or undefined
/// ```
fn record_at(path: &Path) -> Option<Record> {
    let Ok(bytes) = read_plain_file(path, RECORD_LIMIT) else {
        return None;
    };
    let text: std::borrow::Cow<'_, str> = String::from_utf8_lossy(bytes.as_slice());
    let lines: Vec<(bool, i64, String)> = lines_of(text.as_ref());
    let holders: Vec<(i64, String)> = lines
        .iter()
        .map(|(_, pid, identity)| { return (*pid, identity.clone()); })
        .collect();
    if holders.is_empty() {
        return None;
    }
    // The first own line is the writer's: a record has one, and one written by a release that named its writer twice named
    //  the same process both times.
    let writer: Option<(i64, String)> = lines
        .iter()
        .find(|(own, _, _)| { return *own; })
        .map(|(_, pid, identity)| { return (*pid, identity.clone()); });
    return Some(Record { path: path.to_path_buf(), writer, holders });
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
    for held in every_record(common_dir) {
        for (holder, identity) in held.holders {
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
///  Remove every record nothing can still need,
///  from records the caller has already read,
///  and the folder itself once it holds no record.
/// Why:
///  A record may be removed by a command that did not write it only when all of this holds:
///
///  - no lock that stands names any process the record names,
///    and the caller's walk is made after the records were read,
///    so a lock another command's Git made in between is in that walk and keeps its record;
///  - every process the record names is proved gone;
///  - the record names its writer and that writer is proved gone,
///    so nothing can append a line after the reading the caller made.
///    A record of another boot is gone,
///    one of another machine or PID namespace is unknown and keeps it,
///    and a record an earlier release wrote without its own line is never removed here.
///
///  The folder is litter once the last record it held is gone,
///  and a person's own tool that lists a Git directory counts a folder that explains nothing like any other file of
///  theirs,
///  so it goes with that record.
///  `remove_dir` refuses a folder that still holds anything,
///  so a record another command wrote meanwhile keeps it.
///  Nothing here waits and nothing is renamed over another file,
///  so two commands removing the same litter do so harmlessly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const record of records) if (nothingCanNeed(record, pidsInUse)) rm(record.path); try { rmdir(folder) } catch {}
/// ```
pub fn remove_records_no_one_needs(common_dir: &Path, records: &[Record], pids_in_use: &[i64]) {
    for record in records {
        if !nothing_can_need(record, pids_in_use, false) {
            continue;
        }
        let _removed = std::fs::remove_file(record.path.as_path());
    }
    let _removed_folder = std::fs::remove_dir(folder_of(common_dir).as_path());
}

/// What:
///  Remove the record this process wrote,
///  when the caller's walk of the lock files names no process the record names,
///  and the folder with it once it holds no record.
///  `pids_in_use` is read from a walk the caller made after it read the record.
/// Why:
///  The writer knows what no other command can know:
///  it will append nothing more.
///  So it may remove its own record on a walk made after its last Git ended,
///  while every other process must prove the writer gone before it removes the same record.
///  The processes the record names must all be proved gone as well:
///  a Git this command left running may still be killed and leave a lock the record explains.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const record = readThisProcessRecord(commonDir); if (record && nothingCanNeed(record, pidsInUse, true)) rm(record.path); rmdir(folder) catch {}
/// ```
pub fn remove_this_process_record_if_no_lock_needs_it(common_dir: &Path, pids_in_use: &[i64]) {
    let Some(path) = crate::lock_holder_records::this_process_record_path(common_dir) else {
        return;
    };
    let Some(record) = record_at(path.as_path()) else {
        return;
    };
    if !nothing_can_need(&record, pids_in_use, true) {
        return;
    }
    let _removed = std::fs::remove_file(path.as_path());
    let _removed_folder = std::fs::remove_dir(folder_of(common_dir).as_path());
}

/// What:
///  Whether no lock and no running process can still need `record`.
///  `the_writer_is_this_process` says the caller wrote the record and will append nothing more,
///  which is what lets the writer answer for a process that is still running.
/// Why:
///  This is the whole rule of who may remove a record,
///  in one place,
///  so the writer and every other command answer it the same way:
///  a standing lock naming any process the record names keeps it,
///  every process it names must be proved gone,
///  and only the writer itself may stand in for the proof that nothing will be appended.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const nothingCanNeed = (record, pidsInUse, writerIsMine) =>
///   !record.holders.some(([pid]) => pidsInUse.includes(pid))
///   && record.holders.every((holder) => (writerIsMine && holder === record.writer) || holderState(holder) === 'gone')
///   && (writerIsMine || holderState(record.writer) === 'gone');
/// ```
fn nothing_can_need(record: &Record, pids_in_use: &[i64], the_writer_is_this_process: bool) -> bool {
    if record.holders.iter().any(|(pid, _)| { return pids_in_use.contains(pid); }) {
        return false;
    }
    for holder in &record.holders {
        if the_writer_is_this_process && record.writer.as_ref() == Some(holder) {
            // The writer is this process, and it has ended every Git it started and will append nothing more.
            continue;
        }
        if holder_state(Some(holder)) != HolderState::Gone {
            return false;
        }
    }
    if the_writer_is_this_process {
        return true;
    }
    // Every other remover must show that the file cannot change any more.
    return record.writer.as_ref().is_some_and(|writer| { return holder_state(Some(writer)) == HolderState::Gone; });
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
