/**
 The scratch music library the app scans during a run, and the WAV files it holds.
 
 @example
 ```ts
 import { buildLibrary, } from './library.ts';
 
 const { library, } = await buildLibrary({ root: scratch, generatorPath, },);
 ```
 
 @packageDocumentation
 */

import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';

import { runCommand, } from './command.ts';
import { expect, } from './errors.ts';
import { pathExists, } from './files.ts';

/**
 Samples per second of every generated tone.
 */
const SAMPLE_RATE = 22_050;

/**
 Bytes of one 16-bit mono sample.
 */
const BYTES_PER_SAMPLE = 2;

/**
 Bytes of a 32-bit WAV field.
 */
const BYTES_PER_WORD = 4;

/**
 Bits of one sample.
 */
const BITS_PER_SAMPLE = 16;

/**
 Bytes of the format chunk of a PCM WAV file.
 */
const FORMAT_CHUNK_BYTES = 16;

/**
 Bytes of a WAV header that the RIFF size field does not count: everything after the size field except the data.
 */
const RIFF_SIZE_OVERHEAD_BYTES = 36;

/**
 Tone level as a fraction of full scale: quiet enough to leave headroom, loud enough to be a real signal.
 */
const TONE_LEVEL = 0.05;

/**
 Largest value of a signed 16-bit sample.
 */
const INT16_MAX = 32_767;

/**
 Number of loose files the fallback library puts at its root, matching the three percent loose share of the generator.
 */
const FALLBACK_LOOSE_FILES = 30;

/**
 Number of tracks in the fallback artist folder.
 */
const FALLBACK_ARTIST_TRACKS = 30;

/**
 The folder of long tone tracks that makes seeking and track changes possible.
 */
const LONG_TONES = [
  {
    name: '01 Alpha tone.wav',
    seconds: 200,
    hertz: 220,
  },
  {
    name: '02 Bravo tone.wav',
    seconds: 200,
    hertz: 247,
  },
  {
    name: '03 Charlie tone.wav',
    seconds: 200,
    hertz: 262,
  },
  {
    name: '04 Delta tone.wav',
    seconds: 200,
    hertz: 294,
  },
] as const;

/**
 Builds a 4-byte ASCII chunk tag.
 
 @param tag - four ASCII characters
 
 @returns the tag as bytes
 
 @example
 ```ts
 asciiTag('RIFF',); // <Buffer 52 49 46 46>
 ```
 */
function asciiTag(tag: string,): Buffer {
  return Buffer.from(
    tag,
    'ascii',
  );
}

/**
 Builds a little-endian 32-bit field.
 
 @param value - unsigned value
 
 @returns the field as bytes
 
 @example
 ```ts
 u32(1,); // <Buffer 01 00 00 00>
 ```
 */
function u32(value: number,): Buffer {
  /**
   Four zeroed bytes.
   */
  const field = Buffer.alloc(BYTES_PER_WORD,);
  field.writeUInt32LE(
    value,
    0,
  );
  return field;
}

/**
 Builds a little-endian 16-bit field.
 
 @param value - unsigned value
 
 @returns the field as bytes
 
 @example
 ```ts
 u16(1,); // <Buffer 01 00>
 ```
 */
function u16(value: number,): Buffer {
  /**
   Two zeroed bytes.
   */
  const field = Buffer.alloc(2,);
  field.writeUInt16LE(
    value,
    0,
  );
  return field;
}

/**
 Builds a mono 16-bit PCM WAV file that holds one sine tone.
 
 @param seconds - length of the tone
 
 @param hertz - frequency of the tone
 
 @returns the whole file
 
 @example
 ```ts
 const bytes = waveBytes({ seconds: 1, hertz: 440, },);
 ```
 */
export function waveBytes(
  {
    seconds,
    hertz,
  }: {
    readonly hertz: number;
    readonly seconds: number;
  },
): Buffer {
  /**
   Samples in the tone.
   */
  const frames = SAMPLE_RATE * seconds;

  /**
   Bytes of sample data.
   */
  const dataBytes = frames * BYTES_PER_SAMPLE;

  /**
   The sample data, filled below.
   */
  const samples = Buffer.alloc(dataBytes,);
  for (let frame = 0; frame < frames; frame++) {
    /**
     Phase of the sine at this sample, in radians.
     */
    const phase = (2 * Math.PI
      * hertz
      * frame) / SAMPLE_RATE;
    samples.writeInt16LE(
      Math.round(Math.sin(phase,) * TONE_LEVEL
        * INT16_MAX,),
      frame * BYTES_PER_SAMPLE,
    );
  }
  return Buffer.concat([
    asciiTag('RIFF',),
    u32(RIFF_SIZE_OVERHEAD_BYTES + dataBytes,),
    asciiTag('WAVE',),
    asciiTag('fmt ',),
    u32(FORMAT_CHUNK_BYTES,),
    // PCM format, one channel.
    u16(1,),
    u16(1,),
    u32(SAMPLE_RATE,),
    u32(SAMPLE_RATE * BYTES_PER_SAMPLE,),
    u16(BYTES_PER_SAMPLE,),
    u16(BITS_PER_SAMPLE,),
    asciiTag('data',),
    u32(dataBytes,),
    samples,
  ],);
}

