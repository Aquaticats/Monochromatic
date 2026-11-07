/**
 The configuration of one move, read from the environment with the plan's defaults. The root steps run through `sudo`
 with the absolute path of `node`, because `sudo node` fails when node comes from the version manager and is not on
 sudo's search path (measured in the plan's section "The copy").

 @module
 */

import { execFile, } from 'node:child_process';
import { userInfo, } from 'node:os';
import { promisify, } from 'node:util';
import { join, } from 'node:path';
import { realpath, } from 'node:fs/promises';
import type { GuidedConfig, } from './guided-types.ts';
import { isPlainPath, } from './text-checks.ts';

/**
 Runs a program and waits for it to end.
 */
const execFileAsync = promisify(execFile,);

/**
 Transfer deadline in seconds: the plan's value, section "Deadlines".
 */
const PLAN_TRANSFER_DEADLINE_SECONDS = 14_400;

/**
 External rsync deadline in seconds: the plan's value, section "The copy".
 */
const PLAN_EXTERNAL_DEADLINE_SECONDS = 7_200;

/**
 Free space the 860 must keep, in GiB: the plan's value, section "Capacity policy".
 */
const PLAN_MIN_FREE_GIB = 150;

/**
 Free space the NVMe must keep while source snapshots exist, in GiB: the plan's value, section "Capacity policy".
 */
const PLAN_NVME_MIN_FREE_GIB = 100;

/**
 Margin on the size estimate in percent: one quarter, section "Capacity policy".
 */
const PLAN_MARGIN_PERCENT = 25;

/**
 Sharing limit in percent: the same quarter margin applied to the copy's footprint.
 */
const PLAN_SHARING_LIMIT_PERCENT = 125;

/**
 Seconds of one worktree scan: the plan's value, section "Quiesce and freeze".
 */
const PLAN_SCAN_PER_WORKTREE_SECONDS = 180;

/**
 Seconds of the whole scan: the plan's value, section "Quiesce and freeze".
 */
const PLAN_SCAN_TIMEOUT_SECONDS = 3_000;

/**
 Seconds the whole first pass may take before it stops with every record kept: the author's choice, a day; a rerun continues.
 */
const AUTHOR_FIRST_PASS_TOTAL_SECONDS = 86_400;

/**
 Seconds of one fixture step: the author's choice.
 */
const AUTHOR_FIXTURE_STEP_TIMEOUT_SECONDS = 1_800;

/**
 Seconds of one footprint measurement: the author's choice; the plan measured 12.5 minutes for the same `du`.
 */
const AUTHOR_MEASURE_TIMEOUT_SECONDS = 5_400;

/**
 Seconds of the size table: the author's choice.
 */
const AUTHOR_SIZE_TABLE_TIMEOUT_SECONDS = 7_200;

/**
 Grace between the terminate signal and the kill of a timed-out step: the plan's `timeout --kill-after=30`.
 */
const PLAN_KILL_GRACE_SECONDS = 30;

/**
 Resource bounds of a root scope: sized to one stream by the plan author.
 */
const ROOT_SCOPE = '--scope --collect --property=CPUQuota=200% --property=MemoryMax=2G --property=TasksMax=32 --property=IOWeight=10';

/**
 Resource bounds of a user scope.
 */
const USER_SCOPE = '--user --scope --property=CPUQuota=100% --property=MemoryMax=1G --property=TasksMax=64 --property=IOWeight=10';

/**
 Reads a numeric environment variable.

 @param env - the environment
 
 @param name - the variable
 
 @param fallback - the value when absent
 
 @returns the number
 
 @throws Error when the value is not a positive number
 */
function numberFrom(
  {
    env,
    name,
    fallback,
  }: {
    readonly env: Readonly<Record<string, string>>;
    readonly name: string;
    readonly fallback: number;
  },
): number {
  /**
   The text this step works on.
   */
  const text = env[name];
  if ((text === undefined) || (text === '')) {
    return fallback;
  }
  /**
   The value this step works on.
   */
  const value = Number(text,);
  if ((!Number.isFinite(value,)) || (value <= 0)) {
    throw new Error(`${name}=${text} is not a positive number`,);
  }
  return value;
}

/**
 Refuses a path with a character the shown commands would not quote.

 @param name - the setting
 
 @param value - the path
 
 @throws Error when the path holds an unquoted character
 */
function requirePlainPath(
  {
    name,
    value,
  }: {
    readonly name: string;
    readonly value: string;
  },
): void {
  if (!isPlainPath(value,)) {
    throw new Error(`The setting ${name}=${value} holds a character this runner does not quote; use a plain path.`,);
  }
}

/**
 Builds the configuration.

 @param env - the environment, read for overrides (`MOVE_*`)
 
 @param nodePath - the absolute path of the node binary that root steps run
 
 @returns the configuration
 
 @throws Error when a path holds a character the shown commands would not quote
 */
