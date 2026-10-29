/**
 `sync.ts`: converges every tool to `toolchain.lock.json`.

 The sync is lock-constrained the way `pnpm install` honors `pnpm-lock.yaml`:
 it installs the versions the lock fixes and never re-resolves (bumping the
 lock is a separate update action). Staging order: the pinned `bin` CLI, `uv`,
 then the pinned `mpm`, the Rust toolchain (cargo's home), `cargo-binstall`,
 Node, and pnpm, then every remaining entry in plan order.

 @example
 ```bash
 node package/dev-script/file-enforcer/src/toolchain/sync.ts
 ```
 */

import { join, } from 'node:path';
import { hostPlatformId, } from './layout.ts';
import { run, } from './exec.ts';
import {
  rebuildExposure,
  runPostinstallLinks,
  unresolvableBins,
  writeToolEnvFile,
} from './exposure.ts';
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
import {
  observeTool,
  versionMatches,
} from './status.ts';
import type { ToolPlanEntry, } from './types.ts';

//region Staging order

/**
 Tool ids staged before the plan-order remainder, in staging order.
 `rustup` precedes `rust` and `go` precedes any `go-install` tool, which the
 global-scope plan (`doc/planning/mpm-global-toolchain-migration.md`) needs.
 */
const STAGING_IDS: readonly string[] = [
  'bin',
  'uv',
  'mpm',
  'rustup',
  'rust',
  'cargo-binstall',
  'go',
  'node',
  'pnpm',
] as const;

/**
 Orders plan entries for installation: the staging prerequisites first, then
 every remaining entry in plan order.
 
 @param entries - Plan entries to order
 
 @returns Ordered entries
 */
export function syncOrder(
  { entries, }: { readonly entries: readonly ToolPlanEntry[]; },
): readonly ToolPlanEntry[] {
  /**
   Staged entries, in staging order.
   */
  const staged: ToolPlanEntry[] = [];
  for (const id of STAGING_IDS) {
    /**
     Matching plan entry for this staging id.
     */
    const entry = entries.find(
      (candidate,) => candidate.id === id,
    );
    if (entry !== undefined)
      staged.push(entry,);
  }
  return [
    ...staged,
    ...entries.filter(
      function notStaged(entry,): boolean {
        return !STAGING_IDS.includes(entry.id,);
      },
    ),
  ];
}

//endregion Staging order

//region Platform filter

/**
 Whether one plan entry applies to the running host.
 
 @param entry - Plan entry to test
 
 @returns `true` when the entry's OS filter admits this platform
 */
export function appliesToHost(entry: ToolPlanEntry,): boolean {
  if (entry.os === null)
    return true;
  /**
   Platform family of the running host.
   */
  const family = process.platform === 'linux'
    ? 'linux'
    : process.platform === 'darwin'
      ? 'macos'
      : process.platform === 'win32'
        ? 'windows'
        : process.platform;
  return entry.os.includes(family as 'linux' | 'macos' | 'windows');
}

//endregion Platform filter

//region Sync

/**
 Outcome of one `toolchain:sync` run.
 */
export type SyncReport = {
  /** Tool ids that were already at their locked version. */
  readonly unchanged: readonly string[];
  /** Tool ids installed or converged in this run. */
  readonly installed: readonly string[];
  /** Tool ids whose install failed, with the failing output. */
  readonly failed: Readonly<Record<string, string>>;
  /** Declared bin names that do not resolve in the exposure directory. */
  readonly missingBins: readonly string[];
};

/**
 Runs the full sync algorithm against one worktree.
 
 @param options - Plan entries, paths and environment overrides
 
 @returns The sync report
 */
export async function syncToolchain(
  {
    tools = ALL_TOOLS,
    paths,
    env,
  }: {
    readonly tools?: readonly ToolPlanEntry[] | undefined;
    readonly paths: ToolchainPaths;
    readonly env?: PathEnv | undefined;
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
    .filter(appliesToHost,);
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
    /**
     Observed version before installing; a probe that cannot run (its own
     staging prerequisite missing) reads as absent, and the install below
     converges it.
     */
    let observed: string | null = null;
    try {
      observed = await observeTool({
        entry,
        paths,
        env: e,
        tools,
      },);
    } catch {
      observed = null;
    }
    if (observed !== null
      && versionMatches({
        observed,
        locked: entry.version,
      },)) {
      unchanged.push(entry.id,);
      continue;
    }
    /**
     Provider context for this entry.
     */
    const context: ProviderContext = {
      entry,
      paths,
      env: e,
      extraDirs: [paths.toolchainBinDir],
      tools,
    };
    try {
      switch (entry.route) {
        case 'bin-sourced':
          await installBinSourced(context,);
          break;
        case 'archive':
          await installArchive(context,);
          break;
        case 'cargo-binstall':
          await installCargoBinstall(context,);
          break;
        case 'cargo-git':
          await installCargoGit(context,);
          break;
        case 'mpm-pnpm':
          await installMpmPnpm(context,);
          break;
        case 'mpm-uvx':
          await installMpmUvx(context,);
          break;
        case 'go-install':
          await installGoInstall(context,);
          break;
        case 'rustup-toolchain':
          await installRustToolchain(context,);
          break;
        case 'rustup-components':
          await installRustComponents(context,);
          break;
        case 'android-sdk':
          await installAndroidSdk(context,);
          break;
      }
      if (entry.options.rustComponents !== undefined
        || entry.options.rustTargets !== undefined)
        await installRustComponents(context,);
      installed.push(entry.id,);
    } catch (error) {
      failed[entry.id] = error instanceof Error
        ? error.message
        : String(error,);
    }
  }

  // Postinstall steps (the `cli-git-bin` atomic `git` link).
  runPostinstallLinks({
    paths,
    entries,
    env: e,
    extraDirs: [paths.toolchainBinDir],
  },);

  // Rebuild the exposure directory and the tool environment exports.
  rebuildExposure({
    paths,
    entries,
    env: e,
    extraDirs: [paths.toolchainBinDir],
  },);
  writeToolEnvFile({
    paths,
    entries,
  },);

  return {
    unchanged,
    installed,
    failed,
    missingBins: unresolvableBins({
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
 
 @returns Promise resolved after the sync finishes
 */
export async function main(): Promise<void> {
  /**
   Whether this run targets the machine-global scope.
   */
  const global = process.argv.includes('--global',);
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
  process.stdout.write(
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
  process.stdout.write(
    `toolchain sync: ${
      report.installed.length
    } installed, ${
      report.unchanged.length
    } unchanged, ${
      Object.keys(report.failed,)
        .length
    } failed\n`,
  );
  for (const [id, message,] of Object.entries(report.failed,))
    process.stderr.write(`toolchain sync: ${id} failed: ${message}\n`,);
  if (report.missingBins.length > 0)
    process.stderr.write(
      `toolchain sync: unresolved bins: ${
        report.missingBins.join(', ',)
      }\n`,
    );
  if (Object.keys(report.failed,)
    .length > 0)
    process.exitCode = 1;
}

//endregion CLI

if (isMainModule(import.meta.filename,))
  await main();
