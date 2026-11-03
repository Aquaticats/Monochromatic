//! What:
//! The crates.io lookups the policy needs: the sparse index for publish times and yanked flags, the web API for the full
//! version history and the owner list, with the history cached on disk.
//! Why:
//! A published version never changes, so a history that already lists every version being checked is read from
//! `.cache/cargo-shadow/crates-io/<name>.json` with no request. The API allows about one request per second, so requests
//! are spaced and an HTTP 429 is retried after its `Retry-After`, or a doubling backoff, at most five times.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // registry.ts
//! export type Registry = { pubtimes(name): Map<string, string>; yanked(name): Set<string>; history({ name, versions }): VersionRecord[]; owners(name): Set<string> };
//! export function createRegistry({ root, fetchText }): Registry
//! ```

use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::{Duration, Instant};

use serde_json::Value;

use crate::http::{RegistryError, TextFetcher};
use crate::policy::VersionRecord;

/// The sparse index host.
const SPARSE_INDEX_URL: &str = "https://index.crates.io";

/// The web API root.
const API_URL: &str = "https://crates.io/api/v1";

/// The least time between two API requests: crates.io allows one per second, plus a margin.
pub const API_MIN_INTERVAL: Duration = Duration::from_millis(1_100);

/// How many times a rate-limited request is tried.
const RATE_LIMIT_ATTEMPTS: u32 = 5;

/// The first backoff when a 429 carries no `Retry-After`; it doubles each attempt.
const RATE_LIMIT_BACKOFF: Duration = Duration::from_secs(30);

/// The HTTP status for "too many requests".
const TOO_MANY_REQUESTS: u16 = 429;

/// Where histories are cached, relative to the repository root.
pub const HISTORY_CACHE_DIR: &str = ".cache/cargo-shadow/crates-io";

/// What:
/// `pub trait Registry { ... }` is what the checks ask of crates.io.
/// Why:
/// Tests give the checks a fake with canned histories.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface Registry { pubtimes(name): Map<string, string>; yanked(name): Set<string>; history(args): VersionRecord[]; owners(name): Set<string> }
/// ```
pub trait Registry {
    /// Publish time (RFC 3339) of each version of a crate, from the sparse index; empty when the index has none.
    fn pubtimes(&self, name: &str) -> Result<BTreeMap<String, String>, RegistryError>;
    /// The yanked versions of a crate.
    fn yanked(&self, name: &str) -> Result<BTreeSet<String>, RegistryError>;
    /// Every release of a crate; read from the cache when it lists all of `versions`.
    fn history(&self, name: &str, versions: &[String]) -> Result<Vec<VersionRecord>, RegistryError>;
    /// The account ids of the crate's listed individual owners.
    fn owners(&self, name: &str) -> Result<BTreeSet<String>, RegistryError>;
}

/// The sparse index path of a crate: `1/a`, `2/ab`, `3/a/abc`, `ab/cd/abcdef`.
pub fn sparse_index_path(name: &str) -> String {
    let lower: String = name.to_lowercase();
    if lower.len() <= 2 {
        return format!("{}/{lower}", lower.len());
    }
    if lower.len() == 3 {
        return format!("3/{}/{lower}", &lower[..1]);
    }
    return format!("{}/{}/{lower}", &lower[..2], &lower[2..4]);
}

/// Reads a string field, empty when absent or not a string.
fn string_field(object: &Value, key: &str) -> String {
    return object.get(key).and_then(|value| { return value.as_str(); }).unwrap_or("").to_string();
}

/// Reads an id that crates.io writes as a number and the cache writes as text.
fn id_field(object: &Value, key: &str) -> String {
    return match object.get(key) {
        Some(Value::Number(number)) => number.to_string(),
        Some(Value::String(text)) => text.clone(),
        _ => String::new(),
    };
}

/// The sparse index lines as JSON values; a line that is not JSON is an error naming the crate.
fn index_lines(text: &str) -> Result<Vec<Value>, RegistryError> {
    let mut records: Vec<Value> = Vec::new();
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let record: Value = serde_json::from_str(line)
            .map_err(|error| { return RegistryError { message: format!("sparse index line: {error}"), status: None, retry_after_seconds: 0 }; })?;
        records.push(record);
    }
    return Ok(records);
}

