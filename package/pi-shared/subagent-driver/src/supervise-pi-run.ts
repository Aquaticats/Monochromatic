/**
 One whole run: starts Pi, sends the prompt, hands the run to the watch, ends Pi, and writes the summary.

 @module
 */

import { mkdir, } from 'node:fs/promises';
import { join, } from 'node:path';
import {
  type ConnectionToPi,
  startConnectionToPi,
} from './connection-to-pi.ts';
import {
  COMMAND_ANSWER_MILLISECONDS,
  FEED_PREVIEW_CHARACTERS,
  MILLISECONDS_PER_SECOND,
  NESTED_RUN_VARIABLE,
  SHUTDOWN_GRACE_MILLISECONDS,
  SHUTDOWN_STAGES,
  TEARDOWN_ANSWER_MILLISECONDS,
  TEARDOWN_MILLISECONDS,
} from './driver-constants.ts';
import {
  answerMillisecondsBefore,
  millisecondsLeftUntil,
} from './deadline.ts';
import {
  CLEAN_END_REASONS,
  finalEndReason,
} from './end-reasons.ts';
import {
  DRIVER_USAGE,
  DriverOptionsError,
} from './parse-driver-options.ts';
import type {
  DriverOptions,
  EndReason,
  RunProgress,
  RunSummary,
} from './driver-types.ts';
import { preview, } from './previews-of-pi-messages.ts';
import { readOrdinaryFile, } from './read-ordinary-file.ts';
import {
  fieldsOf,
  recordsIn,
} from './read-pi-fields.ts';
import { startRecordOfPiRun, } from './record-of-pi-run.ts';
import {
  type WatchRequest,
  watchUntilEnded,
} from './watch-pi-run.ts';
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
 Builds Pi's command line. `--session` is its own word, so a path holding spaces stays one argument.

 @param options - options of this run

 @returns the arguments for Pi
 */
