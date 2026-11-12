/**
 The turn cap a run gets when its caller names none: a quarter above the longest finished run of its kind, read from the
 summaries in the runs folder.

 The owner, 2026-11-09: "Always Size turn limits to the task based on past data." A cap exists to end a run that stopped getting
 anywhere; one that fires on a working run costs a resume and leaves its work uncommitted.

 @module
 */

import { join, } from 'node:path';
import {
  readdir,
  readFile,
} from 'node:fs/promises';
import { errorCodeOf, } from './error-code-of.ts';
import {
  fieldsOf,
  isRecord,
} from './read-pi-fields.ts';

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 The share above the longest finished run of a kind that a cap allows, so a run as long as the longest that finished still has
 room.
 */
export const TURN_CAP_MARGIN = 1.25;

/**
 How many finished runs of one kind a sample needs before it speaks for that kind alone. Fewer, and every kind is read together,
 because a kind with two finished runs says nothing about the next one.
 */
export const RUNS_OF_ONE_KIND_TO_TRUST = 10;

/**
 No finished run says how long this kind of work takes, as a symbol so that no cap can be mistaken for it.
 */
export const NO_FINISHED_RUN: unique symbol = Symbol('no finished run says how many turns Pi work takes, so the cap needs naming',);

/**
 The start of an audit's run name.
 */
const AUDIT_PREFIX = 'sol-';

/**
 The part of a run name that marks a fix round.
 */
const FIX_ROUND_MARK = '-findings-fixed';

/**
 The end reason of a run that finished its work, which is the only kind whose length says how long the work takes.
 */
const FINISHED_END_REASON = 'idle';

/**
 One finished run's length, and what kind of work it was.
 */
export type FinishedRunTurns = {
  /**
   The kind of work: `audit`, `fix round`, or `other`.
   */
  readonly kind: string;

  /**
   The turns the run used.
   */
  readonly turns: number;
};

/**
 What the finished runs say about this kind of work.
 */
export type TurnCapSizing = {
  /**
   The turns the longest finished run of the sample used, which the cap is measured from.
   */
  readonly longestTurns: number;

  /**
   How many finished runs the sample holds.
   */
  readonly sampleRuns: number;

  /**
   What the sample holds: this run's kind, or every kind together because too few runs of this kind have finished.
   */
  readonly sampleKind: string;
};

/**
 The kind of work a run's name marks, which is what its length is compared against.

 @param runName - the run's name

 @returns the kind of work

 @example
 ```ts
 const kind = kindOfRun({ runName: 'kimi-never-stuck-audit-findings-2', },);
 ```
 */
export function kindOfRun({ runName, }: { readonly runName: string },): string {
  if (runName.startsWith(AUDIT_PREFIX,)) {
    return 'audit';
  }
  return runName.includes(FIX_ROUND_MARK,) ? 'fix round' : 'other';
}

/**
 Reads what the finished runs say about how many turns this run's kind of work takes.

 @param runName - the run's name, which says what kind of work it is

 @param finished - the finished runs of the runs folder

 @returns the longest turns of the sample, or {@link NO_FINISHED_RUN}

 @example
 ```ts
 const sizing = turnCapSizing({ runName: 'copy-route', finished: [{ kind: 'other', turns: 40, },], },);
 ```
 */
export function turnCapSizing({
  runName,
  finished,
}: {
  readonly runName: string;
  readonly finished: readonly FinishedRunTurns[];
}): TurnCapSizing | typeof NO_FINISHED_RUN {
  /**
   This run's kind of work.
   */
  const kind = kindOfRun({ runName, },);
  /**
   The finished runs of this kind.
   */
  const sameKind = finished.filter(function isSameKind(run,) {
    return run.kind === kind;
  },);
  /**
   The runs the cap is measured from, which are this kind's alone only when enough of them finished.
   */
  const sample = (sameKind.length >= RUNS_OF_ONE_KIND_TO_TRUST) ? sameKind : finished;
  if (sample.length === 0) {
    return NO_FINISHED_RUN;
  }
  return {
    longestTurns: Math.max(...sample.map(function turnsOf(run,) {
      return run.turns;
    },)),
    sampleRuns: sample.length,
    sampleKind: (sample === sameKind) ? kind : 'every kind',
  };
}

/**
 Reads a summary's text, or nothing when the folder holds no summary.

 @param summaryFile - the file the driver writes whole and last

 @returns the text, or empty text for a run that has not finished

 @example
 ```ts
 const shown = await summaryTextOrNothingIn('/runs/a-1/summary.json',);
 ```
 */
async function summaryTextOrNothingIn(summaryFile: string,): Promise<string> {
  try {
    return await readFile(
      summaryFile,
      'utf8',
    );
  }
  catch (error) {
    if (errorCodeOf(error,) === MISSING_PATH_CODE) {
      return '';
    }
    throw error;
  }
}

/**
 Reads the turns of every finished run in a runs folder.

 @param runsFolder - folder the driver writes one run folder per run into

 @returns the finished runs' turns and kinds, in the order the folder lists them

 @throws Error when a summary is there but cannot be read as one, because a runs folder that says nothing about a finished run
 is a defect worth seeing

 @example
 ```ts
 const finished = await finishedTurnsIn({ runsFolder: '/runs', },);
 ```
 */
export async function finishedTurnsIn({
  runsFolder,
}: {
  readonly runsFolder: string;
},): Promise<readonly FinishedRunTurns[]> {
  /**
   The runs folder's entries.
   */
  const entries = await readdir(
    runsFolder,
    { withFileTypes: true, },
  );
  /**
   The finished runs found so far.
   */
  const finished: FinishedRunTurns[] = [];
  for (const entry of entries) {
    if (!entry.isDirectory()) {
      continue;
    }
    /**
     The summary's text, or nothing when this folder holds none.
     */
    // oxlint-disable-next-line eslint/no-await-in-loop -- one run folder after another, and each summary is small; see doc/lint/sequential-awaits.md
    const shown = await summaryTextOrNothingIn(join(
      runsFolder,
      entry.name,
      'summary.json',
    ),);
    if (shown === '') {
      continue;
    }
    /**
     What the run's summary says.
     */
    const summary: unknown = JSON.parse(shown,);
    if (!isRecord(summary,)) {
      throw new Error(`${entry.name} holds a summary that is no JSON object, so the turns it used cannot be read.`,);
    }
    /**
     The summary's fields, as this reads them.
     */
    const {
      endReason,
      turns,
    } = fieldsOf(summary,);
    if ((endReason !== FINISHED_END_REASON) || ((typeof turns) !== 'number')) {
      continue;
    }
    finished.push({
      kind: kindOfRun({ runName: entry.name, },),
      turns,
    },);
  }
  return finished;
}
