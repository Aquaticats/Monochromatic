//! The track fingerprint:
//!  the opaque `u64` cache key both music players derive from a file's path,
//!  size,
//!  and modified-time.
//!
//! One function owns the hashed byte layout and the hash,
//!  so the desktop and Android flavors cannot drift apart:
//!  identical `(path,
//!  size,
//!  mtime)` inputs hash to identical keys on both by construction.
//!  The hash is XXH3-64
//! from `twox-hash` with its default seed 0.
//!  XXH3 seeds are not domain separators,
//!  so the domain
//! tag and the field encoding live inside the hashed bytes instead:
//!  a fixed tag,
//!  then the
//! path length as 8 little-endian bytes,
//!  the path bytes,
//!  the size as 8 little-endian bytes,
//!  and
//! the mtime nanoseconds as 16 little-endian bytes.
//!  Privacy is unchanged:
//!  only the one-way
//! digest is stored,
//!  never a filename or path.
//!
//! The known-answer tests in `fingerprint_tests.rs` pin the digests to the xxHash reference C
//! implementation,
//!  so a change of crate,
//!  seed,
//!  tag,
//!  or field order fails them.
//!  Decision record:
//! `doc/decision/music-player-fingerprint-hash.md`.

/// What:
///      `use twox_hash::XxHash3_64;`.
///  The XXH3-64 hasher type;
///  its associated
///           `oneshot(input: &[u8]) -> u64` hashes a whole slice with the default seed 0.
///           Run-time AVX2, SSE2, or NEON selection comes from the crate's `std` feature.
/// Why:
///       The one hash behind every fingerprint,
///  identical on x86_64 and aarch64 for the same
///           bytes,
///  with no CPU target-feature build flag.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { xxh3_64 } from "xxh3"; // hypothetical; any XXH3-64 port emits the same digest
/// ```
use twox_hash::XxHash3_64;

/// What:
///      `pub const FINGERPRINT_DOMAIN_TAG: &[u8] = b"...";`.
///  The fixed domain tag every
///           fingerprint's hashed bytes start with.
///  `&[u8]` (sibling `&str`) because it is
///           hashed as raw bytes.
/// Why:
///       Separates this key space from any other XXH3-64 digest of similar bytes.
///           A seed cannot do that job:
///  XXH3 seeds are not domain separators,
///  so the tag
///           rides inside the hashed bytes and the seed stays at the default 0.
///  The value is
///           part of the on-disk key format;
///  changing it re-keys every cache.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const FINGERPRINT_DOMAIN_TAG = new TextEncoder().encode("monochromatic.music-player.track-fingerprint.v1");
/// ```
pub const FINGERPRINT_DOMAIN_TAG: &[u8] = b"monochromatic.music-player.track-fingerprint.v1";

