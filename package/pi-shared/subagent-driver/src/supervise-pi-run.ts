/**
 One whole run: starts Pi, sends the prompt, hands the run to the watch, ends Pi, and writes the summary.

 @module
 */

import {
  mkdir,
  readFile,
  rename,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import {
  type ConnectionToPi,
  startConnectionToPi,
} from './connection-to-pi.ts';
import {
  FEED_PREVIEW_CHARACTERS,
  MILLISECONDS_PER_SECOND,
  NESTED_RUN_VARIABLE,
} from './driver-constants.ts';
import type {
  DriverOptions,
  EndReason,
  RunProgress,
  RunSummary,
} from './driver-types.ts';
import {
  fieldsOf,
  recordsIn,
} from './read-pi-fields.ts';
import {
  preview,
  startRecordOfPiRun,
} from './record-of-pi-run.ts';
import {
  type WatchRequest,
  watchUntilEnded,
} from './watch-pi-run.ts';

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
 Reasons a run ends on its own terms.
 */
const CLEAN_END_REASONS: ReadonlySet<EndReason> = new Set<EndReason>(['idle', 'stop-file',],);

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
    ...(options.session === undefined ? [] : ['--session', options.session,]),
    ...options.piArguments,
  ];
}

/**
 Writes the summary so that a reader sees all of it or none.

 @param runFolder - run's folder

 @param summary - what to write
 */
async function writeSummary({ runFolder, summary, }: { readonly runFolder: string; readonly summary: RunSummary; },): Promise<void> {
  /**
   Where the summary is written first.
   */
  const unfinished = join(runFolder, '.summary.json.unfinished',);
  await writeFile(unfinished, `${JSON.stringify(summary, null, 2,)}\n`,);
  await rename(unfinished, join(runFolder, 'summary.json',),);
}

/**
 Notes the model and commands Pi starts with, which show at a glance what the run is using.

 @param connection - running Pi

 @param record - run's record
 */
async function noteStartingState({ connection, record, }: WatchRequest,): Promise<void> {
  /**
   Pi's state.
   */
  const state = fieldsOf((await connection.send({ command: { type: 'get_state', }, },)).data,);
  record.note({ kind: 'state', model: fieldsOf(state.model,).id, thinkingLevel: state.thinkingLevel, },);
  /**
   Pi's commands.
   */
  const { commands, } = fieldsOf((await connection.send({ command: { type: 'get_commands', }, },)).data,);
  record.note({
    kind: 'commands',
    names: recordsIn(commands,).map(function nameOf(command,) {
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
async function sendFirstPrompt({ connection, record, options, }: WatchRequest,): Promise<RunProgress> {
  /**
   The prompt.
   */
  const prompt = await readFile(options.promptFile, 'utf8',);
  /**
   Pi's answer.
   */
  const answer = await connection.send({ command: { type: 'prompt', message: prompt, }, },);
  /**
   What Pi did with the prompt.
   */
  const { disposition, } = fieldsOf(answer.data,);
  record.note({ kind: 'prompt_sent', success: answer.success, disposition, error: answer.error ?? null, chars: prompt.length, },);
  if (disposition === 'handled') {
    record.countHandledPrompt();
  }
  if (answer.success !== false) {
    return 'still-running';
  }
  return connection.hasExited() ? 'pi-exited' : 'prompt-rejected';
}

/**
 Takes a run from a started Pi to its end. A failure in the driver itself still ends in a summary and a stopped Pi.

 @param watch - what the run needs

 @returns why the run ended
 */
async function endReasonOfRun(watch: WatchRequest,): Promise<EndReason> {
  try {
    await noteStartingState(watch,);
    /**
     Where the run stands once the prompt is sent.
     */
    const afterPrompt = await sendFirstPrompt(watch,);
    return afterPrompt === 'still-running' ? await watchUntilEnded(watch,) : afterPrompt;
  }
  catch (error) {
    watch.record.noteDriverFailure(String(error,),);
    return 'driver-failed';
  }
}

/**
 Runs Pi once and writes everything the caller reads afterwards. `summary.json` appears last, so its presence means the feed is
 complete and Pi has gone.

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
  { options, environment, }: { readonly options: DriverOptions; readonly environment: NodeJS.ProcessEnv; },
): Promise<RunSummary> {
  if (environment[NESTED_RUN_VARIABLE] !== undefined) {
    throw new NestedDriverError(
      'This driver was started by a Pi that a driver is already running, and it stops here so agents cannot start agents without end. Do this work in the current session, or finish and let the lead start another run.',
    );
  }
  /**
   When the run must be over, counted from before Pi starts so that Pi's start-up is inside the limit.
   */
  const deadline = Date.now() + (options.wallSeconds * MILLISECONDS_PER_SECOND);
  /**
   Where messages for Pi arrive.
   */
  const inbox = join(options.runFolder, 'inbox',);
  await mkdir(inbox, { recursive: true, },);
  /**
   The run's feed and counts.
   */
  const record = startRecordOfPiRun(join(options.runFolder, 'feed.jsonl',),);
  /**
   The running Pi.
   */
  const connection: ConnectionToPi = startConnectionToPi({
    piArguments: piCommandLine(options,),
    workingFolder: options.workingFolder,
    environment: { ...environment, [NESTED_RUN_VARIABLE]: '1', },
    standardErrorFile: join(options.runFolder, 'stderr.log',),
    listeners: {
      onEvent: function onEvent(event,) {
        record.observe({
          event,
          declineDialog: function declineDialog(id,) {
            connection.write({ type: 'extension_ui_response', id, cancelled: true, },);
          },
        },);
      },
      onUnreadableLine: function onUnreadableLine(line,) {
        record.note({ kind: 'non-json', line: preview({ value: line, length: FEED_PREVIEW_CHARACTERS.unreadableLine, },), },);
      },
      onExit: function onExit(exit,) {
        record.note({ kind: 'exit', ...exit, },);
      },
      onProblem: function onProblem(problem,) {
        record.note({ kind: 'problem', problem, },);
      },
    },
  },);
  /**
   Why the run ended.
   */
  const endReason = await endReasonOfRun({ connection, record, options, inbox, deadline, },);
  /**
   Pi's own statistics, asked for while it can still answer.
   */
  const stats = await connection.send({ command: { type: 'get_session_stats', }, },);
  if (record.isRunning()) {
    await connection.send({ command: { type: 'abort', }, },);
  }
  await connection.shutDown();
  await record.close();
  /**
   What the caller reads.
   */
  const summary = record.summarize({
    endReason,
    piExit: connection.exitDetails(),
    piHasExited: connection.hasExited(),
    sessionStats: stats.data ?? stats.error ?? null,
  },);
  await writeSummary({ runFolder: options.runFolder, summary, },);
  if (!CLEAN_END_REASONS.has(endReason,)) {
    throw new PiRunEndedEarlyError(
      `The Pi run ended with "${endReason}". The summary is in ${join(options.runFolder, 'summary.json',)} and Pi's standard error in ${
        join(options.runFolder, 'stderr.log',)
      }.`,
    );
  }
  return summary;
}
