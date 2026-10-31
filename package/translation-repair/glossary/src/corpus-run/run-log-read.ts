import { createReadStream, } from 'node:fs';
import { join, } from 'node:path';

import { _neutralizeControlCharacters as neutralizeControlCharacters, } from '@monochromatic-dev/logger-core/ts';
import {
  allInInputOrder,
  isJsonRecord,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import { textsInCodePointOrder, } from '@monochromatic-dev/translation-repair-markup/ts';
import {
  RUN_LOG_DIRECTORY,
  RUN_LOG_SUFFIX,
} from '@monochromatic-dev/translation-repair-typography/ts';
import { StatedRefusalError, } from '../stated-refusal.ts';
import {
  namesOfKind,
  readingOf,
} from './directory-listing.ts';

//region Run log read
// THE READING HALF OF `typography/src/run-log.ts`: the records a run kept, handed back as the lines the report parsers
// were written against.
//
// A RECORD IS READ AS THE LOGGER WROTE IT OR NOT AT ALL AS A RECORD. A line is a record only when it is a JSON object
// with exactly `level`, `message` and `timestamp`, the level one the logger has, the message a string and the
// timestamp a whole number of milliseconds a date can hold. Anything else (a line a crash cut off, a foreign line, an
// object with a field the sink never writes) is handed on as it stands, whole, so the report that reads it counts it
// as the damaged or foreign line it is and no count of lines is quietly one short.
//
// A RECORD IS RENDERED AS THE CONSOLE SINK PRINTS IT, `[level] [ISO instant] message`, which is the text every report
// parser here was written against, so each parser stays as it was and a report over records cannot drift from a
// report over the same lines as text. That includes the escaping: the console sink writes control characters other than
// the line feed and the tab (C0, DEL and C1) as `\uXXXX`, and so does this, through the logger's own function for it
// (ledger B1118), so a terminal that is handed the rendering never receives a control sequence a message carried.
//
// THE CONSOLE DID NOT SHOW DEBUG AND TRACE RECORDS unless a run was verbose, and the file sink records them always. A
// reader hands the parsers what the console showed, so every count a report makes is the count it made over a
// console capture, and a debug line that quotes a marker cannot be counted as the line it quotes; a caller that wants
// the evidence in full asks for the verbose records (ledger B1117).
//
// WHERE A RUN'S RECORDS ARE: `<run folder>/log/<UTC day>/<timestamp>.<uuid>.log.jsonl`, one file for each process that
// opened the run log. A reader is handed the run folder, its `log` directory, a day directory or one file, and finds the
// records under it; the files are read in code point order of their names, which is the order the processes started.

/**
 Levels a record may carry, as the logger has them.
 */
const LEVELS: ReadonlySet<string> = new Set([
  'debug',
  'error',
  'fatal',
  'info',
  'trace',
  'warn',
],);

/**
 Levels the console sink drops unless a run is verbose, so a reader drops them unless it is asked for them.
 */
const VERBOSE_LEVELS: ReadonlySet<string> = new Set([
  'debug',
  'trace',
],);

/**
 Names a record carries and no other.
 */
const RECORD_FIELDS: readonly string[] = [
  'level',
  'message',
  'timestamp',
];

/**
 What `indexOf` returns for text that is not there.
 */
const NOT_FOUND = -1;

/**
 Latest instant, in epoch milliseconds, a date can hold in either direction.
 */
const MAX_DATE_MS = 8_640_000_000_000_000;

/**
 Characters of a day directory's name, `YYYY-MM-DD`.
 */
const DAY_NAME_LENGTH = 10;

/**
 Place of the hyphen after the year in a day directory's name.
 */
const YEAR_END = 4;

/**
 Place of the hyphen after the month in a day directory's name.
 */
const MONTH_END = 7;

/**
 One record as the logger wrote it.

 @example
 ```ts
 const record: RunLogRecord = { level: 'info', message: 'purr', timestamp: 0, };
 ```
 */
export type RunLogRecord = {
  /**
   Level the line was logged at.
   */
  readonly level: string;

  /**
   Text of the line, tags and all.
   */
  readonly message: string;

  /**
   Epoch milliseconds the line was logged at.
   */
  readonly timestamp: number;
};

/**
 What reading a line as JSON gave.
 */
type JsonReading =
  | {
    readonly kind: 'json';
    readonly value: unknown;
  }
    | {
      readonly kind: 'not-json';
      readonly cause: unknown;
    };

/**
 Reads a line as JSON.

 @param line - one line of the file

 @returns What it holds, or that it is not JSON with the parser's complaint

 @example
 ```ts
 const reading = jsonOf({ line: '{"a":1}', },);
 ```
 */
function jsonOf({ line, }: { readonly line: string; },): JsonReading {
  try {
    return {
      kind: 'json',
      value: JSON.parse(line,),
    };
  } catch (error) {
    // A LINE A CRASH CUT OFF, OR A FOREIGN ONE: handed on whole by the caller to be counted as the line it is.
    return {
      kind: 'not-json',
      cause: error,
    };
  }
}

/**
 Whether a directory name is a day the sink named, `YYYY-MM-DD`, read by position and not by pattern.

 @param name - name of a directory entry

 @returns Whether it is four digits, a hyphen, two digits, a hyphen and two digits

 @example
 ```ts
 isDayDirectoryName({ name: '2026-10-31', },); // true
 ```
 */
function isDayDirectoryName({ name, }: { readonly name: string; },): boolean {
  if (name.length !== DAY_NAME_LENGTH)
    return false;
  for (let index = 0; index < name.length; index += 1) {
    /**
     Character at this place.
     */
    const character = name.charAt(index,);
    if ((index === YEAR_END) || (index === MONTH_END)) {
      if (character !== '-')
        return false;
    }
    else if ((character < '0') || (character > '9'))
      return false;
  }
  return true;
}

/**
 Reads one line of a run log file as a record.

 @param line - one line of the file, without its line feed

 @returns Record, or that the line is not exactly one: a JSON object of `level`, `message` and `timestamp` alone,
 the level one the logger has, the message a string and the timestamp a whole number of milliseconds a date can hold

 @example
 ```ts
 const record = parseRunLogRecord({ line: '{"level":"info","message":"purr","timestamp":0}', },);
 ```
 */
export function parseRunLogRecord({ line, }: { readonly line: string; },): RunLogRecord | 'not-a-record' {
  /**
   The line read as JSON.
   */
  const reading = jsonOf({ line, },);
  if ((reading.kind === 'not-json') || (!isJsonRecord(reading.value,)))
    return 'not-a-record';
  /**
   Fields the object carries, by name.
   */
  const fields = new Map<string, unknown>(Object.entries(reading.value,),);
  if ((fields.size !== RECORD_FIELDS.length) || (!RECORD_FIELDS.every(function carried(name,): boolean {
    return fields.has(name,);
  },)))
    return 'not-a-record';
  /**
   Level the record names.
   */
  const level = fields.get('level',);
  /**
   Text the record carries.
   */
  const message = fields.get('message',);
  /**
   Instant the record names.
   */
  const timestamp = fields.get('timestamp',);
  if (((typeof level) !== 'string') || (!LEVELS.has(level,)))
    return 'not-a-record';
  if ((typeof message) !== 'string')
    return 'not-a-record';
  if (((typeof timestamp) !== 'number') || (!Number.isSafeInteger(timestamp,))
    || (Math.abs(timestamp,) > MAX_DATE_MS))
    return 'not-a-record';
  return {
    level,
    message,
    timestamp,
  };
}

/**
 Renders one line of a run log file.

 @param line - one line of the file, without its line feed

 @returns Record as `[level] [ISO instant] message`, its message's control characters escaped as the console sink
 escapes them, or the line itself when it is not exactly a record

 @example
 ```ts
 renderRunLogLine({ line: '{"level":"info","message":"purr","timestamp":0}', },); // '[info] [1970-01-01T00:00:00.000Z] purr'
 ```
 */
export function renderRunLogLine({ line, }: { readonly line: string; },): string {
  /**
   The line as a record, if it is one.
   */
  const record = parseRunLogRecord({ line, },);
  if (record === 'not-a-record')
    return line;
  return renderRecord({ record, },);
}

/**
 Renders one record as the console sink prints it, the message's control characters escaped as the sink escapes them.

 @param record - record read

 @returns `[level] [ISO instant] message`

 @example
 ```ts
 renderRecord({ record: { level: 'info', message: 'purr', timestamp: 0, }, },); // '[info] [1970-01-01T00:00:00.000Z] purr'
 ```
 */
function renderRecord({ record, }: { readonly record: RunLogRecord; },): string {
  return `[${record.level}] [${new Date(record.timestamp,).toISOString()}] ${
    neutralizeControlCharacters(record.message,)
  }`;
}

/**
 What a directory holds that a run's records could be in.
 */
type PlaceListing = {
  /**
   Names of its regular files.
   */
  readonly files: readonly string[];

  /**
   Names of its directories.
   */
  readonly directories: readonly string[];
};

/**
 Lists a directory's files and directories.

 @param dir - directory to list

 @returns Its files and directories, or that it cannot be listed because it is not there, is no directory or may not be read

 @example
 ```ts
 const listing = await listingOf({ dir: '/runs/corpus-pass-tabby', },);
 ```
 */
async function listingOf({ dir, }: { readonly dir: string; },): Promise<PlaceListing | 'not-listed'> {
  /**
   Regular files of the directory.
   */
  const files = await readingOf({
    list: async function listFiles(): Promise<readonly string[]> {
      return await namesOfKind({
        dir,
        kind: 'file',
      },);
    },
  },);
  /**
   Directories of the directory.
   */
  const directories = await readingOf({
    list: async function listDirectories(): Promise<readonly string[]> {
      return await namesOfKind({
        dir,
        kind: 'directory',
      },);
    },
  },);
  if ((files.kind === 'unreadable') || (directories.kind === 'unreadable'))
    return 'not-listed';
  return {
    files: files.names,
    directories: directories.names,
  };
}

/**
 Day directories among the directories of a listing, named by a path, in code point order.

 @param dir - directory the listing is of

 @param listing - its listing

 @returns Paths of the directories named as the sink names a day

 @example
 ```ts
 const days = dayDirectoriesOf({ dir, listing, },);
 ```
 */
function dayDirectoriesOf(
  {
    dir,
    listing,
  }: {
    readonly dir: string;
    readonly listing: PlaceListing;
  },
): readonly string[] {
  return textsInCodePointOrder({
    texts: listing.directories
      .filter(function isDay(name,): boolean {
        return isDayDirectoryName({ name, },);
      },),
  },)
    .map(function inDir(name,): string {
      return join(
        dir,
        name,
      );
    },);
}

/**
 Record files among the files of a listing, named by a path, in code point order.

 @param dir - directory the listing is of

 @param listing - its listing

 @returns Paths of the files ending the sink's suffix

 @example
 ```ts
 const files = recordFilesOf({ dir, listing, },);
 ```
 */
function recordFilesOf(
  {
    dir,
    listing,
  }: {
    readonly dir: string;
    readonly listing: PlaceListing;
  },
): readonly string[] {
  return textsInCodePointOrder({
    texts: listing.files
      .filter(function isRecordFile(name,): boolean {
        return name.endsWith(RUN_LOG_SUFFIX,);
      },),
  },)
    .map(function inDir(name,): string {
      return join(
        dir,
        name,
      );
    },);
}

/**
 Files holding a run's records under a path.

 @param path - run folder, its `log` directory, a day directory or one file

 @returns Record files: those in the named directory, then those in the day directories it holds, then those in the
 day directories of its `log` directory, each group in code point order

 @throws {@link StatedRefusalError} when the path is neither a directory nor a `.log.jsonl` file, or when a directory holds
 no record file

 @example
 ```ts
 const files = await runLogFilesUnder({ path: '/runs/corpus-pass-tabby', },);
 ```
 */
export async function runLogFilesUnder({ path, }: { readonly path: string; },): Promise<readonly string[]> {
  if (path.endsWith(RUN_LOG_SUFFIX,))
    return [path,];

  /**
   What the named directory holds, or that it cannot be listed.
   */
  const top = await listingOf({ dir: path, },);
  if (top === 'not-listed')
    throw new StatedRefusalError({
      says: `${path} is neither a run folder nor a file ending ${RUN_LOG_SUFFIX}. A pass writes its records to `
        + `<run folder>/${RUN_LOG_DIRECTORY}/<day>/*${RUN_LOG_SUFFIX}, so name that run folder or one of those files `
        + '(a .log text capture of the console is not read). Nothing was counted.',
    },);

  /**
   Where the run folder's `log` directory would be.
   */
  const logDirectory = join(
    path,
    RUN_LOG_DIRECTORY,
  );
  /**
   What the `log` directory holds, or that the named directory has none.
   */
  const inLog = top.directories
    .includes(RUN_LOG_DIRECTORY,) ? await listingOf({ dir: logDirectory, },) : 'not-listed';

  /**
   Directories whose files are record files, in the order they are read.
   */
  const places: readonly string[] = [
    path,
    ...dayDirectoriesOf({
      dir: path,
      listing: top,
    },),
    ...((inLog === 'not-listed') ? [] : dayDirectoriesOf({
      dir: logDirectory,
      listing: inLog,
    },)),
  ];

  /**
   Record files of every place, in the order of the places.
   */
  const files = (await allInInputOrder({
    members: places.map(async function filesIn(place,): Promise<readonly string[]> {
      /**
       What the place holds.
       */
      const listing = (place === path) ? top : await listingOf({ dir: place, },);
      return (listing === 'not-listed') ? [] : recordFilesOf({
        dir: place,
        listing,
      },);
    },),
  },)).flat();

  if (files.length === 0)
    throw new StatedRefusalError({
      says: `${path} holds no record file (${RUN_LOG_DIRECTORY}/<day>/*${RUN_LOG_SUFFIX}), so there is nothing to `
        + 'count. Name the run folder of a pass that ran, or one of its record files. Nothing was counted.',
    },);
  return files;
}

/**
 Lines of one record file, split on the line feed alone, one at a time.

 @param file - record file

 @returns Each line without its line feed, a last line no line feed ends included

 @example
 ```ts
 for await (const line of linesOfFile({ file, },))
   count += 1;
 ```
 */
async function* linesOfFile({ file, }: { readonly file: string; },): AsyncGenerator<string> {
  /**
   Pieces of the line a chunk ended in the middle of.
   */
  const unfinished: string[] = [];
  for await (const chunk of createReadStream(
    file,
    { encoding: 'utf8', },
  )) {
    if ((typeof chunk) !== 'string')
      throw new TypeError(`the stream of ${file} gave a ${typeof chunk} and not text`,);
    /**
     Where the next line of this chunk starts.
     */
    let start = 0;
    for (let end = chunk.indexOf('\n',); end !== NOT_FOUND; end = chunk.indexOf(
      '\n',
      start,
    )) {
      unfinished.push(chunk.slice(
        start,
        end,
      ),);
      yield unfinished.join('',);
      unfinished.length = 0;
      start = end + 1;
    }
    if (start < chunk.length)
      unfinished.push(chunk.slice(start,),);
  }
  // A LAST LINE NO LINE FEED ENDS is a line too: a process killed mid-write leaves one.
  if (unfinished.length > 0)
    yield unfinished.join('',);
}

/**
 Reads a run's records one line at a time, as the lines the report parsers read.

 NO FILE IS HELD WHOLE: each is read in chunks and cut into lines as they come, so what stands in memory is a chunk and a
 line however large the run's records are, and a caller that stops asking stops the read.

 @param path - run folder, its `log` directory, a day directory or one file

 @param verbose - whether to hand on the debug and trace records too, which the console showed only for a verbose run

 @returns One line for each record the console would have shown (or every record when verbose), a cut or foreign line
 as it stands, in the order of the files

 @throws {@link StatedRefusalError} when no record file can be found under the path

 @example
 ```ts
 for await (const line of readRunLogLines({ path: '/runs/corpus-pass-tabby', },))
   console.log(line,);
 ```
 */
export async function* readRunLogLines(
  {
    path,
    verbose = false,
  }: {
    readonly path: string;
    readonly verbose?: boolean;
  },
): AsyncGenerator<string> {
  for (const file of await runLogFilesUnder({ path, },)) {
    /* oxlint-disable no-await-in-loop -- one file after the other, so the lines come in the order of the files and no later file is opened before an earlier one is read out */
    for await (const line of linesOfFile({ file, },)) {
      /**
       The line as a record, if it is one.
       */
      const record = parseRunLogRecord({ line, },);
      if (record === 'not-a-record')
        yield line;
      else if (verbose || (!VERBOSE_LEVELS.has(record.level,)))
        yield renderRecord({ record, },);
    }
    /* oxlint-enable no-await-in-loop */
  }
}

/**
 Reads a run's records as the text of the lines the report parsers read.

 @param path - run folder, its `log` directory, a day directory or one file

 @param verbose - whether to include the debug and trace records, which the console showed only for a verbose run

 @returns One line for each record the console would have shown, a cut or foreign line as it stands, each ended by a line
 feed

 @throws {@link StatedRefusalError} when no record file can be found under the path

 @example
 ```ts
 const text = await readRunLogText({ path: '/runs/corpus-pass-tabby', },);
 ```
 */
export async function readRunLogText(
  {
    path,
    verbose = false,
  }: {
    readonly path: string;
    readonly verbose?: boolean;
  },
): Promise<string> {
  /**
   Rendered lines, each ended by a line feed.
   */
  const rendered: string[] = [];
  for await (const line of readRunLogLines({
    path,
    verbose,
  },))
    rendered.push(`${line}\n`,);
  return rendered.join('',);
}

//endregion Run log read
