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

/// What:
/// `use std::collections::{BTreeMap, BTreeSet};` imports `BTreeMap`,
/// `BTreeSet` from the standard library so this file can name them unqualified.
/// Why:
/// `BTreeMap` is used by `fill_histories_from_dump`;
/// `BTreeSet` is used by `fill_histories_from_dump`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { BTreeMap, BTreeSet } from 'std/collections';
/// ```
use std::collections::{
    BTreeMap,
    BTreeSet,
};
/// What:
/// `use std::fs;` imports `fs` from the standard library so this file can name it unqualified.
/// Why:
/// `fs` is used by `fill_histories_from_dump`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { fs } from 'std';
/// ```
use std::fs;
/// What:
/// `use std::io::Read;` imports `Read` from the standard library so this file can name it unqualified.
/// Why:
/// `Read` is used by `read_table`,
/// `fill_histories_from_dump`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Read } from 'std/io';
/// ```
use std::io::Read;
/// What:
/// `use std::path::Path;` imports `Path` from the standard library so this file can name it unqualified.
/// Why:
/// `Path` is used by `fill_histories_from_dump`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Path } from 'std/path';
/// ```
use std::path::Path;

/// What:
/// `use flate2::read::GzDecoder;` imports `GzDecoder` from the external crate `flate2` so this file can name it unqualified.
/// Why:
/// `GzDecoder` is used by `fill_histories_from_dump`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { GzDecoder } from 'flate2/read';
/// ```
use flate2::read::GzDecoder;
/// What:
/// `use serde_json::Value;` imports `Value` from the external crate `serde_json` so this file can name it unqualified.
/// Why:
/// `Value` is used by `history_json`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Value } from 'serde_json';
/// ```
use serde_json::Value;
/// What:
/// `use tar::Archive;` imports `Archive` from the external crate `tar` so this file can name it unqualified.
/// Why:
/// `Archive` is used by `fill_histories_from_dump`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Archive } from 'tar';
/// ```
use tar::Archive;

/// What:
/// `use crate::csv::DumpFormatError;` imports `DumpFormatError` from another module of this crate so this file can name it unqualified.
/// Why:
/// `DumpFormatError` is used by `dump_timestamp`,
/// `fill_histories_from_dump`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { DumpFormatError } from 'crate/csv';
/// ```
use crate::csv::DumpFormatError;
/// What:
/// `use crate::policy::VersionRecord;` imports `VersionRecord` from another module of this crate so this file can name it unqualified.
/// Why:
/// `VersionRecord` is used by `needs_api`,
/// `history_json`,
/// `fill_histories_from_dump`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { VersionRecord } from 'crate/policy';
/// ```
use crate::policy::VersionRecord;
/// What:
/// `use crate::time_text::parse_rfc3339_millis;` imports `parse_rfc3339_millis` from another module of this crate so this file can name it unqualified.
/// Why:
/// `parse_rfc3339_millis` is used by `dump_timestamp`,
/// `needs_api`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { parse_rfc3339_millis } from 'crate/time_text';
/// ```
use crate::time_text::parse_rfc3339_millis;

/// Reads one table of the crates.io database dump.
mod table_reading;

/// Imports the table reading of the dump,
/// which lives in its own file (split out of this module).
use self::table_reading::{
    CrateNameRows,
    VersionRows,
    read_table,
};

/// The first moment Trusted Publishing existed on crates.io;
/// a publisher-less release since then may be a trusted one.
pub const TRUSTED_PUBLISHING_START: &str = "2025-07-01T00:00:00Z";

/// The tail of the entry path of the crates table.
const CRATES_ENTRY: &str = "/data/crates.csv";

/// The tail of the entry path of the versions table.
const VERSIONS_ENTRY: &str = "/data/versions.csv";

/// The most releases kept for one crate while the versions table streams past.
/// A chosen ceiling far above the release count of any crate on crates.io (a guess,
/// not measured on a real dump);
/// the retained history is bounded by this times the number of crates asked for.
/// Delegated pick under the owner's "Pick options as if I had picked them and invite veto when I say I'm back"
/// (handover "Picks made under the owner's delegation"),
/// veto open;
/// not a measurement.
pub const MAX_VERSIONS_PER_CRATE: usize = 100_000;

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
        return Err(DumpFormatError {
            message: format!(
                "the crates.io dump has a created_at value that is not a UTC timestamp: {value:?}"
            ),
        });
    }
    return Ok(iso);
}

/// Whether a history may hide a Trusted Publishing release the dump cannot show.
pub fn needs_api(history: &[VersionRecord]) -> bool {
    let start: i64 = parse_rfc3339_millis(TRUSTED_PUBLISHING_START).unwrap_or(0);
    for record in history {
        if record.publisher_id.is_empty()
            && parse_rfc3339_millis(&record.created_at).unwrap_or(0) >= start
        {
            return true;
        }
    }
    return false;
}

