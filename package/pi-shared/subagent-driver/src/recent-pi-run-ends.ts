/**
 The runs that finished lately, with how each ended, what the model said when it ended on an error, how many turns it took, and
 whether a later attempt of the same run exists.

 This is what a coordinator reads after a provider incident: the runs that just ended, which of them the provider ended, and
 which of those were started again.

 @module
 */

import {
  readdir,
  readFile,
  stat,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { errorCodeOf, } from './error-code-of.ts';
import { defaultRunsFolder, } from './pi-runs-folders.ts';
import { isWholeNumberText, } from './whole-number-text.ts';

/**
 The summary a finished run leaves last.
 */
const SUMMARY_FILE = 'summary.json';

/**
 Milliseconds in a minute, which the window is measured in.
 */
const MILLISECONDS_PER_MINUTE = 60_000;

/**
 The minutes of finished runs a reader sees when the caller names no window, which is one incident's worth.
 */
export const DEFAULT_SINCE_MINUTES = 40;

/**
 The command's command line, for the error message.
 */
export const RUN_ENDS_USAGE =
  'list-recent-pi-run-ends [--since-minutes=<n>] [--end=<reason>]... [--runs-folder=<folder>]';

/**
 One run that finished lately, as the reader reports it.
 */
export type RecentRunEnd = {
  /**
   The run folder's name.
   */
  readonly folder: string;

  /**
   How the run ended, as its summary says.
   */
  readonly endReason: string;

  /**
   What the model said, when the run ended on an error.
   */
  readonly modelError: string;

  /**
   How many turns the run took.
   */
  readonly turns: string;

  /**
   Whether a later attempt of the same run exists.
   */
  readonly laterRun: boolean;
};

/**
 A command line the reader cannot use.
 */
export class RunEndOptionsError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'RunEndOptionsError';
}

/**
 What one run of the command was asked for.
 */
export type RunEndOptions = {
  /**
   Folder the driver writes one run folder per run into.
   */
  readonly runsFolder: string;

  /**
   How many minutes back a run must have finished to be listed.
   */
  readonly sinceMinutes: number;

  /**
   The ends to list, empty for every end.
   */
  readonly endReasons: readonly string[];
};

/**
 Reads every value one option of this command was given, in the order written.

 @param commandLine - arguments after the program name

 @param name - the option's name, without the leading dashes

 @returns the values, empty when the option was not given

 @example
 ```ts
 const reasons = valuesOf({ commandLine: ['--end=model-error',], name: 'end', });
 ```
 */
function valuesOf({
  commandLine,
  name,
}: {
  readonly commandLine: readonly string[];
  readonly name: string;
},): readonly string[] {
  /**
   The prefix one value of this option is written with.
   */
  const prefix = `--${name}=`;
  return commandLine.filter(function isThisOption(argument,) {
    return argument.startsWith(prefix,);
  },)
    .map(function asValue(argument,) {
      return argument.slice(prefix.length,);
    },);
}

/**
 Reads the command line.

 @param commandLine - arguments after the program name

 @returns what the run was asked for

 @throws RunEndOptionsError when an option is unknown or unusable

 @example
 ```ts
 const options = parseRunEndOptions(['--since-minutes=10',],);
 ```
 */
