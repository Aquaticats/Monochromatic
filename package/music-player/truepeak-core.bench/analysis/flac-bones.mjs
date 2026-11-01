// FLAC frame-size bones: no-decode byte-rate profiles for the lossless bucket.
// Lossy byte-rate profiles failed to locate crests (bits follow spectral busyness,
// not peak height; see correlate.mjs). FLAC is different in principle: bits track
// residual entropy which tracks signal level much more directly. This script
// 1. parses each target FLAC's framing only (never decodes a sample) into 0.1 s
//    byte slots (out/flac-profiles.jsonl),
// 2. validates profile durations against the decoded corpus and spot-checks
//    byte-vs-peak correlation,
// 3. measures the decisive number: the crest bin's byte-rank percentile,
// 4. if the rank is strong, simulates a bones-guided low-coverage probe.
//
// Usage: node analysis/flac-bones.mjs [extract|analyze|simulate|all]  (default: all;
// simulate forces the probe simulation even when the rank gate fails)
import { once } from 'node:events';
import {
  createReadStream,
  createWriteStream
} from 'node:fs';
import { readFile } from 'node:fs/promises';
import { createInterface } from 'node:readline';
import { pathToFileURL } from 'node:url';
import { SLOT_SECS } from './bones-parsers.mjs';
import {
  db,
  loadTracks,
  quantile,
  sampledMaxEven
} from './corpus.mjs';

//region Constants and paths
const CORPUS_PATH = new URL(
  '../out/tracks-fine.jsonl',
  import.meta.url
).pathname;
const PROFILES_PATH = new URL(
  '../out/flac-profiles.jsonl',
  import.meta.url
).pathname;
/** Loud threshold (-1 dBTP) as a linear peak; matches the bench ceiling. */
const LOUD_LINEAR = 10 ** ((-1) / 20);
/** Tracks at or under this duration decode fully under the shipped policy. */
const SHORT_MAX_SECS = 90;
/** Seek points spaced wider than this are too coarse; walk frame headers instead. */
const SEEK_GRANULARITY_MAX_SECS = 2;
/** Duration mismatch beyond this many seconds counts as a parse failure. */
const MISMATCH_LIMIT_SECS = 0.5;
/** Smallest conceivable FLAC frame (header + one constant subframe + CRC-16). */
const MIN_FRAME_BYTES = 9;
/** How many files are parsed concurrently; readFile dominates, so keep this small. */
const POOL_SIZE = 4;
//endregion

//region FLAC bitstream tables
/** CRC-8 lookup table, polynomial 0x07, init 0 (FLAC frame-header CRC). */
const CRC8_TABLE = new Uint8Array(256).map((
  _,
  seed
) => {
  let crc = seed;
  for (let bit = 0; bit < 8; bit += 1) {
    crc = (crc & 0x80) !== 0 ? ((crc << 1) ^ 0x07) & 0xFF : (crc << 1) & 0xFF;
  }
  return crc;
});
/** Frame-header block sizes by the 4 block-size bits; 0 reserved, 6/7 coded at end. */
const FRAME_BLOCK_SIZES = [
  0,
  192,
  576,
  1_152,
  2_304,
  4_608,
  0,
  0,
  256,
  512,
  1_024,
  2_048,
  4_096,
  8_192,
  16_384,
  32_768
];
/** Frame-header sample rates by the 4 rate bits; 0 = STREAMINFO, 12-14 coded at end. */
const FRAME_RATES = [
  0,
  88_200,
  176_400,
  192_000,
  8_000,
  16_000,
  22_050,
  24_000,
  32_000,
  44_100,
  48_000,
  96_000,
  0,
  0,
  0,
  0
];
//endregion

//region Slot helpers (shared shape with bones-parsers.mjs)
/** Add payload bytes into the slot holding a frame's start time (sparse-safe). */
function addBytes({
  slots,
  slot,
  byteCount
}) {
  slots[slot] = (slots[slot] ?? 0) + byteCount;
}

/** Replace sparse holes with zeros, extend to the full duration, round to bytes. */
function finalizeSlots({
  slots,
  durationSecs
}) {
  const length = Math.max(
    slots.length,
    Math.ceil((durationSecs / SLOT_SECS) - 1e-9)
  );
  return Array.from(
    { length },
    (
      _,
      index
    ) => Math.round(slots[index] ?? 0)
  );
}