/// Publish times by version from sparse index text.
pub fn parse_index_pubtimes(text: &str) -> Result<BTreeMap<String, String>, RegistryError> {
    let mut times: BTreeMap<String, String> = BTreeMap::new();
    for record in index_lines(text)? {
        if record.get("vers").and_then(|value| { return value.as_str(); }).is_some() {
            times.insert(string_field(&record, "vers"), string_field(&record, "pubtime"));
        }
    }
    return Ok(times);
}

/// Yanked versions from sparse index text.
pub fn parse_index_yanked(text: &str) -> Result<BTreeSet<String>, RegistryError> {
    let mut yanked: BTreeSet<String> = BTreeSet::new();
    for record in index_lines(text)? {
        if record.get("yanked").and_then(|value| { return value.as_bool(); }) == Some(true) {
            yanked.insert(string_field(&record, "vers"));
        }
    }
    return Ok(yanked);
}

/// The failure for a body that is not the JSON the API documents.
fn body_error(what: &str, error: impl std::fmt::Display) -> RegistryError {
    return RegistryError { message: format!("{what}: {error}"), status: None, retry_after_seconds: 0 };
}

/// What:
/// `pub fn parse_version_history(text: &str) -> Result<Vec<VersionRecord>, RegistryError>` reads the API's
/// `/crates/<name>/versions` body.
/// Why:
/// Only the fields the trust rule reads are kept: number, publish time, publishing account, Trusted Publishing repository
/// and declared repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function parseVersionHistory(text: string): VersionRecord[]
/// ```
pub fn parse_version_history(text: &str) -> Result<Vec<VersionRecord>, RegistryError> {
    let body: Value = serde_json::from_str(text).map_err(|error| { return body_error("version list", error); })?;
    let mut records: Vec<VersionRecord> = Vec::new();
    let empty: Vec<Value> = Vec::new();
    for version in body.get("versions").and_then(|value| { return value.as_array(); }).unwrap_or(&empty) {
        let publisher: Value = version.get("published_by").cloned().unwrap_or(Value::Null);
        let trustpub: Value = version.get("trustpub_data").cloned().unwrap_or(Value::Null);
        records.push(VersionRecord {
            num: string_field(version, "num"),
            created_at: string_field(version, "created_at"),
            publisher_id: id_field(&publisher, "id"),
            trustpub_repository: string_field(&trustpub, "repository"),
            repository: string_field(version, "repository"),
        });
    }
    return Ok(records);
}

/// The account ids of listed individual owners (`kind: user`) from the API's owners body.
pub fn parse_owner_ids(text: &str) -> Result<BTreeSet<String>, RegistryError> {
    let body: Value = serde_json::from_str(text).map_err(|error| { return body_error("owner list", error); })?;
    let mut ids: BTreeSet<String> = BTreeSet::new();
    let empty: Vec<Value> = Vec::new();
    for user in body.get("users").and_then(|value| { return value.as_array(); }).unwrap_or(&empty) {
        if string_field(user, "kind") == "user" {
            let id: String = id_field(user, "id");
            if !id.is_empty() {
                ids.insert(id);
            }
        }
    }
    return Ok(ids);
}

/// The cache file format: the history as a JSON array with camelCase keys.
fn history_to_json(history: &[VersionRecord]) -> Value {
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
    return Value::Array(entries);
}

/// Reads a cached history; a missing or unreadable cache file is an empty history (the API then fills it).
fn read_cached_history(path: &Path) -> Vec<VersionRecord> {
    let text: String = match fs::read_to_string(path) {
        Ok(found) => found,
        Err(_) => return Vec::new(),
    };
    let parsed: Value = match serde_json::from_str(&text) {
        Ok(found) => found,
        Err(_) => return Vec::new(),
    };
    let mut records: Vec<VersionRecord> = Vec::new();
    for entry in parsed.as_array().unwrap_or(&Vec::new()) {
        records.push(VersionRecord {
            num: string_field(entry, "num"),
            created_at: string_field(entry, "createdAt"),
            publisher_id: string_field(entry, "publisherId"),
            trustpub_repository: string_field(entry, "trustpubRepository"),
            repository: string_field(entry, "repository"),
        });
    }
    return records;
}

