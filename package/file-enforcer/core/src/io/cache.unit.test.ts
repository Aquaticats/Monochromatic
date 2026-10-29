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
import {
  join,
  resolve,
} from 'node:path';
import {
  invalidatePaths,
  readCache,
  readCached,
  updateCache,
} from '../../dist/final/node/index.mjs';

/** Fresh temp directory for each test */
async function makeTmpDir(): Promise<string> {
  return mkdtemp(join(scratchRoot(), 'file-enforcer-cache-',),);
}

// No afterEach(readCache.clear): each test uses a unique mkdtemp path,
// so cache keys never collide. A global .clear() under concurrentTestGlob
// wipes other tests' entries mid-execution, causing false failures.

await describe({
  name: '',
  children: [
    describe({
      name: readCached.name,
      children: [
        it({
          name: 'reads file from disk on first call',
          fn: async () => {
            const dir = await makeTmpDir();
            const file = join(dir, 'a.txt',);
            await writeFile(file, 'hello',);
            expect(await readCached(file,),).toBe('hello',);
            await rm(dir, { recursive: true, },);
          },
        },),
        it({
          name: 'returns cached content on subsequent calls without re-reading',
          fn: async () => {
            const dir = await makeTmpDir();
            const file = join(dir, 'a.txt',);
            await writeFile(file, 'original',);
            expect(await readCached(file,),).toBe('original',);
            // Modify file on disk; cache should still return old content
            await writeFile(file, 'modified',);
            expect(await readCached(file,),).toBe('original',);
            await rm(dir, { recursive: true, },);
          },
        },),
        it({
          name: 'caches by absolute path',
          fn: async () => {
            const dir = await makeTmpDir();
            const file = join(dir, 'b.txt',);
            await writeFile(file, 'content',);
            await readCached(file,);
            expect(
              readCache.has(resolve(file,),),
            ).toBe(true,);
            await rm(dir, { recursive: true, },);
          },
        },),
      ],
    },),
    describe({
      name: invalidatePaths.name,
      children: [
        it({
          name: 'removes specific path from cache',
          fn: async () => {
            const dir = await makeTmpDir();
            const file = join(dir, 'c.txt',);
            await writeFile(file, 'v1',);
            await readCached(file,);
            expect(
              readCache.has(resolve(file,),),
            ).toBe(true,);
            invalidatePaths([file,],);
            expect(
              readCache.has(resolve(file,),),
            ).toBe(false,);
            await rm(dir, { recursive: true, },);
          },
        },),
        it({
          name: 're-reads file after invalidation picks up new content',
          fn: async () => {
            const dir = await makeTmpDir();
            const file = join(dir, 'd.txt',);
            await writeFile(file, 'v1',);
            expect(await readCached(file,),).toBe('v1',);
            await writeFile(file, 'v2',);
            invalidatePaths([file,],);
            expect(await readCached(file,),).toBe('v2',);
            await rm(dir, { recursive: true, },);
          },
        },),
        it({
          name: 'does not affect other cached entries',
          fn: async () => {
            const dir = await makeTmpDir();
            const fileA = join(dir, 'a.txt',);
            const fileB = join(dir, 'b.txt',);
            await writeFile(fileA, 'aaa',);
            await writeFile(fileB, 'bbb',);
            await readCached(fileA,);
            await readCached(fileB,);
            invalidatePaths([fileA,],);
            expect(
              readCache.has(resolve(fileB,),),
            ).toBe(true,);
            await rm(dir, { recursive: true, },);
          },
        },),
      ],
    },),
    describe({
      name: updateCache.name,
      children: [
        it({
          name: 'sets cache entry without reading from disk',
          fn: async () => {
            updateCache({ filePath: '/fake/path.txt', content: 'injected', },);
            expect(
              readCache.get(resolve('/fake/path.txt',),),
            ).toBe('injected',);
          },
        },),
        it({
          name: 'subsequent readCached returns the updated content',
          fn: async () => {
            const dir = await makeTmpDir();
            const file = join(dir, 'e.txt',);
            await writeFile(file, 'disk-content',);
            // Pre-populate cache with different content (simulates post-write update)
            updateCache({ filePath: file, content: 'written-content', },);
            expect(await readCached(file,),).toBe('written-content',);
            await rm(dir, { recursive: true, },);
          },
        },),
      ],
    },),
  ],
},);
