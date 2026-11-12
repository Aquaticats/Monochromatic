#!/usr/bin/env node
/**
 `print-the-final-report-of-the-newest-run-of-each-agent`: prints, for every agent named, the newest run of that name that
 finished, why it ended, and the head of the report it ended with. See `README.md` for the options.

 A coordinator reading several agents at once gets one headed block per agent, so a run that never finished says so instead of
 being left out.

 @module
 */

import { readFile, } from 'node:fs/promises';
import { homedir, } from 'node:os';
import { join, } from 'node:path';
import {
  argv,
  stdout,
} from 'node:process';
import {
  NO_FINISHED_RUN_OF_THAT_NAME,
  SUMMARY_FILE_NAME,
  newestFinishedRunOf,
} from './newest-finished-run-of-a-name.ts';
import { optionIn, } from './option-in.ts';
import {
  REPORT_LINES,
  REPORT_WIDTH,
  noFinishedRunLineOf,
  reportLinesOf,
  summaryIn,
} from './report-of-a-run.ts';
import {
  requiredPositiveNumber,
  requiredText,
} from './required-option-values.ts';

/**
 The command's command line, for the error message.
 */
const FINAL_REPORT_USAGE =
  'print-the-final-report-of-the-newest-run-of-each-agent --agent=<name> [--agent=<name>]... [--runs-folder=<folder>] '
    + '[--lines=<n>] [--width=<n>]';

/**
 The option that names one agent.
 */
const AGENT_OPTION = '--agent=';

/**
 The option that names the folder the runs live in.
 */
const RUNS_FOLDER_OPTION = '--runs-folder=';

/**
 The option that names how many lines of the final message to print.
 */
const LINES_OPTION = '--lines=';

/**
 The option that names how much of each line to print.
 */
const WIDTH_OPTION = '--width=';

/**
 The parts of the coordinator's runs folder below the home directory of whoever runs this.
 */
const DEFAULT_RUNS_FOLDER_IN_HOME: readonly string[] = [
  'temp',
  'agent',
  'pi-runs',
];

/**
 A command line the command cannot use.
 */
class FinalReportOptionsError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'FinalReportOptionsError';
}

/**
 Builds the error this command answers a refused option with.

 @param message - what is wrong with the command line

 @returns the error the command exits with

 @example
 ```ts
 throw finalReportOptionsErrorOf('--agent is required.',);
 ```
 */
function finalReportOptionsErrorOf(message: string,): Error {
  return new FinalReportOptionsError(message,);
}

/**
 Reads a number of lines or a width, which is the shipped figure when the caller names none.

 @param given - the option's text, empty when the line does not carry it

 @param name - the option's name, without the leading dashes

 @param shipped - the figure the command ships

 @returns the number

 @throws FinalReportOptionsError when the option is not a number above zero

 @example
 ```ts
 const lines = figureOf({ given: '', name: 'lines', shipped: REPORT_LINES, },);
 ```
 */
function figureOf({
  given,
  name,
  shipped,
}: {
  readonly given: string;
  readonly name: string;
  readonly shipped: number;
},): number {
  return (given === '')
    ? shipped
    : requiredPositiveNumber({
      value: given,
      name,
      usage: FINAL_REPORT_USAGE,
      errorOf: finalReportOptionsErrorOf,
    },);
}

/**
 The words the command was given.
 */
const words = argv.slice(2,);

/**
 The agents named, in the order they were written.
 */
const agents = words.filter(function namesAnAgent(word,) {
  return word.startsWith(AGENT_OPTION,);
},)
  .map(function agentOf(word,) {
    return requiredText({
      value: word.slice(AGENT_OPTION.length,),
      name: 'agent',
      usage: FINAL_REPORT_USAGE,
      errorOf: finalReportOptionsErrorOf,
    },);
  },);
if (agents.length === 0) {
  throw new FinalReportOptionsError(`--agent is required at least once.\nUsage: ${FINAL_REPORT_USAGE}`,);
}

/**
 The folder the runs live in.
 */
const runsFolderGiven = optionIn({
  commandLine: words,
  option: RUNS_FOLDER_OPTION,
},);
/**
 The folder the runs live in, which is the coordinator's own under the home directory unless the caller names one.
 */
const runsFolder = (runsFolderGiven === '')
  ? join(
    homedir(),
    ...DEFAULT_RUNS_FOLDER_IN_HOME,
  )
  : runsFolderGiven;

/**
 How many lines of each final message to print.
 */
const lines = figureOf({
  given: optionIn({
    commandLine: words,
    option: LINES_OPTION,
  },),
  name: 'lines',
  shipped: REPORT_LINES,
},);

/**
 How much of each line to print.
 */
const width = figureOf({
  given: optionIn({
    commandLine: words,
    option: WIDTH_OPTION,
  },),
  name: 'width',
  shipped: REPORT_WIDTH,
},);

/**
 The lines to print, one block per agent, in the order the agents were named.
 */
const printed: string[] = [];
/* oxlint-disable eslint/no-await-in-loop -- one agent after another, each reading its own run's summary, so the blocks print in
   the order the caller named them and a failure names the agent it belongs to. See doc/lint/sequential-awaits.md. */
for (const agent of agents) {
  /**
   The newest run of that name that finished.
   */
  const newest = await newestFinishedRunOf({
    runsFolder,
    runName: agent,
  },);
  if ((typeof newest) === 'symbol') {
    printed.push(noFinishedRunLineOf(agent,),);
    continue;
  }
  printed.push(...reportLinesOf({
    runFolder: newest,
    summary: summaryIn({
      shown: await readFile(
        join(
          runsFolder,
          newest,
          SUMMARY_FILE_NAME,
        ),
        'utf8',
      ),
    },),
    lines,
    width,
  },),);
}
/* oxlint-enable eslint/no-await-in-loop */
stdout.write(`${printed.join('\n',)}\n`,);
