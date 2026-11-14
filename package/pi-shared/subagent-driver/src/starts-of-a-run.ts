/**
 The starts one run makes: a Pi, and another when a start ends before its Pi takes the prompt, within the bounds the constants name.

 A start that fails this way did no work, so making it again costs the run nothing it has done; a start whose Pi took the prompt is
 never made again, because the prompt is where work may begin.

 @module
 */

import { setTimeout as wait, } from 'node:timers/promises';
import {
  type ConnectionToPi,
  startConnectionToPi,
} from './connection-to-pi.ts';
import {
  FEED_PREVIEW_CHARACTERS,
  MILLISECONDS_PER_SECOND,
  NESTED_RUN_VARIABLE,
  START_ATTEMPTS,
  START_SEQUENCE_MILLISECONDS,
} from './driver-constants.ts';
import type {
  DriverOptions,
  EndReason,
  NoFurtherStart,
  PiStartFailure,
  RecordOfPiRun,
} from './driver-types.ts';
import {
  type AnotherStart,
  anotherStartOf,
  type StartThatTookNoPrompt,
  startThatTookNoPromptOf,
  waitForAnotherStart,
} from './another-start-after-a-pi-exited-before-its-prompt.ts';
import {
  bytesInPiStandardError,
  lastOfPiStandardError,
} from './last-of-pi-standard-error.ts';
import { preview, } from './previews-of-pi-messages.ts';
import {
  type RunPutAway,
  putTheRunAway,
} from './put-the-run-away.ts';
import { runCommandWithChildProcess, } from './run-command.ts';
import type { SecretValue, } from './secret-value-masking.ts';
import {
  SECRET_VALUES_DESCRIPTOR_VARIABLE,
  SECRET_VALUES_FD,
  secretValuesHandover,
} from './secret-values-handed-to-pi.ts';
import { endReasonOfRun, } from './take-the-first-prompt.ts';
import { workOutlivingTheTurnIn, } from './work-the-run-left-running.ts';

/**
 What a run's starts left: the last Pi, why the run ended, and every start that failed before it took the prompt.
 */
export type StartsOfARun = {
  /**
   The Pi of the last start, whose ending the summary reads and which the run is put away with unless it was put away already.
   */
  readonly connection: ConnectionToPi;

  /**
   What putting the last start away left, when that start ended before its Pi took the prompt and was put away before the next
   one: a run the last start went on with is put away once the run is over, not here.
   */
  readonly putAway?: RunPutAway;

  /**
   Why the run ended: the ending of the last start that went on, or `pi-exited-before-prompt` when none did.
   */
  readonly endReason: EndReason;

  /**
   How many times the driver started Pi, which every run reports so a later reader counts over a runs folder.
   */
  readonly startsTaken: number;

  /**
   Every start that ended before its Pi took the prompt, in the order they happened.
   */
  readonly startFailures: readonly PiStartFailure[];

  /**
   Why no further start followed, when every start ended before the prompt was taken.
   */
  readonly startStoppedBy?: NoFurtherStart;
};

/**
 Builds Pi's command line. `--session` is its own word, so a path holding spaces stays one argument.

 The masking extension's words come first, so an operator's own arguments are read after them: Pi loads an explicitly named
 extension even with `--no-extensions`, so a run cannot be started with the masking left out by the arguments a caller adds.

 @param options - options of this run

 @param maskingArguments - the words that load the extension masking this run's secret values

 @returns the arguments for Pi
 */
function piCommandLine({
  options,
  maskingArguments,
}: {
  readonly options: DriverOptions;
  readonly maskingArguments: readonly string[];
},): readonly string[] {
  return [
    '--mode',
    'rpc',
    ...maskingArguments,
    '--provider',
    options.provider,
    '--model',
    options.model,
    ...(options.session === undefined ? [] : [
      '--session',
      options.session,
    ]),
    ...options.piArguments,
  ];
}

/**
 Says why the starts stopped once the pause the last one called for has run out: the bound the pause itself crossed, or the one
 the decision named before it.

 @param another - what the decision said before the pause

 @param now - where the driver reads the time after the pause

 @param deadline - when the run must be over

 @returns the bound that stopped the starts

 @example
 ```ts
 const why = stopReasonOf({ another, now: Date.now(), deadline, },);
 ```
 */
