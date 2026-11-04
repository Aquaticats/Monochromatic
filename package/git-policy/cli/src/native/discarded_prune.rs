//! What:
//!  The bound on `<common git dir>/cli-git/discarded/`: copies older than a number of days are removed, and the folder is
//!  kept under a size by removing the oldest copies first, never one younger than a day and never the copy just written.
//!  Each removed copy leaves one line in the audit file, and `git cli-git doctor` reports the folder.
//! Why:
//!  The copies of conflicted files were never removed, so the folder grew for as long as the repository lived. They are the
//!  only remaining content of a file a command replaced, so the bound errs toward keeping: both limits are settings of
//!  `cli-git.config.jsonc` (`discardedCopies.maxAgeDays`, `discardedCopies.maxSize`, read by `config_discarded.rs`), a copy
//!  younger than a day is out of reach of both, and the pass runs only after a new copy is complete and durable (so a kill while copying costs the older copies
//!  nothing), and touches only directories named like the wrapper's own (`2026-11-02T14-01-07`, with an optional
//!  `-<number>`). The folder is opened once without following a link and everything is read and removed through that
//!  descriptor (`discarded_store.rs`). No lock is taken: passes may overlap, and a file is counted and audited by the one pass
//!  whose removal of it succeeded, so overlapping passes split a directory's files between their audit lines and every copy
//!  is audited once. A directory another pass removed first is not a problem. No commit waits on a pass. What a pass cannot do
//!  is kept and said, never skipped silently.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const removed = await pruneDiscarded({ commonDir, nowMillis, limits, keep: newestDirectoryName });
//! ```

use super::conflict_sides::SAVED_DIRECTORY;
use super::diagnostic_log::debug;
use super::discard_audit::{AUDIT_FILE, append_line, check_appendable};
use super::discarded_store::{Interrupted, OpenFailure, Store, Tally};
use super::iso_time::format_iso_milliseconds;
use super::json_record::{quote, string_array};
use super::service_protocol::RELEASE;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

pub use super::discarded_store::Identity;

/// Days after which a copy is removed unless the setting says otherwise.
///
/// No measurement sets this yet. Measured on this host (read-only, 2026-11-03T14:55Z): the folder `cli-git/discarded` does
/// not exist in any repository with a Git directory within three levels of the home directory, and the audit file of the main
/// repository (950 lines, 2026-10-21T00:35:32Z to 2026-11-03T14:15:51Z) holds no `saved-conflict` or `pruned` line, while the
/// feature is a day old (commit 1d2f4bec8e, 2026-11-02T14:08:46-05:00). The value is the coordinator's earlier default, kept
/// until growth can be measured; `doc/planning/cli-git-shipped-limits-audit.md`, "Defaults for saved conflict copies", has
/// the measurement, the options and the open question about copies that may be the only copy of a person's work.
pub const DEFAULT_MAX_AGE_DAYS: u64 = 30;

/// Bytes the folder is kept under unless the setting says otherwise: one gibibyte. No measurement sets this yet; see
/// `DEFAULT_MAX_AGE_DAYS`.
pub const DEFAULT_MAX_BYTES: u64 = 1024 * 1024 * 1024;

/// A copy younger than this is never removed, by either limit. No measurement sets this yet; see `DEFAULT_MAX_AGE_DAYS`.
pub const YOUNG_FLOOR_MILLISECONDS: i64 = 24 * 60 * 60 * 1000;

/// Milliseconds in a day.
const DAY_MILLISECONDS: i64 = 24 * 60 * 60 * 1000;

/// The resolution of a directory name: it says the second of the save, and the save may have been made at any millisecond
/// of that second, so an age is counted from the end of it.
pub const STAMP_RESOLUTION_MILLISECONDS: i64 = 1000;

/// How many copy names one audit line lists; the line also states the total.
///  Exempt from a citation under `LQM` (class: display cap): it limits how much text a message, log field or record
///  names, and no command result or duration depends on it. Classed once in `doc/planning/cli-git-shipped-limits-
///  audit.md`, "Exempt limits"; the coordinator's pick under the owner's delegation, open to the owner's veto.
const LISTED_COPIES: usize = 100;

