#!/usr/bin/env node
/**
 `watch-overdue-pi-runs`: one command that ends when a Pi run should have come back, printing a line for each such run and
 nothing else. See `README.md` for the options and for how the limits are computed.

 @module
 */

import { argv, } from 'node:process';
import { setTimeout as wait, } from 'node:timers/promises';
import {
  defaultCheckedRunsFile,
  readCheckedRuns,
  writeCheckedRuns,
} from './checked-runs.ts';
import { MILLISECONDS_PER_SECOND, } from './driver-constants.ts';
import {
  type OverdueLook,
  lookAtPiRuns,
} from './look-at-pi-runs.ts';
import { parseWatchOptions, } from './parse-watch-options.ts';
import { runCommandWithChildProcess, } from './run-command.ts';
import { startRunningPiUnits, } from './running-pi-units.ts';
import { watchForOverdueRuns, } from './watch-for-overdue-runs.ts';

/**
 What this run of the command was asked for.
 */
const options = parseWatchOptions(argv.slice(2,),);

/**
 Where what was reported is kept.
 */
const stateFile = options.stateFile ?? defaultCheckedRunsFile();

/**
 Reads the Pi runs going now.
 */
const runningPiUnits = startRunningPiUnits({ run: runCommandWithChildProcess, },);

/**
 When this run of the command started looking, which its deadline is measured from.
 */
const startedAt = Date.now();

/**
 Looks at the runs going now, and keeps what it reported so a rerun on the next turn stays quiet about the same run.

 @returns what the look found

 @example
 ```ts
 const found = await look();
 ```
 */
async function look(): Promise<OverdueLook> {
  /**
   What the runs going now look like.
   */
  const found = await lookAtPiRuns({
    runsFolder: options.runsFolder,
    units: await runningPiUnits(),
    checked: await readCheckedRuns({ file: stateFile, },),
    now: Date.now(),
    recheckMilliseconds: options.recheckSeconds * MILLISECONDS_PER_SECOND,
    ...(options.limitMinutes === undefined ? {} : { limitMinutes: options.limitMinutes, }),
  },);
  // A limit the caller set reports a run now and leaves the recheck interval as it was, so the next automatic look still
  // reports what it would have reported.
  if ((found.lines
    .length
    > 0) && (options.limitMinutes === undefined)) {
    await writeCheckedRuns({
      file: stateFile,
      checked: found.reported,
    },);
  }
  return found;
}

/**
 The lines this run of the command answers with.
 */
const lines = await watchForOverdueRuns({
  look,
  now: Date.now,
  wait: async function passTime(milliseconds,) {
    // A wait that lets the event loop go empty would end the process between two looks, so this one holds it up: it is the
    // only thing pending while the runs work.
    await wait(milliseconds,);
  },
  deadline: startedAt + (options.deadlineSeconds * MILLISECONDS_PER_SECOND),
  intervalMilliseconds: options.intervalSeconds * MILLISECONDS_PER_SECOND,
  once: options.once,
},);

console.log(lines.join('\n',),);
