//! Unit tests for file-derived probe inputs.

use super::*;
use crate::policy::default_policy;

// Independent bitwise CRC-8 (poly 0x07, init 0) for building synthetic frames.
fn crc8_bitwise(bytes: &[u8]) -> u8 {
    return bytes.iter().fold(0u8, |mut crc, &byte| {
        crc ^= byte;
        for _ in 0..8 {
            crc = if crc & 0x80 != 0 { (crc << 1) ^ 0x07 } else { crc << 1 };
        }
        return crc
    })
}

// A minimal fixed-blocking FLAC stream (same shape as the bones tests build).
fn synthetic_flac(payload_sizes: &[usize]) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(b"fLaC");
    buf.extend_from_slice(&[0x80, 0x00, 0x00, 0x22]);
    let mut streaminfo = [0u8; 34];
    streaminfo[0] = 0x10;
    streaminfo[2] = 0x10;
    streaminfo[10] = 0x0a;
    streaminfo[11] = 0xc4;
    streaminfo[12] = 0x40;
    buf.extend_from_slice(&streaminfo);
    for (frame_number, payload) in payload_sizes.iter().enumerate() {
        let header = [0xff, 0xf8, 0xc0, 0x18, frame_number as u8];
        buf.extend_from_slice(&header);
        buf.push(crc8_bitwise(&header));
        buf.extend(std::iter::repeat_n(0x00, *payload));
    }
    return buf
}

// FLAC bytes yield lossless provenance with bones; the hot frame's slot leads the seeds.
#[test]
fn flac_bytes_yield_lossless_with_bones() {
    let bytes = synthetic_flac(&[300, 3000, 300]);
    let (provenance, maybe_bones) = probe_inputs_from_bytes(&bytes, &default_policy());
    assert!(provenance.lossless);
    let bones = maybe_bones.expect("a walkable flac yields bones");
    assert_eq!(bones.first(), Some(&1));
}

// An ID3-prefixed FLAC still sniffs as FLAC.
#[test]
fn id3_prefixed_flac_sniffs() {
    let mut bytes = vec![
        b'I', b'D', b'3', 3, 0, 0, // header, no footer flag
        0, 0, 0, 2, // syncsafe size 2
        0, 0, // the two tag bytes
    ];
    bytes.extend_from_slice(&synthetic_flac(&[300, 3000, 300]));
    let (provenance, bones) = probe_inputs_from_bytes(&bytes, &default_policy());
    assert!(provenance.lossless);
    assert!(bones.is_some());
}

// Non-FLAC bytes land in the uninformed (bare) provenance with no bones.
#[test]
fn other_bytes_stay_uninformed() {
    let (provenance, bones) = probe_inputs_from_bytes(b"OggS not a flac", &default_policy());
    assert_eq!(provenance, TrackProvenance::unknown());
    assert!(bones.is_none());
}

// A FLAC magic with an unwalkable body keeps lossless provenance, without bones.
#[test]
fn broken_flac_keeps_lossless_without_bones() {
    let (provenance, bones) = probe_inputs_from_bytes(b"fLaCgarbage", &default_policy());
    assert!(provenance.lossless);
    assert!(bones.is_none());
}

// A missing file degrades to the uninformed provenance.
#[test]
fn missing_file_stays_uninformed() {
    let (provenance, bones) =
        probe_inputs_from_file(Path::new("/nonexistent/no-such-track.flac"), &default_policy());
    assert_eq!(provenance, TrackProvenance::unknown());
    assert!(bones.is_none());
}
