import {
  existsSync,
  mkdtempSync,
  rmSync,
  truncateSync,
  utimesSync,
  writeFileSync,
} from 'node:fs';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { maintainPersistentEffectCache, } from '../../dist/final/node/index.mjs';

/**
 Disposable persistent cache root.
 */
type CacheRootFixture = {
  readonly root: string;
  [Symbol.dispose]: () => void;
};

/**
 Bytes in one mebibyte.
 */
const BYTES_PER_MEBIBYTE = 1_048_576;

/**
 Seconds in one day.
 */
const SECONDS_PER_DAY = 86_400;

/**
 Entry count the cache keeps per dependency root.
 */
const RETAINED_ENTRY_COUNT = 16_384;

/**
 Creates a fresh cache root, because maintenance keeps a write counter per root.

 @returns disposable root under the operating-system temporary directory.
 */
function createCacheRootFixture(): CacheRootFixture {
  /**
   Unique cache root path.
   */
  const root = mkdtempSync(join(scratchRoot(), 'effect-cache-maintenance-',),);
  return {
    root,
    [Symbol.dispose]: function removeCacheRoot(): void {
      rmSync(root, { recursive: true, force: true, },);
    },
  };
}

/**
 Writes one cache entry with an explicit modification time.

 @param root - Cache root receiving the entry.

 @param name - Entry file name.

 @param modifiedAtSeconds - Modification time in seconds since the epoch.

 @param apparentMebibytes - Sparse apparent size in mebibytes, or zero for a tiny entry.

 @returns absolute entry path.
 */
function writeEntry({
  root,
  name,
  modifiedAtSeconds,
  apparentMebibytes,
}: {
  readonly root: string;
  readonly name: string;
  readonly modifiedAtSeconds: number;
  readonly apparentMebibytes: number;
}): string {
  /**
   Absolute entry path.
   */
  const path = join(
    root,
    name,
  );
  writeFileSync(
    path,
    '{}',
  );
  if (apparentMebibytes > 0)
    truncateSync(
      path,
      apparentMebibytes * BYTES_PER_MEBIBYTE,
    );
  utimesSync(
    path,
    modifiedAtSeconds,
    modifiedAtSeconds,
  );
  return path;
}

/**
 Writes entries whose modification times increase with their index.

 @param root - Cache root receiving the entries.

 @param count - Number of entries.

 @param apparentMebibytes - Sparse apparent size of each entry in mebibytes, or zero for tiny entries.

 @returns entry paths, oldest first.
 */
function writeAgedEntries({
  root,
  count,
  apparentMebibytes,
}: {
  readonly root: string;
  readonly count: number;
  readonly apparentMebibytes: number;
}): readonly string[] {
  /**
   Start of the entry timestamps, an hour old so no entry expires by age.
   */
  const startSeconds = Math.floor(Date.now() / 1_000,) - 3_600;
  return Array.from(
    { length: count, },
    function writeAgedEntry(
      _unused,
      index,
    ): string {
      return writeEntry({
        root,
        name: `entry-${String(index,).padStart(
          5,
          '0',
        )}.json`,
        modifiedAtSeconds: startSeconds + index,
        apparentMebibytes,
      },);
    },
  );
}

/**
 Lists the indexes of entries that no longer exist.

 @param paths - Entry paths in index order.

 @returns indexes whose files were removed.
 */
function removedIndexes(paths: readonly string[],): readonly number[] {
  return paths.flatMap(function removedIndex(
    path,
    index,
  ): readonly number[] {
    return existsSync(path,) ? [] : [index,];
  },);
}

