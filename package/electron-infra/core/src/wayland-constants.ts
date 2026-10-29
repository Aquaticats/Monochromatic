/**
 Defaults for pure-Wayland Electron boundary tests.
 
 @example
 ```ts
 console.log(nestedScreenSize);
 ```
 */

import { tmpdir, } from 'node:os';
import {
  join,
  resolve,
} from 'node:path';

/**
 Repository root derived from a package root at `package/<family>/<name>`.
 
 @param packageRoot - Package directory used as task working directory.
 
 @returns Repository root path.
 
 @example
 ```ts
 defaultRepoRoot({ packageRoot: process.cwd() });
 ```
 */
export function defaultRepoRoot({ packageRoot, }: { readonly packageRoot: string; },): string {
  return resolve(
    packageRoot,
    '..',
    '..',
    '..',
  );
}

/**
 Staged app directory loaded by Electron.
 
 @param packageRoot - Package directory used as task working directory.
 
 @returns Default staged app directory.
 
 @example
 ```ts
 defaultAppDir({ packageRoot: process.cwd() });
 ```
 */
export function defaultAppDir({ packageRoot, }: { readonly packageRoot: string; },): string {
  return join(
    packageRoot,
    'dist',
    'app',
  );
}

/**
 Path to this repo's nested Wayland compositor binary.
 
 @param repoRoot - Repository root path.
 
 @returns Default nested Wayland compositor binary path.
 
 @example
 ```ts
 defaultNestedWaylandBinary({ repoRoot: '/repo' });
 ```
 */
export function defaultNestedWaylandBinary(
  { repoRoot, }: { readonly repoRoot: string; },
): string {
  return join(
    repoRoot,
    'package',
    'cli',
    'nested-wayland-session',
    'target',
    'release',
    'monochromatic-nested-wayland-session',
  );
}

/**
 Temp directory for per-run boundary-test fixtures.
 
 @example
 ```ts
 console.log(boundaryFixtureRootParent);
 ```
 */
export const boundaryFixtureRootParent: string = tmpdir();

/**
 Nested test screen size.
 
 @example
 ```ts
 console.log(nestedScreenSize);
 ```
 */
export const nestedScreenSize = '800x600';

/**
 Polling interval for socket and state readiness checks, in milliseconds.
 
 @example
 ```ts
 console.log(pollIntervalMs);
 ```
 */
export const pollIntervalMs = 50;

/**
 Deadline for compositor socket readiness, in milliseconds.
 
 @example
 ```ts
 console.log(socketReadyDeadlineMs);
 ```
 */
export const socketReadyDeadlineMs = 10_000;

/**
 Deadline for renderer state observation, in milliseconds.
 
 @example
 ```ts
 console.log(stateReadyDeadlineMs);
 ```
 */
export const stateReadyDeadlineMs = 10_000;

/**
 Deadline for one compositor control-socket command, in milliseconds.
 
 @example
 ```ts
 console.log(controlResponseDeadlineMs);
 ```
 */
export const controlResponseDeadlineMs = 10_000;

/**
 Deadline for hosted app shutdown after `quit`, in milliseconds.
 
 @example
 ```ts
 console.log(shutdownDeadlineMs);
 ```
 */
export const shutdownDeadlineMs = 10_000;

/**
 Runtime values allocated for a single boundary test run.
 
 @example
 ```ts
 const fixture: WaylandBoundaryFixture = {
   root: '/tmp/x',
   socketPath: '/tmp/x/nws.sock',
   statePath: '/tmp/x/state.json',
   [Symbol.asyncDispose]: async () => {},
 };
 ```
 */
export type WaylandBoundaryFixture = {
  readonly root: string;
  readonly socketPath: string;
  readonly statePath: string;

  /**
   Removes temporary fixture files at end of boundary-test scope.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};
