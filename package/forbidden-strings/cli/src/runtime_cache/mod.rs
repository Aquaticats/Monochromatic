//! Runtime rule-cache module.
//!
//! Its interface hides content hashing,
//!  platform-root resolution,
//!  envelope
//! validation,
//!  warning classification,
//!  and atomic publication.
//!  Scan callers receive
//! compiled rules plus redacted warnings;
//!  compile-command callers receive only
//! success or a redacted fatal error.
//!
//! The cache key is the SHA-256 of the joined rules text,
//! which a [`RulesSnapshot`] builds from the files of one
//! rules list.
//! The compiled rules are a pure function of that text,
//! so the key is complete:
//!  a changed byte in any file,
//! a file added or removed,
//! and a changed file order all change the text and miss.
//! Revalidation before publication re-reads every listed file,
//! so an optional file that appears while the rules
//! compile also blocks publication.

/// Registers scanner-owned envelope codec.
mod envelope;
/// Registers content key and platform cache path resolution.
mod path;
/// Registers complete read and atomic publication.
mod publish;
/// Registers closed JSON warning protocol.
mod warning;

/// Imports application-level redacted error construction.
use anyhow::{anyhow, Result};

/// Imports the list entry type `compile_rules_to_cache` takes.
use crate::rules_list::RulesEntry;
/// Imports the joined,
/// revalidatable snapshot of the files of one rules list.
use crate::rules_snapshot::RulesSnapshot;
/// Imports hybrid runtime matcher compiled from authoritative source.
use crate::runtime_matcher::RuntimeRules;

/// Imports envelope operations and complete artifact ceiling.
use envelope::{decode, encode, MAX_ARTIFACT_BYTES};
/// Imports source identity and cache-location operations.
use path::{
    cache_location, current_environment, current_platform, resolve_cache_root, source_digest,
    CacheRootError, SourceDigest,
};
/// Imports complete artifact IO operations.
use publish::{publish_artifact, read_artifact, ArtifactReadError};
/// Re-exports warning value for loaded-rules output.
pub use warning::CacheWarning;
/// Imports closed warning reason values.
use warning::CacheWarningReason;

/// What:
///  Whether a load may add the rules it compiled to the shared cache.
/// Why:
///   A host that publishes the compiled rules somewhere else
///  (a resident service reading the same repository)
///  must not pay for the write itself,
///  and every read validates the whole artifact,
///  so a load that only reads is as correct as one that writes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CacheAccess = 'read-write' | 'read-only';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CacheAccess {
    /// Read a valid artifact,
    ///  compile on a miss,
    ///  and publish what was compiled.
    ReadWrite,
    /// Read a valid artifact and compile on a miss,
    ///  never writing the cache.
    ReadOnly,
}

/// Runtime cache result consumed by frx loader.
pub(crate) struct CacheLoad {
    /// Valid compiled rules from artifact or authoritative text.
    pub(crate) compiled: RuntimeRules,
    /// Redacted cache diagnostics emitted before scan findings.
    pub(crate) warnings: Vec<CacheWarning>,
}

/// Loads compatible artifact or compiles and,
///  under [`CacheAccess::ReadWrite`],
///  best-effort repairs from one snapshot of the rules files.
pub(crate) fn load_or_compile(snapshot: &RulesSnapshot, access: CacheAccess) -> Result<CacheLoad> {
    let digest = match source_digest(snapshot.text().as_bytes()) {
        Ok(digest) => digest,
        Err(error) => return Err(anyhow!("rules {}: {}", snapshot.label(), error)),
    };
    let environment = current_environment();
    let root = match resolve_cache_root(&environment, current_platform()) {
        Ok(root) => root,
        Err(CacheRootError::InvalidOverride) => {
            return Err(anyhow!("rules {}: {}", snapshot.label(), CacheRootError::InvalidOverride));
        }
        Err(CacheRootError::Unavailable) => {
            let compiled = snapshot.compile()?;
            return Ok(CacheLoad {
                compiled,
                warnings: vec![CacheWarning::compile_from_text(
                    CacheWarningReason::CacheRootUnavailable,
                )],
            });
        }
    };
    let location = cache_location(&root, digest);
    match read_artifact(&location.artifact_path, MAX_ARTIFACT_BYTES) {
        Ok(bytes) => match decode(&bytes, digest) {
            Ok(compiled) => return Ok(CacheLoad { compiled, warnings: Vec::new() }),
            Err(error) => {
                return compile_and_repair(snapshot, digest, &location, error.warning_reason(), access);
            }
        },
        Err(ArtifactReadError::Missing) => {
            return compile_and_repair(snapshot, digest, &location, CacheWarningReason::Missing, access);
        }
        Err(ArtifactReadError::Unreadable) => {
            return compile_and_repair(snapshot, digest, &location, CacheWarningReason::Unreadable, access);
        }
    }
}

