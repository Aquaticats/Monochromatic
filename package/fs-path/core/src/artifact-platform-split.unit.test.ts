import {
  readdir,
  readFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

// Guards the platform split of the two built artifacts. Every `.mjs` file in
// a build directory is read (named entries plus hashed shared chunks) and
// concatenated, so an assertion about "the node build" covers whatever
// rolldown hoisted out of the entries. Positive controls come first: a probe
// that cannot see `ensureDir` in the node build proves nothing about its
// absence elsewhere. Same shape as the logger's guard.

//region Build text
/**
 Directory holding both platform builds, resolved from this test file so
 the guard reads whatever the last `build` task emitted.
 */
const distFinalDir = join(
  import.meta.dirname,
  '..',
  'dist',
  'final',
);

/**
 Reads every `.mjs` file in one build directory and joins their text.

 @param build - Build directory name under `dist/final`.

 @returns Concatenated text of every `.mjs` file in that build.

 @throws Error when the directory holds no `.mjs` file, which means the
 build has not run and every negative assertion would pass vacuously.

 @example
 ```ts
 const text = await buildText({ build: 'node' });
 ```
 */
async function buildText({ build, }: { readonly build: 'neutral' | 'node'; },): Promise<string> {
  /**
   Build directory whose modules are concatenated.
   */
  const dir = join(
    distFinalDir,
    build,
  );
  /**
   Module file names in the build, chunks included; declaration files are skipped.
   */
  const names = (await readdir(dir,)).filter(function isModule(name: string,): boolean {
    return name.endsWith('.mjs',);
  },);
  if (names.length === 0)
    throw new Error(`no .mjs files in ${dir}; run the build before this guard`,);
  /**
   Module texts in directory order.
   */
  const texts = await Promise.all(names.map(async function readModule(name: string,): Promise<string> {
    return await readFile(
      join(
        dir,
        name,
      ),
      'utf8',
    );
  },),);
  return texts.join('\n',);
}

/**
 Concatenated text of the Node artifact (`dist/final/node/*.mjs`).
 */
const nodeText = await buildText({ build: 'node', },);

/**
 Concatenated text of the platform-neutral artifact (`dist/final/neutral/*.mjs`).
 */
const neutralText = await buildText({ build: 'neutral', },);
//endregion Build text

await describe({
  name: 'artifact platform split',
  children: [
    it({
      name: 'positive control: node build ships the fs helpers with static node:fs/promises and node:path imports',
      fn: async () => {
        expect(nodeText.includes('ensureDir',),).toBe(true,);
        expect(nodeText.includes('node:fs/promises',),).toBe(true,);
        expect(nodeText.includes('node:path',),).toBe(true,);
      },
    },),

    it({
      name: 'positive control: neutral build ships the origin private file system backend',
      fn: async () => {
        expect(neutralText.includes('navigator.storage',),).toBe(true,);
        expect(neutralText.includes('getDirectory',),).toBe(true,);
      },
    },),

    it({
      name: 'positive control: both builds ship the walker, the markers, and the memory adapter',
      fn: async () => {
        for (const text of [nodeText, neutralText,]) {
          expect(text.includes('RootNotFoundError',),).toBe(true,);
          expect(text.includes('createMemoryRootFilesystem',),).toBe(true,);
          expect(text.includes('[monorepo]',),).toBe(true,);
        }
      },
    },),

    it({
      name: 'neither build contains a dynamic import',
      fn: async () => {
        expect(nodeText.includes('import(',),).toBe(false,);
        expect(neutralText.includes('import(',),).toBe(false,);
      },
    },),

    it({
      name: 'neutral build names neither node:fs nor node:path',
      fn: async () => {
        expect(neutralText.includes('node:fs',),).toBe(false,);
        expect(neutralText.includes('node:path',),).toBe(false,);
      },
    },),

    it({
      name: 'node build touches neither navigator.storage nor a directory handle',
      fn: async () => {
        expect(nodeText.includes('navigator.storage',),).toBe(false,);
        expect(nodeText.includes('getDirectory',),).toBe(false,);
      },
    },),

    it({
      name: 'neither build mentions happy-opfs',
      fn: async () => {
        expect(nodeText.includes('happy-opfs',),).toBe(false,);
        expect(neutralText.includes('happy-opfs',),).toBe(false,);
      },
    },),

    it({
      name: 'Node-only helpers stay out of the neutral build',
      fn: async () => {
        expect(neutralText.includes('ensureDir',),).toBe(false,);
        expect(neutralText.includes('emptyDir',),).toBe(false,);
      },
    },),
  ],
},);
