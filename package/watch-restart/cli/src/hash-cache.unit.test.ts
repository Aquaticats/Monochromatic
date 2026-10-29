import { BYTES_PER_MIB, } from '@monochromatic-dev/const-core/ts';
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
import {
  DEFAULT_MAX_HASH_SIZE_BYTES,
  HashCache,
  OVERSIZED,
} from './hash-cache.ts';

/**
 Known sha256 hex of the literal bytes "hello".
 Round-trip assertions use this so "outputs something" is not the only check.
 
 @example
 ```ts
 expect(await cache.hashFile(fileWithHello,),).toBe(SHA256_HEX_OF_HELLO,);
 ```
 */
const SHA256_HEX_OF_HELLO =
  '2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824';

/**
 Creates a fresh temp directory per test so file fixtures cannot leak between cases.
 Returned path is absolute.
 
 @example
 ```ts
 const dir = await makeTmpDir();
 const file = join(dir, 'a.txt',);
 await writeFile(file, 'hello',);
 await rm(dir, { recursive: true, },);
 ```
 
 @returns absolute path of a freshly-created temp directory
 */
async function makeTmpDir(): Promise<string> {
  return mkdtemp(join(scratchRoot(), 'watch-restart-hash-cache-',),);
}

await describe({
  name: HashCache.name,
  children: [
    describe({
      name: 'hashFile',
      children: [
        it({
          name: 'returns sha256 hex of file bytes (round-trip)',
          fn: async () => {
            const dir = await makeTmpDir();
            const file = join(dir, 'a.txt',);
            await writeFile(file, 'hello',);
            const cache = new HashCache();
            expect(await cache.hashFile(file,),).toBe(SHA256_HEX_OF_HELLO,);
            await rm(dir, { recursive: true, },);
          },
        },),
        it({
          name: 'returns OVERSIZED when file size exceeds maxHashSize',
          fn: async () => {
            const dir = await makeTmpDir();
            const file = join(dir, 'big.txt',);
            await writeFile(file, 'too-big',);
            // 'too-big' is 7 bytes; cap at 6 forces the OVERSIZED branch.
            const cache = new HashCache({ maxHashSize: 6, },);
            expect(await cache.hashFile(file,),).toBe(OVERSIZED,);
            await rm(dir, { recursive: true, },);
          },
        },),
        it({
          name: 'hashes up to and including maxHashSize boundary',
          fn: async () => {
            const dir = await makeTmpDir();
            const file = join(dir, 'exact.txt',);
            await writeFile(file, 'hello',);
            // 'hello' is 5 bytes; cap at 5 must still hash (size > cap, not >=).
            const cache = new HashCache({ maxHashSize: 5, },);
            expect(await cache.hashFile(file,),).toBe(SHA256_HEX_OF_HELLO,);
            await rm(dir, { recursive: true, },);
          },
        },),
        it({
          name: 'distinguishes different byte sequences by digest',
          fn: async () => {
            const dir = await makeTmpDir();
            const fileA = join(dir, 'a.txt',);
            const fileB = join(dir, 'b.txt',);
            await writeFile(fileA, 'one',);
            await writeFile(fileB, 'two',);
            const cache = new HashCache();
            const hashA = await cache.hashFile(fileA,);
            const hashB = await cache.hashFile(fileB,);
            expect(hashA,).not.toBe(hashB,);
            await rm(dir, { recursive: true, },);
          },
        },),
        it({
          name: 'is deterministic for the same bytes across separate cache instances',
          fn: async () => {
            const dir = await makeTmpDir();
            const file = join(dir, 'c.txt',);
            await writeFile(file, 'hello',);
            const cacheA = new HashCache();
            const cacheB = new HashCache();
            expect(await cacheA.hashFile(file,),).toBe(await cacheB.hashFile(file,),);
            await rm(dir, { recursive: true, },);
          },
        },),
        it({
          name: 'does not mutate the cache map (read and store are separate)',
          fn: async () => {
            const dir = await makeTmpDir();
            const file = join(dir, 'd.txt',);
            await writeFile(file, 'hello',);
            const cache = new HashCache();
            await cache.hashFile(file,);
            expect(cache.size,).toBe(0,);
            expect(cache.has(file,),).toBe(false,);
            await rm(dir, { recursive: true, },);
          },
        },),
      ],
    },),
    describe({
      name: 'get / has / set / delete',
      children: [
        it({
          name: 'starts empty',
          fn: async () => {
            const cache = new HashCache();
            expect(cache.size,).toBe(0,);
            expect(cache.has('/anything',),).toBe(false,);
            expect(cache.get('/anything',),).toBeUndefined();
          },
        },),
        it({
          name: 'set records the value; has and get observe it',
          fn: async () => {
            const cache = new HashCache();
            cache.set({ path: '/abs/a', hash: 'deadbeef', },);
            expect(cache.has('/abs/a',),).toBe(true,);
            expect(cache.get('/abs/a',),).toBe('deadbeef',);
            expect(cache.size,).toBe(1,);
          },
        },),
        it({
          name: 'set overwrites previous value at the same path',
          fn: async () => {
            const cache = new HashCache();
            cache.set({ path: '/abs/a', hash: 'first', },);
            cache.set({ path: '/abs/a', hash: 'second', },);
            expect(cache.get('/abs/a',),).toBe('second',);
            expect(cache.size,).toBe(1,);
          },
        },),
        it({
          name: 'delete removes the entry; size shrinks; has reports false',
          fn: async () => {
            const cache = new HashCache();
            cache.set({ path: '/abs/a', hash: 'aaa', },);
            cache.set({ path: '/abs/b', hash: 'bbb', },);
            expect(cache.size,).toBe(2,);
            expect(cache.delete('/abs/a',),).toBe(true,);
            expect(cache.size,).toBe(1,);
            expect(cache.has('/abs/a',),).toBe(false,);
            expect(cache.has('/abs/b',),).toBe(true,);
          },
        },),
        it({
          name: 'delete returns false for an unknown path',
          fn: async () => {
            const cache = new HashCache();
            expect(cache.delete('/not-there',),).toBe(false,);
          },
        },),
        it({
          name: 'two entries with different paths coexist',
          fn: async () => {
            const cache = new HashCache();
            cache.set({ path: '/abs/a', hash: 'aaa', },);
            cache.set({ path: '/abs/b', hash: 'bbb', },);
            expect(cache.get('/abs/a',),).toBe('aaa',);
            expect(cache.get('/abs/b',),).toBe('bbb',);
            expect(cache.size,).toBe(2,);
          },
        },),
      ],
    },),
    describe({
      name: 'DEFAULT_MAX_HASH_SIZE_BYTES',
      children: [
        it({
          name: 'is 16 MiB',
          fn: async () => {
            expect(DEFAULT_MAX_HASH_SIZE_BYTES,).toBe(16 * BYTES_PER_MIB,);
          },
        },),
      ],
    },),
  ],
},);
