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
