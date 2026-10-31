//! What:
//!  The update step:
//! ask the registry for the newest `caniuse-lite` and `node-releases`,
//! ask Mozilla for the
//!  current `FIREFOX_ESR`,
//! prove the downloads usable,
//! and build the lock that pins them.
//! Why:
//!  The library's resolve path never picks versions;
//! only this step does,
//! and only when a person runs the
//!  update task,
//! so a version bump is always a reviewed lock diff.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! async function buildLock(source: DataSource, mozillaUrl: string): Promise<DataLock>;
//! ```

/// What:
///  Import `BrowserslistError, Outcome` from `crate::error`.
/// Why:
///  Failures use the crate's single error type,
/// so callers match one thing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { BrowserslistError, Outcome } from './error';
/// ```
use crate::error::{BrowserslistError, Outcome};
/// What:
///  Import `get_bytes` from `crate::fetch::get_bytes`.
/// Why:
///  Every download goes through the one bounded GET.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { getBytes } from 'crate/fetch';
/// ```
use crate::fetch::get_bytes;
/// What:
///  Import `parse_esr_major` from `crate::firefox_esr::parse_esr_major`.
/// Why:
///  The ESR value is validated in one place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { parseEsrMajor } from 'crate/firefox_esr';
/// ```
use crate::firefox_esr::parse_esr_major;
/// What:
///  Import `parse_sha512` from `crate::integrity::parse_sha512`.
/// Why:
///  Every download is checked against the digest the lock pins.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { parseSha512 } from 'crate/integrity';
/// ```
use crate::integrity::parse_sha512;
/// What:
///  Import `DataLock, PinnedPackage` from `crate::lock`.
/// Why:
///  The lock says which data revisions to use.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { DataLock, PinnedPackage } from './lock';
/// ```
use crate::lock::{DataLock, PinnedPackage};
/// What:
///  Import `DataSource, load_data` from `crate::source`.
/// Why:
///  Pinned data is loaded through the cache.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { DataSource, loadData } from './source';
/// ```
use crate::source::{DataSource, load_data};
/// What:
///  Import `Value` from `serde_json::Value`.
/// Why:
///  JSON documents are read and written through serde_json.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Value } from 'serde_json';
/// ```
use serde_json::Value;

/// Mozilla's product-details document that carries `FIREFOX_ESR`.
pub const MOZILLA_PRODUCT_DETAILS: &str = "https://product-details.mozilla.org/1.0/firefox_versions.json";

/// The largest metadata answer accepted.
const MAX_METADATA_BYTES: usize = 4 * 1024 * 1024;

/// Build a data error for an answer from a remote document.
fn answer_error(url: &str, message: &str) -> BrowserslistError {
    return BrowserslistError::Data { file: url.to_string(), offset: None, message: message.to_string() };
}

/// Fetch a URL and parse its body as a JSON object.
fn get_json_object(url: &str) -> Outcome<serde_json::Map<String, Value>> {
    let bytes = get_bytes(url, MAX_METADATA_BYTES)?;
    let parsed: Value = serde_json::from_slice(&bytes).map_err(|error| return answer_error(url, &format!("not valid JSON: {error}")))?;
    let Value::Object(object) = parsed else {
        return Err(answer_error(url, "the top level is not an object"));
    };
    return Ok(object);
}

/// What:
///  Ask the registry for the newest published version of a package and its integrity.
/// Why:
///  `GET <registry>/<name>/latest` answers with the version document npm itself installs from.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function latestPackage(registryUrl: string, name: string): Promise<PinnedPackage>;
/// ```
pub fn latest_package(registry_url: &str, name: &str) -> Outcome<PinnedPackage> {
    let url = format!("{registry_url}/{name}/latest");
    let document = get_json_object(&url)?;
    let Some(Value::String(version)) = document.get("version") else {
        return Err(answer_error(&url, "no string \"version\" field"));
    };
    let found_integrity = document.get("dist").and_then(|dist| return dist.get("integrity")).and_then(Value::as_str);
    let Some(integrity) = found_integrity else {
        return Err(answer_error(&url, "no \"dist.integrity\" string; a sha512 integrity is required"));
    };
    parse_sha512(integrity)?;
    return Ok(PinnedPackage { name: name.to_string(), version: version.clone(), integrity: integrity.to_string() });
}

/// What:
///  Read `FIREFOX_ESR` from Mozilla's product-details document and validate its shape.
/// Why:
///  A value not shaped `<major>.<minor>.<patch>esr` must stop the update with a diagnostic.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function latestFirefoxEsr(url: string): Promise<string>;
/// ```
pub fn latest_firefox_esr(mozilla_url: &str) -> Outcome<String> {
    let document = get_json_object(mozilla_url)?;
    let Some(Value::String(raw)) = document.get("FIREFOX_ESR") else {
        return Err(answer_error(mozilla_url, "no string \"FIREFOX_ESR\" field"));
    };
    parse_esr_major(raw)?;
    return Ok(raw.clone());
}

/// What:
///  Build a lock from the newest published data,
/// after downloading,
/// verifying and decoding it.
/// Why:
///  The decoded-data check proves the pinned tarballs are usable before the lock is ever written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function buildLock(source: DataSource, mozillaUrl: string): Promise<DataLock>;
/// ```
pub fn build_lock(source: &DataSource, mozilla_url: &str) -> Outcome<DataLock> {
    let lock = DataLock {
        caniuse_lite: latest_package(&source.registry_url, "caniuse-lite")?,
        node_releases: latest_package(&source.registry_url, "node-releases")?,
        firefox_esr: latest_firefox_esr(mozilla_url)?,
    };
    load_data(&lock, source)?;
    return Ok(lock);
}

/// What:
///  The unit tests of `update.rs`,
/// kept in the sidecar file `update_tests.rs`.
/// Why:
///  The repository keeps Rust tests beside the code they cover,
/// out of the source file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import './update.test';
/// ```
#[cfg(test)]
#[path = "update_tests.rs"]
mod tests;