/// Whether the shared cache already holds a compiled form of this snapshot's text.
/// Why:
///  A host asks this before it loads the rules,
///  to learn whether the load can miss and so whether it must arrange for someone else to publish;
///  the answer is the same validation the load itself accepts,
///  so a cache a read would reject is one this reports absent.
///
pub(crate) fn holds_compiled_rules(snapshot: &RulesSnapshot) -> bool {
    let Ok(digest) = source_digest(snapshot.text().as_bytes()) else { return false };
    let Ok(root) = resolve_cache_root(&current_environment(), current_platform()) else { return false };
    let location = cache_location(&root, digest);
    return match read_artifact(&location.artifact_path, MAX_ARTIFACT_BYTES) {
        Ok(bytes) => decode(&bytes, digest).is_ok(),
        Err(_absent) => false,
    };
}

/// Whether the shared cache holds a compiled form of the text of these rules files.
/// Why:
///  An embedding host arranges for the compiled rules to be published off its own path
///  when the cache holds none,
///  and every rules-file question this asks is the one its own load asks
///  (the files in list order,
///  an absent optional entry skipped).
/// A list that cannot be read answers no,
///  so the caller keeps the write itself and the next load says why it failed.
///
pub fn has_compiled_rules(entries: &[RulesEntry]) -> bool {
    let Ok(snapshot) = RulesSnapshot::read(entries) else { return false };
    if snapshot.is_empty() {
        return false
    }
    return holds_compiled_rules(&snapshot);
}

/// Compiles source snapshot and,
///  when the access allows it,
///  appends a write warning when repair cannot publish safely.
fn compile_and_repair(
    snapshot: &RulesSnapshot,
    digest: SourceDigest,
    location: &path::CacheLocation,
    reason: CacheWarningReason,
    access: CacheAccess,
) -> Result<CacheLoad> {
    let compiled = snapshot.compile()?;
    let mut warnings = vec![CacheWarning::compile_from_text(reason)];
    if access == CacheAccess::ReadOnly {
        return Ok(CacheLoad { compiled, warnings })
    }
    if !snapshot.still_current() || !publish_compiled_snapshot(&compiled, digest, location) {
        warnings.push(CacheWarning::write_failed());
    }
    return Ok(CacheLoad { compiled, warnings })
}

/// Encodes and publishes compiled snapshot without conflating codec and IO errors.
fn publish_compiled_snapshot(
    compiled: &RuntimeRules,
    digest: SourceDigest,
    location: &path::CacheLocation,
) -> bool {
    let Ok(bytes) = encode(compiled, digest) else {
        return false;
    };
    return publish_artifact(location, &bytes).is_ok()
}

/// The fatal error for an artifact that could not be encoded,
///  whatever the redacted cause.
fn artifact_not_encoded<Cause>(_cause: Cause) -> anyhow::Error {
    return anyhow!("runtime rules cache artifact could not be encoded");
}

/// The fatal error for an artifact that could not be published,
///  whatever the redacted cause.
fn artifact_not_published<Cause>(_cause: Cause) -> anyhow::Error {
    return anyhow!("runtime rules cache artifact could not be published");
}

/// Eagerly compiles the files of one rules list into the derived per-user cache artifact.
///
/// Returns the optional entries that were absent,
/// so the caller can log them.
/// A list whose files are all absent compiles nothing and fails.
pub(crate) fn compile_rules_to_cache(entries: &[RulesEntry]) -> Result<Vec<std::path::PathBuf>> {
    let snapshot = RulesSnapshot::read(entries)?;
    snapshot.require_present()?;
    let label = snapshot.label();
    let digest = match source_digest(snapshot.text().as_bytes()) {
        Ok(digest) => digest,
        Err(error) => return Err(anyhow!("rules {}: {}", label, error)),
    };
    let root = resolve_cache_root(&current_environment(), current_platform()).map_err(anyhow::Error::msg)?;
    let location = cache_location(&root, digest);

    let cached_artifact_decodes: bool = match read_artifact(&location.artifact_path, MAX_ARTIFACT_BYTES) {
        Ok(bytes) => decode(&bytes, digest).is_ok(),
        Err(_) => false,
    };
    if cached_artifact_decodes {
        if snapshot.still_current() {
            return Ok(snapshot.skipped());
        }
        return Err(anyhow!("rules {} changed while validating cache", label));
    }

    let compiled = snapshot.compile()?;
    if !snapshot.still_current() {
        return Err(anyhow!("rules {} changed during compilation", label));
    }
    let artifact = encode(&compiled, digest)
        .map_err(artifact_not_encoded)?;
    publish_artifact(&location, &artifact)
        .map_err(artifact_not_published)?;
    return Ok(snapshot.skipped());
}

/// Source-snapshot publication checks and optional fuzz exports need executed positive/negative controls.
#[cfg(test)]
#[path = "verification_tests.rs"]
mod verification_tests;

/// Re-exports fuzz-only envelope decoder entry point.
#[cfg(feature = "fuzzing")]
pub fn decode_artifact_for_fuzzing(
    bytes: &[u8],
    source: &[u8],
) -> bool {
    let Ok(digest) = source_digest(source) else {
        return false;
    };
    return decode(bytes, digest).is_ok()
}
