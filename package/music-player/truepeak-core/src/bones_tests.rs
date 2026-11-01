//! Unit tests for the FLAC bones walk,
//!  driven by a synthetic in-memory FLAC stream.

use super::*;

// Independent bitwise CRC-8 (poly 0x07, init 0), so the table in the module is not
// checking itself.
fn crc8_bitwise(bytes: &[u8]) -> u8 {
    return bytes.iter().fold(0u8, |mut crc, &byte| {
        crc ^= byte;
        for _ in 0..8 {
            crc = if crc & 0x80 != 0 { (crc << 1) ^ 0x07 } else { crc << 1 };
        }
        return crc
    })
}

// The standard CRC-8 check vector: "123456789" hashes to 0xF4 under poly 0x07 init 0.
#[test]
fn crc8_table_matches_check_vector() {
    let crc = b"123456789"
        .iter()
        .fold(0u8, |accumulator, &byte| return CRC8_TABLE[usize::from(accumulator ^ byte)]);
    assert_eq!(crc, 0xf4);
    assert_eq!(crc8_bitwise(b"123456789"), 0xf4);
}

// FLAC's UTF-8-style coded numbers decode like UTF-8 code points.
#[test]
fn coded_numbers_decode() {
    assert_eq!(decode_coded_number(&[0x00], 0, 6), Some((0, 1)));
    assert_eq!(decode_coded_number(&[0x7f], 0, 6), Some((127, 1)));
    assert_eq!(decode_coded_number(&[0xc2, 0x80], 0, 6), Some((128, 2)));
    // A continuation byte without its 10 prefix is malformed.
    assert_eq!(decode_coded_number(&[0xc2, 0x40], 0, 6), None);
}

// Build a fixed-blocking stereo 44100 Hz FLAC stream with three frames whose payload
// sizes differ, so the middle frame is the byte-rate hot spot.
fn synthetic_flac(payload_sizes: &[usize]) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(b"fLaC");
    // STREAMINFO: last-block flag + type 0, length 34.
    buf.extend_from_slice(&[0x80, 0x00, 0x00, 0x22]);
    let mut streaminfo = [0u8; 34];
    // min/max block size 4096.
    streaminfo[0] = 0x10;
    streaminfo[1] = 0x00;
    streaminfo[2] = 0x10;
    streaminfo[3] = 0x00;
    // Sample rate 44100 packed as 20 bits at bytes 10..13.
    streaminfo[10] = 0x0a;
    streaminfo[11] = 0xc4;
    streaminfo[12] = 0x40;
    buf.extend_from_slice(&streaminfo);
    for (frame_number, payload) in payload_sizes.iter().enumerate() {
        // Header: sync, fixed blocking, block size bits 12 (4096), rate bits 0
        // (STREAMINFO), stereo, 16-bit, frame number as a one-byte coded number.
        let header = [0xff, 0xf8, 0xc0, 0x18, frame_number as u8];
        buf.extend_from_slice(&header);
        buf.push(crc8_bitwise(&header));
        // Payload bytes carry no 0xff so the walk cannot false-sync inside them.
        buf.extend(std::iter::repeat_n(0x00, *payload));
    }
    return buf
}

// The walk confirms every synthetic frame and measures their byte spans.
#[test]
fn walk_confirms_frames_and_byte_spans() {
    let buf = synthetic_flac(&[300, 3000, 300]);
    let (info, first_frame_offset) = parse_metadata(&buf).expect("metadata parses");
    assert_eq!(info.sample_rate, 44100);
    let (frames, duration_secs) = walk_frames(&buf, first_frame_offset, &info).expect("frames walk");
    assert_eq!(frames.len(), 3);
    // Header is 5 bytes plus the CRC, so each span is payload + 6.
    assert_eq!(frames[0].byte_count, 306);
    assert_eq!(frames[1].byte_count, 3006);
    assert_eq!(frames[2].byte_count, 306);
    assert_eq!(frames[1].start_sample, 4096);
    let expected_secs = 3.0 * 4096.0 / 44100.0;
    assert!((duration_secs - expected_secs).abs() < 1e-9);
}

// The profile spreads bytes over slots and the hot frame's slot wins the bones seed.
#[test]
fn hot_frame_wins_the_bones_seed() {
    let buf = synthetic_flac(&[300, 3000, 300]);
    let profile = flac_bones_profile(&buf).expect("profile builds");
    // Three 4096-sample frames at 44100 Hz last ~0.279 s, so three 0.1 s slots.
    assert_eq!(profile.len(), 3);
    let hot = bones_hot_bins(&profile, 1);
    // The middle frame (samples 4096..8192, ~0.093 s..0.186 s) dominates slot 1.
    assert_eq!(hot, vec![1]);
    // Every frame byte lands somewhere: the profile mass equals the walked span total.
    let mass: f64 = profile.iter().sum();
    assert!((mass - (306.0 + 3006.0 + 306.0)).abs() < 1.0);
}

// Garbage bytes are rejected with a message instead of a panic.
#[test]
fn garbage_is_rejected() {
    let error = flac_bones_profile(&[0u8; 32]).expect_err("garbage must not parse");
    assert!(error.message.contains("fLaC"));
}
