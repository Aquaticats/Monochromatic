/**
 Tests for the persisted attempt map.

 `readAttemptMap` had no test. It backs the last ordering tiebreak, so an
 entry that keeps failing deprioritizes instead of blocking the queue ahead of
 entries that would settle.

 Its doctrine is the OPPOSITE of the artifact guards, deliberately: those
 throw on anything malformed because they feed a precision measurement, while
 this tolerates a corrupt cache because losing an ordering hint is cheaper
 than aborting a run that costs hours. The cases in this file pin where tolerance
 stops, since a reader that swallowed everything would make a misconfigured
 path look like "no attempts yet" forever and the ordering would never
 deprioritize anything.

 Fixtures are cat-themed invention written into throwaway directories.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import {
  readdir,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  type AttemptMap,
  attemptsOf,
  countAttempt,
  readAttemptMap,
  writeAttemptMap,
} from '../../dist/final/node/index.mjs';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

/**
 Writes an attempts file and reads it back.

 @param directory - throwaway directory

 @param contents - exact file bytes, so malformed cases stay malformed

 @param name - file name, so cases sharing a directory never share a file

 @returns Parsed attempt map

 @example
 ```ts
 const attempts = await readWritten({ directory, contents: '{"Kitten":2}', },);
 ```
 */
async function readWritten(
  {
    directory,
    contents,
    name = 'attempts.json',
  }: {
    readonly directory: string;
    readonly contents: string;
    readonly name?: string;
  },
): Promise<AttemptMap> {
  /**
   Path the attempts file occupies.
   */
  const attemptsPath = join(
    directory,
    name,
  );
  await writeFile(
    attemptsPath,
    contents,
    'utf8',
  );
  return await readAttemptMap(attemptsPath,);
}

/**
 Reads one attempts file while `console.warn`, where the tagged logger's
 `warn` sink resolves, is diverted into a list.

 @param contents - exact file bytes

 @returns Warnings the read printed, beside the path of the file it read

 @example
 ```ts
 const { lines, attemptsPath, } = await warningsReading({ contents: '{"Kitten":', },);
 ```
 */
