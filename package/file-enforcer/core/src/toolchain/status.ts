/**
 `status.ts`: prints observed versus locked tool versions and exits non-zero
 on drift.

 @example
 ```bash
 node package/file-enforcer/core/src/toolchain/status.ts
 ```
 */

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

//region Version comparison

/**
 Letters and digits a release-tag prefix (`bun-v1.3.14`) may contain besides its separators.
 */
const TAG_PREFIX_CHARACTERS = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789._-';

/**
 ASCII letters, which a release-tag prefix must start with.
 */
const ASCII_LETTERS = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz';

/**
 ASCII digits.
 */
const ASCII_DIGITS = '0123456789';

/**
 Whether text is exactly one ASCII digit.

 @param ch - Text to test

 @returns `true` for `0` to `9`
 */
function isAsciiDigit(ch: string,): boolean {
  return (ch.length === 1) && ASCII_DIGITS.includes(ch,);
}

/**
 Index of the character that ends the leading release-tag prefix run: a letter followed by letters, digits,
 `.`, `_` and `-`.

 @param version - Raw version text

 @returns Length of the prefix run; zero when the text does not start with a letter
 */
function tagPrefixRunLength(version: string,): number {
  if ((version.length === 0) || (!ASCII_LETTERS.includes(version.charAt(0,),)))
    return 0;
  for (let index = 1; index < version.length; index += 1) {
    if (!TAG_PREFIX_CHARACTERS.includes(version.charAt(index,),))
      return index;
  }
  return version.length;
}

/**
 Normalizes a version string for comparison: drops one leading `v`, and the
 release-tag prefix before it (`bun-v1.3.14` becomes `1.3.14`).

 Release tags carry prefixes (`v1.9.0`, `bun-v1.3.14`); the lock holds the bare version. Anything without a
 v-before-digit stays as written. When several `-v<digit>` boundaries could end the prefix, the last one wins.

 @param version - Raw version text

 @returns Normalized version text

 @example
 ```ts
 normalizeVersion('bun-v1.3.14',); // '1.3.14'
 ```
 */
export function normalizeVersion(version: string,): string {
  /**
   Length of the leading run that may form a tag prefix.
   */
  const runLength = tagPrefixRunLength(version,);
  // A prefix is a letter, more prefix characters, then a `-` right before `v<digit>`.
  for (let dash = runLength - 1; dash >= 1; dash -= 1) {
    if ((version.charAt(dash,) === '-')
      && (version.charAt(dash + 1,) === 'v')
      && isAsciiDigit(version.charAt(dash + 2,),))
      return version.slice(dash + 2,);
  }
  return (version.startsWith('v',) && isAsciiDigit(version.charAt(1,),))
    ? version.slice(1,)
    : version;
}

/**
 Whether an observed version satisfies a locked version:
 equal after normalization, or the observed value extends the lock with a
 platform suffix (`nightly-2026-10-21-x86_64-unknown-linux-gnu`).

 @param observed - Version text the machine reports

 @param locked - Version text the lock records

 @returns `true` when the observed version matches the lock

 @example
 ```ts
 versionMatches({ observed: 'v1.9.0', locked: '1.9.0', },); // true
 ```
 */
export function versionMatches(
  {
    observed,
    locked,
  }: {
    readonly observed: string;
    readonly locked: string;
  },
): boolean {
  /**
   Normalized observed version.
   */
  const o = normalizeVersion(observed,);
  /**
   Normalized locked version.
   */
  const l = normalizeVersion(locked,);
  return (o === l) || o.startsWith(`${l}-`,)
    || l.startsWith(`${o}-`,);
}

//endregion Version comparison

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
   Whether the observed version matches the lock.
   */
  readonly inSync: boolean;
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
  return {
    id: entry.id,
    locked: entry.version,
    ...(observed === NOT_INSTALLED
      ? {}
      : { observed, }),
    inSync: (observed !== NOT_INSTALLED)
      && versionMatches({
        observed,
        locked: entry.version,
      }),
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
