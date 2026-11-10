/**
 How many turns a new run is allowed, sized from what the finished runs of the same kind used.

 The owner, 2026-11-09: "Always Size turn limits to the task based on past data." A cap exists to end a run that stopped getting
 anywhere, so one that fires on a working run costs a resume and leaves its work uncommitted.

 @module
 */

import {
  readdir,
  readFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { errorCodeOf, } from './error-code-of.ts';

/**
 The file a finished run leaves last, which is where a run's turn count is read.
 */
const SUMMARY_FILE = 'summary.json';

/**
 What the summary says a run that ended on its own terms ended with.
 */
const IDLE_END_REASON = 'idle';

/**
 A cap sits this much above the longest finished run, so a run as long as the longest that finished still has room.
 */
const TURN_CAP_MARGIN = 1.25;

/**
 How many finished runs of one kind it takes to size a cap from that kind alone. Fewer says nothing, so every kind is read
 instead.
 */
const RUNS_THAT_SAY_SOMETHING_OF_ONE_KIND = 10;

/**
 The prefix of a run's name that makes it an audit, which runs on another provider and does different work.
 */
const AUDIT_NAME_PREFIX = 'sol-';

/**
 The part of a run's name that makes it a round of fixes to what a review or an audit found.
 */
const FIX_ROUND_NAME_PART = '-findings-fixed';

/**
 The kind of work a run does, which decides which finished runs say how long such work takes.
 */
export type TurnCapKind = 'audit' | 'fix round' | 'other';

/**
 Every finished run had a turn count, and the sample they form was empty: nothing here says how long this work takes.
 */
export class NoFinishedRunToSizeATurnCapError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'NoFinishedRunToSizeATurnCapError';
}

/**
 What sizing a turn cap found.
 */
export type SizedTurnCap = {
  /**
   The cap, a quarter above the longest run in the sample.
   */
  readonly cap: number;

  /**
   How many turns the longest run in the sample used.
   */
  readonly longest: number;

  /**
   How many finished runs form the sample.
   */
  readonly sampleSize: number;

  /**
   The kind this run is, which is what the sample was narrowed to when enough of it had finished.
   */
  readonly kind: TurnCapKind;

  /**
   What the sample was narrowed to, which is every kind when too few of this one had finished.
   */
  readonly sampleKind: TurnCapKind | 'every kind';

  /**
   The line a start prints about what it read.
   */
  readonly line: string;
};

/**
 An entry of the runs folder that is no finished run, as a symbol so no run's name can be mistaken for it.
 */
const NOT_A_FINISHED_RUN: unique symbol = Symbol('the entry is no finished run',);

/**
 One finished run, as sizing a cap reads it.
 */
export type FinishedRun = {
  /**
   The run folder's name, which carries the kind.
   */
  readonly folder: string;

  /**
   How many turns the run used.
   */
  readonly turns: number;
};

/**
 Reads the kind of work a run's folder name names.

 @param name - the run's name, or the name of one of its run folders

 @returns the kind

 @example
 ```ts
 const kind = kindOfRun({ name: 'copy-route-2', });
 ```
 */
export function kindOfRun({
  name,
}: {
  readonly name: string;
},): TurnCapKind {
  if (name.startsWith(AUDIT_NAME_PREFIX,)) {
    return 'audit';
  }
  return name.includes(FIX_ROUND_NAME_PART,) ? 'fix round' : 'other';
}

/**
 Reads the summary one run folder holds, or says why it holds none to read.

 @param runsFolder - the folder the driver writes one run folder per run into

 @param entry - one entry of that folder, known to be a folder

 @returns the summary's text, or {@link NO_SUMMARY_TEXT}

 @throws the error a read gave for a summary that is there and cannot be read

 @example
 ```ts
 const text = await summaryTextIn({ runsFolder: '/runs', entry: 'a-1', });
 ```
 */
async function summaryTextIn({
  runsFolder,
  entry,
}: {
  readonly runsFolder: string;
  readonly entry: string;
},): Promise<string | typeof NO_SUMMARY_TEXT> {
  try {
    return await readFile(
      join(
        runsFolder,
        entry,
        SUMMARY_FILE,
      ),
      { encoding: 'utf8', },
    );
  }
  catch (error) {
    // A run that has not finished has no summary yet, which says nothing about how long work takes.
    if (errorCodeOf(error,) === 'ENOENT') {
      return NO_SUMMARY_TEXT;
    }
    throw error;
  }
}

/**
 No summary to read, as a symbol so no summary's text can be mistaken for it.
 */
const NO_SUMMARY_TEXT: unique symbol = Symbol('the run folder holds no summary to read',);