/** Spread an interval's bytes across 0.1 s slots proportionally to time overlap. */
function spreadBytes({
  slots,
  startSecs,
  endSecs,
  byteCount
}) {
  if (endSecs <= startSecs) return;
  const firstSlot = Math.floor((startSecs / SLOT_SECS) + 1e-9);
  const lastSlot = Math.max(
    firstSlot,
    Math.ceil((endSecs / SLOT_SECS) - 1e-9) - 1
  );
  for (let slot = firstSlot; slot <= lastSlot; slot += 1) {
    const lo = Math.max(
      startSecs,
      slot * SLOT_SECS
    );
    const hi = Math.min(
      endSecs,
      (slot + 1) * SLOT_SECS
    );
    if (hi <= lo) continue;
    addBytes({
      slots,
      slot,
      byteCount: (byteCount * (hi - lo)) / (endSecs - startSecs)
    });
  }
}
//endregion

//region FLAC metadata parsing
/**
 Walk the metadata blocks after the "fLaC" magic (skipping a nonstandard leading
 ID3v2 tag). Returns STREAMINFO fields, SEEKTABLE points (sample number plus byte
 offset relative to the first frame), and the first audio-frame byte offset.
 */
function parseMetadata(buf) {
  let offset = 0;
  if ((buf.length >= 10) && (buf.toString(
    'latin1',
    0,
    3
  ) === 'ID3')) {
    const tagSize = ((buf[6] & 0x7F) << 21) | ((buf[7] & 0x7F) << 14)
      | ((buf[8] & 0x7F) << 7)
      | (buf[9] & 0x7F);
    const footer = (buf[5] & 0x10) !== 0 ? 10 : 0;
    offset = 10 + tagSize
      + footer;
  }
  if (buf.toString(
    'latin1',
    offset,
    offset + 4
  ) !== 'fLaC') throw new Error('flac: no fLaC magic');
  offset += 4;
  let streamInfo = null;
  const seekPoints = [];
  let last = false;
  while (!last) {
    if ((offset + 4) > buf.length) throw new Error('flac: truncated metadata');
    const head = buf[offset];
    last = (head & 0x80) !== 0;
    const type = head & 0x7F;
    const length = (buf[offset + 1] << 16) | (buf[offset + 2] << 8)
      | buf[offset + 3];
    const contentStart = offset + 4;
    if ((contentStart + length) > buf.length) throw new Error('flac: metadata block past EOF');
    if (type === 0) {
      // STREAMINFO: min/max block size (16+16), min/max frame size (24+24),
      // rate (20), channels-1 (3), bps-1 (5), total samples (36), MD5 (128).
      streamInfo = {
        minBlockSize: buf.readUInt16BE(contentStart),
        maxBlockSize: buf.readUInt16BE(contentStart + 2),
        sampleRate: (buf[contentStart + 10] << 12) | (buf[contentStart + 11] << 4)
          | (buf[contentStart + 12] >> 4),
        totalSamples: ((buf[contentStart + 13] & 0x0F) * (2
          ** 32)) + buf.readUInt32BE(contentStart + 14),
      };
    }
    if (type === 3) {
      // SEEKTABLE: 18-byte points of (sample number u64, byte offset u64, span u16);
      // sample number 0xFFF...F marks a placeholder.
      for (let point = 0; (point + 18) <= length; point += 18) {
        const sampleNumber = buf.readBigUInt64BE(contentStart + point);
        if (sampleNumber === 0xFF_FF_FF_FF_FF_FF_FF_FFn) continue;
        seekPoints.push({
          sample: Number(sampleNumber),
          byteOffset: Number(buf.readBigUInt64BE(contentStart + point
            + 8)),
        });
      }
    }
    offset = contentStart + length;
  }
  if (streamInfo === null) throw new Error('flac: no STREAMINFO block');
  if (streamInfo.sampleRate === 0) throw new Error('flac: zero sample rate');
  return {
    streamInfo,
    seekPoints,
    firstFrameOffset: offset
  };
}
//endregion

