// No-decode byte-rate profiles: walk container/frame framing only (Ogg/Opus pages,
// ISO-BMFF sample tables, MP3 frame headers) and bucket compressed audio payload bytes
// into 0.1 s slots keyed by each packet's start time. Never decodes a single sample;
// the byte-rate curve is a free side channel for locating loud/busy transients.
// Driven by bones-extract.mjs; validated against the decoded corpus by bones-validate.mjs.

/** Slot width in seconds shared by every profile (matches corpus bin_seconds). */
export const SLOT_SECS = 0.1;
/** Opus timestamps tick at 48 kHz regardless of the original sample rate. */
const OPUS_RATE = 48_000;
/** One 0.1 s slot expressed in 48 kHz samples, for integer slot math on Opus. */
const SLOT_SAMPLES_48K = OPUS_RATE * SLOT_SECS;

/** Add payload bytes into the slot holding a packet's start time (sparse-safe). */
function addBytes({
  slots,
  slot,
  byteCount
}) {
  slots[slot] = (slots[slot] ?? 0) + byteCount;
}

/** Replace sparse holes with zeros and extend to cover the full timeline duration. */
function finalizeSlots({
  slots,
  slotCount
}) {
  const length = Math.max(
    slots.length,
    slotCount
  );
  return Array.from(
    { length },
    (
      _,
      index
    ) => slots[index] ?? 0
  );
}

/**
 Duration of one Opus packet in 48 kHz samples, from the TOC byte alone
 (RFC 6716 section 3.1). configs 0-11 are SILK ([10,20,40,60] ms per group of 4),
 12-15 are Hybrid ([10,20] ms pairs), 16-31 are CELT ([2.5,5,10,20] ms per group).
 */
export function opusPacketSamples({
  toc,
  second
}) {
  const config = toc >> 3;
  const code = toc & 3;
  let frameSamples;
  if (config < 12) {
    frameSamples = [
      480,
      960,
      1_920,
      2_880
    ][config & 3];
  } else if (config < 16) {
    frameSamples = [
      480,
      960
    ][config & 1];
  } else {
    frameSamples = [
      120,
      240,
      480,
      960
    ][config & 3];
  }
  let frameCount;
  if (code === 0) {
    frameCount = 1;
  } else if (code === 3) {
    // Code 3: frame count lives in the byte after the TOC (low 6 bits).
    frameCount = second === (-1) ? 0 : second & 0x3F;
  } else {
    frameCount = 2;
  }
  return frameSamples * frameCount;
}

/**
 Walk Ogg pages of the first Opus logical stream. Packets are concatenated lacing
 segments terminated by a value < 255; header_type bit 0x01 continues the previous
 page's unfinished packet. The first two packets (OpusHead, OpusTags) carry no audio.
 */
