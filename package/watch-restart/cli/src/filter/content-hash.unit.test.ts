import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  HashCache,
  OVERSIZED,
} from '../hash-cache.ts';
import type {
  WatchCtx,
  WatchEvent,
} from '../types.ts';
import { contentHashFilter, } from './content-hash.ts';

/**
 Logger root for watch-restart after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l: defaultLogger, },);
 ```
 */
const defaultLogger = tagged({ tag: 'watch-restart', },);

/**
 Tiny size cap exercised by the "file too large" case; 4 bytes is below
 any realistic file, so a 5-byte write trivially exceeds the cap and
 forces `hashFile` to return `null`.
 */
const TINY_HASH_CAP_BYTES = 4;

/**
 Helper that returns a fresh temp directory dedicated to one test run.
 
 @returns absolute path of a freshly-created temp directory
 
 @example
 ```ts
 const dir = await makeTmpDir();
 ```
 */
async function makeTmpDir(): Promise<string> {
  return mkdtemp(join(scratchRoot(), 'watch-restart-content-hash-',),);
}

/**
 Builds a minimal {@link WatchCtx} with a real {@link HashCache} and a
 disposable AbortController; tests that need to share a cache pass one
 in explicitly so the cache reads / writes from the filter become
 inspectable across multiple invocations.
 
 @param hashCache - cache instance to reuse; defaults to a fresh empty cache
 
 @returns context object suitable for handing to a filter
 
 @example
 ```ts
 const ctx = makeCtx({ hashCache: cache, },);
 ```
 */
function makeCtx(
  {
    hashCache,
  }: {
    readonly hashCache?: HashCache;
  } = {},
): WatchCtx {
  return {
    logger: defaultLogger,
    hashCache: hashCache ?? new HashCache(),
    signal: new AbortController().signal,
  };
}

/**
 Builds a {@link WatchEvent} from overrides; defaults match a `change`
 to `/abs/file.ts`.
 
 @param overrides - partial event fields to override on the default
 
 @returns a fully-populated {@link WatchEvent}
 
 @example
 ```ts
 const event = makeEvent({ kind: 'unlink', },);
 ```
 */
function makeEvent(
  overrides: {
    readonly kind?: WatchEvent['kind'];
    readonly entity?: WatchEvent['entity'];
    readonly path?: WatchEvent['path'];
    readonly relativePath?: WatchEvent['relativePath'];
    readonly ext?: WatchEvent['ext'];
  } = {},
): WatchEvent {
  return {
    kind: overrides.kind ?? 'change',
    entity: overrides.entity ?? 'file',
    path: overrides.path ?? '/abs/file.ts',
    relativePath: overrides.relativePath ?? 'file.ts',
    ext: overrides.ext ?? '.ts',
  };
}

/**
 Narrows a {@link HashCache.hashFile} result to a hex digest, throwing if
 the test fixture unexpectedly exceeded the size cap.
 
 @param value - hashFile result (digest or the OVERSIZED sentinel)
 
 @returns hex digest string
 
 @throws Error when the fixture file is unexpectedly oversized
 
 @example
 ```ts
 const hash = requireHash(await cache.hashFile(file,),);
 ```
 */
function requireHash(value: string | typeof OVERSIZED,): string {
  if (value === OVERSIZED) {
    throw new Error(
      'test fixture file unexpectedly exceeded the hash size cap',
    );
  }
  return value;
}