//region FLAC frame-header parsing
/**
 Decode FLAC's UTF-8-style coded number (frame number for fixed blocking, sample
 number for variable). Returns { value, length } or null when malformed.
 */
function decodeCodedNumber({
  buf,
  offset,
  maxLength
}) {
  const first = buf[offset];
  if (first === undefined) return null;
  if ((first & 0x80) === 0) return {
    value: first,
    length: 1
  };
  let length = 0;
  while ((length < 8) && ((first & (0x80 >> length)) !== 0)) length += 1;
  if ((length < 2) || (length > maxLength)) return null;
  // High bits come from the lead byte, then 6 payload bits per continuation byte.
  let value = first & (0x7F >> length);
  for (let index = 1; index < length; index += 1) {
    const byte = buf[offset + index];
    if ((byte === undefined) || ((byte & 0xC0) !== 0x80)) return null;
    value = (value * 64) + (byte & 0x3F);
  }
  return {
    value,
    length
  };
}

/**
 Try to parse a FLAC frame header at offset: 14-bit sync (0xFFF8/0xFFF9), field
 validation against reserved values and STREAMINFO, then the CRC-8 (poly 0x07)
 over the header bytes. Returns null on any mismatch, so false syncs inside
 compressed payloads are rejected. { blockingStrategy, blockSize, sampleRate,
 codedNumber, headerLength } on success.
 */
function parseFrameHeader({
  buf,
  offset,
  streamInfo
}) {
  if ((offset + 6) > buf.length) return null;
  if ((buf[offset] !== 0xFF) || ((buf[offset + 1] & 0xFE) !== 0xF8)) return null;
  const blockingStrategy = buf[offset + 1] & 1;
  const blockSizeBits = buf[offset + 2] >> 4;
  const rateBits = buf[offset + 2] & 0x0F;
  if ((blockSizeBits === 0) || (rateBits === 15)) return null;
  const channelBits = buf[offset + 3] >> 4;
  const sampleSizeBits = (buf[offset + 3] >> 1) & 7;
  if ((channelBits > 10) || (sampleSizeBits === 3)
    || ((buf[offset + 3] & 1) !== 0)) return null;
  const coded = decodeCodedNumber({
    buf,
    offset: offset + 4,
    maxLength: blockingStrategy === 1 ? 7 : 6,
  });
  if (coded === null) return null;
  let cursor = offset + 4
    + coded.length;
  let blockSize = FRAME_BLOCK_SIZES[blockSizeBits];
  if (blockSizeBits === 6) {
    if ((cursor + 1) > buf.length) return null;
    blockSize = buf[cursor] + 1;
    cursor += 1;
  } else if (blockSizeBits === 7) {
    if ((cursor + 2) > buf.length) return null;
    blockSize = buf.readUInt16BE(cursor) + 1;
    cursor += 2;
  }
  let sampleRate = rateBits === 0 ? streamInfo.sampleRate : FRAME_RATES[rateBits];
  if (rateBits === 12) {
    if ((cursor + 1) > buf.length) return null;
    sampleRate = buf[cursor] * 1_000;
    cursor += 1;
  } else if ((rateBits === 13) || (rateBits === 14)) {
    if ((cursor + 2) > buf.length) return null;
    sampleRate = buf.readUInt16BE(cursor) * (rateBits === 14 ? 10 : 1);
    cursor += 2;
  }
  // Cross-checks against STREAMINFO kill most false syncs before the CRC does.
  if (sampleRate !== streamInfo.sampleRate) return null;
  if ((streamInfo.maxBlockSize >= 16) && (blockSize > streamInfo.maxBlockSize)) return null;
  if (cursor >= buf.length) return null;
  let crc = 0;
  for (let index = offset; index < cursor; index += 1) crc = CRC8_TABLE[crc ^ buf[index]];
  if (crc !== buf[cursor]) return null;
  return {
    blockingStrategy,
    blockSize,
    sampleRate,
    codedNumber: coded.value,
    headerLength: (cursor + 1) - offset,
  };
}
//endregion

