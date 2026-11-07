//! What:
//!  SHA-1 and SHA-256 of a byte string, from the published `sha1` and `sha2` crates.
//! Why:
//!  The last bytes of a Git index file are a checksum of everything before them (SHA-1, or SHA-256 in a repository of
//!  that object format), and Git refuses an index whose checksum is wrong. The owner chose published crates over the
//!  hand-written functions this file held (2026-11-07, "Digests: sha1 + sha2"); the controls still compare both functions with
//!  the published test vectors and, in the migration controls, with the checksum real Git accepts.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! import { createHash } from 'node:crypto';
//! const sha1 = (bytes: Uint8Array) => createHash('sha1').update(bytes).digest();
//! ```

/// `Digest` is the trait that gives a hash type its one-call `digest` function.
use sha1::Digest as _;

/// What:
///  The SHA-1 digest (20 bytes) of `bytes`.
///  `[u8; 20]` is a fixed-size byte array (sibling `Vec<u8>` is growable).
/// Why:
///   The checksum of a SHA-1 repository's index.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// createHash('sha1').update(bytes).digest()
/// ```
pub fn sha1(bytes: &[u8]) -> [u8; 20] {
    return sha1::Sha1::digest(bytes).into();
}

/// What:
///  The SHA-256 digest (32 bytes) of `bytes`.
/// Why:
///   The checksum of a SHA-256 repository's index.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// createHash('sha256').update(bytes).digest()
/// ```
pub fn sha256(bytes: &[u8]) -> [u8; 32] {
    return sha2::Sha256::digest(bytes).into();
}

/// SHA control vectors stay out of the release executable.
#[cfg(test)]
#[path = "subvolume_sha_tests.rs"]
mod tests;