/**
 Reads one entry of the runs folder as a finished run.

 @param runsFolder - the folder the driver writes one run folder per run into

 @param entry - one entry of that folder

 @returns the run's turns, or nothing when the entry is no finished run

 @example
 ```ts
 const finished = await finishedRunOf({ runsFolder: '/runs', entry: 'copy-route-1', done: false, });
 ```
 */
async function finishedRunOf({
  runsFolder,
  entry,
  done,
}: {
  readonly runsFolder: string;
  readonly entry: string;
  readonly done: boolean;
},): Promise<FinishedRun | typeof NOT_A_FINISHED_RUN> {
  if (!done) {
    return NOT_A_FINISHED_RUN;
  }
  /**
   The summary the run left last, or the reason there is none to read.
   */
  const text = await summaryTextIn({
    runsFolder,
    entry,
  },);
  if ((typeof text) === 'symbol') {
    return NOT_A_FINISHED_RUN;
  }
  /**
   What the summary holds, in the shape this reads.
   */
  const summary: unknown = JSON.parse(text,);
  if (((typeof summary) !== 'object') || (summary === null)) {
    return NOT_A_FINISHED_RUN;
  }
  /**
   The two fields a finished run of the sample has.
   */
  const {
    endReason,
    turns,
  } = summary as {
    readonly endReason?: unknown;
    readonly turns?: unknown;
  };
  if ((endReason !== IDLE_END_REASON) || ((typeof turns) !== 'number')) {
    return NOT_A_FINISHED_RUN;
  }
  return {
    folder: entry,
    turns,
  };
}

/**
 Reads every finished run of a runs folder.

 @param runsFolder - the folder the driver writes one run folder per run into

 @returns every run that ended on its own terms, which is the sample a cap is sized from

 @throws the error a read gave for a runs folder that is not there, which a caller must not take for an empty sample

 @example
 ```ts
 const finished = await finishedRunsIn({ runsFolder: '/runs', });
 ```
 */
export async function finishedRunsIn({
  runsFolder,
}: {
  readonly runsFolder: string;
},): Promise<readonly FinishedRun[]> {
  /**
   The entries of the runs folder.
   */
  const entries = await readdir(
    runsFolder,
    { withFileTypes: true, },
  );
  /**
   Every entry that is a finished run, in the order the folder holds them.
   */
  const finished = (await Promise.all(entries.map(async function finishedIfSo(entry,) {
    return await finishedRunOf({
      runsFolder,
      entry: entry.name,
      done: entry.isDirectory(),
    },);
  },),))
    .filter(function isAFinishedRun(run,): run is FinishedRun {
      return (typeof run) !== 'symbol';
    },);
  return finished;
}

/**
 Sizes a turn cap from the finished runs of the same kind, or from every finished run when too few of that kind have finished.

 @param runsFolder - the folder the driver writes one run folder per run into

 @param runName - the name of the run about to start

 @returns the cap and what it was read from

 @throws NoFinishedRunToSizeATurnCapError when no finished run says how long work takes, so the caller must name a cap itself

 @example
 ```ts
 const sized = await sizedTurnCap({ runsFolder: '/runs', runName: 'copy-route', });
 ```
 */
export async function sizedTurnCap({
  runsFolder,
  runName,
}: {
  readonly runsFolder: string;
  readonly runName: string;
},): Promise<SizedTurnCap> {
  /**
   The kind this run does.
   */
  const kind = kindOfRun({ name: runName, },);
  /**
   Every finished run that says how long work takes.
   */
  const finished = await finishedRunsIn({ runsFolder, },);
  /**
   The finished runs of this kind.
   */
  const sameKind = finished.filter(function isTheSameKind(run,) {
    return kindOfRun({ name: run.folder, },) === kind;
  },);
  /**
   What the cap is read from: this kind when enough of it has finished, and every kind otherwise.
   */
  const sample = (sameKind.length >= RUNS_THAT_SAY_SOMETHING_OF_ONE_KIND) ? sameKind : finished;
  if (sample.length === 0) {
    throw new NoFinishedRunToSizeATurnCapError(
      `no finished run under ${runsFolder} says how many turns this work takes, so the cap must be named with --turn-cap`,
    );
  }
  /**
   How many turns the longest run in the sample used.
   */
  const longest = Math.max(...sample.map(function turnsOf(run,) {
    return run.turns;
  },));
  /**
   The cap itself.
   */
  const cap = Math.ceil(
    longest * TURN_CAP_MARGIN,
  );
  /**
   What the sample was narrowed to.
   */
  const sampleKind = (sample === sameKind) ? kind : 'every kind';
  return {
    cap,
    longest,
    sampleSize: sample.length,
    kind,
    sampleKind,
    line: `turn cap ${String(cap,)}: longest of ${String(sample.length,)} finished runs (${sampleKind}) used ${String(longest,)}`,
  };
}
