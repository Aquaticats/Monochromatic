//! What:
//!  The digest a push's id carries for its branch,
//! and what may follow the kind in an id.
//! Why:
//!  A record's id is its file name and its idempotency key,
//! so it must hold every choice the id stands for:
//! the commit on `refs/heads/topic` and the same commit on `refs/heads/release` are two pushes to two remotes,
//! and the queue's key is the only thing that keeps their records apart.
//! The digest rules sit beside the record rather than in it,
//! because they are about a name and not about the work.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const branchDigest = (ref) => sha256(ref).slice(0, 8); const isIdSuffix = (kind, suffix) => …
//! ```

use crate::subvolume_sha::sha256;
use crate::work_item_kinds_and_their_names::WorkKind;

/// The most characters an object name has:
/// 64 hexadecimal digits of a SHA-256 name.
/// With the longest kind,
/// a generation,
/// a reading that
/// fills every field and the suffix of a recovery's temporary file,
/// such an id still makes a name of at most 223 bytes,
/// under
/// the 255 a file name may have;
/// a longer id could make a claim's name fit and the name of its temporary file not.
pub(crate) const MAX_OBJECT_NAME_LENGTH: usize = 64;

/// Whether `object_name` is what follows the kind in an id:
/// 1 to `MAX_OBJECT_NAME_LENGTH` ASCII letters and digits,
/// so it
/// holds no separator of a claim's name and no character a file name treats specially.
pub(crate) fn is_object_name(object_name: &str) -> bool {
    return !object_name.is_empty() && object_name.len() <= MAX_OBJECT_NAME_LENGTH && object_name.as_bytes().iter().all(u8::is_ascii_alphanumeric);
}

/// Whether `id` is a kind and an object name,
/// which makes every name built from it fit in a file name.
pub(crate) fn is_valid_id(id: &str) -> bool {
    for kind in [WorkKind::Review, WorkKind::Push, WorkKind::Maintenance, WorkKind::Trash, WorkKind::Prewarm, WorkKind::Relink, WorkKind::SubvolumeDelete] {
        if let Some(rest) = id.strip_prefix(format!("{}-", kind.name()).as_str())
            && crate::work_item_ids_and_names::is_id_suffix(&kind, rest)
        {
            return true;
        }
    }
    return false;
}


/// The most characters the part of an id after its kind may have:
/// an object name,
/// and for a push the branch digest a hyphen separates from it.
/// With the longest kind,
/// a generation,
/// a reading that fills every field and the suffix of a recovery's temporary file,
/// such an id still makes a name of at most 233 bytes,
/// under the 255 a file name may have;
/// a longer id could make a claim's name fit and the name of its temporary file not.
pub(crate) const MAX_ID_SUFFIX_LENGTH: usize = MAX_OBJECT_NAME_LENGTH + 1 + BRANCH_DIGEST_LENGTH;

/// The hexadecimal digits of the digest a push's id carries for its branch.
/// Eight bytes of SHA-256 name one branch as well as any longer digest does:
/// two ref names that a push's record confuses would have to be found,
/// and no one does that by accident.
const BRANCH_DIGEST_LENGTH: usize = 16;

/// What:
///  The digest of a branch reference that a push's id carries.
/// Why:
///   A record's id is its file name and its idempotency key,
/// so it must hold the branch:
/// the same commit on two branches is two pushes to two remotes.
/// Only ASCII letters and digits may join the id,
/// because the id is part of a claim's name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// createHash('sha256').update(branchRef).digest('hex').slice(0, 16)
/// ```
pub(crate) fn branch_digest(branch_ref: &str) -> String {
    let full: [u8; 32] = sha256(branch_ref.as_bytes());
    return crate::hexadecimal_of_bytes::hexadecimal_of_bytes(&full[..BRANCH_DIGEST_LENGTH / 2]);
}

/// Whether `digest` is a branch digest as `branch_digest` writes one:
/// `BRANCH_DIGEST_LENGTH` lowercase hexadecimal digits.
fn is_branch_digest(digest: &str) -> bool {
    return digest.len() == BRANCH_DIGEST_LENGTH && digest.as_bytes().iter().all(is_lowercase_hexadecimal_digit);
}

/// Whether a byte is a lowercase hexadecimal digit,
/// the only digits a branch digest holds.
fn is_lowercase_hexadecimal_digit(byte: &u8) -> bool {
    return byte.is_ascii_digit() || (b'a'..=b'f').contains(byte);
}

/// What:
///  Whether `suffix` is what follows the kind in an id of `kind`,
/// which makes every name built from the id fit in a file name.
/// Why:
///   A push's suffix carries the branch its record is for,
/// so the check must read the object name and the digest separately;
/// every other kind's suffix is the object name alone,
/// and a push record an earlier release named by the kind and the object alone keeps its name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (kind) => (suffix) => kind === 'push' ? /^[\w]{1,64}(-[0-9a-f]{16})?$/.test(suffix) : /^[\w]{1,64}$/.test(suffix)
/// ```
pub(crate) fn is_id_suffix(kind: &WorkKind, suffix: &str) -> bool {
    if *kind == WorkKind::Push
        && let Some((object_name, digest)) = suffix.split_once('-')
    {
        return is_object_name(object_name) && is_branch_digest(digest);
    }
    return is_object_name(suffix);
}

/// What:
///  The commit a push record's name is for:
/// the object after the kind,
/// without the digest this release carries for the branch.
/// Why:
///   A command that reads a held record says which commit is held,
/// and this release's name carries the branch digest after the object,
///  which is no part of a commit's name.
/// A record an earlier release named by the kind and the object alone answers the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const objectOfRecordName = (name) => name.startsWith('push-') ? name.slice(5).replace(/-[0-9a-f]{16}$/, '') : undefined
/// ```
pub(crate) fn object_of_a_record_name(name: &str) -> Option<&str> {
    let kind: String = format!("{}-", WorkKind::Push.name());
    let rest: &str = name.strip_prefix(kind.as_str())?;
    let object_name: &str = match rest.split_once('-') {
        Some((object_name, digest)) if is_branch_digest(digest) => object_name,
        _ => rest,
    };
    return is_object_name(object_name).then_some(object_name);
}

/// The id-name rules are controlled in the test module.
#[cfg(test)]
#[path = "work_item_ids_and_names_tests.rs"]
mod tests;
