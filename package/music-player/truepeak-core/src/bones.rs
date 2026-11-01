//! FLAC frame-size profiling without decoding:
//!  the lossless "encoding bones" channel.
//!
//! Lossless bits track residual entropy,
//!  which tracks signal level,
//!  so a FLAC file's
//! per-time compressed byte-rate points at its loud passages (the crest's slot ranks at
//! the 8th byte-rank percentile at the median on the corpus).
//!  Perceptual codecs were
//! measured useless for this (bits follow busyness,
//!  not height),
//!  so this module is
//! FLAC-only.
//!  The walk reads only container framing:
//!  every frame start is confirmed by
//! header field validation against STREAMINFO plus the header CRC-8 plus the coded
//! frame/sample number matching the previous frame's expectation,
//!  so false syncs inside
//! compressed payloads are rejected.
//!  Frames last ~0.095 s,
//!  nearly one whole 0.1 s slot,
//! so frame bytes are spread overlap-proportionally across the slots they cover;
//! start-time binning aliases into a 2x sawtooth and halves the correlation.

/// Slot length in seconds;
///  matches the shipped probe window so hot slots map to probe bins.
const SLOT_SECS: f64 = 0.1;
/// Smallest plausible frame byte span;
///  the walk skips ahead by this after each frame.
const MIN_FRAME_BYTES: usize = 9;
/// Frame-header block sizes by the 4 block-size bits;
///  0 reserved,
///  6/7 coded at end.
const FRAME_BLOCK_SIZES: [u32; 16] =
    [0, 192, 576, 1152, 2304, 4608, 0, 0, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768];
/// Frame-header sample rates by the 4 rate bits;
///  0 = STREAMINFO,
///  12 to 14 coded at end.
const FRAME_RATES: [u32; 16] =
    [0, 88200, 176400, 192000, 8000, 16000, 22050, 24000, 32000, 44100, 48000, 96000, 0, 0, 0, 0];

/// CRC-8 table for polynomial 0x07,
///  init 0 (the FLAC frame-header CRC).
const CRC8_TABLE: [u8; 256] = build_crc8_table();

/// Build the CRC-8 (poly 0x07) lookup table at compile time.
const fn build_crc8_table() -> [u8; 256] {
    // Fill each seed's remainder by shifting eight times through the polynomial.
    let mut table = [0u8; 256];
    let mut seed = 0usize;
    while seed < 256 {
        let mut crc = seed as u8;
        let mut bit = 0;
        while bit < 8 {
            crc = if crc & 0x80 != 0 { (crc << 1) ^ 0x07 } else { crc << 1 };
            bit += 1;
        }
        table[seed] = crc;
        seed += 1;
    }
    return table
}

/// A bones failure:
///  the bytes are not a walkable FLAC stream.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BonesError {
    /// What went wrong,
    ///  for logs;
    ///  bones are optional so callers degrade to plain probing.
    pub message: String,
}

/// STREAMINFO fields the frame walk validates against.
struct StreamInfo {
    /// Largest block size a frame may claim (0 when the header leaves it unset).
    max_block_size: u32,
    /// The stream's sample rate;
    ///  every frame must agree.
    sample_rate: u32,
}

/// One confirmed frame:
///  where it starts in samples,
///  how long it plays,
///  how many bytes.
struct WalkedFrame {
    /// First sample of the frame.
    start_sample: u64,
    /// Samples the frame carries.
    block_size: u32,
    /// Compressed bytes from this frame start to the next confirmed start.
    byte_count: usize,
}