/// Module tag for diagnostics.
const TAG: &str = "discardedPrune";

/// The two limits.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Limits {
    /// Days after which a copy goes.
    pub max_age_days: u64,
    /// Bytes the folder is kept under.
    pub max_bytes: u64,
}

impl Default for Limits {
    fn default() -> Self {
        return Self { max_age_days: DEFAULT_MAX_AGE_DAYS, max_bytes: DEFAULT_MAX_BYTES };
    }
}

/// What:
///  A size as Git's integer settings write it: whole bytes, or a number followed by `k`, `m` or `g` (any case, powers of 1024).
/// Why:
///   `git config --type=int` reads these, so a person who writes `"512m"` means what Git would read; zero, a sign, a fraction
///  and anything else is no size.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// parseSize('512m') // 536870912
/// ```
pub fn parse_size(text: &str) -> Option<u64> {
    let trimmed: &str = text.trim();
    let (digits, unit): (&str, u64) = match trimmed.chars().last()?.to_ascii_lowercase() {
        'k' => (&trimmed[..trimmed.len() - 1], 1024),
        'm' => (&trimmed[..trimmed.len() - 1], 1024 * 1024),
        'g' => (&trimmed[..trimmed.len() - 1], 1024 * 1024 * 1024),
        _ => (trimmed, 1),
    };
    if digits.is_empty() || !digits.bytes().all(|byte| { return byte.is_ascii_digit(); }) {
        return None;
    }
    return digits.parse::<u64>().ok()?.checked_mul(unit).filter(|bytes| { return *bytes > 0; });
}

/// Days from 1970-01-01 to a civil date, the inverse of `iso_time::civil_from_days` (Howard Hinnant's `days_from_civil`).
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let shifted_year: i64 = if month <= 2 { year - 1 } else { year };
    let era: i64 = shifted_year.div_euclid(400);
    let year_of_era: i64 = shifted_year.rem_euclid(400);
    let shifted_month: i64 = if month > 2 { month - 3 } else { month + 9 };
    let day_of_year: i64 = (153 * shifted_month + 2) / 5 + day - 1;
    let day_of_era: i64 = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    return era * 146_097 + day_of_era - 719_468;
}

/// The days of a month of the proleptic Gregorian calendar, February's by the leap-year rule.
fn days_in_month(year: i64, month: i64) -> i64 {
    return match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        _ if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        _ => 28,
    };
}

/// What:
///  The time a copy directory's name stands for, in milliseconds since the epoch, or nothing when the name is not one the
///  wrapper makes.
/// Why:
///   The name (`2026-11-02T14-01-07`, or the same with `-<number>` for a second save in that second) is the time of the
///  save in UTC; reading it, not the directory's modification time, keeps a copy that someone later edited or touched from
///  looking young, and leaves every directory with another name alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// parseStamp('2026-11-02T14-01-07-3') // 1793628067000
/// ```
pub fn parse_stamp(name: &str) -> Option<i64> {
    let bytes: &[u8] = name.as_bytes();
    if bytes.len() < 19 || !name.is_ascii() {
        return None;
    }
    let shape: &[u8; 19] = b"dddd-dd-ddTdd-dd-dd";
    let fits = bytes[..19].iter().zip(shape).all(|(found, wanted)| { return if *wanted == b'd' { found.is_ascii_digit() } else { found == wanted }; });
    if !fits {
        return None;
    }
    let suffix: &str = &name[19..];
    if !(suffix.is_empty() || (suffix.starts_with('-') && suffix.len() > 1 && suffix[1..].bytes().all(|byte| { return byte.is_ascii_digit(); }))) {
        return None;
    }
    let number = |from: usize, to: usize| -> i64 { return name[from..to].parse::<i64>().unwrap_or(0); };
    let (year, month, day): (i64, i64, i64) = (number(0, 4), number(5, 7), number(8, 10));
    let (hour, minute, second): (i64, i64, i64) = (number(11, 13), number(14, 16), number(17, 19));
    if !(1..=12).contains(&month) || day < 1 || day > days_in_month(year, month) || hour > 23 || minute > 59 || second > 59 {
        return None;
    }
    return Some(((days_from_civil(year, month, day) * 24 + hour) * 60 + minute) * 60_000 + second * 1000);
}

