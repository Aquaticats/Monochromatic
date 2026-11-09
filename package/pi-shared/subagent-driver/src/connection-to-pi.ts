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
  PiExit,
  PiRecord,
} from './driver-types.ts';
import { keepStandardError, } from './keep-standard-error.ts';
import { isRecord, } from './read-pi-fields.ts';
import { startLineSplitter, } from './split-pi-output.ts';

/**
 Stands for a line that held no record.
 */
const UNREADABLE_LINE: PiRecord = Object.freeze({},);

/**
 What the connection tells the run about.
 */
export type ConnectionListeners = {
  readonly onEvent: (record: PiRecord,) => void;
  readonly onUnreadableLine: (line: string,) => void;
  readonly onExit: (exit: PiExit,) => void;

  /**
   A failure of the channel itself, such as a write to a Pi that has gone.
   */
  readonly onProblem: (problem: string,) => void;
};

/**
 What starting Pi needs.
 */
export type ConnectionRequest = {
  readonly piArguments: readonly string[];
  readonly workingFolder: string;

  /**
   Pi's environment, passed as given: this seam starts one process and applies no policy. `supervisePiRun` is where the
   credential rule runs, so a caller of this function that builds its own environment keeps whatever it passes.
   */
  readonly environment: NodeJS.ProcessEnv;
  readonly standardErrorFile: string;
  readonly listeners: ConnectionListeners;

  /**
   How long Pi gets at each stage of ending; tests shorten it.
   */
  readonly graceMilliseconds?: number;
};

/**
 A command to send and how long to wait for its answer.
 */
export type CommandRequest = {
  readonly command: PiRecord;
  readonly answerMilliseconds?: number;
};

/**
 What ending Pi needs.
 */
export type ShutDownRequest = {
  /**
   How long each stage of ending Pi may take. The whole ending takes at most `SHUTDOWN_STAGES` of these, so a caller that has its
   own bound divides it by that count.
   */
  readonly graceMilliseconds?: number;
};

/**
 How ending Pi went.
 */
export type ShutDownOutcome =
  | { readonly stopped: true }
    | {
      readonly stopped: false;

    /**
     Why the driver could not confirm that Pi had gone.
     */
      readonly reason: string;
    };

/**
 A running Pi.
 */
export type ConnectionToPi = {
  readonly hasExited: () => boolean;

  /**
   How Pi ended, or nothing while it runs.
   */
  readonly exitDetails: () => PiExit;

  /**
   Writes a command with no answer to wait for.
   */
  readonly write: (command: PiRecord,) => void;

  /**
   Sends a command and waits for its answer, within a bound. Pi's silence or absence comes back as a failed answer.
   */
  readonly send: (request: CommandRequest,) => Promise<PiRecord>;

  /**
   Ends Pi: closes its input, then signals it if it stays, then settles every handle. It reports whether Pi's leaving could be
   confirmed, because a signal the operating system refused is not proof that Pi went.
   */
  readonly shutDown: (request?: ShutDownRequest,) => Promise<ShutDownOutcome>;
};

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
  const waitingForAnswer = new Map<string, (answer: PiRecord,) => void>();
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
   Records that Pi has gone, once, and answers every command still waiting.

   @param exit - how Pi ended
   */
  function markExited(exit: PiExit,): void {
    if (state.exit !== undefined) {
      return;
    }
    state.exit = exit;
    listeners.onExit(exit,);
    for (const answer of waitingForAnswer.values()) {
      answer({
        success: false,
        error: 'pi exited',
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
   Parses a line.

   @param line - text of one record

   @returns the record, or the marker for a line that held none, after reporting it
   */
  function parseLine(line: string,): PiRecord {
    try {
      /**
       The line's value, of any shape.
       */
      const parsed: unknown = JSON.parse(line,);
      if (isRecord(parsed,)) {
        return parsed;
      }
      listeners.onUnreadableLine(line,);
      return UNREADABLE_LINE;
    }
    catch (error) {
      listeners.onUnreadableLine(`${line} (${String(error,)})`,);
      return UNREADABLE_LINE;
    }
  }

  /**
   Hands one line to whoever waits for it.

   @param line - text of one record
   */
  function readLine(line: string,): void {
    if (line.trim() === '') {
      return;
    }
    /**
     The record, or the marker for a line that held none.
     */
    const record = parseLine(line,);
    if (record === UNREADABLE_LINE) {
      return;
    }
    if ((record.type === 'response') && ((typeof record.id) === 'string')) {
      waitingForAnswer.get(record.id,)?.(record,);
      return;
    }
    listeners.onEvent(record,);
  }

  /**
   Writes a command if Pi can still read it.

   @param command - record to send
   */
  function write(command: PiRecord,): void {
    if (state.exit === undefined) {
      child.stdin
        .write(`${JSON.stringify(command,)}\n`,);
    }
  }

  /**
   Sends a command and waits for its answer.

   @param command - record to send

   @param answerMilliseconds - longest wait

   @returns Pi's answer, or a failed answer when Pi has gone or stays silent
   */
  async function send(
    {
      command,
      answerMilliseconds = COMMAND_ANSWER_MILLISECONDS,
    }: CommandRequest,
  ): Promise<PiRecord> {
    if (state.exit !== undefined) {
      return {
        success: false,
        error: 'pi exited',
      };
    }
    state.commandsSent += 1;
    /**
     The identifier that ties the answer to this command.
     */
    const id = `driver-${String(state.commandsSent,)}`;
    /**
     The answer, settled by whichever comes first.
     */
    const answer = Promise.withResolvers<PiRecord>();
    /**
     The bound on the wait.
     */
    const timer = setTimeout(
      function giveUpOnAnswer() {
        answer.resolve({
          success: false,
          error: `no answer in ${String(answerMilliseconds,)} ms`,
        },);
      },
      answerMilliseconds,
    );
    waitingForAnswer.set(
      id,
      answer.resolve,
    );
    write({
      ...command,
      id,
    },);
    /**
     What came back.
     */
    const result = await answer.promise;
    clearTimeout(timer,);
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

  child.stdout
    .on(
    'data',
    startLineSplitter({ onLine: readLine, },),
    );
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
    write,
    send,
    shutDown,
  };
}
