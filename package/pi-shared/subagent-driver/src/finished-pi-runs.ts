/**
 Which finished runs the overdue watch learns from: the run folders a driver left behind, how long each run lasted, and
 whether that length says anything about how long the work takes.

 @module
 */

import {
  readdir,
  readFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { errorCodeOf, } from './error-code-of.ts';
import { isRecord, } from './read-pi-fields.ts';

/**
 The code the operating system gave a missing path.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 A run whose folder holds no summary yet, which is every run still going.

 A symbol, so no summary's text can be mistaken for it.
 */
const NO_SUMMARY: unique symbol = Symbol('attempt folder holds zero summaries, so the job it belongs to is still going',);

/**
 A run whose summary says its length is no evidence, as a symbol so no run can be mistaken for it.
 */
const NOT_EVIDENCE: unique symbol = Symbol('finished run whose length is no evidence of how long work takes',);

/**
 A finished run of the driver.
 */
export type FinishedPiRun = {
  /**
   The folder the run wrote to.
   */
  readonly folder: string;

  /**
   How long the run lasted, as its summary reports it.
   */
  readonly totalMilliseconds: number;

  /**
   Whether that length says how long the work takes. A run the model provider ended, a prompt Pi refused or a compaction Pi could
   not finish, and a driver that failed all stopped for their own reasons, so their length is no estimate; the calls they
   finished before stopping are still calls that finished.
   */
  readonly lengthCountsAsWork: boolean;
};

/**
 End reasons whose duration says nothing about how long the work takes, so a run that ended on one is left out of the length
 sample: the model answered with an error, the prompt was refused, Pi could not compact its session, or the driver itself
 failed.

 A run that hit its wall clock or its turn cap counts all the same: it did the work for as long as it was allowed. Whatever a
 run ended on, the tool calls it finished before stopping are calls that finished, so its feed stays in the call sample.
 */
export const END_REASONS_WITHOUT_A_JOB: readonly string[] = [
  'model-error',
  'prompt-rejected',
  'compaction-failed',
  'driver-failed',
];

/**
 Reads a run's summary, which is absent while the run is still going.

 @param folder - run folder

 @returns the summary's text, or {@link NO_SUMMARY}

 @example
 ```ts
 const summary = await summaryTextIn('/run',);
 ```
 */
async function summaryTextIn(folder: string,): Promise<string | typeof NO_SUMMARY> {
  try {
    return await readFile(
      join(
        folder,
        'summary.json',
      ),
      { encoding: 'utf8', },
    );
  }
  catch (error) {
    if (errorCodeOf(error,) !== MISSING_PATH_CODE) {
      throw error;
    }
    return NO_SUMMARY;
  }
}

/**
 Reads what one run folder says about how long its run took.

 @param folder - run folder

 @returns the finished run, or {@link NO_SUMMARY} when it has not finished, or {@link NOT_EVIDENCE} when its summary says
 nothing usable

 @example
 ```ts
 const run = await finishedRunIn('/runs/a-1',);
 ```
 */
async function finishedRunIn(folder: string,): Promise<FinishedPiRun | typeof NO_SUMMARY | typeof NOT_EVIDENCE> {
  /**
   The run's summary, absent until the run has ended.
   */
  const text = await summaryTextIn(folder,);
  if ((typeof text) === 'symbol') {
    return NO_SUMMARY;
  }
  /**
   The summary as it was written.
   */
  const summary: unknown = JSON.parse(text,);
  if (!isRecord(summary,)) {
    return NOT_EVIDENCE;
  }
  /**
   How long the run lasted, as the summary reports it.
   */
  const {totalMilliseconds} = summary;
  if (((typeof totalMilliseconds) !== 'number') || (!Number.isFinite(totalMilliseconds,))) {
    return NOT_EVIDENCE;
  }
  /**
   Why the run ended, which decides whether its length is evidence.
   */
  const {endReason} = summary;
  return {
    folder,
    totalMilliseconds,
    lengthCountsAsWork: !(((typeof endReason) === 'string') && END_REASONS_WITHOUT_A_JOB.includes(endReason,)),
  };
}

/**
 Reads every finished run in a runs folder.

 @param runsFolder - folder the driver writes one run folder per run into

 @returns the finished runs, in folder order, each saying whether its length is evidence

 @throws the error a read gave, other than a folder without a summary

 @example
 ```ts
 const runs = await finishedPiRunsIn({ runsFolder: '/runs', },);
 ```
 */
export async function finishedPiRunsIn({
  runsFolder,
}: {
  readonly runsFolder: string;
}): Promise<readonly FinishedPiRun[]> {
  /**
   Everything the runs folder holds.
   */
  const entries = await readdir(
    runsFolder,
    { withFileTypes: true, },
  );
  /**
   What each run folder says, with the folders that hold no usable summary left out.
   */
  const finished = await Promise.all(entries
    .filter(function isFolder(entry,) {
      return entry.isDirectory();
    },)
    .map(function ofFolder(entry,) {
      return finishedRunIn(join(
        runsFolder,
        entry.name,
      ),);
    },));
  return finished.filter(function isFinished(run,): run is FinishedPiRun {
    return (typeof run) !== 'symbol';
  },);
}
