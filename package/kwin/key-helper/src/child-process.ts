/**
 Running external commands from the daemon, in the two shapes it needs: one that
 ends and whose output is a diagnostic, and one that starts an application which
 keeps running past the call.

 The daemon starts `ydotool` and the launcher and can do nothing about their
 failures, but it must not lose them either: a nonzero exit, a signal, or a
 command that could not be started all throw {@link CommandFailedError} naming
 the command, how it ended, and what it wrote to stderr. Callers log the error
 and carry on, so a broken injection never crashes the daemon.

 Both shapes live here, so `ydotool` and the launcher cannot drift apart in how
 they report a failure, and neither ends a child for writing more than the
 daemon wanted to read: a launched application writes for its whole lifetime,
 and a bound that terminated it would close the window this helper just opened.

 @module
 */

import { spawn } from 'node:child_process';
import { once } from 'node:events';
import type { Readable, } from 'node:stream';

import { BYTES_PER_MIB, } from '@monochromatic-dev/const-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { CommandFailedError } from './errors.ts';

//region Logging

/**
 Logger for this module, re-tagged with the enclosing function name at each
 site.
 */
const childProcessLogger = tagged({ tag: 'kwin-key-helper-child-process', },);

//endregion Logging

//region Diagnostic capture

/**
 Bytes of a child's stderr kept for its failure message.

 This is the bound Node's `execFile` applies by default (`maxBuffer`), kept
 because the failure lines an operator reads have always been bounded by it.
 What differs is what happens past it: the text kept here is truncated and
 named as truncated, and the daemon keeps draining the stream, so the child
 finishes writing and runs on instead of being ended for exceeding it.
 */
const DIAGNOSTIC_LIMIT_BYTES: number = BYTES_PER_MIB;

/**
 Drain one child's stderr into a text with a fixed bound.

 The stream is read continuously, so a child writing more than the bound never
 blocks on a full pipe: only the first {@link DIAGNOSTIC_LIMIT_BYTES} are kept,
 and everything the child writes after them is discarded as it arrives.

 @param stream - The child's stderr, which this module always pipes

 @returns Read of the text kept so far, naming the bound when it was passed

 @example
 ```ts
 const standardErrorText = boundedStandardError({ stream: child.stderr });
 ```
 */
function boundedStandardError({ stream }: {
  readonly stream: Readable;
}): () => string {
  /**
   Chunks kept so far, in the order the child wrote them.
   */
  const chunks: Buffer[] = [];
  /**
   The reader's own counters: bytes kept, and whether the child wrote past the
   bound, so the kept text is a prefix of what it wrote.
   */
  const kept = {
    bytes: 0,
    truncated: false,
  };
  stream.on(
    'data',
    function keepWithinBound(chunk: Buffer): void {
      if (kept.truncated) {
        return;
      }
      /**
       Bytes that still fit below the bound.
       */
      const room = DIAGNOSTIC_LIMIT_BYTES - kept.bytes;
      if (chunk.byteLength > room) {
        chunks.push(chunk.subarray(
          0,
          room,
        ),);
        kept.bytes = DIAGNOSTIC_LIMIT_BYTES;
        kept.truncated = true;
        return;
      }
      chunks.push(chunk,);
      kept.bytes += chunk.byteLength;
    },
  );
  return function standardErrorText(): string {
    /**
     Everything kept, decoded as the child's own text.
     */
    const text = Buffer.concat(chunks,)
      .toString('utf8',);
    if (!kept.truncated) {
      return text;
    }
    return `${text}\n[stderr truncated after ${String(DIAGNOSTIC_LIMIT_BYTES,)} bytes]`;
  };
}

//endregion Diagnostic capture

//region Reporting

/**
 What a failed child reports about itself, read off the caught value.
 
 Every field is the empty string when the runtime did not supply it, so a
 caller reads absence as "this is not how it ended" rather than as a nullish
 branch.

 @example
 ```ts
 const failure: ChildFailure = childFailureOf(error);
 ```
 */