/// FLAC's UTF-8-style coded number (frame number or sample number) at `offset`.
///
/// What:
///  returns the value and its byte length,
///  or None when malformed.
///  Why:
///  the walk
/// chains frames by expecting the next coded number,
///  killing false syncs.
fn decode_coded_number(buf: &[u8], offset: usize, max_length: usize) -> Option<(u64, usize)> {
    let first = *buf.get(offset)?;
    if first & 0x80 == 0 {
        return Some((u64::from(first), 1));
    }
    // Count the leading ones to get the coded length, then fold the continuation bytes.
    let length = (0..8).take_while(|shift| return first & (0x80 >> shift) != 0).count();
    if !(2..=max_length).contains(&length) {
        return None;
    }
    let mut value = u64::from(first & (0x7f >> length));
    for index in 1..length {
        let byte = *buf.get(offset + index)?;
        if byte & 0xc0 != 0x80 {
            return None;
        }
        value = (value << 6) | u64::from(byte & 0x3f);
    }
    return Some((value, length))
}

/// A parsed frame header:
///  blocking strategy,
///  block size,
///  coded number,
///  header length.
struct FrameHeader {
    /// 1 = variable blocking (sample-number coded),
    ///  0 = fixed (frame-number coded).
    blocking_strategy: u8,
    /// Samples this frame carries.
    block_size: u32,
    /// The coded frame number (fixed) or start sample (variable).
    coded_number: u64,
}

/// Try to parse a frame header at `offset`;
///  None on any mismatch.
///
/// What:
///  14-bit sync,
///  reserved-field checks,
///  STREAMINFO cross-checks,
///  then CRC-8 over
/// the header bytes.
///  Why:
///  every check that fails here is a false sync avoided.
fn parse_frame_header(buf: &[u8], offset: usize, info: &StreamInfo) -> Option<FrameHeader> {
    if offset + 6 > buf.len() {
        return None;
    }
    if buf[offset] != 0xff || buf[offset + 1] & 0xfe != 0xf8 {
        return None;
    }
    let blocking_strategy = buf[offset + 1] & 1;
    let block_size_bits = usize::from(buf[offset + 2] >> 4);
    let rate_bits = usize::from(buf[offset + 2] & 0x0f);
    if block_size_bits == 0 || rate_bits == 15 {
        return None;
    }
    let channel_bits = buf[offset + 3] >> 4;
    let sample_size_bits = (buf[offset + 3] >> 1) & 7;
    if channel_bits > 10 || sample_size_bits == 3 || buf[offset + 3] & 1 != 0 {
        return None;
    }
    let max_number_length = if blocking_strategy == 1 { 7 } else { 6 };
    let (coded_number, number_length) = decode_coded_number(buf, offset + 4, max_number_length)?;
    let mut cursor = offset + 4 + number_length;
    // Block size: table value, or coded at the header's end for bits 6 and 7.
    let mut block_size = FRAME_BLOCK_SIZES[block_size_bits];
    if block_size_bits == 6 {
        block_size = u32::from(*buf.get(cursor)?) + 1;
        cursor += 1;
    } else if block_size_bits == 7 {
        block_size = (u32::from(*buf.get(cursor)?) << 8 | u32::from(*buf.get(cursor + 1)?)) + 1;
        cursor += 2;
    }
    // Sample rate: STREAMINFO, table value, or coded at the end for bits 12 to 14.
    let mut sample_rate = if rate_bits == 0 { info.sample_rate } else { FRAME_RATES[rate_bits] };
    if rate_bits == 12 {
        sample_rate = u32::from(*buf.get(cursor)?) * 1000;
        cursor += 1;
    } else if rate_bits == 13 || rate_bits == 14 {
        let coded = u32::from(*buf.get(cursor)?) << 8 | u32::from(*buf.get(cursor + 1)?);
        sample_rate = coded * if rate_bits == 14 { 10 } else { 1 };
        cursor += 2;
    }
    if sample_rate != info.sample_rate {
        return None;
    }
    if info.max_block_size >= 16 && block_size > info.max_block_size {
        return None;
    }
    // CRC-8 over every header byte up to the CRC itself.
    let crc_byte = *buf.get(cursor)?;
    let crc = buf[offset..cursor]
        .iter()
        .fold(0u8, |accumulator, &byte| return CRC8_TABLE[usize::from(accumulator ^ byte)]);
    if crc != crc_byte {
        return None;
    }
    return Some(FrameHeader { blocking_strategy, block_size, coded_number })
}

