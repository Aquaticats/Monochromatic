/**
 Tests for the scratch library: the WAV writer, the fallback library, and the generator path.
 
 @module
 */

import {
  chmod,
  mkdtempDisposable,
  type DisposableTempDir,
  readdir,
  readFile,
  stat,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import {
  describe,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  BITS_PER_SAMPLE,
  buildLibrary,
  BYTES_PER_SAMPLE,
  FALLBACK_ARTIST_FOLDER,
  FALLBACK_ARTIST_TRACKS,
  FALLBACK_LOOSE_FILES,
  FORMAT_CHUNK_BYTES,
  INT16_MAX,
  LONG_TONES,
  OMEGA_TONE_FILE,
  RIFF_SIZE_OVERHEAD_BYTES,
  SAMPLE_RATE,
  TONE_LEVEL,
  TONES_FOLDER,
  TRASH_FOLDER,
  TRASH_TONE_FILE,
  waveBytes,
} from './library.ts';

/**
 Makes a scratch directory, removed when the caller's scope ends.
 
 @returns the directory, which owns its own removal
 */
async function scratch(): Promise<DisposableTempDir> {
  return await mkdtempDisposable(join(scratchRoot(), 'e2e-library-',),);
}

await describe({
  name: waveBytes.name,
  children: [
    it({
      name: 'writes a mono 16-bit PCM WAV header that matches the data size',
      fn: async ({ expect, }) => {
        const bytes = waveBytes({ seconds: 1, hertz: 440, },);
        // One second of samples, and the header the RIFF size field does not count.
        const dataBytes = SAMPLE_RATE * BYTES_PER_SAMPLE;
        const headerBytes = 8 + RIFF_SIZE_OVERHEAD_BYTES;
        expect(bytes.subarray(0, 4,).toString('ascii',),).toBe('RIFF',);
        expect(bytes.subarray(8, 12,).toString('ascii',),).toBe('WAVE',);
        expect(bytes.subarray(12, 16,).toString('ascii',),).toBe('fmt ',);
        expect(bytes.readUInt32LE(16,),).toBe(FORMAT_CHUNK_BYTES,);
        expect(bytes.readUInt16LE(20,),).toBe(1,);
        expect(bytes.readUInt16LE(22,),).toBe(1,);
        expect(bytes.readUInt32LE(24,),).toBe(SAMPLE_RATE,);
        expect(bytes.readUInt32LE(28,),).toBe(SAMPLE_RATE * BYTES_PER_SAMPLE,);
        expect(bytes.readUInt16LE(32,),).toBe(BYTES_PER_SAMPLE,);
        expect(bytes.readUInt16LE(34,),).toBe(BITS_PER_SAMPLE,);
        expect(bytes.subarray(36, 40,).toString('ascii',),).toBe('data',);
        expect(bytes.readUInt32LE(40,),).toBe(dataBytes,);
        expect(bytes.readUInt32LE(4,),).toBe(bytes.length - 8,);
        expect(bytes.length,).toBe(headerBytes + dataBytes,);
      },
    },),
    it({
      name: 'holds a quiet tone: the first sample is zero and no sample is louder than the tone level',
      fn: async ({ expect, }) => {
        const bytes = waveBytes({ seconds: 1, hertz: 440, },);
        const dataStart = 8 + RIFF_SIZE_OVERHEAD_BYTES;
        expect(bytes.readInt16LE(dataStart,),).toBe(0,);
        let peak = 0;
        for (let offset = dataStart; offset < bytes.length; offset += 2)
          peak = Math.max(peak, Math.abs(bytes.readInt16LE(offset,),),);
        expect(peak,).toBeGreaterThan(1_000,);
        expect(peak,).toBeLessThanOrEqual(Math.round(TONE_LEVEL * INT16_MAX,),);
      },
    },),
  ],
},);

await describe({
  name: buildLibrary.name,
  children: [
    it({
      name: 'writes the fallback library when the generator is absent: loose files, a middle folder, and the tone folders',
      fn: async ({ expect, }) => {
        await using scratchDirectory = await scratch();
        const root = scratchDirectory.path;
        const { library, } = await buildLibrary({ root, generatorPath: join(root, 'missing.ts',), },);
        expect(library,).toBe(join(root, 'library',),);
        const top = (await readdir(library,)).toSorted();
        expect(top.filter((name,) => name.endsWith('.wav',),),).toHaveLength(FALLBACK_LOOSE_FILES,);
        expect(top.filter((name,) => !name.endsWith('.wav',),),).toEqual([TONES_FOLDER, FALLBACK_ARTIST_FOLDER, TRASH_FOLDER],);
        expect(
          await readdir(join(library, TONES_FOLDER,),),
        ).toHaveLength(LONG_TONES.length,);
        expect((await readdir(join(library, FALLBACK_ARTIST_FOLDER,),)).length,).toBe(FALLBACK_ARTIST_TRACKS,);
        expect((await readdir(join(library, TRASH_FOLDER,),)).toSorted(),).toEqual([OMEGA_TONE_FILE, TRASH_TONE_FILE],);
        expect(
          (await stat(join(library, TONES_FOLDER, LONG_TONES[0].name,),)).size,
        ).toBe(8 + RIFF_SIZE_OVERHEAD_BYTES + (LONG_TONES[0].seconds * SAMPLE_RATE * BYTES_PER_SAMPLE));
      },
      timeout: 60_000,
    },),
    it({
      name: 'materializes the library with the generator when it is present, then adds the tone folders',
      fn: async ({ expect, }) => {
        await using scratchDirectory = await scratch();
        const root = scratchDirectory.path;
        const generator = join(root, 'generator.ts',);
        await writeFile(
          generator,
          [
            "import { mkdirSync, writeFileSync } from 'node:fs';",
            "import { join } from 'node:path';",
            'const args = process.argv.slice(2);',
            "const target = args[args.indexOf('--materialize') + 1];",
            "mkdirSync(join(target, 'Generated Artist'), { recursive: true });",
            "writeFileSync(join(target, 'Generated Artist', 'one.wav'), 'x');",
          ].join('\n',),
        );
        const { library, } = await buildLibrary({ root, generatorPath: generator, },);
        expect((await readdir(library,)).toSorted(),).toEqual([TONES_FOLDER, 'Generated Artist', TRASH_FOLDER],);
        expect(await readFile(join(library, 'Generated Artist', 'one.wav',), 'utf8',),).toBe('x',);
      },
      timeout: 60_000,
    },),
    it({
      name: 'fails with the generator message when the generator fails',
      fn: async ({ expect, }) => {
        await using scratchDirectory = await scratch();
        const root = scratchDirectory.path;
        const generator = join(root, 'broken.ts',);
        await writeFile(generator, "console.error('no corpus'); process.exit(3);\n",);
        await chmod(generator, 0o644,);
        let caught: unknown;
        try {
          await buildLibrary({ root, generatorPath: generator, },);
        }
        catch (error: unknown) {
          caught = error;
        }
        expect((caught as Error).message,).toContain('the library generator failed',);
      },
      timeout: 60_000,
    },),
  ],
},);
