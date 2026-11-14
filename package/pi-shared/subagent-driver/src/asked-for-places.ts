/**
 The places asked for: every entry of a batch list whose unit is still active and whose run has not appeared, counted against the
 level the filling is given from the moment the batch is asked for.

 The record it is written from: on 2026-11-14 a filling at a level of 24 passed it by four. Its own lines counted the `pi-*` units
 going, and a review it had "asked for" was waiting in a launcher for a place in its provider's pool, with no unit yet, so the next
 round read its place as free and started another entry into it. A batch's own unit is `launch-<its list>`, and the list sits beside
 the runs, so every asked-for entry is found there without any state of the filling's own: a `launch-*` unit that is active, a list
 file it names, and no `pi-<name>.service` unit for an entry of that list.

 A place is counted against every pool its entry could fill until the run's own unit says which model it runs, because the pool gate
 decides that when it starts the run.

 @module
 */

import { join, } from 'node:path';
import {
  BATCH_UNIT_START,
  LIST_EXTENSION,
} from './batch-run-as-a-unit.ts';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { poolAPlaceAskedForWouldHold, } from './entry-of-a-queue.ts';
import { errorCodeOf, } from './error-code-of.ts';
import { isThere, } from './newest-finished-run-of-a-name.ts';
import type { GoingRun, } from './going-runs.ts';
import {
  readQueueOfRuns,
  type QueueRecord,
} from './queue-of-runs.ts';
import type { RunCommand, } from './run-command.ts';
import { firstWordOf, } from './text-of-an-option.ts';

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 The end of every systemd unit name.
 */
const UNIT_SUFFIX = '.service';

/**
 The units a place asked for is looked for among.
 */
const BATCH_UNIT_PATTERN = `${BATCH_UNIT_START}*${UNIT_SUFFIX}`;

/**
 Reads the batch list a `launch-*` unit was started from, which sits beside the runs under the unit's own name.

 @param unitName - the unit's name, as systemd printed it

 @param runsFolder - folder the driver writes its runs into

 @returns the list's path

 @example
 ```ts
 const file = batchListOfAUnit({ unitName: 'launch-batch-a.json', runsFolder: '/runs' });
 ```
 */
export function batchListOfAUnit({
  unitName,
  runsFolder,
}: {
  readonly unitName: string;
  readonly runsFolder: string;
},): string {
  /**
   The list's own name, which is the unit's name without its start and its unit suffix.
   */
  const name = unitName.slice(
    BATCH_UNIT_START.length,
    -UNIT_SUFFIX.length,
  );
  return join(
    runsFolder,
    `${name}${LIST_EXTENSION}`,
  );
}

/**
 Reads the entries of the batch list one `launch-*` unit was started from, reporting a list that could not be read.

 @param unitName - the unit's name, as systemd printed it

 @param runsFolder - folder the driver writes its runs into

 @param onProblem - told about a list that is there and cannot be read

 @returns the list's entries, and nothing when the unit names no list this runs folder holds

 @example
 ```ts
 const entries = await entriesOfAList({ unitName: 'launch-batch-a.json', runsFolder: '/runs', onProblem: () => {} });
 ```
 */
async function entriesOfAList({
  unitName,
  runsFolder,
  onProblem,
}: {
  readonly unitName: string;
  readonly runsFolder: string;
  readonly onProblem: (problem: string) => void;
},): Promise<readonly QueueRecord[]> {
  /**
   The list the unit's name gives.
   */
  const file = batchListOfAUnit({
    unitName,
    runsFolder,
  },);
  if (!(await isThere(file,))) {
    // A `launch-*` unit that is no batch of this runs folder (`launch-review-...` is a scope of the coordinator's own) names no
    // list at all, which is nothing to report.
    return [];
  }
  try {
    return await readQueueOfRuns({ queueFile: file, },);
  }
  catch (error) {
    // A list that is there and cannot be read is a problem a reader must see, because the places it names are then missing from
    // the level's count.
    onProblem(`${unitName} names a batch list at ${file} that could not be read: ${caughtValueText(error,)}`);
    return [];
  }
}

/**
 Reads the places asked for from the batch units that are still active.

 @param runsFolder - folder the driver writes its runs into

 @param run - how a command is run, which a test replaces

 @param goingNames - the runs going now, by name, whose places are already counted

 @param onProblem - told about a batch unit whose list could not be read

 @returns one entry per place asked for whose run has not appeared

 @example
 ```ts
 const places = await placesAskedFor({ runsFolder: '/runs', run: runCommandWithChildProcess, goingNames: new Set(), onProblem: () => {} });
 ```
 */
export async function placesAskedFor({
  runsFolder,
  run,
  goingNames,
  onProblem,
}: {
  readonly runsFolder: string;
  readonly run: RunCommand;
  readonly goingNames: ReadonlySet<string>;
  readonly onProblem: (problem: string) => void;
},): Promise<readonly GoingRun[]> {
  /**
   What systemd said about the batch units.
   */
  const listed = await run({
    command: 'systemctl',
    words: [
      '--user',
      'list-units',
      '--state=active,activating',
      '--no-legend',
      '--plain',
      BATCH_UNIT_PATTERN,
    ],
  },);
  if (listed.status !== 0) {
    // A machine that cannot list its units leaves the level counted from the units and launchers a run can see, which is what the
    // filling counted before this existed; the line says so rather than reading the silence as no place asked for.
    onProblem(`the units of the batches asked for could not be listed: ${listed.problem
      ?? listed.stderr
        .trim()}`);
    return [];
  }
  /**
   The places found so far.
   */
  const places: GoingRun[] = [];
  /**
   The unit names listed.
   */
  const units = listed.stdout
    .split('\n',)
    .map(function firstWordOfLine(line,) {
      return firstWordOf(line,);
    },)
    .filter(function isABatchUnit(unit,) {
      return unit.startsWith(BATCH_UNIT_START,) && unit.endsWith(UNIT_SUFFIX,);
    },);
  for (const unitName of units) {
    /**
     The entries of the list the unit was started from, or the problem that kept them from being read.
     */
    // oxlint-disable-next-line eslint/no-await-in-loop -- one unit after another, and each list is its own file
    const listedEntries = await entriesOfAList({
      unitName,
      runsFolder,
      onProblem,
    },);
    for (const record of listedEntries) {
      if (goingNames.has(record.name,)) {
        continue;
      }
      places.push({
        name: record.name,
        membership: poolAPlaceAskedForWouldHold({ record, },),
      },);
    }
  }
  return places;
}
