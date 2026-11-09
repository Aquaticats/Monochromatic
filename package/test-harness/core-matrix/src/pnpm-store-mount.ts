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

/**
 Names every spelling of the store a container may have to see.
 */
export const STORE_DIRECTORY_VARIABLE = 'PNPM_CONFIG_STORE_DIR';

//endregion Names

//region Mount arguments

/**
 Mount options of one store spelling: source and destination equal, and read-only.

 @param directory - absolute store directory, spelled the way the container must see it

 @returns `--volume` and the mount argument

 @example
 ```ts
 const options = mountOptionsOf('/home/user/.local/share/pnpm/store');
 ```
 */
function mountOptionsOf(directory: string,): readonly string[] {
  return [
    '--volume',
    `${directory}:${directory}:ro`,
  ];
}

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

 pnpm writes each `node_modules` link as a relative path from the link to the store's physical directory, so the path
 a link resolves to depends on how deep the checkout is mounted: a checkout mounted at `/workspace` or `/work` makes
 the link climb past the filesystem root and land on the home-relative spelling of the store, while a checkout mounted
 at its host path keeps the physical spelling. On a host whose home directory is reached through a symbolic link those
 two spellings differ (`doc/troubleshooting/pnpm-symlinked-store-path.md`), so both directories are mounted.

 @param home - current user's home directory; taken from the runtime unless a caller supplies one

 @param physicalStoreDirectory - physical store spelling, read from {@link STORE_DIRECTORY_VARIABLE} unless supplied

 @returns `--volume` options, one per distinct store spelling, each read-only and with the source and destination spelled
   the same path

 @example
 ```ts
 const options = pnpmStoreMountArguments({ home: '/home/user' });
 // ['--volume', '/home/user/.local/share/pnpm/store:/home/user/.local/share/pnpm/store:ro']
 ```
 */
export function pnpmStoreMountArguments({
  home,
  physicalStoreDirectory = process.env[STORE_DIRECTORY_VARIABLE],
}: {
  readonly home?: string;
  readonly physicalStoreDirectory?: string;
} = {},): readonly string[] {
  /**
   Store directory spelled from the home directory, which is the spelling a link keeps when the checkout is mounted
   shallower than it is on the host.
   */
  const logicalDirectory = pnpmStoreDirectory(home === undefined ? {} : { home, },);
  /**
   Physical spelling this host installs into, from mise; empty when nothing supplies one.
   */
  const physicalDirectory = (physicalStoreDirectory ?? '').trim();
  /**
   Every directory to mount, deduplicated so a host whose home is not a symbolic link gets one mount.
   */
  const directories = ((physicalDirectory === '') || (physicalDirectory === logicalDirectory))
    ? [
      logicalDirectory,
    ]
    : [
      logicalDirectory,
      physicalDirectory,
    ];

  return directories.flatMap(mountOptionsOf,);
}

//endregion Mount arguments
