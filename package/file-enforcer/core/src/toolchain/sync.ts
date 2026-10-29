/**
 `sync.ts`: converges every tool to `toolchain.lock.json`.

 The sync is lock-constrained the way `pnpm install` honors `pnpm-lock.yaml`:
 it installs the versions the lock fixes and never re-resolves (bumping the
 lock is a separate update action). Staging order: the pinned `bin` CLI, `uv`,
 then the pinned `mpm`, the Rust toolchain (cargo's home), `cargo-binstall`,
 Node, and pnpm, then every remaining entry in plan order.

 @example
 ```bash
 node package/file-enforcer/core/src/toolchain/sync.ts
 ```
 */

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import { l, } from '../logger.ts';
import { hostPlatformId, } from './layout.ts';
import {
  rebuildExposure,
  runPostinstallCommands,
  runPostinstallLinks,
  unresolvableBins,
} from './exposure.ts';
import { writeToolEnvFile, } from './exposure.env.ts';
import {
  globalToolchainPaths,
  homeDirFromEnv,
  resolveToolchainPaths,
  type PathEnv,
  type ToolchainPaths,
  isMainModule,
} from './paths.ts';
import { ALL_TOOLS, } from './plan.ts';
import { loadPlanFromDir, } from './plan.load.ts';
import {
  artifactForHost,
  installAndroidSdk,
  installArchive,
  installBinSourced,
  installCargoBinstall,
  installCargoGit,
  installGoInstall,
  installMpmPnpm,
  installMpmUvx,
  installRustComponents,
  installRustToolchain,
  type ProviderContext,
} from './providers.ts';
import { pruneStaleOwnCrates, } from './prune.ts';
import { NOT_INSTALLED, } from './status.observe.ts';
import {
  appliesToHost,
  syncOrder,
} from './sync.order.ts';
import {
  observeTool,
  versionMatches,
} from './status.ts';
import type {
  ToolPlanEntry,
  ToolRoute,
} from './types.ts';

//region Sync

/**
 Outcome of one `toolchain:sync` run.
 */
export type SyncReport = {
  /**
   Tool ids that were already at their locked version.
   */
  readonly unchanged: readonly string[];
  /**
   Tool ids installed or converged in this run.
   */
  readonly installed: readonly string[];
  /**
   Tool ids whose install failed, with the failing output.
   */
  readonly failed: Readonly<Record<string, string>>;
  /**
   Declared bin names that do not resolve in the exposure directory.
   */
  readonly missingBins: readonly string[];
};

/**
 Installer for each install route.
 */
const INSTALLERS: Readonly<Record<ToolRoute, (context: ProviderContext,) => Promise<unknown>>> = {
  'bin-sourced': installBinSourced,
  'archive': installArchive,
  'cargo-binstall': installCargoBinstall,
  'cargo-git': installCargoGit,
  'mpm-pnpm': installMpmPnpm,
  'mpm-uvx': installMpmUvx,
  'go-install': installGoInstall,
  'rustup-toolchain': installRustToolchain,
  'rustup-components': installRustComponents,
  'android-sdk': installAndroidSdk,
};

/**
 The warning for an install that would replace a different installed version.

 Trees whose locks pin different versions of one tool share one install
 (the `cargo binstall` record under `~/.cargo`), so installing the locked
 version here changes the tool for every tree that shares it.

 @param id - Plan entry id the warning names

 @param observed - Version the machine reports, or the absent marker

 @param locked - Version this tree's lock pins

 @returns One warning line when a different version is installed, none when the tool is absent or already at the lock

 @example
 ```ts
 replacementWarning({ id: 'cargo:monochromatic-lint', observed: '0.4.0', locked: '0.3.0', },);
 // ['cargo:monochromatic-lint is installed at 0.4.0 but this tree\'s lock pins 0.3.0; ...']
 ```
 */
export function replacementWarning(
  {
    id,
    observed,
    locked,
  }: {
    readonly id: string;
    readonly observed: string | typeof NOT_INSTALLED;
    readonly locked: string;
  },
): readonly string[] {
  if ((observed === NOT_INSTALLED) || versionMatches({
    observed,
    locked,
  },))
    return [];
  return [
    `${id} is installed at ${observed} but this tree's lock pins ${locked}; installing ${locked} replaces it `
      + 'for every tree that shares this install, and a tree whose lock pins the other version replaces it back on its next sync',
  ];
}

/**
 Whether one entry already sits at its locked version.

 A probe that cannot run (its own staging prerequisite missing) reads as absent, and the install converges it.
 An installed version that differs from the lock logs a warning, because the install that follows replaces it.

 @param entry - Plan entry to probe

 @param paths - Resolved toolchain paths of the scope

 @param env - Environment the probe resolves manager homes from

 @param tools - Plan the probe resolves sibling tools against

 @returns `true` when the observed version matches the lock
 */
async function isAtLockedVersion(
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
): Promise<boolean> {
  /**
   Logger tagged with this probe.
   */
  const log = tagged({
    tag: isAtLockedVersion.name,
    l,
  },);
  try {
    /**
     Observed version before installing.
     */
    const observed = await observeTool({
      entry,
      paths,
      env,
      tools,
    },);
    for (const warning of replacementWarning({
      id: entry.id,
      observed,
      locked: entry.version,
    },))
      log.warn(warning,);
    return (observed !== NOT_INSTALLED)
      && versionMatches({
        observed,
        locked: entry.version,
      },);
  } catch (error) {
    log.debug(`${entry.id} could not be probed, installing it: ${caughtValueText(error,)}`,);
    return false;
  }
}

