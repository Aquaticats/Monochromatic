/**
 `status.ts`: prints observed versus locked tool versions and exits non-zero
 on drift.

 @example
 ```bash
 node package/dev-script/file-enforcer/src/toolchain/status.ts
 ```
 */

import { existsSync, readdirSync, } from 'node:fs';
import { join, } from 'node:path';
import { run, } from './exec.ts';
import { findToolBin, } from './layout.ts';
import {
  binCliPath,
  parseBinList,
  parseCargoBinstallList,
  toolchainPathEnv,
} from './providers.ts';
import {
  homeDirFromEnv,
  resolveToolchainPaths,
  toolchainRootFromEnv,
  type PathEnv,
  type ToolchainPaths,
  isMainModule,
} from './paths.ts';
import { toolInstallDir, } from './paths.ts';
import {
  ALL_TOOLS,
  toolById,
} from './plan.ts';
import type { ToolPlanEntry, } from './types.ts';

//region Version comparison

/**
 Normalizes a version string for comparison: drops one leading `v`.
 
 @param version - Raw version text
 
 @returns Normalized version text
 */
export function normalizeVersion(version: string,): string {
  // Release tags carry prefixes (`v1.9.0`, `bun-v1.3.14`); the lock holds
  // the bare version. Anything without a v-before-digit stays as written.
  return version.replace(
    /^(?:[A-Za-z][A-Za-z0-9._-]*-)?v(?=\d)/u,
    '',
  );
}

/**
 Whether an observed version satisfies a locked version:
 equal after normalization, or the observed value extends the lock with a
 platform suffix (`nightly-2026-10-21-x86_64-unknown-linux-gnu`).
 
 @param options - Observed and locked versions
 
 @returns `true` when the observed version matches the lock
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
  return o === l || o.startsWith(`${l}-`,) || l.startsWith(`${o}-`,);
}

//endregion Version comparison

//region Observation

/**
 Observed state of one tool.
 */
export type ToolObservation = {
  readonly id: string;
  readonly locked: string;
  /** Observed version text, or `null` when nothing is installed. */
  readonly observed: string | null;
  /** Whether the observed version matches the lock. */
  readonly inSync: boolean;
};

/**
 Probes the observed version of one plan entry on this machine.
 
 @param options - Plan entry, paths and environment
 
 @returns Observed version text, or `null` when the tool is absent
 */
