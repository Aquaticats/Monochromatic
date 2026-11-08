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
use std::collections::{BTreeMap, BTreeSet};
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
/// `use crate::csv::{DumpFormatError, column_indexes, read_csv};` imports `DumpFormatError`,
/// `column_indexes`,
/// `read_csv` from another module of this crate so this file can name them unqualified.
/// Why:
/// `DumpFormatError` is used by `dump_timestamp`,
/// `read_table`,
/// `fill_histories_from_dump`;
/// `column_indexes` is used by `read_table`;
/// `read_csv` is used by `read_table`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { DumpFormatError, column_indexes, read_csv } from 'crate/csv';
/// ```
use crate::csv::{DumpFormatError, column_indexes, read_csv};
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
        return Err(DumpFormatError { message: format!("the crates.io dump has a created_at value that is not a UTC timestamp: {value:?}") });
    }
    return Ok(iso);
}

/// Whether a history may hide a Trusted Publishing release the dump cannot show.
pub fn needs_api(history: &[VersionRecord]) -> bool {
    let start: i64 = parse_rfc3339_millis(TRUSTED_PUBLISHING_START).unwrap_or(0);
    for record in history {
        if record.publisher_id.is_empty() && parse_rfc3339_millis(&record.created_at).unwrap_or(0) >= start {
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
fn read_table(entry: &mut dyn Read, columns: &[&str], on_row: &mut dyn TableRow) -> Result<(), DumpFormatError> {
    let mut reader: TableReader = TableReader { columns, on_row, positions: None, failure: None, header_width: 0, row_number: 0 };
    let outcome = read_csv(entry, &mut reader);
    if let Some(error) = reader.failure {
        return Err(error);
    }
    return match outcome {
        Ok(found) => Ok(found),
        Err(error) => Err(DumpFormatError { message: format!("reading the crates.io dump failed: {error}") }),
    };
}

/// What:
/// `trait TableRow` is what `read_table` hands the wanted fields of each row to.
/// Why:
/// The reader takes the handler as a value instead of a closure,
/// so a handler can be a struct that holds its own state.
/// Every `FnMut(&[String]) -> Result<(), DumpFormatError>` is a `TableRow` too,
/// so a test passes a closure.
trait TableRow {
    /// What:
    /// `row(&mut self, fields)` takes the wanted fields of one row,
    /// in the order of the wanted columns.
    /// Why:
    /// An error stops the read and is returned by it.
    fn row(&mut self, fields: &[String]) -> Result<(), DumpFormatError>;
}

/// What:
/// The implementation of `TableRow` for every function with the handler's shape.
/// Why:
/// Callers that hold a closure need no wrapper.
impl<F: FnMut(&[String]) -> Result<(), DumpFormatError>> TableRow for F {
    /// What:
    /// Calls the function.
    /// Why:
    /// The function is the handler.
    fn row(&mut self, fields: &[String]) -> Result<(), DumpFormatError> {
        return self(fields);
    }
}

/// What:
/// `struct TableReader<'a>` is the row handler `read_table` gives to `read_csv`:
/// it finds the wanted columns in the header,
/// checks every row's width,
/// and passes the wanted fields on.
/// Why:
/// It holds the state that the closure that stood here captured.
struct TableReader<'a> {
    /// The names of the wanted columns.
    columns: &'a [&'a str],
    /// The handler of the wanted fields of each row.
    on_row: &'a mut dyn TableRow,
    /// The positions of the wanted columns,
    /// known once the header was read.
    positions: Option<Vec<usize>>,
    /// The first failure,
    /// which stops the read.
    failure: Option<DumpFormatError>,
    /// The header's field count.
    header_width: usize,
    /// The number of the row being read (the header is row 1).
    row_number: usize,
}

/// What:
/// The implementation of `RowHandler` for `TableReader`.
/// Why:
/// `read_csv` hands each finished row to it.
impl crate::csv::RowHandler for TableReader<'_> {
    /// What:
    /// Reads the header on the first row and checks and passes on every later row.
    /// Why:
    /// `false` stops the read after the first failure.
    fn on_row(&mut self, row: &[String]) -> bool {
        self.row_number += 1;
        match &self.positions {
            None => match column_indexes(row, self.columns) {
                Ok(found) => {
                    self.header_width = row.len();
                    self.positions = Some(found);
                }
                Err(error) => {
                    self.failure = Some(error);
                    return false;
                }
            },
            Some(found) => {
                if row.len() != self.header_width {
                    self.failure = Some(DumpFormatError {
                        message: format!("row {} of a crates.io dump table has {} fields, but its header has {}", self.row_number, row.len(), self.header_width),
                    });
                    return false;
                }
                let mut picked: Vec<String> = Vec::new();
                for index in found {
                    picked.push(row.get(*index).cloned().unwrap_or_default());
                }
                if let Err(error) = self.on_row.row(&picked) {
                    self.failure = Some(error);
                    return false;
                }
            }
        }
        return true;
    }
}

