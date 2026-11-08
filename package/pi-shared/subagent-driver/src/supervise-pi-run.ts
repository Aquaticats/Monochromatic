/**
 One whole run: starts Pi, sends the prompt, watches the limits and the inbox until the run ends, and writes the summary.

 @module
 */

import {
  access,
  mkdir,
  readdir,
  readFile,
  rename,
  unlink,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { setTimeout as wait, } from 'node:timers/promises';
import { ConnectionToPi, } from './connection-to-pi.ts';
import {
  FEED_PREVIEW_CHARACTERS,
  MILLISECONDS_PER_SECOND,
  NESTED_RUN_VARIABLE,
  POLL_MILLISECONDS,
} from './driver-constants.ts';
import type {
  DriverOptions,
  EndReason,
  PiRecord,
  RunSummary,
} from './driver-types.ts';
import {
  preview,
  RecordOfPiRun,
} from './record-of-pi-run.ts';

/**
 A driver started by a Pi that a driver runs.
 */
export class NestedDriverError extends Error {
  override readonly name = 'NestedDriverError';
}

/**
 A run that did not end on its own terms. The summary is already written.
 */
export class PiRunEndedEarlyError extends Error {
  override readonly name = 'PiRunEndedEarlyError';
}

/**
 Reasons a run ends on its own terms.
 */
const CLEAN_END_REASONS: ReadonlySet<EndReason> = new Set<EndReason>(['idle', 'stop-file',],);

/**
 Tells whether a path exists.

 @param path - path to look at

 @returns whether it can be reached
 */
async function exists(path: string,): Promise<boolean> {
  try {
    await access(path,);
    return true;
  }
  catch (error) {
    return (error as NodeJS.ErrnoException).code !== 'ENOENT';
  }
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
    ...(options.session === null ? [] : ['--session', options.session,]),
    ...options.piArguments,
  ];
}

/**
 Writes the summary so that a reader sees all of it or none.

 @param request - run folder and summary
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
 What the loop that watches a run needs.
 */
type WatchRequest = {
  readonly connection: ConnectionToPi;
  readonly record: RecordOfPiRun;
  readonly options: DriverOptions;
  readonly inbox: string;
  readonly stopFile: string;
  readonly deadline: number;
};

/**
 Delivers one inbox message. The file goes only once Pi has it; a refused or unanswered one stays readable under a dot name,
 which this loop skips, so it is neither lost nor sent again.

 @param request - watch request and file name
 */
async function deliverMessage({ watch, name, }: { readonly watch: WatchRequest; readonly name: string; },): Promise<void> {
  /**
   The message file.
   */
  const path = join(watch.inbox, name,);
  /**
   The message.
   */
  const text = await readFile(path, 'utf8',);
  /**
   How the message reaches Pi: a steer while it works, a new prompt while it rests.
   */
  const sentAs = watch.record.running ? 'steer' : 'prompt';
  /**
   Pi's answer.
   */
  const answer = await watch.connection.send({ command: { type: sentAs, message: text, }, },);
  await (answer.success === true ? unlink(path,) : rename(path, join(watch.inbox, `.failed-${name}`,),));
  watch.record.note({
    kind: 'inbox',
    file: name,
    sentAs,
    success: answer.success,
    error: answer.error ?? null,
    preview: preview({ value: text, length: FEED_PREVIEW_CHARACTERS.inboxMessage, },),
  },);
}

/**
 Looks once at every reason a run could end.

 @param watch - watch request

 @returns why the run is over, or null while it goes on
 */
async function reasonToEnd(watch: WatchRequest,): Promise<EndReason | null> {
  if (watch.connection.exit !== null) {
    return 'pi-exited';
  }
  if (Date.now() >= watch.deadline) {
    return 'wall-clock';
  }
  if (await exists(watch.stopFile,)) {
    return 'stop-file';
  }
  if (watch.record.turns >= watch.options.turnCap) {
    return 'turn-cap';
  }
  return null;
}

/**
 Delivers what is in the inbox, then decides whether the run has been idle long enough, until something ends it.

 @param watch - watch request

 @returns why the run ended
 */
async function watchUntilEnded(watch: WatchRequest,): Promise<EndReason> {
  for (;;) {
    /**
     Why the run is over, if it is.
     */
    const reason = await reasonToEnd(watch,);
    if (reason !== null) {
      return reason;
    }
    /**
     Messages ready to read; a name starting with a dot is still being written, or was refused.
     */
    const messages = (await readdir(watch.inbox,)).filter((name,) => !name.startsWith('.',)).toSorted();
    for (const name of messages) {
      await deliverMessage({ watch, name, },);
    }
    if (
      !watch.record.running && (watch.record.settled >= 1) && (messages.length === 0)
      && (watch.record.silentMilliseconds > (watch.options.idleSeconds * MILLISECONDS_PER_SECOND))
    ) {
      return 'idle';
    }
    await wait(POLL_MILLISECONDS,);
  }
}

