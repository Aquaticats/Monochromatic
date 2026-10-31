import { realpath, } from 'node:fs/promises';
import {
  basename,
  resolve,
} from 'node:path';

import { allInInputOrder, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { failureName, } from '@monochromatic-dev/translation-repair-markup/ts';
import {
  readRunLogLines,
  readRunLogText,
  runLogFilesUnder,
  StatedRefusalError,
} from '@monochromatic-dev/translation-repair-glossary/ts';

//region Report log read
// Reads the records a report command was handed, the same way for every
// report that totals a run's records.
//
// WHAT IS READ IS A RUN'S RECORDS, never a capture of its console. A pass writes them through the shared logger's file
// sink into `<run folder>/log/<day>/*.log.jsonl` (`typography/src/run-log.ts`), and a path named here is the run
// folder or one of those files. Each record is handed to the report parsers as the line the console sink would have
// printed for it, so a parser reads a record exactly as it read the text, and a line that is not a record (a cut
// one, a foreign one) reaches it as it stands (`glossary/src/corpus-run/run-log-read.ts`).
//
// A LOG THAT WILL NOT READ IS A STATED REFUSAL. A path mistyped on the command
// line is the operator's to mend, and it used to leave the command as a fault,
// with no word of which log or why, since the filesystem's own message quotes
// a path this package does not forward. The refusal names the path the
// operator typed and the filesystem's code, which carries no path. A folder
// that holds no record file is refused in the same way, in its own words: an
// empty report would read as a run that did nothing.
//
// A LOG NAMED TWICE IS REFUSED where its lines would be totalled, by the files it reaches and not by how it is
// spelled. Reading the same file twice counts every call, round and credit in it twice, and the total looks exactly
// like a run twice as large; a run folder and its `log` directory, or a folder and one of its record files, reach the
// same files by two spellings (ledger B1119).
//
// THE FINAL NEWLINE IS NOT A LINE. A log a run wrote ends in a newline, and a
// split on it leaves an empty piece after the last line that a count of lines
// took for one more.

/**
 The refusal for a log that would not read, in the words every reader of records uses.

 @param path - run folder or record file named on the command line

 @param error - what the read raised

 @returns The refusal to throw: the one the read raised when it is a stated refusal, else a stated refusal naming the
 path and the failure's name

 @example
 ```ts
 throw unreadableLog({ path, error, },);
 ```
 */
function unreadableLog(
  {
    path,
    error,
  }: {
    readonly path: string;
    readonly error: unknown;
  },
): StatedRefusalError {
  // A PATH THAT IS NO RUN FOLDER, OR A FOLDER WITH NO RECORD FILE, was refused in words of its own.
  if (error instanceof StatedRefusalError)
    return error;
  return new StatedRefusalError({
    says: `the log ${path} could not be read (${failureName({ error, },)}), so nothing was counted. `
      + 'Name a run folder a pass wrote, or one of its .log.jsonl record files, by a path that exists.',
    cause: error,
  },);
}

/**
 Reads the records of every named run folder or record file, refusing one that will not read.

 @param paths - run folders or record files named on the command line, which the refusal repeats

 @returns Each one's records as the lines the console printed, in the order named

 @throws {@link StatedRefusalError} naming a path (the first one named on the command line that
 cannot be read, where several cannot)
 that could not be read, with the filesystem's code and no text from the
 failure's own message, or a path that is no run folder and no record file, or a folder that holds no record file

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
        return await readRunLogText({ path, },);
      } catch (error) {
        throw unreadableLog({
          path,
          error,
        },);
      }
    },),
  },);
}

/**
 Streams the records of every named run folder or record file as the lines the console printed, one at a time.

 @param paths - run folders or record files named on the command line, which the refusal repeats

 @param verbose - whether to include the debug and trace records, which the console showed only for a verbose run

 @returns The lines of each one's records, in the order named

 @throws {@link StatedRefusalError} as {@link readLogTexts} does

 @example
 ```ts
 for await (const line of streamLogLines({ paths: line.positionals, },))
   console.log(line,);
 ```
 */
export async function* streamLogLines(
  {
    paths,
    verbose = false,
  }: {
    readonly paths: readonly string[];
    readonly verbose?: boolean;
  },
): AsyncGenerator<string> {
  for (const path of paths) {
    try {
      yield* readRunLogLines({
        path,
        verbose,
      },);
    } catch (error) {
      throw unreadableLog({
        path,
        error,
      },);
    }
  }
}

/**
 Reads the records of every named run folder or record file as one list of the lines the console printed.

 NO TEXT OF A WHOLE LOG IS BUILT: the lines are read one at a time and kept in the list the report counts over, so a
 report that counts lines holds them once.

 @param paths - run folders or record files named on the command line, which the refusal repeats

 @returns The lines of each one's records, in the order named

 @throws {@link StatedRefusalError} as {@link readLogTexts} does

 @example
 ```ts
 const lines = await readLogLines({ paths: line.positionals, },);
 ```
 */
export async function readLogLines(
  { paths, }: { readonly paths: readonly string[]; },
): Promise<readonly string[]> {
  /**
   Lines read so far, in the order the logs were named.
   */
  const lines: string[] = [];
  for await (const line of streamLogLines({ paths, },))
    lines.push(line,);
  return lines;
}

/**
 Record files a named log holds, refusing one that will not list as every reader of records does.

 @param path - run folder or record file named on the command line

 @returns Its record files

 @throws {@link StatedRefusalError} as {@link readLogTexts} does

 @example
 ```ts
 const files = await recordFilesNamed({ path, },);
 ```
 */
async function recordFilesNamed({ path, }: { readonly path: string; },): Promise<readonly string[]> {
  try {
    return await runLogFilesUnder({ path, },);
  } catch (error) {
    throw unreadableLog({
      path,
      error,
    },);
  }
}

/**
 The real file a record file is: past every link and every spelling of its directory.

 @param file - record file a named log holds

 @param path - the log that holds it, which the refusal names

 @returns Its real path

 @throws {@link StatedRefusalError} as {@link readLogTexts} does when the file is not there

 @example
 ```ts
 const real = await realFileOf({ file, path, },);
 ```
 */
async function realFileOf(
  {
    file,
    path,
  }: {
    readonly file: string;
    readonly path: string;
  },
): Promise<string> {
  try {
    return await realpath(file,);
  } catch (error) {
    throw unreadableLog({
      path,
      error,
    },);
  }
}

/**
 Refuses a command line that names one log more than once, where the logs are
 totalled as one run: by one spelling, or by two that reach the same record file (a run folder and its `log`
 directory, a run folder and one of its files, a link and its target).

 @param paths - logs named on the command line

 @throws {@link StatedRefusalError} naming the first log named a second time, or the first one that reaches a record
 file an earlier one reaches, and naming a log that cannot be read as {@link readLogTexts} does

 @example
 ```ts
 await refuseRepeatedLogs({ paths: line.positionals, },);
 ```
 */
export async function refuseRepeatedLogs({ paths, }: { readonly paths: readonly string[]; },): Promise<void> {
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

  /**
   Real record files met so far, each with the log that named it first.
   */
  const reachedBy = new Map<string, string>();

  for (const path of paths) {
    /**
     Record files under the path, as `readLogTexts` would find them.
     */
    // oxlint-disable-next-line no-await-in-loop -- in the order named, so the refusal names the first log that overlaps an earlier one
    const files = await recordFilesNamed({ path, },);
    for (const file of files) {
      /**
       The file itself, past every link and every spelling of its directory.
       */
      // oxlint-disable-next-line no-await-in-loop -- in the order named, so the refusal names the first log that overlaps an earlier one
      const real = await realFileOf({
        file,
        path,
      },);
      /**
       The log that reached this file first.
       */
      const first = reachedBy.get(real,);
      if (first !== undefined)
        throw new StatedRefusalError({
          says: `the log ${path} reaches the record file ${basename(real,)}, which the log ${first} reaches too, so `
            + 'its lines would be counted twice. Name each run folder or record file once, and not a run folder '
            + 'together with its log directory or one of its files.',
        },);
      reachedBy.set(
        real,
        path,
      );
    }
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
