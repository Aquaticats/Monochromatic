/**
 Tests for the zstd compression helpers: worker distribution, the worker-count
 heuristic, tally arithmetic, and per-bucket compression.
 
 @module
 */

import {
  mkdtemp,
  readFile,
  rm,
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
  type Tally,
  resolveWorkerCount,
} from './compress-lib.ts';

/**
 Benchmark fixture size, from the doc example the worker heuristic carries.
 */
const DOCUMENTED_FILE_COUNT = 412;

/**
 Repeated byte that zstd shrinks far below its source size.
 */
const COMPRESSIBLE_PAYLOAD = 'a'.repeat(4096,);

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
 Creates a disposable directory under the shared test scratch root.
 
 @returns absolute path of the new directory
 
 @example
 ```ts
 const directory = await createScratchDirectory();
 ```
 */
async function createScratchDirectory(): Promise<string> {
  return mkdtemp(join(
    scratchRoot(),
    'compress-lib-',
  ),);
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
            process.env['ZSTD_WORKERS'] = '3';
            try {
              expect(await resolveWorkerCount({ fileCount: DOCUMENTED_FILE_COUNT, },),).toBe(3,);
            }
            finally {
              delete process.env['ZSTD_WORKERS'];
            }
          },
        },),
        it({
          name: 'caps the override at the file count',
          fn: async function capsOverrideAtFileCount(): Promise<void> {
            process.env['ZSTD_WORKERS'] = '8';
            try {
              expect(await resolveWorkerCount({ fileCount: 2, },),).toBe(2,);
            }
            finally {
              delete process.env['ZSTD_WORKERS'];
            }
          },
        },),
        it({
          name: 'rejects an override that is not a positive integer',
          fn: async function rejectsInvalidOverride(): Promise<void> {
            process.env['ZSTD_WORKERS'] = '0';
            /**
             Error raised for the unusable override.
             */
            let caught: unknown;
            try {
              await resolveWorkerCount({ fileCount: 10, },);
            }
            catch (error) {
              caught = error;
            }
            finally {
              delete process.env['ZSTD_WORKERS'];
            }

            expect(caught,).toBeInstanceOf(Error,);
            expect((caught as Error).message,).toContain('ZSTD_WORKERS',);
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
            /**
             Disposable directory holding the source file.
             */
            const directory = await createScratchDirectory();
            /**
             Path of the compressible source file.
             */
            const source = join(
              directory,
              'page.html',
            );
            try {
              await writeFile(
                source,
                COMPRESSIBLE_PAYLOAD,
                'utf8',
              );

              /**
               Tally reported for the bucket.
               */
              const tally = compressBucket({ files: [source,], },);

              expect(tally,).toEqual({
                written: 1,
                skipped: 0,
                savedBytes: COMPRESSIBLE_PAYLOAD.length
                  - (await readFile(`${source}.zst`,)).length,
              },);
            }
            finally {
              await rm(
                directory,
                { force: true, recursive: true, },
              );
            }
          },
        },),
        it({
          name: 'skips an extension the compressor cannot shrink',
          fn: async function skipsIncompressibleExtension(): Promise<void> {
            /**
             Disposable directory holding the source file.
             */
            const directory = await createScratchDirectory();
            /**
             Path of the source file with an already-compressed extension.
             */
            const source = join(
              directory,
              `image.${INCOMPRESSIBLE_EXTENSION}`,
            );
            try {
              await writeFile(
                source,
                COMPRESSIBLE_PAYLOAD,
                'utf8',
              );

              expect(compressBucket({ files: [source,], },),).toEqual({
                written: 0,
                skipped: 1,
                savedBytes: 0,
              },);
            }
            finally {
              await rm(
                directory,
                { force: true, recursive: true, },
              );
            }
          },
        },),
        it({
          name: 'removes a stale companion before deciding',
          fn: async function removesStaleCompanion(): Promise<void> {
            /**
             Disposable directory holding the source and its stale companion.
             */
            const directory = await createScratchDirectory();
            /**
             Path of the source file.
             */
            const source = join(
              directory,
              `image.${INCOMPRESSIBLE_EXTENSION}`,
            );
            try {
              await writeFile(
                source,
                COMPRESSIBLE_PAYLOAD,
                'utf8',
              );
              await writeFile(
                `${source}.zst`,
                'stale',
                'utf8',
              );

              compressBucket({ files: [source,], },);

              /**
               Contents of the companion after the run, or a marker when it is gone.
               */
              let companion = 'present';
              try {
                await readFile(`${source}.zst`,);
              }
              catch {
                companion = 'removed';
              }
              expect(companion,).toBe('removed',);
            }
            finally {
              await rm(
                directory,
                { force: true, recursive: true, },
              );
            }
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
