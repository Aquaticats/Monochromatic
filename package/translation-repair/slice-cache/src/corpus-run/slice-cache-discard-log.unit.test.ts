/**
 Tests that the slice cache SAYS what it threw away, and only when it did.

 WHY THE LINE MATTERS. Slices are bought from the roster, so discarding a
 lane's cache costs real calls to rebuy. The count is the only notice an
 operator gets that a generation change just spent that money, and the module
 beside it records six occasions where an unregistered prefix made the repair
 lane delete another lane's work while reporting it as its own.

 WHAT WAS MEASURED. On 2026-08-25, inverting the guard that decides whether to
 print at all failed no test in this package. A cache holding nothing of this
 lane's would then announce that it discarded zero slices on every run, which
 is the line an operator reads as "money was spent" appearing where none was.

 THE QUIET CASE IS THE ONE THAT PROVES IT. Any guard at all satisfies "a
 discard says so"; only "a lane owning nothing here says nothing" separates a
 count that is read from a line that is always printed.

 OWNERSHIP IS ASSERTED ALONGSIDE, since the same call removes what it names:
 another lane's files and this lane's own marker must survive a discard, and a
 count that was right while the removal was wrong would still be a defect.

 THE SUITE RUNS AT `concurrency: 1`, since each case diverts the one global
 `console.log` across an await. Run concurrently they capture each other's
 lines, and the assertions then describe whichever case happened to be inside
 the window rather than the one under test.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  mkdir,
  readdir,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  discardNamespace,
  openNamespacedCache,
  type SliceNamespace,
} from '../../dist/final/node/index.mjs';
import { scratchDirOfEmptyRecords, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

//region Fixtures

/**
 Lane under test, owning every `.json` name carrying its prefix.
 */
const MITTENS: SliceNamespace = {
  prefix: 'mittens.',
  marker: 'mittens.filled-by',
};

/**
 File another lane owns, which a discard must leave alone.
 */
const OTHER_LANE_FILE = 'whiskers.c.json';

/**
 Diverts `console.log` into a list until disposed.

 @param lines - where diverted lines are appended

 @returns Capture holding those lines, which restores logging on disposal

 @example
 ```ts
 using capture = collectingInto({ lines, },);
 ```
 */
function collectingInto(
  { lines, }: { readonly lines: string[]; },
): { readonly lines: readonly string[]; } & Disposable {
  /**
   Real logger, put back on disposal.
   */
  const printed = console.log;

  /**
   `console.info` as it was, since the tagged logger's `info` sink resolves
   to it rather than to `console.log`.
   */
  const informed = console.info;
  console.log = (...parts: readonly unknown[]) => {
    lines.push(parts.map(String,)
      .join(' ',),);
  };
  console.info = (...parts: readonly unknown[]) => {
    lines.push(parts.map(String,)
      .join(' ',),);
  };
  return {
    lines,
    [Symbol.dispose]: () => {
      console.log = printed;
      console.info = informed;
    },
  };
}

/**
 Builds a throwaway cache directory holding the named files, each empty.

 @param names - file names to create

 @returns Directory holding them, removed when its `await using` scope ends

 @example
 ```ts
 await using cache = await cacheHolding({ names: ['mittens.a.json',], },);
 const dir = cache.dir;
 ```
 */
async function cacheHolding(
  { names, }: { readonly names: readonly string[]; },
): Promise<{ readonly dir: string; } & AsyncDisposable> {
  // Throwaway directory standing in for a shared slice cache.
  return await scratchDirOfEmptyRecords({
    prefix: 'translation-repair-slice-cache-',
    names,
  },);
}

/**
 Discards one lane's slices and reports what was printed and what survived.

 @param names - files the cache holds beforehand

 @param cached - pipeline stamp the discarded slices were filled by

 @returns Printed lines and the names still on disk afterwards

 @example
 ```ts
 const { lines, left, } = await discarding({ names, cached: 'nap-3', },);
 ```
 */
