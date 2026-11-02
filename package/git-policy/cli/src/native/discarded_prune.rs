//! What:
//!  The bound on `<common git dir>/cli-git/discarded/`: copies older than a number of days are removed, and the folder is
//!  kept under a size by removing the oldest copies first, never one younger than a day and never the copy just written.
//!  Each removed copy leaves one line in the audit file, and `git cli-git doctor` reports the folder.
//! Why:
//!  The copies of conflicted files were never removed, so the folder grew for as long as the repository lived. They are the
//!  only remaining content of a file a command replaced, so the bound errs toward keeping: both limits are read from Git
//!  configuration (`cli-git.discardedMaxAgeDays`, `cli-git.discardedMaxSize`), a copy younger than a day is out of reach of
//!  both, and the pass runs only after a new copy is complete and durable (so a kill while copying costs the older copies
//!  nothing), takes no lock, and touches only directories named like the wrapper's own (`2026-11-02T14-01-07`, with an
//!  optional `-<number>`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const removed = await pruneDiscarded({ commonDir, nowMillis, limits, keep: newestDirectoryName });
//! ```

use super::conflict_sides::SAVED_DIRECTORY;
use super::diagnostic_log::{debug, warn};
use super::discard_audit::{AUDIT_FILE, append_line};
use super::iso_time::format_iso_milliseconds;
use super::json_record::{quote, string_array};
use super::repo_read_config_list::list_config;
use super::repo_read_config_scan::ScopeEnvironment;
use super::repo_read_config_text::{ConfigEntry, last_entry};
use super::service_protocol::RELEASE;
use std::path::{Path, PathBuf};

/// The Git configuration key (lower case, as the reader lists it) for the age limit in whole days.
pub const AGE_KEY: &str = "cli-git.discardedmaxagedays";

/// The Git configuration key (lower case, as the reader lists it) for the size limit, in bytes or with the suffix `k`, `m`
/// or `g` that Git's integer settings take.
pub const SIZE_KEY: &str = "cli-git.discardedmaxsize";

/// Days after which a copy is removed unless the setting says otherwise.
pub const DEFAULT_MAX_AGE_DAYS: u64 = 30;

/// Bytes the folder is kept under unless the setting says otherwise: one gibibyte.
pub const DEFAULT_MAX_BYTES: u64 = 1024 * 1024 * 1024;

/// A copy younger than this is never removed, by either limit.
pub const YOUNG_FLOOR_MILLISECONDS: i64 = 24 * 60 * 60 * 1000;

/// Milliseconds in a day.
const DAY_MILLISECONDS: i64 = 24 * 60 * 60 * 1000;

/// How many copy names one audit line lists; the line also states the total.
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
///   `git config --type=int` reads these, so a person who writes `512m` means what Git would read; zero, a sign, a fraction
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

/// What:
///  The limits the configuration entries say, and a sentence for each value that is no limit.
/// Why:
///   A value that is not a positive whole number (or size) keeps the default and is named, never read as "no limit" or as a
///  zero that would remove everything.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { limits, problems } = limitsFrom(entries);
/// ```
pub fn limits_from(entries: &[ConfigEntry]) -> (Limits, Vec<String>) {
    let mut limits: Limits = Limits::default();
    let mut problems: Vec<String> = Vec::new();
    let value_of = |key: &str| -> Option<String> {
        return last_entry(entries, key).map(|entry| { return String::from_utf8_lossy(entry.value.as_deref().unwrap_or(&[])).into_owned(); });
    };
    if let Some(text) = value_of(AGE_KEY) {
        match text.trim().parse::<u64>() {
            Ok(days) if days > 0 => limits.max_age_days = days,
            _ => problems.push(format!("{AGE_KEY} is {text:?}, which is not a whole number of days above zero; {DEFAULT_MAX_AGE_DAYS} days apply")),
        }
    }
    if let Some(text) = value_of(SIZE_KEY) {
        match parse_size(text.as_str()) {
            Some(bytes) => limits.max_bytes = bytes,
            None => problems.push(format!("{SIZE_KEY} is {text:?}, which is not a size above zero (bytes, or a number with k, m or g); one gibibyte applies")),
        }
    }
    return (limits, problems);
}

/// The limits Git configuration sets for this repository (system, global, repository and worktree files), the defaults
/// where it sets none or the reader cannot settle the listing. A value that is no limit is said on standard error.
pub fn read_limits(common_dir: &Path, git_dir: &Path) -> Limits {
    let variables: Vec<(std::ffi::OsString, std::ffi::OsString)> = std::env::vars_os().collect();
    let scope: ScopeEnvironment = ScopeEnvironment::from_variables(variables.as_slice());
    return read_limits_with(common_dir, git_dir, &scope, variables.as_slice());
}

