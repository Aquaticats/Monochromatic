/**
 The turns a running run has taken and how long its feed has been quiet: what a coordinator reads to tell a run that is working
 from one that started and never took a turn.

 A feed is the driver's own record, so the turn count is the driver's own count and the quiet is the last write to that record.
 Both are read from the runs folder the driver writes into and from systemd's own list of running units, so nothing here asks a
 run anything.

 @module
 */

import {
  readFile,
  stat,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { errorCodeOf, } from './error-code-of.ts';
import { isRecord, } from './read-pi-fields.ts';
import {
  NO_RUN_FOLDER,
  latestRunFolderOf,
} from './run-folder-of-a-unit.ts';
import type { RunningPiUnit, } from './running-pi-units.ts';
import { wholeLinesOf, } from './whole-lines-of-a-feed.ts';

/**
 The kind of feed record the driver writes once per turn, whose count is the run's own count.
 */
const TURN_START_KIND = 'turn_start';

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 What one running run's feed says.
 */
export type FeedOfARunningRun = {
  /**
   How many turns the feed records.
   */
  readonly turns: number;

  /**
   How long ago the feed was last written, in milliseconds.
   */
  readonly quietMilliseconds: number;
};

/**
 Counts the turns a feed records.

 The feed is read while its driver still writes it, so the line it ends with may be one the writer has not finished, which is no
 record yet and is left out.

 @param feedShown - the feed, one JSON record per line

 @returns how many turn starts it holds

 @example
 ```ts
 const turns = turnsInFeed({ feedShown: '{"kind":"turn_start","n":1}\n', },);
 ```
 */
export function turnsInFeed({ feedShown, }: { readonly feedShown: string },): number {
  /**
   The turns counted so far.
   */
  let turns = 0;
  for (const line of wholeLinesOf({ shown: feedShown, },)) {
    if (line === '') {
      continue;
    }
    /**
     One record of the feed, of any shape.
     */
    const record: unknown = JSON.parse(line,);
    if ((isRecord(record,)) && (record.kind === TURN_START_KIND)) {
      turns += 1;
    }
  }
  return turns;
}

/**
 Reads what a run's feed says, or nothing when the run has written none yet.

 @param feedFile - the run's feed

 @param now - where the quiet is read from, as milliseconds since the epoch

 @returns the turns and the quiet, or nothing when no feed is there

 @example
 ```ts
 const said = await feedOfARunningRun({ feedFile: '/runs/a-1/feed.jsonl', now: Date.now, },);
 ```
 */
export async function feedOfARunningRun({
  feedFile,
  now,
}: {
  readonly feedFile: string;
  readonly now: () => number;
},): Promise<{ readonly said?: FeedOfARunningRun }> {
  try {
    return {
      said: {
        turns: turnsInFeed({
          feedShown: await readFile(
            feedFile,
            'utf8',
          ),
        },),
        quietMilliseconds: now() - (await stat(feedFile,)).mtimeMs,
      },
    };
  }
  catch (error) {
    // A run that has written no feed yet is described as one with no feed, never as one whose feed says nothing.
    if (errorCodeOf(error,) === MISSING_PATH_CODE) {
      return {};
    }
    throw error;
  }
}

/**
 Seconds in a minute, so an age is printed in the unit a person reads it in.
 */
const SECONDS_PER_MINUTE = 60;

/**
 Milliseconds in a second.
 */
const MILLISECONDS_PER_SECOND = 1_000;

/**
 Prints an age as minutes and seconds, which is how long a coordinator has been waiting.

 @param milliseconds - the age

 @returns the age, as `4m 12s`

 @example
 ```ts
 const shown = ageTextOf(252_000,);
 ```
 */
export function ageTextOf(milliseconds: number,): string {
  /**
   The whole seconds of the age.
   */
  const seconds = Math.round(milliseconds / MILLISECONDS_PER_SECOND,);
  return `${String(Math.floor(seconds / SECONDS_PER_MINUTE,),)}m ${String(seconds % SECONDS_PER_MINUTE,)}s`;
}

/**
 Builds the line one running run prints.

 @param name - the run's name

 @param unit - the unit that is running it

 @param said - what its feed says, or nothing when it has written none

 @returns the line

 @example
 ```ts
 const line = runningRunLineOf({ name: 'a', unit, said: { turns: 3, quietMilliseconds: 1_000 }, },);
 ```
 */
export function runningRunLineOf({
  name,
  unit,
  said,
}: {
  readonly name: string;
  readonly unit: RunningPiUnit;
  readonly said?: FeedOfARunningRun;
},): string {
  /**
   The unit's own age, which is how long the run has been up whatever its feed says.
   */
  const up = ageTextOf(unit.elapsedMilliseconds,);
  if (said === undefined) {
    return `${name}: up ${up}, no feed written yet; ${unit.unitName}`;
  }
  return `${name}: turns ${String(said.turns,)}, feed written ${ageTextOf(said.quietMilliseconds,)} ago, up ${up}; ${
    unit.unitName}`;
}

/**
 Describes every running run, one line each, oldest unit first.

 @param units - the Pi runs going now, as systemd lists them

 @param runsFolder - the folder the driver writes one run folder per run into

 @param now - where the ages are read from

 @returns one line per run, and the runs whose feed could not be found among them

 @example
 ```ts
 const lines = await runningRunLinesOf({ units, runsFolder: '/runs', now: Date.now, },);
 ```
 */
export async function runningRunLinesOf({
  units,
  runsFolder,
  now,
}: {
  readonly units: readonly RunningPiUnit[];
  readonly runsFolder: string;
  readonly now: () => number;
},): Promise<readonly string[]> {
  /**
   The lines, in the order the units came.
   */
  const lines: string[] = [];
  /* oxlint-disable eslint/no-await-in-loop -- one run after another, each feed read before the next, so a line names the run it
     belongs to. See doc/lint/sequential-awaits.md. */
  for (const unit of units) {
    /**
     The folder the run going now writes into.
     */
    const runFolder = await latestRunFolderOf({
      runsFolder,
      runName: unit.name,
    },);
    if (runFolder === NO_RUN_FOLDER) {
      lines.push(`${unit.name}: up ${ageTextOf(unit.elapsedMilliseconds,)}, no run folder yet; ${unit.unitName}`,);
      continue;
    }
    /**
     What that run's feed says.
     */
    const { said, } = await feedOfARunningRun({
      feedFile: join(
        runFolder,
        'feed.jsonl',
      ),
      now,
    },);
    lines.push(runningRunLineOf({
      name: unit.name,
      unit,
      ...((said === undefined) ? {} : { said, }),
    },),);
  }
  /* oxlint-enable eslint/no-await-in-loop */
  return lines;
}
