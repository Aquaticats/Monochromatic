/**
 First-party crate roots that carry a `Cargo.lock`, and snapshots of their lockfiles.

 @module
 */

import {
  glob,
  readFile,
  writeFile,
} from 'node:fs/promises';
import {
  dirname,
  join,
} from 'node:path';

//region Discovery

/**
 Manifest discovery globs, matching file-enforcer's Rust catalog sync.
 */
const MANIFEST_GLOBS = [
  'package/*/*/Cargo.toml',
  'package/*/*/*/Cargo.toml',
] as const;

/**
 Name of the lockfile in each crate root.
 */
const LOCKFILE_NAME = 'Cargo.lock';

/**
 Repo-relative directories of every first-party crate root with a `Cargo.lock`, sorted.

 @param root - repository root

 @returns crate directories

 @example
 ```ts
 await lockedCrateDirs('/repo'); // ['package/nested-wayland-session/cli', ...]
 ```
 */
export async function lockedCrateDirs(root: string,): Promise<readonly string[]> {
  /**
   Lockfile paths beside every manifest.
   */
  const locks = await Promise.all(MANIFEST_GLOBS.map(async function collect(pattern,): Promise<readonly string[]> {
    return await Array.fromAsync(glob(
      pattern.replace(
        'Cargo.toml',
        LOCKFILE_NAME,
      ),
      { cwd: root, },
    ),);
  },),);
  return [...new Set(locks.flat()
    .filter(function isFirstParty(path: string,): boolean {
      return (!path.includes('/target/',)) && (!path.includes('/node_modules/',));
    },)
    .map(function toDir(path: string,): string {
      return dirname(path,);
    },),),].toSorted();
}

//endregion Discovery

//region Snapshots

/**
 Lockfile text of one crate root.

 @example
 ```ts
 const snapshot: LockSnapshot = { crateDir: 'package/cli/a', source: 'version = 4\n', };
 ```
 */
export type LockSnapshot = {
  readonly crateDir: string;
  readonly source: string;
};

/**
 Reads every crate's lockfile.

 @param root - repository root

 @param crateDirs - repo-relative crate directories

 @returns one snapshot per crate, in order

 @example
 ```ts
 await readLocks({ root, crateDirs, });
 ```
 */
export async function readLocks({
  root,
  crateDirs,
}: {
  readonly root: string;
  readonly crateDirs: readonly string[]
},): Promise<readonly LockSnapshot[]> {
  return await Promise.all(crateDirs.map(async function read(crateDir: string,): Promise<LockSnapshot> {
    return {
      crateDir,
      source: await readFile(
        join(
          root,
          crateDir,
          LOCKFILE_NAME,
        ),
        'utf8',
      ),
    };
  },),);
}

/**
 Writes snapshots back, so a refused or dry-run update leaves every lockfile as it was.

 @param root - repository root

 @param snapshots - lockfiles to restore

 @example
 ```ts
 await restoreLocks({ root, snapshots, });
 ```
 */
export async function restoreLocks({
  root,
  snapshots,
}: {
  readonly root: string;
  readonly snapshots: readonly LockSnapshot[]
},): Promise<void> {
  await Promise.all(snapshots.map(async function restore(snapshot: LockSnapshot,): Promise<void> {
    await writeFile(
      join(
        root,
        snapshot.crateDir,
        LOCKFILE_NAME,
      ),
      snapshot.source,
    );
  },),);
}

//endregion Snapshots
