/**
 Keeps the whole of what a failed start printed beside the runs, so the one line a batch or a review starter prints can stay short
 and still be enough to find out more.

 The file appends, so a name that failed twice keeps both attempts, and it sits beside the run folders where every other record of
 a run is.

 @module
 */

import { appendFile, } from 'node:fs/promises';
import { join, } from 'node:path';

/**
 Reports one line on standard error, which is where a command's own notes go rather than into its answer.

 @param line - the line to report

 @example
 ```ts
 reportOnStandardError('the output of a could not be kept');
 ```
 */
export function reportOnStandardError(line: string,): void {
  process.stderr
    .write(`${line}\n`,);
}

/**
 Keeps the whole of what a failed start printed.

 @param runsFolder - folder the driver writes its runs into

 @param name - the run's name

 @param output - what the start printed on both streams

 @param report - where a file that could not be written is reported, so the line still says why the start failed

 @returns where the output was kept, or nothing when it could not be written

 @example
 ```ts
 const kept = await keepFailedOutput({ runsFolder: '/runs', name: 'a', output, report: function report(line) { process.stderr.write(`${line}\n`); }, },);
 ```
 */
export async function keepFailedOutput({
  runsFolder,
  name,
  output,
  report,
}: {
  readonly runsFolder: string;
  readonly name: string;
  readonly output: string;
  readonly report: (line: string) => void;
}): Promise<{ readonly keptAt?: string }> {
  /**
   The file this run's failed attempts are kept in.
   */
  const file = join(
    runsFolder,
    `${name}-start-failure.log`,
  );
  try {
    await appendFile(
      file,
      `=== ${new Date().toISOString()} ===\n${output.trimEnd()}\n`,
    );
    return { keptAt: file, };
  }
  catch (error) {
    // The line still has to say why the start failed, so a file that could not be written is reported and the line goes without it.
    report(`the output of ${name} could not be kept in ${file}: ${String(error,)}`,);
    return {};
  }
}
