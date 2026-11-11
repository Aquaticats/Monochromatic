/**
 One named run's last words: its end, what the model said when it ended on an error, its turns, and a slice of its final report.

 A run folder is named here rather than guessed, which is what a coordinator wants when a line of `watch-overdue-pi-runs` or
 an answer of `runs-ended-by-provider` already named the attempt.

 @module
 */

import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';
import {
  type RequiredOption,
  requiredPositiveNumber,
  requiredText,
} from './required-option-values.ts';
import { defaultRunsFolder, } from './pi-runs-folders.ts';

/**
 The command's command line, for the error message.
 */
export const NAMED_REPORT_USAGE =
  'latest-pi-run-reports --run=<folder>... [--from=<n>] [--to=<n>] [--width=<n>] [--runs-folder=<folder>]';

/**
 The summary a finished run leaves last.
 */
const SUMMARY_FILE = 'summary.json';

/**
 The first line of a report printed when the caller names none, counting from one.
 */
const DEFAULT_FROM = 1;

/**
 The last line of a report printed when the caller names none, which is the slice a coordinator reads at a prompt.
 */
const DEFAULT_TO = 40;

/**
 How many characters of one line are printed when the caller names no width.
 */
const DEFAULT_WIDTH = 400;

/**
 A named report's command line cannot be used.
 */
export class NamedReportOptionsError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'NamedReportOptionsError';
}

/**
 Builds the error a refused option is answered with.

 @param message - what is wrong with the command line

 @returns the error the command exits with

 @example
 ```ts
 throw namedReportOptionsErrorOf('--run is required.',);
 ```
 */
function namedReportOptionsErrorOf(message: string,): Error {
  return new NamedReportOptionsError(message,);
}

/**
 Describes one option for the readers that refuse it.

 @param value - the option's value as the command line read it

 @param name - the option's name, without the leading dashes

 @returns what a reader of an option needs to refuse it

 @example
 ```ts
 const option = optionOf({ value: '40', name: 'to', });
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
    usage: NAMED_REPORT_USAGE,
    errorOf: namedReportOptionsErrorOf,
  };
}

/**
 What one run of the command was asked for.
 */
export type NamedReportOptions = {
  /**
   Folder the driver writes one run folder per run into.
   */
  readonly runsFolder: string;

  /**
   The run folders to read, in the order the caller named them.
   */
  readonly runFolders: readonly string[];

  /**
   The first line of each report that is printed, counting from one.
   */
  readonly from: number;

  /**
   The last line of each report that is printed.
   */
  readonly to: number;

  /**
   How many characters of one line are printed.
   */
  readonly width: number;
};

/**
 Reads every value one option of this command was given, in the order written.

 @param commandLine - arguments after the program name

 @param name - the option's name, without the leading dashes

 @returns the values, empty when the option was not given

 @example
 ```ts
 const runs = valuesOf({ commandLine: ['--run=a-1',], name: 'run', });
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
 Reads the value one option of this command was given, when it was given once.

 @param commandLine - arguments after the program name

 @param name - the option's name, without the leading dashes

 @returns the value, or nothing when the option was not given

 @example
 ```ts
 const to = valueOf({ commandLine: ['--to=10',], name: 'to', });
 ```
 */
function valueOf({
  commandLine,
  name,
}: {
  readonly commandLine: readonly string[];
  readonly name: string;
},): string | typeof NOT_GIVEN {
  /**
   Every value written for this option.
   */
  const written = valuesOf({
    commandLine,
    name,
  },);
  if (written.length > 1) {
    throw namedReportOptionsErrorOf(`--${name} takes one value.\nUsage: ${NAMED_REPORT_USAGE}`,);
  }
  return written[0] ?? NOT_GIVEN;
}

/**
 An option that was not given, as a symbol so no value can be mistaken for it.
 */
const NOT_GIVEN: unique symbol = Symbol('the option was not given',);