export async function createGuidedConfig(
  {
    env,
    nodePath,
  }: {
    readonly env: Readonly<Record<string, string>>;
    readonly nodePath: string;
  },
): Promise<GuidedConfig> {
  /**
   The user's home folder.
   */
  const homeDir = await realpath(env.HOME
    ?? userInfo()
      .homedir,);
  /**
   The repository path.
   */
  const repo = env.MOVE_REPOSITORY ?? join(
    homeDir,
    'Monochromatic',
  );
  /**
   The 860's mount point.
   */
  const drive = env.MOVE_DRIVE ?? '/var/mnt/encrypted';
  /**
   The folder on the 860 that holds the copy.
   */
  const destRoot = env.MOVE_DESTINATION_ROOT ?? join(
    drive,
    'repository',
  );
  /**
   The runner's state folder.
   */
  const moveState = env.MOVE_STATE_DIR ?? join(
    homeDir,
    'repository-move-state',
  );
  /**
   The folder of the move tools.
   */
  const toolsFolder = env.MOVE_TOOLS_DIR ?? join(
    repo,
    'package',
    'repository-move',
    'cli',
    'src',
  );
  /**
   The runner's configuration.
   */
  const config: GuidedConfig = {
    homeDir,
    userName: userInfo()
      .username,
    groupName: (await execFileAsync(
      'id',
      ['-gn',],
      { encoding: 'utf8', },
    )).stdout
      .trim(),
    repo,
    externalWorktrees: env.MOVE_EXTERNAL_WORKTREES ?? join(
      homeDir,
      'worktrees',
    ),
    drive,
    destRoot,
    received: join(
      destRoot,
      'received-read-only',
    ),
    finalPath: join(
      destRoot,
      'Monochromatic',
    ),
    externalFinal: join(
      destRoot,
      'external-worktrees',
    ),
    snapshots: env.MOVE_SNAPSHOTS ?? join(
      homeDir,
      'repository-move-snapshots',
    ),
    moveState,
    wiz: join(
      moveState,
      'wizard',
    ),
    toolsFolder,
    copyTool: join(
      toolsFolder,
      'move-repository.ts',
    ),
    scanTool: join(
      toolsFolder,
      'list-uncommitted-work.ts',
    ),
    inventory: join(
      moveState,
      'inventory-nested.txt',
    ),
    inventoryMain: env.MOVE_INVENTORY_MAIN ?? 'home/user/Monochromatic',
    fixtureParent: env.MOVE_FIXTURE_PARENT ?? join(
      homeDir,
      'temp',
      'agent',
    ),
    cliGitState: join(
      env.XDG_STATE_HOME ?? join(
        homeDir,
        '.local',
        'state',
      ),
      'cli-git',
      'service',
    ),
    nodePath,
    linkBinFolder: join(
      env.XDG_DATA_HOME ?? join(
        homeDir,
        '.local',
        'share',
      ),
      'monochromatic-dev',
      'repository-link',
    ),
    unitPath: env.MOVE_MOUNT_UNIT ?? '/etc/systemd/system/mount-860.service',
    firstPassTotalSeconds: numberFrom({
      env,
      name: 'MOVE_FIRST_PASS_TOTAL_SECONDS',
      fallback: AUTHOR_FIRST_PASS_TOTAL_SECONDS,
    },),
    transferDeadlineSeconds: numberFrom({
      env,
      name: 'MOVE_TRANSFER_DEADLINE_SECONDS',
      fallback: PLAN_TRANSFER_DEADLINE_SECONDS,
    },),
    externalDeadlineSeconds: numberFrom({
      env,
      name: 'MOVE_EXTERNAL_DEADLINE_SECONDS',
      fallback: PLAN_EXTERNAL_DEADLINE_SECONDS,
    },),
    minFreeGiB: numberFrom({
      env,
      name: 'MOVE_MIN_FREE_GIB',
      fallback: PLAN_MIN_FREE_GIB,
    },),
    nvmeMinFreeGiB: numberFrom({
      env,
      name: 'MOVE_NVME_MIN_FREE_GIB',
      fallback: PLAN_NVME_MIN_FREE_GIB,
    },),
    marginPercent: PLAN_MARGIN_PERCENT,
    sharingLimitPercent: PLAN_SHARING_LIMIT_PERCENT,
    scanPerWorktreeSeconds: PLAN_SCAN_PER_WORKTREE_SECONDS,
    fixtureStepTimeoutSeconds: numberFrom({
      env,
      name: 'MOVE_FIXTURE_TIMEOUT_SECONDS',
      fallback: AUTHOR_FIXTURE_STEP_TIMEOUT_SECONDS,
    },),
    measureTimeoutSeconds: numberFrom({
      env,
      name: 'MOVE_MEASURE_TIMEOUT_SECONDS',
      fallback: AUTHOR_MEASURE_TIMEOUT_SECONDS,
    },),
    sizeTableTimeoutSeconds: numberFrom({
      env,
      name: 'MOVE_SIZE_TABLE_TIMEOUT_SECONDS',
      fallback: AUTHOR_SIZE_TABLE_TIMEOUT_SECONDS,
    },),
    scanTimeoutSeconds: numberFrom({
      env,
      name: 'MOVE_SCAN_TIMEOUT_SECONDS',
      fallback: PLAN_SCAN_TIMEOUT_SECONDS,
    },),
    killGraceSeconds: PLAN_KILL_GRACE_SECONDS,
    windowMaxOverrideSeconds: numberFrom({
      env,
      name: 'MOVE_WINDOW_MAX_SECONDS',
      fallback: 0,
    },),
    rootScope: ROOT_SCOPE,
    userScope: USER_SCOPE,
  };
  for (const [name, value,] of Object.entries({
    repo: config.repo,
    externalWorktrees: config.externalWorktrees,
    drive: config.drive,
    destRoot: config.destRoot,
    snapshots: config.snapshots,
    moveState: config.moveState,
    toolsFolder: config.toolsFolder,
    fixtureParent: config.fixtureParent,
    cliGitState: config.cliGitState,
    nodePath: config.nodePath,
    unitPath: config.unitPath,
  },)) {
    requirePlainPath({
      name,
      value,
    },);
  }
  return config;
}
