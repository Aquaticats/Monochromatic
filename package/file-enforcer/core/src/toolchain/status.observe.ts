/**
 Per-route version probes for `status.ts`: one observer per install route reads what that route's manager (or
 install directory) reports for a plan entry.

 Every observer resolves to the observed version text, or to {@link NOT_INSTALLED} when the route shows nothing for
 the entry.
 */

import { readdir, } from 'node:fs/promises';
import { join, } from 'node:path';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import { l, } from '../logger.ts';
import { run, } from './exec.ts';
import { pathExists, } from './fs-probe.ts';
import { findToolBin, } from './layout.ts';
import {
  globalPnpmVersions,
  parseUvToolList,
  splitAtFirstWhitespace,
} from './status.parse.ts';
import {
  binCliPath,
  parseBinList,
  parseCargoBinstallList,
  planTool,
  toolchainPathEnv,
} from './providers.ts';
import {
  homeDirFromEnv,
  toolInstallDir,
  type PathEnv,
  type ToolchainPaths,
} from './paths.ts';
import type {
  ToolPlanEntry,
  ToolRoute,
} from './types.ts';

//region Shapes

/**
 Marks that a route shows nothing installed for a plan entry.
 */
export const NOT_INSTALLED: unique symbol = Symbol('route observer found no installed version',);

/**
 What every route observer receives.
 */
export type ObserveParams = {
  readonly entry: ToolPlanEntry;
  readonly paths: ToolchainPaths;
  readonly env: PathEnv;
  /**
   Plan that sibling lookups (the staged `pnpm`, `uv`, `bin`) resolve against.
   */
  readonly tools: readonly ToolPlanEntry[];
};

/**
 Probes one route for the installed version of a plan entry.
 */
export type RouteObserver = (
  params: ObserveParams,
) => Promise<string | typeof NOT_INSTALLED>;

//endregion Shapes

//region Observers

/**
 Resolves the staged `bin` CLI; an unprobeable database reads as an absent tool, not a crash.

 @param paths - Resolved toolchain paths of the scope being observed

 @param env - Environment the probe resolves manager homes from

 @param tools - Plan that the staged `bin` entry resolves against

 @returns Path of the `bin` CLI, or {@link NOT_INSTALLED} on a fresh machine before the `bin` stage runs
 */
function resolveBinCli(
  {
    paths,
    env,
    tools,
  }: Omit<ObserveParams, 'entry'>,
): string | typeof NOT_INSTALLED {
  try {
    return binCliPath({
      paths,
      env,
      tools,
    },);
  } catch (error) {
    /**
     Logger tagged with this probe.
     */
    const log = tagged({
      tag: resolveBinCli.name,
      l,
    },);
    log.debug(`bin CLI not available, reading the tool as absent: ${caughtValueText(error,)}`,);
    return NOT_INSTALLED;
  }
}

/**
 Observes a `bin`-sourced tool through the scope's own `bin list` database.

 @param entry - Plan entry being observed

 @param paths - Resolved toolchain paths of the scope being observed

 @param env - Environment the probe resolves manager homes from

 @param tools - Plan that the staged `bin` entry resolves against

 @returns Version recorded for the first declared binary name that `bin list` reports
 */
async function observeBinSourced(
  {
    entry,
    paths,
    env,
    tools,
  }: ObserveParams,
): Promise<string | typeof NOT_INSTALLED> {
  /**
   Bootstrap CLI, absent on a fresh machine before the `bin` stage runs.
   */
  const cli = resolveBinCli({
    paths,
    env,
    tools,
  },);
  if (cli === NOT_INSTALLED)
    return NOT_INSTALLED;
  /**
   `bin list` output from the scope's own install database.
   */
  const listed = await run({
    command: cli,
    args: ['list'],
    env: {
      ...process.env,
      HOME: homeDirFromEnv({ env, }),
      BIN_CONFIG: paths.binConfigFile,
    },
  },);
  /**
   Name to version pairs from the `bin` database.
   */
  const versions = parseBinList(`${listed.stdout}${listed.stderr}`,);
  for (const name of entry.bins) {
    /**
     Version recorded for this binary name.
     */
    const version = versions.get(name,);
    if (version !== undefined)
      return version;
  }
  return NOT_INSTALLED;
}

/**
 Observes a `cargo binstall` tool through `cargo binstall --list`.

 @param entry - Plan entry being observed

 @param paths - Resolved toolchain paths of the scope being observed

 @param env - Environment the probe resolves manager homes from

 @returns Version `cargo binstall` lists for the crate
 */
async function observeCargoBinstall(
  {
    entry,
    paths,
    env,
  }: ObserveParams,
): Promise<string | typeof NOT_INSTALLED> {
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
  return parseCargoBinstallList(listed.stdout,)
    .get(entry.packageId ?? entry.id,)
    ?? NOT_INSTALLED;
}

/**
 Observes a global `pnpm` package through the toolchain `pnpm`.

 @param entry - Plan entry being observed

 @param paths - Resolved toolchain paths of the scope being observed

 @param env - Environment the probe resolves manager homes from

 @param tools - Plan that the staged `pnpm` entry resolves against

 @returns Version the global package list reports for the package
 */