//region Frame walk and seektable profiles
/**
 Walk audio frames from the first frame to EOF without decoding. Each next frame
 start is confirmed by header validation + CRC-8 + the coded number matching the
 expectation from the previous frame (frame number + 1 for fixed blocking, start
 sample + block size for variable), which makes false syncs vanishingly unlikely.
 Frame byte size = distance between consecutive confirmed starts.
 */
function walkFrames({
  buf,
  firstFrameOffset,
  streamInfo,
  streamEnd
}) {
  const first = parseFrameHeader({
    buf,
    offset: firstFrameOffset,
    streamInfo
  });
  if (first === null) throw new Error('flac: no frame at first-frame offset');
  const {blockingStrategy} = first;
  // Fixed blocking: start sample = frame number x the constant block size (last
  // frame may be shorter, but its number still indexes the constant grid).
  const nominalBlockSize = first.blockSize;
  const rate = streamInfo.sampleRate;
  function startSampleOf(header) { return header.blockingStrategy === 1 ? header.codedNumber : header.codedNumber * nominalBlockSize; }
  const frames = [];
  let previous = {
    offset: firstFrameOffset,
    header: first
  };
  let searchFrom = firstFrameOffset + MIN_FRAME_BYTES;
  while (searchFrom < streamEnd) {
    const candidate = buf.indexOf(
      0xFF,
      searchFrom
    );
    if ((candidate === (-1)) || (candidate >= streamEnd)) break;
    const header = parseFrameHeader({
      buf,
      offset: candidate,
      streamInfo
    });
    if ((header === null) || (header.blockingStrategy !== blockingStrategy)) {
      searchFrom = candidate + 1;
      continue;
    }
    const expected = blockingStrategy === 1
      ? startSampleOf(previous.header)
        + previous.header
          .blockSize
      : previous.header
        .codedNumber
        + 1;
    if (header.codedNumber !== expected) {
      searchFrom = candidate + 1;
      continue;
    }
    frames.push({
      startSample: startSampleOf(previous.header),
      blockSize: previous.header
        .blockSize,
      byteCount: candidate - previous.offset,
    });
    previous = {
      offset: candidate,
      header
    };
    searchFrom = candidate + MIN_FRAME_BYTES;
  }
  const lastStart = startSampleOf(previous.header);
  frames.push({
    startSample: lastStart,
    blockSize: previous.header
      .blockSize,
    byteCount: streamEnd - previous.offset,
  });
  const durationSecs = (lastStart
    + previous.header
      .blockSize) / rate;
  return {
    frames,
    durationSecs
  };
}

/**
 Bin walked frames into 0.1 s slots. FLAC frames last 0.093-0.096 s, nearly one
 whole slot, so binning a frame's bytes at its start time ("start") aliases:
 slots alternately hold one or two frame starts, a 2x sawtooth unrelated to the
 audio. Spreading each frame's bytes over the slots it overlaps ("spread")
 removes that artifact; "start" is kept for comparison with the lossy profiles.
 */
function binFrames({
  frames,
  rate,
  binning
}) {
  const slots = [];
  for (const frame of frames) {
    if (binning === 'start') {
      addBytes({
        slots,
        slot: Math.floor((frame.startSample * 10) / rate),
        byteCount: frame.byteCount,
      });
    } else {
      spreadBytes({
        slots,
        startSecs: frame.startSample / rate,
        endSecs: (frame.startSample + frame.blockSize) / rate,
        byteCount: frame.byteCount,
      });
    }
  }
  return slots;
}

/**
 Build a profile from SEEKTABLE points alone (byte deltas over time deltas),
 spreading each interval's bytes evenly across its 0.1 s slots. Only used when
 points are finer than SEEK_GRANULARITY_MAX_SECS; the actual spacing is recorded.
 */
