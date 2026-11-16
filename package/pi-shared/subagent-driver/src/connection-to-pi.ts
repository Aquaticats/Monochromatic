/**
 The Pi process and its RPC channel: starts Pi, reads one JSON record per line, matches answers to commands, and ends Pi within a
 bound.

 @module
 */

import { spawn, } from 'node:child_process';
import { Writable, } from 'node:stream';
import { setTimeout as wait, } from 'node:timers/promises';
import { SECRET_VALUES_FD, } from './secret-values-handed-to-pi.ts';
import {
  COMMAND_ANSWER_MILLISECONDS,
  SHUTDOWN_GRACE_MILLISECONDS,
} from './driver-constants.ts';
import { didPiLeave, } from './did-pi-leave.ts';
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

 @param standardErrorAppends - whether that file is added to rather than replaced, for a run whose Pi is started more than once

 @param listeners - what the run wants to hear about

 @param secretValuesHandover - the secret values Pi's extension reads from its own descriptor, when this run masks any

 @param mask - the masking Pi's standard error is kept through

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
    standardErrorAppends = false,
    listeners,
    secretValuesHandover,
    mask,
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
      stdio: (secretValuesHandover === undefined)
        ? [
          'pipe',
          'pipe',
          'pipe',
        ]
        : [
          'pipe',
          'pipe',
          'pipe',
          'pipe',
        ],
      env: environment,
    },
  );
  /**
   The descriptor Pi's masking extension reads while it loads: the values are written once and the driver's end is closed, so
   nothing of them is left in the driver's own pipes.

   A write that fails is a problem of this channel and never the run's ending: Pi reports whether the masking is in place from
   its own command list, which is the answer the run turns on.
   */
  if (secretValuesHandover !== undefined) {
    /**
     The driver's end of that descriptor, when the operating system opened one.
     */
    const valuesPipe = child.stdio[SECRET_VALUES_FD];
    if (valuesPipe instanceof Writable) {
      valuesPipe.on(
        'error',
        function onValuesWriteError({ message, }: { readonly message: string },) {
          listeners.onProblem(`could not hand Pi the secret values it masks: ${message}`,);
        },
      );
      valuesPipe.end(secretValuesHandover,);
    }
    else {
      listeners.onProblem('Pi was started without the descriptor its masking extension reads the secret values from',);
    }
  }

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
    append: standardErrorAppends,
    ...(mask === undefined ? {} : { mask, }),
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
   How Pi left, as soon as the driver can know it: the exit the run's records were told of, or the status the operating system
   gave the process before its output ended.

   A tool Pi started can hold Pi's output open after Pi has gone, and then the close of that output, which is what this channel
   calls Pi's leaving, arrives as long as the tool lives. What a run records must not wait for that: Pi has left, and a run whose Pi
   has left is never one the bounds on ending it ran out on.

   @returns how Pi ended, with every field absent while it runs
   */
  function exitKnown(): PiExit {
    return state.exit ?? exitOfChild();
  }

  /**
   Whether this channel can say Pi has gone, which every stage of an ending waits for.

   @returns whether Pi's leaving is known, by the exit this channel marked or by the status the operating system gave the process
   */
  function piHasGone(): boolean {
    return (state.exit !== undefined) || didPiLeave(exitOfChild(),);
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

   @returns whether the signal was sent, which a process that had already gone is never sent
   */
  function signalPi(signal: NodeJS.Signals,): boolean {
    if ((child.exitCode !== null) || (child.signalCode !== null)) {
      return false;
    }
    if (!child.kill(signal,)) {
      state.problem = `the operating system refused ${signal}`;
      listeners.onProblem(`could not send ${signal} to Pi: the operating system refused it.`,);
      return false;
    }
    return true;
  }

  /**
   Closes Pi's input, then signals it while it stays, and reports whether it went in time.

   @param stageMilliseconds - how long each stage may take

   @returns whether Pi went, and every signal the driver had to send to try for it
   */
  async function leavePi(stageMilliseconds: number,): Promise<{
    readonly stopped: boolean;
    readonly signalsSent: readonly NodeJS.Signals[];
  }> {
    /**
     The signals this ending sent, in the order it sent them.
     */
    const signalsSent: NodeJS.Signals[] = [];
    if (!child.stdin
      .destroyed) {
      child.stdin
        .end();
    }
    if (await wentWithin(stageMilliseconds,)) {
      return {
        stopped: true,
        signalsSent,
      };
    }
    if (signalPi('SIGTERM',)) {
      signalsSent.push('SIGTERM',);
    }
    if (await wentWithin(stageMilliseconds,)) {
      return {
        stopped: true,
        signalsSent,
      };
    }
    if (signalPi('SIGKILL',)) {
      signalsSent.push('SIGKILL',);
    }
    return {
      stopped: await wentWithin(stageMilliseconds,),
      signalsSent,
    };
  }

  /**
   Ends Pi, closing its input first and then signalling it if it stays, and settles Pi's standard error before letting go of any
   handle.

   Pi's leaving is what the answer turns on, and it is asked for once every stage is over: a Pi that left while its output was
   still open is one that left, and the run's record says so rather than that the driver could not end it.

   @param graceMilliseconds - how long each stage may take

   @returns what the ending took, and whether Pi's leaving was known when it was over
   */
  async function shutDown({
    graceMilliseconds: stageMilliseconds = graceMilliseconds,
  }: ShutDownRequest = {},): Promise<ShutDownOutcome> {
    /**
     What the stages of leaving did, which is where each signal and each spent bound comes from.
     */
    const leaving = await leavePi(stageMilliseconds,);
    await settleStandardError(stageMilliseconds,);
    /**
     Whether Pi's leaving was known once every stage of the ending was over, whenever the driver heard of it.
     */
    const stopped = piHasGone();
    // Pi's pipes are let go of either way: a tool Pi started can hold its output open long after Pi has gone, and a read that
    // never ends cannot keep the driver from ending.
    releaseHandles();
    return {
      stopped,
      signalsSent: leaving.signalsSent,
      stageMilliseconds,
      outlivedTheLastBound: !leaving.stopped,
      ...(state.problem === undefined ? {} : { refusedSignal: state.problem, }),
    };
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
    hasExited: piHasGone,
    exitDetails: exitKnown,
    write: records.write,
    send,
    shutDown,
  };
}