/**
 Reads a field that holds an object.

 @param value - field's value

 @returns the object, or an empty one when field holds anything else
 */
function fieldsOf(value: unknown,): PiRecord {
  return ((typeof value === 'object') && (value !== null)) ? value as PiRecord : {};
}

/**
 Sends the first prompt and notes what Pi made of it.

 @param request - connection, record and options

 @returns why the run is already over, or null when it goes on
 */
async function sendFirstPrompt(
  { connection, record, options, }: { readonly connection: ConnectionToPi; readonly record: RecordOfPiRun; readonly options: DriverOptions; },
): Promise<EndReason | null> {
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
  const disposition = fieldsOf(answer.data,).disposition;
  record.note({ kind: 'prompt_sent', success: answer.success, disposition, error: answer.error ?? null, chars: prompt.length, },);
  if (disposition === 'handled') {
    record.countHandledPrompt();
  }
  return answer.success === false ? 'prompt-rejected' : null;
}

/**
 Notes the model and commands Pi starts with, which show at a glance what the run is using.

 @param request - connection and record
 */
async function noteStartingState(
  { connection, record, }: { readonly connection: ConnectionToPi; readonly record: RecordOfPiRun; },
): Promise<void> {
  /**
   Pi's state.
   */
  const state = fieldsOf((await connection.send({ command: { type: 'get_state', }, },)).data,);
  record.note({ kind: 'state', model: fieldsOf(state.model,).id, thinkingLevel: state.thinkingLevel, },);
  /**
   Pi's commands.
   */
  const commands = fieldsOf((await connection.send({ command: { type: 'get_commands', }, },)).data,).commands;
  record.note({
    kind: 'commands',
    names: Array.isArray(commands,) ? (commands as readonly PiRecord[]).map((command,) => command.name) : [],
  },);
}

/**
 Runs Pi once and writes everything the caller reads afterwards.

 @param request - options of this run and the environment Pi inherits

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
   Where messages for Pi arrive.
   */
  const inbox = join(options.runFolder, 'inbox',);
  await mkdir(inbox, { recursive: true, },);
  /**
   The run's feed and counts.
   */
  const record = new RecordOfPiRun(join(options.runFolder, 'feed.jsonl',),);
  /**
   The running Pi.
   */
  const connection: ConnectionToPi = new ConnectionToPi({
    piArguments: piCommandLine(options,),
    workingFolder: options.workingFolder,
    environment: { ...environment, [NESTED_RUN_VARIABLE]: '1', },
    standardErrorFile: join(options.runFolder, 'stderr.log',),
    listeners: {
      onEvent: (event,) => {
        record.observe({
          event,
          declineDialog: (id,) => {
            connection.write({ type: 'extension_ui_response', id, cancelled: true, },);
          },
        },);
      },
      onUnreadableLine: (line,) => {
        record.note({ kind: 'non-json', line: preview({ value: line, length: FEED_PREVIEW_CHARACTERS.unreadableLine, },), },);
      },
      onExit: (exit,) => {
        record.note({ kind: 'exit', ...exit, },);
      },
      onProblem: (problem,) => {
        record.note({ kind: 'problem', problem, },);
      },
    },
  },);
  await noteStartingState({ connection, record, },);
  /**
   Why the run ended.
   */
  const endReason = await sendFirstPrompt({ connection, record, options, },) ?? await watchUntilEnded({
    connection,
    record,
    options,
    inbox,
    stopFile: join(options.runFolder, 'stop',),
    deadline: Date.now() + (options.wallSeconds * MILLISECONDS_PER_SECOND),
  },);
  /**
   Pi's own statistics, asked for while it can still answer.
   */
  const stats = await connection.send({ command: { type: 'get_session_stats', }, },);
  if (record.running) {
    await connection.send({ command: { type: 'abort', }, },);
  }
  /**
   What the caller reads.
   */
  const summary = record.summarize({ endReason, piExit: connection.exit, sessionStats: stats.data ?? stats.error ?? null, },);
  await writeSummary({ runFolder: options.runFolder, summary, },);
  await connection.shutDown();
  if (!CLEAN_END_REASONS.has(endReason,)) {
    throw new PiRunEndedEarlyError(
      `The Pi run ended with "${endReason}". The summary is in ${join(options.runFolder, 'summary.json',)} and Pi's standard error in ${
        join(options.runFolder, 'stderr.log',)
      }.`,
    );
  }
  return summary;
}