type ChildFailure = {
  /**
   Exit status the child reported, or the empty string when it did not exit.
   */
  readonly exitCode: string;
  /**
   Signal that ended the child, or the empty string when none did.
   */
  readonly signal: string;
  /**
   Text the child wrote to stderr, or the empty string.
   */
  readonly standardError: string;
  /**
   The runtime's own account, which explains a command that never started.
   */
  readonly message: string;
};

/**
 One command to run to completion.
 
 @example
 ```ts
 const request: RunCommandOptions = { command: 'ydotool', args: ['key', '29:1'] };
 ```
 */
export type RunCommandOptions = {
  /**
   Executable to run, resolved through `PATH` unless absolute.
   */
  readonly command: string;
  /**
   Arguments passed to the executable, never through a shell.
   */
  readonly args: readonly string[];
};

/**
 Read one field of a caught value without assuming its shape.
 
 @param source - Caught value of unknown type
 
 @param key - Field name to read
 
 @returns The field's value, or `undefined` when it cannot be read
 
 @example
 ```ts
 readField({ source: error, key: 'stderr' });
 ```
 */
function readField({
  source,
  key,
}: {
  readonly source: unknown;
  readonly key: string;
}): unknown {
  if (((typeof source) !== 'object') || (source === null)) {
    return undefined;
  }
  return Reflect.get(
    source,
    key
  );
}

/**
 Read a number field as text.
 
 @param value - Field value of unknown type
 
 @returns The number as text, or the empty string when it is not a number
 
 @example
 ```ts
 numberTextOf(7); // '7'
 ```
 */
function numberTextOf(value: unknown): string {
  return (typeof value) === 'number' ? String(value) : '';
}

/**
 Read a string field.
 
 @param value - Field value of unknown type
 
 @returns The string, or the empty string when it is not one
 
 @example
 ```ts
 textOf('boom'); // 'boom'
 ```
 */
function textOf(value: unknown): string {
  return (typeof value) === 'string' ? value : '';
}

/**
 Project the parts of a failed child this module reports.
 
 @param error - Value the command rejected with
 
 @returns Its exit status, signal, stderr and message, each possibly empty
 
 @example
 ```ts
 childFailureOf(error); // { exitCode: '7', signal: '', standardError: 'boom', message: '...' }
 ```
 */
function childFailureOf(error: unknown): ChildFailure {
  return {
    exitCode: numberTextOf(readField({
      source: error,
      key: 'code',
    })),
    signal: textOf(readField({
      source: error,
      key: 'signal',
    })),
    standardError: textOf(readField({
      source: error,
      key: 'stderr',
    })),
    message: textOf(readField({
      source: error,
      key: 'message',
    })),
  };
}

/**
 Say how one child ended, in the words an operator reads.
 
 @param failure - What the failed child reported
 
 @returns One clause naming the exit status, the signal, or the start failure
 
 @example
 ```ts
 endingTextOf({ exitCode: '7', signal: '', standardError: '', message: '' });
 // => 'exited with code 7'
 ```
 */
function endingTextOf(failure: ChildFailure): string {
  if (failure.exitCode !== '') {
    return `exited with code ${failure.exitCode}`;
  }
  if (failure.signal !== '') {
    return `was ended by ${failure.signal}`;
  }
  return `could not be started: ${failure.message}`;
}

/**
 Build the diagnostic for one failed child: the invocation, how it ended, and
 whatever it wrote to stderr.
 
 @param command - Executable that was run
 
 @param args - Arguments it was given
 
 @param failure - What the failed child reported
 
 @returns One line naming the command and its failure
 
 @example
 ```ts
 failureTextOf({ command: 'ydotool', args: ['key'], failure });
 // => "'ydotool key' exited with code 7; stderr: ydotoold is not running"
 ```
 */
function failureTextOf({
  command,
  args,
  failure,
}: {
  readonly command: string;
  readonly args: readonly string[];
  readonly failure: ChildFailure;
}): string {
  /**
   The command line an operator would retype.
   */
  const invocation = [
    command,
    ...args
  ].join(' ');
  /**
   Invocation and ending, the part every failure reports.
   */
  const headline = `'${invocation}' ${endingTextOf(failure)}`;
  /**
   stderr the child wrote, trimmed.
   */
  const output = failure.standardError
    .trim();
  return output === '' ? headline : `${headline}; stderr: ${output}`;
}

