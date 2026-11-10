#!/usr/bin/env node
/**
 `latest-pi-run-reports`: prints the newest finished run of every agent named, with its end, its turns and a slice of its final
 report. See `README.md` for the options.

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

/**
 What this run of the command was asked for.
 */
const options = parseReportOptions(argv.slice(2,),);

/**
 The lines to print: one block per agent.
 */
const lines = await latestReportsOf({
  runsFolder: options.runsFolder,
  names: options.names,
  lines: options.lines,
  width: options.width,
},);

stdout.write(`${lines.join('\n',)}\n`,);
