/**
 The Pi process and its RPC channel: starts Pi, reads one JSON record per line, matches answers to commands, and ends Pi within a
 bound.

 @module
 */

import { spawn, } from 'node:child_process';
import { setTimeout as wait, } from 'node:timers/promises';
import {
  COMMAND_ANSWER_MILLISECONDS,
  SHUTDOWN_GRACE_MILLISECONDS,
} from './driver-constants.ts';
import type {
  CommandAnswer,
  PiExit,
  PiRecord,
} from './driver-types.ts';
import type {
  CommandRequest,
  ConnectionListeners,
  ConnectionRequest,
  ConnectionToPi,
  ShutDownOutcome,
  ShutDownRequest,
} from './shapes-of-the-connection.ts';
import {
  startReadingPiRecords,
  type PiRecords,
} from './pi-records-on-a-channel.ts';
import {
  startWaitingForAnAnswer,
  type WaitingForAnAnswer,
} from './wait-for-an-answer.ts';
import { keepStandardError, } from './keep-standard-error.ts';

export type {
  CommandRequest,
  ConnectionListeners,
  ConnectionRequest,
  ConnectionToPi,
  ShutDownOutcome,
  ShutDownRequest,
} from './shapes-of-the-connection.ts';

/**
 Starts Pi and returns the connection to it.

 @param piArguments - Pi's command line

 @param workingFolder - folder Pi starts in

 @param environment - Pi's environment, passed as given; the credential rule lives in `supervisePiRun`

 @param standardErrorFile - file for Pi's standard error

 @param listeners - what the run wants to hear about

 @param graceMilliseconds - how long Pi gets at each stage of ending

 @returns the connection

 @example
 ```ts
 const connection = startConnectionToPi(request,);
 ```
 */
