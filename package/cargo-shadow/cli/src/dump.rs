//! What:
//! Fills the on-disk version-history cache from the crates.io database dump,
//! in one streaming pass over the tar.gz.
//! Why:
//! crates.io asks bulk readers to use `https://static.crates.io/db-dump.tar.gz`,
//! which has no rate limit;
//! reading the histories of
//! a thousand crates from the API at one request per 1.1 seconds takes tens of minutes,
//! the dump about half a minute plus the
//! download.
//! The reader reads only `crates.csv` and `versions.csv` and stops before the later tables.
//! The dump keeps every
//! Trusted Publishing column private and a Trusted Publishing release has no publisher,
//! so a crate with a publisher-less release
//! since 2025-07-01 is left to the API.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // dump.ts
//! export async function fillHistoriesFromDump({ tarChunks, names, cacheDir }): Promise<DumpFillResult>
//! ```

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Read;
use std::path::Path;

use flate2::read::GzDecoder;
use serde_json::Value;
use tar::Archive;

use crate::csv::{DumpFormatError, column_indexes, read_csv};
use crate::policy::VersionRecord;
use crate::time_text::parse_rfc3339_millis;

/// The first moment Trusted Publishing existed on crates.io;
/// a publisher-less release since then may be a trusted one.
pub const TRUSTED_PUBLISHING_START: &str = "2025-07-01T00:00:00Z";

/// The tail of the entry path of the crates table.
const CRATES_ENTRY: &str = "/data/crates.csv";

/// The tail of the entry path of the versions table.
const VERSIONS_ENTRY: &str = "/data/versions.csv";

/// What a dump fill did.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DumpFillResult {
    /// Crates of the wanted names that the dump lists.
    pub found: usize,
    /// Histories written to the cache.
    pub written: usize,
    /// Crates left to the API.
    pub left_to_api: Vec<String>,
    /// Histories that were already cached and kept.
    pub kept: usize,
}

/// A dump timestamp (`2026-10-29 12:00:00.123456+00`) as RFC 3339,
/// or an error when it is not UTC.
pub fn dump_timestamp(value: &str) -> Result<String, DumpFormatError> {
    let mut parts = value.splitn(2, ' ');
    let date: &str = parts.next().unwrap_or("");
    let time: &str = parts.next().unwrap_or("");
    let iso: String = match time.strip_suffix("+00") {
        Some(without) => format!("{date}T{without}Z"),
        None => value.to_string(),
    };
    if parse_rfc3339_millis(&iso).is_none() {
        return Err(DumpFormatError { message: format!("the crates.io dump has a created_at value that is not a UTC timestamp: {value:?}") });
    }
    return Ok(iso);
}

/// Whether a history may hide a Trusted Publishing release the dump cannot show.
pub fn needs_api(history: &[VersionRecord]) -> bool {
    let start: i64 = parse_rfc3339_millis(TRUSTED_PUBLISHING_START).unwrap_or(0);
    return history.iter().any(|record| { return record.publisher_id.is_empty() && parse_rfc3339_millis(&record.created_at).unwrap_or(0) >= start; });
}

/// One history as the cache file format.
fn history_json(history: &[VersionRecord]) -> String {
    let mut entries: Vec<Value> = Vec::new();
    for record in history {
        let mut entry: serde_json::Map<String, Value> = serde_json::Map::new();
        entry.insert("num".to_string(), Value::String(record.num.clone()));
        entry.insert("createdAt".to_string(), Value::String(record.created_at.clone()));
        entry.insert("publisherId".to_string(), Value::String(record.publisher_id.clone()));
        entry.insert("trustpubRepository".to_string(), Value::String(record.trustpub_repository.clone()));
        entry.insert("repository".to_string(), Value::String(record.repository.clone()));
        entries.push(Value::Object(entry));
    }
    return format!("{}\n", Value::Array(entries));
}

