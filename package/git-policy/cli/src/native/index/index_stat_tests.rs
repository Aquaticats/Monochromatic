//! Controls for the stat fields and Git's varint.

use super::*;

/// The varints Git's `encode_varint` makes for the values at the edges of each length.
#[test]
fn varints_match_the_values_git_writes() {
    for (value, expected) in [(0u64, vec![0x00u8]), (127, vec![0x7f]), (128, vec![0x80, 0x00]), (129, vec![0x80, 0x01]), (16_511, vec![0xff, 0x7f]), (16_512, vec![0x80, 0x80, 0x00])] {
        let mut out: Vec<u8> = Vec::new();
        encode_varint(value, &mut out);
        assert_eq!(out, expected, "encode {value}");
        assert_eq!(decode_varint(out.as_slice()), Some((value, &[][..])), "decode {value}");
    }
}

/// A varint cut off in the middle,
/// and one too long for 64 bits,
/// are not read.
#[test]
fn damaged_varints_are_not_read() {
    assert_eq!(decode_varint(&[0x80]), None);
    assert_eq!(decode_varint(&[]), None);
    assert_eq!(decode_varint(&[0xff; 11]), None);
}

/// The untracked cache's nine fields keep their order through bytes.
#[test]
fn stat_fields_round_trip() {
    let stat: StatData = StatData { ctime_sec: 1, ctime_nsec: 2, mtime_sec: 3, mtime_nsec: 4, dev: 5, ino: 6, uid: 7, gid: 8, size: 9 };
    let mut out: Vec<u8> = Vec::new();
    stat.to_cache_bytes(&mut out);
    assert_eq!(out.len(), CACHE_STAT_LENGTH);
    assert_eq!(StatData::from_cache_bytes(out.as_slice()), Some(stat));
    assert_eq!(StatData::from_cache_bytes(&out[..CACHE_STAT_LENGTH - 1]), None);
}
