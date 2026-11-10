/**
 Tests for the zstd compression helpers: worker distribution, the worker-count
 heuristic, tally arithmetic, and per-bucket compression.
 
 @module
 */

import {
  mkdtemp,
  readFile,
  rm,
  stat,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import {
  addTallies,
  compressBucket,
  distribute,
  EMPTY_TALLY,
  resolveWorkerCount,
  type Tally,
} from '../../dist/final/node/index.mjs';

/**
 Benchmark fixture size, from the doc example the worker heuristic carries.
 */
const DOCUMENTED_FILE_COUNT = 412;

/**
 Repeated byte that zstd shrinks far below its source size.
 */
const COMPRESSIBLE_PAYLOAD = 'a'.repeat(4_096,);

/**
 Extension the compressor never reads, because zstd cannot shrink it.
 */
const INCOMPRESSIBLE_EXTENSION = 'avif';

/**
 Tallies whose sum is asserted field by field.
 */
const LEFT_TALLY: Tally = {
  written: 2,
  skipped: 1,
  savedBytes: 100,
};

/**
 Second tally added to {@link LEFT_TALLY}.
 */
const RIGHT_TALLY: Tally = {
  written: 3,
  skipped: 4,
  savedBytes: 250,
};

/**
 Report of whether a path exists, distinguishing absence from other failures.
 
 @param path - path to probe
 
 @returns `true` when the path exists
 
 @throws when the probe fails for any reason other than absence
 
 @example
 ```ts
 await pathExists('/nonexistent');
 // false
 ```
 */
async function pathExists(path: string,): Promise<boolean> {
  try {
    await stat(path,);
    return true;
  }
  catch (error) {
    /**
     Signals a path that is simply not there, the expected answer.
     */
    const isAbsent = Error.isError(error,)
      && ('code' in error)
      && (error.code
        === 'ENOENT');
    if (isAbsent)
      return false;
    throw error;
  }
}

/**
 Sets the `ZSTD_WORKERS` override and returns a handle that unsets it.
 
 @param value - raw override value to install
 
 @returns handle whose disposal removes the override
 
 @example
 ```ts
 using override = setWorkerOverride('3');
 ```
 */
function setWorkerOverride(value: string,): Disposable {
  process.env.ZSTD_WORKERS = value;
  return {
    [Symbol.dispose](): void {
      delete process.env.ZSTD_WORKERS;
    },
  };
}

/**
 Scratch fixture: a fresh directory plus a writer, removed when the enclosing
 scope ends.
 */
type ScratchFixture = {
  /**
   Absolute path of the fixture directory.
   */
  readonly path: string;
  /**
   Writes a file into the directory and returns its absolute path.
   */
  readonly writeFile: (input: {
    readonly name: string;
    readonly content: string;
  },) => Promise<string>;
  /**
   Removes the directory and everything in it.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Creates a scratch fixture under the shared test scratch root.
 
 @returns fixture whose disposal deletes the directory
 
 @example
 ```ts
 await using scratch = await createScratchDirectory();
 ```
 */
async function createScratchDirectory(): Promise<ScratchFixture> {
  /**
   Directory name minted for this fixture alone.
   */
  const path = await mkdtemp(join(
    scratchRoot(),
    'compress-lib-',
  ),);

  return {
    path,
    async writeFile(
      {
        name,
        content,
      },
    ): Promise<string> {
      /**
       Absolute path of the file inside the fixture directory.
       */
      const filePath = join(
        path,
        name,
      );
      await writeFile(
        filePath,
        content,
        'utf8',
      );
      return filePath;
    },
    async [Symbol.asyncDispose](): Promise<void> {
      await rm(
        path,
        { force: true, recursive: true, },
      );
    },
  };
}

await describe({
  name: '',
  children: [
    describe({
      name: distribute.name,
      children: [
        it({
          name: 'round-robins files into one bucket per worker',
          fn: async function roundRobinsFiles(): Promise<void> {
            expect(distribute({
              files: ['a', 'b', 'c',],
              workers: 2,
            },),).toEqual([['a', 'c',], ['b',],],);
          },
        },),
        it({
          name: 'returns one empty bucket per worker for no files',
          fn: async function returnsEmptyBuckets(): Promise<void> {
            expect(distribute({
              files: [],
              workers: 3,
            },),).toEqual([[], [], [],],);
          },
        },),
        it({
          name: 'leaves surplus buckets empty when workers outnumber files',
          fn: async function leavesSurplusBucketsEmpty(): Promise<void> {
            expect(distribute({
              files: ['only',],
              workers: 3,
            },),).toEqual([['only',], [], [],],);
          },
        },),
      ],
    },),
    describe({
      name: resolveWorkerCount.name,
      concurrency: 1,
      children: [
        it({
          name: 'honours a ZSTD_WORKERS override',
          fn: async function honoursOverride(): Promise<void> {
            using override = setWorkerOverride('3',);

            expect(await resolveWorkerCount({ fileCount: DOCUMENTED_FILE_COUNT, },),).toBe(3,);
          },
        },),
        it({
          name: 'caps the override at the file count',
          fn: async function capsOverrideAtFileCount(): Promise<void> {
            using override = setWorkerOverride('8',);

            expect(await resolveWorkerCount({ fileCount: 2, },),).toBe(2,);
          },
        },),
        it({
          name: 'rejects an override that is not a positive integer',
          fn: async function rejectsInvalidOverride(): Promise<void> {
            using override = setWorkerOverride('0',);
            /**
             Error raised for the unusable override.
             */
            const caught = await (async function catchWorkerError(): Promise<unknown> {
              try {
                return await resolveWorkerCount({ fileCount: 10, },);
              }
              catch (error) {
                return error;
              }
            })();

            expect(caught,).toBeInstanceOf(Error,);
            if (!Error.isError(caught,))
              throw new Error('Expected resolveWorkerCount to reject the override.',);
            expect(caught.message,).toContain('ZSTD_WORKERS',);
          },
        },),
        it({
          name: 'derives at least one worker with no override',
          fn: async function derivesAtLeastOneWorker(): Promise<void> {
            /**
             Worker count for a single candidate file.
             */
            const workers = await resolveWorkerCount({ fileCount: 1, },);

            expect(workers,).toBe(1,);
          },
        },),
      ],
    },),
    describe({
      name: addTallies.name,
      children: [
        it({
          name: 'sums every field',
          fn: async function sumsEveryField(): Promise<void> {
            expect(addTallies({
              left: LEFT_TALLY,
              right: RIGHT_TALLY,
            },),).toEqual({
              written: LEFT_TALLY.written + RIGHT_TALLY.written,
              skipped: LEFT_TALLY.skipped + RIGHT_TALLY.skipped,
              savedBytes: LEFT_TALLY.savedBytes + RIGHT_TALLY.savedBytes,
            },);
          },
        },),
        it({
          name: 'returns the other tally when one side is the empty tally',
          fn: async function returnsOtherTallies(): Promise<void> {
            expect(addTallies({
              left: EMPTY_TALLY,
              right: RIGHT_TALLY,
            },),).toEqual(RIGHT_TALLY,);
          },
        },),
      ],
    },),
    describe({
      name: compressBucket.name,
      children: [
        it({
          name: 'writes a zstd companion for a compressible file',
          fn: async function writesCompanion(): Promise<void> {
            await using scratch = await createScratchDirectory();
            /**
             Path of the compressible source file.
             */
            const source = await scratch.writeFile({
              name: 'page.html',
              content: COMPRESSIBLE_PAYLOAD,
            },);

            /**
             Tally reported for the bucket.
             */
            const tally = compressBucket({ files: [source,], },);
            /**
             Size of the companion the bucket wrote.
             */
            const companionSize = (await readFile(`${source}.zst`,)).length;

            expect(tally,).toEqual({
              written: 1,
              skipped: 0,
              savedBytes: COMPRESSIBLE_PAYLOAD.length
                - companionSize,
            },);
          },
        },),
        it({
          name: 'skips an extension the compressor cannot shrink',
          fn: async function skipsIncompressibleExtension(): Promise<void> {
            await using scratch = await createScratchDirectory();
            /**
             Path of the source file with an already-compressed extension.
             */
            const source = await scratch.writeFile({
              name: `image.${INCOMPRESSIBLE_EXTENSION}`,
              content: COMPRESSIBLE_PAYLOAD,
            },);

            expect(compressBucket({ files: [source,], },),).toEqual({
              written: 0,
              skipped: 1,
              savedBytes: 0,
            },);
          },
        },),
        it({
          name: 'removes a stale companion before deciding',
          fn: async function removesStaleCompanion(): Promise<void> {
            await using scratch = await createScratchDirectory();
            /**
             Path of the source file.
             */
            const source = await scratch.writeFile({
              name: `image.${INCOMPRESSIBLE_EXTENSION}`,
              content: COMPRESSIBLE_PAYLOAD,
            },);
            await writeFile(
              `${source}.zst`,
              'stale',
              'utf8',
            );

            compressBucket({ files: [source,], },);

            expect(await pathExists(`${source}.zst`,),).toBe(false,);
          },
        },),
        it({
          name: 'reports an empty accumulator for an empty bucket',
          fn: async function reportsEmptyBucket(): Promise<void> {
            expect(compressBucket({ files: [], },),).toEqual(EMPTY_TALLY,);
          },
        },),
      ],
    },),
  ],
},);
