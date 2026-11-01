// The frame-pattern analysis behind the IDE's cadence check (inspect-cadence.ts), in one place.
// What: the hash of one recorded BMP frame, the frame slots that an input stream covers, the count of frames that
// changed against the frame before them, the runs of identical frames, and the text the check prints.
// Why: a recorder photographs the window at a steady 60 Hz whether or not the application drew, so a frame identical
// to its predecessor during a drag or a scroll means the picture did not move for that tick. The check needs that
// count and those runs to tell a smooth pipeline from a stalling one, and they are pure functions of the hashes, so
// they are tested here (cadence.test.ts) and the compositor-driving script stays thin. The functions are linear passes
// over their input.

import { createHash } from 'node:crypto';

/**
 Frame rate of the recorder command (`record DIR 60 bmp`); the slot of a moment is its time times this rate.
 */
export const recordFps = 60;

/**
 Size in bytes of the BMP file header, which holds the offset of the pixel data at byte 10.
 */
export const bmpFileHeaderBytes = 14;

/**
 What a caller sees when a recorded file is not a BMP.
 */
export class FrameFormatError extends Error {
  override name = 'FrameFormatError';
}

/**
 Hash of the pixels of one BMP frame, in hexadecimal.
 
 The header is left out so that the hash follows the picture alone; two frames with the same pixels hash alike.
 
 @param bytes - the whole BMP file as the recorder wrote it
 
 @returns the SHA-1 digest of the pixel data that follows the header
 
 @throws {FrameFormatError} when the bytes do not start with the BMP signature or the pixel offset lies outside them
 */
export function frameHash(bytes: Uint8Array): string {
  if ((bytes.length < bmpFileHeaderBytes) || (bytes[0] !== 0x42)
    || (bytes[1] !== 0x4D)) {
    throw new FrameFormatError('A recorded frame must start with the BMP signature BM and hold a 14 byte file header');
  }
  const offset = new DataView(
    bytes.buffer,
    bytes.byteOffset,
    bytes.byteLength
  ).getUint32(
    10,
    true
  );
  if ((offset < bmpFileHeaderBytes) || (offset > bytes.length)) {
    throw new FrameFormatError(`The BMP pixel data offset ${  String(offset)
       } lies outside the ${
       String(bytes.length)
       } bytes of the frame`);
  }
  return createHash('sha1')
    .update(bytes.subarray(offset))
    .digest('hex');
}

/**
 The first and last frame slot of an input stream, counted from the first recorded frame.
 */
export type InputWindow = {
  readonly firstSlot: number;
  readonly lastSlot: number
};

/**
 Frame slots that an input stream covers.
 
 @param offsetMs - milliseconds from the recording start to the moment the input command was sent
 
 @param inputMs - milliseconds the input command took to answer
 
 @returns the slot of the first and of the last moment, at the recorder rate
 */
export function inputWindow({
  offsetMs,
  inputMs
}: {
  readonly offsetMs: number;
  readonly inputMs: number
}): InputWindow {
  return {
    firstSlot: Math.round((offsetMs * recordFps) / 1_000),
    lastSlot: Math.round(((offsetMs + inputMs) * recordFps) / 1_000),
  };
}

/**
 Frames in a row that each equal the frame before them; `start` is the slot of the first of them.
 */
export type IdenticalRun = {
  readonly start: number;
  readonly length: number
};

/**
 What the frames of an input stream did against the frame before each.
 */
export type Cadence = {
  /**
   Every recorded frame, inside the window or not.
   */
  readonly total: number;
  readonly firstSlot: number;
  readonly lastSlot: number;
  /**
   Frames inside the window that differ from the frame before them.
   */
  readonly changed: number;
  /**
   Frames inside the window that equal the frame before them.
   */
  readonly identical: number;
  readonly longestIdenticalRun: number;
  /**
   Every run of identical frames inside the window, in order, with the slot of its first frame.
   */
  readonly identicalRuns: readonly IdenticalRun[];
  /**
   One character per recorded frame: X when it differs from the frame before it (the first always does), . when not.
   */
  readonly pattern: string;
};

/**
 Compare each frame of the window against the frame before it.
 
 @param hashes - one hash per recorded frame in recording order
 
 @param window - the slots to judge; frames outside it (the quiet time around the input) are not counted
 
 @returns the counts, the runs of identical frames, and the pattern of the whole recording
 */
export function analyzeCadence({
  hashes,
  window
}: {
  readonly hashes: readonly string[];
  readonly window: InputWindow
}): Cadence {
  let changed = 0;
  let identical = 0;
  let run = 0;
  let longestIdenticalRun = 0;
  const identicalRuns: IdenticalRun[] = [];
  const first = Math.max(
    window.firstSlot + 1,
    1
  );
  const last = Math.min(
    window.lastSlot,
    hashes.length - 1
  );
  for (let slot = first; slot <= last; slot++) {
    if (hashes[slot] === hashes[slot - 1]) {
      identical++;
      run++;
      longestIdenticalRun = Math.max(
        longestIdenticalRun,
        run
      );
    } else {
      changed++;
      if (run > 0) identicalRuns.push({
        start: slot - run,
        length: run
      });
      run = 0;
    }
  }
  if (run > 0) identicalRuns.push({
    start: (last + 1) - run,
    length: run
  });
  const pattern = hashes.map((
    hash,
    slot
  ) => ((slot === 0) || (hash !== hashes[slot - 1]) ? 'X' : '.'))
    .join('');
  return {
    total: hashes.length,
    firstSlot: window.firstSlot,
    lastSlot: window.lastSlot,
    changed,
    identical,
    longestIdenticalRun,
    identicalRuns,
    pattern
  };
}

/**
 The lines the check prints for one run.
 
 @param cadence - the result of {@link analyzeCadence}
 
 @returns the summary lines; the pattern marks the window by brackets so that the quiet frames around it stand apart
 */
export function formatCadence(cadence: Cadence): readonly string[] {
  const marked = cadence.pattern
    .split('')
    .map((
      mark,
      slot
    ) => (slot === cadence.firstSlot ? '[' : '') + mark
      + (slot === cadence.lastSlot ? ']' : ''))
    .join('');
  return [
    `frames recorded: ${  String(cadence.total)  }, input window: slots ${  String(cadence.firstSlot)  } to ${  String(cadence.lastSlot)}`,
    `in window: ${  String(cadence.changed + cadence.identical)  } compared, ${  String(cadence.changed)  } changed, ${  String(cadence.identical)  } identical`,
    `longest identical run: ${  String(cadence.longestIdenticalRun)}`,
    `identical runs (length@first slot): [${  cadence.identicalRuns
      .map(run => `${String(run.length)  }@${  String(run.start)}`)
      .join(', ')  }]`,
    `pattern (X changed, . identical): ${  marked}`,
  ];
}
