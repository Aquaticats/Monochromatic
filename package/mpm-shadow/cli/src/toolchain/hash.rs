//! What:     SHA-256 as lowercase hex, for bytes in memory and for files.
//! Why:      The lock's digest, a downloaded archive's check against the lock, and the plan and lock revisions all
//!           use the same hash, so one place holds the formatting.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! import { createHash } from 'node:crypto';
//! export const sha256Hex = (bytes: Uint8Array) => createHash('sha256').update(bytes).digest('hex');
//! ```

/// What:     Imports the `Digest` trait (the `new`, `update`, `finalize` methods) and the SHA-256 type from `sha2`.
/// Why:      The trait must be in scope for the methods to be callable.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { createHash } from 'node:crypto';
/// ```
use sha2::{Digest, Sha256};

use std::fs::File;
use std::io::Read;
use std::path::Path;

/// The size of the buffer a file is hashed through, in bytes.
const READ_CHUNK_BYTES: usize = 64 * 1024;

/// What:     Turns a digest (raw bytes) into lowercase hex text, two characters per byte.
/// Why:      The lock stores hex, as `sha256sum` prints it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Buffer.from(bytes).toString('hex')
/// ```
fn hex_text(bytes: &[u8]) -> String {
    let mut text: String = String::new();
    for byte in bytes {
        text.push_str(&format!("{byte:02x}"));
    }
    return text;
}

/// SHA-256 of `bytes` as 64 lowercase hex characters.
pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    return hex_text(digest.as_slice());
}

/// What:     SHA-256 of the file at `path`, read in chunks so a large archive never sits in memory whole.
/// Why:      Archives are tens of megabytes; the check runs before anything is extracted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export async function sha256File(path: string): Promise<string>
/// ```
pub fn sha256_file(path: &Path) -> Result<String, String> {
    let mut file: File = match File::open(path) {
        Ok(opened) => opened,
        Err(error) => return Err(format!("cannot open {} to hash it: {error}", path.display())),
    };
    let mut hasher: Sha256 = Sha256::new();
    let mut buffer: Vec<u8> = vec![0; READ_CHUNK_BYTES];
    loop {
        let read: usize = match file.read(&mut buffer) {
            Ok(count) => count,
            Err(error) => return Err(format!("cannot read {} to hash it: {error}", path.display())),
        };
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    let digest = hasher.finalize();
    return Ok(hex_text(digest.as_slice()));
}