/**
 Writes the fallback base of the library for a machine without the library generator: loose files at the root and one
 artist folder of short tracks, the same shape the generator makes, so the folder order the checks walk is the same.
 
 @param library - library directory
 
 @example
 ```ts
 await writeFallbackBase(join(root, 'library',),);
 ```
 */
async function writeFallbackBase(library: string,): Promise<void> {
  /**
   Tagged logger naming the fallback.
   */
  const rl = tagged({
    tag: writeFallbackBase.name,
    l: logger,
  },);
  rl.info('the library generator is not present: writing the fallback library',);

  /**
   Artist folder between the two tone folders in sort order.
   */
  const artist = join(
    library,
    'Mmm Tones',
  );
  await mkdir(
    artist,
    { recursive: true, },
  );

  /**
   One second of tone that every fallback file holds.
   */
  const shortTone = waveBytes({
    seconds: 1,
    hertz: 330,
  },);
  await Promise.all([
    ...Array.from(
      { length: FALLBACK_LOOSE_FILES, },
      function looseFile(
        _unused,
        index,
      ): Promise<void> {
        return writeFile(
          join(
            library,
            `Fallback ${String(index + 1,)
              .padStart(
              2,
              '0',
            )}.wav`,
          ),
          shortTone,
        );
      },
    ),
    ...Array.from(
      { length: FALLBACK_ARTIST_TRACKS, },
      function artistTrack(
        _unused,
        index,
      ): Promise<void> {
        return writeFile(
          join(
            artist,
            `${String(index + 1,)
              .padStart(
              2,
              '0',
            )} Mid tone.wav`,
          ),
          shortTone,
        );
      },
    ),
  ],);
}

/**
 Builds the scratch library: the one-thousand-track generator library (real files) when the generator is present,
 else the fallback base, plus the folders of long tone tracks and the short file that the Delete check trashes.
 
 @param root - scratch directory the library goes into
 
 @param generatorPath - path of the library generator script; the fallback is used when the file is absent
 
 @returns the library directory
 
 @throws CheckFailureError when the generator fails
 
 @example
 ```ts
 const { library, } = await buildLibrary({ root, generatorPath, },);
 ```
 */
export async function buildLibrary(
  {
    root,
    generatorPath,
  }: {
    readonly generatorPath: string;
    readonly root: string;
  },
): Promise<{ readonly library: string; }> {
  /**
   Tagged logger naming the library build.
   */
  const rl = tagged({
    tag: buildLibrary.name,
    l: logger,
  },);

  /**
   Library directory inside the scratch root.
   */
  const library = join(
    root,
    'library',
  );
  if (await pathExists(generatorPath,)) {
    rl.info(`materializing the 1k preset with ${generatorPath}`,);

    /**
     Result of the generator run.
     */
    const generated = await runCommand({
      command: 'node',
      args: [
        generatorPath,
        '--preset',
        '1k',
        '--out',
        join(
          root,
          'paths-1k.txt',
        ),
        '--materialize',
        library,
      ],
    },);
    expect({
      condition: generated.status === 0,
      message: `the library generator failed: ${generated.stderr}`,
    },);
  }
  else
    await writeFallbackBase(library,);

  /**
   Folder that sorts first.
   */
  const tones = join(
    library,
    'Aaa Tones',
  );
  await mkdir(
    tones,
    { recursive: true, },
  );

  /**
   Folder that sorts last, with the file the Delete check sends to the trash.
   */
  const last = join(
    library,
    'Zzz Tones',
  );
  await mkdir(
    last,
    { recursive: true, },
  );
  await Promise.all([
    ...LONG_TONES.map(function writeLongTone({
      name,
      seconds,
      hertz,
    },): Promise<void> {
      return writeFile(
        join(
          tones,
          name,
        ),
        waveBytes({
          seconds,
          hertz,
        },),
      );
    },),
    writeFile(
      join(
        last,
        '01 Omega tone.wav',
      ),
      waveBytes({
        seconds: 30,
        hertz: 392,
      },),
    ),
    writeFile(
      join(
        last,
        '99 Trash me.wav',
      ),
      waveBytes({
        seconds: 2,
        hertz: 440,
      },),
    ),
  ],);
  return { library, };
}
