/**
 The Pi process and its RPC channel: starts Pi, reads one JSON record per line, matches answers to commands, and ends Pi within a
 bound.

 @module
 */

import { spawn, } from 'node:child_process';
import { createWriteStream, } from 'node:fs';
import type { Readable, } from 'node:stream';
import { pipeline, } from 'node:stream/promises';
import { setTimeout as wait, } from 'node:timers/promises';
import {
  COMMAND_ANSWER_MILLISECONDS,
  SHUTDOWN_GRACE_MILLISECONDS,
} from './driver-constants.ts';
import type {
  PiExit,
  PiRecord,
} from './driver-types.ts';
import { isRecord, } from './read-pi-fields.ts';

/**
 The byte that ends a record. Records are split on it alone, because Unicode line separators are valid inside JSON strings.
 */
const LINE_FEED = 0x0A;

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
   Ends Pi: closes its input, then signals it if it stays.
   */
  readonly shutDown: () => Promise<void>;
};

/**
 Copies Pi's standard error to a file for as long as Pi writes it.

 @param source - Pi's standard error

 @param file - file that receives it

 @param onProblem - where to report a failure

 @example
 ```ts
 await keepStandardError({ source: child.stderr, file: 'stderr.log', onProblem: report, },);
 ```
 */
async function keepStandardError(
  {
    source,
    file,
    onProblem,
  }: {
    readonly source: Readable;
    readonly file: string;
    readonly onProblem: (problem: string,) => void;
  },
): Promise<void> {
  try {
    await pipeline(
      source,
      createWriteStream(file,),
    );
  }
  catch (error) {
    onProblem(`could not keep Pi's standard error: ${String(error,)}`,);
  }
}

/**
 Finds where each record in a chunk ends.

 @param chunk - bytes as they arrived

 @returns offset of each line feed, one at a time

 @example
 ```ts
 const ends = [...lineEndsIn(Buffer.from('a\nb\n',),),];
 ```
 */
function* lineEndsIn(chunk: Buffer,): Generator<number> {
  for (let lineEnd = chunk.indexOf(LINE_FEED,); lineEnd !== (-1); lineEnd = chunk.indexOf(
    LINE_FEED,
    lineEnd + 1,
  )) {
    yield lineEnd;
  }
}

/**
 Starts Pi and returns the connection to it.

 @param piArguments - Pi's command line

 @param workingFolder - folder Pi starts in

 @param environment - Pi's environment

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
   What changes while Pi runs.
   */
  const state: {
    exit?: PiExit;
    commandsSent: number;
    partialLine: Buffer[]
  } = {
    commandsSent: 0,
    partialLine: [],
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
   Splits a chunk of Pi's output into records. Pieces of an unfinished line are joined once, when it ends, so a long line costs
   one pass.

   @param chunk - bytes as they arrived, which may end inside a record or inside a character
   */
  function readChunk(chunk: Buffer,): void {
    /**
     Where the part after the last complete line starts.
     */
    const tailStart = [...lineEndsIn(chunk,),].reduce(
      function readOneLine(
        lineStart,
        lineEnd,
      ) {
      /**
       The whole line.
       */
        const line = Buffer.concat([
          ...state.partialLine,
          chunk.subarray(
            lineStart,
            lineEnd,
          ),
        ],)
          .toString('utf8',);
        state.partialLine = [];
        readLine(line,);
        return lineEnd + 1;
      },
      0,
    );
    if (tailStart < chunk.length) {
      state.partialLine
        .push(chunk.subarray(tailStart,),);
    }
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
      id,
      ...command,
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

   @returns whether Pi went in time
   */
  async function wentWithinGrace(): Promise<boolean> {
    /**
     Which came first.
     */
    const outcome = await Promise.race([
      closed.promise,
      wait(
        graceMilliseconds,
        'still-running' as const,
        { ref: false, },
      ),
    ],);
    return outcome === 'closed';
  }

  /**
   Ends Pi, closing its input first and then signalling it if it stays.
   */
  async function shutDown(): Promise<void> {
    if (state.exit !== undefined) {
      return;
    }
    child.stdin
      .end();
    if (await wentWithinGrace()) {
      return;
    }
    child.kill('SIGTERM',);
    if (await wentWithinGrace()) {
      return;
    }
    child.kill('SIGKILL',);
    await wentWithinGrace();
    releaseHandles();
  }

  child.stdout
    .on(
    'data',
    readChunk,
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
      markExited({ processError: message, },);
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
  void keepStandardError({
    source: child.stderr,
    file: standardErrorFile,
    onProblem: listeners.onProblem,
  },);

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