async function discarding(
  {
    names,
    cached,
  }: {
    readonly names: readonly string[];
    readonly cached: string;
  },
): Promise<{ readonly lines: readonly string[]; readonly left: readonly string[]; }> {
  /**
   Cache to discard from.
   */
  await using cache = await cacheHolding({ names, },);
  const { dir, } = cache;

  /**
   Lines the discard printed.
   */
  const lines: string[] = [];

  using capture = collectingInto({ lines, },);

  await discardNamespace({
    dir,
    namespace: MITTENS,
    cached,
  },);

  /**
   Names still on disk, sorted so the assertion does not depend on readdir
   order.
   */
  const left = (await readdir(dir,)).toSorted();

  return {
    lines: [...capture.lines,],
    left,
  };
}

/**
 Diverts `console.warn`, where the tagged logger's `warn` sink resolves, into
 a list until disposed.

 @param lines - where diverted warnings are appended

 @returns Capture restoring `console.warn` on disposal

 @example
 ```ts
 using capture = warningsInto({ lines, },);
 ```
 */
function warningsInto(
  { lines, }: { readonly lines: string[]; },
): Disposable {
  /**
   `console.warn` as it was, put back on disposal.
   */
  const warned = console.warn;
  console.warn = (...parts: readonly unknown[]) => {
    lines.push(parts.map(String,)
      .join(' ',),);
  };
  return {
    [Symbol.dispose]: () => {
      console.warn = warned;
    },
  };
}

/**
 Accepts any stored object, so the envelope decides what resumes; the loader
 hands a guard nothing at all for a file that is no envelope.

 @param value - stored record, or nothing

 @returns Whether a record is there

 @example
 ```ts
 await openNamespacedCache({ dir, generation, namespace, isValue: anyRecord, },);
 ```
 */
function anyRecord(value: unknown,): value is object {
  return ((typeof value) === 'object') && (value !== null);
}

