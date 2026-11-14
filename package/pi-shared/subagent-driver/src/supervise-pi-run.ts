/**
 One whole run: starts Pi, sends the prompt, hands the run to the watch, ends Pi, and writes the summary. The starts themselves,
 including another start after a Pi that left before it took the prompt, are in `starts-of-a-run.ts`.

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
import {
  type RunPutAway,
  putTheRunAway,
} from './put-the-run-away.ts';
import { startRecordOfPiRun, } from './record-of-pi-run.ts';
import { startsOfARun, } from './starts-of-a-run.ts';
import { endingLineOf, } from './the-line-a-driver-ends-with.ts';
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
 Runs Pi once and writes everything the caller reads afterwards. `summary.json` appears last, so its presence means the driver
 has finished putting the run away. `piExit` is there only when Pi's exit was seen, and `piHasExited` says whether the driver
 could confirm Pi had gone.

 A start whose Pi exits before it takes the prompt is started again, within the number of starts and the pause the constants name,
 because nothing of the run's work can have happened. When they are used up the run ends as `pi-exited-before-prompt`, naming what
 Pi printed, and `piStartFailures` holds every start that failed so a reader counts them.

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
  // The git a session would run is asked by the launcher that builds the session's environment, and only there: the owner's
  // decision on issue 1768 is "Launcher preflight only", taken over also closing a hole in the shell-command analyzer's PATH
  // prefixes (doc/handover/cli-git-rust-implementation.md, the answers of 2026-11-11T16:35-05:00). A driver started by hand
  // therefore relies on the `PATH` it was given, and nothing in this package checks it.
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
   The run's feed and counts, which every start of it writes to, so the run's record covers all of them.
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
   Where Pi's standard error is kept for every start of this run: a start that will not go on says why there and nowhere else, and
   one file holds what each start printed.
   */
  const standardErrorFile = join(
    options.runFolder,
    'stderr.log',
  );
  /**
   The run's starts: Pi until one of them takes the prompt, or until the starts or their time run out.
   */
  const starts = await startsOfARun({
    options,
    record,
    inbox,
    deadline,
    environment: givenToPi.environment,
    standardErrorFile,
  },);
  /**
   The Pi of the run's last start, whose ending the summary reads.
   */
  const { connection: lastConnection, } = starts;
  /**
   What putting the last start away left: Pi's own statistics, and whether its leaving could be confirmed. A start that failed
   before its Pi took the prompt is already put away, so its own answer stands.
   */
  const putAway: RunPutAway = starts.putAway ?? await putTheRunAway({
    connection: lastConnection,
    record,
  },);
  /**
   Every start that ended before its Pi took the prompt, and why none followed the last of them.
   */
  const {
    startFailures,
    startStoppedBy,
  } = starts;
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
   How Pi ended, which the summary and the reason both read.
   */
  const piExit = lastConnection.exitDetails();
  /**
   Why the run ended, once every late failure is known.
   */
  const finalReason = finalEndReason({
    endReason: starts.endReason,
    stopped: putAway.stopped,
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
    piHasExited: lastConnection.hasExited(),
    sessionStats: putAway.sessionStats,
    piStartAttempts: starts.startsTaken,
    ...(startFailures.length === 0 ? {} : { piStartFailures: startFailures, }),
    ...(startStoppedBy === undefined ? {} : { piStartStoppedBy: startStoppedBy, }),
  },);
  await writeSummary({
    runFolder: options.runFolder,
    summary,
  },);
  if (!CLEAN_END_REASONS.has(summary.endReason,)) {
    throw new PiRunEndedEarlyError(endingLineOf({
      summary,
      runFolder: options.runFolder,
    },),);
  }
  return summary;
}