/// What:
///      `fn fingerprint_material(path: &str, size: u64, mtime_nanos: u128) -> Vec<u8>`.
///           Build the exact bytes that get hashed:
///  tag,
///  path length (`u64` little-endian),
///  path
///           bytes,
///  size (`u64` little-endian),
///  mtime nanoseconds (`u128` little-endian).
///  Module-private.
/// Why:
///       The length prefix and the fixed-width integers make the encoding unambiguous:
///           no two different `(path,
///  size,
///  mtime)` triples produce the same byte string.
///           Only the owner of the layout builds it,
///  so a layout change is one edit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function fingerprintMaterial(path: string, size: bigint, mtimeNanos: bigint): Uint8Array {
///   const pathBytes = new TextEncoder().encode(path);
///   return concat(FINGERPRINT_DOMAIN_TAG, u64le(BigInt(pathBytes.length)), pathBytes, u64le(size), u128le(mtimeNanos));
/// }
/// ```
fn fingerprint_material(path: &str, size: u64, mtime_nanos: u128) -> Vec<u8> {
    // What:     `let path_bytes = path.as_bytes();`. The path's UTF-8 bytes, borrowed.
    // Why:      Both the length prefix and the body come from the same slice.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const pathBytes = new TextEncoder().encode(path);
    // ```
    let path_bytes = path.as_bytes();
    // What:     `Vec::with_capacity(...)`. A byte buffer sized for the whole material.
    // Why:      One allocation per fingerprint instead of a regrow per field.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const material: number[] = [];
    // ```
    let mut material: Vec<u8> = Vec::with_capacity(
        FINGERPRINT_DOMAIN_TAG.len()
            + size_of::<u64>()
            + path_bytes.len()
            + size_of::<u64>()
            + size_of::<u128>(),
    );
    // What:     `material.extend_from_slice(FINGERPRINT_DOMAIN_TAG);`. Append the tag first.
    // Why:      The domain separation sits at the start of every hashed input.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // material.push(...FINGERPRINT_DOMAIN_TAG);
    // ```
    material.extend_from_slice(FINGERPRINT_DOMAIN_TAG);
    // What:     `material.extend_from_slice(&(path_bytes.len() as u64).to_le_bytes());`. The
    //           path length as 8 little-endian bytes (`usize` widens losslessly to `u64`).
    // Why:      Makes the path field self-delimiting, so a path cannot swallow the size bytes.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // material.push(...u64le(BigInt(pathBytes.length)));
    // ```
    material.extend_from_slice(&(path_bytes.len() as u64).to_le_bytes());
    // What:     `material.extend_from_slice(path_bytes);`. Append the path bytes.
    // Why:      Distinguish tracks at different paths.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // material.push(...pathBytes);
    // ```
    material.extend_from_slice(path_bytes);
    // What:     `material.extend_from_slice(&size.to_le_bytes());`. The file size as 8
    //           little-endian bytes.
    // Why:      A re-encode changes the size and so the key.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // material.push(...u64le(size));
    // ```
    material.extend_from_slice(&size.to_le_bytes());
    // What:     `material.extend_from_slice(&mtime_nanos.to_le_bytes());`. The modified-time
    //           as 16 little-endian bytes (`u128`).
    // Why:      An in-place edit with the same size still changes the key.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // material.push(...u128le(mtimeNanos));
    // ```
    material.extend_from_slice(&mtime_nanos.to_le_bytes());
    // What:     `material`. The finished bytes. Tail -> return.
    // Why:      Hand the buffer to the hash.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return new Uint8Array(material);
    // ```
    return material
}

/// What:
///      `pub fn track_fingerprint(path: &str, size: u64, mtime_nanos: u128) -> u64`.
///           Derive the opaque cache key for one file:
///  XXH3-64 (seed 0) over the domain-tagged,
///           length-prefixed material.
///  `&str` borrows the path text the caller already decoded
///           (lossy on desktop,
///  the URI string on Android).
/// Why:
///       Identify a track without storing anything identifying;
///  size and mtime change the key
///           when the file is replaced or edited in place.
///  Both flavors call this one function,
///           which is the fingerprint bit-parity contract.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function trackFingerprint(path: string, size: bigint, mtimeNanos: bigint): bigint {
///   return xxh3_64(fingerprintMaterial(path, size, mtimeNanos)); // default seed 0
/// }
/// ```
pub fn track_fingerprint(path: &str, size: u64, mtime_nanos: u128) -> u64 {
    // What:     `let material = fingerprint_material(path, size, mtime_nanos);`. Build the bytes.
    // Why:      The hash input is defined in one place.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const material = fingerprintMaterial(path, size, mtimeNanos);
    // ```
    let material = fingerprint_material(path, size, mtime_nanos);
    // What:     `let fingerprint = XxHash3_64::oneshot(&material);`. Hash the slice with the
    //           default seed 0. `oneshot` is the crate's whole-slice entry point.
    // Why:      The material is small and fully in memory, so no streaming state is needed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const fingerprint = xxh3_64(material);
    // ```
    let fingerprint = XxHash3_64::oneshot(&material);
    // The path itself is never logged: only its length, so logs keep the privacy guarantee.
    tracing::trace!(
        path_len = path.len(),
        size,
        material_len = material.len(),
        fingerprint,
        "track fingerprint computed"
    );
    // What:     `fingerprint`. The `u64` key. Tail -> return.
    // Why:      The shared cache keys on the `u64` directly.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return fingerprint;
    // ```
    return fingerprint
}

/// What:
///      `#[cfg(test)] #[path = "fingerprint_tests.rs"] mod tests;`.
///  Test-only submodule in the
///           sibling file.
/// Why:
///       Keep this file to production code;
///  sibling `*_tests.rs` is max-lines exempt.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // fingerprint.unit.test.ts
/// ```
#[cfg(test)]
#[path = "fingerprint_tests.rs"]
mod tests;
