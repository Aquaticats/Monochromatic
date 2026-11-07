/**
 The check that comes before the first snapshot: a btrfs snapshot must be on the same filesystem as its source, and the
 placement of a received copy is a snapshot too, so the received folder and the final path must share a filesystem.
 A snapshot folder on another filesystem fails late, after the sources were found and the clock started, with only
 `Invalid cross-device link` to say why.

 Folders are compared by filesystem identity, never by device number: every btrfs subvolume has its own device number,
 so the checkout and the snapshot folder beside it on the same filesystem differ in `st_dev`.

 @module
 */

import { execFile, } from 'node:child_process';
import { promisify, } from 'node:util';
import { ancestorChain, } from './canonical-path.ts';
import { pathExists, } from './completion-records.ts';
import { MILLISECONDS_PER_SECOND, } from './constants.ts';
import { MoveStopError, } from './errors.ts';
import type { Subvolume, } from './inventory.ts';
import type { MoveOptions, } from './move-options.ts';

/**
 Specialized promisified `execFile`, typed through Node's own adapter contract.
 The generic `promisify(execFile)` form trips `strict-void-return`;
 see `doc/troubleshooting/oxlint-promisify-void-return.md`.
 */
const promisifyExecFile: (original: typeof execFile) => typeof execFile.__promisify__ = promisify;

/**
 Runs a program and waits for it to end.
 */
const execFileAsync = promisifyExecFile(execFile,);

/**
 The program that names the mount a path is on; read-only.
 */
const FINDMNT = 'findmnt';

/**
 Finds the nearest folder that exists, going up from a path that may not exist yet.

 @param path - the path

 @returns the path itself when it exists, else its nearest existing ancestor

 @example
 ```ts
 import { nearestExisting, } from './filesystem-preflight.ts';

 const folder = await nearestExisting('/var/home/user/not/made/yet',);
 // '/var/home/user'
 ```
 */
export async function nearestExisting(path: string,): Promise<string> {
  /**
   The path and its ancestors, deepest first.
   */
  const chain = ancestorChain(path,);
  /**
   Whether each of them exists.
   */
  const present = await Promise.all(chain.map(pathExists,),);
  return chain[present.indexOf(true,)] ?? path;
}

/**
 A source subvolume with the filesystem it is on.
 */
type SourceSide = {
  readonly path: string;
  readonly filesystem: string;
};

/**
 Reads one column of `findmnt` for the mount a path is on.

 @param column - the column name, such as `UUID` or `TARGET`

 @param path - an existing path

 @param deadlineSeconds - how long to wait for `findmnt`

 @returns the value, empty when the mount has none

 @throws MoveStopError when `findmnt` fails or does not answer in time
 */
async function mountColumn(
  {
    column,
    path,
    deadlineSeconds,
  }: {
    readonly column: string;
    readonly path: string;
    readonly deadlineSeconds: number;
  },
): Promise<string> {
  try {
    return (await execFileAsync(
      FINDMNT,
      [
        '--noheadings',
        '--output',
        column,
        '--target',
        path,
      ],
      {
        encoding: 'utf8',
        timeout: deadlineSeconds * MILLISECONDS_PER_SECOND,
      },
    )).stdout
      .trim();
  }
  catch (error) {
    throw new MoveStopError(
      `cannot tell which filesystem ${path} is on (findmnt --output ${column} --target failed)`,
      { cause: error, },
    );
  }
}

/**
 Names the filesystem a path is on: the filesystem UUID, or for a filesystem without one the mount point it is mounted
 at.

 @param path - a path that may not exist yet

 @param deadlineSeconds - how long to wait for each `findmnt` call

 @returns the identity, such as `uuid 6f1c...` or `mount /dev/shm`

 @throws MoveStopError when `findmnt` fails or does not answer in time

 @example
 ```ts
 import { filesystemOf, } from './filesystem-preflight.ts';

 const identity = await filesystemOf({ path: '/var/home/user', deadlineSeconds: 30, },);
 ```
 */
export async function filesystemOf(
  {
    path,
    deadlineSeconds,
  }: {
    readonly path: string;
    readonly deadlineSeconds: number;
  },
): Promise<string> {
  /**
   The existing folder to ask about.
   */
  const existing = await nearestExisting(path,);
  /**
   The filesystem UUID, which a memory filesystem does not have.
   */
  const uuid = await mountColumn({
    column: 'UUID',
    path: existing,
    deadlineSeconds,
  },);
  if (uuid !== '') {
    return `uuid ${uuid}`;
  }
  return `mount ${await mountColumn({
    column: 'TARGET',
    path: existing,
    deadlineSeconds,
  },)}`;
}

/**
 Refuses, before any snapshot, a layout whose snapshots or placements would cross filesystems.

 @param options - the options (canonical paths)

 @param subvolumes - every subvolume the pass covers

 @throws MoveStopError naming both paths and both filesystems when the snapshot folder is not on the sources' filesystem, or the received folder is not on the final path's filesystem

 @example
 ```ts
 import { requireSharedFilesystems, } from './filesystem-preflight.ts';

 await requireSharedFilesystems({ options, subvolumes, },);
 ```
 */
export async function requireSharedFilesystems(
  {
    options,
    subvolumes,
  }: {
    readonly options: MoveOptions;
    readonly subvolumes: readonly Subvolume[];
  },
): Promise<void> {
  /**
   How long one `findmnt` call may take.
   */
  const deadlineSeconds = options.commandDeadlineSeconds;
  /**
   The filesystem of the snapshot folder.
   */
  const snapshotSide = await filesystemOf({
    path: options.snapshotFolder,
    deadlineSeconds,
  },);
  /**
   Each source subvolume with its filesystem.
   */
  const sources = await Promise.all(subvolumes.map(async function sourceSide(subvolume,) {
    return {
      path: subvolume.path,
      filesystem: await filesystemOf({
        path: subvolume.path,
        deadlineSeconds,
      },),
    };
  },),);
  /**
   The first source on another filesystem than the snapshot folder.
   */
  const stranger = sources.find(function isElsewhere(source: Readonly<SourceSide>,) {
    return source.filesystem !== snapshotSide;
  },);
  if (stranger !== undefined) {
    throw new MoveStopError(
      `the snapshot folder ${options.snapshotFolder} (${snapshotSide}) and the source subvolume ${stranger.path} `
        + `(${stranger.filesystem}) are not on the same filesystem; a btrfs snapshot must be on the same filesystem as its `
        + 'source (the kernel refuses with Invalid cross-device link). Nothing was snapshotted.',
    );
  }
  /**
   The filesystem of the received folder.
   */
  const receivedSide = await filesystemOf({
    path: options.receivedFolder,
    deadlineSeconds,
  },);
  /**
   The filesystem of the final path.
   */
  const finalSide = await filesystemOf({
    path: options.finalPath,
    deadlineSeconds,
  },);
  if (receivedSide !== finalSide) {
    throw new MoveStopError(
      `the received folder ${options.receivedFolder} (${receivedSide}) and the final path ${options.finalPath} `
        + `(${finalSide}) are not on the same filesystem; placing a received copy is a snapshot of it, so both must be on `
        + 'the same filesystem. Nothing was snapshotted.',
    );
  }
}
