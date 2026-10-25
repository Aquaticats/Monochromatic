import { readFile, } from 'node:fs/promises';

import { tagged, } from '@monochromatic-dev/module-logger/ts';

import {
  contextRoot,
  isMissingPathError,
} from '@monochromatic-dev/translation-repair-typography/ts';
import { isJsonRecord, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { writeFileAtomic, } from './atomic-write.ts';

//region Attempt store
// Persisted per-entry attempt counts, used as the last ordering tiebreak so an
// entry that keeps failing deprioritizes instead of blocking the queue. Kept
// beside the driver rather than inside it so the driver stays within its line
// budget.
//
// TOLERANT, AND SAYS SO (ledger A11). A malformed file still costs an ordering
// hint rather than the run, but every reset and every coerced count is logged:
// read silently, a file that never parses makes every run order as if no entry
// had ever failed. The write is atomic, so an interrupted run can no longer
// leave the truncated file that made the reset necessary.
//
// A MAP, NOT A PLAIN OBJECT (ledger B77). The counts are keyed by entry ids,
// the corpus's own directory names, and a plain object answered an id spelled
// `constructor` with the inherited function and took a count set for
// `__proto__` as a new prototype, dropping it. The file keeps the plain JSON
// object it always held.

/**
 Logger root for the attempt store.
 */
const l = contextRoot({ tag: 'translation-repair', },);

/**
 Attempt counts keyed by entry id, for fewest-attempts-first ordering.

 @example
 ```ts
 const attempts: AttemptMap = new Map([['Kitten', 2,],],);
 ```
 */
export type AttemptMap = Map<string, number>;

/**
 Whether a value read from the attempts file can stand as an entry's attempt count.

 A COUNT IS A NON-NEGATIVE WHOLE NUMBER THE PASS CAN ADD ONE TO. The pass adds one to the count it reads and the
 lifecycle stamp of the lines it prints for the attempt refuses anything that is not a whole number from 1 (ledger B960),
 so a hand-edited 1.5, -3, 1e999 (which parses to Infinity) or 1e20 passed through would end the pass at its first
 entry, outside any `try`, and again at every launch since the file keeps it.

 @param value - what the file holds for one entry

 @returns Whether it is a safe integer of at least zero

 @example
 ```ts
 const usable = isAttemptCount(1.5,); // => false
 ```
 */
function isAttemptCount(value: unknown,): value is number {
  if ((typeof value) !== 'number')
    return false;
  return Number.isSafeInteger(value,) && (value >= 0);
}

/**
 Reads the persisted attempt map, tolerating a missing or malformed file so a
 corrupt cache never aborts a run.

 @param attemptsPath - location of the attempts JSON

 @returns Entry-id to attempt-count map, empty when absent or unreadable, each count that is no non-negative whole
 number read as 0 beside a warning naming the entry, the file and the value

 @throws {@link Error} when the file exists and is readable but fails for any
 reason other than absence or malformed JSON

 @example
 ```ts
 const attempts = await readAttemptMap('/runs/attempts.json',);
 ```
 */
export async function readAttemptMap(attemptsPath: string,): Promise<AttemptMap> {
  /**
   Logger pre-tagged with this function's name.
   */
  const rl = tagged({
    tag: readAttemptMap.name,
    l,
  },);
  try {
    /**
     Parsed JSON of unknown shape until guarded.
     */
    const parsed: unknown = JSON.parse(await readFile(
      attemptsPath,
      'utf8',
    ),);
    if (!isJsonRecord(parsed,)) {
      rl.warn('attempts file holds no object; attempt counts start over, so the ordering forgets which entries kept failing',);
      return new Map();
    }

    return new Map(
      Object.entries(parsed,)
        .map(function toCount(
          [
            id,
            value,
          ]: readonly [
            string,
            unknown,
          ],
        ): readonly [
          string,
          number,
        ] {
          if (isAttemptCount(value,))
            return [
              id,
              value,
            ];
          rl.warn(
            `attempts file ${attemptsPath} counts ${id} as ${
              (typeof value) === 'number' ? String(value,) : JSON.stringify(value,)
            }, which is no non-negative whole number; read as 0, so ${id} sorts first`,
          );
          return [
            id,
            0,
          ];
        },),
    );
  }
  catch (error) {
    // Missing (ENOENT) is a first run and says nothing; malformed (SyntaxError)
    // resets to empty and says so; any other read fault is real and surfaces.
    if (isMissingPathError({ error, },))
      return new Map();
    if (error instanceof SyntaxError) {
      rl.warn(`attempts file does not parse (${error.name}); attempt counts start over, so the ordering forgets which entries kept failing`,);
      return new Map();
    }
    throw error;
  }
}

/**
 Attempts counted for one entry.

 @param attempts - counts read from the attempts file and kept through this run

 @param id - entry id as the corpus names it, which the counts are keyed by

 @returns Its count, 0 where the counts hold none

 @example
 ```ts
 const count = attemptsOf({ attempts, id: 'Kitten', },);
 ```
 */
export function attemptsOf(
  {
    attempts,
    id,
  }: {
    readonly attempts: ReadonlyMap<string, number>;
    readonly id: string;
  },
): number {
  return attempts.get(id,) ?? 0;
}

/**
 Counts one more attempt for an entry, before the attempt runs.

 @param attempts - counts this run keeps and persists

 @param id - entry about to be attempted

 @mutates attempts - sets this entry's count to one past what it held

 @example
 ```ts
 countAttempt({ attempts, id: 'Kitten', },);
 ```
 */
export function countAttempt(
  {
    attempts,
    id,
  }: {
    readonly attempts: AttemptMap;
    readonly id: string;
  },
): void {
  attempts.set(
    id,
    attemptsOf({
      attempts,
      id,
    },) + 1,
  );
}

/**
 Persists the attempt map so no reader sees it half-written.

 @param attemptsPath - location of the attempts JSON

 @param attempts - counts to record

 @example
 ```ts
 await writeAttemptMap({ attemptsPath, attempts, },);
 ```
 */
export async function writeAttemptMap(
  {
    attemptsPath,
    attempts,
  }: {
    readonly attemptsPath: string;
    readonly attempts: ReadonlyMap<string, number>;
  },
): Promise<void> {
  await writeFileAtomic({
    path: attemptsPath,
    text: `${JSON.stringify(
      Object.fromEntries(attempts,),
      undefined,
      2,
    )}\n`,
  },);
}

//endregion Attempt store