function profileFromSeekTable({
  seekPoints,
  streamInfo,
  firstFrameOffset,
  streamEnd
}) {
  const rate = streamInfo.sampleRate;
  const slots = [];
  const durationSecs = streamInfo.totalSamples / rate;
  const streamBytes = streamEnd - firstFrameOffset;
  for (let index = 0; index < seekPoints.length; index += 1) {
    const start = seekPoints[index];
    const next = seekPoints[index + 1];
    const endSample = next === undefined ? streamInfo.totalSamples : next.sample;
    const endByte = next === undefined ? streamBytes : next.byteOffset;
    spreadBytes({
      slots,
      startSecs: start.sample / rate,
      endSecs: endSample / rate,
      byteCount: endByte - start.byteOffset,
    });
  }
  if (seekPoints[0]
    .sample
    > 0) {
    // Bytes before the first point still belong to the timeline head.
    spreadBytes({
      slots,
      startSecs: 0,
      endSecs: seekPoints[0]
        .sample
        / rate,
      byteCount: seekPoints[0]
        .byteOffset
    });
  }
  return {
    slots,
    durationSecs
  };
}

/** Median spacing of consecutive seek points in seconds, or Infinity when unusable. */
function seekGranularitySecs({
  seekPoints,
  sampleRate
}) {
  if (seekPoints.length < 2) return Infinity;
  const deltas = seekPoints.slice(1)
    .map((
      point,
      index
    ) => point.sample
      - seekPoints[index]
        .sample);
  if (deltas.some((delta) => delta <= 0)) return Infinity;
  return quantile(
    deltas,
    0.5
  ) / sampleRate;
}

/**
 Parse one FLAC file's framing into 0.1 s byte slots without decoding. Prefers
 the SEEKTABLE when its points are finer than ~2 s (recording granularitySecs);
 otherwise walks and CRC-confirms every frame header. binning: 'spread'
 (default, overlap-proportional) or 'start' (start-time bin, see binFrames).
 */
export function parseFlac(
  buf,
  { binning = 'spread' } = {}
) {
  const {
    streamInfo,
    seekPoints,
    firstFrameOffset
  } = parseMetadata(buf);
  // A trailing ID3v1 tag (nonstandard but common) is not audio payload.
  const hasTrailingTag = (buf.length >= 128)
    && (buf.toString(
      'latin1',
      buf.length - 128,
      buf.length - 125
    ) === 'TAG');
  const streamEnd = hasTrailingTag ? buf.length - 128 : buf.length;
  const granularity = seekGranularitySecs({
    seekPoints,
    sampleRate: streamInfo.sampleRate
  });
  if ((granularity <= SEEK_GRANULARITY_MAX_SECS) && (streamInfo.totalSamples > 0)) {
    const {
      slots,
      durationSecs
    } = profileFromSeekTable({
      seekPoints,
      streamInfo,
      firstFrameOffset,
      streamEnd,
    });
    return {
      bytes: finalizeSlots({
        slots,
        durationSecs
      }),
      durationSecs,
      method: 'seektable',
      granularitySecs: granularity,
    };
  }
  const {
    frames,
    durationSecs
  } = walkFrames({
    buf,
    firstFrameOffset,
    streamInfo,
    streamEnd
  });
  const slots = binFrames({
    frames,
    rate: streamInfo.sampleRate,
    binning
  });
  return {
    bytes: finalizeSlots({
      slots,
      durationSecs
    }),
    durationSecs,
    method: 'frames',
    frameCount: frames.length,
  };
}
//endregion

//region Corpus targets
/** Long, loud FLAC tracks: the bucket where a bones-guided probe would matter. */
async function loadTargets() {
  const tracks = await loadTracks(CORPUS_PATH);
  return tracks.filter((track) =>
    (track.dur > SHORT_MAX_SECS)
      && (track.full > LOUD_LINEAR)
      && track.path
        .toLowerCase()
        .endsWith('.flac'));
}
//endregion

//region Extract stage
/** Bounded worker pool: POOL_SIZE workers pull indexes off a shared cursor. */
async function runPool({
  items,
  handler
}) {
  let cursor = 0;
  const workers = Array.from(
    { length: POOL_SIZE },
    async () => {
      while (cursor < items.length) {
        const index = cursor;
        cursor += 1;
        await handler(
          items[index],
          index
        );
      }
    }
  );
  await Promise.all(workers);
}