export function parseOgg(buf) {
  const slots = [];
  let offset = 0;
  let opusSerial = null;
  let packetsDone = 0;
  let samples = 0;
  // Unfinished packet carried across a page boundary: byte length so far plus the
  // first two payload bytes (TOC + code-3 frame count), and a drop flag for
  // continuation data whose start we never saw.
  let pending = null;
  while ((offset + 27) <= buf.length) {
    if (
      (buf[offset] !== 0x4F) || (buf[offset + 1] !== 0x67)
        || (buf[offset + 2] !== 0x67)
        || (buf[offset + 3] !== 0x53)
    ) {
      // Lost sync: scan forward to the next "OggS" capture pattern.
      const next = buf.indexOf(
        'OggS',
        offset + 1,
        'latin1'
      );
      if (next === (-1)) break;
      offset = next;
      continue;
    }
    const headerType = buf[offset + 5];
    const serial = buf.readUInt32LE(offset + 14);
    const segmentCount = buf[offset + 26];
    const lacingStart = offset + 27;
    if ((lacingStart + segmentCount) > buf.length) break;
    let payloadLength = 0;
    for (let seg = 0; seg < segmentCount; seg += 1) {
      payloadLength += buf[lacingStart + seg];
    }
    const payloadStart = lacingStart + segmentCount;
    if ((payloadStart + payloadLength) > buf.length) break;
    const pageEnd = payloadStart + payloadLength;
    // Bind to the first logical stream whose BOS packet starts with "OpusHead".
    if (opusSerial === null) {
      const isBos = (headerType & 0x02) !== 0;
      if (isBos && (buf.toString(
        'latin1',
        payloadStart,
        payloadStart + 8
      ) === 'OpusHead')) {
        opusSerial = serial;
      }
    }
    if (serial !== opusSerial) {
      offset = pageEnd;
      continue;
    }
    const continued = (headerType & 0x01) !== 0;
    if (!continued) pending = null;
    if (continued && (pending === null)) {
      // Continuation of a packet whose start we never saw (mid-stream join); swallow
      // its remaining segments without emitting anything.
      pending = {
        length: 0,
        toc: -1,
        second: -1,
        drop: true
      };
    }
    let current = pending;
    pending = null;
    let segmentStart = payloadStart;
    for (let seg = 0; seg < segmentCount; seg += 1) {
      const lacing = buf[lacingStart + seg];
      if (current === null) current = {
        length: 0,
        toc: -1,
        second: -1,
        drop: false
      };
      // Capture the first two payload bytes of the packet (TOC, code-3 count).
      if ((current.length === 0) && (lacing >= 1)) current.toc = buf[segmentStart];
      if ((current.length === 0) && (lacing >= 2)) current.second = buf[segmentStart + 1];
      if ((current.length === 1) && (lacing >= 1)) current.second = buf[segmentStart];
      current.length += lacing;
      segmentStart += lacing;
      if (lacing < 255) {
        // Packet complete on this page.
        if ((!current.drop) && (current.length > 0)) {
          packetsDone += 1;
          if (packetsDone > 2) {
            // Audio packet: bucket its bytes at its start time, then advance.
            addBytes({
              slots,
              slot: Math.floor(samples / SLOT_SAMPLES_48K),
              byteCount: current.length
            });
            samples += opusPacketSamples(current);
          }
        }
        current = null;
      }
    }
    pending = current;
    offset = pageEnd;
    // End of our logical stream: stop (chained streams would restart the timeline).
    if ((headerType & 0x04) !== 0) break;
  }
  if (packetsDone <= 2) throw new Error('ogg: no audio packets found');
  const durationSecs = samples / OPUS_RATE;
  return {
    bytes: finalizeSlots({
      slots,
      slotCount: Math.ceil(samples / SLOT_SAMPLES_48K)
    }),
    durationSecs,
  };
}

/** List direct child boxes ({ type, contentStart, contentEnd }) between two offsets. */
function childBoxes({
  buf,
  start,
  end
}) {
  const boxes = [];
  let offset = start;
  while ((offset + 8) <= end) {
    const size32 = buf.readUInt32BE(offset);
    const type = buf.toString(
      'latin1',
      offset + 4,
      offset + 8
    );
    let headerSize = 8;
    let boxSize = size32;
    if (size32 === 1) {
      if ((offset + 16) > end) break;
      boxSize = Number(buf.readBigUInt64BE(offset + 8));
      headerSize = 16;
    } else if (size32 === 0) {
      // Size 0 means "extends to end of enclosing scope".
      boxSize = end - offset;
    }
    if ((boxSize < headerSize) || ((offset + boxSize) > end)) break;
    boxes.push({
      type,
      contentStart: offset + headerSize,
      contentEnd: offset + boxSize
    });
    offset += boxSize;
  }
  return boxes;
}

/** First direct child box of a given type, or null. */
function findBox({
  buf,
  start,
  end,
  type
}) {
  return childBoxes({
    buf,
    start,
    end
  })
    .find((box) => box.type === type)
    ?? null;
}

/** Per-sample byte size accessor over stsz (constant or table) or stz2 (packed). */
function sampleSizeReader({
  buf,
  stbl
}) {
  const stsz = findBox({
    buf,
    start: stbl.contentStart,
    end: stbl.contentEnd,
    type: 'stsz'
  });
  if (stsz !== null) {
    const constantSize = buf.readUInt32BE(stsz.contentStart + 4);
    const sampleCount = buf.readUInt32BE(stsz.contentStart + 8);
    const tableStart = stsz.contentStart + 12;
    if (constantSize !== 0) return {
      sampleCount,
      sizeAt: () => constantSize
    };
    return {
      sampleCount,
      sizeAt: (index) => buf.readUInt32BE(tableStart + (4
        * index))
    };
  }
  const stz2 = findBox({
    buf,
    start: stbl.contentStart,
    end: stbl.contentEnd,
    type: 'stz2'
  });
  if (stz2 === null) throw new Error('mp4: no stsz/stz2 in audio stbl');
  const fieldSize = buf[stz2.contentStart + 7];
  const sampleCount = buf.readUInt32BE(stz2.contentStart + 8);
  const tableStart = stz2.contentStart + 12;
  if (fieldSize === 16) return {
    sampleCount,
    sizeAt: (index) => buf.readUInt16BE(tableStart + (2
      * index))
  };
  if (fieldSize === 8) return {
    sampleCount,
    sizeAt: (index) => buf[tableStart + index]
  };
  if (fieldSize === 4) {
    return {
      sampleCount,
      sizeAt: (index) => {
        const byte = buf[tableStart + (index >> 1)];
        return (index % 2) === 0 ? byte >> 4 : byte & 0x0F;
      },
    };
  }
  throw new Error(`mp4: unsupported stz2 field size ${fieldSize}`);
}