export async function observeTool(
  {
    entry,
    paths,
    env,
  }: {
    readonly entry: ToolPlanEntry;
    readonly paths: ToolchainPaths;
    readonly env: PathEnv;
  },
): Promise<string | null> {
  /**
   Home directory for manager-native probes.
   */
  const home = homeDirFromEnv({ env, });
  switch (entry.route) {
    case 'bin-sourced': {
      /**
       `bin list` output.
       */
      const listed = await run({
        command: binCliPath({
          paths,
          env,
        },),
        args: ['list'],
        env: {
          ...process.env,
          HOME: home,
        },
      },);
      /**
       Name to version pairs from the `bin` database.
       */
      const versions = parseBinList(listed.stdout + listed.stderr);
      for (const name of entry.bins) {
        /**
         Version recorded for this binary name.
         */
        const version = versions.get(name,);
        if (version !== undefined)
          return version;
      }
      return null;
    }
    case 'cargo-binstall': {
      /**
       `cargo binstall --list` output.
       */
      const listed = await run({
        command: 'cargo',
        args: [
          'binstall',
          '--list',
        ],
        env: toolchainPathEnv({
          paths,
          env,
        },),
      },);
      return parseCargoBinstallList(listed.stdout,).get(entry.packageId ?? entry.id,) ?? null;
    }
    case 'mpm-pnpm': {
      /**
       `pnpm list --global --depth=0 --json` output.
       */
      /**
       Toolchain `pnpm` that owns the global package list.
       */
      const pnpmBin = findToolBin({
        entry: toolById('pnpm',),
        name: 'pnpm',
        paths,
        env,
      },);
      if (pnpmBin === null)
        return null;
      const listed = await run({
        command: pnpmBin,
        args: [
          'list',
          '--global',
          '--depth=0',
          '--json',
        ],
        env: toolchainPathEnv({
          paths,
          env,
        },),
      },);
      try {
        /**
         Parsed global package map; pnpm emits one root object per entry in
         an array, older versions emitted the object directly.
         */
        const parsed = JSON.parse(listed.stdout,) as {
          dependencies?: Record<string, { version?: string; }>;
        } | readonly {
          dependencies?: Record<string, { version?: string; }>;
        }[];
        /**
         Dependency map of the global root.
         */
        const roots: readonly { dependencies?: Record<string, { version?: string; }>; }[] = Array.isArray(parsed,)
          ? parsed
          : [parsed,];
        const dependencies: Record<string, { version?: string; }> = {};
        for (const root of roots)
          Object.assign(dependencies, root.dependencies ?? {},);
        return dependencies[entry.packageId ?? entry.id]?.version ?? null;
      } catch {
        return null;
      }
    }
    case 'mpm-uvx': {
      /**
       `uv tool list` output.
       */
      /**
       Toolchain `uv` that owns the tool list.
       */
      const uvBin = findToolBin({
        entry: toolById('uv',),
        name: 'uv',
        paths,
        env,
      },);
      if (uvBin === null)
        return null;
      const listed = await run({
        command: uvBin,
        args: ['tool', 'list'],
        env: toolchainPathEnv({
          paths,
          env,
        },),
      },);
      for (const line of listed.stdout.split('\n',)) {
        /**
         One `name v<version>` row.
         */
        const match = line.match(/^(\S+)\s+v?(\S+)/u);
        if (match !== null && match[1] === (entry.packageId ?? entry.id))
          return match[2] ?? null;
      }
      return null;
    }
    case 'rustup-toolchain': {
      /**
       `rustup toolchain list` output.
       */
      const listed = await run({
        command: 'rustup',
        args: ['toolchain', 'list'],
      },);
      for (const line of listed.stdout.split('\n',)) {
        /**
         Toolchain id from one listing line.
         */
        const id = line.split(/\s/u)[0] ?? '';
        if (id.startsWith(entry.version,))
          return id;
      }
      return null;
    }
    case 'android-sdk': {
      /**
       cmdline-tools directory for the locked version.
       */
      const cmdlineTools = join(
        toolInstallDir({
          toolchainRoot: paths.toolchainRoot,
          id: entry.id,
          version: entry.version,
        }),
        'cmdline-tools',
        entry.version,
      );
      return existsSync(cmdlineTools,)
        ? entry.version
        : null;
    }
    case 'archive':
    case 'rustup-components': {
      /**
       Version directories present for this tool.
       */
      const toolRoot = join(
        paths.toolchainRoot,
        entry.id.replaceAll(
          /[^A-Za-z0-9._-]+/g,
          '-',
        ),
      );
      if (!existsSync(toolRoot,))
        return null;
      /**
       Installed version directory names.
       */
      const dirs = readdirSync(toolRoot, { withFileTypes: true, },)
        .filter(
          function isVersionDir(entryDir,): boolean {
            return entryDir.isDirectory()
              && !entryDir.name.startsWith('.',);
          },
        )
        .map(
          function dirName(entryDir,): string {
            return entryDir.name;
          },
        );
      return dirs.includes(entry.version,)
        ? entry.version
        : (dirs[0] ?? null);
    }
  }
}

/**
 Observes every plan entry against its locked version.
 
 @param options - Plan entries, paths and environment
 
 @returns One observation per plan entry, in plan order
 */
export async function observeTools(
  {
    entries = ALL_TOOLS,
    paths,
    env,
  }: {
    readonly entries?: readonly ToolPlanEntry[];
    readonly paths: ToolchainPaths;
    readonly env: PathEnv;
  },
): Promise<readonly ToolObservation[]> {
  /**
   Observations built in plan order.
   */
  const observations: ToolObservation[] = [];
  for (const entry of entries) {
    /**
     Observed version for this entry.
     */
    const observed = await observeTool({
      entry,
      paths,
      env,
    },);
    observations.push({
      id: entry.id,
      locked: entry.version,
      observed,
      inSync: observed !== null
        && versionMatches({
          observed,
          locked: entry.version,
        }),
    },);
  }
  return observations;
}

//endregion Observation

//region CLI

/**
 Entry point: prints the drift report on stdout.
 
 @returns Promise resolved after the report is written
 */
export async function main(): Promise<void> {
  /**
   Observed state of every plan entry.
   */
  const observations = await observeTools({
    paths: resolveToolchainPaths(),
    env: process.env,
  },);
  /**
   Number of drifted entries.
   */
  let drift = 0;
  for (const observation of observations) {
    if (observation.inSync) {
      process.stdout.write(`ok      ${observation.id}  ${observation.locked}\n`,);
      continue;
    }
    drift += 1;
    process.stdout.write(
      `drift   ${observation.id}  locked=${observation.locked}  observed=${
        observation.observed ?? '(absent)'
      }\n`,
    );
  }
  process.stdout.write(
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

if (isMainModule(import.meta.filename,))
  await main();
