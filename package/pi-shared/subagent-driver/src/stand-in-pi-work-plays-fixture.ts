/**
 What the stand-in Pi does with a prompt when the subject of a test is work a run leaves running: it starts a real process the
 machine's own process table holds, reports it the way the `process` tool reports one, and ends its turn. A driver that reads the
 run's feed and the process table has something real to find, and the process it finds belongs to no stub.

 The plays here are the two a wake is judged by: work that ends while the run has time left, and work that outlasts it.

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
  emit,
  finishRun,
} from './stand-in-pi-records-fixture.ts';

/**
 Milliseconds the work a run leaves behind takes when it ends inside the run: shorter than the run's wall clock and long enough
 that the driver looks at the process table more than once while it runs.
 */
export const WORK_MILLISECONDS = 1_500;

/**
 Milliseconds the work takes when it outlasts the run, so the run can only end on its own wall clock.
 */
export const WORK_BEYOND_THE_RUN_MILLISECONDS = 30_000;

/**
 Milliseconds in a second, which `sleep` takes its argument in.
 */
const MILLISECONDS_PER_SECOND = 1_000;

/**
 How many looks the driver takes while the woken agent is still reading its message, so its reply lands well after the look that
 sent the message.
 */
const LOOKS_A_WAKE_ANSWER_SPANS = 3;

/**
 Milliseconds the agent takes to read the message that wakes it, several looks above the one that sent it: a driver that ends the
 run as soon as its work is over cuts that reply off, and a real Pi takes far longer than this.
 */
const WAKE_ANSWER_MILLISECONDS = LOOKS_A_WAKE_ANSWER_SPANS * POLL_MILLISECONDS;

/**
 The `process` tool's own name for the work, which the driver's line carries and the agent's reply is matched against.
 */
const WORK_ID = 'proc_work1';

/**
 The name the agent gave the work, which is the one a reader of a build remembers.
 */
const WORK_NAME = 'native-build';

/**
 The command the tool reported, as the agent wrote it.
 */
const WORK_COMMAND = 'cargo build';

/**
 Starts a real process that the machine's process table holds, and says its number in the log a test reads.

 A process the stand-in starts is in the driver's own control group, which is why the play reports it through the `process` tool
 instead: that is how the driver finds work the table alone cannot tell from the run's own Pi.

 @param milliseconds - how long the process lives

 @returns the process number
 */
async function startWork(milliseconds: number,): Promise<number> {
  /**
   The work itself, detached so it outlives a stand-in its driver stops, and quiet so it writes nothing to a channel.
   */
  const work = spawn(
    'sleep',
    [String(milliseconds / MILLISECONDS_PER_SECOND,),],
    {
      stdio: [
        'ignore',
        'ignore',
        'ignore',
      ],
      detached: true,
    },
  );
  work.unref();
  /**
   Its number, which is none when the system refused to start it.
   */
  const { pid } = work;
  if (pid === undefined) {
    throw new Error('the stand-in could not start the work its scenario needs',);
  }
  await writeLogLine(`marker work ${String(pid,)}`,);
  return pid;
}

/**
 Plays one prompt as a turn that leaves work running, and the message that follows it as the agent reading that work.

 @param id - command's identifier

 @param milliseconds - how long the work lives
 */
async function leaveWorkRunning({
  id,
  milliseconds,
}: {
  readonly id: unknown;
  readonly milliseconds: number
},): Promise<void> {
  // The message the driver sends once the work ended is the agent going on: it reads what the build said and finishes its task.
  if (seen.prompts > 1) {
    answer({ id, },);
    await wait(WAKE_ANSWER_MILLISECONDS,);
    finishRun('the build finished and the task goes on',);
    return;
  }
  answer({ id, },);
  /**
   The process the run leaves behind.
   */
  const pid = await startWork(milliseconds,);
  emit({
    type: 'tool_execution_start',
    toolName: 'process',
    args: {
      action: 'start',
      command: WORK_COMMAND,
    },
  },);
  // The record a process tool's `start` answers with, in the shape `work-the-run-left-running.ts` reads: the number it holds, the
  // folder it works in and the status it reports.
  emit({
    type: 'tool_execution_end',
    toolName: 'process',
    isError: false,
    result: {
      details: {
        process: {
          id: WORK_ID,
          name: WORK_NAME,
          pid,
          command: WORK_COMMAND,
          cwd: '/',
          status: 'running',
        },
      },
    },
  },);
  finishRun('started the build and ended my turn',);
}

/**
 What each of these scenarios does with a prompt.

 @param id - command's identifier
 */
export const workPlays: ReadonlyMap<string, (id: unknown,) => void> = new Map<string, (id: unknown,) => void>([
  [
    'leaves-work-running',
    function leaveWorkThatEnds(id,) {
      void leaveWorkRunning({
        id,
        milliseconds: WORK_MILLISECONDS,
      },);
    },
  ],
  [
    'leaves-work-running-past-its-wall-clock',
    function leaveWorkThatOutlasts(id,) {
      void leaveWorkRunning({
        id,
        milliseconds: WORK_BEYOND_THE_RUN_MILLISECONDS,
      },);
    },
  ],
],);
