/**
 The Pi process and its RPC channel: starts Pi, reads one JSON record per line, matches answers to commands, and ends Pi within a
 bound.

 @module
 */

import {
  type ChildProcessWithoutNullStreams,
  spawn,
} from 'node:child_process';
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

/**
 The byte that ends a record. Records are split on it alone, because Unicode line separators are valid inside JSON strings.
 */
const LINE_FEED = 0x0a;

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
};

/**
 Copies Pi's standard error to a file for as long as Pi writes it.

 @param request - stream, file and where to report a failure

 @example
 ```ts
 await keepStandardError({ source: child.stderr, file: 'stderr.log', onProblem: console.error, },);
 ```
 */
async function keepStandardError(
  { source, file, onProblem, }: { readonly source: Readable; readonly file: string; readonly onProblem: (problem: string,) => void; },
): Promise<void> {
  try {
    await pipeline(source, createWriteStream(file,),);
  }
  catch (error) {
    onProblem(`could not keep Pi's standard error: ${String(error,)}`,);
  }
}

/**
 One running Pi and the commands waiting for its answers.
 */
export class ConnectionToPi {
  readonly #child: ChildProcessWithoutNullStreams;
  readonly #listeners: ConnectionListeners;
  readonly #waitingForAnswer = new Map<string, (answer: PiRecord,) => void>();
  readonly #closed = Promise.withResolvers<'closed'>();

  /**
   Pieces of a record whose line has not ended yet; joined once, when it does, so a long line costs one pass.
   */
  #partialLine: Buffer[] = [];
  #exit: PiExit | null = null;
  #commandsSent = 0;