/// One copy directory, as listed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Stamped {
    /// The directory's name.
    pub name: String,
    /// The time its name stands for.
    pub millis: i64,
    /// The bytes of the files it holds; wide, so that no sum of file lengths can wrap.
    pub bytes: u128,
    /// The directory the listing saw under that name.
    pub identity: Identity,
}

/// What went wrong with the folder or one of its directories; each is kept, never guessed at.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProblemKind {
    /// The folder cannot be used as a store: a link stands where it should be, or it cannot be opened.
    StoreUnusable,
    /// A directory could not be read, so its size is unknown and it is kept.
    EntryUnreadable,
    /// A directory was replaced or changed between the listing and the removal, and was kept.
    EntryChanged,
    /// A removal failed after some files were gone.
    PartiallyRemoved,
    /// A removal failed before anything was removed.
    NotRemoved,
    /// The audit file could not be opened for appending, so nothing was removed.
    AuditUnwritable,
    /// A directory was removed but its audit line could not be written.
    RemovedWithoutAudit,
}

/// One thing a pass or a listing could not do, with the path it concerns and the reason in plain words.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Problem {
    /// The folder or directory concerned.
    pub path: PathBuf,
    /// What kind of trouble.
    pub kind: ProblemKind,
    /// The reason, as the system gave it.
    pub detail: String,
}

/// What a listing found: the directories named like the wrapper's own, and what it could not read.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Listing {
    /// The directories, oldest first.
    pub entries: Vec<Stamped>,
    /// What could not be read.
    pub problems: Vec<Problem>,
    /// How many files the entries hold.
    pub files: usize,
    /// The files each entry held when it was listed, by the entry's name, so that a removal can tell a directory that only
    /// lost files (another pass removing them) from one that gained a file after it was judged.
    pub listed_files: std::collections::HashMap<String, std::collections::HashSet<String>>,
}

impl Problem {
    /// What:
    ///  The sentence that tells a person what was kept or not done, which input it concerns and what to do about it.
    /// Why:
    ///   The copies are what a person may need back, so every case where the limits could not be applied, or an audit line
    ///  is missing, is said where the person sees it (the notice of the command, and `git cli-git doctor`).
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// `${problem.path} was kept: ${problem.detail}`
    /// ```
    pub fn message(&self) -> String {
        let path = self.path.display();
        let detail: &str = self.detail.as_str();
        return match self.kind {
            ProblemKind::StoreUnusable => format!(
                "cli-git: the saved copies of replaced files in {path} were not pruned, and the age and size limits did not apply: {detail}. \
                 Make it a real directory (move the copies there and remove the link) and the next save prunes it again.\n"
            ),
            ProblemKind::EntryUnreadable => format!(
                "cli-git: {path} was kept and not counted: it cannot be read ({detail}). Fix its permissions or remove it yourself; \
                 until then the size limit sees the folder as smaller than it is.\n"
            ),
            ProblemKind::EntryChanged => format!(
                "cli-git: {path} was kept: it was replaced or changed while the pass ran ({detail}), so it is not the directory the pass judged.\n"
            ),
            ProblemKind::PartiallyRemoved => format!(
                "cli-git: {path} was only partly removed ({detail}). The audit file lists the copies that went and the rest are still there; \
                 fix the cause and the next pass finishes the removal.\n"
            ),
            ProblemKind::NotRemoved => format!("cli-git: {path} could not be removed and was left whole ({detail}). The next pass tries again.\n"),
            ProblemKind::AuditUnwritable => format!(
                "cli-git: no saved copy was removed because the audit file {path} cannot be appended to ({detail}); every removal needs its \
                 line there. Fix the file or its permissions and the next pass removes what the limits allow.\n"
            ),
            ProblemKind::RemovedWithoutAudit => format!(
                "cli-git: {path} was removed but its line in the audit file could not be written ({detail}). Its copies were older or \
                 larger than the limits allow.\n"
            ),
        };
    }
}