async function warningsReading(
  { contents, }: { readonly contents: string; },
): Promise<{ readonly lines: readonly string[]; readonly attemptsPath: string; }> {
  await using scratch = await scratchDir({ prefix: 'whiskers-attempts-', },);
  /**
   Warnings the read printed.
   */
  const lines: string[] = [];
  /**
   `console.warn` as it was, put back once the read returns.
   */
  const warned = console.warn;
  console.warn = (...parts: readonly unknown[]) => {
    lines.push(parts.map(String,)
      .join(' ',),);
  };
  await using restore = {
    [Symbol.asyncDispose]: async () => {
      console.warn = warned;
    },
  };
  await readWritten({
    directory: scratch.path,
    contents,
  },);
  return {
    lines,
    attemptsPath: join(
      scratch.path,
      'attempts.json',
    ),
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: readAttemptMap.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'reads a well-formed map through unchanged, which is the ordinary '
            + 'case the ordering depends on',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'whiskers-attempts-', },);

            expect(
              await readWritten({
                directory: scratch.path,
                contents: JSON.stringify({
                  Mittens: 2,
                  Marmalade: 5,
                },),
              },),
            ).toStrictEqual(new Map([
              ['Mittens', 2,],
              ['Marmalade', 5,],
            ],),);
          },
        },),

        it({
          name: 'returns an empty map when the file is ABSENT, since a first run '
            + 'has no attempts recorded and that is not an error',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'whiskers-attempts-', },);

            expect(
              await readAttemptMap(join(
                scratch.path,
                'never-written.json',
              ),),
            ).toStrictEqual(new Map(),);
          },
        },),

        it({
          name: 'returns an empty map for MALFORMED JSON rather than aborting, '
            + 'because a truncated write from an interrupted run costs an ordering '
            + 'hint and must not cost the run itself',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'whiskers-attempts-', },);

            expect(
              await readWritten({
                directory: scratch.path,
                contents: '{"Mittens": 2,',
              },),
            ).toStrictEqual(new Map(),);
          },
        },),

        it({
          name: 'returns an empty map for well-formed JSON that is not an object, '
            + 'so a file holding a bare number or string cannot become an ordering '
            + 'input',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'whiskers-attempts-', },);

            // Each case writes its own file, so they share no state and run
            // concurrently rather than sequentially.
            const maps = await Promise.all([
              '42',
              '"Mittens"',
              'null',
              'true',
            ].map(async function toMap(contents, index,) {
              return await readWritten({
                directory: scratch.path,
                contents,
                name: `not-an-object-${String(index,)}.json`,
              },);
            },),);

            for (const map of maps)
              expect(map,).toStrictEqual(new Map(),);
          },
        },),

        it({
          name: 'returns an empty map for a well-formed JSON ARRAY rather than reading its index keys '
            + 'as entry ids, which is read through today as counts "0" and "1" instead of starting the '
            + 'ordering over (ledger B92)',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'whiskers-attempts-', },);

            expect(
              await readWritten({
                directory: scratch.path,
                contents: '[2, 5]',
              },),
            ).toStrictEqual(new Map(),);
          },
        },),

        it({
          name: 'COERCES a non-numeric count to zero rather than dropping the '
            + 'entry, keeping every recorded id in the map. Zero means fewest '
            + 'attempts, so a corrupted count makes that entry sort first, which '
            + 'is the tolerant direction: it retries an entry rather than starving '
            + 'it',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'whiskers-attempts-', },);

            expect(
              await readWritten({
                directory: scratch.path,
                contents: JSON.stringify({
                  Mittens: 'many',
                  Marmalade: 5,
                },),
              },),
            ).toStrictEqual(new Map([
              ['Mittens', 0,],
              ['Marmalade', 5,],
            ],),);
          },
        },),

        it({
          name: 'READS A COUNT THAT IS NOT A NON-NEGATIVE WHOLE NUMBER AS ZERO (1.5, -3, 1e999 and 1e20), keeping the '
            + 'entry and its neighbours, since the pass adds one to a count it reads and a stamp refuses a count that '
            + 'is not a whole number from 1, which a hand-edited file would otherwise turn into a crash at every launch',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'whiskers-attempts-', },);

            // Written as text: `JSON.stringify` writes the 1e999 that parses to Infinity as null.
            const maps = await Promise.all([
              '1.5',
              '-3',
              '1e999',
              '1e20',
            ].map(async function toMap(written, index,) {
              return await readWritten({
                directory: scratch.path,
                contents: `{"Mittens": ${written}, "Marmalade": 5}`,
                name: `hand-edited-${String(index,)}.json`,
              },);
            },),);

            for (const map of maps)
              expect(map,).toStrictEqual(new Map([
                ['Mittens', 0,],
                ['Marmalade', 5,],
              ],),);
          },
        },),

        it({
          name: 'READS THE LARGEST SAFE INTEGER AS ZERO, keeping the entry and its neighbours, since the pass adds one '
            + 'to a count it reads and that sum is no safe integer, which a stamp refuses, so a hand-edited '
            + '9007199254740991 would end the pass at its first entry and again at every launch',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'whiskers-attempts-', },);

            expect(
              await readWritten({
                directory: scratch.path,
                contents: '{"Mittens": 9007199254740991, "Marmalade": 5}',
              },),
            ).toStrictEqual(new Map([
              ['Mittens', 0,],
              ['Marmalade', 5,],
            ],),);
          },
        },),

        it({
          name: 'THROWS on a read fault that is neither absence nor malformed '
            + 'JSON. Pointing the path at a directory is the shape a misconfigured '
            + 'runs directory produces, and swallowing it would make every run '
            + 'read "no attempts yet" forever, so the ordering would never '
            + 'deprioritize an entry that keeps failing',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'whiskers-attempts-', },);

            await expect(readAttemptMap(scratch.path,),).rejects.toThrow();
          },
        },),

        it({
          name: 'reads an empty object as an empty map, distinguishing a run that '
            + 'recorded nothing from a file that was never written, both of which '
            + 'are legitimate',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'whiskers-attempts-', },);

            expect(
              await readWritten({
                directory: scratch.path,
                contents: '{}',
              },),
            ).toStrictEqual(new Map(),);
          },
        },),
      ],
    },),

    describe({
      name: 'readAttemptMap announcements',
      // SEQUENTIAL: each case diverts the one global `console.warn` across an
      // await, and concurrent cases would capture each other's lines.
      concurrency: 1,
      children: [
        it({
          name: 'SAYS it starts the counts over for malformed JSON and for JSON that is no object, and names a '
            + 'count it read as zero, since each silently forgets which entries kept failing (ledger A11)',
          fn: async () => {
            expect((await warningsReading({ contents: '{"Mittens": 2,', },)).lines.length,).toBe(1,);
            expect((await warningsReading({ contents: '42', },)).lines.length,).toBe(1,);

            /**
             Warnings for a count that is no number.
             */
            const coerced = (await warningsReading({
              contents: JSON.stringify({
                Mittens: 'many',
                Marmalade: 5,
              },),
            },)).lines;
            expect(coerced.length,).toBe(1,);
            expect(coerced[0],).toContain('Mittens',);
          },
        },),
        it({
          name: 'NAMES THE ENTRY, THE FILE AND THE VALUE of a count it reads as zero because it is no non-negative '
            + 'whole number, since a reader of the log cannot tell which hand-edited count was thrown away from a '
            + 'warning that names only the entry',
          fn: async () => {
            for (
              const [
                written,
                shown,
              ] of [
                [
                  '1.5',
                  '1.5',
                ],
                [
                  '-3',
                  '-3',
                ],
                [
                  '1e999',
                  'Infinity',
                ],
                [
                  '1e20',
                  '100000000000000000000',
                ],
                [
                  '9007199254740991',
                  '9007199254740991',
                ],
                // A count written as text keeps its quotes, which tell "5" from the number 5.
                [
                  '"5"',
                  '"5"',
                ],
              ] as const
            ) {
              /* oxlint-disable no-await-in-loop -- each read diverts the one global console.warn, so they run in turn */
              /**
               What the read of this file printed, and the file's path.
               */
              const { lines, attemptsPath, } = await warningsReading({
                contents: `{"Mittens": ${written}, "Marmalade": 5}`,
              },);
              /* oxlint-enable no-await-in-loop */
              expect(lines.length,).toBe(1,);
              expect(lines[0],).toContain('Mittens',);
              expect(lines[0],).toContain(attemptsPath,);
              expect(lines[0],).toContain(`counts Mittens as ${shown},`,);
              // The range a count may hold is part of the warning, since it is what the operator edits the file to.
              expect(lines[0],).toContain('no whole number from 0 to 9007199254740990; read as 0, so Mittens sorts first',);
            }
          },
        },),
        it({
          name: 'STAYS QUIET for a well-formed map, so the warnings the "SAYS it starts the counts over" case '
            + 'reads are worth reading',
          fn: async () => {
            expect((await warningsReading({ contents: '{"Mittens": 2}', },)).lines,).toStrictEqual([],);
          },
        },),
        it({
          name: 'STAYS QUIET for the smallest and the largest count a file can hold, which is the largest safe integer '
            + 'less one since the pass adds one to it, so the whole-number rule does not throw away a count that is one',
          fn: async () => {
            expect(
              (await warningsReading({ contents: '{"Mittens": 0, "Marmalade": 9007199254740990}', },)).lines,
            ).toStrictEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: writeAttemptMap.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'WRITES a map the reader reads back whole, leaving no partial file beside it',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'whiskers-attempts-', },);
            /**
             Path the attempts file occupies.
             */
            const attemptsPath = join(
              scratch.path,
              'attempts.json',
            );
            /**
             Counts written and expected back.
             */
            const attempts: AttemptMap = new Map([
              ['Mittens', 2,],
              ['Marmalade', 5,],
            ],);
            await writeAttemptMap({
              attemptsPath,
              attempts,
            },);
            expect(await readAttemptMap(attemptsPath,),).toStrictEqual(attempts,);
            expect(await readdir(scratch.path,),).toStrictEqual(['attempts.json',],);
          },
        },),
      ],
    },),

    describe({
      name: `${attemptsOf.name} and ${countAttempt.name} (ledger B77)`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'COUNTS NO ATTEMPT for an entry whose id is a name every object inherits, where a plain-object '
            + 'map answered with the inherited function or object as the count',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'whiskers-attempts-', },);
            /**
             Counts read from a file that records none.
             */
            const attempts = await readWritten({
              directory: scratch.path,
              contents: '{}',
            },);
            expect([
              'constructor',
              '__proto__',
              'toString',
            ].map(function countOf(id,): number {
              return attemptsOf({
                attempts,
                id,
              },);
            },),).toStrictEqual([0, 0, 0,],);
          },
        },),

        it({
          name: 'COUNTS AND WRITES an attempt for an entry whose id is __proto__, where a plain-object map took the '
            + 'count as setting its prototype and dropped it, so the entry sorted as never tried',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'whiskers-attempts-', },);
            /**
             Path the attempts file occupies.
             */
            const attemptsPath = join(
              scratch.path,
              'attempts.json',
            );
            await writeFile(
              attemptsPath,
              '{}',
              'utf8',
            );
            /**
             Counts this case adds to.
             */
            const attempts = await readAttemptMap(attemptsPath,);
            countAttempt({
              attempts,
              id: '__proto__',
              attemptsPath,
            },);
            expect(attemptsOf({
              attempts,
              id: '__proto__',
            },),).toBe(1,);
            await writeAttemptMap({
              attemptsPath,
              attempts,
            },);
            expect(attemptsOf({
              attempts: await readAttemptMap(attemptsPath,),
              id: '__proto__',
            },),).toBe(1,);
          },
        },),

        it({
          name: 'REFUSES TO COUNT an attempt past the largest safe integer, since the sum is no whole number a stamp or '
            + 'the attempts file can hold, and leaves the count it was given',
          fn: async () => {
            /**
             An entry whose count is the largest a number holds exactly.
             */
            const attempts: AttemptMap = new Map([['Mittens', Number.MAX_SAFE_INTEGER,],],);

            expect(() => {
              countAttempt({
                attempts,
                id: 'Mittens',
                attemptsPath: '/runs/attempts.json',
              },);
            },).toThrow();
            expect(attemptsOf({
              attempts,
              id: 'Mittens',
            },),).toBe(Number.MAX_SAFE_INTEGER,);
          },
        },),

        it({
          name: 'SAYS, where it refuses to count, which entry and which file hold the count, what the count is, that '
            + 'the next launch reads it as zero beside a warning, and what an operator can do, since the refusal ends '
            + 'the pass between entries and reaches the person who ran it as the message alone (ledger B963)',
          fn: async () => {
            /**
             An entry whose count is the largest a number holds exactly.
             */
            const attempts: AttemptMap = new Map([['Mittens', Number.MAX_SAFE_INTEGER,],],);

            /**
             Where the attempts file is.
             */
            const attemptsPath = '/runs/attempts.json';
            for (
              const piece of [
                `Mittens holds an attempt count of 9007199254740991 in ${attemptsPath}`,
                'one more attempt would be no safe integer',
                'this pass ends here',
                `The next launch reads that count as 0 beside a warning naming Mittens and ${attemptsPath}`,
                `To go on, launch again, or set the count of Mittens in ${attemptsPath} to a whole number below ${
                  String(Number.MAX_SAFE_INTEGER,)
                }`,
              ]
            ) {
              expect(() => {
                countAttempt({
                  attempts,
                  id: 'Mittens',
                  attemptsPath,
                },);
              },).toThrow(piece,);
            }
          },
        },),

        it({
          name: 'COUNTS the attempt that reaches the largest safe integer, the last one it can hold',
          fn: async () => {
            /**
             An entry one attempt short of the largest count.
             */
            const attempts: AttemptMap = new Map([['Mittens', Number.MAX_SAFE_INTEGER - 1,],],);
            countAttempt({
              attempts,
              id: 'Mittens',
              attemptsPath: '/runs/attempts.json',
            },);
            expect(attemptsOf({
              attempts,
              id: 'Mittens',
            },),).toBe(Number.MAX_SAFE_INTEGER,);
          },
        },),

        it({
          name: 'READS BACK a count a file records for an entry whose id is __proto__, which the file holds as a '
            + 'field of its own',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'whiskers-attempts-', },);
            expect(attemptsOf({
              attempts: await readWritten({
                directory: scratch.path,
                contents: '{"__proto__": 2}',
              },),
              id: '__proto__',
            },),).toBe(2,);
          },
        },),
      ],
    },),
  ],
},);