  /**
   Starts Pi.

   @param request - arguments, folder, environment, file for standard error, and listeners

   @example
   ```ts
   const connection = new ConnectionToPi(request,);
   ```
   */
  constructor({ piArguments, workingFolder, environment, standardErrorFile, listeners, }: ConnectionRequest,) {
    this.#listeners = listeners;
    this.#child = spawn('pi', [...piArguments,], { cwd: workingFolder, stdio: ['pipe', 'pipe', 'pipe',], env: environment, },);
    this.#child.stdout.on('data', (chunk: Buffer,) => {
      this.#readChunk(chunk,);
    },);
    this.#child.stdin.on('error', (error: Error,) => {
      listeners.onProblem(`could not write to Pi: ${error.message}`,);
    },);
    this.#child.on('error', (error: Error,) => {
      if (this.#child.pid === undefined) {
        this.#markExited({ code: null, signal: null, startError: error.message, },);
      }
      else {
        listeners.onProblem(`Pi's process reported: ${error.message}`,);
      }
    },);
    this.#child.on('close', (code: number | null, signal: string | null,) => {
      this.#markExited({ code, signal, startError: null, },);
    },);
    void keepStandardError({ source: this.#child.stderr, file: standardErrorFile, onProblem: listeners.onProblem, },);
  }

  /**
   How Pi ended, or null while it runs.

   @returns the exit, once there is one
   */
  get exit(): PiExit | null {
    return this.#exit;
  }

  /**
   Records that Pi has gone, once, and answers every command still waiting.

   @param exit - how Pi ended
   */
  #markExited(exit: PiExit,): void {
    if (this.#exit !== null) {
      return;
    }
    this.#exit = exit;
    this.#listeners.onExit(exit,);
    for (const answer of this.#waitingForAnswer.values()) {
      answer({ success: false, error: 'pi exited', },);
    }
    this.#closed.resolve('closed',);
  }

  /**
   Splits a chunk of Pi's output into records.

   @param chunk - bytes as they arrived, which may end inside a record or inside a character
   */
  #readChunk(chunk: Buffer,): void {
    /**
     Where the part of the chunk not yet read starts.
     */
    let start = 0;
    for (let lineEnd = chunk.indexOf(LINE_FEED,); lineEnd !== -1; lineEnd = chunk.indexOf(LINE_FEED, start,)) {
      this.#partialLine.push(chunk.subarray(start, lineEnd,),);
      /**
       The whole line.
       */
      const line = Buffer.concat(this.#partialLine,).toString('utf8',);
      this.#partialLine = [];
      start = lineEnd + 1;
      this.#readLine(line,);
    }
    if (start < chunk.length) {
      this.#partialLine.push(chunk.subarray(start,),);
    }
  }

  /**
   Hands one line to whoever waits for it.

   @param line - text of one record
   */
  #readLine(line: string,): void {
    if (line.trim() === '') {
      return;
    }
    /**
     The parsed record, or undefined for a line that is not JSON.
     */
    const record = this.#parse(line,);
    if (record === undefined) {
      return;
    }
    if ((record.type === 'response') && (typeof record.id === 'string')) {
      this.#waitingForAnswer.get(record.id,)?.(record,);
      return;
    }
    this.#listeners.onEvent(record,);
  }

  /**
   Parses a line.

   @param line - text of one record

   @returns the record, or undefined after reporting a line that is not JSON
   */
  #parse(line: string,): PiRecord | undefined {
    try {
      return JSON.parse(line,) as PiRecord;
    }
    catch (error) {
      this.#listeners.onUnreadableLine(`${line} (${String(error,)})`,);
      return undefined;
    }
  }

  /**
   Writes a command with no answer to wait for.

   @param command - record to send

   @example
   ```ts
   connection.write({ type: 'extension_ui_response', id: 'd1', cancelled: true, },);
   ```
   */
  write(command: PiRecord,): void {
    if (this.#exit === null) {
      this.#child.stdin.write(`${JSON.stringify(command,)}\n`,);
    }
  }

  /**
   Sends a command and waits for its answer, within a bound.

   @param request - command and how long to wait

   @returns Pi's answer, or a failed answer when Pi has gone or stays silent

   @example
   ```ts
   const state = await connection.send({ command: { type: 'get_state', }, },);
   ```
   */
  async send(
    { command, answerMilliseconds = COMMAND_ANSWER_MILLISECONDS, }: { readonly command: PiRecord; readonly answerMilliseconds?: number; },
  ): Promise<PiRecord> {
    if (this.#exit !== null) {
      return { success: false, error: 'pi exited', };
    }
    this.#commandsSent += 1;
    /**
     The identifier that ties the answer to this command.
     */
    const id = `driver-${String(this.#commandsSent,)}`;
    /**
     The answer, settled by whichever comes first.
     */
    const answer = Promise.withResolvers<PiRecord>();
    /**
     The bound on the wait.
     */
    const timer = setTimeout(() => {
      answer.resolve({ success: false, error: `no answer in ${String(answerMilliseconds,)} ms`, },);
    }, answerMilliseconds,);
    this.#waitingForAnswer.set(id, answer.resolve,);
    this.write({ id, ...command, },);
    /**
     What came back.
     */
    const result = await answer.promise;
    clearTimeout(timer,);
    this.#waitingForAnswer.delete(id,);
    return result;
  }

  /**
   Waits for Pi to go, within a bound.

   @returns whether Pi went in time
   */
  async #wentWithinGrace(): Promise<boolean> {
    /**
     Which came first; the timer holds nothing open once Pi has gone.
     */
    const outcome = await Promise.race([
      this.#closed.promise,
      wait(SHUTDOWN_GRACE_MILLISECONDS, 'still-running' as const, { ref: false, },),
    ],);
    return outcome === 'closed';
  }

  /**
   Ends Pi: closes its input, then signals it if it stays.

   @example
   ```ts
   await connection.shutDown();
   ```
   */
  async shutDown(): Promise<void> {
    if (this.#exit !== null) {
      return;
    }
    this.#child.stdin.end();
    if (await this.#wentWithinGrace()) {
      return;
    }
    this.#child.kill('SIGTERM',);
    if (await this.#wentWithinGrace()) {
      return;
    }
    this.#child.kill('SIGKILL',);
    await this.#wentWithinGrace();
  }
}