/// What a pass did.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PruneOutcome {
    /// The directories removed, in order.
    pub removed: Vec<Removed>,
    /// What could not be done.
    pub problems: Vec<Problem>,
}

/// Why a directory goes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Reason {
    /// Older than the age limit.
    Age,
    /// The folder is over the size limit and this is among the oldest.
    Size,
}

impl Reason {
    /// The word in the audit line.
    pub fn word(self) -> &'static str {
        return match self {
            Reason::Age => "age",
            Reason::Size => "size",
        };
    }
}

/// What:
///  Which directories to remove, as indexes into `entries` with the reason, from the time, the limits and the name to keep.
/// Why:
///   Pure, so the rule is tested without a disk: first every copy past the age limit; then, while the rest is over the
///  size limit, the oldest of those that remain. A directory younger than a day, and the one named `keep`, is never chosen.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const doomed = plan(entries, nowMillis, limits, 'the-new-one') // [[0, 'age'], [3, 'size']]
/// ```
pub fn plan(entries: &[Stamped], now_millis: i64, limits: Limits, keep: Option<&str>) -> Vec<(usize, Reason)> {
    let age_limit: i128 = i128::from(limits.max_age_days) * i128::from(DAY_MILLISECONDS);
    // Counted from the end of the named second, in a width no clock or limit can overflow.
    let age_of = |entry: &Stamped| -> i128 { return i128::from(now_millis) - (i128::from(entry.millis) + i128::from(STAMP_RESOLUTION_MILLISECONDS)); };
    let reachable = |entry: &Stamped| -> bool { return age_of(entry) >= i128::from(YOUNG_FLOOR_MILLISECONDS) && keep != Some(entry.name.as_str()); };
    let mut doomed: Vec<(usize, Reason)> = Vec::new();
    for (index, entry) in entries.iter().enumerate() {
        if reachable(entry) && age_of(entry) > age_limit {
            doomed.push((index, Reason::Age));
        }
    }
    let mut total: u128 = entries.iter().enumerate().filter(|(index, _)| { return !doomed.iter().any(|(gone, _)| { return gone == index; }); }).map(|(_, entry)| { return entry.bytes; }).sum();
    let mut oldest_first: Vec<usize> = (0..entries.len()).filter(|index| { return reachable(&entries[*index]) && !doomed.iter().any(|(gone, _)| { return gone == index; }); }).collect();
    oldest_first.sort_by(|left, right| { return entries[*left].millis.cmp(&entries[*right].millis).then_with(|| { return entries[*left].name.cmp(&entries[*right].name); }); });
    for index in oldest_first {
        if total <= u128::from(limits.max_bytes) {
            break;
        }
        total = total.saturating_sub(entries[index].bytes);
        doomed.push((index, Reason::Size));
    }
    return doomed;
}

/// What:
///  Every directory of the folder that is named like the wrapper's own, oldest first, with its size and what it was when
///  listed, and what could not be read.
/// Why:
///   The folder is opened without following a link and each directory is opened through it, so the sizes are those of the
///  directories that a later removal will meet by identity. A directory that cannot be read has no size to judge by, so it
///  is left out of the entries and named in the problems; a folder that is absent is simply empty.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { entries, problems } = await listStamped(commonDir);
/// ```
pub fn list_stamped(common_dir: &Path) -> Listing {
    return match Store::open(common_dir, SAVED_DIRECTORY) {
        Ok(Some(store)) => list_in(&store, common_dir.join(SAVED_DIRECTORY).as_path()),
        Ok(None) => Listing::default(),
        Err(failure) => Listing { problems: vec![unusable(common_dir.join(SAVED_DIRECTORY).as_path(), &failure)], ..Listing::default() },
    };
}