/// `read_limits` over a given set of configuration files and environment, so a control reads a fixture and not the host.
pub fn read_limits_with(common_dir: &Path, git_dir: &Path, scope: &ScopeEnvironment, variables: &[(std::ffi::OsString, std::ffi::OsString)]) -> Limits {
    let Some(listed) = list_config(common_dir, git_dir, scope, variables) else {
        debug(TAG, "the configuration could not be listed in process, so the default limits apply");
        return Limits::default();
    };
    let entries: Vec<ConfigEntry> = listed.into_iter().map(|item| { return item.entry; }).collect();
    let (limits, problems) = limits_from(entries.as_slice());
    for problem in problems {
        warn(TAG, problem.as_str());
    }
    return limits;
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
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) || hour > 23 || minute > 59 || second > 60 {
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
    /// The bytes of the files it holds.
    pub bytes: u64,
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
    let age_limit: i64 = i64::try_from(limits.max_age_days).unwrap_or(i64::MAX).saturating_mul(DAY_MILLISECONDS);
    let reachable = |entry: &Stamped| -> bool { return now_millis - entry.millis >= YOUNG_FLOOR_MILLISECONDS && keep != Some(entry.name.as_str()); };
    let mut doomed: Vec<(usize, Reason)> = Vec::new();
    for (index, entry) in entries.iter().enumerate() {
        if reachable(entry) && now_millis - entry.millis > age_limit {
            doomed.push((index, Reason::Age));
        }
    }
    let mut total: u64 = entries.iter().enumerate().filter(|(index, _)| { return !doomed.iter().any(|(gone, _)| { return gone == index; }); }).map(|(_, entry)| { return entry.bytes; }).sum();
    let mut oldest_first: Vec<usize> = (0..entries.len()).filter(|index| { return reachable(&entries[*index]) && !doomed.iter().any(|(gone, _)| { return gone == index; }); }).collect();
    oldest_first.sort_by(|left, right| { return entries[*left].millis.cmp(&entries[*right].millis).then_with(|| { return entries[*left].name.cmp(&entries[*right].name); }); });
    for index in oldest_first {
        if total <= limits.max_bytes {
            break;
        }
        total = total.saturating_sub(entries[index].bytes);
        doomed.push((index, Reason::Size));
    }
    return doomed;
}

/// The files below a directory (symbolic links counted by their own size, never followed), as relative names and the sum of
/// their sizes; a work stack walks it, so a deep tree takes no recursion.
fn walk(root: &Path) -> (Vec<String>, u64) {
    let mut names: Vec<String> = Vec::new();
    let mut bytes: u64 = 0;
    let mut pending: Vec<PathBuf> = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        let Ok(listing) = std::fs::read_dir(&directory) else { continue };
        for entry in listing.flatten() {
            let Ok(kind) = entry.file_type() else { continue };
            if kind.is_dir() {
                pending.push(entry.path());
            } else {
                bytes += entry.metadata().map_or(0, |metadata| { return metadata.len(); });
                names.push(entry.path().strip_prefix(root).unwrap_or(entry.path().as_path()).to_string_lossy().into_owned());
            }
        }
    }
    names.sort();
    return (names, bytes);
}

/// Every directory of the folder that is named like the wrapper's own, with its size; none when the folder is absent.
pub fn list_stamped(saved_root: &Path) -> Vec<Stamped> {
    let Ok(listing) = std::fs::read_dir(saved_root) else {
        return Vec::new();
    };
    let mut found: Vec<Stamped> = Vec::new();
    for entry in listing.flatten() {
        let name: String = entry.file_name().to_string_lossy().into_owned();
        let Some(millis) = parse_stamp(name.as_str()) else { continue };
        if !entry.file_type().is_ok_and(|kind| { return kind.is_dir(); }) {
            continue;
        }
        found.push(Stamped { name, millis, bytes: walk(entry.path().as_path()).1 });
    }
    found.sort_by(|left, right| { return left.millis.cmp(&right.millis).then_with(|| { return left.name.cmp(&right.name); }); });
    return found;
}

