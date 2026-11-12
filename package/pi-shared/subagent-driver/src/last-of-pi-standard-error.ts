/**
 The last lines of Pi's standard error, and the reason among them: what a launcher and a batch print when Pi never got the run
 going.

 Pi writes why it would not start to its standard error and nowhere else. An extension that cannot be loaded, a provider it
 refuses, a missing native module: Pi prints the error there and exits. On 2026-11-11 thirteen runs ended that way, and every one
 of them was reported as `status 1` with the unit's memory line after it, so nothing named the cause; the run folder's
 `stderr.log` held it the whole time.

 The reason is not simply the last line: Pi ends its message with a line of advice (`Hint: Start without extensions using
 "pi -ne".`), and the advice is not why the run failed. The tail travels with the reason so a reader sees Pi's whole answer, and
 the reason is printed last, where a batch's one line picks it up.

 @module
 */

import {
  open,
  stat,
} from 'node:fs/promises';
import { errorCodeOf, } from './error-code-of.ts';
import { reasonInFailedStart, } from './reason-in-a-failed-start.ts';

//region Constants

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 How many lines of Pi's standard error are read as the tail. Pi's own error, the stack under it and its advice line fit in this;
 a longer tail would bury the error above lines a reader does not need, and the whole of the file stays beside the run as
 `stderr.log` for anyone who does.
 */
export const PI_STANDARD_ERROR_TAIL_LINES: number = 12;

/**
 How many bytes of the end of Pi's standard error are read, which is enough for far more than the lines above. A file is not read
 whole because a run that failed after hours may hold megabytes of Pi's warnings, and a launcher only ever prints the end.
 */
export const PI_STANDARD_ERROR_TAIL_BYTES: number = 16_384;

/**
 The start of Pi's own advice line, which reads `Hint: Start without extensions using "pi -ne".`. It is advice about how to go on
 without the thing that failed, never the reason a run did not start, so it is not the line a reader is given as the reason.
 */
const PI_ADVICE_LINE_START = 'Hint: ';

//endregion Constants

//region Types

/**
 What the end of Pi's standard error says.
 */
export type PiStandardErrorTail = {
  /**
   The last lines Pi printed, in the order it printed them, which empty when it printed nothing.
   */
  readonly lines: readonly string[];

  /**
   Why Pi would not go on, in its own words, or nothing when the tail names no reason.
   */
  readonly reason?: string;
};

//endregion Types

//region Reading

/**
 Reads the end of a file, which is absent when the file is.

 @param file - the file to read

 @param bytes - how many bytes of its end to read

 @returns its text, or empty text when the file is not there

 @throws Error when the file is there and cannot be read for another reason, which would hide what it holds

 @example
 ```ts
 const text = await endOfFile({ file: '/runs/a-1/stderr.log', bytes: 16_384 });
 ```
 */
async function endOfFile({
  file,
  bytes,
}: {
  readonly file: string;
  readonly bytes: number;
}): Promise<string> {
  try {
    /**
     The file, opened at its end only. The handle closes when this block ends, whichever way it ends, which is what keeps a
     launcher from holding a run's file.
     */
    await using handle = await open(
      file,
      'r',
    );
    /**
     How large the file is.
     */
    const { size, } = await handle.stat();
    /**
     Where reading starts, which is the whole file when it is shorter than the tail.
     */
    const from = Math.max(
      0,
      size - bytes,
    );
    /**
     The bytes of the end, in a buffer of their own.
     */
    const read = Buffer.alloc(size - from,);
    await handle.read(
      read,
      0,
      read.length,
      from,
    );
    return read.toString('utf8',);
  }
  catch (error) {
    // A run whose Pi never started wrote no standard error, which is the empty tail; any other failure would hide what the file
    // holds, so it is carried out.
    if (errorCodeOf(error,) !== MISSING_PATH_CODE) {
      throw error;
    }
    return '';
  }
}

/**
 Reads the last lines of Pi's standard error, and the reason among them.

 @param file - the file Pi's standard error was kept in

 @returns the last lines, and why Pi would not go on

 @example
 ```ts
 const { lines, reason } = await lastOfPiStandardError({ file: '/runs/a-1/stderr.log', });
 ```
 */
export async function lastOfPiStandardError({ file, }: { readonly file: string },): Promise<PiStandardErrorTail> {
  /**
   The end of the file, of which only the last lines are wanted.
   */
  const text = await endOfFile({
    file,
    bytes: PI_STANDARD_ERROR_TAIL_BYTES,
  },);
  /**
   The lines it holds, without the empty ones a file ends with.
   */
  const written = text.split('\n',)
    .filter(function isNotBlank(line,) {
      return line.trim() !== '';
    },);
  /**
   The last lines Pi printed.
   */
  const lines = written.slice(-PI_STANDARD_ERROR_TAIL_LINES,);
  /**
   The lines that could be a reason, which leave out Pi's advice about going on without what failed.
   */
  const reasons = lines.filter(function isNotAdvice(line,) {
    return !line.trim()
      .startsWith(PI_ADVICE_LINE_START,);
  },);
  return {
    lines,
    ...reasonInFailedStart({ output: reasons.join('\n',), },),
  };
}

//endregion Reading
