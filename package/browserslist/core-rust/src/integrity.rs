//! What:
//!  npm `dist.integrity` handling:
//! parse `sha512-<base64>`,
//! verify bytes against it,
//! and derive a file-name-safe key.
//! Why:
//!  Every download is trusted only after its SHA-512 digest equals the one pinned in the lock,
//!  and the cache is keyed by that digest.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! import { createHash } from 'node:crypto';
//! const ok = `sha512-${createHash('sha512').update(bytes).digest('base64')}` === integrity;
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
///  Import `Engine` from `base64::Engine`.
/// Why:
///  npm integrity strings carry their digest in base64.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Engine } from 'base64';
/// ```
use base64::Engine;
/// What:
///  Import `STANDARD` from `base64::engine::general_purpose::STANDARD`.
/// Why:
///  npm integrity strings carry their digest in base64.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { STANDARD } from 'base64/engine/general_purpose';
/// ```
use base64::engine::general_purpose::STANDARD;
/// What:
///  Import `Digest, Sha512` from `sha2`.
/// Why:
///  npm integrity strings are SHA-512 digests.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { Digest, Sha512 } from 'sha2';
/// ```
use sha2::{Digest, Sha512};

/// The only algorithm prefix the crate accepts.
const PREFIX: &str = "sha512-";

/// Length in bytes of a SHA-512 digest.
const DIGEST_LENGTH: usize = 64;

/// Build the error for a malformed integrity string.
fn malformed(integrity: &str, why: &str) -> BrowserslistError {
    return BrowserslistError::Data {
        file: "integrity value".to_string(),
        offset: None,
        message: format!("{integrity:?} is not a single sha512-<base64> npm integrity string: {why}"),
    };
}

/// What:
///  Decode a single `sha512-<base64>` integrity string into its 64 digest bytes.
/// Why:
///  A lock edited by hand,
/// or a registry answer in another algorithm,
/// fails here instead of weakening the check.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function parseSha512(integrity: string): Uint8Array; // throws unless 64 bytes
/// ```
pub fn parse_sha512(integrity: &str) -> Outcome<Vec<u8>> {
    let Some(encoded) = integrity.strip_prefix(PREFIX) else {
        return Err(malformed(integrity, "it does not start with sha512-"));
    };
    let decoded = STANDARD.decode(encoded).map_err(|error| return malformed(integrity, &error.to_string()))?;
    if decoded.len() != DIGEST_LENGTH {
        return Err(malformed(integrity, "the digest is not 64 bytes long"));
    }
    return Ok(decoded);
}

/// What:
///  Compute the integrity string (`sha512-<base64>`) of some bytes.
/// Why:
///  The update command and the mismatch diagnostic both need to print a digest in npm's spelling.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function integrityOf(bytes: Uint8Array): string;
/// ```
pub fn integrity_of(bytes: &[u8]) -> String {
    let digest = Sha512::digest(bytes);
    return format!("{PREFIX}{}", STANDARD.encode(digest.as_slice()));
}

/// What:
///  Succeed only when the bytes hash to the pinned integrity;
/// otherwise report both digests.
/// Why:
///  This is the single gate between downloaded or cached bytes and any use of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function verify(subject: string, integrity: string, bytes: Uint8Array): void; // throws on mismatch
/// ```
pub fn verify(subject: &str, integrity: &str, bytes: &[u8]) -> Outcome<()> {
    let expected = parse_sha512(integrity)?;
    let actual = Sha512::digest(bytes);
    if actual.as_slice() == expected.as_slice() {
        return Ok(());
    }
    return Err(BrowserslistError::Integrity {
        subject: subject.to_string(),
        expected: integrity.to_string(),
        actual: integrity_of(bytes),
    });
}

/// What:
///  The lowercase hex of the pinned digest,
/// used as the cache file stem.
/// Why:
///  Base64 contains `/` and `+`,
/// which are unsafe in a file name;
/// hex is not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function cacheKey(integrity: string): string;
/// ```
pub fn cache_key(integrity: &str) -> Outcome<String> {
    let digest = parse_sha512(integrity)?;
    return Ok(digest.iter().map(|byte| format!("{byte:02x}")).collect());
}

/// What:
///  The unit tests of `integrity.rs`,
/// kept in the sidecar file `integrity_tests.rs`.
/// Why:
///  The repository keeps Rust tests beside the code they cover,
/// out of the source file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import './integrity.test';
/// ```
#[cfg(test)]
#[path = "integrity_tests.rs"]
mod tests;