/// The problem that says why the folder cannot be used.
fn unusable(saved_root: &Path, failure: &OpenFailure) -> Problem {
    let detail: String = match failure {
        OpenFailure::Link => String::from("a symbolic link, or something that is not a directory, stands at it or at its parent `cli-git`, and a removal never follows one"),
        OpenFailure::Refused(error) => format!("it cannot be opened: {error}"),
    };
    return Problem { path: saved_root.to_path_buf(), kind: ProblemKind::StoreUnusable, detail };
}

/// The listing of an opened folder; `saved_root` is the path the problems and the removal records show.
fn list_in(store: &Store, saved_root: &Path) -> Listing {
    let mut listing: Listing = Listing::default();
    let names = match store.names() {
        Ok(found) => found,
        Err(error) => {
            listing.problems.push(Problem { path: saved_root.to_path_buf(), kind: ProblemKind::StoreUnusable, detail: format!("it cannot be read: {error}") });
            return listing;
        }
    };
    for name in names {
        let text: String = name.to_string_lossy().into_owned();
        let Some(millis) = parse_stamp(text.as_str()) else { continue };
        let path: PathBuf = saved_root.join(text.as_str());
        match survey_entry(store, name.as_os_str()) {
            Ok(None) => {}
            Ok(Some((identity, tally))) => {
                listing.files += tally.names.len();
                listing.entries.push(Stamped { name: text.clone(), millis, bytes: tally.bytes, identity });
                listing.listed_files.insert(text, tally.names.into_iter().collect());
            }
            Err(detail) => listing.problems.push(Problem { path, kind: ProblemKind::EntryUnreadable, detail }),
        }
    }
    listing.entries.sort_by(|left, right| { return left.millis.cmp(&right.millis).then_with(|| { return left.name.cmp(&right.name); }); });
    return listing;
}

/// The identity and contents of the directory `name`, nothing when `name` is not a directory (a link or a file), or the
/// reason it cannot be read.
fn survey_entry(store: &Store, name: &OsStr) -> Result<Option<(Identity, Tally)>, String> {
    let Some(seen) = store.directory_identity(name).map_err(|error| { return error.to_string(); })? else {
        return Ok(None);
    };
    let Some(directory) = store.open_directory(name).map_err(|error| { return error.to_string(); })? else {
        return Ok(None);
    };
    let opened: Identity = directory.identity().map_err(|error| { return error.to_string(); })?;
    if opened != seen {
        return Err(String::from("it was replaced while it was being read"));
    }
    return match directory.survey() {
        Ok(tally) => Ok(Some((opened, tally))),
        // Removed by another pass while it was read: nothing to count.
        Err(Interrupted { error, .. }) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(Interrupted { error, .. }) => Err(error.to_string()),
    };
}

/// One removed directory.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Removed {
    /// The directory that was removed.
    pub path: PathBuf,
    /// Its bytes.
    pub bytes: u128,
    /// Why.
    pub reason: Reason,
}

/// What:
///  The audit line for one removed directory: the time, the path, the bytes, the reason, the limit it was judged against,
///  the size of the folder before the pass, and the names of the copies in it.
/// Why:
///   A person looking for a file that vanished finds here that the wrapper removed its copy, when, and by which rule. The
///  common fields are those of the other audit lines, so the file reads the same throughout.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// JSON.stringify({ schemaVersion: 1, time, form: 'PruneDiscarded', outcome: 'pruned', reason, path, bytes, copies }) + '\n'
/// ```
pub fn prune_line(removed: &Removed, copies: &[String], limits: Limits, folder_bytes: u128, now_millis: i64) -> String {
    return audit_line("pruned", removed, copies, limits, folder_bytes, now_millis);
}

