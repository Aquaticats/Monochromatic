/**
 What the path of a nested subvolume holds when its copy is placed there. A snapshot shows each nested subvolume as an
 empty folder, but a send never carries that folder (measured on the fixture: `rmdir` of the placeholder in the placed
 copy of the main checkout failed with "No such file or directory"), so the path is judged by what is there.

 @module
 */

import {
  chmod,
  chown,
  lstat,
  mkdir,
  readdir,
  stat,
} from 'node:fs/promises';
import {
  dirname,
  join,
  relative,
} from 'node:path';
import { ancestorChain, } from './canonical-path.ts';
import type { Runner, } from './command-runner.ts';
import { pathExists, } from './completion-records.ts';
import { errorCodeOf, } from './error-code-of.ts';
import { MoveStopError, } from './errors.ts';
import type { MoveOptions, } from './move-options.ts';
import { isSubvolume, } from './subvolume-root-check.ts';

/**
 How many entries of a folder that is not empty a stop names.
 */
const NAMED_ENTRIES = 5;

/**
 Gives a folder the owner and permission bits of the source folder it stands for.

 @param folder - the folder just made

 @param source - the folder of the live tree it stands for

 @example
 ```ts
 import { copyFolderAttributes, } from './placement-path.ts';

 await copyFolderAttributes({ folder: '/final/wt', source: '/main/wt', },);
 ```
 */
export async function copyFolderAttributes(
  {
    folder,
    source,
  }: {
    readonly folder: string;
    readonly source: string;
  },
): Promise<void> {
  try {
    /**
     The source folder's owner and mode.
     */
    const info = await stat(source,);
    await chown(
      folder,
      info.uid,
      info.gid,
    );
    // The permission bits are the low twelve bits of the mode; the file type bits above them are not set.
    await chmod(
      folder,
      info.mode % PERMISSION_MODULUS,
    );
  }
  catch (error) {
    if (errorCodeOf(error,) !== 'ENOENT') {
      throw error;
    }
  }
}

/**
 Twelve bits of permissions (setuid, setgid, sticky and the nine rwx bits), as a modulus on a mode.
 */
const PERMISSION_MODULUS = 4_096;

/**
 Makes the path of a nested subvolume ready for its copy: parent folders made if missing, an empty folder removed, and
 anything else refused.

 @param options - the options

 @param runner - the runner

 @param target - the final path of the nested subvolume

 @throws MoveStopError naming the path when it is a file, a link, a subvolume, or a folder that is not empty

 @example
 ```ts
 import { preparePlacementTarget, } from './placement-path.ts';

 await preparePlacementTarget({ options, runner, target, },);
 ```
 */
export async function preparePlacementTarget(
  {
    options,
    runner,
    target,
  }: {
    readonly options: MoveOptions;
    readonly runner: Runner;
    readonly target: string;
  },
): Promise<void> {
  if (!options.run) {
    runner.log(
      `${target}: checked when run: absent (parent folders are made with the source's owner and mode), an empty folder `
        + '(removed), anything else refused',
    );
    return;
  }
  /**
   The target and every folder above it, deepest first.
   */
  const chain = ancestorChain(target,);
  /**
   The parent folders between the final path and the target.
   */
  const parents = chain.slice(
    1,
    chain.indexOf(options.finalPath,),
  );
  /**
   Whether each of those exists.
   */
  const present = await Promise.all(parents.map(async function exists(folder,) {
    return await pathExists(folder,);
  },),);
  /**
   The parent folders that do not exist yet.
   */
  const missing = parents.filter(function isMissing(
    _folder,
    index,
  ) {
    return present[index] === false;
  },);
  if (missing.length > 0) {
    await mkdir(
      dirname(target,),
      { recursive: true, },
    );
    await Promise.all(missing.map(async function attributes(folder,) {
      await copyFolderAttributes({
        folder,
        source: join(
          options.mainPath,
          relative(
            options.finalPath,
            folder,
          ),
        ),
      },);
    },),);
  }
  if (!await pathExists(target,)) {
    return;
  }
  if (await isSubvolume({
    path: target,
    marker: options.subvolumeMarker,
  },)) {
    throw new MoveStopError(`${target} is already a subvolume, not an empty folder; placement refuses it and changed nothing.`,);
  }
  /**
   What the target is.
   */
  const info = await lstat(target,);
  if (!info.isDirectory()) {
    throw new MoveStopError(`${target} is not a folder (a file or a link); placement refuses it and changed nothing.`,);
  }
  /**
   What the folder holds.
   */
  const entries = await readdir(target,);
  if (entries.length > 0) {
    throw new MoveStopError(
      `${target} is a folder that is not empty (${entries.slice(
        0,
        NAMED_ENTRIES,
      )
        .join(', ',)}); placement refuses it and changed nothing.`,
    );
  }
  await runner.exec({
    command: 'rmdir',
    args: [target,],
  },);
}
