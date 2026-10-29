//! Resolves platform cache roots and content-addressed artifact paths.
//!
//! Resolution is pure over an injected environment snapshot and platform tag,
//! letting Linux tests cover the macOS and Windows branches shipped by releases.

/// Imports path values used without assuming UTF-8 environment contents.
use std::path::{Path, PathBuf};

/// Fixed application directory below selected platform cache root.
const APPLICATION_DIRECTORY: &str = "forbidden-strings";
/// Fixed artifact filename below content digest directory.
const ARTIFACT_FILENAME: &str = "rules.bin";
/// Lowercase hexadecimal alphabet for source SHA-256 directory names.
const HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";

/// Exact 32-byte SHA-256 of authoritative rules-file bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SourceDigest(
    /// Exact digest bytes retained without source content.
    [u8; 32],
);

/// Hashing failure at the existing `gix-hash` seam.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct DigestError;

/// User-cache root resolution failure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CacheRootError {
    /// Explicit scanner override was present but not absolute.
    InvalidOverride,
    /// Native platform root could not be derived from available environment.
    Unavailable,
}

/// Platform family controlling native cache-root convention.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum HostPlatform {
    /// Linux and other XDG-oriented Unix systems.
    XdgUnix,
    /// Apple macOS native cache convention.
    Macos,
    /// Microsoft Windows local application-data convention.
    Windows,
}

/// Environment fields relevant to cache-root resolution.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct CacheEnvironment {
    /// Explicit forbidden-strings cache-root override.
    pub(super) override_root: Option<std::ffi::OsString>,
    /// XDG user cache root.
    pub(super) xdg_cache_home: Option<std::ffi::OsString>,
    /// User home used by Unix and macOS fallbacks.
    pub(super) home: Option<std::ffi::OsString>,
    /// Windows native local application-data root.
    pub(super) local_app_data: Option<std::ffi::OsString>,
}

/// Resolved protected directories and final content-addressed artifact path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct CacheLocation {
    /// Application-owned root whose descendants contain sensitive fingerprints.
    pub(super) application_directory: PathBuf,
    /// Ordered application-owned directories that should receive private modes.
    pub(super) protected_directories: Vec<PathBuf>,
    /// Final compiled artifact path.
    pub(super) artifact_path: PathBuf,
}

/// Returns SHA-256 over exact source bytes using existing direct `gix-hash` dependency.
pub(super) fn source_digest(bytes: &[u8]) -> Result<SourceDigest, DigestError> {
    let mut hasher = gix_hash::hasher(gix_hash::Kind::Sha256);
    hasher.update(bytes);
    let object_id = hasher.try_finalize().map_err(|_| return DigestError)?;
    let digest: [u8; 32] = object_id.as_slice().try_into().map_err(|_| return DigestError)?;
    return Ok(SourceDigest(digest))
}

/// Reads process environment values once for production cache resolution.
#[cfg(not(test))]
pub(super) fn current_environment() -> CacheEnvironment {
    return CacheEnvironment {
        override_root: std::env::var_os("FORBIDDEN_STRINGS_CACHE_DIR"),
        xdg_cache_home: std::env::var_os("XDG_CACHE_HOME"),
        home: std::env::var_os("HOME"),
        local_app_data: std::env::var_os("LOCALAPPDATA"),
    }
}

/// Uses disposable process-scoped cache root for unit tests.
#[cfg(test)]
pub(super) fn current_environment() -> CacheEnvironment {
    return CacheEnvironment {
        override_root: Some(
            std::env::temp_dir()
                .join(format!("forbidden-strings-unit-cache-{}", std::process::id()))
                .into_os_string(),
        ),
        ..CacheEnvironment::default()
    }
}

/// Returns compile-target platform family for production resolution.
pub(super) fn current_platform() -> HostPlatform {
    if cfg!(target_os = "macos") {
        return HostPlatform::Macos;
    }
    if cfg!(target_os = "windows") {
        return HostPlatform::Windows;
    }
    return HostPlatform::XdgUnix
}

