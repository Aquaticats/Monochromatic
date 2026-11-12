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
  readFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import {
  argv,
  env,
} from 'node:process';
import { setTimeout as wait, } from 'node:timers/promises';
import { errorCodeOf, } from './error-code-of.ts';
import {
  fieldsOf,
  isRecord,
} from './read-pi-fields.ts';
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
import {
  makeWorktree,
  removeWorktree,
  worktreeOutcomeOf,
} from './worktree-of-a-run.ts';

/**
 The code the operating system gives a path that is not there.
 */
const MISSING_PATH_CODE = 'ENOENT';

/**
 How git is asked a question in this command.
 */
const git = gitRunnerOf(runCommandWithChildProcess,);

/**
 The top of the checkout this command runs in, which is where a run's worktree is made unless the caller names another checkout.
 */
const checkoutFolder = await requiredOutputOf({
  run: runCommandWithChildProcess,
  command: 'git',
  words: [
    'rev-parse',
    '--show-toplevel',
  ],
},);

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
 Reads why a run ended from the summary it wrote.

 @param folder - the run's folder

 @returns the end reason, or nothing when no summary is there

 @example
 ```ts
 const reason = await endReasonIn('/runs/a-1');
 ```
 */
async function endReasonIn(folder: string,): Promise<{ readonly endReason?: string }> {
  try {
    /**
     The summary, which the driver writes whole and last.
     */
    const shown = await readFile(
      join(
        folder,
        'summary.json',
      ),
      'utf8',
    );
    /**
     What the run's summary holds, of any shape.
     */
    const summary: unknown = JSON.parse(shown,);
    if (!isRecord(summary,)) {
      throw new Error(`the summary of ${folder} is no JSON object, so why that run ended cannot be read`,);
    }
    /**
     Why that run ended, as its summary says.
     */
    const { endReason, } = fieldsOf(summary,);
    return ((typeof endReason) === 'string') ? { endReason, } : {};
  }
  catch (error) {
    // A run that wrote no summary ended before the driver could write one, so there is no end reason to weigh, and the file's own
    // absence is what says so.
    if (errorCodeOf(error,) !== MISSING_PATH_CODE) {
      throw error;
    }
    return {};
  }
}

/**
 Says whether the worktree's HEAD is a branch, which a removal needs: a detached HEAD's commits may be reachable from this
 worktree alone.

 @returns whether the worktree's HEAD is a branch

 @example
 ```ts
 const onABranch = await onABranchNow();
 ```
 */
async function onABranchNow(): Promise<boolean> {
  if (readOnlyInMain) {
    return false;
  }
  /**
   What git answered about the worktree's HEAD.
   */
  const answered = await runCommandWithChildProcess({
    command: 'git',
    words: [
      'symbolic-ref',
      '--quiet',
      'HEAD',
    ],
    folder: worktree,
  },);
  return (answered.status === 0) && (answered.stdout
    .trim()
    !== '');
}

/**
 Reads the worktree's git status, when a removal turns on it.

 @param endReason - why the run ended, or nothing when it wrote no summary

 @param onABranch - whether the worktree's HEAD is a branch

 @returns how status ended and what it printed, or the same fields as a status nobody needed

 @example
 ```ts
 const status = await statusOf({ endReason: 'idle', onABranch: true, });
 ```
 */
async function statusOf({
  endReason,
  onABranch,
}: {
  readonly endReason?: string;
  readonly onABranch: boolean;
}): Promise<{
  readonly status: number;
  readonly statusText: string
}> {
  if (
    readOnlyInMain
      || (!onABranch)
      || ((endReason !== 'idle') && (endReason !== 'stop-file'))
  ) {
    return {
      status: 0,
      statusText: '',
    };
  }
  /**
   What git said about the worktree's files.
   */
  const shown = await runCommandWithChildProcess({
    command: 'git',
    words: [
      '--no-optional-locks',
      'status',
      '--porcelain=v1',
      '--untracked-files=normal',
    ],
    folder: worktree,
  },);
  return {
    status: shown.status,
    statusText: shown.stdout,
  };
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
 Why the run ended, or nothing when it wrote no summary.
 */
const { endReason, } = await endReasonIn(runFolder,);

/**
 Whether the worktree's HEAD is a branch.
 */
const onABranch = await onABranchNow();

/**
 What is to be done with the worktree.
 */
const outcome = worktreeOutcomeOf({
  readOnlyInMain,
  ...(endReason === undefined ? {} : { endReason, }),
  onABranch,
  ...await statusOf({
    ...(endReason === undefined ? {} : { endReason, }),
    onABranch,
  },),
},);
if (outcome.action === 'keep') {
  console.log(`${options.name}: ${outcome.line}`,);
}
if (outcome.action === 'remove') {
  console.log(`${options.name}: ${await removeWorktree({
    git,
    checkoutFolder: options.checkoutFolder,
    worktree,
  },)}`,);
}
process.exitCode = status;
