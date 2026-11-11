/**
 Runs one package check with a time bound: the check's first error is found as its output streams past, wherever in
 that output it stands, the output is never copied to the journal, and the whole process group is stopped when the
 bound passes.

 @module
 */

import { spawn, } from 'node:child_process';
import { once, } from 'node:events';
import type { Readable, } from 'node:stream';

import {
  endOf,
  stopGroupAfter,
} from './daily-update-command.ts';
import {
  type FirstErrorScan,
  firstErrorScan,
  NO_ERROR_IN_OUTPUT,
} from './daily-update-first-error.ts';
import { reproduceCommand, } from './daily-update-package-check-discovery.ts';
import type {
  CheckOutcome,
  PackageCheck,
} from './daily-update-package-check-types.ts';

/**
 Characters of one output line kept while it arrives. A longer line is read up to this many and the rest of it is
 dropped, so a stream that never ends a line cannot grow what a check holds; it also bounds one excerpt line.

 The longest line of the output recorded from the run that opened issue 6284 was 902 characters; this is nine times
 that, and far above the longest line a compiler diagnostic makes.
 */
export const MAXIMUM_LINE_CHARACTERS = 8_192;

/**
 Milliseconds in one second.
 */
const MILLISECONDS_PER_SECOND = 1_000;

/**
 Tenths of a second in one second, for rounding durations to one decimal.
 */
const TENTHS_PER_SECOND = 10;

/**
 Reads a child's output streams line by line into a scan, so the first error is found wherever it stands.

 The streams are read whole: a check whose start holds megabytes of warnings naming the pnpm link farm (cargo prints
 one for every symlink loop its walk meets) still hands the report the error at its end. The returned promise resolves
 once every stream can deliver nothing more, so the scan then holds all the check printed.

 @param streams - the child's output streams, either of which may be absent

 @param scan - where the lines are read

 @throws Error when a stream fails and can carry nothing more

 @example
 ```ts
 await keepFirstError({ streams: [child.stdout, child.stderr,], scan, },);
 ```
 */
export async function keepFirstError({
  streams,
  scan,
}: {
  // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- mirrors the `node:child_process` stdio streams this reads, which are typed `Readable | null` because a stream is absent when the child was given no pipe for it
  readonly streams: readonly (Readable | null)[];
  readonly scan: FirstErrorScan;
},): Promise<void> {
  /**
   Reads one line, keeping only its start when it is longer than the bound.

   @param line - the line
   */
  function readLine(line: string,): void {
    scan.pushLine(line.length > MAXIMUM_LINE_CHARACTERS ? line.slice(
      0,
      MAXIMUM_LINE_CHARACTERS,
    ) : line,);
  }
  /**
   One promise per stream, resolved when that stream can deliver nothing more.
   */
  const ended: Promise<unknown>[] = [];
  for (const stream of streams) {
    if (stream === null)
      continue;
    // The stream decodes its own bytes, so a character split across two chunks arrives whole instead of as a stand-in.
    stream.setEncoding('utf8',);
    /**
     The line this stream has read so far; it is cut at the bound once it passes it.
     */
    const reading = { text: '', };
    stream.on(
      'data',
      function readText(chunk: string,) {
        /**
         The lines this chunk completes, and the start of the line it leaves open.
         */
        const lines = `${reading.text}${chunk}`.split('\n',);
        /**
         The start of the line the chunk leaves open; every other piece is a line of its own.
         */
        const open = lines.pop() ?? '';
        for (const line of lines) {
          readLine(line,);
        }
        // A line that never ends is kept only up to the bound, so a stream that prints one cannot grow what a check holds.
        reading.text = open.length > MAXIMUM_LINE_CHARACTERS ? open.slice(
          0,
          MAXIMUM_LINE_CHARACTERS,
        ) : open;
      },
    );
    // A pipe closes after its last byte and also when the child is stopped, so its close is what ends the read of it.
    ended.push(once(
      stream,
      'close',
    ),);
  }
  await Promise.all(ended,);
}

/**
 Runs one package check in the repository.

 @param repository - absolute repository path, the working directory

 @param check - the check

 @param environment - extra environment for the check

 @returns how it ended

 @example
 ```ts
 const outcome = await runCheckBounded({ repository: '/r', check, environment: {}, },);
 ```
 */
export async function runCheckBounded({
  repository,
  check,
  environment,
}: {
  readonly repository: string;
  readonly check: PackageCheck;
  readonly environment: Readonly<Record<string, string>>;
},): Promise<CheckOutcome> {
  /**
   When the check started.
   */
  const startedAt = Date.now();
  /**
   Program and arguments, the package's own `mise` task.
   */
  const [program = 'mise', ...args] = reproduceCommand(check,)
    .split(' ',);
  /**
   The check in its own process group, so one signal reaches everything it started.
   */
  const child = spawn(
    program,
    args,
    {
      cwd: repository,
      detached: true,
      env: {
        ...process.env,
        ...environment,
      },
      stdio: [
        'ignore',
        'pipe',
        'pipe',
      ],
    },
  );
  /**
   Where the check's first error is read as its output arrives.
   */
  const scan = firstErrorScan();
  /**
   The reading of the check's output, resolved once both streams ended.
   */
  const reading = keepFirstError({
    streams: [
      child.stdout,
      child.stderr,
    ],
    scan,
  },);
  /**
   The pending stop of the check's process group, cancelled when this function ends.
   */
  using stopper = stopGroupAfter({
    pid: child.pid ?? 0,
    timeoutMs: check.timeoutMs,
  },);
  /**
   How the check ended, read together with its output because neither waits for the other.
   */
  const [end,] = await Promise.all([
    endOf(child,),
    reading,
  ],);
  /**
   The first error the check printed, or nothing when no line of its output starts one.
   */
  const found = scan.error();
  return {
    exitCode: end.exitCode,
    timedOut: stopper.fired(),
    seconds: Math.round(((Date.now() - startedAt) / MILLISECONDS_PER_SECOND) * TENTHS_PER_SECOND,) / TENTHS_PER_SECOND,
    ...(found === NO_ERROR_IN_OUTPUT ? {} : { firstError: found, }),
  };
}
