/**
 Node-only scratch root for module tests that create files.

 Which device backs the system temporary directory is unknowable: it can be a
 small tmpfs with a per-user quota, and when leaked fixtures filled one, every
 command in every session that wrote there failed. Tests therefore create their
 fixtures under `scratchRoot` instead of `tmpdir()`.

 Import it by its own subpath, `@monochromatic-dev/test-harness-core/ts/scratch-root.ts`.
 It is not re-exported from `index.ts`, because the platform-neutral build must
 load in browsers, where `node:` imports do not exist.

 @module
 */

import {
  existsSync,
  mkdirSync,
} from 'node:fs';
import { homedir, } from 'node:os';
import {
  dirname,
  join,
} from 'node:path';

/**
 Name of the environment variable that moves the scratch root, for a run that
 wants its fixtures somewhere specific.
 */
const SCRATCH_ROOT_VARIABLE = 'MONOCHROMATIC_TEST_SCRATCH';

/**
 File that marks the workspace root, whose `.cache` directory is ignored by
 version control and so is a safe home for fixtures.
 */
const WORKSPACE_MARKER = 'pnpm-workspace.yaml';

/**
 Finds the nearest directory at or above `start` that holds the workspace
 marker.

 @param start - Directory to begin the upward walk from
 @returns The workspace root, or `undefined` when no ancestor holds the marker
 */
function findWorkspaceRoot(start: string,): string | undefined {
  let directory = start;
  while (!existsSync(join(directory, WORKSPACE_MARKER,),)) {
    const parent = dirname(directory,);
    if (parent === directory) {
      return undefined;
    }
    directory = parent;
  }
  return directory;
}

/**
 Picks where this run's fixtures live, without creating it.

 @returns The override from `MONOCHROMATIC_TEST_SCRATCH` when set, else
   `.cache/test-scratch` under the workspace root, else `temp/agent/test-scratch`
   under the home directory when the process runs outside a checkout
 */
function chooseScratchRoot(): string {
  const override = process.env[SCRATCH_ROOT_VARIABLE];
  if (override !== undefined && override !== '') {
    return override;
  }
  const workspaceRoot = findWorkspaceRoot(process.cwd(),);
  if (workspaceRoot !== undefined) {
    return join(workspaceRoot, '.cache', 'test-scratch',);
  }
  return join(homedir(), 'temp', 'agent', 'test-scratch',);
}

/**
 Directory that holds every fixture a module test creates, never the system
 temporary directory.

 Create the directory on demand, so a caller can pass it straight to
 `mkdtemp(join(scratchRoot(), 'prefix-'))`.

 @returns Absolute path of the existing scratch root

 @example
 ```ts
 const directory = await mkdtemp(join(scratchRoot(), 'fixture-',),);
 ```
 */
export function scratchRoot(): string {
  const root = chooseScratchRoot();
  mkdirSync(root, { recursive: true, },);
  return root;
}