//endregion Fixtures

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: discardNamespace.name,
      // SEQUENTIAL BECAUSE THE CAPTURE IS GLOBAL. `console.log` is one binding, and
      // each case here holds it across an await while the discard runs. Run
      // concurrently, the cases divert each other's lines into each other's lists,
      // which is how this file first failed: one case saw none of its own and
      // another saw two.
      concurrency: 1,
      children: [
        it({
          name: 'COUNTS what it removed and NAMES who filled it, since slices are bought from the roster '
            + 'and this line is the only notice that a generation change just spent that money again',
          fn: async () => {
            /**
             A cache holding two of this lane's slices, one of another lane's, and
             this lane's own marker, which is deliberately not a `.json` name.
             */
            const { lines, left, } = await discarding({
              names: [
                'mittens.a.json',
                'mittens.b.json',
                OTHER_LANE_FILE,
                MITTENS.marker,
              ],
              cached: 'nap-3',
            },);

            /**
             Lines announcing a discard, which is the only kind this asks about.
             THROUGH THE LOGGER, so the line carries its level and tag prefix and
             the notice is found inside it rather than at its start.
             */
            const announced = lines.filter(function isDiscard(line,): boolean {
              return line.includes('SLICE discarding',);
            },);

            expect(announced.length,).toBe(1,);
            // THE LOGGER'S TAG proves the notice went through the tagged logger
            // rather than a bare console call, which is what lets it be filtered
            // and attributed like every other cache line.
            expect(announced[0],).toContain('[discardNamespace]',);
            expect(announced[0],).toContain('discarding 2 cached slices',);
            expect(announced[0],).toContain('filled by nap-3',);
            expect(left,).toStrictEqual([
              MITTENS.marker,
              OTHER_LANE_FILE,
            ],);
          },
        },),

        it({
          name: 'STAYS QUIET when this lane owned nothing here, which is what keeps the line the "COUNTS what '
            + 'it removed" case reads worth reading rather than printed on every run',
          fn: async () => {
            /**
             A cache holding only another lane's slice.
             */
            const { lines, left, } = await discarding({
              names: [OTHER_LANE_FILE,],
              cached: 'nap-3',
            },);

            expect(lines.filter(function isDiscard(line,): boolean {
              return line.includes('SLICE discarding',);
            },),).toStrictEqual([],);
            expect(left,).toStrictEqual([OTHER_LANE_FILE,],);
          },
        },),

        it({
          name: 'SAYS unstamped where no pipeline claimed the slices, so a reader is never shown an empty '
            + 'name and left to guess whether one was recorded',
          fn: async () => {
            /**
             A cache whose lane never wrote a marker.
             */
            const { lines, } = await discarding({
              names: ['mittens.a.json',],
              cached: '',
            },);

            expect(lines.length,).toBe(1,);
            expect(lines[0],).toContain('filled by (unstamped)',);
          },
        },),

        it({
          name: 'LEAVES A DIRECTORY NAMED LIKE ONE OF ITS SLICES, which no lane wrote, rather than raising out of '
            + 'the open that asked for the discard (ledger B65)',
          fn: async () => {
            /**
             A cache holding one of this lane's slices beside a directory named
             like another.
             */
            await using cache = await cacheHolding({ names: ['mittens.a.json',], },);
            const { dir, } = cache;
            await mkdir(join(
              dir,
              'mittens.b.json',
            ),);

            /**
             Lines the discard printed, captured so none reach the test output.
             */
            const lines: string[] = [];
            {
              using capture = collectingInto({ lines, },);
              await discardNamespace({
                dir,
                namespace: MITTENS,
                cached: 'nap-3',
              },);
            }

            /**
             Names still on disk.
             */
            const left = (await readdir(dir,)).toSorted();

            expect(left,).toStrictEqual(['mittens.b.json',],);
          },
        },),
      ],
    },),

    describe({
      name: openNamespacedCache.name,
      // Sequential for the same reason as the discard cases: `console.warn` is one
      // binding held across an await.
      concurrency: 1,
      children: [
        it({
          name: 'WARNS about a slice file that does not parse, naming it, since that slice is bought again '
            + '(ledger A11)',
          fn: async () => {
            /**
             Cache this lane filled under the running pipeline: one slice torn
             mid-write, one that parses but is no envelope this loader wrote.
             */
            await using holding = await cacheHolding({ names: ['mittens.b.json',], },);
            const { dir, } = holding;
            await writeFile(
              join(
                dir,
                MITTENS.marker,
              ),
              'nap-3\n',
              'utf8',
            );
            await writeFile(
              join(
                dir,
                'mittens.a.json',
              ),
              '{"key":"a","val',
              'utf8',
            );

            /**
             Warnings the open printed.
             */
            const lines: string[] = [];
            {
              using capture = warningsInto({ lines, },);
              /**
               Cache as the lane opens it.
               */
              const cache = await openNamespacedCache({
                dir,
                generation: 'nap-3',
                namespace: MITTENS,
                isValue: anyRecord,
              },);
              expect(cache.resumed.size,).toBe(0,);
            }

            // Positive control: the refused envelope was already announced, so the
            // capture sees this loader's warnings.
            expect(lines.filter(function namesB(line,): boolean {
              return line.includes('mittens.b.json',);
            },).length,).toBe(1,);

            /**
             Warnings naming the torn slice.
             */
            const torn = lines.filter(function namesA(line,): boolean {
              return line.includes('mittens.a.json',);
            },);
            expect(torn.length,).toBe(1,);
            expect(torn[0],).toContain('recomputed',);
          },
        },),

        it({
          name: 'RESUMES PAST A DIRECTORY NAMED LIKE ONE OF ITS SLICES, which no lane wrote, rather than stopping '
            + 'the entry on EISDIR (ledger B65)',
          fn: async () => {
            /**
             Cache this lane filled under the running pipeline: one slice the
             loader wrote, beside a directory named like another.
             */
            await using cache = await cacheHolding({ names: [], },);
            const { dir, } = cache;
            await writeFile(
              join(
                dir,
                MITTENS.marker,
              ),
              'nap-3\n',
              'utf8',
            );
            await writeFile(
              join(
                dir,
                'mittens.a.json',
              ),
              JSON.stringify({
                cacheKey: 'a',
                record: { purred: true, },
              },),
              'utf8',
            );
            await mkdir(join(
              dir,
              'mittens.b.json',
            ),);

            /**
             Keys the open resumed.
             */
            const resumed = [
              ...(await openNamespacedCache({
                dir,
                generation: 'nap-3',
                namespace: MITTENS,
                isValue: anyRecord,
              },)).resumed.keys(),
            ];
            expect(resumed,).toStrictEqual(['a',],);
          },
        },),
      ],
    },),
  ],
},);
