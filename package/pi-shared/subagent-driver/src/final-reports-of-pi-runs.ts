/**
 The last thing each named run said: its end, how many turns it took, and a slice of its final report.

 A run's folder is `<name>-<number>`, and the newest number with a summary is the run that finished last, so a coordinator that
 asked for a round of work can read what came back without remembering which attempt it was.

 @module
 */

import {
  readdir,
  readFile,
  stat,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { errorCodeOf, } from './error-code-of.ts';
import {
  type RequiredOption,
  requiredPositiveNumber,
  requiredText,
} from './required-option-values.ts';
import { defaultRunsFolder, } from './pi-runs-folders.ts';
import { isWholeNumberText, } from './whole-number-text.ts';

/**
 The command's command line, for the error message.
 */
export const REPORTS_USAGE =
  'latest-pi-run-reports [--lines=<n>] [--width=<n>] [--runs-folder=<folder>] <agent>...';

/**
 The summary a finished run leaves last.
 */
const SUMMARY_FILE = 'summary.json';

/**
 How many lines of a report are printed when the caller names no count.
 */
const DEFAULT_LINES = 30;

/**
 How many characters of a report's line are printed when the caller names no width, which keeps a long paragraph readable at a
 prompt.
 */
const DEFAULT_WIDTH = 420;

/**
 A command line the report reader cannot use.
 */
export class ReportOptionsError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'ReportOptionsError';
}

/**
 Builds the error a refused option is answered with.

 @param message - what is wrong with the command line

 @returns the error the command exits with

 @example
 ```ts
 throw reportOptionsErrorOf('--width must be a number above zero.',);
 ```
 */
function reportOptionsErrorOf(message: string,): Error {
  return new ReportOptionsError(message,);
}

/**
 Describes one option for the readers that refuse it.

 @param value - the option's value as the command line parser read it

 @param name - the option's name, without the leading dashes

 @returns what a reader of an option needs to refuse it

 @example
 ```ts
 const option = optionOf({ value: '30', name: 'lines', });
 ```
 */
function optionOf({
  value,
  name,
}: {
  readonly value: unknown;
  readonly name: string;
},): RequiredOption {
  return {
    value,
    name,
    usage: REPORTS_USAGE,
    errorOf: reportOptionsErrorOf,
  };
}

/**
 What one run of the command was asked for.
 */
export type ReportOptions = {
  /**
   The agents whose newest finished run is printed.
   */
  readonly names: readonly string[];

  /**
   Folder the driver writes one run folder per run into.
   */
  readonly runsFolder: string;

  /**
   How many lines of a report are printed.
   */
  readonly lines: number;

  /**
   How many characters of one line are printed.
   */
  readonly width: number;
};

/**
 Reads the command line.

 @param commandLine - arguments after the program name

 @returns what the run was asked for

 @throws ReportOptionsError when an option is unknown or unusable, or no agent is named

 @example
 ```ts
 const options = parseReportOptions(['--lines=5', 'copy-route',],);
 ```
 */
export function parseReportOptions(commandLine: readonly string[],): ReportOptions {
  /**
   The option values found, by name.
   */
  const lines = commandLine.find(function isLines(argument,) {
    return argument.startsWith('--lines=',);
  },)
    ?.slice('--lines='.length,);
  /**
   The width the caller asked for.
   */
  const width = commandLine.find(function isWidth(argument,) {
    return argument.startsWith('--width=',);
  },)
    ?.slice('--width='.length,);
  /**
   The runs folder the caller named, which is where this machine keeps its runs when nothing is named.
   */
  const runsFolder = commandLine.find(function isRunsFolder(argument,) {
    return argument.startsWith('--runs-folder=',);
  },)
    ?.slice('--runs-folder='.length,);
  /**
   The agents named, which are every argument that is no option of this command.
   */
  const names = commandLine.filter(function isAName(argument,) {
    return (!argument.startsWith('--',));
  },);
  if (names.some(function isAnOption(argument,) {
    return argument.startsWith('--',);
  },)) {
    throw reportOptionsErrorOf(`an argument starting with -- is no option of this command.\nUsage: ${REPORTS_USAGE}`,);
  }
  if (names.length === 0) {
    throw reportOptionsErrorOf(`name at least one agent.\nUsage: ${REPORTS_USAGE}`,);
  }
  return {
    names,
    runsFolder: (runsFolder === undefined) ? defaultRunsFolder({},) : requiredText(optionOf({
      value: runsFolder,
      name: 'runs-folder',
    },),),
    lines: (lines === undefined) ? DEFAULT_LINES : requiredPositiveNumber(optionOf({
      value: lines,
      name: 'lines',
    },),),
    width: (width === undefined) ? DEFAULT_WIDTH : requiredPositiveNumber(optionOf({
      value: width,
      name: 'width',
    },),),
  };
}

/**
 An entry that is no attempt of the agent, as a symbol so no number can be mistaken for it.
 */
const NOT_AN_ATTEMPT: unique symbol = Symbol('the entry is no attempt of this agent',);

/**
 Reads one entry of the runs folder as an attempt of a run.

 @param entry - the entry's name

 @param name - the agent's name

 @returns the attempt's number, or nothing when the entry is no attempt of that agent

 @example
 ```ts
 const number = attemptNumberIn({ entry: 'copy-route-3', name: 'copy-route', });
 ```
 */
