//! Controls for the EWAH bitmap:
//! byte-exact vectors worked out from `ewah_bitmap.c`,
//! and the round trip of many shapes.

use super::*;

/// More bits than any shape below sets.
const BIT_LIMIT: usize = 1 << 20;

/// The bytes of a bitmap with one literal word after one run-length word.
fn one_literal(bit_size: u32, literal: u64) -> Vec<u8> {
    let mut bytes: Vec<u8> = Vec::new();
    bytes.extend_from_slice(&bit_size.to_be_bytes());
    bytes.extend_from_slice(&2u32.to_be_bytes());
    // A run-length word with no run and one literal word: the literal count sits above the 33 low bits.
    bytes.extend_from_slice(&(1u64 << 33).to_be_bytes());
    bytes.extend_from_slice(&literal.to_be_bytes());
    bytes.extend_from_slice(&0u32.to_be_bytes());
    return bytes;
}

/// A single low bit is one literal word;
/// the bit size is one past the bit.
#[test]
fn a_single_bit_is_one_literal_word() {
    let mut bytes: Vec<u8> = Vec::new();
    Ewah::from_bits(&[0]).serialize(&mut bytes);
    assert_eq!(bytes, one_literal(1, 1));
    let mut high: Vec<u8> = Vec::new();
    Ewah::from_bits(&[WORD_BITS - 1]).serialize(&mut high);
    assert_eq!(high, one_literal(WORD_BITS as u32, 1 << (WORD_BITS - 1)));
}

/// A gap of whole words becomes a run of empty words before the literal.
#[test]
fn a_gap_of_empty_words_is_a_run() {
    let mut bytes: Vec<u8> = Vec::new();
    Ewah::from_bits(&[3 * WORD_BITS + 1]).serialize(&mut bytes);
    let mut expected: Vec<u8> = Vec::new();
    expected.extend_from_slice(&((3 * WORD_BITS + 2) as u32).to_be_bytes());
    expected.extend_from_slice(&2u32.to_be_bytes());
    // A run of three empty words, then one literal word.
    expected.extend_from_slice(&((3u64 << 1) | (1u64 << 33)).to_be_bytes());
    expected.extend_from_slice(&2u64.to_be_bytes());
    expected.extend_from_slice(&0u32.to_be_bytes());
    assert_eq!(bytes, expected);
}

/// A word whose every bit is set turns into a run of one-words,
/// as `ewah_set` does when it completes a stream of ones.
#[test]
fn a_full_word_becomes_a_run_of_ones() {
    let bits: Vec<usize> = (0..WORD_BITS).collect();
    let bitmap: Ewah = Ewah::from_bits(bits.as_slice());
    let mut bytes: Vec<u8> = Vec::new();
    bitmap.serialize(&mut bytes);
    let mut expected: Vec<u8> = Vec::new();
    expected.extend_from_slice(&(WORD_BITS as u32).to_be_bytes());
    expected.extend_from_slice(&1u32.to_be_bytes());
    expected.extend_from_slice(&(1u64 | (1u64 << 1)).to_be_bytes());
    expected.extend_from_slice(&0u32.to_be_bytes());
    assert_eq!(bytes, expected);
    assert_eq!(bitmap.bits(), bits);
}

/// Many shapes read back as the bits they were built from,
/// and written again give the same bytes.
#[test]
fn shapes_round_trip_through_bytes() {
    let shapes: Vec<Vec<usize>> = vec![
        Vec::new(),
        vec![0, 1, 2, 63, 64, 65, 200],
        (0..300).collect(),
        (0..300).step_by(7).collect(),
        vec![5, 5 + 64 * 100, 5 + 64 * 100 + 1],
        (10..20).chain(64 * 4..64 * 6).chain(64 * 9 + 3..64 * 9 + 5).collect(),
    ];
    for bits in shapes {
        let bitmap: Ewah = Ewah::from_bits(bits.as_slice());
        let mut bytes: Vec<u8> = Vec::new();
        bitmap.serialize(&mut bytes);
        let (read, used) = Ewah::read(bytes.as_slice(), BIT_LIMIT).expect("read");
        assert_eq!(used, bytes.len());
        assert_eq!(read.bits(), bits);
        let mut again: Vec<u8> = Vec::new();
        read.serialize(&mut again);
        assert_eq!(again, bytes);
        assert_eq!(read.bit_size(), bits.last().map_or(0, |last| { return last + 1; }));
    }
}

/// Truncated or inconsistent bytes are not a bitmap.
#[test]
fn damaged_bytes_are_not_a_bitmap() {
    let mut bytes: Vec<u8> = Vec::new();
    Ewah::from_bits(&[1, 2, 3]).serialize(&mut bytes);
    for cut in 0..bytes.len() {
        assert!(Ewah::read(&bytes[..cut], BIT_LIMIT).is_none(), "cut at {cut}");
    }
    let mut far: Vec<u8> = bytes.clone();
    let last: usize = far.len() - 1;
    far[last] = 9;
    assert!(Ewah::read(far.as_slice(), BIT_LIMIT).is_none(), "a run-length position past the words");
}

/// The bytes of a bitmap of `bit_size` bits made of the given words,
/// with the last run-length word at `rlw`.
fn raw(bit_size: u32, words: &[u64], rlw: u32) -> Vec<u8> {
    let mut bytes: Vec<u8> = Vec::new();
    bytes.extend_from_slice(&bit_size.to_be_bytes());
    bytes.extend_from_slice(&(words.len() as u32).to_be_bytes());
    for word in words {
        bytes.extend_from_slice(&word.to_be_bytes());
    }
    bytes.extend_from_slice(&rlw.to_be_bytes());
    return bytes;
}

/// A bitmap whose runs describe far more bits than its bit size,
/// whose literal count passes the words it has,
/// or whose last
/// run-length word is not where it says is not one Git's writer makes,
/// and reading it must not expand it.
#[test]
fn inconsistent_bitmaps_are_not_read() {
    // One run-length word: a run of 2^20 words of ones, with a bit size of one bit.
    let long_run: u64 = 1 | ((1u64 << 20) << 1);
    assert!(Ewah::read(raw(1, &[long_run], 0).as_slice(), BIT_LIMIT).is_none(), "a run past the bit size");
    // One run-length word that announces five literal words and has none.
    assert!(Ewah::read(raw(64, &[5u64 << 33], 0).as_slice(), BIT_LIMIT).is_none(), "literals past the words");
    // A run-length word with one literal, then another run-length word, and the position names the first.
    assert!(Ewah::read(raw(64, &[1u64 << 33, 1, 0], 0).as_slice(), BIT_LIMIT).is_none(), "the last run-length word is elsewhere");
}

/// A bitmap of more bits than the caller can use is not read,
/// however consistent it is;
/// one of exactly that size is.
#[test]
fn a_bitmap_larger_than_the_caller_can_use_is_not_read() {
    let mut bytes: Vec<u8> = Vec::new();
    Ewah::from_bits(&[0, 9]).serialize(&mut bytes);
    assert!(Ewah::read(bytes.as_slice(), 10).is_some());
    assert!(Ewah::read(bytes.as_slice(), 9).is_none());
}

/// The twenty-byte bitmap of the review,
/// a run of 2^32 minus 1 words of ones under a bit size of one,
/// is refused before
/// anything is listed.
#[test]
fn the_review_s_tiny_bitmap_is_refused() {
    let bytes: Vec<u8> = raw(1, &[0x0000_0001_ffff_ffff], 0);
    assert_eq!(bytes.len(), 20);
    assert!(Ewah::read(bytes.as_slice(), usize::MAX).is_none());
}
