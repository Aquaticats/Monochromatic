/**
 `status.ts`: prints observed versus locked tool versions and exits non-zero
 on drift.

 @example
 ```bash
 node package/file-enforcer/core/src/toolchain/status.ts
 ```
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  globalToolchainPaths,
  resolveToolchainPaths,
  type PathEnv,
  type ToolchainPaths,
  isMainModule,
} from './paths.ts';
import {
  ALL_TOOLS,
} from './plan.ts';
import { loadPlanFromDir, } from './plan.load.ts';
import {
  NOT_INSTALLED,
  ROUTE_OBSERVERS,
} from './status.observe.ts';
import type { ToolPlanEntry, } from './types.ts';
import {
  keptNewerWarning,
  lockOutcome,
} from './lock.outcome.ts';
import { l, } from '../logger.ts';

export {
  normalizeVersion,
  versionMatches,
} from './version.ts';

//region Observation

/**
 Observed state of one tool.
 */
export type ToolObservation = {
  readonly id: string;
  readonly locked: string;
  /**
   Observed version text; absent when nothing is installed.
   */
  readonly observed?: string;
  /**
   Whether the observed version satisfies the lock: it matches, or it is a newer repository-published tool that stays.
   */
  readonly inSync: boolean;
  /**
   Whether the installed version is a repository-published tool newer than the lock, kept instead of downgraded.
   */
  readonly keptNewer: boolean;
};

/**
 Probes the observed version of one plan entry on this machine.

 @param entry - Plan entry to probe

 @param paths - Resolved toolchain paths of the scope being observed

 @param env - Environment the probe resolves manager homes from

 @param tools - Plan of the scope being observed; absent means the repository plan

 @returns Observed version text, or {@link NOT_INSTALLED} when the tool is absent

 @example
 ```ts
 await observeTool({ entry: toolById('node',), paths: resolveToolchainPaths(), env: process.env, },); // '26.12.0'
 ```
 */
export async function observeTool(
  {
    entry,
    paths,
    env,
    tools = ALL_TOOLS,
  }: {
    readonly entry: ToolPlanEntry;
    readonly paths: ToolchainPaths;
    readonly env: PathEnv;
    readonly tools?: readonly ToolPlanEntry[];
  },
): Promise<string | typeof NOT_INSTALLED> {
  return await ROUTE_OBSERVERS[entry.route]({
    entry,
    paths,
    env,
    tools,
  },);
}

/**
 Observes one plan entry against its locked version.

 @param entry - Plan entry to observe

 @param paths - Resolved toolchain paths of the scope being observed

 @param env - Environment the probe resolves manager homes from

 @param tools - Plan sibling lookups resolve against

 @returns The entry's observation
 */
async function observeEntry(
  {
    entry,
    paths,
    env,
    tools,
  }: {
    readonly entry: ToolPlanEntry;
    readonly paths: ToolchainPaths;
    readonly env: PathEnv;
    readonly tools: readonly ToolPlanEntry[];
  },
): Promise<ToolObservation> {
  /**
   Observed version for this entry.
   */
  const observed = await observeTool({
    entry,
    paths,
    env,
    tools,
  },);
  /**
   What the sync would do with the observed version.
   */
  const outcome = lockOutcome({
    entry,
    observed: (observed === NOT_INSTALLED)
      ? undefined
      : observed,
  },);
  if ((outcome === 'kept-newer') && (observed !== NOT_INSTALLED)) {
    tagged({
      tag: observeEntry.name,
      l,
    },)
      .warn(keptNewerWarning({
        id: entry.id,
        observed,
        locked: entry.version,
      },),);
  }
  return {
    id: entry.id,
    locked: entry.version,
    ...(observed === NOT_INSTALLED
      ? {}
      : { observed, }),
    inSync: (outcome === 'at-lock') || (outcome === 'kept-newer'),
    keptNewer: outcome === 'kept-newer',
  };
}

/**
 Observes every plan entry against its locked version.

 @param entries - Plan entries to observe; the repository plan when absent

 @param tools - Plan sibling lookups resolve against; defaults to `entries`

 @param paths - Resolved toolchain paths of the scope being observed

 @param env - Environment the probes resolve manager homes from

 @returns One observation per plan entry, in plan order

 @example
 ```ts
 const observations = await observeTools({ paths: resolveToolchainPaths(), env: process.env, },);
 observations.every((observation,) => observation.inSync,);
 ```
 */
export async function observeTools(
  {
    entries = ALL_TOOLS,
    tools = entries,
    paths,
    env,
  }: {
    readonly entries?: readonly ToolPlanEntry[];
    readonly tools?: readonly ToolPlanEntry[];
    readonly paths: ToolchainPaths;
    readonly env: PathEnv;
  },
): Promise<readonly ToolObservation[]> {
  /**
   Observations built in plan order.
   */
  const observations: ToolObservation[] = [];
  for (const entry of entries) {
    // oxlint-disable-next-line no-await-in-loop -- each probe runs a manager CLI, so one at a time keeps process fan-out at one
    observations.push(await observeEntry({
      entry,
      paths,
      env,
      tools,
    },),);
  }
  return observations;
}

//endregion Observation

//region CLI

/**
 Entry point: prints the drift report on stdout.

 `--global` observes the machine-global scope
 (`doc/planning/mpm-global-toolchain-migration.md`).

 @example
 ```ts
 await main();
 ```
 */
export async function main(): Promise<void> {
  /**
   Whether this run targets the machine-global scope.
   */
  const global = process.argv
    .includes('--global',);
  /**
   Resolved scope paths.
   */
  const paths = global
    ? globalToolchainPaths({ env: process.env, },)
    : resolveToolchainPaths();
  /**
   Observed state of every plan entry.
   */
  const observations = await observeTools({
    entries: global
      ? await loadPlanFromDir({ dir: paths.worktreeRoot, },)
      : ALL_TOOLS,
    paths,
    env: process.env,
  },);
  for (const observation of observations) {
    if (observation.keptNewer) {
      process.stdout
        .write(
        `ok      ${observation.id}  locked=${observation.locked}  kept newer installed=${observation.observed}\n`,
        );
      continue;
    }
    if (observation.inSync) {
      process.stdout
        .write(`ok      ${observation.id}  ${observation.locked}\n`,);
      continue;
    }
    process.stdout
      .write(
      `drift   ${observation.id}  locked=${observation.locked}  observed=${
        observation.observed ?? '(absent)'
      }\n`,
      );
  }
  /**
   Number of drifted entries.
   */
  const drift = observations.filter(
    function hasDrifted(observation,): boolean {
      return !observation.inSync;
    },
  )
    .length;
  process.stdout
    .write(
    drift === 0
      ? `toolchain status: ${
        observations.length
      } tools in sync with toolchain.lock.json\n`
      : `toolchain status: ${drift} of ${
        observations.length
      } tools drifted from toolchain.lock.json\n`,
    );
  if (drift > 0)
    process.exitCode = 1;
}

//endregion CLI

if (await isMainModule(import.meta.filename,))
  await main();
