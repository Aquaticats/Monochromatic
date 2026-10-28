/**
 Install layout: where each route keeps its binaries, and how the exposure
 directory resolves a declared bin name to its link target.

 Used by `sync.ts` (to create links) and `status.ts` (to probe installs).
 Node builtins only.
 */

import { accessSync, constants, readdirSync, statSync, } from 'node:fs';
import { join, } from 'node:path';
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
    const report = process.report?.getReport() as
      | { readonly header?: { readonly glibcVersionRuntime?: string; }; }
      | string;
    /**
     Whether the host libc is musl rather than glibc.
     */
    const musl = typeof report === 'object'
      && report.header?.glibcVersionRuntime === undefined;
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
 
 @param path - Absolute path to test
 
 @returns `true` when the path resolves to an executable
 */
export function isExecutable(path: string,): boolean {
  try {
    accessSync(
      path,
      constants.X_OK,
    );
    return statSync(path,)
      .isFile();
  } catch {
    return false;
  }
}

/**
 Finds a binary file by name under a directory, searching shallowest first
 and lexicographic order within one depth, bounded to three directory levels.
 
 @param options - Root directory and binary name
 
 @returns Absolute path of the match, or `null` when none resolves
 */
export function findBinUnder(
  {
    root,
    name,
  }: {
    readonly root: string;
    readonly name: string;
  },
): string | null {
  /**
   Pending directories, as `[dir, depth]` pairs.
   */
  const pending: { dir: string; depth: number; }[] = [{ dir: root, depth: 0 }];
  while (pending.length > 0) {
    /**
     Next directory to inspect.
     */
    const current = pending.shift();
    if (current === undefined)
      break;
    /**
     Direct candidate in this directory.
     */
    const direct = join(
      current.dir,
      name,
    );
    if (isExecutable(direct,))
      return direct;
    if (current.depth >= 3)
      continue;
    /**
     Child directories of the current one, sorted for deterministic order.
     */
    let children: string[] = [];
    try {
      children = readdirSync(current.dir,)
        .sort();
    } catch {
      continue;
    }
    for (const child of children) {
      /**
       Absolute child path.
       */
      const childPath = join(
        current.dir,
        child,
      );
      try {
        if (statSync(childPath,)
          .isDirectory())
          pending.push(
            {
              dir: childPath,
              depth: current.depth + 1,
            },
          );
      } catch {
        // Unreadable entries are simply not candidates.
      }
    }
  }
  return null;
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
 
 @param options - Plan entry plus search context
 
 @returns Absolute directories, in order
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
  switch (entry.route) {
    case 'cargo-binstall':
      return [
        join(
          e.CARGO_HOME !== undefined && e.CARGO_HOME !== ''
            ? e.CARGO_HOME
            : join(home, '.cargo',),
          'bin',
        ),
      ];
    case 'bin-sourced':
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
    case 'mpm-pnpm':
      return [
        ...(extraDirs ?? []),
        e.PNPM_HOME !== undefined && e.PNPM_HOME !== ''
          ? e.PNPM_HOME
          : join(home, '.local', 'share', 'pnpm',),
        join(
          paths.toolchainRoot,
          'node',
        ),
      ];
    case 'mpm-uvx':
      return [
        e.UV_TOOL_BIN_DIR !== undefined && e.UV_TOOL_BIN_DIR !== ''
          ? e.UV_TOOL_BIN_DIR
          : join(home, '.local', 'bin',),
        ...(extraDirs ?? []),
      ];
    case 'cargo-git':
    case 'go-install':
      // Scoped installs: `cargo install --root <installDir>` and
      // `GOBIN=<installDir>/bin` both land binaries under the tool's own
      // install directory (`binSearchDirs` callers walk its `bin/` level).
      return [
        installDir,
      ];
    case 'rustup-toolchain':
      return [
        join(
          e.CARGO_HOME !== undefined && e.CARGO_HOME !== ''
            ? e.CARGO_HOME
            : join(home, '.cargo',),
          'bin',
        ),
        join(
          e.RUSTUP_HOME !== undefined && e.RUSTUP_HOME !== ''
            ? e.RUSTUP_HOME
            : join(home, '.rustup',),
          'toolchains',
        ),
      ];
    case 'android-sdk':
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
          entry.options.androidPackages?.find(
            (component,) => component.startsWith('build-tools;',),
          )?.slice('build-tools;'.length,) ?? entry.version,
        ),
      ];
    case 'archive':
      return [installDir];
    case 'rustup-components':
      return [];
  }
}

/**
 Resolves one declared bin name to the file the exposure directory should
 link, searching every directory the tool's route owns.
 
 @param options - Plan entry, bin name, and search context
 
 @returns Absolute link target, or `null` when the bin is not installed
 */
export function findToolBin(
  {
    entry,
    name,
    ...context
  }: BinSearchContext & {
    readonly entry: ToolPlanEntry;
    readonly name: string;
  },
): string | null {
  for (const dir of binSearchDirs({
    entry,
    ...context,
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
    if (nested !== null)
      return nested;
  }
  return null;
}

//endregion Link targets