/// Reads one table of the archive into rows of the wanted columns;
/// the callback gets the row's wanted fields in order.
fn read_table(entry: &mut dyn Read, columns: &[&str], on_row: &mut dyn FnMut(&[String]) -> Result<(), DumpFormatError>) -> Result<(), DumpFormatError> {
    let mut positions: Option<Vec<usize>> = None;
    let mut failure: Option<DumpFormatError> = None;
    let outcome = read_csv(entry, &mut |row: &[String]| {
        match &positions {
            None => match column_indexes(row, columns) {
                Ok(found) => positions = Some(found),
                Err(error) => {
                    failure = Some(error);
                    return false;
                }
            },
            Some(found) => {
                let picked: Vec<String> = found.iter().map(|index| { return row.get(*index).cloned().unwrap_or_default(); }).collect();
                if let Err(error) = on_row(&picked) {
                    failure = Some(error);
                    return false;
                }
            }
        }
        return true;
    });
    if let Some(error) = failure {
        return Err(error);
    }
    return outcome.map_err(|error| { return DumpFormatError { message: format!("reading the crates.io dump failed: {error}") }; });
}

/// What:
/// `pub fn fill_histories_from_dump(archive: Box<dyn Read>, names: &BTreeSet<String>, cache_dir: &Path) -> Result<DumpFillResult, DumpFormatError>`
/// reads the gzip tar stream and writes a cache file for each wanted crate whose history the dump shows completely.
/// Why:
/// `crates.csv` comes first in the dump and maps ids to wanted names;
/// `versions.csv` then yields the histories.
/// A crate with a
/// possibly trusted release is left to the API,
/// and an existing cache file is never overwritten.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export async function fillHistoriesFromDump({ tarChunks, names, cacheDir }): Promise<DumpFillResult>
/// ```
pub fn fill_histories_from_dump(archive: Box<dyn Read>, names: &BTreeSet<String>, cache_dir: &Path) -> Result<DumpFillResult, DumpFormatError> {
    let mut id_to_name: BTreeMap<String, String> = BTreeMap::new();
    let mut histories: BTreeMap<String, Vec<VersionRecord>> = BTreeMap::new();
    let mut crates_read: bool = false;
    let mut versions_read: bool = false;
    let mut tar: Archive<GzDecoder<Box<dyn Read>>> = Archive::new(GzDecoder::new(archive));
    let entries = tar.entries().map_err(|error| { return DumpFormatError { message: format!("the crates.io dump is not a tar archive: {error}") }; })?;
    for listed in entries {
        let mut entry = listed.map_err(|error| { return DumpFormatError { message: format!("reading the crates.io dump failed: {error}") }; })?;
        let path: String = entry.path().map(|found| { return found.to_string_lossy().to_string(); }).unwrap_or_default();
        if path.ends_with(CRATES_ENTRY) {
            read_table(&mut entry, &["id", "name"], &mut |row: &[String]| {
                if names.contains(&row[1]) {
                    id_to_name.insert(row[0].clone(), row[1].clone());
                }
                return Ok(());
            })?;
            crates_read = true;
        } else if path.ends_with(VERSIONS_ENTRY) {
            if !crates_read {
                return Err(DumpFormatError { message: "the crates.io dump lists versions.csv before crates.csv; the single-pass reader needs crates.csv first".to_string() });
            }
            read_table(&mut entry, &["crate_id", "num", "created_at", "published_by", "repository"], &mut |row: &[String]| {
                let name: &String = match id_to_name.get(&row[0]) {
                    Some(found) => found,
                    None => return Ok(()),
                };
                histories.entry(name.clone()).or_default().push(VersionRecord {
                    num: row[1].clone(),
                    created_at: dump_timestamp(&row[2])?,
                    publisher_id: row[3].clone(),
                    trustpub_repository: String::new(),
                    repository: row[4].clone(),
                });
                return Ok(());
            })?;
            versions_read = true;
        }
        if crates_read && versions_read {
            break;
        }
    }
    if !versions_read {
        return Err(DumpFormatError { message: "the crates.io dump ended before versions.csv was read completely".to_string() });
    }
    fs::create_dir_all(cache_dir).map_err(|error| { return DumpFormatError { message: format!("{}: {error}", cache_dir.display()) }; })?;
    let mut result: DumpFillResult = DumpFillResult { found: histories.len(), ..DumpFillResult::default() };
    for (name, history) in &histories {
        if needs_api(history) {
            result.left_to_api.push(name.clone());
            continue;
        }
        let path = cache_dir.join(format!("{name}.json"));
        if path.exists() {
            result.kept += 1;
            continue;
        }
        fs::write(&path, history_json(history)).map_err(|error| { return DumpFormatError { message: format!("{}: {error}", path.display()) }; })?;
        result.written += 1;
    }
    return Ok(result);
}
