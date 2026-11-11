#!/usr/bin/env node
/**
 `latest-pi-run-reports`: prints what finished runs said.

 Two jobs, one command, because both read the same folder:

 - each agent named gives its newest finished run, with its end, its turns and a slice of its report;
 - each `--run=<folder>` gives that exact attempt, with the lines `--from` to `--to` of its report,
   which is what a line of `watch-overdue-pi-runs` or `runs-ended-by-provider` names.

 @module
 */

import {
  argv,
  stdout,
} from 'node:process';
import {
  latestReportsOf,
  parseReportOptions,
} from './final-reports-of-pi-runs.ts';
import {
  parseNamedReportOptions,
  reportOfANamedRun,
} from './report-of-a-named-run.ts';

/**
 The option that names one run folder, which chooses the second job.
 */
const RUN_OPTION_PREFIX = '--run=';

/**
 Reads one report per run folder named.

 @returns the lines to print

 @example
 ```ts
 const lines = await namedRunLines();
 ```
 */
async function namedRunLines(): Promise<readonly string[]> {
  /**
   What this run of the command was asked for.
   */
  const options = parseNamedReportOptions(argv.slice(2,),);
  /**
   One block of lines per run, in the order the caller named them.
   */
  const blocks: string[][] = [];
  for (const runFolder of options.runFolders) {
    // oxlint-disable-next-line eslint/no-await-in-loop -- one run's report is read before the next run's, so a long report prints as one block; see doc/lint/sequential-awaits.md
    blocks.push([...await reportOfANamedRun({
      runsFolder: options.runsFolder,
      runFolder,
      from: options.from,
      to: options.to,
      width: options.width,
    },),],);
  }
  return blocks.flat();
}

/**
 Reads each named agent's newest finished report.

 @returns the lines to print

 @example
 ```ts
 const lines = await agentLines();
 ```
 */
async function agentLines(): Promise<readonly string[]> {
  /**
   What this run of the command was asked for.
   */
  const options = parseReportOptions(argv.slice(2,),);
  return await latestReportsOf({
    runsFolder: options.runsFolder,
    names: options.names,
    lines: options.lines,
    width: options.width,
  },);
}

/**
 Whether this run of the command reads named run folders instead of each agent's newest.
 */
const readsNamedRuns = argv.some(function namesARun(argument,) {
  return argument.startsWith(RUN_OPTION_PREFIX,);
},);

/**
 The lines to print.
 */
const lines: readonly string[] = readsNamedRuns
  ? await namedRunLines()
  : await agentLines();

stdout.write(`${lines.join('\n',)}\n`,);