/**
 Installs one entry through its route, then adds the Rust components and targets its options ask for.

 @param entry - Plan entry to install

 @param paths - Resolved toolchain paths of the scope

 @param env - Environment the installers run with

 @param tools - Plan the installers resolve sibling tools against

 @throws Error when the route's installer fails
 */
async function installEntry(
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
): Promise<void> {
  /**
   Provider context for this entry.
   */
  const context: ProviderContext = {
    entry,
    paths,
    env,
    extraDirs: [paths.toolchainBinDir],
    tools,
  };
  await INSTALLERS[entry.route](context,);
  if ((entry.options
    .rustComponents
    !== undefined)
    || (entry.options
      .rustTargets
      !== undefined))
    await installRustComponents(context,);
}

/**
 Runs the full sync algorithm against one worktree.

 @param tools - Plan entries to converge; the repository plan when absent

 @param paths - Resolved toolchain paths of the scope

 @param env - Environment overrides; the process environment when absent

 @returns The sync report

 @example
 ```ts
 const report = await syncToolchain({ paths: resolveToolchainPaths(), },);
 report.failed; // {}
 ```
 */
export async function syncToolchain(
  {
    tools = ALL_TOOLS,
    paths,
    env,
  }: {
    readonly tools?: readonly ToolPlanEntry[];
    readonly paths: ToolchainPaths;
    readonly env?: PathEnv;
  },
): Promise<SyncReport> {
  /**
   Effective environment.
   */
  const e = env ?? process.env;
  /**
   Plan entries applying to this host, in sync order.
   */
  const entries = syncOrder({ entries: tools, },)
    .filter(function appliesHere(entry,) {
      return appliesToHost(entry,);
    },);
  /**
   Already-converged tool ids.
   */
  const unchanged: string[] = [];
  /**
   Installed or converged tool ids.
   */
  const installed: string[] = [];
  /**
   Failed tool ids to their failing output.
   */
  const failed: Record<string, string> = {};

  for (const entry of entries) {
    // oxlint-disable-next-line no-await-in-loop -- staging order makes later probes depend on earlier installs
    if (await isAtLockedVersion({
      entry,
      paths,
      env: e,
      tools,
    },)) {
      unchanged.push(entry.id,);
      continue;
    }
    try {
      // oxlint-disable-next-line no-await-in-loop -- installers share manager state and staging order, so one at a time
      await installEntry({
        entry,
        paths,
        env: e,
        tools,
      },);
      installed.push(entry.id,);
      // oxlint-disable-next-line no-await-in-loop -- the prune follows its own install, before the next entry starts
      await pruneStaleOwnCrates({
        entry,
        tools,
        env: e,
      },);
    } catch (error) {
      failed[entry.id] = caughtValueText(error,);
    }
  }

  // Postinstall steps (the `cli-git-bin` atomic `git` link).
  await runPostinstallLinks({
    paths,
    entries,
    env: e,
    extraDirs: [paths.toolchainBinDir],
  },);

  // Rebuild the exposure directory and the tool environment exports.
  await rebuildExposure({
    paths,
    entries,
    env: e,
    extraDirs: [paths.toolchainBinDir],
  },);
  await writeToolEnvFile({
    paths,
    entries,
  },);

  /**
   Outcome of the postinstall commands. They run last because they find the
   other tools through the rebuilt exposure directory.
   */
  const commands = await runPostinstallCommands({
    paths,
    entries,
    env: e,
    extraDirs: [paths.toolchainBinDir],
  },);
  for (const [id, output,] of Object.entries(commands.failed,))
    failed[id] = output;

  return {
    unchanged,
    installed,
    failed,
    missingBins: await unresolvableBins({
      paths,
      entries,
    },),
  };
}

//endregion Sync

//region CLI

/**
 Entry point: prints the drift report on stdout.
 
 `--global` syncs the machine-global scope
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
   Plan entries of the scope being synced.
   */
  const tools = global
    ? await loadPlanFromDir({ dir: paths.worktreeRoot, },)
    : ALL_TOOLS;
  process.stdout
    .write(
    `toolchain sync: platform ${
      hostPlatformId()
    }, root ${
      paths.toolchainRoot
    }, exposure ${
      paths.exposureDir
    }\n`,
    );
  /**
   Sync report for this run.
   */
  const report = await syncToolchain({
    tools,
    paths,
    env: process.env,
  },);
  process.stdout
    .write(
    `toolchain sync: ${
      report.installed
        .length
    } installed, ${
      report.unchanged
        .length
    } unchanged, ${
      Object.keys(report.failed,)
        .length
    } failed\n`,
    );
  for (const [id, message,] of Object.entries(report.failed,))
    process.stderr
      .write(`toolchain sync: ${id} failed: ${message}\n`,);
  if (report.missingBins
    .length
    > 0)
    process.stderr
      .write(
      `toolchain sync: unresolved bins: ${
        report.missingBins
          .join(', ',)
      }\n`,
      );
  if (Object.keys(report.failed,)
    .length
    > 0)
    process.exitCode = 1;
}

//endregion CLI

if (await isMainModule(import.meta.filename,))
  await main();