async function observeMpmPnpm(
  {
    entry,
    paths,
    env,
    tools,
  }: ObserveParams,
): Promise<string | typeof NOT_INSTALLED> {
  /**
   Toolchain `pnpm` that owns the global package list.
   */
  const pnpmBin = findToolBin({
    entry: planTool({
      tools,
      id: 'pnpm',
    },),
    name: 'pnpm',
    paths,
    env,
  },);
  if (pnpmBin === null)
    return NOT_INSTALLED;
  /**
   `pnpm list --global --depth=0 --json` output.
   */
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
    return globalPnpmVersions(JSON.parse(listed.stdout,) as unknown,)
      .get(entry.packageId ?? entry.id,)
      ?? NOT_INSTALLED;
  } catch (error) {
    /**
     Logger tagged with this probe.
     */
    const log = tagged({
      tag: observeMpmPnpm.name,
      l,
    },);
    log.warn(`pnpm global list was not JSON, reading the tool as absent: ${caughtValueText(error,)}`,);
    return NOT_INSTALLED;
  }
}

/**
 Observes a `uv` tool through the toolchain `uv`.

 @param entry - Plan entry being observed

 @param paths - Resolved toolchain paths of the scope being observed

 @param env - Environment the probe resolves manager homes from

 @param tools - Plan that the staged `uv` entry resolves against

 @returns Version `uv tool list` reports for the package
 */
async function observeMpmUvx(
  {
    entry,
    paths,
    env,
    tools,
  }: ObserveParams,
): Promise<string | typeof NOT_INSTALLED> {
  /**
   Toolchain `uv` that owns the tool list.
   */
  const uvBin = findToolBin({
    entry: planTool({
      tools,
      id: 'uv',
    },),
    name: 'uv',
    paths,
    env,
  },);
  if (uvBin === null)
    return NOT_INSTALLED;
  /**
   `uv tool list` output.
   */
  const listed = await run({
    command: uvBin,
    args: [
      'tool',
      'list'
    ],
    env: toolchainPathEnv({
      paths,
      env,
    },),
  },);
  /**
   Plan package spelling with hyphens as underscores, the key `parseUvToolList` uses.
   */
  const packageKey = (entry.packageId ?? entry.id).replaceAll(
    '-',
    '_',
  );
  return parseUvToolList(listed.stdout,)
    .get(packageKey,)
    ?? NOT_INSTALLED;
}

/**
 Observes a rustup toolchain through `rustup toolchain list`.

 @param entry - Plan entry being observed

 @returns First listed toolchain id that starts with the locked version
 */
async function observeRustupToolchain(
  { entry, }: ObserveParams,
): Promise<string | typeof NOT_INSTALLED> {
  /**
   `rustup toolchain list` output.
   */
  const listed = await run({
    command: 'rustup',
    args: [
      'toolchain',
      'list'
    ],
  },);
  for (const line of listed.stdout
    .split('\n',)) {
    /**
     Toolchain id from one listing line.
     */
    const [id,] = splitAtFirstWhitespace(line,);
    if (id.startsWith(entry.version,))
      return id;
  }
  return NOT_INSTALLED;
}

/**
 Observes the Android SDK through its `cmdline-tools` directory for the locked version.

 @param entry - Plan entry being observed

 @param paths - Resolved toolchain paths of the scope being observed

 @returns The locked version when its `cmdline-tools` directory exists
 */
async function observeAndroidSdk(
  {
    entry,
    paths,
  }: ObserveParams,
): Promise<string | typeof NOT_INSTALLED> {
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
  return (await pathExists(cmdlineTools,))
    ? entry.version
    : NOT_INSTALLED;
}

/**
 Observes a directory-installed tool through its version directories.

 The scoped install routes (`cargo-git`, `go-install`) observe the same way: the pinned ref or module version keys
 the install directory, so presence at that key is the locked version.

 @param entry - Plan entry being observed

 @param paths - Resolved toolchain paths of the scope being observed

 @returns The locked version when its directory exists, else the first installed version directory
 */
async function observeVersionDirectory(
  {
    entry,
    paths,
  }: ObserveParams,
): Promise<string | typeof NOT_INSTALLED> {
  /**
   Directory that holds one subdirectory per installed version of this tool; an empty version leaves the install
   directory at the tool's own root.
   */
  const toolRoot = toolInstallDir({
    toolchainRoot: paths.toolchainRoot,
    id: entry.id,
    version: '',
  });
  if (!(await pathExists(toolRoot,)))
    return NOT_INSTALLED;
  /**
   Directory entries of the tool root.
   */
  const children = await readdir(
    toolRoot,
    { withFileTypes: true, },
  );
  /**
   Installed version directory names.
   */
  const dirs = children
    .filter(
    function isVersionDir(child,): boolean {
      return child.isDirectory()
        && (!child.name
          .startsWith('.',));
    },
    )
    .map(
    function dirName(child,): string {
      return child.name;
    },
    );
  return dirs.includes(entry.version,)
    ? entry.version
    : (dirs[0] ?? NOT_INSTALLED);
}

/**
 Observer for each install route.
 */
export const ROUTE_OBSERVERS: Readonly<Record<ToolRoute, RouteObserver>> = {
  'bin-sourced': observeBinSourced,
  'cargo-binstall': observeCargoBinstall,
  'mpm-pnpm': observeMpmPnpm,
  'mpm-uvx': observeMpmUvx,
  'rustup-toolchain': observeRustupToolchain,
  'android-sdk': observeAndroidSdk,
  'archive': observeVersionDirectory,
  'cargo-git': observeVersionDirectory,
  'go-install': observeVersionDirectory,
  'rustup-components': observeVersionDirectory,
};

//endregion Observers
