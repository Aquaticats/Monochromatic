/**
 What the stand-in Pi does when the subject of a test is a command that says nothing: it reports one tool start and then writes
 nothing at all, so the driver has only the silence to go on. Three plays: a command that never ends, a command that reports output
 steadily so no silence ever comes, and a command whose own process is gone while a descendant of it holds this process's output
 open.

 A play for `abort` is here too, because ending the operation in flight is what the driver does to such a command: Pi kills what
 the command started, settles, and only then answers, so a stand-in that answered first would let a driver send its message into a
 busy session and call that a run gone on.

 @module
 */

import { spawn, } from 'node:child_process';
import { setTimeout as wait, } from 'node:timers/promises';
import { POLL_MILLISECONDS, } from '../dist/final/node/index.mjs';
import {
  seen,
  writeLogLine,
} from './stand-in-pi-log-fixture.ts';
import {
  answer,
  assistantReply,
  emit,
} from './stand-in-pi-records-fixture.ts';

/**
 The command a play reports, shaped as the search that held one real run for 15.6 hours.
 */
export const SILENT_COMMAND = 'rg -l --fixed-strings something .';

/**
 What the agent says it did once the driver told it the command was ended, which a test reads to see that the run went on.
 */
export const WENT_ON = 'read what the command left and went on';

/**
 How many output reports the command that works makes for every look at a run, so a report is never the reason a look sees a
 silence: a host that delivers one ten looks late still leaves the run inside the bound a test states.
 */
const OUTPUT_REPORTS_PER_LOOK = 30;

/**
 Milliseconds between two output reports of the command that works.
 */
const OUTPUT_UPDATE_MILLISECONDS = Math.floor(POLL_MILLISECONDS / OUTPUT_REPORTS_PER_LOOK,);

/**
 Milliseconds the command that works keeps reporting output for, measured on the clock rather than counted in reports: a loaded
 host delivers fewer reports over the same stretch, so its own life stays what the test states and ends inside the run's wall
 clock.
 */
const OUTPUT_MILLISECONDS = 4_000;

/**
 Seconds the descendant that holds this process's output open lives, so a test that fails before it ends the descendant leaves
 nothing running for longer than this.
 */
const DESCENDANT_SECONDS = '30';

/**
 The code the operating system gives a process or a group that is already gone.
 */
const NO_SUCH_PROCESS_CODE = 'ESRCH';

/**
 The tool a play reports the command under.
 */
const TOOL = 'bash';

/**
 The groups a play started whose descendant holds this process's output open, which the play for `abort` ends.
 */
const descendantGroups = new Set<number>();

/**
 The code the operating system gave a failure, when it gave one.

 Kept here rather than imported from `error-code-of.ts`, because everything a test imports must reach the artifact the package
 ships, and that helper is not part of it.

 @param error - what was caught

 @returns the code, or the empty text when the failure carries none

 @example
 ```ts
 const gone = failureCodeOf(error,) === NO_SUCH_PROCESS_CODE;
 ```
 */
export function failureCodeOf(error: unknown,): string {
  if (((typeof error) !== 'object') || (error === null)
    || (!('code' in error))) {
    return '';
  }
  return ((typeof error.code) === 'string') ? error.code : '';
}

/**
 Writes the record of a command that ended because the driver aborted it.
 */
function emitAbortedCommand(): void {
  emit({
    type: 'tool_execution_end',
    toolName: TOOL,
    isError: true,
    result: {
      content: [{
        type: 'text',
        text: 'Command aborted',
      },],
    },
  },);
}

/**
 Writes the turn that answers the driver's message: the agent read what the command left and went on.

 @param id - command's identifier
 */
function playTheMessage(id: unknown,): void {
  answer({ id, },);
  emit({ type: 'agent_start', },);
  emit({ type: 'turn_start', },);
  emit(assistantReply(WENT_ON,),);
  emit({ type: 'agent_end', },);
  emit({ type: 'agent_settled', },);
}

/**
 Writes the turn that starts a command and never ends it.

 @param id - command's identifier
 */
function startTheCommand(id: unknown,): void {
  answer({ id, },);
  emit({ type: 'agent_start', },);
  emit({ type: 'turn_start', },);
  emit({
    type: 'tool_execution_start',
    toolName: TOOL,
    args: { command: SILENT_COMMAND, },
  },);
}

/**
 Ends the group a shell led, which is what Pi's own kill of a tool's process tree does when the operation is aborted.

 @param pid - the shell's process number, which is the group's
 */
function endTheGroup(pid: number,): void {
  try {
    process.kill(
      -pid,
      'SIGKILL',
    );
  }
  catch (error) {
    // A group that is already gone is what this asks for, so only another refusal is a failure.
    if (failureCodeOf(error,) !== NO_SUCH_PROCESS_CODE) {
      throw error;
    }
  }
}

