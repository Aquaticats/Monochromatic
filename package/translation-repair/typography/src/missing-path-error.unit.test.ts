/**
 Tests for reading the filesystem's "nothing stands here" answer apart from
 every other failure.

 THE FAILURES ARE REAL ONES, raised by `readFile` in a throwaway directory:
 the property that matters is what Node actually throws, and a hand-built
 object with a `code` field is exactly the value the predicate must refuse.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  isMissingPathError,
  readTextOrEmptyIfMissing,
  rethrowUnlessMissingPath,
} from '../dist/final/node/index.mjs';
import {
  scratchDir,
  scratchDirWith,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

/**
 Runs a body and hands back what it threw.

 @param body - what to run

 @returns Thrown value

 @throws {@link Error} when the body throws nothing, since every case here
 expects a failure

 @example
 ```ts
 const thrown = await caughtFrom({ body: async () => { await readFile(path,); }, },);
 ```
 */
async function caughtFrom(
  { body, }: { readonly body: () => Promise<void>; },
): Promise<unknown> {
  try {
    await body();
  }
  catch (error) {
    return error;
  }
  throw new Error('the body threw nothing',);
}

/**
 A throwaway directory holding one regular file, and the two real failures
 reading under it raises.

 @returns `ENOENT` of a file that is not there, `ENOTDIR` of a path
 through the regular file, and how to remove the directory

 @example
 ```ts
 await using failures = await realFailures();
 ```
 */
async function realFailures(): Promise<{
  readonly absent: unknown;
  readonly throughFile: unknown;
} & AsyncDisposable> {
  // Directory this case owns.
  return await scratchDirWith({
    prefix: 'missing-path-error-',
    setup: async function seeded({ path: dir, },): Promise<{
      readonly absent: unknown;
      readonly throughFile: unknown;
    }> {
      /**
       A regular file where a directory would have to be.
       */
      const litterBox = join(
        dir,
        'litter-box.txt',
      );
      await writeFile(
        litterBox,
        'A cat sleeps here.\n',
      );
      return {
        absent: await caughtFrom({
          body: async () => {
            await readFile(join(
              dir,
              'no-cat-here.txt',
            ),);
          },
        },),
        throughFile: await caughtFrom({
          body: async () => {
            await readFile(join(
              litterBox,
              'kitten.txt',
            ),);
          },
        },),
      };
    },
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: isMissingPathError.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS a real ENOENT as the missing-path answer, and a real ENOTDIR, a plain object carrying '
            + 'the code, and a thrown string as not it',
          fn: async () => {
            await using failures = await realFailures();
            // The fixture's failures are the ones it names, so the readings that
            // follow are about those codes and not about some other failure.
            expect(failures.absent,).toHaveProperty(
              'code',
              'ENOENT',
            );
            expect(failures.throughFile,).toHaveProperty(
              'code',
              'ENOTDIR',
            );

            expect({
              absent: isMissingPathError({ error: failures.absent, },),
              throughFile: isMissingPathError({ error: failures.throughFile, },),
              plainObject: isMissingPathError({ error: { code: 'ENOENT', }, },),
              thrownString: isMissingPathError({ error: 'ENOENT', },),
            },).toEqual({
              absent: true,
              throughFile: false,
              plainObject: false,
              thrownString: false,
            },);
          },
        },),
      ],
    },),

    describe({
      name: rethrowUnlessMissingPath.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'RETURNS for the missing-path answer, so the catch goes on to its "not there" reading',
          fn: async () => {
            await using failures = await realFailures();

            expect(rethrowUnlessMissingPath({ error: failures.absent, },),).toBeUndefined();
          },
        },),
        it({
          name: 'RETHROWS every other failure as it came, the same value and not a wrapper, so the caller\'s '
            + 'caller sees the real fault',
          fn: async () => {
            await using failures = await realFailures();
            /**
             A plain object carrying the code, which no Node call throws.
             */
            const lookalike = { code: 'ENOENT', };

            expect({
              throughFile: await caughtFrom({
                body: async () => {
                  rethrowUnlessMissingPath({ error: failures.throughFile, },);
                },
              },) === failures.throughFile,
              lookalike: await caughtFrom({
                body: async () => {
                  rethrowUnlessMissingPath({ error: lookalike, },);
                },
              },) === lookalike,
            },).toEqual({
              throughFile: true,
              lookalike: true,
            },);
          },
        },),
      ],
    },),

    describe({
      name: readTextOrEmptyIfMissing.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS a file\'s text, EMPTY where nothing stands at the path, and RAISES a read that failed for '
            + 'any other reason rather than calling it empty',
          fn: async () => {
            /**
             Directory this case owns.
             */
            await using owned = await scratchDir({ prefix: 'read-text-if-present-', },);
            /**
             Working path of the directory this case owns.
             */
            const dir = owned.path;
            /**
             A file with a line in it.
             */
            const diary = join(
              dir,
              'diary.txt',
            );
            await writeFile(
              diary,
              'The cat napped.\n',
            );

            expect({
              present: await readTextOrEmptyIfMissing({ path: diary, },),
              absent: await readTextOrEmptyIfMissing({
                path: join(
                  dir,
                  'no-diary.txt',
                ),
              },),
            },).toEqual({
              present: 'The cat napped.\n',
              absent: '',
            },);
            await expect(readTextOrEmptyIfMissing({
              path: join(
                diary,
                'page.txt',
              ),
            },),).rejects.toHaveProperty(
              'code',
              'ENOTDIR',
            );
          },
        },),
      ],
    },),
  ],
},);
