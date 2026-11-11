#!/usr/bin/env node
/**
 `list-recent-pi-run-ends`: one line per run that finished lately, saying how it ended, what the model said when it ended on an
 error, how many turns it took, and whether a later attempt of the same run exists.

 This is what a coordinator reads after a provider incident. `--end=model-error` narrows it to the runs the provider ended.

 @module
 */

import {
  argv,
  stdout,
} from 'node:process';
import {
  parseRunEndOptions,
  recentRunEndsOf,
} from './recent-pi-run-ends.ts';

/**
 What this run of the command was asked for.
 */
const options = parseRunEndOptions(argv.slice(2,),);

/**
 The lines to print.
 */
const lines = await recentRunEndsOf({
  runsFolder: options.runsFolder,
  sinceMinutes: options.sinceMinutes,
  endReasons: options.endReasons,
  now: Date.now(),
},);

stdout.write(`${lines.join('\n',)}\n`,);
