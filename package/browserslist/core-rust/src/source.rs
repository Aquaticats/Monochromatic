//! What:
//!  Turns a lock into decoded `BrowserData`: read the pinned tarballs from the per-user cache,
//!  fetch exactly the pinned ones that are missing, verify every byte against the lock, and decode the data files.
//! Why:
//!  A fresh machine must produce the same output as a warm one,
//!  and nothing unpinned is ever fetched or trusted at resolve time.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! async function loadData(lock: DataLock, source: DataSource): Promise<BrowserData>;
//! ```

use crate::data::{BrowserData, DataFiles};
use crate::error::{BrowserslistError, Outcome};
use crate::fetch::get_bytes;
use crate::integrity::{cache_key, verify};
use crate::lock::{DataLock, PinnedPackage};
use crate::tarball::extract_files;
use directories::ProjectDirs;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// The registry the crate fetches from unless told otherwise.
pub const NPM_REGISTRY: &str = "https://registry.npmjs.org";

/// The largest tarball accepted (the real caniuse-lite tarball is about 350 KiB).
const MAX_TARBALL_BYTES: usize = 64 * 1024 * 1024;

/// Distinguishes temporary file names within one process.
static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// What:
///  Where the cache lives and which registry serves missing tarballs.
/// Why:
///  Tests point both at disposable locations; production uses the user cache directory and npm.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type DataSource = { cacheDir: string; registryUrl: string };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataSource {
    /// Directory holding verified tarballs named `<hex digest>.tgz`.
    pub cache_dir: PathBuf,
    /// Registry base URL without a trailing slash.
    pub registry_url: String,
}

impl DataSource {
    /// What:
    ///  The production source: the platform user cache directory for `monochromatic-browserslist`, and npm.
    /// Why:
    ///  Callers that have no reason to redirect the cache use this.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function userDefault(): DataSource;
    /// ```
    pub fn user_default() -> Outcome<Self> {
        let Some(directories) = ProjectDirs::from("dev", "monochromatic", "browserslist") else {
            return Err(BrowserslistError::Io {
                path: "user cache directory".to_string(),
                message: "the operating system reports no home directory".to_string(),
            });
        };
        return Ok(Self { cache_dir: directories.cache_dir().to_path_buf(), registry_url: NPM_REGISTRY.to_string() });
    }
}

/// Build an I/O error for a path.
fn io_error(path: &Path, error: &std::io::Error) -> BrowserslistError {
    return BrowserslistError::Io { path: path.display().to_string(), message: error.to_string() };
}

/// The tarball URL npm uses for a package version: `<registry>/<name>/-/<name>-<version>.tgz`.
pub fn tarball_url(registry_url: &str, package: &PinnedPackage) -> String {
    return format!("{registry_url}/{name}/-/{name}-{version}.tgz", name = package.name, version = package.version);
}

/// Write bytes to `path` through a sibling temporary file and a rename, so readers never see a partial file.
pub fn write_atomically(path: &Path, bytes: &[u8]) -> Outcome<()> {
    let Some(parent) = path.parent() else {
        return Err(BrowserslistError::Io { path: path.display().to_string(), message: "has no parent directory".to_string() });
    };
    fs::create_dir_all(parent).map_err(|error| io_error(parent, &error))?;
    let counter = TEMP_COUNTER.fetch_add(1, Ordering::SeqCst);
    let temporary = parent.join(format!(".tmp-{}-{counter}", std::process::id()));
    fs::write(&temporary, bytes).map_err(|error| io_error(&temporary, &error))?;
    return fs::rename(&temporary, path).map_err(|error| io_error(path, &error));
}

/// What:
///  The verified tarball bytes of a pinned package: from the cache when the cached file still hashes correctly,
///  otherwise fetched from the pinned URL, verified, and cached.
/// Why:
///  A corrupted or tampered cache entry is replaced, never trusted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function pinnedTarball(pkg: PinnedPackage, source: DataSource): Promise<Uint8Array>;
/// ```
pub fn pinned_tarball(package: &PinnedPackage, source: &DataSource) -> Outcome<Vec<u8>> {
    let path = source.cache_dir.join(format!("{}.tgz", cache_key(&package.integrity)?));
    let subject = format!("{} {}", package.name, package.version);
    if let Ok(cached) = fs::read(&path) {
        match verify(&subject, &package.integrity, &cached) {
            Ok(()) => {
                tracing::debug!(%subject, path = %path.display(), "cache hit");
                return Ok(cached);
            }
            Err(error) => tracing::warn!(%subject, %error, "cached tarball failed verification; fetching again"),
        }
    }
    let url = tarball_url(&source.registry_url, package);
    tracing::info!(%subject, %url, "cache miss; fetching the pinned tarball");
    let fetched = get_bytes(&url, MAX_TARBALL_BYTES)?;
    verify(&subject, &package.integrity, &fetched)?;
    write_atomically(&path, &fetched)?;
    return Ok(fetched);
}

/// Turn the two tarballs plus the lock's ESR value into decoded data.
pub fn decode_tarballs(caniuse_tgz: &[u8], node_releases_tgz: &[u8], firefox_esr: &str) -> Outcome<BrowserData> {
    let caniuse = extract_files(
        "caniuse-lite",
        caniuse_tgz,
        &["package/data/agents.js", "package/data/browsers.js", "package/data/browserVersions.js"],
    )?;
    let node = extract_files("node-releases", node_releases_tgz, &["package/data/processed/envs.json"])?;
    let agents_js = text_of("caniuse-lite data/agents.js", &caniuse[0])?;
    let browsers_js = text_of("caniuse-lite data/browsers.js", &caniuse[1])?;
    let browser_versions_js = text_of("caniuse-lite data/browserVersions.js", &caniuse[2])?;
    let envs_json = text_of("node-releases data/processed/envs.json", &node[0])?;
    return BrowserData::from_files(DataFiles {
        agents_js: &agents_js,
        browsers_js: &browsers_js,
        browser_versions_js: &browser_versions_js,
        envs_json: &envs_json,
        firefox_esr,
    });
}

/// Decode bytes as UTF-8 or fail naming the file.
fn text_of(file: &str, bytes: &[u8]) -> Outcome<String> {
    return String::from_utf8(bytes.to_vec()).map_err(|error| BrowserslistError::Data {
        file: file.to_string(),
        offset: Some(error.utf8_error().valid_up_to()),
        message: "not valid UTF-8".to_string(),
    });
}

/// What:
///  Load the data a lock pins.
/// Why:
///  The one call between "I have a lock" and "I have data to resolve against".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function loadData(lock: DataLock, source: DataSource): Promise<BrowserData>;
/// ```
pub fn load_data(lock: &DataLock, source: &DataSource) -> Outcome<BrowserData> {
    let caniuse = pinned_tarball(&lock.caniuse_lite, source)?;
    let node = pinned_tarball(&lock.node_releases, source)?;
    return decode_tarballs(&caniuse, &node, &lock.firefox_esr);
}

#[cfg(test)]
#[path = "source_tests.rs"]
mod tests;