export function startConnectionToPi(
  {
    piArguments,
    workingFolder,
    environment,
    standardErrorFile,
    listeners,
    graceMilliseconds = SHUTDOWN_GRACE_MILLISECONDS,
  }: ConnectionRequest,
): ConnectionToPi {
  /**
   The Pi process.
   */
  const child = spawn(
    'pi',
    [...piArguments,],
    {
      cwd: workingFolder,
      stdio: [
        'pipe',
        'pipe',
        'pipe',
      ],
      env: environment,
    },
  );
  /**
   The commands waiting for Pi's answers, by identifier.
   */
  const waitingForAnswer = new Map<string, WaitingForAnAnswer>();
  /**
   Settles once Pi is known to have gone.
   */
  const closed = Promise.withResolvers<'closed'>();
  /**
   The copy of Pi's standard error, kept so that ending the run waits for it instead of leaving it to outlive the run.
   */
  const standardErrorKept = keepStandardError({
    source: child.stderr,
    file: standardErrorFile,
    onProblem: listeners.onProblem,
  },);
  /**
   What changes while Pi runs.
   */
  const state: {
    exit?: PiExit;
    commandsSent: number;

    /**
     Whether the operating system started Pi, which tells a failed spawn from a failed signal.
     */
    spawned: boolean;

    /**
     The last problem of this channel, such as a signal the operating system refused, which is never proof that Pi went.
     */
    problem?: string;
  } = {
    commandsSent: 0,
    spawned: false,
  };

  /**
   Pi's records, which say when a compaction begins and ends and carry the answers every command waits for.
   */
  const records: PiRecords = startReadingPiRecords({
    output: child.stdout,
    input: child.stdin,
    hasExited: function hasExited() {
      return state.exit !== undefined;
    },
    waitingForAnswer,
    listeners,
  },);


  /**
   Records that Pi has gone, once, and answers every command still waiting.

   @param exit - how Pi ended
   */
  function markExited(exit: PiExit,): void {
    if (state.exit !== undefined) {
      return;
    }
    state.exit = exit;
    listeners.onExit(exit,);
    for (const waiting of waitingForAnswer.values()) {
      waiting.settle({
        ending: 'pi-exited',
        problem: 'pi exited',
      },);
    }
    closed.resolve('closed',);
  }

  /**
   Reads how Pi ended from the process.

   @returns the exit status or signal
   */
  function exitOfChild(): PiExit {
    return {
      ...(child.exitCode === null ? {} : { code: child.exitCode, }),
      ...(child.signalCode === null ? {} : { signal: child.signalCode, }),
    };
  }

  /**
   Lets go of Pi's pipes and handle, so a process that will not end cannot keep the driver alive.
   */
  function releaseHandles(): void {
    child.stdin
      .destroy();
    child.stdout
      .destroy();
    child.stderr
      .destroy();
    child.unref();
  }

  /**
   Sends a command and waits for its answer, counting only the time Pi is silent.

   Pi's silence is the time it spends neither answering nor compacting: a compaction under way is Pi at work on its own session,
   not Pi gone or Pi refusing, so this wait stops counting silence for as long as Pi reports one (its `compaction_start` and
   `compaction_end` records) and starts counting again when the compaction ends. The deadline its caller gives ends the wait
   whatever Pi is doing, which is what bounds a compaction that never ends.

   @param command - record to send

   @param answerMilliseconds - longest silence waited out

   @param deadline - when the wait must be over whatever Pi reports

   @returns Pi's answer, or how the wait ended instead
   */
  async function send(
    {
      command,
      answerMilliseconds = COMMAND_ANSWER_MILLISECONDS,
      deadline,
    }: CommandRequest,
  ): Promise<CommandAnswer> {
    if (state.exit !== undefined) {
      return {
        ending: 'pi-exited',
        problem: 'pi exited',
      };
    }
    if (deadline <= Date.now()) {
      // Nothing is written for a wait that is already over, so Pi is never sent a command whose answer nobody would read.
      return {
        ending: 'deadline',
        problem: 'no answer before the deadline',
      };
    }
    state.commandsSent += 1;
    /**
     The identifier that ties the answer to this command.
     */
    const id = `driver-${String(state.commandsSent,)}`;
    /**
     The wait for this answer, which counts only the silence Pi keeps and never outlasts the deadline its caller gave.
     */
    const waiting = startWaitingForAnAnswer({
      answerMilliseconds,
      deadline,
      compactionOfPi: records.compactionOfPi,
    },);
    waitingForAnswer.set(
      id,
      waiting,
    );
    records.write({
      ...command,
      id,
    },);
    /**
     What came back.
     */
    const result = await waiting.answer;
    waitingForAnswer.delete(id,);
    return result;
  }

  /**
   Waits for Pi to go, within a bound. The timer holds nothing open once Pi has gone.

   @param grace - longest wait

   @returns whether Pi went in time
   */
  async function wentWithin(grace: number,): Promise<boolean> {
    /**
     Which came first.
     */
    const outcome = await Promise.race([
      closed.promise,
      wait(
        grace,
        'still-running' as const,
        { ref: false, },
      ),
    ],);
    return outcome === 'closed';
  }

  /**
   Waits for Pi's standard error copy to end, so a failure of it is reported while the run still has a feed to report it to.

   @param grace - longest wait
   */
  async function settleStandardError(grace: number,): Promise<void> {
    /**
     Which came first: the copy's end, or the bound.
     */
    const outcome = await Promise.race([
      standardErrorKept,
      wait(
        grace,
        'still-copying' as const,
        { ref: false, },
      ),
    ],);
    if (outcome === 'still-copying') {
      listeners.onProblem(
        `Pi's standard error was still being copied ${String(grace,)} ms after the run ended, so the rest of it is not in ${standardErrorFile}.`,
      );
    }
  }

  /**
   Sends Pi a signal, saying so when the operating system refuses it. A refused signal is a problem of this channel, never a
   notification that Pi went.

   @param signal - signal to send
   */
  function signalPi(signal: NodeJS.Signals,): void {
    if ((child.exitCode !== null) || (child.signalCode !== null)) {
      return;
    }
    if (!child.kill(signal,)) {
      state.problem = `the operating system refused ${signal}`;
      listeners.onProblem(`could not send ${signal} to Pi: the operating system refused it.`,);
    }
  }

  /**
   Closes Pi's input, then signals it while it stays, and reports whether it went in time.

   @param stageMilliseconds - how long each stage may take

   @returns whether Pi went
   */
  async function leavePi(stageMilliseconds: number,): Promise<boolean> {
    if (!child.stdin
      .destroyed) {
      child.stdin
        .end();
    }
    if (await wentWithin(stageMilliseconds,)) {
      return true;
    }
    signalPi('SIGTERM',);
    if (await wentWithin(stageMilliseconds,)) {
      return true;
    }
    signalPi('SIGKILL',);
    return await wentWithin(stageMilliseconds,);
  }

  /**
   Ends Pi, closing its input first and then signalling it if it stays, and settles Pi's standard error before letting go of any
   handle.

   @param graceMilliseconds - how long each stage may take

   @returns whether Pi's leaving could be confirmed
   */
  async function shutDown({
    graceMilliseconds: stageMilliseconds = graceMilliseconds,
  }: ShutDownRequest = {},): Promise<ShutDownOutcome> {
    /**
     Whether Pi went, which is settled by its input closing and then by each signal in turn.
     */
    const stopped = (state.exit !== undefined) || (await leavePi(stageMilliseconds,));
    await settleStandardError(stageMilliseconds,);
    if (!stopped) {
      releaseHandles();
      return {
        stopped: false,
        reason: (state.problem === undefined)
          ? `Pi was still running ${String(stageMilliseconds,)} ms after SIGKILL.`
          : `Pi was still running ${String(stageMilliseconds,)} ms after SIGKILL, and a signal had failed: ${state.problem}.`,
      };
    }
    return { stopped: true, };
  }

  child.on(
    'spawn',
    function onSpawn() {
      state.spawned = true;
    },
  );
  child.stdin
    .on(
    'error',
    function onInputError({ message, }: { readonly message: string; },) {
      listeners.onProblem(`could not write to Pi: ${message}`,);
    },
    );
  child.on(
    'error',
    function onProcessError({ message, }: { readonly message: string; },) {
      if (!state.spawned) {
        markExited({ processError: message, },);
        return;
      }
      // A signal or a message that failed says this channel broke, never that Pi went: its exit is still what closes this.
      state.problem = message;
      listeners.onProblem(`could not signal Pi: ${message}`,);
    },
  );
  child.on(
    'exit',
    function onChildExit() {
    // A tool Pi started can keep Pi's output open after Pi has gone, and then 'close' never comes.
      setTimeout(
        function stopWaitingForOutput() {
          releaseHandles();
          markExited(exitOfChild(),);
        },
        graceMilliseconds,
      )
        .unref();
    },
  );
  child.on(
    'close',
    function onChildClose() {
      markExited(exitOfChild(),);
    },
  );

  return {
    hasExited: function hasExited() {
      return state.exit !== undefined;
    },
    exitDetails: function exitDetails() {
      return state.exit ?? {};
    },
    write: records.write,
    send,
    shutDown,
  };
}
