/**
 One whole run: starts Pi, sends the prompt, hands the run to the watch, ends Pi, and writes the summary.

 @module
 */

import { mkdir, } from 'node:fs/promises';
import { join, } from 'node:path';
import {
  MILLISECONDS_PER_SECOND,
  NESTED_RUN_VARIABLE,
} from './driver-constants.ts';
import {
  CLEAN_END_REASONS,
  finalEndReason,
} from './end-reasons.ts';
import {
  type EnvironmentWithoutCredentials,
  stripCredentialsFromEnvironment,
} from './environment-without-credentials.ts';
import {
  DRIVER_USAGE,
  DriverOptionsError,
} from './parse-driver-options.ts';
import type {
  DriverOptions,
  RunSummary,
} from './driver-types.ts';
import { startRecordOfPiRun, } from './record-of-pi-run.ts';
import { watchEveryPiOfARun, } from './watch-every-pi-of-a-run.ts';
import { writeSummary, } from './write-summary.ts';

/**
 A driver started by a Pi that a driver runs.
 */
export class NestedDriverError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'NestedDriverError';
}

/**
 A run that did not end on its own terms. The summary is already written.
 */
export class PiRunEndedEarlyError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'PiRunEndedEarlyError';
}

/**
 Says how many attempts a run made at the model error it ended on, so the reason the driver prints names them.

 @param summary - what the caller reads

 @returns the attempts, in words, or nothing when the run ended on no model error
 */
function attemptsOf(summary: RunSummary,): string {
  if (summary.modelErrorAttempts === undefined) {
    return '';
  }
  /**
   How many attempts the run made, as text.
   */
  const attempts = String(summary.modelErrorAttempts,);
  return (summary.modelErrorAttempts === 1)
    ? ' after 1 attempt'
    : ` after ${attempts} attempts`;
}

/**
 Says what Pi reported of a compaction, when one kept the run's prompt waiting, so the message names why Pi took nothing rather
 than leaving a reader to guess.

 @param summary - what the caller reads

 @returns the note, in words, or nothing when no compaction kept the prompt waiting
 */
function compactionNoteOf(summary: RunSummary,): string {
  return (summary.compactionProblem === undefined)
    ? ''
    : `, because Pi could not put its session in order: ${summary.compactionProblem}`;
}

/**
 Runs Pi once and writes everything the caller reads afterwards. `summary.json` appears last, so its presence means the driver
 has finished putting the run away. `piExit` is there only when Pi's exit was seen, and `piHasExited` says whether the driver
 could confirm Pi had gone.

 @param options - options of this run

 @param environment - environment Pi inherits

 @returns the summary, which is also in `summary.json`

 @throws NestedDriverError when started by a Pi that a driver runs

 @throws PiRunEndedEarlyError when the run did not end on its own terms, after the summary is written

 @example
 ```ts
 const summary = await supervisePiRun({ options, environment: process.env, },);
 ```
 */
export async function supervisePiRun(
  {
    options,
    environment,
  }: {
    readonly options: DriverOptions;
    readonly environment: NodeJS.ProcessEnv
  },
): Promise<RunSummary> {
  if (environment[NESTED_RUN_VARIABLE] !== undefined) {
    throw new NestedDriverError(
      'This driver was started by a Pi that a driver is already running, and it stops here so agents cannot start agents without end. Do this work in the current session, or finish and let the lead start another run.',
    );
  }
  /**
   The wall limit in milliseconds, which is what every deadline and timer is computed from.
   */
  const wallMilliseconds = options.wallSeconds * MILLISECONDS_PER_SECOND;
  if ((!Number.isFinite(wallMilliseconds,)) || (!Number.isFinite(Date.now() + wallMilliseconds,))) {
    throw new DriverOptionsError(`--wall-seconds must be a number of seconds whose milliseconds stay finite; it was "${String(options.wallSeconds,)}".\nUsage: ${DRIVER_USAGE}`,);
  }
  /**
   When the run must be over, counted from before Pi starts so that Pi's start-up is inside the limit.
   */
  const deadline = Date.now() + wallMilliseconds;
  /**
   Where messages for Pi arrive.
   */
  const inbox = join(
    options.runFolder,
    'inbox',
  );
  await mkdir(
    inbox,
    { recursive: true, },
  );
  /**
   The run's feed and counts.
   */
  const record = startRecordOfPiRun({ feedFile: join(
    options.runFolder,
    'feed.jsonl',
  ), },);
  /**
   What Pi starts with: the driver's own environment with every credential-named variable removed, except the allow list in
   `environment-without-credentials.ts`, which names what stays and why. This bounds what an inherited environment holds, not
   what a command can reach: the credential stores stay where they are.
   */
  const givenToPi: EnvironmentWithoutCredentials = stripCredentialsFromEnvironment({ environment, },);
  record.note({
    kind: 'environment',
    removedNames: givenToPi.removedNames,
  },);
  /**
   The last Pi of the run, why the run ended, and what putting that Pi away left: its own statistics, and whether its leaving
   could be confirmed.
   */
  const watched = await watchEveryPiOfARun({
    options,
    record,
    inbox,
    deadline,
    environment: givenToPi.environment,
    standardErrorFile: join(
      options.runFolder,
      'stderr.log',
    ),
  },);
  /**
   Why the run ended.
   */
  const { endReason, } = watched;
  await record.close();
  /**
   The error Pi settled on, when nothing after it recovered.
   */
  const modelError = record.unrecoveredModelError();
  /**
   Why the feed is incomplete, when writing it failed.
   */
  const feedProblem = record.feedProblem();
  /**
   Why the run ended, once those two are known.
   */
  /**
   How Pi ended, which the summary and the reason both read.
   */
  const piExit = watched.connection
    .exitDetails();
  /**
   Why the run ended, once every late failure is known.
   */
  const finalReason = finalEndReason({
    endReason,
    stopped: watched.putAway
      .stopped,
    modelError,
    feedProblem,
    piExit,
  },);
  /**
   What the caller reads.
   */
  const summary = record.summarize({
    endReason: finalReason,
    piExit,
    piHasExited: watched.connection
      .hasExited(),
    sessionStats: watched.putAway
      .sessionStats,
  },);
  await writeSummary({
    runFolder: options.runFolder,
    summary,
  },);
  if (!CLEAN_END_REASONS.has(summary.endReason,)) {
    throw new PiRunEndedEarlyError(
      `The Pi run ended with "${summary.endReason}"${attemptsOf(summary,)}${compactionNoteOf(summary,)}. The summary is in ${join(
        options.runFolder,
        'summary.json',
      )} and Pi's standard error in ${
        join(
          options.runFolder,
          'stderr.log',
        )
      }.`,
    );
  }
  return summary;
}