//endregion Reporting

//region Running

/**
 Run one command to completion, throwing unless it exited successfully.
 
 A successful child resolves; a nonzero exit, a signal, or a command that could
 not be started throws {@link CommandFailedError}, whose message names the
 command, how it ended, and any stderr it wrote. Output past the diagnostic
 bound is truncated rather than ending the child.
 
 @param command - Executable to run
 
 @param args - Arguments passed to it
 
 @throws {@link CommandFailedError} when the command did not exit successfully
 
 @example
 ```ts
 await runCommand({ command: 'ydotool', args: ['key', '29:1'] });
 ```
 */
export async function runCommand({
  command,
  args
}: RunCommandOptions): Promise<void> {
  /**
   Child running the command, with its stderr piped for the failure message and
   its other streams discarded.
   */
  const child = spawn(
    command,
    [...args],
    { stdio: [
      'ignore',
      'ignore',
      'pipe'
    ], },
  );
  /**
   What it wrote to stderr, drained into a bounded text.
   */
  const standardErrorText = boundedStandardError({ stream: child.stderr, });
  try {
    // `close` rather than `exit`: it fires once the piped stderr has closed too,
    // so the text read below is everything the child wrote.
    await once(
      child,
      'close',
    );
  } catch (error) {
    throw new CommandFailedError(failureTextOf({
      command,
      args,
      failure: childFailureOf(error),
    },));
  }
  if (child.exitCode === 0) {
    return;
  }
  throw new CommandFailedError(failureTextOf({
    command,
    args,
    failure: {
      exitCode: numberTextOf(child.exitCode,),
      signal: textOf(child.signalCode,),
      standardError: standardErrorText(),
      message: '',
    },
  },));
}

/**
 Start one command and return while it keeps running.
 
 The shape for an application: a launcher starts a program that outlives the
 call, so waiting for it to end would hold the daemon's promise for the whole
 session, and buffering its output would end it once the application wrote
 enough. The call therefore resolves as soon as the program has started, and
 reports a program that could not be started, or one that later ended
 unsuccessfully, through the daemon's log rather than by throwing.
 
 @param command - Executable to run
 
 @param args - Arguments passed to it
 
 @throws {@link CommandFailedError} when the command could not be started
 
 @example
 ```ts
 await startCommand({ command: 'ghostty', args: ['--gtk-single-instance=false'] });
 ```
 */
export async function startCommand({
  command,
  args
}: RunCommandOptions): Promise<void> {
  /**
   Child running the command, with its stderr piped so a later failure can say
   what it wrote.
   */
  const child = spawn(
    command,
    [...args],
    { stdio: [
      'ignore',
      'ignore',
      'pipe'
    ], },
  );
  /**
   What it wrote to stderr, drained into a bounded text.
   */
  const standardErrorText = boundedStandardError({ stream: child.stderr, });
  try {
    // `spawn` fires once the program is running, and `error` fires instead when
    // it could not be started at all.
    await once(
      child,
      'spawn',
    );
  } catch (error) {
    throw new CommandFailedError(failureTextOf({
      command,
      args,
      failure: childFailureOf(error),
    },));
  }
  // A started program is out of this module's hands: it may run for the whole
  // session, or fail later on its own. Both endings are logged, never thrown,
  // because the caller has already returned by then.
  child.on(
    'error',
    function reportLateError(error: Error): void {
      tagged({
        tag: startCommand.name,
        l: childProcessLogger,
      },)
        .error(failureTextOf({
          command,
          args,
          failure: childFailureOf(error),
        },));
    },
  );
  child.once(
    'close',
    function reportFailureOnExit(): void {
      if (child.exitCode === 0) {
        return;
      }
      tagged({
        tag: startCommand.name,
        l: childProcessLogger,
      },)
        .error(failureTextOf({
          command,
          args,
          failure: {
            exitCode: numberTextOf(child.exitCode,),
            signal: textOf(child.signalCode,),
            standardError: standardErrorText(),
            message: '',
          },
        },));
    },
  );
}

//endregion Running