/// crates.io through a fetcher, with spaced API requests and the on-disk history cache.
pub struct CratesIo {
    cache_dir: PathBuf,
    fetcher: Box<dyn TextFetcher>,
    min_interval: Duration,
    backoff: Duration,
    last_request: Cell<Option<Instant>>,
}

impl CratesIo {
    /// Builds the registry for a repository root with the real spacing and backoff.
    pub fn with_defaults(root: &Path, fetcher: Box<dyn TextFetcher>) -> CratesIo {
        return CratesIo::new(root, fetcher, API_MIN_INTERVAL, RATE_LIMIT_BACKOFF);
    }

    /// Builds the registry for a repository root; `min_interval` and `backoff` are parameters so tests do not wait.
    pub fn new(root: &Path, fetcher: Box<dyn TextFetcher>, min_interval: Duration, backoff: Duration) -> CratesIo {
        return CratesIo {
            cache_dir: root.join(HISTORY_CACHE_DIR),
            fetcher,
            min_interval,
            backoff,
            last_request: Cell::new(None),
        };
    }

    /// One API GET: waits out the spacing, then retries an HTTP 429 up to the attempt limit.
    fn spaced_api_get(&self, url: &str) -> Result<String, RegistryError> {
        let mut attempt: u32 = 1;
        loop {
            if let Some(last) = self.last_request.get() {
                let elapsed: Duration = last.elapsed();
                if elapsed < self.min_interval {
                    sleep(self.min_interval - elapsed);
                }
            }
            self.last_request.set(Some(Instant::now()));
            eprintln!("cargo-shadow: fetching {url}");
            match self.fetcher.get(url) {
                Ok(body) => return Ok(body),
                Err(error) => {
                    if error.status != Some(TOO_MANY_REQUESTS) || attempt >= RATE_LIMIT_ATTEMPTS {
                        return Err(error);
                    }
                    let pause: Duration = if error.retry_after_seconds > 0 {
                        Duration::from_secs(error.retry_after_seconds)
                    } else {
                        self.backoff * 2u32.pow(attempt - 1)
                    };
                    eprintln!("cargo-shadow: {url} answered HTTP 429 (attempt {attempt} of {RATE_LIMIT_ATTEMPTS}); retrying in {pause:?}");
                    sleep(pause);
                    attempt += 1;
                }
            }
        }
    }
}

impl Registry for CratesIo {
    fn pubtimes(&self, name: &str) -> Result<BTreeMap<String, String>, RegistryError> {
        return parse_index_pubtimes(&self.fetcher.get(&format!("{SPARSE_INDEX_URL}/{}", sparse_index_path(name)))?);
    }

    fn yanked(&self, name: &str) -> Result<BTreeSet<String>, RegistryError> {
        return parse_index_yanked(&self.fetcher.get(&format!("{SPARSE_INDEX_URL}/{}", sparse_index_path(name)))?);
    }

    fn owners(&self, name: &str) -> Result<BTreeSet<String>, RegistryError> {
        return parse_owner_ids(&self.spaced_api_get(&format!("{API_URL}/crates/{name}/owners"))?);
    }

    fn history(&self, name: &str, versions: &[String]) -> Result<Vec<VersionRecord>, RegistryError> {
        let path: PathBuf = self.cache_dir.join(format!("{name}.json"));
        let cached: Vec<VersionRecord> = read_cached_history(&path);
        let known: BTreeSet<&str> = cached.iter().map(|record| { return record.num.as_str(); }).collect();
        if !cached.is_empty() && versions.iter().all(|version| { return known.contains(version.as_str()); }) {
            return Ok(cached);
        }
        let fresh: Vec<VersionRecord> = parse_version_history(&self.spaced_api_get(&format!("{API_URL}/crates/{name}/versions"))?)?;
        if fs::create_dir_all(&self.cache_dir).is_ok() {
            let _ = fs::write(&path, format!("{}\n", history_to_json(&fresh)));
        }
        return Ok(fresh);
    }
}
