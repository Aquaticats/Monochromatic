#!/usr/bin/env node
/**
 `retry-pi-run-start-until-a-pool-has-room`: asks the pool gate for a slot again and again until it starts the run or its own
 deadline passes. See `README.md` for the options.

 The attempt is the whole run: the gate starts the launcher, which waits for the unit, so this command answers with the unit's
 status once the run is over. What bounds that wait is the unit's own `RuntimeMaxSec`, set from the run's wall clock.

 @module
 */

import { argv, } from 'node:process';
import { setTimeout as wait, } from 'node:timers/promises';
import { MILLISECONDS_PER_SECOND, } from './driver-constants.ts';
import { MILLISECONDS_PER_MINUTE, } from './overdue-constants.ts';
import {
  type CommandOutput,
  runCommandWithChildProcess,
} from './run-command.ts';
import { parseRetryStartOptions, } from './parse-retry-start-options.ts';
import { retryStartUntilAPoolHasRoom, } from './retry-start.ts';
import { siblingCommandEntry, } from './sibling-command-entry.ts';

/**
 Minutes in one hour.
 */
const MINUTES_PER_HOUR = 60;

/**
 Milliseconds in one hour.
 */
const MILLISECONDS_PER_HOUR = MINUTES_PER_HOUR * MILLISECONDS_PER_MINUTE;

/**
 Hours a gate may take to answer. It answers when the run it started is over, which the unit's `RuntimeMaxSec` bounds at the run's
 own wall clock plus the driver's teardown. The coordinator's batch lists ask for two hours a run, so a day sits far above any run
 this command starts, and it is only the bound that keeps this one from waiting on a gate that would never answer.
 */
const HOURS_A_GATE_MAY_TAKE = 24;

/**
 How long the gate may take to answer.
 */
const GATE_TIMEOUT_MILLISECONDS = HOURS_A_GATE_MAY_TAKE * MILLISECONDS_PER_HOUR;

/**
 What this run of the command was asked for.
 */
const options = parseRetryStartOptions(argv.slice(2,),);

/**
 The pool gate this package ships beside this command.
 */
const gate = await siblingCommandEntry({
  folder: import.meta.dirname,
  command: 'start-pi-run-when-a-pool-has-room',
},);

/**
 Asks the pool gate once, and answers with both streams it printed.

 @returns what the gate printed and how it ended

 @throws Error when the gate could not be started at all

 @example
 ```ts
 const asked = await askTheGate();
 ```
 */
async function askTheGate(): Promise<CommandOutput> {
  return await runCommandWithChildProcess({
    command: process.execPath,
    words: [
      gate,
      ...options.words,
    ],
    timeoutMilliseconds: GATE_TIMEOUT_MILLISECONDS,
  },);
}

/**
 What asking for a slot ended with.
 */
const outcome = await retryStartUntilAPoolHasRoom({
  attempt: askTheGate,
  now: Date.now,
  wait,
  deadline: Date.now() + (options.deadlineSeconds * MILLISECONDS_PER_SECOND),
  periodMilliseconds: options.periodSeconds * MILLISECONDS_PER_SECOND,
  words: options.words,
},);

process.stdout
  .write(outcome.text,);
process.exitCode = outcome.status;