/// The audit line of one directory with the outcome word `pruned` (all of it went) or `partial` (some of it went and the
/// rest was kept).
fn audit_line(outcome: &str, removed: &Removed, copies: &[String], limits: Limits, folder_bytes: u128, now_millis: i64) -> String {
    let limit: u64 = match removed.reason {
        Reason::Age => limits.max_age_days,
        Reason::Size => limits.max_bytes,
    };
    let listed: Vec<String> = copies.iter().take(LISTED_COPIES).cloned().collect();
    return format!(
        "{{\"schemaVersion\":1,\"time\":{},\"level\":\"info\",\"timeMillis\":{},\"release\":{},\"pid\":{},\"form\":\"PruneDiscarded\",\
         \"outcome\":{},\"reason\":{},\"limit\":{limit},\"folderBytes\":{folder_bytes},\"path\":{},\"bytes\":{},\
         \"copiesTotal\":{},\"copies\":{}}}\n",
        quote(&format_iso_milliseconds(now_millis)),
        quote(&now_millis.to_string()),
        quote(RELEASE),
        std::process::id(),
        quote(outcome),
        quote(removed.reason.word()),
        quote(&removed.path.to_string_lossy()),
        removed.bytes,
        copies.len(),
        string_array(&listed),
    );
}

/// What:
///  Removes what `plan` chooses from the folder, one directory at a time, and writes one audit line after each removal.
/// Why:
///   A line per directory, written once it is gone, means a kill between two removals leaves a line for each directory that
///  went and a whole directory for each that did not; a directory that cannot be removed is left, said as a debug line, and
///  has no line. No lock is taken and no commit waits on this; a directory that another pass removes first is that pass's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const removed = await pruneFolder({ commonDir, nowMillis, limits, keep })
/// ```
pub fn prune(common_dir: &Path, now_millis: i64, limits: Limits, keep: Option<&str>) -> PruneOutcome {
    return prune_with(common_dir, now_millis, limits, keep, &mut |_candidate: &Path| {});
}

/// `prune` with a step run for each chosen directory after the listing and before its removal: the controls use it to
/// change the folder at the moment a pass is most exposed; `prune` passes a step that does nothing.
pub fn prune_with(common_dir: &Path, now_millis: i64, limits: Limits, keep: Option<&str>, before_removal: &mut dyn FnMut(&Path)) -> PruneOutcome {
    let saved_root: PathBuf = common_dir.join(SAVED_DIRECTORY);
    let mut outcome: PruneOutcome = PruneOutcome::default();
    let store: Store = match Store::open(common_dir, SAVED_DIRECTORY) {
        Ok(Some(opened)) => opened,
        Ok(None) => return outcome,
        Err(failure) => {
            outcome.problems.push(unusable(saved_root.as_path(), &failure));
            return outcome;
        }
    };
    let listing: Listing = list_in(&store, saved_root.as_path());
    outcome.problems.extend(listing.problems);
    let doomed: Vec<(usize, Reason)> = plan(listing.entries.as_slice(), now_millis, limits, keep);
    if doomed.is_empty() {
        return outcome;
    }
    let audit_path: PathBuf = common_dir.join(AUDIT_FILE);
    // Every removal is followed by its line, so a file that cannot take one stops the pass before the first removal.
    if let Err(error) = check_appendable(audit_path.as_path()) {
        outcome.problems.push(Problem { path: audit_path, kind: ProblemKind::AuditUnwritable, detail: error.to_string() });
        return outcome;
    }
    let folder_bytes: u128 = listing.entries.iter().map(|entry| { return entry.bytes; }).sum();
    for (index, reason) in doomed {
        let entry: &Stamped = &listing.entries[index];
        let path: PathBuf = saved_root.join(entry.name.as_str());
        before_removal(path.as_path());
        let mut context: Removal = Removal { store: &store, audit_path: audit_path.as_path(), limits, folder_bytes, now_millis, listed_files: &listing.listed_files };
        remove_entry(&mut context, entry, path, reason, &mut outcome);
    }
    return outcome;
}

