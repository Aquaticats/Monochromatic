//! True-peak fingerprinting plus the cache module wiring.
//!
//! The cache store now lives in a Turso-backed actor (`peakcache_service.rs`)
//! reached through the synchronous `CacheHandle` (`peakcache_handle.rs`);
//!  this root
//! keeps only the storage-independent pieces:
//!  `fingerprint` (the opaque cache key)
//! and `db_path` (the one place that decides where the database file lives).
//! Privacy is unchanged:
//!  a fingerprint is a one-way hash of (path,
//!  size,
//!  mtime),
//!  so
//! only `fingerprint -> peak` rows ever hit disk,
//!  never a filename,
//!  path,
//!  or tag.

/// What:
///      `use std::path::{Path, PathBuf};`.
///  Borrowed and owned filesystem paths.
/// Why:
///       `fingerprint` borrows a path;
///  `db_path` returns an owned one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // both are just `string` in TS
/// ```
use std::path::{Path, PathBuf};

/// What:
///      `use std::time::UNIX_EPOCH;`.
///  The 1970 reference instant.
/// Why:
///       Convert a file's modified-time into a number for the fingerprint.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const UNIX_EPOCH = 0; // ms-since-1970 baseline
/// ```
use std::time::UNIX_EPOCH;

/// What:
///      `use crate::identity;` imports the shared identity-strings module
///           (reads stay qualified as `identity::config_dir`).
/// Why:
///       `db_path` builds the config dir from the same reverse-DNS triple the
///           session uses,
///  so the cache and session share a directory.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as identity from "./identity";
/// ```
use crate::identity;

/// What:
///      `use truepeak_core::{CacheIdentity, default_policy, track_fingerprint};`.
///  The four-part
///           cache identity type,
///  the one shipped policy,
///  and the shared fingerprint function,
///  from the
///           shared crate.
/// Why:
///       `cache_identity` pairs the shared policy's `policy_id`/`meter_id`/`schema`
///           with the desktop's `decoder_stack_id`,
///  so a cache row is reused only when the
///           whole measurement environment matches.
///  `fingerprint` hands its three inputs to `track_fingerprint`,
///           so desktop and Android share one hash and one byte layout.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { CacheIdentity, defaultPolicy, trackFingerprint } from "truepeak-core";
/// ```
use truepeak_core::{CacheIdentity, default_policy, track_fingerprint};

/// What:
///      `#[path = "peakcache_service.rs"] mod service;`.
///  The async cache actor,
///           loaded from a flat sibling file.
/// Why:
///       Keep the Turso connection,
///  runtime,
///  and serve-loop out of this root.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as service from "./peakcache_service";
/// ```
#[path = "peakcache_service.rs"]
mod service;

/// What:
///      `#[path = "peakcache_handle.rs"] mod handle;`.
///  The synchronous handle,
///           loaded from a flat sibling file.
/// Why:
///       Keep the channel plumbing beside,
///  but separate from,
///  the actor.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as handle from "./peakcache_handle";
/// ```
#[path = "peakcache_handle.rs"]
mod handle;

/// What:
///      `pub(crate) use handle::CacheHandle;`.
///  Re-export the handle type at the
///           `peakcache` path.
/// Why:
///       Callers say `peakcache::CacheHandle`,
///  unaware of the submodule split.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export { CacheHandle } from "./peakcache_handle";
/// ```
pub(crate) use handle::CacheHandle;

/// What:
///      `pub(crate) fn fingerprint(path: &Path) -> Option<u64>`.
///  Build the opaque
///           cache key for a file from its path,
///  size,
///  and modified-time.
///  Returns `None`
///           if the file cannot be stat'd.
///  `pub(crate)` for the controller and background
///           worker.
///  The value is the shared `track_fingerprint` output (XXH3-64 over a
///           domain-tagged,
///  length-prefixed layout),
///  since the shared `DecisionCache` keys on a `u64`
///           (stored as a bijective `i64` bit-cast),
///  not a hex string.
/// Why:
///       Identify a track without storing anything identifying;
///  size+mtime make the key
///           change when the file is replaced or edited in place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function fingerprint(path: string): bigint | null {
///   let meta; try { meta = statSync(path); } catch { return null; }
///   return trackFingerprint(path, meta.size, meta.mtimeNanos);
/// }
/// ```
pub(crate) fn fingerprint(path: &Path) -> Option<u64> {
    // What:     `let meta = std::fs::metadata(path).ok()?;`. Read filesystem metadata;
    //           `?` returns `None` on error.
    // Why:      Need the size and modified time; bail to "no fingerprint" if absent.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let meta; try { meta = statSync(path); } catch { return null; }
    // ```
    let meta = std::fs::metadata(path).ok()?;
    // What:     `let modified = meta.modified().ok()?;`. The last-modified `SystemTime`.
    // Why:      Part of the key so editing a file in place invalidates its entry.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const modified = meta.mtime;
    // ```
    let modified = meta.modified().ok()?;
    // What:     `let mtime_nanos = modified.duration_since(UNIX_EPOCH).ok()?.as_nanos();`.
    //           Nanoseconds since 1970 as a `u128`; bails if the time predates 1970.
    // Why:      A plain number to hash.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const mtimeNanos = BigInt(modified) * 1_000_000n;
    // ```
    let mtime_nanos = modified.duration_since(UNIX_EPOCH).ok()?.as_nanos();
    // What:     `let path_text = path.to_string_lossy();`. The path as text, lossy.
    // Why:      Stable bytes to feed the hash; lossy is fine since we only hash it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const pathText = String(path);
    // ```
    let path_text = path.to_string_lossy();
    // What:     `track_fingerprint(&path_text, meta.len(), mtime_nanos)`. Hand the path text,
    //           the file size (`u64`), and the mtime nanoseconds (`u128`) to the shared
    //           function, which owns the hashed layout and the hash. Tail -> return.
    // Why:      The opaque key stored on disk; a non-cryptographic 64-bit hash is not
    //           reversible to the path in practice, preserving the privacy guarantee. The
    //           shared cache keys on the `u64` directly, so no hex rendering is needed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return trackFingerprint(pathText, meta.size, mtimeNanos);
    // ```
    return Some(track_fingerprint(&path_text, meta.len(), mtime_nanos))
}

