//! Controls for the lock's sentence:
//!  the bytes Git holds,
//!  a cut-short write that covers the nonce,
//!  and the two prefixes that attribute nothing.

use crate::worktree_clone_binding_record::lock_words::*;
use crate::worktree_clone_mount_protocol::RequestNonce;

/// The fixture's nonce.
fn nonce() -> RequestNonce {
    return RequestNonce::parse("9f2c41d7b0a34e6c8f5d1a2b3c4d5e6f")
        .expect("the fixture nonce reads");
}

/// The sentence is the fixed parts around the nonce,
/// and Git's file is that sentence and one newline.
#[test]
fn the_sentence_is_the_fixed_parts_and_the_nonce() {
    let sentence: String = lock_words(&nonce());
    assert_eq!(
        sentence,
        format!("{LOCK_WORDS_PREFIX}{}{LOCK_WORDS_SUFFIX}", nonce().as_str())
    );
    assert!(sentence.ends_with("git worktree remove"));
    let bytes: Vec<u8> = lock_file_bytes(&nonce());
    assert_eq!(bytes.len() as u64, LOCK_WORDS_MAXIMUM_BYTES);
    assert_eq!(bytes.last(), Some(&b'\n'));
}

/// The whole sentence is the only verified arm.
#[test]
fn the_whole_sentence_is_verified() {
    assert_eq!(
        lock_standing_of(lock_file_bytes(&nonce()).as_slice(), &nonce()),
        LockStanding::Verified
    );
    let mut longer: Vec<u8> = lock_file_bytes(&nonce());
    longer.push(b'x');
    assert_eq!(
        lock_standing_of(longer.as_slice(), &nonce()),
        LockStanding::Foreign
    );
}

/// A proper prefix that covers the fixed part and the whole nonce is a cut-short write,
/// and one that stops there reads as no attribution at all.
#[test]
fn a_prefix_attributes_only_when_it_covers_the_nonce() {
    let expected: Vec<u8> = lock_file_bytes(&nonce());
    let covering: usize = LOCK_WORDS_PREFIX.len() + 32 + 1;
    let cut: &[u8] = &expected[..covering];
    assert_eq!(
        lock_standing_of(cut, &nonce()),
        LockStanding::CutShort {
            at: covering as u64
        }
    );
    let short: &[u8] = &expected[..(LOCK_WORDS_PREFIX.len() + 32) as usize];
    assert_eq!(lock_standing_of(short, &nonce()), LockStanding::Foreign);
    assert!(is_unattributable_prefix(short));
    assert!(!is_unattributable_prefix(cut));
}

/// An empty file and a person's words are foreign,
/// and neither is this add's sentence.
#[test]
fn an_empty_file_and_a_persons_words_are_foreign() {
    assert_eq!(lock_standing_of(b"", &nonce()), LockStanding::Foreign);
    assert_eq!(
        lock_standing_of(b"initializing\n", &nonce()),
        LockStanding::Foreign
    );
    assert_eq!(lock_standing_of(b"held\n", &nonce()), LockStanding::Foreign);
}

/// Another add's sentence is foreign here,
/// because the nonce in it is another's.
#[test]
fn another_nonce_is_foreign() {
    let other: RequestNonce =
        RequestNonce::parse("00000000000000000000000000000000").expect("the other nonce reads");
    assert_eq!(
        lock_standing_of(lock_file_bytes(&other).as_slice(), &nonce()),
        LockStanding::Foreign
    );
}