/// What:
/// `struct CrateNameRows<'a>` is the handler of the `crates.csv` rows:
/// it keeps the id of each wanted crate name.
/// Why:
/// It holds what the closure that stood here captured.
struct CrateNameRows<'a> {
    /// The crate names that are wanted.
    names: &'a BTreeSet<String>,
    /// The crate id of each wanted name,
    /// filled by the rows.
    id_to_name: &'a mut BTreeMap<String, String>,
}

/// What:
/// The implementation of `TableRow` for `CrateNameRows`.
/// Why:
/// `read_table` hands each row's id and name to it.
impl TableRow for CrateNameRows<'_> {
    /// What:
    /// Keeps the id of the row's crate when its name is wanted.
    /// Why:
    /// The versions table is read against these ids.
    fn row(&mut self, fields: &[String]) -> Result<(), DumpFormatError> {
        if self.names.contains(&fields[1]) {
            self.id_to_name.insert(fields[0].clone(), fields[1].clone());
        }
        return Ok(());
    }
}

/// What:
/// `struct VersionRows<'a>` is the handler of the `versions.csv` rows:
/// it appends each release of a wanted crate to that crate's history.
/// Why:
/// It holds what the closure that stood here captured.
struct VersionRows<'a> {
    /// The crate name of each wanted crate id.
    id_to_name: &'a BTreeMap<String, String>,
    /// The history of each wanted crate,
    /// filled by the rows.
    histories: &'a mut BTreeMap<String, Vec<VersionRecord>>,
}

/// What:
/// The implementation of `TableRow` for `VersionRows`.
/// Why:
/// `read_table` hands each row's wanted fields to it.
impl TableRow for VersionRows<'_> {
    /// What:
    /// Appends the release to the history of its crate when the crate is wanted,
    /// and refuses a crate with more releases than the bound.
    /// Why:
    /// The retained history of one crate is bounded.
    fn row(&mut self, fields: &[String]) -> Result<(), DumpFormatError> {
        let name: &String = match self.id_to_name.get(&fields[0]) {
            Some(found) => found,
            None => return Ok(()),
        };
        let kept: usize = self.histories.get(name).map(Vec::len).unwrap_or(0);
        if kept >= MAX_VERSIONS_PER_CRATE {
            return Err(DumpFormatError { message: format!("the crates.io dump lists more than {MAX_VERSIONS_PER_CRATE} releases of {name}") });
        }
        self.histories.entry(name.clone()).or_default().push(VersionRecord {
            num: fields[1].clone(),
            created_at: dump_timestamp(&fields[2])?,
            publisher_id: fields[3].clone(),
            trustpub_repository: String::new(),
            repository: fields[4].clone(),
        });
        return Ok(());
    }
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
fn path_failure(path: &Path, error: &std::io::Error) -> DumpFormatError {
    return DumpFormatError { message: format!("{}: {error}", path.display()) };
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
    let entries = match tar.entries() {
        Ok(found) => found,
        Err(error) => return Err(DumpFormatError { message: format!("the crates.io dump is not a tar archive: {error}") }),
    };
    for listed in entries {
        let mut entry = match listed {
            Ok(found) => found,
            Err(error) => return Err(DumpFormatError { message: format!("reading the crates.io dump failed: {error}") }),
        };
        let path: String = entry.path().map(entry_path_text).unwrap_or_default();
        if path.ends_with(CRATES_ENTRY) {
            read_table(&mut entry, &["id", "name"], &mut CrateNameRows { names, id_to_name: &mut id_to_name })?;
            crates_read = true;
        } else if path.ends_with(VERSIONS_ENTRY) {
            if !crates_read {
                return Err(DumpFormatError { message: "the crates.io dump lists versions.csv before crates.csv; the single-pass reader needs crates.csv first".to_string() });
            }
            read_table(&mut entry, &["crate_id", "num", "created_at", "published_by", "repository"], &mut VersionRows { id_to_name: &id_to_name, histories: &mut histories })?;
            versions_read = true;
        }
        if crates_read && versions_read {
            break;
        }
    }
    if !versions_read {
        return Err(DumpFormatError { message: "the crates.io dump ended before versions.csv was read completely".to_string() });
    }
    if let Err(error) = fs::create_dir_all(cache_dir) {
        return Err(path_failure(cache_dir, &error));
    }
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
        if let Err(error) = fs::write(&path, history_json(history)) {
            return Err(path_failure(&path, &error));
        }
        result.written += 1;
    }
    return Ok(result);
}