/**
 Reads the command line of a report for a named run.

 @param commandLine - arguments after the program name

 @returns what the run was asked for

 @throws NamedReportOptionsError when an option is unknown or unusable, or no run folder is named

 @example
 ```ts
 const options = parseNamedReportOptions(['--run=copy-route-3',],);
 ```
 */
export function parseNamedReportOptions(commandLine: readonly string[],): NamedReportOptions {
  /**
   The run folders named.
   */
  const runFolders = valuesOf({
    commandLine,
    name: 'run',
  },);
  /**
   The runs folder named, when it was named.
   */
  const runsFolder = valueOf({
    commandLine,
    name: 'runs-folder',
  },);
  /**
   The first line named, when it was named.
   */
  const from = valueOf({
    commandLine,
    name: 'from',
  },);
  /**
   The last line named, when it was named.
   */
  const to = valueOf({
    commandLine,
    name: 'to',
  },);
  /**
   The width named, when it was named.
   */
  const width = valueOf({
    commandLine,
    name: 'width',
  },);
  /**
   How many options this command has, which is every argument that starts with two dashes and names one of them.
   */
  const known = [
    'run',
    'runs-folder',
    'from',
    'to',
    'width',
  ];
  if (commandLine.some(function isNoOptionOfThisCommand(argument,) {
    return argument.startsWith('--',)
      && (!known.some(function isKnown(name,) {
        return argument.startsWith(`--${name}=`,);
      },));
  },)) {
    throw namedReportOptionsErrorOf(`an argument starting with -- is no option of this command.\nUsage: ${NAMED_REPORT_USAGE}`,);
  }
  if (runFolders.length === 0) {
    throw namedReportOptionsErrorOf(`name at least one run folder with --run.\nUsage: ${NAMED_REPORT_USAGE}`,);
  }
  return {
    runsFolder: ((typeof runsFolder) === 'symbol')
      ? defaultRunsFolder({},)
      : requiredText(optionOf({
        value: runsFolder,
        name: 'runs-folder',
      },),),
    runFolders,
    from: ((typeof from) === 'symbol') ? DEFAULT_FROM : requiredPositiveNumber(optionOf({
      value: from,
      name: 'from',
    },),),
    to: ((typeof to) === 'symbol') ? DEFAULT_TO : requiredPositiveNumber(optionOf({
      value: to,
      name: 'to',
    },),),
    width: ((typeof width) === 'symbol') ? DEFAULT_WIDTH : requiredPositiveNumber(optionOf({
      value: width,
      name: 'width',
    },),),
  };
}

/**
 Reads one named run's report.

 @param runsFolder - the folder the driver writes one run folder per run into

 @param runFolder - the run folder to read

 @param from - the first line of the report that is printed

 @param to - the last line of the report that is printed

 @param width - how many characters of one line are printed

 @returns the lines to print

 @throws the error a read gave, or the JSON error of a summary that is not JSON

 @example
 ```ts
 const lines = await reportOfANamedRun({ runsFolder: '/runs', runFolder: 'copy-route-3', from: 1, to: 10, width: 80, });
 ```
 */
export async function reportOfANamedRun({
  runsFolder,
  runFolder,
  from,
  to,
  width,
}: {
  readonly runsFolder: string;
  readonly runFolder: string;
  readonly from: number;
  readonly to: number;
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
    readonly modelError?: unknown;
    readonly turns?: unknown;
    readonly lastAssistantText?: unknown;
  };
  /**
   The report's lines as the run wrote them, blank lines included, because a caller names the slice it wants by their count.
   */
  const text = ((typeof summary.lastAssistantText) === 'string')
    ? summary.lastAssistantText
      .split('\n',)
    : [];
  return [
    `=== ${runFolder}: ${String(summary.endReason,)} ${((typeof summary.modelError) === 'string') ? summary.modelError : ''} turns ${String(summary.turns,)}`,
    `(report has ${String(text.length,)} lines)`,
    ...text.slice(
      from - 1,
      to,
    )
      .map(function cutToWidth(line,) {
        return line.slice(
          0,
          width,
        );
      },),
  ];
}
