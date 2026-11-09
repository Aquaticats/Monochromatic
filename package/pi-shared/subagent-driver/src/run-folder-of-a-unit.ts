/**
 Which run folder a running unit is writing to, found by the name the recipe in `.agents/skills/pi-as-subagent` gives both.

 A unit is `pi-<task>.service` and each attempt of that task writes `<task>-<number>`, so the run going now is the highest number
 of that name. A name the driver records itself would say this more directly; until a run writes its own folder into its summary,
 the two names are all there is to go on.

 @module
 */

import { readdir, } from 'node:fs/promises';
import { join, } from 'node:path';

/**
 The character between a run's name and the number of one attempt.
 */
const ATTEMPT_SEPARATOR = '-';

/**
 No run folder of that name, as a symbol so no folder path can be mistaken for it.
 */
export const NO_RUN_FOLDER: unique symbol = Symbol('runs folder holds zero numbered attempts of this run name',);

/**
 No attempt number in that name, as a symbol so no number can be mistaken for it.
 */
const NOT_AN_ATTEMPT: unique symbol = Symbol('folder name is not an attempt of that run',);

/**
 Reads whether text is a whole number.

 @param text - the text to read

 @returns whether it is one or more digits and nothing else

 @example
 ```ts
 const whole = isWholeNumber('12',);
 ```
 */
function isWholeNumber(text: string,): boolean {
  if (text === '') {
    return false;
  }
  for (const character of text) {
    if ((character < '0') || (character > '9')) {
      return false;
    }
  }
  return true;
}

/**
 Reads whether a folder name is one attempt of a run's name.

 @param entry - a name in the runs folder

 @param runName - the run's name, which is the unit's name without its prefix and suffix

 @returns the attempt's number, or {@link NOT_AN_ATTEMPT}

 @example
 ```ts
 const attempt = attemptNumberIn({ entry: 'copy-route-3', runName: 'copy-route', },);
 ```
 */
function attemptNumberIn({
  entry,
  runName,
}: {
  readonly entry: string;
  readonly runName: string;
},): number | typeof NOT_AN_ATTEMPT {
  /**
   The start every attempt of this run's folder name has.
   */
  const prefix = `${runName}${ATTEMPT_SEPARATOR}`;
  if (!entry.startsWith(prefix,)) {
    return NOT_AN_ATTEMPT;
  }
  /**
   What follows the run's name.
   */
  const attempt = entry.slice(prefix.length,);
  return isWholeNumber(attempt,) ? Number(attempt,) : NOT_AN_ATTEMPT;
}

/**
 Reads the run folder of the attempt going now.

 @param runsFolder - folder the driver writes one run folder per run into

 @param runName - the run's name, which is the unit's name without its prefix and suffix

 @returns the highest-numbered attempt's folder, or {@link NO_RUN_FOLDER}

 @throws the error a read gave, such as a missing runs folder

 @example
 ```ts
 const folder = await latestRunFolderOf({ runsFolder: '/runs', runName: 'copy-route', },);
 ```
 */
export async function latestRunFolderOf({
  runsFolder,
  runName,
}: {
  readonly runsFolder: string;
  readonly runName: string;
},): Promise<string | typeof NO_RUN_FOLDER> {
  /**
   The attempts of this run already in the folder.
   */
  const attempts = (await readdir(runsFolder,))
    .map(function numberOf(entry,) {
      return attemptNumberIn({
        entry,
        runName,
      },);
    },)
    .filter(function isAttempt(attempt,): attempt is number {
      return (typeof attempt) !== 'symbol';
    },);
  if (attempts.length === 0) {
    return NO_RUN_FOLDER;
  }
  return join(
    runsFolder,
    `${runName}${ATTEMPT_SEPARATOR}${String(Math.max(...attempts,),)}`,
  );
}
