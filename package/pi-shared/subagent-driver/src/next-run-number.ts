/**
 The number of a run's folder. A run that reuses one writes over the summary of the run before it, so the folder is always a
 number nothing in the runs folder holds.

 @module
 */

import { isDigitsOnly, } from './text-of-an-option.ts';

/**
 The number a run's folder gets when no folder of that name is there yet.
 */
const FIRST_RUN_NUMBER = 1;

/**
 Reads the number a run's folder carries, when its name says it carries one.

 @param entry - one entry of the runs folder

 @param runName - the run's name

 @returns the number, or nothing when the entry is not a run folder of that name

 @example
 ```ts
 const number = runNumberIn({ entry: 'copy-route-4', runName: 'copy-route', },);
 ```
 */
function runNumberIn({
  entry,
  runName,
}: {
  readonly entry: string;
  readonly runName: string;
}): { readonly number?: number } {
  /**
   The separator between a run's name and its number.
   */
  const prefix = `${runName}-`;
  if (!entry.startsWith(prefix,)) {
    return {};
  }
  /**
   The text after the run's name.
   */
  const written = entry.slice(prefix.length,);
  return isDigitsOnly(written,) ? { number: Number(written,), } : {};
}

/**
 Reads the number the next folder of this run's name takes: one after the highest number any folder of that name holds, so a
 restart never writes over the run before it. The scratch launcher defaulted this to 1, and 16 reports of 2026-11-11 were written
 over before they were recovered from their sessions.

 @param runName - the run's name

 @param entries - the entries of the runs folder, as the folder lists them

 @returns the number the run's folder takes

 @example
 ```ts
 const number = nextRunNumber({ runName: 'copy-route', entries: ['copy-route-1', 'copy-route-4',], },);
 ```
 */
export function nextRunNumber({
  runName,
  entries,
}: {
  readonly runName: string;
  readonly entries: readonly string[];
}): number {
  /**
   The highest number any folder of this name holds, which is none when no folder does.
   */
  const highest = entries.reduce(
    function higher(
      number,
      entry,
    ) {
    /**
     The number this entry carries, when it carries one.
     */
      const carried = runNumberIn({
        entry,
        runName,
      },);
      return (carried.number === undefined) ? number : Math.max(
        number,
        carried.number,
      );
    },
    FIRST_RUN_NUMBER - 1,
  );
  return highest + FIRST_RUN_NUMBER;
}
