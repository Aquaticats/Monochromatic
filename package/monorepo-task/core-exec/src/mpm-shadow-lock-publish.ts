/**
 Making an install visible, the way `mpm-shadow install` does for an archive-route tool that this repository publishes
 (`archive_staging.rs`, `publish_staged`; `published_archive.rs`): a staged folder takes the version's name with one rename
 and a copy that was there is kept only when it is finished and prints the version; `current` is pointed at the newest
 finished version; and every older version folder is removed ("No older repo-maintained tools should ever exist when newer
 is present").

 Imports only `node:` modules, so it runs from source on a checkout without `node_modules`.

 @module
 */

import {
  lstat,
  rename,
  rm,
  symlink,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  isNotFound,
  VERSION_TIMEOUT_MS,
  versionOf,
} from './mpm-shadow-lock-fetch.ts';
import {
  findProgramUnder,
  isFinishedVersion,
  listVersionNames,
  SEARCH_DEPTH,
} from './mpm-shadow-lock-folder.ts';
import {
  compareVersions,
  CURRENT_LINK_NAME,
  MpmShadowLockError,
} from './mpm-shadow-lock-read.ts';

/**
 Points the `current` link at a version, replacing the link in one step (a new link is made beside it and renamed over it).

 @param toolDirectory - `<toolchain root>/mpm-shadow`

 @param version - Name of the version folder, which is the link's relative target

 @example
 ```ts
 await pointCurrentAt({ toolDirectory, version: '0.2.1', },);
 ```
 */
async function pointCurrentAt(
  {
    toolDirectory,
    version,
  }: {
    readonly toolDirectory: string;
    readonly version: string;
  },
): Promise<void> {
  /**
   The new link, beside the old one.
   */
  const fresh = join(
    toolDirectory,
    `.${CURRENT_LINK_NAME}.link.${String(process.pid,)}`,
  );
  await rm(
    fresh,
    { force: true, },
  );
  await symlink(
    version,
    fresh,
  );
  await rename(
    fresh,
    join(
      toolDirectory,
      CURRENT_LINK_NAME,
    ),
  );
}

/**
 Makes a staged install the version folder.

 A folder at the version's name that is finished and whose program prints the version is kept, because another task installed
 the same checked release meanwhile; any other folder there (an unfinished one, a fake, a different program) is moved aside
 first, so a failed rename puts it back, and removed once the new one is in place.

 @param staging - The staged folder, with the marker written

 @param final - `<toolchain root>/mpm-shadow/<version>`

 @param version - The version that the program in the folder must print

 @param aside - Name for the moved-aside copy

 @throws MpmShadowLockError when the staged folder cannot take the version's name

 @example
 ```ts
 await publishStaged({ staging, final, version: '0.2.1', aside, },);
 ```
 */
export async function publishStaged(
  {
    staging,
    final,
    version,
    aside,
  }: {
    readonly staging: string;
    readonly final: string;
    readonly version: string;
    readonly aside: string;
  },
): Promise<void> {
  if (await isFinishedVersion(final,)) {
    /**
     What the folder's program says it is.
     */
    const answer = await versionOf({
      binary: await findProgramUnder({
        directory: final,
        depth: SEARCH_DEPTH,
      },),
      timeoutMs: VERSION_TIMEOUT_MS,
    },);
    if (answer.usable && (answer.version === version)) {
      await rm(
        staging,
        {
          recursive: true,
          force: true,
        },
      );
      return;
    }
  }
  /**
   Whether anything is at the version's name.
   */
  const hadCopy = await lstat(final,)
    .then(
    function exists() {
      return true;
    },
    function isMissing(error: unknown,) {
      if (isNotFound(error,))
        return false;
      throw error;
    },
    );
  await rm(
    aside,
    {
      recursive: true,
      force: true,
    },
  );
  if (hadCopy)
    await rename(
      final,
      aside,
    );
  try {
    await rename(
      staging,
      final,
    );
  }
  catch (error) {
    await rm(
      staging,
      {
        recursive: true,
        force: true,
      },
    );
    if (hadCopy)
      await rename(
        aside,
        final,
      );
    throw new MpmShadowLockError(
      `could not move the finished install to ${final} (${String(error,)})`,
      { cause: error, },
    );
  }
  await rm(
    aside,
    {
      recursive: true,
      force: true,
    },
  );
}

/**
 Points `current` at the newest finished version and removes every older version folder, as `mpm-shadow install` does after an
 archive install of a tool this repository publishes.

 @param toolDirectory - `<toolchain root>/mpm-shadow`

 @param log - Prints one note for the person running the task

 @example
 ```ts
 await finishLayout({ toolDirectory, log, },);
 ```
 */
export async function finishLayout(
  {
    toolDirectory,
    log,
  }: {
    readonly toolDirectory: string;
    readonly log: (line: string,) => void;
  },
): Promise<void> {
  /**
   Version folders, newest first, and which of them finished.
   */
  const names = await listVersionNames(toolDirectory,);
  /**
   For each version folder, whether its install finished.
   */
  const finished = await Promise.all(names.map(function check(name,) {
    return isFinishedVersion(join(
      toolDirectory,
      name,
    ),);
  },),);
  /**
   The newest finished version, when there is one.
   */
  const newest = names.find(function isFinishedName(
    _name,
    index,
  ) {
    return finished[index] === true;
  },);
  if (newest === undefined)
    return;
  await pointCurrentAt({
    toolDirectory,
    version: newest,
  },);
  await Promise.all(names.filter(function isOlder(name,) {
    return compareVersions({
      left: name,
      right: newest,
    },) < 0;
  },)
    .map(async function removeOlder(name,) {
      await rm(
        join(
          toolDirectory,
          name,
        ),
        {
          recursive: true,
          force: true,
        },
      );
      log(`removed the older mpm-shadow ${name}; only the newest is kept`,);
    },),);
}