/// One removed directory.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Removed {
    /// The directory that was removed.
    pub path: PathBuf,
    /// Its bytes.
    pub bytes: u64,
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
pub fn prune_line(removed: &Removed, copies: &[String], limits: Limits, folder_bytes: u64, now_millis: i64) -> String {
    let limit: u64 = match removed.reason {
        Reason::Age => limits.max_age_days,
        Reason::Size => limits.max_bytes,
    };
    let listed: Vec<String> = copies.iter().take(LISTED_COPIES).cloned().collect();
    return format!(
        "{{\"schemaVersion\":1,\"time\":{},\"level\":\"info\",\"timeMillis\":{},\"release\":{},\"pid\":{},\"form\":\"PruneDiscarded\",\
         \"outcome\":\"pruned\",\"reason\":{},\"limit\":{limit},\"folderBytes\":{folder_bytes},\"path\":{},\"bytes\":{},\
         \"copiesTotal\":{},\"copies\":{}}}\n",
        quote(&format_iso_milliseconds(now_millis)),
        quote(&now_millis.to_string()),
        quote(RELEASE),
        std::process::id(),
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
///  has no line. No lock is taken and no commit waits on this.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const removed = await pruneFolder({ commonDir, nowMillis, limits, keep })
/// ```
pub fn prune(common_dir: &Path, now_millis: i64, limits: Limits, keep: Option<&str>) -> Vec<Removed> {
    let saved_root: PathBuf = common_dir.join(SAVED_DIRECTORY);
    let entries: Vec<Stamped> = list_stamped(saved_root.as_path());
    let folder_bytes: u64 = entries.iter().map(|entry| { return entry.bytes; }).sum();
    let mut removed: Vec<Removed> = Vec::new();
    for (index, reason) in plan(entries.as_slice(), now_millis, limits, keep) {
        let entry: &Stamped = &entries[index];
        let path: PathBuf = saved_root.join(entry.name.as_str());
        let (copies, _) = walk(path.as_path());
        if let Err(error) = std::fs::remove_dir_all(&path) {
            debug(TAG, &format!("{} could not be removed: {error}", path.display()));
            continue;
        }
        let gone: Removed = Removed { path, bytes: entry.bytes, reason };
        let line: String = prune_line(&gone, copies.as_slice(), limits, folder_bytes, now_millis);
        if let Err(error) = append_line(common_dir.join(AUDIT_FILE).as_path(), line.as_bytes()) {
            debug(TAG, &format!("the audit record of {} could not be written: {error}", gone.path.display()));
        }
        removed.push(gone);
    }
    return removed;
}

/// The name of the directory under the folder that holds a copy, or nothing for a path outside the folder.
pub fn directory_name_of(saved_root: &Path, copy: &Path) -> Option<String> {
    return copy.strip_prefix(saved_root).ok()?.components().next().map(|first| { return first.as_os_str().to_string_lossy().into_owned(); });
}

/// What the folder holds, for `git cli-git doctor`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Report {
    /// Bytes of every copy.
    pub bytes: u64,
    /// How many files the copies are.
    pub copies: usize,
    /// How many saves (directories) they came from.
    pub saves: usize,
    /// The name of the oldest save.
    pub oldest: String,
}

/// The folder's size, copy count and oldest save, or nothing when it holds no copy.
pub fn report(common_dir: &Path) -> Option<Report> {
    let saved_root: PathBuf = common_dir.join(SAVED_DIRECTORY);
    let entries: Vec<Stamped> = list_stamped(saved_root.as_path());
    let oldest: &Stamped = entries.first()?;
    let copies: usize = entries.iter().map(|entry| { return walk(saved_root.join(entry.name.as_str()).as_path()).0.len(); }).sum();
    return Some(Report { bytes: entries.iter().map(|entry| { return entry.bytes; }).sum(), copies, saves: entries.len(), oldest: oldest.name.clone() });
}

/// The doctor's text for the folder: empty when it holds no copy, otherwise one block with the size, the count, the oldest
/// save and the limits that apply.
pub fn doctor_section(common_dir: &Path, limits: Limits) -> String {
    let Some(found) = report(common_dir) else {
        return String::new();
    };
    return format!(
        "git cli-git doctor: discarded copies: {} holds {} bytes in {} copies from {} saves; the oldest save is {}. Copies older than {} days \
         are removed, and the oldest are removed past {} bytes, never one younger than a day (git config cli-git.discardedMaxAgeDays and \
         cli-git.discardedMaxSize change the limits).\n",
        common_dir.join(SAVED_DIRECTORY).display(),
        found.bytes,
        found.copies,
        found.saves,
        found.oldest,
        limits.max_age_days,
        limits.max_bytes,
    );
}

/// Pruning is controlled in the test module.
#[cfg(test)]
#[path = "discarded_prune_tests.rs"]
mod tests;
