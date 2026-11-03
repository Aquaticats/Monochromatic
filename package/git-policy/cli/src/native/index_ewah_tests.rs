//! Controls for the EWAH bitmap: byte-exact vectors worked out from `ewah_bitmap.c`, and the round trip of many shapes.

use super::*;

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

/// A single low bit is one literal word; the bit size is one past the bit.
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

/// A word whose every bit is set turns into a run of one-words, as `ewah_set` does when it completes a stream of ones.
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

/// Many shapes read back as the bits they were built from, and written again give the same bytes.
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
        let (read, used) = Ewah::read(bytes.as_slice()).expect("read");
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
        assert!(Ewah::read(&bytes[..cut]).is_none(), "cut at {cut}");
    }
    let mut far: Vec<u8> = bytes.clone();
    let last: usize = far.len() - 1;
    far[last] = 9;
    assert!(Ewah::read(far.as_slice()).is_none(), "a run-length position past the words");
}