/// One history as the cache file format.
fn history_json(history: &[VersionRecord]) -> String {
    let mut entries: Vec<Value> = Vec::new();
    for record in history {
        let mut entry: serde_json::Map<String, Value> = serde_json::Map::new();
        entry.insert("num".to_string(), Value::String(record.num.clone()));
        entry.insert(
            "createdAt".to_string(),
            Value::String(record.created_at.clone()),
        );
        entry.insert(
            "publisherId".to_string(),
            Value::String(record.publisher_id.clone()),
        );
        entry.insert(
            "trustpubRepository".to_string(),
            Value::String(record.trustpub_repository.clone()),
        );
        entry.insert(
            "repository".to_string(),
            Value::String(record.repository.clone()),
        );
        entries.push(Value::Object(entry));
    }
    return format!("{}\n", Value::Array(entries));
}

/// What:
/// `fn entry_path_text(path: Cow<Path>) -> String` turns the path of one archive entry into text,
/// replacing bytes that are not valid UTF-8.
/// Why:
/// The dump reader picks the tables it needs by the end of each entry's path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const entryPathText = (path: string) => path;
/// ```
fn entry_path_text(path: std::borrow::Cow<'_, Path>) -> String {
    return path.to_string_lossy().to_string();
}

/// What:
/// `fn path_failure(path: &Path, error: &io::Error) -> DumpFormatError` words a file-system failure with the path it happened at.
/// Why:
/// A cache directory or file that cannot be written is named in the error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pathFailure = (path: string, error: Error) => new DumpFormatError(`${path}: ${error.message}`);
/// ```
fn path_failure(
    path: &Path,
    error: &std::io::Error,
) -> DumpFormatError {
    return DumpFormatError {
        message: format!("{}: {error}", path.display()),
    };
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
pub fn fill_histories_from_dump(
    archive: Box<dyn Read>,
    names: &BTreeSet<String>,
    cache_dir: &Path,
) -> Result<DumpFillResult, DumpFormatError> {
    let mut id_to_name: BTreeMap<String, String> = BTreeMap::new();
    let mut histories: BTreeMap<String, Vec<VersionRecord>> = BTreeMap::new();
    let mut crates_read: bool = false;
    let mut versions_read: bool = false;
    let mut tar: Archive<GzDecoder<Box<dyn Read>>> = Archive::new(GzDecoder::new(archive));
    let entries = match tar.entries() {
        Ok(found) => found,
        Err(error) => {
            return Err(DumpFormatError {
                message: format!("the crates.io dump is not a tar archive: {error}"),
            });
        }
    };
    for listed in entries {
        let mut entry = match listed {
            Ok(found) => found,
            Err(error) => {
                return Err(DumpFormatError {
                    message: format!("reading the crates.io dump failed: {error}"),
                });
            }
        };
        let path: String = entry.path().map(entry_path_text).unwrap_or_default();
        if path.ends_with(CRATES_ENTRY) {
            read_table(
                &mut entry,
                &["id", "name"],
                &mut CrateNameRows {
                    names,
                    id_to_name: &mut id_to_name,
                },
            )?;
            crates_read = true;
        } else if path.ends_with(VERSIONS_ENTRY) {
            if !crates_read {
                return Err(DumpFormatError { message: "the crates.io dump lists versions.csv before crates.csv; the single-pass reader needs crates.csv first".to_string() });
            }
            read_table(
                &mut entry,
                &[
                    "crate_id",
                    "num",
                    "created_at",
                    "published_by",
                    "repository",
                ],
                &mut VersionRows {
                    id_to_name: &id_to_name,
                    histories: &mut histories,
                },
            )?;
            versions_read = true;
        }
        if crates_read && versions_read {
            break;
        }
    }
    if !versions_read {
        return Err(DumpFormatError {
            message: "the crates.io dump ended before versions.csv was read completely".to_string(),
        });
    }
    if let Err(error) = fs::create_dir_all(cache_dir) {
        return Err(path_failure(cache_dir, &error));
    }
    let mut result: DumpFillResult = DumpFillResult {
        found: histories.len(),
        ..DumpFillResult::default()
    };
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
        if let Err(error) = fs::write(&path, history_json(history)) {
            return Err(path_failure(&path, &error));
        }
        result.written += 1;
    }
    return Ok(result);
}

/// What:
/// `#[cfg(test)] mod dump_tests;` declares the tests in `src/dump_tests.rs`;
/// the attribute compiles it only for test builds (a TS test file that production code never imports).
/// Why:
/// Its header says:
/// "The unit tests of the CSV reader and the dump fill,"
/// so the behavior of `dump` is pinned where a change shows up as a red test.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // dump_tests.test.ts, run by the test runner only
/// ```
/// `#[path]` keeps its file beside this one and named for what it holds,
/// so each module declares its own controls and the crate root's list stays a list of subjects.
#[cfg(test)]
#[path = "dump_tests.rs"]
mod dump_tests;
