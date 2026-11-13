/**
 Starts one Pi run through the retrying start this package ships, keeping the end of both streams.

 A start says why it failed at the end of what it printed, and a status with no reason once left a round's failures unexplained,
 so both streams are kept and the last line of them is what a caller reports.

 @module
 */

import { spawn, } from 'node:child_process';
import { once, } from 'node:events';
import {
  ENTRY_OUTPUT_KEPT_CHARACTERS,
  type StartedEntry,
} from './batch-list.ts';
import { siblingCommandEntry, } from './sibling-command-entry.ts';
import { tailKeeper, } from './tail-of-a-command.ts';

/**
 Reads the retrying start's entry beside this module's callers.

 @returns the command's path

 @example
 ```ts
 const entry = await retryingStartEntry();
 ```
 */
export async function retryingStartEntry(): Promise<string> {
  return await siblingCommandEntry({
    folder: import.meta.dirname,
    command: 'retry-pi-run-start-until-a-pool-has-room',
  },);
}

/**
 Starts one run through the retrying start, keeping both streams.

 @param entry - the retrying start's entry

 @param words - the words to give it

 @param environment - the environment to start it with

 @returns how it ended, and the end of what it printed

 @example
 ```ts
 const started = await startThroughTheRetryingStart({ entry, words: ['--deadline-seconds=7200', '--period-seconds=30', '--', '--name=a'], environment: process.env, },);
 ```
 */
export async function startThroughTheRetryingStart({
  entry,
  words,
  environment,
  report,
}: {
  readonly entry: string;
  readonly words: readonly string[];
  readonly environment: NodeJS.ProcessEnv;
  readonly report: (line: string) => void;
}): Promise<StartedEntry> {
  /**
   The retrying start's process.
   */
  const child = spawn(
    process.execPath,
    [
      entry,
      ...words,
    ],
    {
      stdio: [
        'ignore',
        'pipe',
        'pipe',
      ],
      env: environment,
    },
  );
  /**
   The end of what it printed, on either stream.
   */
  const printed = tailKeeper({ characters: ENTRY_OUTPUT_KEPT_CHARACTERS, },);
  /**
   What the retrying start said on its standard output and has not yet been passed on, which a line is cut out of.
   */
  const partial: { text: string } = { text: '', };
  child.stdout
    .on(
    'data',
    function keepAndSay(chunk: Buffer,) {
      printed.keep(chunk,);
      // An entry whose start waits for a slot says so as it happens: keeping only the end of what it printed left a coordinator
      // watching a batch's output file with nothing in it while the run waited.
      passOnLines({
        partial,
        chunk,
        report,
      },);
    },
    );
  child.stderr
    .on(
    'data',
    printed.keep,
    );
  try {
    /**
     What the process reported when it closed, which is its status or the signal that ended it.
     */
    const closed: readonly unknown[] = await once(
      child,
      'close',
    );
    /**
     The status the retrying start exited with, or nothing when a signal ended it.
     */
    const [status,] = closed;
    return {
      output: printed.text(),
      ...(((typeof status) === 'number')
        ? { status, }
        : {}),
    };
  }
  catch (error) {
    // `once` rejects when the process could not be started at all, and a caller reports that as the entry's own reason.
    return {
      output: printed.text(),
      startProblem: String(error,),
    };
  }
}

/**
 Passes on each whole line of what a command printed, keeping the unfinished end for the next read.

 @param partial - the text that has not been passed on yet, which this extends and then shortens to the unfinished end

 @param chunk - what arrived this time

 @param report - where each whole line goes

 @example
 ```ts
 passOnLines({ partial, chunk, report: process.stdout.write.bind(process.stdout) },);
 ```
 */
function passOnLines({
  partial,
  chunk,
  report,
}: {
  readonly partial: { text: string };
  readonly chunk: Buffer;
  readonly report: (line: string) => void;
},): void {
  /**
   The lines this read finished, and the unfinished end it leaves.
   */
  const lines = `${partial.text}${chunk.toString()}`.split('\n',);
  partial.text = lines.pop() ?? '';
  for (const line of lines) {
    report(`${line}\n`,);
  }
}