/** Parse every target FLAC and write out/flac-profiles.jsonl. */
async function extractStage() {
  const startedAt = Date.now();
  const targets = await loadTargets();
  console.log(`targets: ${targets.length}`);
  const output = createWriteStream(PROFILES_PATH);
  const stats = {
    missing: 0,
    frames: 0,
    seektable: 0,
    failed: 0
  };
  const failures = [];
  let done = 0;
  await runPool({
    items: targets,
    handler: async (track) => {
      let buf;
      try {
        buf = await readFile(track.path);
      } catch (error) {
        // Missing files are expected (library churn); count and move on.
        if (error.code === 'ENOENT') {
          stats.missing += 1;
          return;
        }
        throw error;
      }
      try {
        const {
          bytes,
          method,
          granularitySecs
        } = parseFlac(buf);
        const record = {
          path: track.path,
          slotSecs: SLOT_SECS,
          bytes
        };
        if (method === 'seektable') record.granularitySecs = granularitySecs;
        const line = `${JSON.stringify(record)}\n`;
        if (!output.write(line)) await once(
          output,
          'drain'
        );
        stats[method] += 1;
      } catch (error) {
        stats.failed += 1;
        failures.push({
          path: track.path,
          message: error.message
        });
      }
      done += 1;
      if ((done % 100) === 0) console.log(`  ...${done} files processed`);
    },
  });
  output.end();
  await once(
    output,
    'finish'
  );
  const elapsedSecs = ((Date.now() - startedAt) / 1_000).toFixed(1);
  console.log(`missing on disk: ${stats.missing}`);
  console.log(`parsed: frame-walk=${stats.frames} seektable=${stats.seektable} failed=${stats.failed}`);
  for (const failure of failures.slice(
    0,
    10
  )) {
    console.log(`  FAIL ${failure.path}: ${failure.message}`);
  }
  console.log(`wrote ${PROFILES_PATH} in ${elapsedSecs}s`);
}
//endregion

//region Analyze stage helpers
/** Pearson correlation between two equally-indexed series, truncated to the shorter. */
function pearson({
  xs,
  ys
}) {
  const length = Math.min(
    xs.length,
    ys.length
  );
  if (length < 2) return Number.NaN;
  const meanX = xs.slice(
    0,
    length
  )
    .reduce(
    (
      sum,
      value
    ) => sum + value,
    0
    )
    / length;
  const meanY = ys.slice(
    0,
    length
  )
    .reduce(
    (
      sum,
      value
    ) => sum + value,
    0
    )
    / length;
  let covariance = 0;
  let varianceX = 0;
  let varianceY = 0;
  for (let index = 0; index < length; index += 1) {
    const dx = xs[index] - meanX;
    const dy = ys[index] - meanY;
    covariance += dx * dy;
    varianceX += dx * dx;
    varianceY += dy * dy;
  }
  return covariance / Math.sqrt(varianceX * varianceY);
}

/** Load out/flac-profiles.jsonl into a path-keyed map. */
async function loadProfiles() {
  const profiles = new Map();
  const reader = createInterface({
    input: createReadStream(PROFILES_PATH),
    crlfDelay: Infinity
  });
  for await (const line of reader) {
    if (!line.trim()) continue;
    const profile = JSON.parse(line);
    profiles.set(
      profile.path,
      profile
    );
  }
  return profiles;
}

/**
 Crest byte-rank percentile: fraction of slots with MORE bytes than the crest
 bin's slot, taking the best byte count within +-pad slots to absorb skew.
 0 = crest slot is the byte-richest slot; 0.5 = middle of the pack.
 */
function crestByteRank({
  track,
  bytes,
  pad
}) {
  let crest = 0;
  for (let index = 1; index
    < track.bins
      .length; index += 1) {
    if (track.bins[index] > track.bins[crest]) crest = index;
  }
  const shared = Math.min(
    bytes.length,
    track.bins
      .length
  );
  if (crest >= shared) return null;
  let target = 0;
  for (let index = Math.max(
    0,
    crest - pad
  ); index <= Math.min(
    shared - 1,
    crest + pad
  ); index += 1) {
    target = Math.max(
      target,
      bytes[index]
    );
  }
  let higher = 0;
  for (let index = 0; index < shared; index += 1) {
    if (bytes[index] > target) higher += 1;
  }
  return higher / shared;
}

/**
 Bones-guided probe: decode the top-B byte slots plus +-1 neighbors, plus a
 sparse even pass, and take the peak over decoded bins. Returns the sampled
 peak and the fraction of the track actually decoded.
 */