/// Parse the metadata blocks:
///  STREAMINFO plus the first audio-frame offset.
///
/// What:
///  skips a nonstandard leading ID3v2 tag,
///  requires the fLaC magic,
///  walks blocks
/// to the last-block flag.
///  Why:
///  the frame walk needs the stream's ground truth and its
/// starting offset.
fn parse_metadata(buf: &[u8]) -> Result<(StreamInfo, usize), BonesError> {
    let mut offset = 0usize;
    if buf.len() >= 10 && &buf[0..3] == b"ID3" {
        let tag_size = (usize::from(buf[6] & 0x7f) << 21)
            | (usize::from(buf[7] & 0x7f) << 14)
            | (usize::from(buf[8] & 0x7f) << 7)
            | usize::from(buf[9] & 0x7f);
        let footer = if buf[5] & 0x10 != 0 { 10 } else { 0 };
        offset = 10 + tag_size + footer;
    }
    if buf.len() < offset + 4 || &buf[offset..offset + 4] != b"fLaC" {
        return Err(BonesError { message: "flac: no fLaC magic".to_owned() });
    }
    offset += 4;
    let mut info: Option<StreamInfo> = None;
    // Walk metadata blocks until the last-block flag.
    loop {
        if offset + 4 > buf.len() {
            return Err(BonesError { message: "flac: truncated metadata".to_owned() });
        }
        let head = buf[offset];
        let last = head & 0x80 != 0;
        let block_type = head & 0x7f;
        let length = (usize::from(buf[offset + 1]) << 16)
            | (usize::from(buf[offset + 2]) << 8)
            | usize::from(buf[offset + 3]);
        let content = offset + 4;
        if content + length > buf.len() {
            return Err(BonesError { message: "flac: metadata block past EOF".to_owned() });
        }
        if block_type == 0 && length >= 18 {
            // STREAMINFO: max block size at bytes 2..4, rate is 20 bits at bytes 10..13.
            let max_block_size = u32::from(buf[content + 2]) << 8 | u32::from(buf[content + 3]);
            let sample_rate = (u32::from(buf[content + 10]) << 12)
                | (u32::from(buf[content + 11]) << 4)
                | (u32::from(buf[content + 12]) >> 4);
            info = Some(StreamInfo { max_block_size, sample_rate });
        }
        offset = content + length;
        if last {
            break;
        }
    }
    // The confirmed STREAMINFO gets its own name so it does not shadow the
    // accumulator `Option` above.
    let stream_info = info.ok_or_else(|| return BonesError { message: "flac: no STREAMINFO block".to_owned() })?;
    if stream_info.sample_rate == 0 {
        return Err(BonesError { message: "flac: zero sample rate".to_owned() });
    }
    return Ok((stream_info, offset))
}

/// Walk audio frames from the first frame to EOF,
///  confirming each start.
///
/// What:
///  chains confirmed headers by the expected coded number;
///  frame byte size is the
/// distance between consecutive confirmed starts.
///  Why:
///  the byte spans are the profile.
fn walk_frames(buf: &[u8], first_frame_offset: usize, info: &StreamInfo) -> Result<(Vec<WalkedFrame>, f64), BonesError> {
    let first = parse_frame_header(buf, first_frame_offset, info)
        .ok_or_else(|| return BonesError { message: "flac: no frame at first-frame offset".to_owned() })?;
    let blocking_strategy = first.blocking_strategy;
    // Fixed blocking: start sample = frame number times the constant first block size.
    let nominal_block_size = u64::from(first.block_size);
    let start_sample_of = |header: &FrameHeader| {
        if header.blocking_strategy == 1 { return header.coded_number } else { return header.coded_number * nominal_block_size }
    };
    let mut frames: Vec<WalkedFrame> = Vec::new();
    let mut previous_offset = first_frame_offset;
    let mut previous = first;
    let mut search_from = first_frame_offset + MIN_FRAME_BYTES;
    while search_from < buf.len() {
        // Jump to the next 0xff sync candidate.
        let Some(found) = buf[search_from..].iter().position(|&byte| return byte == 0xff) else {
            break;
        };
        let candidate = search_from + found;
        let Some(header) = parse_frame_header(buf, candidate, info) else {
            search_from = candidate + 1;
            continue;
        };
        if header.blocking_strategy != blocking_strategy {
            search_from = candidate + 1;
            continue;
        }
        let expected = if blocking_strategy == 1 {
            start_sample_of(&previous) + u64::from(previous.block_size)
        } else {
            previous.coded_number + 1
        };
        if header.coded_number != expected {
            search_from = candidate + 1;
            continue;
        }
        frames.push(WalkedFrame {
            start_sample: start_sample_of(&previous),
            block_size: previous.block_size,
            byte_count: candidate - previous_offset,
        });
        previous_offset = candidate;
        previous = header;
        search_from = candidate + MIN_FRAME_BYTES;
    }
    let last_start = start_sample_of(&previous);
    frames.push(WalkedFrame {
        start_sample: last_start,
        block_size: previous.block_size,
        byte_count: buf.len() - previous_offset,
    });
    let duration_secs = (last_start + u64::from(previous.block_size)) as f64 / f64::from(info.sample_rate);
    return Ok((frames, duration_secs))
}