/**
 ISO-BMFF: find the soun trak (moov > trak > mdia), read timescale from mdhd,
 per-sample durations from stts and byte sizes from stsz/stz2, then walk samples
 in order accumulating time. Sample sizes are exactly the compressed AAC payloads.
 */
export function parseMp4(buf) {
  const moov = findBox({
    buf,
    start: 0,
    end: buf.length,
    type: 'moov'
  });
  if (moov === null) throw new Error('mp4: no moov box');
  const traks = childBoxes({
    buf,
    start: moov.contentStart,
    end: moov.contentEnd
  })
    .filter((box) => box.type === 'trak');
  for (const trak of traks) {
    const mdia = findBox({
      buf,
      start: trak.contentStart,
      end: trak.contentEnd,
      type: 'mdia'
    });
    if (mdia === null) continue;
    const hdlr = findBox({
      buf,
      start: mdia.contentStart,
      end: mdia.contentEnd,
      type: 'hdlr'
    });
    if (hdlr === null) continue;
    const handler = buf.toString(
      'latin1',
      hdlr.contentStart + 8,
      hdlr.contentStart + 12
    );
    if (handler !== 'soun') continue;
    const mdhd = findBox({
      buf,
      start: mdia.contentStart,
      end: mdia.contentEnd,
      type: 'mdhd'
    });
    if (mdhd === null) throw new Error('mp4: audio trak lacks mdhd');
    const mdhdVersion = buf[mdhd.contentStart];
    // Version 0: 32-bit creation/modification times; version 1: 64-bit.
    const timescale = mdhdVersion === 1
      ? buf.readUInt32BE(mdhd.contentStart + 20)
      : buf.readUInt32BE(mdhd.contentStart + 12);
    if (timescale === 0) throw new Error('mp4: zero timescale');
    const minf = findBox({
      buf,
      start: mdia.contentStart,
      end: mdia.contentEnd,
      type: 'minf'
    });
    if (minf === null) throw new Error('mp4: audio trak lacks minf');
    const stbl = findBox({
      buf,
      start: minf.contentStart,
      end: minf.contentEnd,
      type: 'stbl'
    });
    if (stbl === null) throw new Error('mp4: audio trak lacks stbl');
    const stts = findBox({
      buf,
      start: stbl.contentStart,
      end: stbl.contentEnd,
      type: 'stts'
    });
    if (stts === null) throw new Error('mp4: audio trak lacks stts');
    const {
      sampleCount,
      sizeAt
    } = sampleSizeReader({
      buf,
      stbl
    });
    if (sampleCount === 0) throw new Error('mp4: zero samples (fragmented file?)');
    const slots = [];
    const entryCount = buf.readUInt32BE(stts.contentStart + 4);
    let timeUnits = 0;
    let sampleIndex = 0;
    for (let entry = 0; (entry < entryCount) && (sampleIndex < sampleCount); entry += 1) {
      const runCount = buf.readUInt32BE(stts.contentStart + 8
        + (entry
          * 8));
      const delta = buf.readUInt32BE(stts.contentStart + 12
        + (entry
          * 8));
      for (let run = 0; (run < runCount) && (sampleIndex < sampleCount); run += 1) {
        const slot = Math.floor((timeUnits * 10) / timescale);
        addBytes({
          slots,
          slot,
          byteCount: sizeAt(sampleIndex)
        });
        timeUnits += delta;
        sampleIndex += 1;
      }
    }
    const durationSecs = timeUnits / timescale;
    return {
      bytes: finalizeSlots({
        slots,
        slotCount: Math.ceil((durationSecs / SLOT_SECS) - 1e-9)
      }),
      durationSecs,
    };
  }
  throw new Error('mp4: no soun trak');
}

