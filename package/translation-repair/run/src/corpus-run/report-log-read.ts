import { readFile, } from 'node:fs/promises';
import { resolve, } from 'node:path';

import { allInInputOrder, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { failureName, } from '@monochromatic-dev/translation-repair-markup/ts';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';

//region Report log read
// Reads the log files a report command was handed, the same way for every
// report that totals logs.
//
// A LOG THAT WILL NOT READ IS A STATED REFUSAL. A path mistyped on the command
// line is the operator's to mend, and it used to leave the command as a fault,
// with no word of which log or why, since the filesystem's own message quotes
// a path this package does not forward. The refusal names the path the
// operator typed and the filesystem's code, which carries no path.
//
// A LOG NAMED TWICE IS REFUSED where its lines would be totalled. Reading the
// same file twice counts every call, round and credit in it twice, and the
// total looks exactly like a run twice as large.
//
// THE FINAL NEWLINE IS NOT A LINE. A log a run wrote ends in a newline, and a
// split on it leaves an empty piece after the last line that a count of lines
// took for one more.

/**
 Reads every named log, refusing one that will not read.

 @param paths - logs named on the command line, which the refusal repeats

 @returns Each log's text, in the order named

 @throws {@link StatedRefusalError} naming a log (the first one named on the command line that
 cannot be read, where several cannot)
 that could not be read, with the filesystem's code and no text from the
 failure's own message

 @example
 ```ts
 const texts = await readLogTexts({ paths: line.positionals, },);
 ```
 */
export async function readLogTexts(
  { paths, }: { readonly paths: readonly string[]; },
): Promise<readonly string[]> {
  return await allInInputOrder({
    members: paths.map(async function one(path,): Promise<string> {
      try {
        return await readFile(
          path,
          'utf8',
        );
      } catch (error) {
        throw new StatedRefusalError({
          says: `the log ${path} could not be read (${failureName({ error, },)}), so nothing was counted. `
            + 'Name a log a pass, probe or calibration wrote, by a path that exists.',
          cause: error,
        },);
      }
    },),
  },);
}

/**
 Refuses a command line that names one log more than once, where the logs are
 totalled as one run.

 @param paths - logs named on the command line

 @throws {@link StatedRefusalError} naming the first log named a second time,
 where two paths reach one file by the same spelling once made absolute

 @example
 ```ts
 refuseRepeatedLogs({ paths: line.positionals, },);
 ```
 */
export function refuseRepeatedLogs({ paths, }: { readonly paths: readonly string[]; },): void {
  /**
   Logs met so far, as the absolute paths they reach.
   */
  const met = new Set<string>();

  for (const path of paths) {
    /**
     Where this spelling of the path reaches from the working directory.
     */
    const reached = resolve(path,);

    if (met.has(reached,))
      throw new StatedRefusalError({
        says: `the log ${path} is named more than once, so its lines would be counted twice. `
          + 'Name each log once.',
      },);

    met.add(reached,);
  }
}

/**
 Splits a log's text into its lines, leaving out the empty piece a closing
 newline leaves after the last.

 @param text - whole log

 @returns Its lines, none for an empty log

 @example
 ```ts
 const lines = linesOfLog({ text: 'one\ntwo\n', },);
 // => ['one', 'two',]
 ```
 */
export function linesOfLog({ text, }: { readonly text: string; },): readonly string[] {
  /**
   Every piece between newlines, the last of which is empty after a closing one.
   */
  const pieces = text.split('\n',);

  if (pieces.at(-1,) === '')
    return pieces.slice(
      0,
      -1,
    );

  return pieces;
}

//endregion Report log read