function bonesGuidedPeak({
  track,
  bytes,
  topCount,
  evenCoverage
}) {
  const {bins} = track;
  const n = bins.length;
  const shared = Math.min(
    bytes.length,
    n
  );
  const decoded = new Uint8Array(n);
  let used = 0;
  function mark(index) {
    if ((index >= 0) && (index < n)
      && (decoded[index] === 0)) {
      decoded[index] = 1;
      used += 1;
    }
  }
  const order = Array.from(
    { length: shared },
    (
      _,
      index
    ) => index
  )
    .sort((
      a,
      b
    ) => bytes[b] - bytes[a]);
  for (const slot of order.slice(
    0,
    topCount
  )) {
    mark(slot - 1);
    mark(slot);
    mark(slot + 1);
  }
  const evenCount = Math.max(
    1,
    Math.round(evenCoverage * n)
  );
  const span = n - 1;
  for (let index = 0; index < evenCount; index += 1) {
    mark(evenCount <= 1 ? Math.floor(span / 2) : Math.round((index / (evenCount - 1)) * span));
  }
  let peak = 0;
  for (let index = 0; index < n; index += 1) {
    if ((decoded[index] === 1) && (bins[index] > peak)) peak = bins[index];
  }
  return {
    peak,
    coverage: used / n
  };
}

/** Print under-read percentiles and clamp counts for one probe variant. */
function reportUnderReads({
  label,
  rows
}) {
  const urs = rows.map((row) => row.ur);
  const line = [
    0.5,
    0.9,
    0.99,
    1
  ]
    .map((fraction) => `p${fraction * 100}=${quantile(
      urs,
      fraction
    )
      .toFixed(2)}`)
    .join(' ');
  const meanCoverage = rows.reduce(
    (
      sum,
      row
    ) => sum + row.coverage,
    0
  ) / rows.length;
  // Clamp = margin 0.8 dB fails to cover the under-read by more than 0.5 dB,
  // the same "too loud" bar correlate.mjs uses.
  const clamps = rows.filter((row) => (row.ur - 0.8) > 0.5)
    .length;
  console.log(
    `  ${label}: ur ${line} | clamps=${clamps}/${rows.length}`
      + ` | mean coverage=${(meanCoverage * 100).toFixed(1)}%`,
  );
}
//endregion