function piCommandLine(options: DriverOptions,): readonly string[] {
  return [
    '--mode',
    'rpc',
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
 Notes the model and commands Pi starts with, which show at a glance what the run is using.

 @param connection - running Pi

 @param record - run's record
 */
async function noteStartingState({
  connection,
  record,
  deadline,
}: WatchRequest,): Promise<void> {
  /**
   Pi's state, waited for only as long as the run has left.
   */
  const state = fieldsOf((await connection.send({
    command: { type: 'get_state', },
    answerMilliseconds: answerMillisecondsBefore(deadline,),
  },)).data,);
  record.note({
    kind: 'state',
    model: fieldsOf(state.model,)
      .id,
    thinkingLevel: state.thinkingLevel,
  },);
  /**
   Pi's commands.
   */
  const { commands, } = fieldsOf((await connection.send({
    command: { type: 'get_commands', },
    answerMilliseconds: answerMillisecondsBefore(deadline,),
  },)).data,);
  record.note({
    kind: 'commands',
    names: recordsIn(commands,)
      .map(function nameOf(command,) {
        return command.name;
      },),
  },);
}

/**
 Sends the first prompt and notes what Pi made of it.

 @param connection - running Pi

 @param record - run's record

 @param options - options of this run

 @returns why the run is already over, or that it goes on
 */
async function sendFirstPrompt({
  connection,
  record,
  options,
  deadline,
}: WatchRequest,): Promise<RunProgress> {
  /**
   The prompt, which must be an ordinary file: a named pipe would wait for a writer.
   */
  const prompt = await readOrdinaryFile(options.promptFile,);
  /**
   Pi's answer, waited for only as long as the run has left.
   */
  const answer = await connection.send({
    command: {
      type: 'prompt',
      message: prompt,
    },
    answerMilliseconds: answerMillisecondsBefore(deadline,),
  },);
  /**
   What Pi did with the prompt.
   */
  const { disposition, } = fieldsOf(answer.data,);
  record.note({
    kind: 'prompt_sent',
    success: answer.success,
    disposition,
    error: answer.error ?? null,
    chars: prompt.length,
  },);
  if (disposition === 'handled') {
    record.countHandledPrompt();
  }
  if (answer.success !== false) {
    return 'still-running';
  }
  if (connection.hasExited()) {
    return 'pi-exited';
  }
  return (millisecondsLeftUntil(deadline,) === 0) ? 'wall-clock' : 'prompt-rejected';
}

/**
 Takes a run from a started Pi to its end. A failure in the driver itself still ends in a summary and a stopped Pi.

 @param watch - what the run needs

 @returns why the run ended
 */
async function endReasonOfRun(watch: WatchRequest,): Promise<EndReason> {
  try {
    if (millisecondsLeftUntil(watch.deadline,) === 0) {
      return 'wall-clock';
    }
    await noteStartingState(watch,);
    /**
     Where the run stands once the prompt is sent.
     */
    const afterPrompt = await sendFirstPrompt(watch,);
    return afterPrompt === 'still-running' ? await watchUntilEnded(watch,) : afterPrompt;
  }
  catch (error) {
    watch.record
      .noteDriverFailure(String(error,),);
    return 'driver-failed';
  }
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
   Declines a dialog, which would otherwise wait for an answer forever.

   @param id - dialog's identifier
   */
  function declineDialog(id: unknown,): void {
    connection.write({
      type: 'extension_ui_response',
      id,
      cancelled: true,
    },);
  }
  /**
   The running Pi.
   */
  const connection: ConnectionToPi = startConnectionToPi({
    piArguments: piCommandLine(options,),
    workingFolder: options.workingFolder,
    environment: {
      ...environment,
      [NESTED_RUN_VARIABLE]: '1',
    },
    standardErrorFile: join(
      options.runFolder,
      'stderr.log',
    ),
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
  /**
   Why the run ended.
   */
  const endReason = await endReasonOfRun({
    connection,
    record,
    options,
    inbox,
    deadline,
  },);
  /**
   When putting the run away must be over, counted from now: the wall clock bounds the run, and this bounds everything after it.
   */
  const teardownDeadline = Date.now() + TEARDOWN_MILLISECONDS;

  /**
   How long one stage of putting the run away may take: its own bound, or what is left of the teardown's.

   @param stageMilliseconds - the stage's own bound

   @returns the milliseconds this stage may take
   */
  function withinTeardown(stageMilliseconds: number,): number {
    return Math.min(
      stageMilliseconds,
      millisecondsLeftUntil(teardownDeadline,),
    );
  }

  /**
   Pi's own statistics, asked for while it can still answer.
   */
  const stats = await connection.send({
    command: { type: 'get_session_stats', },
    answerMilliseconds: withinTeardown(TEARDOWN_ANSWER_MILLISECONDS,),
  },);
  if (record.isRunning()) {
    await connection.send({
      command: { type: 'abort', },
      answerMilliseconds: withinTeardown(TEARDOWN_ANSWER_MILLISECONDS,),
    },);
  }
  /**
   How Pi's ending went: every stage of it shares what the teardown has left, so one stage cannot spend another's.
   */
  const shutDown = await connection.shutDown({
    graceMilliseconds: Math.max(
      1,
      Math.floor(withinTeardown(SHUTDOWN_GRACE_MILLISECONDS * SHUTDOWN_STAGES,) / SHUTDOWN_STAGES),
    ),
  },);
  if (!shutDown.stopped) {
    record.note({
      kind: 'problem',
      problem: shutDown.reason,
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
   Why the run ended, once those two are known.
   */
  /**
   How Pi ended, which the summary and the reason both read.
   */
  const piExit = connection.exitDetails();
  /**
   Why the run ended, once every late failure is known.
   */
  const finalReason = finalEndReason({
    endReason,
    stopped: shutDown.stopped,
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
    piHasExited: connection.hasExited(),
    sessionStats: stats.data ?? stats.error
      ?? null,
  },);
  await writeSummary({
    runFolder: options.runFolder,
    summary,
  },);
  if (!CLEAN_END_REASONS.has(summary.endReason,)) {
    throw new PiRunEndedEarlyError(
      `The Pi run ended with "${summary.endReason}". The summary is in ${join(
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