/** Layer III bitrates in kbps by header index; 0 (free) and 15 (bad) are unusable. */
const MP3_BITRATES_V1 = [
  0,
  32,
  40,
  48,
  56,
  64,
  80,
  96,
  112,
  128,
  160,
  192,
  224,
  256,
  320,
  0
];
const MP3_BITRATES_V2 = [
  0,
  8,
  16,
  24,
  32,
  40,
  48,
  56,
  64,
  80,
  96,
  112,
  128,
  144,
  160,
  0
];
/** Sample rates by version bits (0 = MPEG2.5, 2 = MPEG2, 3 = MPEG1); index 3 reserved. */
const MP3_RATES = {
  0: [
    11_025,
    12_000,
    8_000
  ],
  2: [
    22_050,
    24_000,
    16_000
  ],
  3: [
    44_100,
    48_000,
    32_000
  ]
};

/**
 MP3: skip a leading ID3v2 tag, then walk Layer III frames by header arithmetic,
 resyncing byte-by-byte across garbage. Each frame's whole length counts as payload
 (a leading Xing/Info frame is counted too; ~26 ms skew, noted as a caveat).
 */
export function parseMp3(buf) {
  let offset = 0;
  if ((buf.length >= 10) && (buf[0] === 0x49)
    && (buf[1] === 0x44)
    && (buf[2] === 0x33)) {
    const tagSize = ((buf[6] & 0x7F) << 21) | ((buf[7] & 0x7F) << 14)
      | ((buf[8] & 0x7F) << 7)
      | (buf[9] & 0x7F);
    const footer = (buf[5] & 0x10) !== 0 ? 10 : 0;
    offset = 10 + tagSize
      + footer;
  }
  const slots = [];
  let timeSecs = 0;
  let frameCount = 0;
  while ((offset + 4) <= buf.length) {
    if ((buf[offset] !== 0xFF) || ((buf[offset + 1] & 0xE0) !== 0xE0)) {
      offset += 1;
      continue;
    }
    const byte1 = buf[offset + 1];
    const byte2 = buf[offset + 2];
    const versionBits = (byte1 >> 3) & 3;
    const layerBits = (byte1 >> 1) & 3;
    const bitrateIndex = (byte2 >> 4) & 15;
    const rateIndex = (byte2 >> 2) & 3;
    const padding = (byte2 >> 1) & 1;
    // Only Layer III with a defined bitrate/rate is a real frame; else keep scanning.
    if (
      (versionBits === 1) || (layerBits !== 1)
        || (bitrateIndex === 0)
        || (bitrateIndex === 15)
        || (rateIndex === 3)
    ) {
      offset += 1;
      continue;
    }
    const isMpeg1 = versionBits === 3;
    const bitrate = (isMpeg1 ? MP3_BITRATES_V1 : MP3_BITRATES_V2)[bitrateIndex] * 1_000;
    const sampleRate = MP3_RATES[versionBits][rateIndex];
    // Standard Layer III frame length: 144 * bitrate / rate for MPEG1, 72 for MPEG2/2.5.
    const lengthCoefficient = isMpeg1 ? 144 : 72;
    const frameLength = Math.floor((lengthCoefficient * bitrate) / sampleRate) + padding;
    if (frameLength < 4) {
      offset += 1;
      continue;
    }
    addBytes({
      slots,
      slot: Math.floor((timeSecs / SLOT_SECS) + 1e-9),
      byteCount: frameLength
    });
    timeSecs += (isMpeg1 ? 1_152 : 576) / sampleRate;
    offset += frameLength;
    frameCount += 1;
  }
  if (frameCount === 0) throw new Error('mp3: no frames found');
  return {
    bytes: finalizeSlots({
      slots,
      slotCount: Math.ceil((timeSecs / SLOT_SECS) - 1e-9)
    }),
    durationSecs: timeSecs,
  };
}

/** Pick a parser by magic bytes first, extension second. */
export function detectFormat({
  buf,
  path
}) {
  if ((buf.length >= 4) && (buf.toString(
    'latin1',
    0,
    4
  ) === 'OggS')) return 'ogg';
  if ((buf.length >= 12) && (buf.toString(
    'latin1',
    4,
    8
  ) === 'ftyp')) return 'mp4';
  if ((buf.length >= 3) && (buf.toString(
    'latin1',
    0,
    3
  ) === 'ID3')) return 'mp3';
  const lower = path.toLowerCase();
  if (lower.endsWith('.opus') || lower.endsWith('.ogg')) return 'ogg';
  if (lower.endsWith('.m4a') || lower.endsWith('.mp4')) return 'mp4';
  if (lower.endsWith('.mp3')) return 'mp3';
  throw new Error('unknown container format');
}
