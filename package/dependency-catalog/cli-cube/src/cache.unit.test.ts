/**
 Tests for the per-package JSON file cache.
 
 Each test creates an isolated cache root under `os.scratchRoot()`
 so concurrent runs don't collide; cleanup runs in a `using`
 disposable so failures don't leave temp dirs behind.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  mkdir,
  mkdtemp,
  rm,
  stat,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  dirname,
  join,
} from 'node:path';

import {
  CACHE_MISS,
  createCache,
} from './cache.ts';

/**
 Allocates a fresh temp directory for one cache invocation and
 returns it along with an async-disposable that removes it.
 
 @returns Tuple of root path and an async-disposable that recursively rms it on scope exit.
 */
async function tempCacheRoot(): Promise<{
  rootDir: string;
  [Symbol.asyncDispose]: () => Promise<void>;
}> {
  const rootDir = await mkdtemp(join(scratchRoot(), 'deps-cube-cache-',),);
  return {
    rootDir,
    [Symbol.asyncDispose]: async function dispose() {
      await rm(rootDir, { recursive: true, force: true, },);
    },
  };
}

await describe({
  name: 'cache',
  children: [
    it({
      name: 'read returns CACHE_MISS for a missing key',
      fn: async () => {
        await using temp = await tempCacheRoot();
        const cache = createCache({ rootDir: temp.rootDir, },);
        const value = await cache.read({
          name: 'preact',
          version: '10.0.0',
          field: 'languages',        },);
        expect(value,).toBe(CACHE_MISS,);
      },
    },),

    it({
      name: 'write then read roundtrips the value',
      fn: async () => {
        await using temp = await tempCacheRoot();
        const cache = createCache({ rootDir: temp.rootDir, },);
        const payload = { TypeScript: 1_000, JavaScript: 500, };
        await cache.write({
          name: 'preact',
          version: '10.0.0',
          field: 'languages',
          value: payload,
        },);
        const read = await cache.read<typeof payload>({
          name: 'preact',
          version: '10.0.0',
          field: 'languages',        },);
        expect(read,).toEqual(payload,);
      },
    },),

    it({
      name: 'omitted ttlMs means the value never expires',
      fn: async () => {
        await using temp = await tempCacheRoot();
        const cache = createCache({ rootDir: temp.rootDir, },);
        await cache.write({
          name: 'preact',
          version: '10.0.0',
          field: 'languages',
          value: { TypeScript: 1, },
        },);
        const read = await cache.read({
          name: 'preact',
          version: '10.0.0',
          field: 'languages',        },);
        expect(read,).toEqual({ TypeScript: 1, },);
      },
    },),

    it({
      name: 'negative ttlMs treats every entry as expired',
      fn: async () => {
        await using temp = await tempCacheRoot();
        const cache = createCache({ rootDir: temp.rootDir, },);
        await cache.write({
          name: 'preact',
          version: '10.0.0',
          field: 'downloads',
          value: { downloads: 42, },
        },);
        const expired = await cache.read({
          name: 'preact',
          version: '10.0.0',
          field: 'downloads',
          ttlMs: -1,
        },);
        expect(expired,).toBe(CACHE_MISS,);
      },
    },),

    it({
      name: 'multiple fields coexist inside one (name, version) file',
      fn: async () => {
        await using temp = await tempCacheRoot();
        const cache = createCache({ rootDir: temp.rootDir, },);
        await cache.write({
          name: 'preact',
          version: '10.0.0',
          field: 'languages',
          value: { TypeScript: 100, },
        },);
        await cache.write({
          name: 'preact',
          version: '10.0.0',
          field: 'downloads',
          value: { downloads: 7, },
        },);
        const langs = await cache.read({
          name: 'preact',
          version: '10.0.0',
          field: 'languages',        },);
        const dls = await cache.read({
          name: 'preact',
          version: '10.0.0',
          field: 'downloads',        },);
        expect(langs,).toEqual({ TypeScript: 100, },);
        expect(dls,).toEqual({ downloads: 7, },);
      },
    },),

    it({
      name: 'rootDir is exposed for diagnostics',
      fn: async () => {
        await using temp = await tempCacheRoot();
        const cache = createCache({ rootDir: temp.rootDir, },);
        expect(cache.rootDir,).toBe(temp.rootDir,);
      },
    },),

    it({
      name: 'write creates the parent directory if missing',
      fn: async () => {
        await using temp = await tempCacheRoot();
        const cache = createCache({ rootDir: temp.rootDir, },);
        await cache.write({
          name: '@scope/pkg',
          version: '1.2.3',
          field: 'transitive',
          value: 7,
        },);
        const expectedFile = join(temp.rootDir, '@scope', 'pkg', '1.2.3.json',);
        const exists = await stat(expectedFile,).then(
          function ok() {
            return true;
          },
          function fail() {
            return false;
          },
        );
        expect(exists,).toBe(true,);
        expect(dirname(expectedFile,),).toBe(join(temp.rootDir, '@scope', 'pkg',),);
      },
    },),

    it({
      name: 'malformed cache file is treated as a miss',
      fn: async () => {
        await using temp = await tempCacheRoot();
        const cache = createCache({ rootDir: temp.rootDir, },);
        const corruptDir = join(temp.rootDir, 'corrupt',);
        const corruptPath = join(corruptDir, '1.0.0.json',);
        await mkdir(corruptDir, { recursive: true, },);
        await writeFile(corruptPath, '{not valid json', 'utf8',);
        const read = await cache.read({
          name: 'corrupt',
          version: '1.0.0',
          field: 'languages',        },);
        expect(read,).toBe(CACHE_MISS,);
      },
    },),
  ],
},);
