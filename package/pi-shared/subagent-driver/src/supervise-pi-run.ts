/**
 One whole run: starts Pi, sends the prompt, hands the run to the watch, ends Pi, and writes the summary. The starts themselves,
 including another start after a Pi that left before it took the prompt, are in `starts-of-a-run.ts`.

 @module
 */

import { mkdir, } from 'node:fs/promises';
import { homedir, } from 'node:os';
import { join, } from 'node:path';
import {
  MILLISECONDS_PER_SECOND,
  NESTED_RUN_VARIABLE,
  NO_RUN_FAILURE,
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
  readSecretValuesOfThisMachine,
  type SecretValuesOfThisMachine,
} from './secret-values-of-this-machine.ts';
import {
  createSecretValueMasking,
  MASKING_NOTHING,
  type SecretValue,
  type TextMasking,
} from './secret-value-masking.ts';
import { maskingExtensionArguments, } from './secret-values-handed-to-pi.ts';
import {
  DRIVER_USAGE,
  DriverOptionsError,
} from './parse-driver-options.ts';
import type {
  DriverOptions,
  RecordOfPiRun,
  RunSummary,
} from './driver-types.ts';
import {
  type RunPutAway,
  putTheRunAway,
} from './put-the-run-away.ts';
import { startRecordOfPiRun, } from './record-of-pi-run.ts';
import { startsOfARun, } from './starts-of-a-run.ts';
import { endingLineOf, } from './the-line-a-driver-ends-with.ts';
import {
  keepsNoSession,
  refusedFreeFormWords,
} from './free-form-arguments-of-pi.ts';
import { SecretEnvironmentUnreadableError, } from './secret-environment-unreadable-error.ts';
import {
  reportedSessionFile,
  sessionFileToMaskAtRest,
} from './a-session-file-this-run-may-rewrite.ts';
import {
  maskASessionIntoACopy,
  maskTheSessionAtRest,
} from './session-file-at-rest.ts';
import { writeSummary, } from './write-summary.ts';

/**

 Reads the values of this machine's secret environment, closing the run's feed before saying why there is no run when the store
 cannot be read.

 @param record - the run's feed and counts, which is taken away by this call when the values cannot be read

 @param read - the read itself

 @returns the values

 @throws SecretEnvironmentUnreadableError when the checkout's store is there and cannot be read
 */
async function secretValuesOrNoRun({
  record,
  read,
}: {
  readonly record: RecordOfPiRun;
  readonly read: () => Promise<SecretValuesOfThisMachine>;
},): Promise<SecretValuesOfThisMachine> {
  try {
    return await read();
  }
  catch (error) {
    await record.close();
    throw error;
  }
}

/**
 Says why a session file cannot be rewritten now, naming the file a reader has to look at.

 @param path - the session file Pi may still be writing

 @returns the reason, for the summary and the feed
 */
