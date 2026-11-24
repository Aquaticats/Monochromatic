//! What:
//!  The lock sentence the add writes into Git's `locked` file,
//!  and the comparison that proves one is the wrapper's.
//! Why:
//!  The sentence is a pure function of the binding's `nonce=`,
//!  written before Git runs,
//!  so no second copy of the words exists anywhere
//!  (`doc/planning/cli-git-zfs-add-time-git-lock-design.md`,
//!  "The reason text" and "What proves ownership").
//!  Git completes the file with one newline,
//!  so the bytes on disk are the sentence and a newline,
//!  and a killed write leaves a prefix:
//!  only a prefix that covers the fixed part and the whole nonce attributes anything,
//!  because a shorter one is what any just-opened lock looks like.
//!  The reason is a message for a person and never authority:
//!  nothing here parses the words for permission.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const words = `monochromatic-git-policy-cli holds this Git registration for a ZFS clone slot (add nonce ${nonce}); remove the worktree with git worktree remove`;
//! const standing = locked.equals(Buffer.from(words + '\n')) ? 'verified' : prefixOf(words + '\n', locked) && locked.length >= fixed ? 'cut-short' : 'foreign';
//! ```

use crate::worktree_clone_mount_protocol::RequestNonce;

/// The sentence's fixed part before the nonce.
pub const LOCK_WORDS_PREFIX: &str = "monochromatic-git-policy-cli holds this Git registration for a ZFS clone slot (add nonce ";

/// The sentence's fixed part after the nonce,
/// which ends with the command a person runs and no newline.
pub const LOCK_WORDS_SUFFIX: &str = "); remove the worktree with git worktree remove";

/// The newline Git's `write_file` completes the words with (`F8` of the facts).
pub const GIT_COMPLETES_WITH: u8 = b'\n';

/// The widest `locked` file this wrapper writes,
/// computed from the sentence's two fixed parts,
/// the nonce's 32 digits and Git's newline.
/// It is not a measured limit and it is not a wait (rule `LQM` lets it cite its own bytes).
pub const LOCK_WORDS_MAXIMUM_BYTES: u64 = LOCK_WORDS_PREFIX.len() as u64 + 32 + LOCK_WORDS_SUFFIX.len() as u64 + 1;

/// What:
///  How one `locked` file stands against the sentence recomposed from a record's nonce.
/// Why:
///  The add's resume,
///  the removal and `doctor` each act differently on the three answers,
///  and the difference between the last two is whether any of this add's nonce is in the file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Standing = { kind: 'verified' } | { kind: 'cut-short'; at: number } | { kind: 'foreign' };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LockStanding {
    /// The file holds the whole sentence and Git's newline.
    Verified,
    /// The file holds a proper prefix that covers the fixed part and the whole nonce.
    CutShort {
        /// How many bytes the killed write left.
        at: u64,
    },
    /// The file holds anything else,
    /// a person's words included.
    Foreign,
}

/// The sentence for one nonce,
/// without the newline Git adds.
pub fn lock_words(nonce: &RequestNonce) -> String {
    return format!("{LOCK_WORDS_PREFIX}{}{LOCK_WORDS_SUFFIX}", nonce.as_str());
}

/// The bytes Git's file holds for one nonce,
/// its newline included.
pub fn lock_file_bytes(nonce: &RequestNonce) -> Vec<u8> {
    let mut bytes: Vec<u8> = lock_words(nonce).into_bytes();
    bytes.push(GIT_COMPLETES_WITH);
    return bytes;
}

/// How one `locked` file's bytes stand against the nonce's sentence.
pub fn lock_standing_of(locked: &[u8], nonce: &RequestNonce) -> LockStanding {
    let expected: Vec<u8> = lock_file_bytes(nonce);
    if locked == expected.as_slice() {
        return LockStanding::Verified;
    }
    let covers_the_nonce: u64 = LOCK_WORDS_PREFIX.len() as u64 + 32;
    let length: u64 = locked.len() as u64;
    if length > covers_the_nonce && length < expected.len() as u64 && locked == &expected[..locked.len()] {
        return LockStanding::CutShort { at: length };
    }
    return LockStanding::Foreign;
}

/// Whether these bytes are a prefix of the sentence Git's own write would complete,
/// empty included.
pub fn is_a_prefix_of_the_sentence(locked: &[u8], nonce: &RequestNonce) -> bool {
    let expected: Vec<u8> = lock_file_bytes(nonce);
    return locked.len() <= expected.len() && locked == &expected[..locked.len()];
}

/// Whether one file's bytes hold a lock Git just opened,
/// empty or too short to attribute,
/// which the add's finder reads as no lock rather than as a person's.
///
/// Why:
///  An empty file is also what a person's `git worktree lock` with no reason leaves,
///  so the removal refuses it (the design's S3) while the add's finder clears the row its other names pick.
pub fn is_unattributable_prefix(locked: &[u8]) -> bool {
    return (locked.len() as u64) <= LOCK_WORDS_PREFIX.len() as u64 + 32;
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "lock_words_tests.rs"]
mod tests;