//region Analyze stage
/** Validate profiles, spot-check correlation, measure crest byte-rank, simulate. */
async function analyzeStage() {
  const targets = await loadTargets();
  const profiles = await loadProfiles();
  const profiled = targets.filter((track) => profiles.has(track.path));
  console.log(`\ntargets: ${targets.length}, profiled: ${profiled.length}`);

  // 1. Duration validation: profile timeline vs decoded corpus duration.
  const diffs = profiled.map((track) =>
    Math.abs((profiles.get(track.path)
      .bytes
      .length
      * SLOT_SECS) - track.dur));
  const within = diffs.filter((diff) => diff <= MISMATCH_LIMIT_SECS)
    .length;
  const fmtDur = [
    0.5,
    0.9,
    0.95,
    0.99,
    1
  ]
    .map((fraction) => `p${fraction * 100}=${quantile(
      diffs,
      fraction
    )
      .toFixed(3)}s`)
    .join(' ');
  console.log(`duration |profile - corpus|: ${fmtDur}`);
  console.log(
    `  within ${MISMATCH_LIMIT_SECS}s: ${within}/${diffs.length}`
      + ` (${((within / diffs.length) * 100).toFixed(2)}%)`,
  );
  const offenders = profiled
    .map((
      track,
      index
    ) => ({
      path: track.path,
      diff: diffs[index]
    }))
    .filter((row) => row.diff > MISMATCH_LIMIT_SECS)
    .sort((
      a,
      b
    ) => b.diff - a.diff);
  for (const offender of offenders.slice(
    0,
    5
  )) {
    console.log(`  OVER ${offender.diff
      .toFixed(2)}s ${offender.path}`);
  }

  // 2. Spot-check three files: per-slot bytes vs bin peaks, linear and dB domain.
  const spots = [
    profiled[0],
    profiled[Math.floor(profiled.length / 2)],
    profiled.at(-1)
  ];
  console.log('spot-check Pearson (per-slot bytes vs bin peaks):');
  for (const track of spots) {
    const {bytes} = profiles.get(track.path);
    const rLinear = pearson({
      xs: bytes,
      ys: track.bins
    });
    const dbBins = Array.from(
      track.bins,
      (value) => Math.max(
        db(value),
        -60
      )
    );
    const rDb = pearson({
      xs: bytes,
      ys: dbBins
    });
    console.log(`  rLinear=${rLinear.toFixed(3)} rDb=${rDb.toFixed(3)} ${track.path}`);
  }

  // 3. The decisive measurement: crest slot's byte-rank percentile across tracks.
  //    Lossy comparison: pad 0 median 60%, pad +-1 median 36.6% (useless).
  let padOneMedian = 1;
  for (const pad of [
    0,
    1,
    2
  ]) {
    const percentiles = profiled
      .map((track) => crestByteRank({
        track,
        bytes: profiles.get(track.path)
          .bytes,
        pad
      }))
      .filter((value) => value !== null);
    const fmt = [
      0.5,
      0.9,
      0.95
    ]
      .map((fraction) => `p${fraction * 100}=${(quantile(
        percentiles,
        fraction
      ) * 100).toFixed(1)}%`)
      .join(' ');
    console.log(`crest byte-rank percentile (pad +-${pad}): n=${percentiles.length} ${fmt}`);
    if (pad === 1) padOneMedian = quantile(
      percentiles,
      0.5
    );
  }

  // 4. Bones-guided probe simulation, only when the rank signal is strong
  //    (crest slot in roughly the top 10% of byte-rate for most tracks).
  if (padOneMedian > 0.1) {
    console.log(
      `rank too weak (pad +-1 median ${(padOneMedian * 100).toFixed(1)}% > 10%);`
        + ' skipping probe simulation (force with the simulate stage)',
    );
    return;
  }
  simulationReport({
    profiled,
    profiles
  });
}

/** Bones-guided low-coverage probe vs the even 24% reference, across variants. */
function simulationReport({
  profiled,
  profiles
}) {
  console.log('\nbones-guided probe vs even 24% (under-read dB across profiled tracks):');
  const evenRows = profiled.map((track) => ({
    ur: db(track.full) - db(sampledMaxEven(
      track,
      0.24,
      0.3
    )),
    coverage: 0.24,
  }));
  reportUnderReads({
    label: 'even 24% (reference)',
    rows: evenRows
  });
  for (const topCount of [
    10,
    20,
    40
  ]) {
    const rows = profiled.map((track) => {
      const {
        peak,
        coverage
      } = bonesGuidedPeak({
        track,
        bytes: profiles.get(track.path)
          .bytes,
        topCount,
        evenCoverage: 0.05,
      });
      return {
        ur: db(track.full) - db(peak),
        coverage
      };
    });
    reportUnderReads({
      label: `bones top=${topCount} +-1 + even 5%`,
      rows
    });
  }
}

/** Force the probe simulation regardless of the analyze stage's rank gate. */
async function simulateStage() {
  const targets = await loadTargets();
  const profiles = await loadProfiles();
  const profiled = targets.filter((track) => profiles.has(track.path));
  console.log(`\ntargets: ${targets.length}, profiled: ${profiled.length} (forced simulation)`);
  simulationReport({
    profiled,
    profiles
  });
}
//endregion

//region Stage dispatch
// Run stages only when executed directly, so tests can import parseFlac alone.
const isMain = (process.argv[1] !== undefined)
  && (import.meta.url
    === pathToFileURL(process.argv[1])
      .href);
if (isMain) {
  const stage = process.argv[2] ?? 'all';
  if ((stage !== 'extract') && (stage !== 'analyze')
    && (stage !== 'simulate')
    && (stage !== 'all')) {
    throw new Error(`unknown stage "${stage}" (use extract, analyze, simulate, or all)`);
  }
  if ((stage === 'extract') || (stage === 'all')) await extractStage();
  if ((stage === 'analyze') || (stage === 'all')) await analyzeStage();
  if (stage === 'simulate') await simulateStage();
}
//endregion
