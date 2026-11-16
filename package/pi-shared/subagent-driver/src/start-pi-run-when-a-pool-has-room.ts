#!/usr/bin/env node
/**
 `start-pi-run-when-a-pool-has-room`: starts one Pi run when the provider pool its provider and model share has room, and answers
 why it started none otherwise. See `README.md` for the options and the pools.

 The run itself is the worktree launcher's work: this command only counts, chooses the pool, and hands the launcher the words the
 run needs.

 @module
 */

import {
  statfsSync,
} from 'node:fs';
import {
  argv,
  env,
} from 'node:process';
import {
  BUILD_VOLUME,
  type FreeSpace,
  freeSpaceOnVolume,
} from './free-space-floor.ts';
import {
  goingRunsNow,
  LAUNCHER_ENTRY_NAMES,
  REPLACED_LAUNCHER_ENTRY_NAMES,
} from './going-runs.ts';
import {
  POOL_GATE_USAGE,
  parsePoolGateOptions,
} from './parse-pool-gate-options.ts';
import {
  limitsIn,
  POOL_LIMITS_VARIABLE,
  poolsWithTheseLimits,
} from './pool-limits.ts';
import { gateOutcomeOf, } from './pool-gate.ts';
import { talliesOf, } from './pool-tallies.ts';
import {
  runCommandShowingItsOutput,
  runCommandWithChildProcess,
} from './run-command.ts';
import { sessionFileOfRun, } from './session-file-of-a-run.ts';
import { siblingCommandEntry, } from './sibling-command-entry.ts';
import { usageWasAsked, } from './usage-of-a-command.ts';

// The word that asks for help goes to the command as an argument, and the command answers it before it reads an option
// or touches anything: mise appends a word its task's usage spec does not claim to the task's text, and `--help` is
// mise's own flag, so this is the only place it can land.
if (usageWasAsked({
  words: argv.slice(2,),
  usage: POOL_GATE_USAGE,
},)) {
  process.exitCode = 0;
  process.exit(0,);
}

/**
 The folder the process table is read from, which is where a Linux machine keeps it. A test points this at a folder of its own,
 so no test counts the processes of the machine it runs on.
 */
const PROCESS_FOLDER = env.PI_RUNS_PROCESS_FOLDER ?? '/proc';

/**
 The variable that names the room the build volume has, in bytes, instead of reading the volume. A test sets it so that no test
 depends on how full this machine's disk is.
 */
const GIVEN_FREE_SPACE_BYTES_VARIABLE = 'PI_RUNS_FREE_SPACE_BYTES';

/**
 The room the volume that holds the repository has, which a test may give instead of the machine reading its own disk.
 */
const GIVEN_FREE_SPACE_BYTES = env[GIVEN_FREE_SPACE_BYTES_VARIABLE] ?? '';

/**
 The room the volume that holds the repository has.
 */
const freeSpace: FreeSpace = (GIVEN_FREE_SPACE_BYTES === '')
  ? freeSpaceOnVolume({
    volume: BUILD_VOLUME,
    statfs: statfsSync,
  },)
  : {
    volume: BUILD_VOLUME,
    freeBytes: Number(GIVEN_FREE_SPACE_BYTES,),
  };

/**
 What this run of the command was asked for.
 */
const options = parsePoolGateOptions(argv.slice(2,),);

/**
 The pools this run counts against: the shipped table, with the limits a caller set in the environment overriding them, so a pool
 whose provider has a weekly or monthly quota is closed and reopened without a code change.
 */
const pools = poolsWithTheseLimits({ limits: limitsIn({ listed: env[POOL_LIMITS_VARIABLE] ?? '', },), },);

/**
 The runs going now, and every launcher that has not made its unit yet.
 */
const runs = await goingRunsNow({
  run: runCommandWithChildProcess,
  processFolder: PROCESS_FOLDER,
  launcherEntryNames: [
    ...LAUNCHER_ENTRY_NAMES,
    ...REPLACED_LAUNCHER_ENTRY_NAMES,
  ],
  pools,
  onProblem: function onProblem(problem,) {
    // The pools are counted from what could be read; a unit whose own command line could not be read still holds the fallback
    // pool, and saying so is what tells a reader which run that was.
    process.stderr
      .write(`${problem}\n`,);
  },
},);

/**
 What the gate answers.
 */
const outcome = await gateOutcomeOf({
  options,
  runs,
  tallies: talliesOf({
    runs,
    pools,
  },),
  freeSpace,
  sessionFileOf: function sessionFileOf(runFolder,) {
    return sessionFileOfRun({
      runsFolder: options.runsFolder,
      runFolder,
    },);
  },
},);

for (const line of outcome.lines) {
  console.log(line,);
}

if (outcome.action !== 'start') {
  process.exit((outcome.action === 'counts') ? 0 : outcome.status,);
}
if (outcome.action === 'start') {
  /**
   The launcher this package ships beside this command.
   */
  const launcher = await siblingCommandEntry({
    folder: import.meta.dirname,
    command: 'start-pi-run-in-a-worktree',
  },);
  /**
   What the launcher's process ended with, which this command answers with.
   */
  const launched = await runCommandShowingItsOutput({
    command: process.execPath,
    words: [
      launcher,
      ...outcome.arguments
        .words,
    ],
    environment: env,
  },);
  if (launched.startProblem !== undefined) {
    throw new Error(`the launcher could not be started: ${launched.startProblem}`,);
  }
  process.exitCode = launched.status ?? 1;
}