function mayStillBeWritten({ path, }: { readonly path: string },): string {
  return `Pi's leaving could not be confirmed, so the session may still be written under ${path}`;
}

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
  /**
   A free-form word that resumes a session this package cannot mask before Pi reads it stops the run here, with its reason.
   */
  const refused = refusedFreeFormWords({ words: options.piArguments, },);
  if (refused.length > 0) {
    throw new DriverOptionsError(
      `this run's free-form Pi arguments cannot start it:\n${
        refused.map(function lineOf(refusal,) {
          return `  ${refusal.word}: ${refusal.reason}`;
        },)
          .join('\n',)
      }\nUsage: ${DRIVER_USAGE}`,
    );
  }
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
  /**
   The folder holding one entry per process: the record takes the start the kernel gave each process a tool answer names from it, and
   the scan that asks what the run left running reads the same entries again. A test points this at its own entries, so no test reads
   this machine's process table.
   */
  const processFolder = environment.PI_RUNS_PROCESS_FOLDER ?? '/proc';
  await mkdir(
    inbox,
    { recursive: true, },
  );
  /**
   The masking this run's own files are written through, which is the pass-through until the values are read a few lines below: the
   refusal of a store that cannot be read is noted while nothing is masked yet, and every line after that goes through the values.
   */
  const writeMasking: { masking: TextMasking } = { masking: MASKING_NOTHING, };
  /**
   The run's feed and counts, which every start of it writes to, so the run's record covers all of them.
   */
  const record = startRecordOfPiRun({
    feedFile: join(
      options.runFolder,
      'feed.jsonl',
    ),
    processFolder,
    mask: {
      /**
       The longest form of any armed value, which is what a writer of Pi's standard error holds back between two writes.
       
       @returns the length the masking has at this moment
       */
      get longestFormLength(): number {
        return writeMasking.masking
          .longestFormLength;
      },
      maskText: function maskTextOfAFeedLine(text,) {
        return writeMasking.masking
          .maskText(text,);
      },
      maskData: function maskDataInAFeedLine(value,) {
        return writeMasking.masking
          .maskData(value,);
      },
      heldBackSuffixLength: function heldBackForAFeedLine(text,) {
        return writeMasking.masking
          .heldBackSuffixLength(text,);
      },
    },
  },);
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
   What this machine holds for its secret environment, read once here so every value of it can be masked before it reaches a
   model. Pi's extension gets them over the descriptor this driver opens for it, and nothing of them is written, logged or
   summarised.

   A store that is there and cannot be read stops the run here, before Pi is started: a run whose values this code does not know
   is a run that would show them to a model, and a start that cannot mask is worse than no start.
   */
  const secretValues: SecretValuesOfThisMachine = await secretValuesOrNoRun({
    record,
    read: function readThem() {
      return readSecretValuesOfThisMachine({
        environment,
        home: environment.HOME ?? homedir(),
        checkoutFolder: options.workingFolder,
      },);
    },
  },);
  /**
   Every file this run writes from here on goes through the values: the feed, the summary, Pi's standard error, and the session at
   rest.
   */
  writeMasking.masking = createSecretValueMasking({ values: secretValues.values, },);
  record.note({
    kind: 'secret_values',
    sources: secretValues.sources,
    unread: secretValues.unread,
  },);
  /**
   The values as a list, which is what the start and the check that the masking is in place both count.
   */
  const values: readonly SecretValue[] = secretValues.values;
  /**
   The words that load the extension masking them, which sit beside the driver's own entry.
   */
  const maskingArguments = await maskingExtensionArguments({ folder: import.meta.dirname, },);
  /**
   A session this run resumes is masked before Pi starts, so that no request Pi builds from it, its compaction among them, can carry
   a value an earlier run left in it. Pi's own hooks cover what this run writes, and never what was already there.
   */
  /**
   The session Pi resumes: the run's masked copy of the session it was told to resume, and nothing else.
   */
  let sessionForPi = options.session;
  /**
   The copy this run made of the session it was given, which is the only file of that session this run may rewrite later.
   */
  let copyOfResumedSession: { readonly path: string } | { readonly absent: true } = { absent: true, };
  if (options.session !== undefined) {
    /**
     What masking the resumed session into a copy found.
     */
    const resumed = await maskASessionIntoACopy({
      sessionFile: options.session,
      copyFile: join(
        options.runFolder,
        'resumed-session.jsonl',
      ),
      mask: writeMasking.masking,
    },);
    if ('problem' in resumed) {
      // A value an earlier session holds, which this code cannot take out before Pi reads it: the run does not start.
      throw new SecretEnvironmentUnreadableError({
        source: `the session ${options.session}`,
        cause: resumed.problem,
      },);
    }
    // A session that is not there is one this run's Pi creates, which makes the file this run's own; one that is there is copied and
    // is never written again by this run, so the file the caller named keeps its entries.
    if ('path' in resumed) {
      sessionForPi = resumed.path;
      copyOfResumedSession = { path: resumed.path, };
    }
    record.note({
      kind: 'session_resumed',
      copy: ('path' in resumed) ? resumed.path : null,
      masked: ('path' in resumed) ? 'into-a-copy' : 'no-session-file',
    },);
  }
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
    secretValues: values,
    maskingArguments,
    standardErrorFile,
    processFolder,
    ...(sessionForPi === undefined ? {} : { sessionFile: sessionForPi, }),
    mask: writeMasking.masking,
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
  /**
   The session file the run leaves behind, which Pi reports in its own statistics or its own state, and which is only ever this run's
   own file: the copy it made, or the one its Pi created. The session the run was given to resume belongs to another run and is refused
   by name, so nothing here can rewrite it.
   */
  const sessionFile = sessionFileToMaskAtRest({
    ...(reportedSessionFile({
      sessionStats: putAway.sessionStats,
      ...(starts.sessionFile === undefined ? {} : { state: starts.sessionFile, }),
    })),
    ...(('path' in copyOfResumedSession) ? { copyThisRunMade: copyOfResumedSession.path, } : {}),
    ...(options.session === undefined ? {} : { sessionTheCallerNamed: options.session, }),
  },);
  /**
   What masking that file found: Pi's own hooks should have kept every value out of it, and this closes what no hook reaches.

   It runs before the feed is closed and before the summary is written, because a session that still holds a value is a run that did
   not finish clean, and the reader has to be told where that value is.
   */
  /**
   What masking the session at rest found.
   */
  /**
   Whether Pi has been reaped, which is what makes this run the last writer of its own session file.
   */
  const piIsGone = putAway.stopped && lastConnection.hasExited();
  /**
   What masking the session at rest found.
   */
  const sessionAtRest = ('refused' in sessionFile)
    // The only file named is another run's, which this run leaves as it is: the summary says so rather than rewriting it.
    ? {
      outcome: 'unmasked' as const,
      problem: sessionFile.refused,
    }
    : ('absent' in sessionFile)
      // A run told to keep no session has none to mask, which is an answer; a run that keeps one and cannot say where is not.
      ? ((keepsNoSession({ words: options.piArguments, },))
        ? { outcome: 'absent' as const, }
        : {
          outcome: 'unmasked' as const,
          problem: 'Pi named no session file and the run keeps one, so a value an entry holds was not masked and this summary cannot say where it stands',
        })
      : await maskTheSessionAtRest({
        sessionFile: sessionFile.file,
        mask: writeMasking.masking,
        writer: piIsGone
          ? { gone: true, }
          : {
            mayExist: mayStillBeWritten({ path: sessionFile.file
              .path, },),
          },
      },);
  if (sessionAtRest.outcome === 'unmasked') {
    record.note({
      kind: 'session_at_rest',
      problem: sessionAtRest.problem,
    },);
  }
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
    sessionAtRest: (sessionAtRest.outcome === 'unmasked')
      ? {
        failed: true,
        problem: sessionAtRest.problem,
      }
      : NO_RUN_FAILURE,
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
    ...((sessionAtRest.outcome === 'unmasked') ? { sessionAtRest: sessionAtRest.problem, } : {}),
    ...(startFailures.length === 0 ? {} : { piStartFailures: startFailures, }),
    ...(startStoppedBy === undefined ? {} : { piStartStoppedBy: startStoppedBy, }),
  },);
  await writeSummary({
    runFolder: options.runFolder,
    summary,
    mask: writeMasking.masking,
  },);
  if (!CLEAN_END_REASONS.has(summary.endReason,)) {
    throw new PiRunEndedEarlyError(endingLineOf({
      summary,
      runFolder: options.runFolder,
    },),);
  }
  return summary;
}