function stopReasonOf({
  another,
  now,
  deadline,
}: {
  readonly another: AnotherStart;
  readonly now: number;
  readonly deadline: number;
},): NoFurtherStart {
  if (!another.startAgain) {
    return another.why;
  }
  // The decision above rules this out unless the clock moved under the pause: the bound is read again rather than assumed.
  return (now >= deadline) ? 'wall-clock-too-near' : 'start-sequence-spent';
}

/**
 Starts Pi until one start's Pi takes the prompt, and ends on the starts a run allows, the time one start-up may take, and the
 run's own wall clock.

 @param options - options of this run

 @param record - the run's feed and counts, which every start writes to

 @param inbox - where messages for Pi arrive

 @param deadline - when the run must be over

 @param environment - the environment Pi inherits, without the credentials the caller removed

 @param secretValues - the values of this machine's secret environment, which Pi's extension masks in every tool result

 @param maskingArguments - the words that load the extension masking those values

 @param standardErrorFile - the file Pi's standard error is kept in, for every start

 @returns the last Pi, why the run ended, and every start that failed before it took the prompt

 @example
 ```ts
 const starts = await startsOfARun({ options, record, inbox, deadline, environment, standardErrorFile, },);
 ```
 */
export async function startsOfARun({
  options,
  record,
  inbox,
  deadline,
  environment,
  secretValues,
  maskingArguments,
  standardErrorFile,
}: {
  readonly options: DriverOptions;
  readonly record: RecordOfPiRun;
  readonly inbox: string;
  readonly deadline: number;
  readonly environment: NodeJS.ProcessEnv;
  readonly secretValues: readonly SecretValue[];
  readonly maskingArguments: readonly string[];
  readonly standardErrorFile: string;
},): Promise<StartsOfARun> {
  /**
   When the driver's whole sequence of starts must be over: one start-up bound from now, and never past the run's own wall clock.
   A start that spent that bound ended on a condition that stands, so it is not made again.
   */
  const startSequenceDeadline = Math.min(
    deadline,
    Date.now() + START_SEQUENCE_MILLISECONDS,
  );
  /**
   Every start that ended before its Pi took the prompt.
   */
  const startFailures: PiStartFailure[] = [];
  /**
   The channel of the start now running, set as each start is made. It is a shape rather than a bare variable so the time before
   the first start reads as an absence the way this package says one.
   */
  const running: { connection?: ConnectionToPi } = {};
  /**
   Declines a dialog of the start now running, which would otherwise wait for an answer forever.

   @param id - dialog's identifier
   */
  function declineDialog(id: unknown,): void {
    /**
     The channel of the start now running, when one is connected.
     */
    const channel = running.connection;
    if (channel === undefined) {
      return;
    }
    channel.write({
      type: 'extension_ui_response',
      id,
      cancelled: true,
    },);
  }
  /* oxlint-disable eslint/no-await-in-loop -- one start, then its pause, then the next start: the driver must know how one start
     ended before it decides whether to make another, and the pause is the wait between two tries at the same start. See
     doc/lint/sequential-awaits.md. */
  for (;;) {
    /**
     Which start this is, counting from one: every start before it failed, so each of them is one entry of the failures so far.
     */
    const attempt = startFailures.length + 1;
    /**
     Where this start's own lines begin in the file Pi's standard error is kept in: the first start replaces what the file holds,
     and every later one adds to it, so the file keeps what each start printed while the summary reads the one it is asking about.
     */
    const linesBeginAt = (attempt === 1) ? 0 : await bytesInPiStandardError({ file: standardErrorFile, });
    /**
     The running Pi of this start.
     */
    const connection: ConnectionToPi = startConnectionToPi({
      piArguments: piCommandLine({
        options,
        maskingArguments,
      },),
      workingFolder: options.workingFolder,
      environment: {
        ...environment,
        [NESTED_RUN_VARIABLE]: '1',
        [SECRET_VALUES_DESCRIPTOR_VARIABLE]: String(SECRET_VALUES_FD,),
      },
      secretValuesHandover: secretValuesHandover({ values: secretValues, },),
      standardErrorFile,
      standardErrorAppends: attempt > 1,
      listeners: {
        onEvent: function onEvent(event,) {
          record.observe({
            event,
            declineDialog,
          },);
        },
        onUnreadableLine: function onUnreadableLine(line,) {
          record.note({
            kind: 'non-json',
            line: preview({
              value: line,
              length: FEED_PREVIEW_CHARACTERS.unreadableLine,
            },),
          },);
        },
        onExit: function onExit(exit,) {
          record.note({
            kind: 'exit',
            ...exit,
          },);
        },
        onProblem: function onProblem(problem,) {
          record.note({
            kind: 'problem',
            problem,
          },);
        },
      },
    },);
    running.connection = connection;
    /**
     Why this start left the run.
     */
    const { endReason, } = await endReasonOfRun({
      connection,
      record,
      options,
      inbox,
      deadline,
      secretValuesHandedOver: secretValues.length,
      workLeftRunning: async function workLeftRunning() {
        return await workOutlivingTheTurnIn({
          runFolder: options.runFolder,
          workingFolder: options.workingFolder,
          processFolder: process.env
            .PI_RUNS_PROCESS_FOLDER
            ?? '/proc',
          run: runCommandWithChildProcess,
        },);
      },
    },);
    /**
     Whether this start ended before its Pi took the prompt, which is the only ending the driver makes another start after.
     */
    const ended: StartThatTookNoPrompt = startThatTookNoPromptOf({
      endReason,
      tookPrompt: record.promptTaken(),
      turnsStarted: record.turnCount(),
      exit: connection.exitDetails(),
    },);
    if (!ended.tookNoPrompt) {
      return {
        connection,
        endReason,
        startsTaken: attempt,
        startFailures,
      };
    }
    /**
     What putting this start away left, which the summary reports when it is the run's last start.
     */
    const putAway: RunPutAway = await putTheRunAway({
      connection,
      record,
    },);
    /**
     The last lines Pi printed on this start, which say why it would not go on.
     */
    const tail = await lastOfPiStandardError({
      file: standardErrorFile,
      fromBytes: linesBeginAt,
    },);
    record.note({
      kind: 'start_exited',
      attempt,
      ...ended.exit,
      reason: (tail.reason === undefined) ? null : preview({
        value: tail.reason,
        length: FEED_PREVIEW_CHARACTERS.eventDetail,
      },),
    },);
    /**
     What comes after this start: another one after the pause, or the bound that stops them.
     */
    const another: AnotherStart = anotherStartOf({
      startsTaken: attempt,
      pauseMilliseconds: options.startPauseSeconds * MILLISECONDS_PER_SECOND,
      now: Date.now(),
      wallDeadline: deadline,
      startSequenceDeadline,
    },);
    if (another.startAgain) {
      record.note({
        kind: 'start_again',
        attempt: attempt + 1,
        of: START_ATTEMPTS,
        pauseMilliseconds: another.pauseMilliseconds,
      },);
    }
    /**
     Whether the bounds still left time for a start after the pause, and nothing when no start was owed.
     */
    const inTimeForAnother = (another.startAgain) && await waitForAnotherStart({
      pauseMilliseconds: another.pauseMilliseconds,
      deadline: Math.min(
        deadline,
        startSequenceDeadline,
      ),
      now: Date.now,
      wait,
    },);
    startFailures.push({
      attempt,
      exit: ended.exit,
      pauseMilliseconds: inTimeForAnother ? another.pauseMilliseconds : 0,
      standardErrorLines: tail.lines,
      ...(tail.reason === undefined ? {} : { reason: tail.reason, }),
    },);
    if (!inTimeForAnother) {
      return {
        connection,
        putAway,
        endReason: 'pi-exited-before-prompt',
        startsTaken: attempt,
        startFailures,
        startStoppedBy: stopReasonOf({
          another,
          now: Date.now(),
          deadline,
        },),
      };
    }
  }
  /* oxlint-enable eslint/no-await-in-loop */
}
