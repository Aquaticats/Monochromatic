/**
 The read-only mount that lets a container resolve this checkout's installed packages.

 This repository installs with `storeDir` and `virtualStoreType: global` (`pnpm-workspace.yaml`), so every entry in a
 `node_modules` folder is a symbolic link into one shared store under the home directory, and the package files
 themselves live only in that store. A container that mounts this checkout and nothing else therefore finds links
 whose targets do not exist inside it, and every import of an installed package fails with `ERR_MODULE_NOT_FOUND`
 (measured 2026-11-08T19:18:31-05:00 on the `env-local-json` matrix cell, and again on 2026-11-09; see
 `doc/handover/container-limits-everywhere.md`, section Measured runs on this host).

 Mounting the store read-only at the absolute path the links name is what makes them resolve: a link is interpreted
 against its own directory, so any other mount point leaves it dangling, and the store needs no write access because
 pnpm, not the container, installs into it.

 A container given this mount also needs `--security-opt label=disable`, for two reasons measured on this host: a
 `:Z` relabel of the store would rewrite the labels of a directory every worktree shares, and an unlabelled read-only
 mount is denied under an enforcing SELinux (`ERR_INVALID_PACKAGE_CONFIG ... permission denied`).
 */

import { homedir, } from 'node:os';
import { join, } from 'node:path';

//region Names

/**
 The `storeDir` setting of `pnpm-workspace.yaml`, relative to the home directory, which is where pnpm itself writes the
 links every `node_modules` entry is.

 The workspace file cannot be read here: this module builds `podman run` arguments synchronously, for tasks that start
 containers and for the matrix runner, and this repository bans the synchronous `node:fs` API outside tests. The
 setting is therefore mirrored as this constant, and `pnpm-store-mount.unit.test.ts` reads the workspace file and fails
 when the two disagree.
 */
export const STORE_DIRECTORY_FROM_HOME: string = join(
  '.local',
  'share',
  'pnpm',
  'store',
);

//endregion Names

//region Mount arguments

/**
 The shared store directory, as the container that mounts this checkout must spell it.

 @param home - current user's home directory; taken from the runtime unless a caller supplies one

 @returns absolute store directory the `node_modules` links of this checkout point into

 @example
 ```ts
 const directory = pnpmStoreDirectory({ home: '/home/user' });
 // '/home/user/.local/share/pnpm/store'
 ```
 */
export function pnpmStoreDirectory({
  home = homedir(),
}: {
  readonly home?: string;
} = {},): string {
  return join(
    home,
    STORE_DIRECTORY_FROM_HOME,
  );
}

/**
 The `podman run` options that put the shared store at the path this checkout's links name, read-only.

 @param home - current user's home directory; taken from the runtime unless a caller supplies one

 @returns `['--volume', '<store>:<store>:ro']`, one mount because source and destination must spell the same path

 @example
 ```ts
 const options = pnpmStoreMountArguments({ home: '/home/user' });
 // ['--volume', '/home/user/.local/share/pnpm/store:/home/user/.local/share/pnpm/store:ro']
 ```
 */
export function pnpmStoreMountArguments({
  home,
}: {
  readonly home?: string;
} = {},): readonly string[] {
  /**
   Store directory, spelled once so the mount cannot join two different paths.
   */
  const storeDirectory = pnpmStoreDirectory(home === undefined ? {} : { home, },);

  return [
    '--volume',
    `${storeDirectory}:${storeDirectory}:ro`,
  ];
}

//endregion Mount arguments