/// What a removal needs besides the entry: the folder, where the audit goes, the limits and the numbers its line states.
struct Removal<'pass> {
    /// The folder this pass opened.
    store: &'pass Store,
    /// The audit file.
    audit_path: &'pass Path,
    /// The limits the pass judges by.
    limits: Limits,
    /// The size of the folder before the pass.
    folder_bytes: u128,
    /// The time of the pass.
    now_millis: i64,
    /// The files each directory held when the pass listed it.
    listed_files: &'pass std::collections::HashMap<String, std::collections::HashSet<String>>,
}

/// Writes the audit line of a directory of which `tally` went; said as a problem when the line cannot be written.
fn write_audit(context: &Removal, outcome_word: &str, removed: &Removed, tally: &Tally, outcome: &mut PruneOutcome) {
    let line: String = audit_line(outcome_word, removed, tally.names.as_slice(), context.limits, context.folder_bytes, context.now_millis);
    if let Err(error) = append_line(context.audit_path, line.as_bytes()) {
        debug(TAG, &format!("the audit record of {} could not be written: {error}", removed.path.display()));
        outcome.problems.push(Problem { path: removed.path.clone(), kind: ProblemKind::RemovedWithoutAudit, detail: error.to_string() });
    }
}

/// What:
///  Removes one chosen directory through the open folder: opens it by name without following a link, checks that it is the
///  directory the listing saw, removes its files and subdirectories through its descriptor, then the empty directory, and
///  audits what went.
/// Why:
///   The directory that was judged is the only thing removed: a name that now stands for another directory is kept and said,
///  a link is never entered, and a failure part way is audited as partial and reported, never counted as a removal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await removeEntry(context, entry, path, reason, outcome)
/// ```
fn remove_entry(context: &mut Removal, entry: &Stamped, path: PathBuf, reason: Reason, outcome: &mut PruneOutcome) {
    let name: &OsStr = OsStr::new(entry.name.as_str());
    let kept = |kind: ProblemKind, detail: String| -> Problem { return Problem { path: path.clone(), kind, detail }; };
    let directory = match context.store.open_directory(name) {
        Ok(Some(opened)) => opened,
        // Gone: another actor removed it, which is what this pass was going to do.
        Ok(None) => return,
        Err(error) if error.kind() == std::io::ErrorKind::NotADirectory => {
            outcome.problems.push(kept(ProblemKind::EntryChanged, String::from("the name is now a link or a file")));
            return;
        }
        Err(error) => {
            outcome.problems.push(kept(ProblemKind::NotRemoved, error.to_string()));
            return;
        }
    };
    match directory.identity() {
        Ok(found) if found == entry.identity => {}
        // The same directory whose change time moved: another pass removing its files does that, and so does a file added after
        //  the listing. Only a directory that gained nothing is the directory that was judged.
        Ok(found) if found.device == entry.identity.device && found.inode == entry.identity.inode => match directory.survey() {
            Ok(now) if now.names.iter().all(|file| { return context.listed_files.get(entry.name.as_str()).is_some_and(|listed| { return listed.contains(file); }); }) => {}
            Ok(_) => {
                outcome.problems.push(kept(ProblemKind::EntryChanged, String::from("a file was added to it after the pass listed it")));
                return;
            }
            // Removed by another pass while it was looked at.
            Err(Interrupted { error, .. }) if error.kind() == std::io::ErrorKind::NotFound => return,
            Err(Interrupted { error, .. }) => {
                outcome.problems.push(kept(ProblemKind::NotRemoved, error.to_string()));
                return;
            }
        },
        Ok(_) => {
            outcome.problems.push(kept(ProblemKind::EntryChanged, String::from("the name now stands for another directory")));
            return;
        }
        Err(error) => {
            outcome.problems.push(kept(ProblemKind::NotRemoved, error.to_string()));
            return;
        }
    }
    let mut gone: Removed = Removed { path: path.clone(), bytes: 0, reason };
    let tally: Tally = match directory.empty() {
        Ok(emptied) => emptied,
        Err(Interrupted { tally, error }) => {
            if tally.names.is_empty() {
                outcome.problems.push(kept(ProblemKind::NotRemoved, error.to_string()));
            } else {
                gone.bytes = tally.bytes;
                write_audit(context, "partial", &gone, &tally, outcome);
                outcome.problems.push(kept(ProblemKind::PartiallyRemoved, error.to_string()));
            }
            return;
        }
    };
    gone.bytes = tally.bytes;
    match context.store.remove_empty_directory(name, entry.identity) {
        Ok(true) => {}
        Ok(false) => match context.store.directory_identity(name) {
            // Another pass removed the emptied directory first. It audits what it removed; this pass audits what it removed.
            Ok(None) if tally.names.is_empty() => return,
            Ok(None) => {}
            _ => outcome.problems.push(kept(ProblemKind::EntryChanged, String::from("the name now stands for another directory, which was kept; the copies judged are gone"))),
        },
        Err(error) => {
            write_audit(context, "partial", &gone, &tally, outcome);
            outcome.problems.push(kept(ProblemKind::PartiallyRemoved, format!("its copies are gone but the empty directory could not be removed: {error}")));
            return;
        }
    }
    write_audit(context, "pruned", &gone, &tally, outcome);
    outcome.removed.push(gone);
}