export function parseRunEndOptions(commandLine: readonly string[],): RunEndOptions {
  /**
   The option this command knows, which is every argument that names one of them.
   */
  const known = [
    'since-minutes',
    'end',
    'runs-folder',
  ];
  if (commandLine.some(function isNoOptionOfThisCommand(argument,) {
    return argument.startsWith('--',)
      && (!known.some(function isKnown(name,) {
        return argument.startsWith(`--${name}=`,);
      },));
  },)) {
    throw new RunEndOptionsError(`an argument starting with -- is no option of this command, which takes ${known.join(', ',)}.\nUsage: ${RUN_ENDS_USAGE}`,);
  }
  /**
   The runs folder named, when it was named.
   */
  const runsFolder = valuesOf({
    commandLine,
    name: 'runs-folder',
  },);
  /**
   The minutes back named, when they were named.
   */
  const since = valuesOf({
    commandLine,
    name: 'since-minutes',
  },);
  /**
   How many minutes back a run must have finished.
   */
  const sinceMinutes = (since[0] === undefined) ? DEFAULT_SINCE_MINUTES : Number(since[0],);
  if ((!Number.isFinite(sinceMinutes,)) || (sinceMinutes <= 0)) {
    throw new RunEndOptionsError(`--since-minutes must be a number above zero; it was "${String(since[0],)}".\nUsage: ${RUN_ENDS_USAGE}`,);
  }
  return {
    runsFolder: runsFolder[0] ?? defaultRunsFolder({},),
    sinceMinutes,
    endReasons: valuesOf({
      commandLine,
      name: 'end',
    },),
  };
}

/**
 A run folder's name that names no attempt of a run, as a symbol so no name can be mistaken for it.
 */
const NOT_AN_ATTEMPT: unique symbol = Symbol('the folder name is no attempt of a run',);

/**
 Reads the run's name and attempt number from its folder's name.

 @param folder - the run folder's name

 @returns the run's name and the attempt's number, or {@link NOT_AN_ATTEMPT}

 @example
 ```ts
 const attempt = attemptOf({ folder: 'copy-route-3', });
 ```
 */
function attemptOf({
  folder,
}: {
  readonly folder: string;
},): {
  readonly name: string;
  readonly run: number
} | typeof NOT_AN_ATTEMPT {
  /**
   Where the attempt's number starts, which is after the last separator.
   */
  const separator = folder.lastIndexOf('-',);
  if ((separator <= 0) || (!isWholeNumberText({ text: folder.slice(separator + 1,), }))) {
    return NOT_AN_ATTEMPT;
  }
  return {
    name: folder.slice(
      0,
      separator,
    ),
    run: Number(folder.slice(separator + 1,),),
  };
}

/**
 No summary to read within the window, as a symbol so no summary can be mistaken for it.
 */
const NOT_A_SUMMARY: unique symbol = Symbol('Absent: no summary inside the window',);

/**
 The fields of a summary this reader reports.
 */
type SummaryFields = {
  /**
   How the run ended.
   */
  readonly endReason?: unknown;

  /**
   What the model said, when the run ended on an error.
   */
  readonly modelError?: unknown;

  /**
   How many turns the run took.
   */
  readonly turns?: unknown;
};

/**
 Reads what a summary says, when it is there and finished within the window.

 @param runsFolder - the folder the driver writes one run folder per run into

 @param folder - the entry's name

 @param cutoff - the moment the window starts, as milliseconds since the epoch

 @returns what the summary says, or {@link NOT_A_SUMMARY}

 @throws the error a read gave for a summary that is there and cannot be read

 @example
 ```ts
 const summary = await summaryWithinWindowOf({ runsFolder: '/runs', folder: 'a-1', cutoff: 0, });
 ```
 */
async function summaryWithinWindowOf({
  runsFolder,
  folder,
  cutoff,
}: {
  readonly runsFolder: string;
  readonly folder: string;
  readonly cutoff: number;
},): Promise<SummaryFields | typeof NOT_A_SUMMARY> {
  /**
   Where this run's summary would be.
   */
  const file = join(
    runsFolder,
    folder,
    SUMMARY_FILE,
  );
  try {
    /**
     The summary's own time, which says when the run finished.
     */
    const stats = await stat(file,);
    if (stats.mtimeMs < cutoff) {
      return NOT_A_SUMMARY;
    }
    return JSON.parse(await readFile(
      file,
      { encoding: 'utf8', },
    ),) as unknown;
  }
  catch (error) {
    // A run still going has written no summary yet, which is not one that finished.
    if (errorCodeOf(error,) === 'ENOENT') {
      return NOT_A_SUMMARY;
    }
    throw error;
  }
}