/**
 Starts a shell that starts a descendant and exits at once, so the descendant holds this process's output open after the
 command's own process is gone, and says so in the log a test reads.

 @returns the process number of the shell, which is the leader of the descendant's group
 */
async function startADescendant(): Promise<number> {
  /**
   The shell, whose group the descendant joins, so the play for `abort` can end both.
   */
  const shell = spawn(
    'sh',
    [
      '--command',
      `sleep ${DESCENDANT_SECONDS} &`,
    ],
    {
      stdio: [
        'ignore',
        'inherit',
        'inherit',
      ],
      detached: true,
    },
  );
  shell.unref();
  /**
   Its number, which is none when the system refused to start it.
   */
  const { pid, } = shell;
  if (pid === undefined) {
    throw new Error('the stand-in could not start the descendant its scenario needs',);
  }
  descendantGroups.add(pid,);
  await writeLogLine(`marker descendant ${String(pid,)}`,);
  return pid;
}

/**
 A prompt in a scenario about a command that says nothing.

 @param id - command's identifier
 */
async function playSilentCommand(id: unknown,): Promise<void> {
  if (seen.prompts > 1) {
    playTheMessage(id,);
    return;
  }
  startTheCommand(id,);
  await writeLogLine('marker command-started',);
}

/**
 A prompt in a scenario about a command whose descendant holds the output open.

 @param id - command's identifier
 */
async function playDescendantHoldingThePipe(id: unknown,): Promise<void> {
  if (seen.prompts > 1) {
    playTheMessage(id,);
    return;
  }
  answer({ id, },);
  await startADescendant();
  emit({ type: 'agent_start', },);
  emit({ type: 'turn_start', },);
  emit({
    type: 'tool_execution_start',
    toolName: TOOL,
    args: { command: SILENT_COMMAND, },
  },);
}

/**
 A prompt in a scenario about a command that reports output steadily and then ends.

 @param id - command's identifier
 */
async function playASteadyCommand(id: unknown,): Promise<void> {
  startTheCommand(id,);
  /**
   When this command began reporting output, which is the clock its life is measured on.
   */
  const beganAt = Date.now();
  /* oxlint-disable eslint/no-await-in-loop -- one output report after another, at the pace a tool reports them. */
  for (let sent = 0; (Date.now() - beganAt) < OUTPUT_MILLISECONDS; sent += 1) {
    await wait(OUTPUT_UPDATE_MILLISECONDS,);
    emit({
      type: 'tool_execution_update',
      toolName: TOOL,
      args: { command: SILENT_COMMAND, },
      partialResult: {
        content: [{
          type: 'text',
          text: `line ${String(sent,)}`,
        },],
      },
    },);
  }
  /* oxlint-enable eslint/no-await-in-loop */
  emit({
    type: 'tool_execution_end',
    toolName: TOOL,
    isError: false,
    result: {
      content: [{
        type: 'text',
        text: 'the command ended',
      },],
    },
  },);
  emit(assistantReply('the command ended and the task goes on',),);
  emit({ type: 'agent_end', },);
  emit({ type: 'agent_settled', },);
}

/**
 What each of these scenarios does with a prompt.
 */
export const silencePlays: ReadonlyMap<string, (id: unknown,) => Promise<void>> = new Map<
  string,
  (id: unknown,) => Promise<void>
  >([
    [
      'silent-command',
      playSilentCommand,
    ],
    [
      'silent-command-descendant-holds-the-pipe',
      playDescendantHoldingThePipe,
    ],
    [
      'silent-command-reports-output',
      playASteadyCommand,
    ],
  ],);

/**
 What each of these scenarios does when the driver aborts the operation in flight: what the command started is killed, the
 session settles, and only then is the abort answered.
 */
export const abortPlays: ReadonlyMap<string, (id: unknown,) => Promise<void>> = new Map<
  string,
  (id: unknown,) => Promise<void>
  >([
    [
      'silent-command',
      async function endTheSilentCommand(id,) {
        emitAbortedCommand();
        emit({
          type: 'agent_settled',
          aborted: true,
        },);
        answer({ id, },);
        await writeLogLine('marker command-ended',);
      },
    ],
    [
      'silent-command-descendant-holds-the-pipe',
      async function endTheDescendant(id,) {
        for (const group of descendantGroups) {
          endTheGroup(group,);
        }
        descendantGroups.clear();
        emitAbortedCommand();
        emit({
          type: 'agent_settled',
          aborted: true,
        },);
        answer({ id, },);
        await writeLogLine('marker descendant-ended',);
      },
    ],
  ]);