/// The name of the directory under the folder that holds a copy, or nothing for a path outside the folder.
pub fn directory_name_of(saved_root: &Path, copy: &Path) -> Option<String> {
    return copy.strip_prefix(saved_root).ok()?.components().next().map(|first| { return first.as_os_str().to_string_lossy().into_owned(); });
}

/// What the folder holds, for `git cli-git doctor`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Report {
    /// Bytes of every copy.
    pub bytes: u128,
    /// How many files the copies are.
    pub copies: usize,
    /// How many saves (directories) they came from.
    pub saves: usize,
    /// The name of the oldest save, empty when no save could be read.
    pub oldest: String,
    /// What could not be read.
    pub problems: Vec<Problem>,
}

/// The folder's size, copy count and oldest save and what could not be read, or nothing when it holds no copy and nothing
/// went wrong.
pub fn report(common_dir: &Path) -> Option<Report> {
    let listing: Listing = list_stamped(common_dir);
    if listing.entries.is_empty() && listing.problems.is_empty() {
        return None;
    }
    return Some(Report {
        bytes: listing.entries.iter().map(|entry| { return entry.bytes; }).sum(),
        copies: listing.files,
        saves: listing.entries.len(),
        oldest: listing.entries.first().map_or_else(String::new, |entry| { return entry.name.clone(); }),
        problems: listing.problems,
    });
}

/// The doctor's text for the folder: empty when it holds no copy and nothing went wrong, otherwise one block with the size,
/// the count, the oldest save and the limits that apply, then a sentence for each thing that could not be read.
pub fn doctor_section(common_dir: &Path, limits: Limits) -> String {
    let Some(found) = report(common_dir) else {
        return String::new();
    };
    let mut text: String = String::new();
    if found.saves > 0 {
        text = format!(
            "git cli-git doctor: discarded copies: {} holds {} bytes in {} copies from {} saves; the oldest save is {}. Copies older than {} days \
             are removed, and the oldest are removed past {} bytes, never one younger than a day (`discardedCopies.maxAgeDays` and \
             `discardedCopies.maxSize` in cli-git.config.jsonc change the limits).\n",
            common_dir.join(SAVED_DIRECTORY).display(),
            found.bytes,
            found.copies,
            found.saves,
            found.oldest,
            limits.max_age_days,
            limits.max_bytes,
        );
    }
    for problem in &found.problems {
        text.push_str(problem.message().replacen("cli-git: ", "git cli-git doctor: discarded copies: ", 1).as_str());
    }
    return text;
}

/// Pruning is controlled in the test module.
#[cfg(test)]
#[path = "discarded_prune_tests.rs"]
mod tests;

/// What a pass may touch and what it must say is controlled in the guard test module.
#[cfg(test)]
#[path = "discarded_prune_guard_tests.rs"]
mod guard_tests;
