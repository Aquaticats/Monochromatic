/**
 Copies Pi's standard error to a file for as long as Pi writes it.

 @module
 */

import { createWriteStream, } from 'node:fs';
import type { Readable, } from 'node:stream';
import { pipeline, } from 'node:stream/promises';

/**
 Copies Pi's standard error to a file, reporting a failure rather than throwing it, so the copy can run beside the run itself.

 @param source - Pi's standard error

 @param file - file that receives it

 @param onProblem - where to report a failure

 @param append - whether the copy is added to what the file holds, which a run whose Pi is started again needs so no start's
 lines replace another's

 @example
 ```ts
 const kept = keepStandardError({ source: child.stderr, file: 'stderr.log', onProblem: report, },);
 ```
 */
export async function keepStandardError(
  {
    source,
    file,
    onProblem,
    append = false,
  }: {
    readonly source: Readable;
    readonly file: string;
    readonly onProblem: (problem: string,) => void;
    readonly append?: boolean;
  },
): Promise<void> {
  try {
    await pipeline(
      source,
      createWriteStream(
        file,
        { flags: append ? 'a' : 'w', },
      ),
    );
  }
  catch (error) {
    onProblem(`could not keep Pi's standard error: ${String(error,)}`,);
  }
}