await describe({
  name: maintainPersistentEffectCache.name,
  children: [
    it({
      name: 'keeps every entry when the count equals the entry bound',
      fn: async () => {
        using fixture = createCacheRootFixture();
        /**
         Entries filling the bound exactly.
         */
        const paths = writeAgedEntries({
          root: fixture.root,
          count: RETAINED_ENTRY_COUNT,
          apparentMebibytes: 0,
        },);
        maintainPersistentEffectCache({
          root: fixture.root,
          retainedPath: paths.at(-1,) ?? '',
        },);
        expect(removedIndexes(paths,).length,).toBe(0,);
      },
    },),
    it({
      name: 'removes the two oldest entries when the count exceeds the entry bound by two',
      fn: async () => {
        using fixture = createCacheRootFixture();
        /**
         Entries two past the bound.
         */
        const paths = writeAgedEntries({
          root: fixture.root,
          count: RETAINED_ENTRY_COUNT + 2,
          apparentMebibytes: 0,
        },);
        maintainPersistentEffectCache({
          root: fixture.root,
          retainedPath: paths.at(-1,) ?? '',
        },);
        /**
         Entries evicted by the call.
         */
        const removed = removedIndexes(paths,);
        expect(removed.length,).toBe(2,);
        expect(removed,).toEqual([
          0,
          1,
        ],);
      },
    },),
    it({
      name: 'never removes the retained entry even when it is the oldest',
      fn: async () => {
        using fixture = createCacheRootFixture();
        /**
         Entries two past the bound.
         */
        const paths = writeAgedEntries({
          root: fixture.root,
          count: RETAINED_ENTRY_COUNT + 2,
          apparentMebibytes: 0,
        },);
        maintainPersistentEffectCache({
          root: fixture.root,
          retainedPath: paths[0] ?? '',
        },);
        /**
         Entries evicted by the call.
         */
        const removed = removedIndexes(paths,);
        expect(removed.length,).toBe(2,);
        expect(removed,).toEqual([
          1,
          2,
        ],);
      },
    },),
    it({
      name: 'keeps two entries of 400 mebibytes under the byte bound',
      fn: async () => {
        using fixture = createCacheRootFixture();
        /**
         Entries totalling 800 mebibytes.
         */
        const paths = writeAgedEntries({
          root: fixture.root,
          count: 2,
          apparentMebibytes: 400,
        },);
        maintainPersistentEffectCache({
          root: fixture.root,
          retainedPath: paths.at(-1,) ?? '',
        },);
        expect(removedIndexes(paths,).length,).toBe(0,);
      },
    },),
    it({
      name: 'removes exactly the oldest of three entries of 400 mebibytes over the byte bound',
      fn: async () => {
        using fixture = createCacheRootFixture();
        /**
         Entries totalling 1200 mebibytes.
         */
        const paths = writeAgedEntries({
          root: fixture.root,
          count: 3,
          apparentMebibytes: 400,
        },);
        maintainPersistentEffectCache({
          root: fixture.root,
          retainedPath: paths.at(-1,) ?? '',
        },);
        expect(removedIndexes(paths,),).toEqual([0,],);
      },
    },),
    it({
      name: 'removes an entry older than seven days and ignores files that are not JSON',
      fn: async () => {
        using fixture = createCacheRootFixture();
        /**
         Current time in whole seconds.
         */
        const nowSeconds = Math.floor(Date.now() / 1_000,);
        /**
         Entry eight days old.
         */
        const expired = writeEntry({
          root: fixture.root,
          name: 'expired.json',
          modifiedAtSeconds: nowSeconds - (8 * SECONDS_PER_DAY),
          apparentMebibytes: 0,
        },);
        /**
         Entry six days old.
         */
        const recent = writeEntry({
          root: fixture.root,
          name: 'recent.json',
          modifiedAtSeconds: nowSeconds - (6 * SECONDS_PER_DAY),
          apparentMebibytes: 0,
        },);
        /**
         Non-JSON file eight days old.
         */
        const foreign = writeEntry({
          root: fixture.root,
          name: 'expired.txt',
          modifiedAtSeconds: nowSeconds - (8 * SECONDS_PER_DAY),
          apparentMebibytes: 0,
        },);
        /**
         Current entry.
         */
        const current = writeEntry({
          root: fixture.root,
          name: 'current.json',
          modifiedAtSeconds: nowSeconds,
          apparentMebibytes: 0,
        },);
        maintainPersistentEffectCache({
          root: fixture.root,
          retainedPath: current,
        },);
        expect(existsSync(expired,),).toBe(false,);
        expect(existsSync(recent,),).toBe(true,);
        expect(existsSync(foreign,),).toBe(true,);
        expect(existsSync(current,),).toBe(true,);
      },
    },),
  ],
},);
