/**
 Install layout: where each route keeps its binaries, and how the exposure
 directory resolves a declared bin name to its link target.

 Used by `sync.ts` (to create links) and `status.ts` (to probe installs).
 Node builtins only.
 */

import { constants, } from 'node:fs';
import {
  access,
  readdir,
  stat,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { isErrnoException, } from './fs-probe.ts';
import type { ToolPlanEntry, } from './types.ts';
import {
  homeDirFromEnv,
  toolInstallDir,
  type PathEnv,
  type ToolchainPaths,
} from './paths.ts';

//region Platform id

/**
 Maps the current host onto the platform ids `toolchain.lock.json` uses
 (`linux-x64`, `linux-arm64-musl`, ...).
 
 @returns Platform id for the running host

 @example
 ```ts
 hostPlatformId(); // 'linux-x64'
 ```
 */
export function hostPlatformId(): string {
  /**
   Normalized architecture half of the platform id.
   */
  const arch = process.arch === 'arm64'
    ? 'arm64'
    : process.arch === 'x64'
      ? 'x64'
      : process.arch;
  if (process.platform === 'linux') {
    /**
     glibc runtime marker from the diagnostic report; absent on musl builds.
     */
    const report = process.report
      ?.getReport() as
      | { readonly header?: { readonly glibcVersionRuntime?: string; }; }
        | string;
    /**
     Whether the host libc is musl rather than glibc.
     */
    const musl = ((typeof report) === 'object')
      && (report.header
        ?.glibcVersionRuntime
        === undefined);
    return `linux-${arch}${musl
      ? '-musl'
      : ''}`;
  }
  if (process.platform === 'darwin')
    return `macos-${arch}`;
  if (process.platform === 'win32')
    return `windows-${arch}`;
  return `${process.platform}-${arch}`;
}

//endregion Platform id

//region Filesystem helpers

/**
 Whether a path exists and is executable by the current user.

 Only operating-system file errors (a missing path, a permission refusal) count as "not executable";
 any other failure propagates.

 @param path - Absolute path to test

 @returns `true` when the path resolves to an executable

 @example
 ```ts
 await isExecutable('/usr/bin/env',); // true
 ```
 */
export async function isExecutable(path: string,): Promise<boolean> {
  try {
    await access(
      path,
      constants.X_OK,
    );
    return (await stat(path,))
      .isFile();
  } catch (error) {
    if (!isErrnoException(error,))
      throw error;
    return false;
  }
}

/**
 Whether a path is a directory; an unreadable path is not.

 @param path - Absolute path to test

 @returns `true` when the path resolves to a directory
 */
async function isDirectory(path: string,): Promise<boolean> {
  try {
    return (await stat(path,))
      .isDirectory();
  } catch (error) {
    if (!isErrnoException(error,))
      throw error;
    // Unreadable entries are simply not candidates.
    return false;
  }
}

/**
 Names inside a directory, sorted for deterministic order; an unreadable directory has none.

 @param dir - Absolute directory to list

 @returns Entry names in lexicographic order
 */
async function sortedChildren(dir: string,): Promise<readonly string[]> {
  try {
    return (await readdir(dir,))
      .toSorted();
  } catch (error) {
    if (!isErrnoException(error,))
      throw error;
    return [];
  }
}

/**
 Directory levels below the search root that `findBinUnder` descends through.
 */
const MAX_BIN_SEARCH_DEPTH = 3;

/**
 Marks that a binary search found no matching binary.
 */
export const BIN_NOT_FOUND: unique symbol = Symbol('searched directories held no executable of the requested name',);

/**
 Searches one directory level for a binary, then descends into the level below.

 The level is the unit of concurrency: every candidate of one depth is probed at once, and the first hit in array
 order wins, which keeps the shallowest-first, lexicographic order of the sequential walk. The recursion depth is
 bounded by `MAX_BIN_SEARCH_DEPTH`.

 @param dirs - Directories of this depth, in visiting order

 @param name - Binary file name to look for

 @param depth - Depth of `dirs` below the search root

 @returns Absolute path of the match, or {@link BIN_NOT_FOUND} when none resolves
 */
async function searchBinLevel(
  {
    dirs,
    name,
    depth,
  }: {
    readonly dirs: readonly string[];
    readonly name: string;
    readonly depth: number;
  },
): Promise<string | typeof BIN_NOT_FOUND> {
  if (dirs.length === 0)
    return BIN_NOT_FOUND;
  /**
   Direct candidate in each directory of this level.
   */
  const directs = dirs.map(
    function directCandidate(dir,): string {
      return join(
        dir,
        name,
      );
    },
  );
  /**
   Whether each direct candidate is an executable file, in the same order as `directs`.
   */
  const executable = await Promise.all(directs.map(
    function candidateIsExecutable(direct,): Promise<boolean> {
      return isExecutable(direct,);
    },
  ),);
  /**
   First executable candidate in visiting order.
   */
  const [hit,] = directs.filter(
    function isHit(
      _direct,
      index,
    ): boolean {
      return executable[index] === true;
    },
  );
  if (hit !== undefined)
    return hit;
  if (depth >= MAX_BIN_SEARCH_DEPTH)
    return BIN_NOT_FOUND;
  /**
   Absolute child paths of every directory of this level, in visiting order.
   */
  const children = (await Promise.all(dirs.map(
    async function childPaths(dir,): Promise<readonly string[]> {
      return (await sortedChildren(dir,)).map(
        function childPath(child,): string {
          return join(
            dir,
            child,
          );
        },
      );
    },
  ),))
    .flat();
  /**
   Whether each child is a directory, in the same order as `children`.
   */
  const childIsDirectory = await Promise.all(children.map(
    function childIsDir(child,): Promise<boolean> {
      return isDirectory(child,);
    },
  ),);
  return searchBinLevel({
    dirs: children.filter(
      function isDirectoryChild(
        _child,
        index,
      ): boolean {
        return childIsDirectory[index] === true;
      },
    ),
    name,
    depth: depth + 1,
  },);
}

/**
 Finds a binary file by name under a directory, searching shallowest first
 and lexicographic order within one depth, bounded to three directory levels.

 @param root - Directory the search starts in

 @param name - Binary file name to look for

 @returns Absolute path of the match, or {@link BIN_NOT_FOUND} when none resolves

 @example
 ```ts
 await findBinUnder({ root: '/opt/tools', name: 'rg', },); // '/opt/tools/ripgrep-15.2.0/rg'
 ```
 */
export async function findBinUnder(
  {
    root,
    name,
  }: {
    readonly root: string;
    readonly name: string;
  },
): Promise<string | typeof BIN_NOT_FOUND> {
  return await searchBinLevel({
    dirs: [root],
    name,
    depth: 0,
  },);
}

//endregion Filesystem helpers

//region Link targets

/**
 Extra directories a provider knows its route installs into
 (for example npm's probed global prefix).
 */
export type BinSearchContext = {
  readonly paths: ToolchainPaths;
  readonly env?: PathEnv;
  readonly extraDirs?: readonly string[];
};

/**
 Directories to search for one tool's binaries, in resolution order.

 @param entry - Plan entry whose route decides where binaries land

 @param paths - Resolved toolchain paths of the scope

 @param env - Environment overriding manager homes; the process environment when absent

 @param extraDirs - Extra directories a staged tool contributes

 @returns Absolute directories, in order

 @example
 ```ts
 binSearchDirs({ entry: toolById('node',), paths, },); // ['/.../node/26.12.0']
 ```
 */
export function binSearchDirs(
  {
    entry,
    paths,
    env,
    extraDirs,
  }: BinSearchContext & { readonly entry: ToolPlanEntry; },
): readonly string[] {
  /**
   Effective environment (defaults to the process environment).
   */
  const e = env ?? process.env;
  /**
   Home directory for manager-native defaults.
   */
  const home = homeDirFromEnv({ env: e, });
  /**
   Machine install directory of this tool.
   */
  const installDir = toolInstallDir({
    toolchainRoot: paths.toolchainRoot,
    id: entry.id,
    version: entry.version,
  });
  if (entry.route === 'cargo-binstall')
    return [
      join(
        (e.CARGO_HOME !== undefined) && (e.CARGO_HOME !== '')
          ? e.CARGO_HOME
          : join(
            home,
            '.cargo',
          ),
        'bin',
      ),
    ];
  if (entry.route === 'bin-sourced')
    return [
      // `bin install` drops files in its configured download directory;
      // `sync` pre-seeds that config with the toolchain bin dir.
      paths.toolchainBinDir,
      join(
        home,
        '.local',
        'bin',
      ),
      ...(extraDirs ?? []),
    ];
  if (entry.route === 'mpm-pnpm')
    return [
      ...(extraDirs ?? []),
      (e.PNPM_HOME !== undefined) && (e.PNPM_HOME !== '')
        ? e.PNPM_HOME
        : join(
          home,
          '.local',
          'share',
          'pnpm',
        ),
      join(
        paths.toolchainRoot,
        'node',
      ),
    ];
  if (entry.route === 'mpm-uvx')
    return [
      (e.UV_TOOL_BIN_DIR !== undefined) && (e.UV_TOOL_BIN_DIR !== '')
        ? e.UV_TOOL_BIN_DIR
        : join(
          home,
          '.local',
          'bin',
        ),
      ...(extraDirs ?? []),
    ];
  // Scoped installs: `cargo install --root <installDir>` and
  // `GOBIN=<installDir>/bin` both land binaries under the tool's own
  // install directory (`binSearchDirs` callers walk its `bin/` level).
  if ((entry.route === 'cargo-git') || (entry.route === 'go-install'))
    return [
      installDir,
    ];
  if (entry.route === 'rustup-toolchain')
    return [
      join(
        (e.CARGO_HOME !== undefined) && (e.CARGO_HOME !== '')
          ? e.CARGO_HOME
          : join(
            home,
            '.cargo',
          ),
        'bin',
      ),
      join(
        (e.RUSTUP_HOME !== undefined) && (e.RUSTUP_HOME !== '')
          ? e.RUSTUP_HOME
          : join(
            home,
            '.rustup',
          ),
        'toolchains',
      ),
    ];
  if (entry.route === 'android-sdk')
    return [
      join(
        installDir,
        'cmdline-tools',
        entry.version,
        'bin',
      ),
      join(
        installDir,
        'platform-tools',
      ),
      join(
        installDir,
        'build-tools',
        entry.options
          .androidPackages
          ?.find(
          function isBuildToolsComponent(component,): boolean {
            return component.startsWith('build-tools;',);
          },
          )
          ?.slice('build-tools;'.length,)
          ?? entry.version,
      ),
    ];
  if (entry.route === 'archive')
    return [installDir];
  // `rustup-components` installs into the rustup toolchain, never a bin directory of its own.
  return [];
}

/**
 Resolves one declared bin name to the file the exposure directory should
 link, searching every directory the tool's route owns.

 @param entry - Plan entry that declares the bin

 @param name - Declared bin name to resolve

 @param paths - Resolved toolchain paths of the scope

 @param env - Environment overriding manager homes; the process environment when absent

 @param extraDirs - Extra directories a staged tool contributes

 @returns Absolute link target, or {@link BIN_NOT_FOUND} when the bin is not installed

 @example
 ```ts
 await findToolBin({ entry: toolById('node',), name: 'node', paths, },); // '/.../node/26.12.0/bin/node'
 ```
 */
export async function findToolBin(
  {
    entry,
    name,
    paths,
    env,
    extraDirs,
  }: BinSearchContext & {
    readonly entry: ToolPlanEntry;
    readonly name: string;
  },
): Promise<string | typeof BIN_NOT_FOUND> {
  for (const dir of binSearchDirs({
    entry,
    paths,
    env: env ?? process.env,
    extraDirs: extraDirs ?? [],
  },)) {
    /**
     Direct hit in one search directory.
     */
    const direct = join(
      dir,
      name,
    );
    /* oxlint-disable no-await-in-loop -- search directories are ordered by precedence, and a later directory's bounded walk is only worth its filesystem cost when every earlier one missed */
    if (await isExecutable(direct,))
      return direct;
    /**
     Bounded nested search for layouts that keep binaries in child
     directories (`ripgrep-15.2.0-x86_64-unknown-linux-musl/rg`,
     `node/26.12.0/bin/npm`, `toolchains/<triple>/bin/rustc`).
     */
    const nested = await findBinUnder({
      root: dir,
      name,
    },);
    /* oxlint-enable no-await-in-loop */
    if (nested !== BIN_NOT_FOUND)
      return nested;
  }
  return BIN_NOT_FOUND;
}

//endregion Link targets
