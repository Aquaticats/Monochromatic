//! What:
//!  The committed lock that pins the data revisions: two npm packages by version and sha-512 integrity,
//!  and the Firefox ESR value.
//! Why:
//!  The decision record keeps the data inputs under version control so a fresh machine produces the same output;
//!  the library only reads this lock and never picks versions.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type DataLock = { 'caniuse-lite': Pinned; 'node-releases': Pinned; firefoxEsr: string };
//! ```

use crate::error::{BrowserslistError, Outcome};
use crate::firefox_esr::parse_esr_major;
use crate::integrity::parse_sha512;
use serde_json::{Map, Value, json};

/// The lock format version this crate reads and writes.
const LOCK_VERSION: u64 = 1;

/// Build a lock error with a message.
fn lock_error(message: &str) -> BrowserslistError {
    return BrowserslistError::Data { file: "browserslist data lock".to_string(), offset: None, message: message.to_string() };
}

/// What:
///  One pinned npm package.
/// Why:
///  Name and version give the tarball URL; the integrity is what the download must hash to.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Pinned = { name: string; version: string; integrity: string };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PinnedPackage {
    /// The npm package name, `caniuse-lite` or `node-releases`.
    pub name: String,
    /// The exact published version.
    pub version: String,
    /// npm's `dist.integrity`, `sha512-<base64>`.
    pub integrity: String,
}

/// What:
///  The whole lock.
/// Why:
///  Resolution takes one value that fully determines which data it reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DataLock = { caniuseLite: Pinned; nodeReleases: Pinned; firefoxEsr: string };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLock {
    /// The caniuse-lite package.
    pub caniuse_lite: PinnedPackage,
    /// The node-releases package.
    pub node_releases: PinnedPackage,
    /// Mozilla's `FIREFOX_ESR` value, for example `153.6.0esr`.
    pub firefox_esr: String,
}

/// A package version must be safe to place in a URL path.
fn valid_version(version: &str) -> bool {
    return !version.is_empty()
        && version.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'.' || byte == b'-' || byte == b'_');
}

/// Read one pinned package out of the lock object.
fn read_package(root: &Map<String, Value>, name: &str) -> Outcome<PinnedPackage> {
    let Some(Value::Object(entry)) = root.get(name) else {
        return Err(lock_error(&format!("\"{name}\" is missing or not an object")));
    };
    if entry.len() != 2 {
        return Err(lock_error(&format!("\"{name}\" must hold exactly \"version\" and \"integrity\"")));
    }
    let (Some(Value::String(version)), Some(Value::String(integrity))) = (entry.get("version"), entry.get("integrity")) else {
        return Err(lock_error(&format!("\"{name}\" needs string \"version\" and \"integrity\" values")));
    };
    if !valid_version(version) {
        return Err(lock_error(&format!("\"{name}\" version {version:?} holds characters outside letters, digits, dot, dash and underscore")));
    }
    parse_sha512(integrity)?;
    return Ok(PinnedPackage { name: name.to_string(), version: version.clone(), integrity: integrity.clone() });
}

impl DataLock {
    /// What:
    ///  Parse and validate lock text; every field is checked, and unknown keys are refused.
    /// Why:
    ///  A hand-edited or truncated lock must fail loudly here, before any download is attempted.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function parseLock(text: string): DataLock;
    /// ```
    pub fn from_json(text: &str) -> Outcome<Self> {
        let parsed: Value = serde_json::from_str(text).map_err(|error| lock_error(&format!("not valid JSON: {error}")))?;
        let Value::Object(root) = parsed else {
            return Err(lock_error("the top level is not an object"));
        };
        if root.len() != 4 {
            return Err(lock_error("expected exactly lockVersion, caniuse-lite, node-releases and firefoxEsr"));
        }
        if root.get("lockVersion") != Some(&json!(LOCK_VERSION)) {
            return Err(lock_error(&format!("lockVersion must be {LOCK_VERSION}")));
        }
        let Some(Value::String(firefox_esr)) = root.get("firefoxEsr") else {
            return Err(lock_error("\"firefoxEsr\" is missing or not a string"));
        };
        parse_esr_major(firefox_esr)?;
        return Ok(Self {
            caniuse_lite: read_package(&root, "caniuse-lite")?,
            node_releases: read_package(&root, "node-releases")?,
            firefox_esr: firefox_esr.clone(),
        });
    }

    /// What:
    ///  Render the lock as pretty JSON with sorted keys and a final newline.
    /// Why:
    ///  The update command writes this; a stable layout keeps the lock's diff to the changed values.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function lockToJson(lock: DataLock): string;
    /// ```
    pub fn to_json(&self) -> String {
        let document = json!({
            "lockVersion": LOCK_VERSION,
            "caniuse-lite": { "version": self.caniuse_lite.version, "integrity": self.caniuse_lite.integrity },
            "node-releases": { "version": self.node_releases.version, "integrity": self.node_releases.integrity },
            "firefoxEsr": self.firefox_esr,
        });
        let mut text = serde_json::to_string_pretty(&document).expect("a JSON value always serializes");
        text.push('\n');
        return text;
    }
}

#[cfg(test)]
#[path = "lock_tests.rs"]
mod tests;
