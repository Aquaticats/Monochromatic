/**
 Node-only scratch root for module tests that create files.

 Which device backs the system temporary directory is unknowable: it can be a
 small tmpfs with a per-user quota, and when leaked fixtures filled one, every
 command in every session that wrote there failed. Tests therefore create their
 fixtures under `scratchRoot` instead of `tmpdir()`.

 The root lives in the user cache directory, outside every checkout, so a
 fixture is never inside a repository or under a project configuration file
 that a tool would find by walking upward.

 Import it by its own subpath, `@monochromatic-dev/test-harness-core/ts/scratch-root.ts`.
 It is not re-exported from `index.ts`, because the platform-neutral build must
 load in browsers, where `node:` imports do not exist.

 @module
 */

import {
  mkdirSync,
  realpathSync,
} from 'node:fs';
import { homedir, } from 'node:os';
import { join, } from 'node:path';

/**
 Name of the environment variable that moves the scratch root, for a run that
 wants its fixtures somewhere specific.
 */
const SCRATCH_ROOT_VARIABLE = 'MONOCHROMATIC_TEST_SCRATCH';

/**
 Picks where this run's fixtures live, without creating it.

 @returns The override from `MONOCHROMATIC_TEST_SCRATCH` when set, else
   `monochromatic-dev/test-scratch` under `XDG_CACHE_HOME`, else under
   `.cache` in the home directory
 */
function chooseScratchRoot(): string {
  const override = process.env[SCRATCH_ROOT_VARIABLE];
  if (override !== undefined && override !== '') {
    return override;
  }
  const cacheHome = process.env['XDG_CACHE_HOME'];
  const base = cacheHome !== undefined && cacheHome !== ''
    ? cacheHome
    : join(homedir(), '.cache',);
  return join(base, 'monochromatic-dev', 'test-scratch',);
}

/**
 Directory that holds every fixture a module test creates, never the system
 temporary directory.

 Creates the directory on demand, so a caller can pass it straight to
 `mkdtemp(join(scratchRoot(), 'prefix-'))`.

 @returns Canonical absolute path of the existing scratch root, with symbolic links resolved

 @example
 ```ts
 const directory = await mkdtemp(join(scratchRoot(), 'fixture-',),);
 ```
 */
export function scratchRoot(): string {
  const root = chooseScratchRoot();
  mkdirSync(root, { recursive: true, },);
  // The canonical path, because a home directory reached through a symbolic link (`/home` pointing at
  // `/var/home`) makes a fixture's real path differ from the one this function would otherwise return.
  return realpathSync(root,);
}
