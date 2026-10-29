/**
 Install layout: where each route keeps its binaries, and how the exposure
 directory resolves a declared bin name to its link target.

 Used by `sync.ts` (to create links) and `status.ts` (to probe installs).
 Node builtins only.
 */

import {
  accessSync,
  constants,
  readdirSync,
  statSync,
} from 'node:fs';
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
 isExecutable('/usr/bin/env',); // true
 ```
 */
export function isExecutable(path: string,): boolean {
  try {
    accessSync(
      path,
      constants.X_OK,
    );
    return statSync(path,)
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
function isDirectory(path: string,): boolean {
  try {
    return statSync(path,)
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
function sortedChildren(dir: string,): readonly string[] {
  try {
    return readdirSync(dir,)
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
 Marks that `findBinUnder` found no matching binary.
 */
export const BIN_NOT_FOUND: unique symbol = Symbol('findBinUnder found no matching binary',);

/**
 Finds a binary file by name under a directory, searching shallowest first
 and lexicographic order within one depth, bounded to three directory levels.

 @param root - Directory the search starts in

 @param name - Binary file name to look for

 @returns Absolute path of the match, or {@link BIN_NOT_FOUND} when none resolves

 @example
 ```ts
 findBinUnder({ root: '/opt/tools', name: 'rg', },); // '/opt/tools/ripgrep-15.2.0/rg'
 ```
 */
export function findBinUnder(
  {
    root,
    name,
  }: {
    readonly root: string;
    readonly name: string;
  },
): string | typeof BIN_NOT_FOUND {
  /**
   Breadth-first queue of directories with their depth. The `for...of` walk
   visits entries pushed during the walk (the array iterator re-reads
   `length` each step), so each dequeue is constant time where `shift()`
   would copy the array every step.
   */
  const pending: {
    dir: string;
    depth: number
  }[] = [{
    dir: root,
    depth: 0
  }];
  for (const current of pending) {
    /**
     Direct candidate in this directory.
     */
    const direct = join(
      current.dir,
      name,
    );
    if (isExecutable(direct,))
      return direct;
    if (current.depth >= MAX_BIN_SEARCH_DEPTH)
      continue;
    for (const child of sortedChildren(current.dir,)) {
      /**
       Absolute child path.
       */
      const childPath = join(
        current.dir,
        child,
      );
      if (isDirectory(childPath,))
        pending.push(
          {
            dir: childPath,
            depth: current.depth + 1,
          },
        );
    }
  }
  return BIN_NOT_FOUND;
}

//endregion Filesystem helpers

//region Link targets

/**
 Extra directories a provider knows its route installs into
 (for example npm's probed global prefix).
 */
export type BinSearchContext = {
  readonly paths: ToolchainPaths;
  readonly env?: PathEnv | undefined;
  readonly extraDirs?: readonly string[] | undefined;
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

 @returns Absolute link target, or `null` when the bin is not installed

 @example
 ```ts
 findToolBin({ entry: toolById('node',), name: 'node', paths, },); // '/.../node/26.12.0/bin/node'
 ```
 */
export function findToolBin(
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
): string | null {
  for (const dir of binSearchDirs({
    entry,
    paths,
    env,
    extraDirs,
  },)) {
    /**
     Direct hit in one search directory.
     */
    const direct = join(
      dir,
      name,
    );
    if (isExecutable(direct,))
      return direct;
    /**
     Bounded nested search for layouts that keep binaries in child
     directories (`ripgrep-15.2.0-x86_64-unknown-linux-musl/rg`,
     `node/26.12.0/bin/npm`, `toolchains/<triple>/bin/rustc`).
     */
    const nested = findBinUnder({
      root: dir,
      name,
    },);
    if (nested !== BIN_NOT_FOUND)
      return nested;
  }
  return null;
}

//endregion Link targets
