#!/usr/bin/env node
/**
 `start-pi-run-in-a-worktree`: makes the worktree a run works in, links its dependencies, starts the driver as a capped systemd
 unit, and removes the worktree once the run has finished with it. See `README.md` for the options.

 One line is printed per step, and the driver's own files land in the run folder.

 @module
 */

import { spawn, } from 'node:child_process';
import {
  mkdir,
  readdir,
} from 'node:fs/promises';
import { join, } from 'node:path';
import {
  argv,
  env,
} from 'node:process';
import { setTimeout as wait, } from 'node:timers/promises';
import { checkoutFolderOf, } from './checkout-of-a-command.ts';
import {
  StartRefusedError,
  startRefusalsIn,
} from './checks-before-a-start.ts';
import { finishTheRun, } from './finish-a-run.ts';
import { gitRunnerOf, } from './git-command.ts';
import { refuseUnlessTheWrapperAnswersGit, } from './git-wrapper-preflight.ts';
import { nextRunNumber, } from './next-run-number.ts';
import {
  NO_WORKTREE_BRANCH,
  parseWorktreeLauncherOptions,
  worktreeFolderOf,
} from './parse-worktree-launcher-options.ts';
import {
  CommandAnswerMissingError,
  requiredOutputOf,
} from './required-output-of-a-command.ts';
import { MILLISECONDS_PER_SECOND, } from './driver-constants.ts';
import { runCommandWithChildProcess, } from './run-command.ts';
import {
  UNIT_WAIT_GRACE_SECONDS,
  waitForUnitToEnd,
} from './wait-for-a-unit-to-end.ts';
import { siblingCommandEntry, } from './sibling-command-entry.ts';
import {
  NO_FINISHED_RUN,
  TURN_CAP_MARGIN,
  finishedTurnsIn,
  turnCapSizing,
} from './turn-cap-of-finished-runs.ts';
import { unitCommandOf, } from './unit-command-of-a-run.ts';
import { unitNameOfGiven, } from './running-pi-units.ts';
import { makeWorktree, } from './worktree-of-a-run.ts';

/**
 How git is asked a question in this command.
 */
const git = gitRunnerOf(runCommandWithChildProcess,);

/**
 The top of the checkout this command runs in, which is where a run's worktree is made unless the caller names another checkout.
 */
const checkoutFolder = await checkoutFolderOf({ from: process.cwd(), },);

/**
 What this run of the command was asked for.
 */
const options = parseWorktreeLauncherOptions({
  commandLine: argv.slice(2,),
  checkoutFolder,
},);

/**
 Whether the run works in the checkout itself, which is what a run that only reads does.
 */
const readOnlyInMain = options.branch === NO_WORKTREE_BRANCH;

/**
 The worktree the run works in.
 */
const worktree = readOnlyInMain
  ? options.checkoutFolder
  : worktreeFolderOf({
    checkoutFolder: options.checkoutFolder,
    runName: options.name,
  },);

/**
 The number the run's folder takes: the one the caller named, or the next number no folder of this name holds.
 */
const runNumber = options.runNumber ?? nextRunNumber({
  runName: options.name,
  entries: await readdir(options.runsFolder,),
},);

/**
 Where the driver writes this run.
 */
const runFolder = join(
  options.runsFolder,
  `${options.name}-${String(runNumber,)}`,
);

/**
 Waits between two looks at the unit's state, which is what keeps this command up while the run works.

 @param milliseconds - how long to wait

 @example
 ```ts
 await waitBetweenLooks(60_000);
 ```
 */
async function waitBetweenLooks(milliseconds: number,): Promise<void> {
  await wait(milliseconds,);
}

/**
 Reads the turn cap from the finished runs of this kind, or stops the run when no finished run says how long this work takes.

 @returns the cap, in turns

 @throws CommandAnswerMissingError when nothing has finished to size a cap from, because a guessed cap would end a working run

 @example
 ```ts
 const cap = await sizedTurnCap();
 ```
 */
async function sizedTurnCap(): Promise<number> {
  /**
   What the finished runs say about this kind of work.
   */
  const sizing = turnCapSizing({
    runName: options.name,
    finished: await finishedTurnsIn({ runsFolder: options.runsFolder, },),
  },);
  if (sizing === NO_FINISHED_RUN) {
    throw new CommandAnswerMissingError(
      `no finished run in ${options.runsFolder} says how many turns ${options.name} takes, so --turn-cap is required. Read what the finished runs of this kind used, and pass a cap a quarter above their longest.`,
    );
  }
  console.log(`turn cap: longest of ${String(sizing.sampleRuns,)} finished runs (${sizing.sampleKind}) used ${String(sizing.longestTurns,)}`,);
  return Math.ceil(sizing.longestTurns * TURN_CAP_MARGIN,);
}

/**
 Every reason this start is refused, which is read before anything is made: a start that was refused leaves no worktree, no run
 folder and no unit, and spends no provider slot on a run that could not have worked.
 */
const refusals = await startRefusalsIn({
  promptFile: options.promptFile,
  git,
  checkoutFolder: options.checkoutFolder,
  base: options.base,
  run: runCommandWithChildProcess,
  runName: options.name,
},);
if (refusals.length > 0) {
  throw new StartRefusedError(`${options.name} was not started:\n${refusals.join('\n')}`,);
}

