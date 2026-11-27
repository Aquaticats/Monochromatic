//! What:
//!  The stable name a fallback's temporary is filled under,
//!  chosen from the stage token and the entry's path.
//! Why:
//!  A filesystem without unnamed files publishes one entry through a temporary that holds a name,
//! and the run that resumes an interrupted copy must remove its own temporary and no other file,
//!  so the name is computed again rather than remembered.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const temporary = temporaryNameOf(stageToken, relativePath); // '.monochromatic-git-policy-cli-partial-' + 16 hex digits
//! ```

use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;

/// What:
///  The name a fallback temporary gets while it is being filled.
/// Why:
///   The fallback is taken only on a filesystem that has no unnamed files,
/// so the name is visible until the publication renames it;
///  the reader must be able to tell it from the person's own files by its name alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const TEMPORARY_PREFIX = '.monochromatic-git-policy-cli-partial-';
/// ```
pub(crate) const TEMPORARY_PREFIX: &str = ".monochromatic-git-policy-cli-partial-";

/// What:
///  How many characters of the temporary's digest are kept.
/// Why:
///   A name is at most `NAME_MAX` bytes and the destination's own name can use all of them,
/// so the temporary's name may not grow with it;
///  sixteen hexadecimal characters name the stage and the entry together,
/// which is enough that two entries of one stage never share one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const TEMPORARY_DIGEST_CHARACTERS = 16;
/// ```
const TEMPORARY_DIGEST_CHARACTERS: usize = 16;

/// What:
///  The events of the first prime of the digest:
/// the one FNV-1a multiplies by.
/// Why:
///   A name that a later release computes again must be the same name,
/// and a hasher chosen by the standard library may change with the compiler.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const FNV_PRIME = 0x100000001b3n;
/// ```
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// What:
///  The offset basis FNV-1a starts from.
/// Why:
///   The same as the prime:
/// the digest is written out here rather than taken from a library whose
///  value may change.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const FNV_OFFSET = 0xcbf29ce484222325n;
/// ```
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;

/// What:
///  A stable sixteen-character name for one entry of one stage.
/// Why:
///   The fallback needs a name that a later run of the same copy computes again,
/// so the run that resumes an interrupted one removes that
///  run's own temporary and no other file;
///  the digest is written out here,
///  so a compiler change cannot rename them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const temporaryName = (token, relativePath) => TEMPORARY_PREFIX + fnv1a(token + '\0' + relativePath).toString(16).padStart(16, '0');
/// ```
pub fn temporary_name_of(
    token: &str,
    relative_path: &[u8],
) -> OsString {
    let mut digest: u64 = FNV_OFFSET;
    for byte in token
        .as_bytes()
        .iter()
        .copied()
        .chain(std::iter::once(0_u8))
        .chain(relative_path.iter().copied())
    {
        digest ^= u64::from(byte);
        digest = digest.wrapping_mul(FNV_PRIME);
    }
    let text: String = format!("{TEMPORARY_PREFIX}{digest:016x}");
    debug_assert_eq!(
        text.len(),
        TEMPORARY_PREFIX.len() + TEMPORARY_DIGEST_CHARACTERS
    );
    return OsString::from_vec(text.into_bytes());
}