await describe({
  name: contentHashFilter.name,
  children: [
    it({
      name: 'unlink passes through (true) without touching the cache',
      fn: async function unlinkPassesThrough() {
        const dir = await makeTmpDir();
        const cache = new HashCache();
        cache.set({
          path: '/abs/file.ts',
          hash: nonNullishOrThrow('cached-hash',),
        },);

        const filter = contentHashFilter();
        const passed = await filter({
          event: makeEvent({ kind: 'unlink', },),
          ctx: makeCtx({ hashCache: cache, },),
        },);

        expect(passed,).toBe(true,);
        // Cache must be untouched: contentHashFilter is not responsible for
        // unlink cleanup (the watcher already removed the entry).
        expect(cache.get('/abs/file.ts',),).toBe('cached-hash',);

        await rm(dir, { recursive: true, },);
      },
    },),
    it({
      name: 'add of a new file fires and records the hash',
      fn: async function addRecordsAndFires() {
        const dir = await makeTmpDir();
        const file = join(dir, 'fresh.ts',);
        await writeFile(file, 'first-content',);
        const cache = new HashCache();

        const filter = contentHashFilter();
        const passed = await filter({
          event: makeEvent({
            kind: 'add',
            path: file,
            relativePath: 'fresh.ts',
          },),
          ctx: makeCtx({ hashCache: cache, },),
        },);

        expect(passed,).toBe(true,);
        expect(cache.has(file,),).toBe(true,);
        // Cached value matches the freshly computed hash; checking that the
        // record happened against the right path-key.
        expect(cache.get(file,),).toBe(
          await cache.hashFile(file,),
        );

        await rm(dir, { recursive: true, },);
      },
    },),
    it({
      name: 'byte-identical change is skipped (false)',
      fn: async function byteIdenticalSkips() {
        const dir = await makeTmpDir();
        const file = join(dir, 'same.ts',);
        await writeFile(file, 'unchanged-bytes',);
        const cache = new HashCache();
        const priorHash = requireHash(await cache.hashFile(file,),);
        cache.set({
          path: file,
          hash: priorHash,
        },);

        const filter = contentHashFilter();
        const passed = await filter({
          event: makeEvent({
            kind: 'change',
            path: file,
            relativePath: 'same.ts',
          },),
          ctx: makeCtx({ hashCache: cache, },),
        },);

        expect(passed,).toBe(false,);
        expect(cache.get(file,),).toBe(priorHash,);

        await rm(dir, { recursive: true, },);
      },
    },),
    it({
      name: 'different-bytes change fires and stores the new hash',
      fn: async function differentBytesFires() {
        const dir = await makeTmpDir();
        const file = join(dir, 'edit.ts',);
        await writeFile(file, 'v1-content',);
        const cache = new HashCache();
        const oldHash = requireHash(await cache.hashFile(file,),);
        cache.set({
          path: file,
          hash: oldHash,
        },);

        await writeFile(file, 'v2-content-which-differs',);

        const filter = contentHashFilter();
        const passed = await filter({
          event: makeEvent({
            kind: 'change',
            path: file,
            relativePath: 'edit.ts',
          },),
          ctx: makeCtx({ hashCache: cache, },),
        },);

        expect(passed,).toBe(true,);
        const newHash = nonNullishOrThrow(cache.get(file,),);
        expect(newHash,).not.toBe(oldHash,);

        await rm(dir, { recursive: true, },);
      },
    },),
    it({
      name: 'file exceeding maxHashSize fires (true) without comparing',
      fn: async function tooLargeFires() {
        const dir = await makeTmpDir();
        const file = join(dir, 'big.ts',);
        // 5 bytes > 4-byte cap → hashFile returns null.
        await writeFile(file, 'hello',);
        const cache = new HashCache({ maxHashSize: TINY_HASH_CAP_BYTES, },);

        const filter = contentHashFilter();
        const passed = await filter({
          event: makeEvent({
            kind: 'change',
            path: file,
            relativePath: 'big.ts',
          },),
          ctx: makeCtx({ hashCache: cache, },),
        },);

        expect(passed,).toBe(true,);
        // No hash was computable, so the cache must not have been written.
        expect(cache.has(file,),).toBe(false,);

        await rm(dir, { recursive: true, },);
      },
    },),
    it({
      name: 'read error (missing file) fires (true) and does not throw',
      fn: async function readErrorFires() {
        const dir = await makeTmpDir();
        const missing = join(dir, 'missing.ts',);
        const cache = new HashCache();

        const filter = contentHashFilter();
        const passed = await filter({
          event: makeEvent({
            kind: 'change',
            path: missing,
            relativePath: 'missing.ts',
          },),
          ctx: makeCtx({ hashCache: cache, },),
        },);

        expect(passed,).toBe(true,);
        expect(cache.has(missing,),).toBe(false,);

        await rm(dir, { recursive: true, },);
      },
    },),
  ],
},);
