#!/usr/bin/env node
/**
 `start-pi-run-in-a-free-slot`: starts one Pi run when its provider pool has a slot free, making its worktree, its run folder and
 its capped systemd unit, and ending when the unit ends with the unit's own status. See `README.md` for every option and every
 default.

 It answers `NO SLOT` with status 3 when the pool is full and `ALREADY RUNNING` with status 4 when a run of that name is
 already going, which is what a caller waiting for a slot reads.

 @module
 */

import { homedir, } from 'node:os';
import {
  argv,
  env,
  pid,
  stdout,
} from 'node:process';
import {
  STARTER_COMMAND_NAME,
  readRunLaunchers,
} from './run-launchers.ts';
import { runCommandWithChildProcess, } from './run-command.ts';
import { runToTheEnd, } from './run-command-to-the-end.ts';
import { startRunningPiUnits, } from './running-pi-units.ts';
import { startAPiRun, } from './start-a-pi-run.ts';
import { parseStartPiRunOptions, } from './start-pi-run-options.ts';
import { sizedTurnCap, } from './turn-caps-from-finished-runs.ts';

/**
 What this run of the command was asked for.
 */
const options = parseStartPiRunOptions(argv.slice(2,),);

/**
 Reads every Pi unit systemd is running.
 */
const runningUnits = startRunningPiUnits({ run: runCommandWithChildProcess, },);

/**
 What the start answered with.
 */
const outcome = await startAPiRun({
  options,
  run: runCommandWithChildProcess,
  runToTheEnd,
  launchers: async function launchersNow() {
    return await readRunLaunchers({
      commandName: STARTER_COMMAND_NAME,
      ownProcessId: String(pid,),
    },);
  },
  runningUnits,
  sizeATurnCap: sizedTurnCap,
  home: env.HOME ?? homedir(),
},);

stdout.write(`${outcome.lines
  .join('\n',)}\n`,);
process.exitCode = outcome.status;