/**
 Reads one entry of the runs folder, when it finished within the window.

 @param runsFolder - the folder the driver writes one run folder per run into

 @param folder - the entry's name

 @param cutoff - the moment the window starts, as milliseconds since the epoch

 @returns the run's end, or {@link NOT_A_FINISHED_RUN}

 @throws the error a read gave for a summary that is there and cannot be read

 @example
 ```ts
 const end = await recentEndOf({ runsFolder: '/runs', folder: 'a-1', cutoff: 0, });
 ```
 */
async function recentEndOf({
  runsFolder,
  folder,
  cutoff,
}: {
  readonly runsFolder: string;
  readonly folder: string;
  readonly cutoff: number;
},): Promise<RecentRunEnd | typeof NOT_A_FINISHED_RUN> {
  /**
   What the entry's summary says, when it has one within the window.
   */
  const summary = await summaryWithinWindowOf({
    runsFolder,
    folder,
    cutoff,
  },);
  if ((typeof summary) === 'symbol') {
    return NOT_A_FINISHED_RUN;
  }
  return {
    folder,
    endReason: String(summary.endReason,),
    modelError: ((typeof summary.modelError) === 'string') ? summary.modelError : '',
    turns: String(summary.turns,),
    laterRun: false,
  };
}

/**
 No finished run in that folder, as a symbol so no run's end can be mistaken for it.
 */
const NOT_A_FINISHED_RUN: unique symbol = Symbol('the folder holds no finished run',);

/**
 Reads the ends of the runs that finished lately.

 @param runsFolder - the folder the driver writes one run folder per run into

 @param sinceMinutes - how many minutes back a run must have finished

 @param endReasons - the ends to list, empty for every end

 @param now - where the window is measured from

 @returns one line per run, newest folder name last, tab-separated as the script this replaces printed them

 @throws the error a read gave for a runs folder that is not there

 @example
 ```ts
 const lines = await recentRunEndsOf({ runsFolder: '/runs', sinceMinutes: 40, endReasons: [], now: Date.now(), });
 ```
 */
export async function recentRunEndsOf({
  runsFolder,
  sinceMinutes,
  endReasons,
  now,
}: {
  readonly runsFolder: string;
  readonly sinceMinutes: number;
  readonly endReasons: readonly string[];
  readonly now: number;
},): Promise<readonly string[]> {
  /**
   The entries of the runs folder.
   */
  const entries = await readdir(
    runsFolder,
    { withFileTypes: true, },
  );
  /**
   The moment the window starts.
   */
  const cutoff = now - (sinceMinutes * MILLISECONDS_PER_MINUTE);
  /**
   The ends found, in the folder's order.
   */
  const ends = (await Promise.all(entries.map(async function endIfRecent(entry,) {
    return entry.isDirectory()
      ? await recentEndOf({
        runsFolder,
        folder: entry.name,
        cutoff,
      },)
      : NOT_A_FINISHED_RUN;
  },),))
    .filter(function isAFinishedRun(end,): end is RecentRunEnd {
      return (typeof end) !== 'symbol';
    },);
  /**
   Every run folder's name, which is how a later attempt is known to exist.
   */
  const folders = new Set(entries.map(function nameOf(entry,) {
    return entry.name;
  },),);
  return ends.filter(function isAnEndAskedFor(end,) {
    return (endReasons.length === 0) || endReasons.includes(end.endReason,);
  },)
    .map(function asLine(end,) {
      /**
       The attempt this run is, when its name says.
       */
      const attempt = attemptOf({ folder: end.folder, },);
      /**
       Whether a later attempt of the same run exists.
       */
      const later = ((typeof attempt) === 'symbol')
        ? false
        : folders.has(`${attempt.name}-${String(attempt.run + 1,)}`,);
      return `${end.folder}\t${end.endReason}\t${end.modelError}\tturns=${end.turns}\tlaterRun=${String(later,)}`;
    },);
}
