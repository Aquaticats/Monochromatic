/**
 Tests for the scratch library: the WAV writer, the fallback library, and the generator path.
 
 @module
 */

import {
  chmod,
  mkdtemp,
  readdir,
  readFile,
  rm,
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
  buildLibrary,
  waveBytes,
} from './library.ts';

/**
 Makes a scratch directory.
 
 @returns its path
 */
async function scratch(): Promise<string> {
  return await mkdtemp(join(scratchRoot(), 'e2e-library-',),);
}

await describe({
  name: waveBytes.name,
  children: [
    it({
      name: 'writes a mono 16-bit PCM WAV header that matches the data size',
      fn: async ({ expect, }) => {
        const bytes = waveBytes({ seconds: 1, hertz: 440, },);
        expect(bytes.subarray(0, 4,).toString('ascii',),).toBe('RIFF',);
        expect(bytes.subarray(8, 12,).toString('ascii',),).toBe('WAVE',);
        expect(bytes.subarray(12, 16,).toString('ascii',),).toBe('fmt ',);
        expect(bytes.readUInt32LE(16,),).toBe(16,);
        expect(bytes.readUInt16LE(20,),).toBe(1,);
        expect(bytes.readUInt16LE(22,),).toBe(1,);
        expect(bytes.readUInt32LE(24,),).toBe(22_050,);
        expect(bytes.readUInt32LE(28,),).toBe(44_100,);
        expect(bytes.readUInt16LE(32,),).toBe(2,);
        expect(bytes.readUInt16LE(34,),).toBe(16,);
        expect(bytes.subarray(36, 40,).toString('ascii',),).toBe('data',);
        expect(bytes.readUInt32LE(40,),).toBe(44_100,);
        expect(bytes.readUInt32LE(4,),).toBe(36 + 44_100,);
        expect(bytes.length,).toBe(44 + 44_100,);
      },
    },),
    it({
      name: 'holds a quiet tone: the first sample is zero and no sample is louder than the tone level',
      fn: async ({ expect, }) => {
        const bytes = waveBytes({ seconds: 1, hertz: 440, },);
        expect(bytes.readInt16LE(44,),).toBe(0,);
        let peak = 0;
        for (let offset = 44; offset < bytes.length; offset += 2)
          peak = Math.max(peak, Math.abs(bytes.readInt16LE(offset,),),);
        expect(peak,).toBeGreaterThan(1_000,);
        expect(peak,).toBeLessThanOrEqual(Math.round(0.05 * 32_767,),);
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
        const root = await scratch();
        const { library, } = await buildLibrary({ root, generatorPath: join(root, 'missing.ts',), },);
        expect(library,).toBe(join(root, 'library',),);
        const top = (await readdir(library,)).toSorted();
        expect(top.filter((name,) => name.startsWith('Fallback ',),),).toHaveLength(30,);
        expect(top.filter((name,) => !name.endsWith('.wav',),),).toEqual(['Aaa Tones', 'Mmm Tones', 'Zzz Tones',],);
        expect(
          await readdir(join(library, 'Aaa Tones',),),
        ).toHaveLength(4,);
        expect((await readdir(join(library, 'Mmm Tones',),)).length,).toBe(30,);
        expect((await readdir(join(library, 'Zzz Tones',),)).toSorted(),).toEqual(['01 Omega tone.wav', '99 Trash me.wav',],);
        expect((await stat(join(library, 'Aaa Tones', '01 Alpha tone.wav',),)).size,).toBe(44 + (200 * 44_100),);
        await rm(root, { recursive: true, force: true, },);
      },
      timeout: 60_000,
    },),
    it({
      name: 'materializes the library with the generator when it is present, then adds the tone folders',
      fn: async ({ expect, }) => {
        const root = await scratch();
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
        expect((await readdir(library,)).toSorted(),).toEqual(['Aaa Tones', 'Generated Artist', 'Zzz Tones',],);
        expect(await readFile(join(library, 'Generated Artist', 'one.wav',), 'utf8',),).toBe('x',);
        await rm(root, { recursive: true, force: true, },);
      },
      timeout: 60_000,
    },),
    it({
      name: 'fails with the generator message when the generator fails',
      fn: async ({ expect, }) => {
        const root = await scratch();
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
        await rm(root, { recursive: true, force: true, },);
      },
      timeout: 60_000,
    },),
  ],
},);