/// What:
///      `const DECODER_STACK_DESCRIPTION: &str = "...";`.
///  A stable text description of
///           the desktop decode stack that produces the PCM the meter reads.
/// Why:
///       Its hash is the `decoder_stack_id`;
///  editing this string when the decoder stack
///           changes (a Symphonia or libopus bump,
///  an f32 conversion change) re-keys the
///           cache,
///  so decisions measured under a different decoder are never reused.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const DECODER_STACK_DESCRIPTION = "desktop:symphonia-0.6+opus-rev-5598766+f32le";
/// ```
const DECODER_STACK_DESCRIPTION: &str = "desktop:symphonia-0.6+opus-rev-5598766+f32le";

/// What:
///      `fn decoder_stack_id() -> u64`.
///  Hash the decoder-stack description into a stable
///           id with the shared crate's `stack_id`.
///  Module-private.
/// Why:
///       The platform owns its description;
///  the shared crate owns the derivation,
///  the
///           same FNV every other identity id uses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function decoderStackId(): bigint { return stackId(DECODER_STACK_DESCRIPTION); }
/// ```
fn decoder_stack_id() -> u64 {
    // What:     `truepeak_core::stack_id(DECODER_STACK_DESCRIPTION)`. Tail -> return.
    // Why:      Deterministic id that changes only when the description does.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return stackId(DECODER_STACK_DESCRIPTION);
    // ```
    return truepeak_core::stack_id(DECODER_STACK_DESCRIPTION)
}

/// What:
///      `pub(crate) fn cache_identity() -> CacheIdentity`.
///  The full four-part identity a
///           desktop cache row must match:
///  the shipped policy's id and meter id,
///  the desktop
///           decoder id,
///  and the schema version.
///  `pub(crate)` for the cache actor.
/// Why:
///       The actor keys every `get`/`put`/`exact_fingerprints` on this identity,
///  so a
///           policy,
///  meter,
///  decoder,
///  or schema change transparently starts a fresh cache.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function cacheIdentity(): CacheIdentity { return defaultPolicy().cacheIdentity(decoderStackId()); }
/// ```
pub(crate) fn cache_identity() -> CacheIdentity {
    // What:     `default_policy().cache_identity(decoder_stack_id())`. Assemble the identity
    //           from the shared policy and the desktop decoder id. Tail -> return.
    // Why:      One place builds the desktop's cache identity.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return defaultPolicy().cacheIdentity(decoderStackId());
    // ```
    return default_policy().cache_identity(decoder_stack_id())
}

/// What:
///      `fn db_path() -> Option<PathBuf>`.
///  The on-disk location of the decision
///           database,
///  or `None` if no config directory is available.
///  Module-private.
/// Why:
///       One place decides where the cache lives (the same `identity::config_dir` the
///           session file uses,
///  so they never drift apart).
/// Gotcha:
///    The filename is `decisions.db`,
///  NOT the legacy `peaks.db`:
///  the shared cache
///           uses a different schema (a `decisions` table keyed by the identity tuple),
///  and
///           the plan says not to import legacy raw-peak rows,
///  so a fresh file starts the
///           new policy from an empty cache and leaves any old `peaks.db` orphaned.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function dbPath(): string | null {
///   const dir = configDir();
///   return dir ? join(dir, "decisions.db") : null;
/// }
/// ```
fn db_path() -> Option<PathBuf> {
    // What:     `identity::config_dir().map(|dir| dir.join("decisions.db"))`. Append the
    //           database filename to the shared config directory when present. Tail
    //           -> return.
    // Why:      Same directory the session uses, so the containerized run's config volume
    //           persists it and it never pollutes the source tree.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const dir = configDir();
    // return dir ? join(dir, "decisions.db") : null;
    // ```
    return identity::config_dir().map(|dir| return dir.join("decisions.db"))
}

/// What:
///      `#[cfg(test)] #[path = "peakcache_tests.rs"] mod tests;` declares a
///           test-only submodule whose code lives in the sibling file.
/// Why:
///       Keep tests beside the code without inflating this file or its max-lines
///           budget (sibling `*_tests.rs` files are exempt from the linter).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // peakcache.unit.test.ts, run only by the test runner
/// ```
#[cfg(test)]
#[path = "peakcache_tests.rs"]
mod tests;
