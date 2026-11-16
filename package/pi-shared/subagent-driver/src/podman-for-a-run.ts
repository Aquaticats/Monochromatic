/**
 The `podman` a run's unit has first on its `PATH`, and the `PATH` that puts it there.

 Why: a unit with any path property runs in a user namespace of its own, and a rootless container needs a namespace range that one
 cannot give, so `podman` started inside the unit fails. The program here hands each `podman` command to a service of the account's
 manager, which is outside that namespace and outside the unit's mounts, so the containers a run's brief tells it to start work.

 What it is: a link named `podman` in the run's own folder, pointing at this package's
 `hand-a-podman-command-to-the-manager` entry, so the package's own code is what runs and no shell line is built anywhere.

 @module
 */

import {
  chmod,
  mkdir,
  symlink,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { siblingCommandEntry, } from './sibling-command-entry.ts';

/**
 What the run's own folder calls the folder holding its `podman`.
 */
const CONTAINER_ROUTE_FOLDER: string = 'bin';

/**
 What the link is called, which is the name a run types.
 */
const PODMAN_NAME: string = 'podman';

/**
 What making the run's `podman` needs.
 */
export type PodmanOfARunRequest = {
  /**
   The run's own folder, which this start made.
   */
  readonly runFolder: string;

  /**
   The entry of the program that hands one `podman` command to the account's manager, which the link names.
   */
  readonly entry: string;
};

/**
 The mode the link is given, which makes it a program the unit can run: the owner may read, write and run it, and nobody else may.
 */
const EXECUTABLE_ONLY_BY_ITS_OWNER = 0o700;

/**
 Makes the `podman` a run's unit finds first on its `PATH`.

 @param request - the run's own folder and the entry of the program that hands a command to the manager

 @returns the folder to put first on the unit's `PATH`

 @throws the error the link gives, such as a run folder that is not there

 @example
 ```ts
 const folder = await makeThePodmanOfARun({ runFolder: '/runs/a-1', entry: '/pkg/src/hand-a-podman-command-to-the-manager.ts', });
 ```
 */
export async function makeThePodmanOfARun(request: PodmanOfARunRequest,): Promise<string> {
  /**
   The run's own folder and the entry the link names, each on its own line.
   */
  const {
    runFolder,
    entry,
  } = request;
  /**
   The folder the link lives in.
   */
  const folder = join(
    runFolder,
    CONTAINER_ROUTE_FOLDER,
  );
  await mkdir(
    folder,
    { recursive: true, },
  );
  await symlink(
    entry,
    join(
      folder,
      PODMAN_NAME,
    ),
  );
  // The entry is already executable when the package is installed; a link to a source file in a checkout keeps its own mode, which
  // is the executable one this package's entries carry.
  await chmod(
    join(
      folder,
      PODMAN_NAME,
    ),
    EXECUTABLE_ONLY_BY_ITS_OWNER,
  );
  return folder;
}

/**
 Puts the run's own `podman` first on a `PATH`.

 Why first: a run that types `podman` must reach this program, not the account's own `podman`, which cannot start a container inside
 the unit at all.

 @param folder - the folder the run's `podman` is in, which `makeThePodmanOfARun` made

 @param path - the `PATH` the session has

 @returns that `PATH` with the folder first

 @example
 ```ts
 const path = pathWithThePodmanOfARun({ folder: '/runs/a-1/bin', path: '/usr/bin:/bin', });
 ```
 */
export function pathWithThePodmanOfARun({
  folder,
  path
}: {
  readonly folder: string;
  readonly path: string
},): string {
  return `${folder}:${path}`;
}

/**
 Makes the run's `podman` and puts it first on the `PATH` a run's units get.

 @param request - the run's own folder, the package folder the program's entry is found in, and the `PATH` the session has

 @returns the `PATH` that finds the run's own `podman` first

 @throws Error when the program's entry cannot be found, and the error the link gives

 @example
 ```ts
 const path = await pathOfARunThatCanStartContainers({ runFolder: '/runs/a-1', packageFolder: '/pkg/src', path: '/usr/bin:/bin', });
 ```
 */
export async function pathOfARunThatCanStartContainers(request: {
  readonly runFolder: string;
  readonly packageFolder: string;
  readonly path: string;
}): Promise<string> {
  /**
   The run's folder, the package folder the entry is found in, and the session's `PATH`, each on its own line.
   */
  const {
    runFolder,
    packageFolder,
    path,
  } = request;
  /**
   The folder the run's own `podman` is in.
   */
  const folder = await makeThePodmanOfARun({
    runFolder,
    entry: await siblingCommandEntry({
      folder: packageFolder,
      command: 'hand-a-podman-command-to-the-manager',
    },),
  },);
  return pathWithThePodmanOfARun({
    folder,
    path,
  },);
}