/// Build the per-slot byte profile from a FLAC file's raw bytes,
///  without decoding.
///
/// What:
///  parses metadata,
///  walks every frame,
///  and spreads each frame's bytes across the
/// 0.1 s slots it overlaps,
///  proportionally to time overlap.
///  Why:
///  the profile's hot
/// slots seed the probe;
///  spreading avoids the start-bin sawtooth artifact.
pub fn flac_bones_profile(buf: &[u8]) -> Result<Vec<f64>, BonesError> {
    let (info, first_frame_offset) = parse_metadata(buf)?;
    let (frames, duration_secs) = walk_frames(buf, first_frame_offset, &info)?;
    let slot_count = ((duration_secs / SLOT_SECS).ceil() as usize).max(1);
    let mut slots = vec![0.0f64; slot_count];
    let rate = f64::from(info.sample_rate);
    for frame in &frames {
        let start_secs = frame.start_sample as f64 / rate;
        let end_secs = (frame.start_sample + u64::from(frame.block_size)) as f64 / rate;
        if end_secs <= start_secs {
            continue;
        }
        // Spread this frame's bytes across every slot it overlaps.
        let first_slot = (start_secs / SLOT_SECS + 1e-9).floor() as usize;
        let last_slot = (((end_secs / SLOT_SECS - 1e-9).ceil() as usize).max(first_slot + 1) - 1)
            .min(slot_count - 1);
        for (step, weight) in slots[first_slot..=last_slot].iter_mut().enumerate() {
            let slot = first_slot + step;
            let lo = start_secs.max(slot as f64 * SLOT_SECS);
            let hi = end_secs.min((slot + 1) as f64 * SLOT_SECS);
            if hi <= lo {
                continue;
            }
            *weight += frame.byte_count as f64 * (hi - lo) / (end_secs - start_secs);
        }
    }
    return Ok(slots)
}

/// The indices of the `top` hottest byte slots,
///  the probe's bones seeds.
///
/// What:
///  sorts slot indices by byte weight descending and keeps the first `top`.
///  Why:
/// the resolver decodes these slots (each with its neighbors) before the even pass.
pub fn bones_hot_bins(profile: &[f64], top: usize) -> Vec<usize> {
    // Sort indices by descending byte weight, then truncate.
    let mut order: Vec<usize> = (0..profile.len()).collect();
    order.sort_by(|&a, &b| return f64::total_cmp(&profile[b], &profile[a]));
    order.truncate(top);
    return order
}

/// What:
///      `#[cfg(test)] #[path = "bones_tests.rs"] mod tests;`.
///  Test-only submodule in
///           the sibling file,
///  gated to test builds.
/// Why:
///       Keep this file to production code;
///  sibling `*_tests.rs` is max-lines exempt.
#[cfg(test)]
#[path = "bones_tests.rs"]
mod tests;