/// Resolves base cache root from settled explicit-override and native-platform policy.
pub(super) fn resolve_cache_root(
    environment: &CacheEnvironment,
    platform: HostPlatform,
) -> Result<PathBuf, CacheRootError> {
    if let Some(override_root) = &environment.override_root {
        return platform_absolute_path(override_root, platform)
            .map_err(|_| return CacheRootError::InvalidOverride);
    }
    if platform == HostPlatform::Windows {
        return environment_absolute_path(environment.local_app_data.as_ref(), platform);
    }
    if platform == HostPlatform::Macos {
        return environment_absolute_path(environment.home.as_ref(), platform)
            .map(|home| return home.join("Library").join("Caches"));
    }
    if let Some(xdg_cache_home) = &environment.xdg_cache_home {
        let path = PathBuf::from(xdg_cache_home);
        if path.is_absolute() {
            return Ok(path);
        }
    }
    return environment_absolute_path(environment.home.as_ref(), platform)
        .map(|home| return home.join(".cache"))
}

/// Converts optional environment value into platform-absolute path or unavailable error.
fn environment_absolute_path(
    value: Option<&std::ffi::OsString>,
    platform: HostPlatform,
) -> Result<PathBuf, CacheRootError> {
    let present = value.ok_or(CacheRootError::Unavailable)?;
    return platform_absolute_path(present, platform)
}

/// Validates path with selected target semantics so every platform branch is host-testable.
fn platform_absolute_path(
    value: &std::ffi::OsString,
    platform: HostPlatform,
) -> Result<PathBuf, CacheRootError> {
    let path = PathBuf::from(value);
    if platform != HostPlatform::Windows {
        if path.is_absolute() {
            return Ok(path);
        }
        return Err(CacheRootError::Unavailable);
    }
    let rendered = value.to_string_lossy();
    let bytes = rendered.as_bytes();
    let has_drive_root = bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && (bytes[2] == b'\\' || bytes[2] == b'/');
    let has_unc_root = rendered.starts_with("\\\\") || rendered.starts_with("//");
    if has_drive_root || has_unc_root {
        return Ok(path);
    }
    return Err(CacheRootError::Unavailable)
}

/// Returns lowercase hexadecimal source digest for directory naming.
fn digest_hex(digest: SourceDigest) -> String {
    let mut rendered = String::with_capacity(64);
    for byte in digest.0 {
        rendered.push(HEX_DIGITS[usize::from(byte >> 4)] as char);
        rendered.push(HEX_DIGITS[usize::from(byte & 0x0f)] as char);
    }
    return rendered
}

/// Returns exact scanner semantic version embedded into artifacts.
pub(super) fn scanner_version() -> &'static str {
    return env!("CARGO_PKG_VERSION")
}

/// Returns compile-target operating-system and architecture identity.
pub(super) fn platform_identity() -> String {
    return format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH)
}

/// Returns scanner-version and compile-platform partition components.
fn compatibility_components() -> (String, String) {
    return (format!("v{}", scanner_version()), platform_identity())
}

/// Builds protected hierarchy and final artifact path for one source digest.
pub(super) fn cache_location(
    root: &Path,
    digest: SourceDigest,
) -> CacheLocation {
    let application_directory = root.join(APPLICATION_DIRECTORY);
    let (version, platform) = compatibility_components();
    let version_directory = application_directory.join(version);
    let platform_directory = version_directory.join(platform);
    let digest_directory = platform_directory.join(digest_hex(digest));
    let artifact_path = digest_directory.join(ARTIFACT_FILENAME);
    return CacheLocation {
        application_directory: application_directory.clone(),
        protected_directories: vec![
            application_directory,
            version_directory,
            platform_directory,
            digest_directory,
        ],
        artifact_path,
    }
}

/// Returns digest bytes for envelope identity checks.
pub(super) fn digest_bytes(digest: SourceDigest) -> [u8; 32] {
    return digest.0
}

/// Renders static root-resolution errors without environment values.
impl std::fmt::Display for CacheRootError {
    /// Writes redacted configuration reason.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self == &CacheRootError::InvalidOverride {
            return formatter.write_str("FORBIDDEN_STRINGS_CACHE_DIR must be an absolute path");
        }
        return formatter.write_str("per-user cache root is unavailable")
    }
}

/// Lets digest errors participate in application error channels.
impl std::fmt::Display for DigestError {
    /// Writes static hashing failure.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return formatter.write_str("SHA-256 source fingerprint failed")
    }
}

/// Standard error marker for cache-root failures.
impl std::error::Error for CacheRootError {}
/// Standard error marker for source-digest failures.
impl std::error::Error for DigestError {}

/// Registers cross-platform path-resolution tests.
#[cfg(test)]
#[path = "path_tests.rs"]
mod tests;