function attemptNumberIn({
  entry,
  name,
}: {
  readonly entry: string;
  readonly name: string;
},): number | typeof NOT_AN_ATTEMPT {
  /**
   The start every attempt of this agent's folder name has.
   */
  const prefix = `${name}-`;
  if (!entry.startsWith(prefix,)) {
    return NOT_AN_ATTEMPT;
  }
  /**
   What follows the agent's name.
   */
  const attempt = entry.slice(prefix.length,);
  return isWholeNumberText({ text: attempt, }) ? Number(attempt,) : NOT_AN_ATTEMPT;
}

/**
 No finished run of that agent, as a symbol so no run folder's name can be mistaken for it.
 */
export const NO_FINISHED_RUN: unique symbol = Symbol('the agent has finished no run',);

/**
 Reads the newest attempt of an agent that left a summary.

 @param runsFolder - the folder the driver writes one run folder per run into

 @param name - the agent's name

 @returns the run folder's name, or nothing when the agent has not finished a run

 @throws the error a read gave for a runs folder that is not there

 @example
 ```ts
 const latest = await latestFinishedRunOf({ runsFolder: '/runs', name: 'copy-route', });
 ```
 */
export async function latestFinishedRunOf({
  runsFolder,
  name,
}: {
  readonly runsFolder: string;
  readonly name: string;
},): Promise<string | typeof NO_FINISHED_RUN> {
  /**
   The entries of the runs folder.
   */
  const entries = await readdir(
    runsFolder,
    { withFileTypes: true, },
  );
  /**
   The attempts of this agent that left a summary.
   */
  const finished = (await Promise.all(entries.map(async function attemptIfFinished(entry,) {
    /**
     This entry's attempt number, when it is one of this agent's.
     */
    const attempt = attemptNumberIn({
      entry: entry.name,
      name,
    },);
    if ((typeof attempt) === 'symbol') {
      return NOT_AN_ATTEMPT;
    }
    try {
      await stat(join(
        runsFolder,
        entry.name,
        SUMMARY_FILE,
      ),);
      return attempt;
    }
    catch (error) {
      // An attempt still running has written no summary yet, which is not a report to print.
      if (errorCodeOf(error,) !== 'ENOENT') {
        throw error;
      }
      return NOT_AN_ATTEMPT;
    }
  },),))
    .filter(function isAnAttempt(attempt,): attempt is number {
      return (typeof attempt) !== 'symbol';
    },);
  if (finished.length === 0) {
    return NO_FINISHED_RUN;
  }
  return `${name}-${String(Math.max(...finished,),)}`;
}

/**
 Reads the summary of one run folder.

 @param runsFolder - the folder the driver writes one run folder per run into

 @param runFolder - the run folder's name

 @returns what the report reads of the summary

 @throws the error a read gave, or the JSON error of a summary that is not JSON

 @example
 ```ts
 const report = await reportOf({ runsFolder: '/runs', runFolder: 'copy-route-3', lines: 5, width: 80, });
 ```
 */
async function reportOf({
  runsFolder,
  runFolder,
  lines,
  width,
}: {
  readonly runsFolder: string;
  readonly runFolder: string;
  readonly lines: number;
  readonly width: number;
},): Promise<readonly string[]> {
  /**
   What the run left last.
   */
  const summary = JSON.parse(await readFile(
    join(
      runsFolder,
      runFolder,
      SUMMARY_FILE,
    ),
    { encoding: 'utf8', },
  ),) as {
    readonly endReason?: unknown;
    readonly turns?: unknown;
    readonly lastAssistantText?: unknown;
  };
  /**
   The report's lines, with the blank ones dropped so a slice holds what was said.
   */
  const text = ((typeof summary.lastAssistantText) === 'string')
    ? summary.lastAssistantText
      .split('\n',)
      .filter(function holdsSomething(line,) {
        return line.trim() !== '';
      },)
    : [];
  return [
    `=== ${runFolder}: ${String(summary.endReason,)}, ${String(summary.turns,)} turns`,
    ...text.slice(
      0,
      lines,
    )
      .map(function cutToWidth(line,) {
        return line.slice(
          0,
          width,
        );
      },),
    ...((text.length > lines) ? [`[${String(text.length - lines,)} more lines]`,] : []),
  ];
}

/**
 Reads the newest report of every agent named.

 @param runsFolder - the folder the driver writes one run folder per run into

 @param names - the agents to read

 @param lines - how many lines of a report are printed

 @param width - how many characters of one line are printed

 @returns the lines to print, one block per agent

 @throws the error a read gave for a runs folder that is not there

 @example
 ```ts
 const lines = await latestReportsOf({ runsFolder: '/runs', names: ['copy-route',], lines: 5, width: 80, });
 ```
 */
export async function latestReportsOf({
  runsFolder,
  names,
  lines,
  width,
}: {
  readonly runsFolder: string;
  readonly names: readonly string[];
  readonly lines: number;
  readonly width: number;
},): Promise<readonly string[]> {
  /**
   One block per agent, read together and kept in the order the caller named them.
   */
  const blocks = await Promise.all(names.map(async function blockOf(name,) {
    /**
     The newest attempt of this agent that left a summary.
     */
    const latest = await latestFinishedRunOf({
      runsFolder,
      name,
    },);
    if ((typeof latest) === 'symbol') {
      return [`=== ${name}: no finished run`,];
    }
    return await reportOf({
      runsFolder,
      runFolder: latest,
      lines,
      width,
    },);
  },),);
  return blocks.flat();
}