/**
 The `PATH` the session's own processes get, which a systemd unit does not inherit from the shell that started this command.
 */
const pathOfMiseEnvironment = await requiredOutputOf({
  run: runCommandWithChildProcess,
  command: 'mise',
  words: [
    'exec',
    '--',
    'printenv',
    'PATH',
  ],
  folder: options.checkoutFolder,
},);

if (!readOnlyInMain) {
  console.log(await makeWorktree({
    git,
    checkoutFolder: options.checkoutFolder,
    worktree,
    branch: options.branch,
    base: options.base,
  },),);
}

try {
  console.log(`git: ${await refuseUnlessTheWrapperAnswersGit({
    run: runCommandWithChildProcess,
    sessionPath: pathOfMiseEnvironment,
    workingFolder: worktree,
  },)} answers as the policy wrapper`,);
}
catch (error) {
  // Issue 1768: a session whose git is the system binary never runs the wrapper, so this run is refused before anything is
  // started rather than started without the policies, the guards against discarding work and the automatic push. The check
  // refuses with an error of this package's own, whose message says everything a reader needs; any other failure is not a
  // refusal and is left as the uncaught error it is.
  if (!Error.isError(error,)) {
    throw error;
  }
  console.error(`${options.name}: refusing to start: ${error.message}`,);
  process.exit(1,);
}

if (!readOnlyInMain) {
  /**
   What linking the worktree's dependencies answered.
   */
  const linked = await runCommandWithChildProcess({
    command: 'mise',
    words: [
      'exec',
      '--',
      'pnpm',
      'install',
      '--offline',
      '--frozen-lockfile',
    ],
    folder: worktree,
  },);
  console.log(`dependencies: status ${String(linked.status,)}`,);
}

await mkdir(
  runFolder,
  { recursive: true, },
);

/**
 How many model turns the run may start.
 */
const turnCap = options.turnCap ?? await sizedTurnCap();

/**
 The unit that runs the driver, with the `PATH` and the node a mise environment gives Pi.
 */
const unit = unitCommandOf({
  runName: options.name,
  cpuPercent: options.cpuPercent,
  memory: options.memory,
  taskCount: options.taskCount,
  wallSeconds: options.wallSeconds,
  pathOfMiseEnvironment,
  home: env.HOME ?? '',
  node: await requiredOutputOf({
    run: runCommandWithChildProcess,
    command: 'mise',
    words: [
      'exec',
      '--',
      'bash',
      '--noprofile',
      '--norc',
      '-c',
      'command -v node',
    ],
    folder: options.checkoutFolder,
  },),
  driverEntry: await siblingCommandEntry({
    folder: import.meta.dirname,
    command: 'drive-pi-subagent',
  },),
  runFolder,
  workingFolder: worktree,
  promptFile: options.promptFile,
  provider: options.provider,
  model: options.model,
  idleSeconds: options.idleSeconds,
  turnCap,
  ...(options.session === undefined ? {} : { session: options.session, }),
},);

/**
 The process that follows the unit, which this launcher ends once the unit has ended rather than waiting on the manager's own
 signals.
 */
const waiter = spawn(
  unit.command,
  [...unit.words,],
  {
    stdio: 'inherit',
    env,
  },
);
/**
 What ended the wait for the unit.
 */
const waited = await waitForUnitToEnd({
  run: runCommandWithChildProcess,
  unitName: unitNameOfGiven({ name: options.name, },),
  deadline: Date.now() + ((options.wallSeconds + UNIT_WAIT_GRACE_SECONDS) * MILLISECONDS_PER_SECOND),
  wait: waitBetweenLooks,
  now: Date.now,
  endWaiter: function endWaiter() {
    // Only the waiter is ended: the unit is the manager's, and it ends on its own terms or at its own RuntimeMaxSec.
    waiter.kill('SIGTERM',);
  },
  waiterIsGone: function waiterIsGone() {
    return (waiter.exitCode !== null) || (waiter.signalCode !== null);
  },
},);
/**
 The status this command answers with: the unit's own when its waiter ended in time, and a failure when the wait gave up.
 */
const status = (waited.ended === 'the-unit-ended')
  ? (waiter.exitCode ?? 0)
  : 1;
console.log(`${options.name}: unit ${waited.ended} with status ${String(status,)}; run folder ${runFolder}; worktree ${worktree}`,);

/**
 What finishing the run answered, which is the status this command exits with.
 */
const finished = await finishTheRun({
  runName: options.name,
  runFolder,
  runsFolder: options.runsFolder,
  wallSeconds: options.wallSeconds,
  readOnlyInMain,
  worktree,
  checkoutFolder: options.checkoutFolder,
  processFolder: env.PI_RUNS_PROCESS_FOLDER ?? '/proc',
  commandLine: argv.slice(2,),
  launcherEntry: argv[1] ?? '',
  wakesSoFar: Number(env.PI_RUN_WAKES ?? '0',),
  environment: env,
  unitStatus: status,
  git,
  run: runCommandWithChildProcess,
  report: function report(line,) {
    console.log(line,);
  },
  wait: waitBetweenLooks,
  now: Date.now,
},);
process.exitCode = finished.status;
