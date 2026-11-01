/**
 Process helpers for pure-Wayland Electron boundary tests.
 
 @example
 ```ts
 const fixture = await createWaylandFixture();
 ```
 */

import { once, } from 'node:events';
import {
  mkdtemp,
  rm,
} from 'node:fs/promises';
import { createRequire, } from 'node:module';
import { join, } from 'node:path';
import {
  spawn,
  type ChildProcess,
} from 'node:child_process';

import {
  boundaryFixtureRootParent,
  nestedScreenSize,
  shutdownDeadlineMs,
  type WaylandBoundaryFixture,
} from './wayland-constants.js';

/**
 CommonJS require rooted at this tool so the Electron binary path can be read.
 
 @example
 ```ts
 console.log(typeof require);
 ```
 */
const require = createRequire(import.meta.url,);

/**
 Options required to spawn nested Wayland hosting Electron.
 
 @example
 ```ts
 const options: SpawnNestedWaylandElectronOptions = { appDir: '/tmp/app', fixture, nestedWaylandBinary: '/bin/nws', statePathEnvironmentVariable: 'STATE' };
 ```
 */
export type SpawnNestedWaylandElectronOptions = {
  readonly appDir: string;
  readonly fixture: WaylandBoundaryFixture;
  readonly nestedWaylandBinary: string;
  readonly statePathEnvironmentVariable: string;
};

/**
 Parses value exported by the `electron` launcher package.
 
 @param value - Unknown value returned by `require('electron')`.
 
 @returns Absolute Electron executable path.
 
 @throws Error when package did not return a string path.
 
 @example
 ```ts
 parseElectronBinaryPath({ value: '/path/to/electron' });
 ```
 */
function parseElectronBinaryPath({ value, }: { readonly value: unknown; },): string {
  if ((typeof value) !== 'string')
    throw new Error('The electron package did not resolve to a binary path.',);

  return value;
}

/**
 Returns installed Electron binary path.
 
 @returns Electron executable path from launcher package.
 
 @example
 ```ts
 const electron = electronBinaryPath();
 ```
 */
function electronBinaryPath(): string {
  return parseElectronBinaryPath({ value: require('electron',) as unknown, },);
}

/**
 Creates temp paths for socket, state file, and screenshot output.
 
 @returns Boundary-test temp fixture paths.
 
 @example
 ```ts
 const fixture = await createWaylandFixture();
 ```
 */
export async function createWaylandFixture(): Promise<WaylandBoundaryFixture> {
  /**
   Temporary root directory for this test run.
   */
  const root = await mkdtemp(
    join(
      boundaryFixtureRootParent,
      'electron-wayland-',
    ),
  );

  return {
    root,
    socketPath: join(
      root,
      'control.sock',
    ),
    statePath: join(
      root,
      'state.json',
    ),
    [Symbol.asyncDispose]: async function removeFixture(): Promise<void> {
      await rm(
        root,
        {
          force: true,
          recursive: true,
        },
      );
    },
  };
}

/**
 Spawns nested Wayland session hosting an Electron app.
 
 @param appDir - Staged Electron app directory.
 
 @param fixture - Temp paths shared with control client.
 
 @param nestedWaylandBinary - Nested compositor binary path.
 
 @param statePathEnvironmentVariable - Environment variable receiving state path.
 
 @returns Spawned nested compositor process.
 
 @example
 ```ts
 const child = spawnNestedWaylandElectron({ appDir: '/tmp/app', fixture, nestedWaylandBinary: '/bin/nws', statePathEnvironmentVariable: 'STATE' });
 ```
 */
export function spawnNestedWaylandElectron(
  {
    appDir,
    fixture,
    nestedWaylandBinary,
    statePathEnvironmentVariable,
  }: SpawnNestedWaylandElectronOptions,
): ChildProcess {
  /**
   Electron executable provided by the installed Electron package.
   */
  const electron = electronBinaryPath();

  return spawn(
    nestedWaylandBinary,
    [
      '--socket',
      fixture.socketPath,
      '--size',
      nestedScreenSize,
      '--',
      '/usr/bin/env',
      '--unset=DISPLAY',
      'XDG_SESSION_TYPE=wayland',
      `${statePathEnvironmentVariable}=${fixture.statePath}`,
      electron,
      appDir,
    ],
    {
      env: {
        ...process.env,
        XDG_SESSION_TYPE: 'wayland',
      },
      stdio: 'inherit',
    },
  );
}

/**
 Formats a Node child-process exit event payload for diagnostics.
 
 @param code - Raw exit code payload.
 
 @param signal - Raw exit signal payload.
 
 @returns Human-readable exit status.
 
 @example
 ```ts
 formatExitStatus({ code: 0, signal: 'SIGTERM' });
 ```
 */
function formatExitStatus(
  {
    code,
    signal,
  }: {
    readonly code: number | string;
    readonly signal: string;
  },
): string {
  return `code ${code} signal ${signal}`;
}

/**
 Checks whether a value is an indexable unknown array.
 
 @param value - Value to check.
 
 @returns Whether value is an array of unknown entries.
 
 @example
 ```ts
 isReadonlyUnknownArray([0]);
 ```
 */
function isReadonlyUnknownArray(value: unknown,): value is readonly unknown[] {
  return Array.isArray(value,);
}

/**
 Parses Node's child-process `exit` event payload.
 
 @param value - Raw payload returned by `events.once`.
 
 @returns Raw exit status values.
 
 @throws Error when event payload is unexpectedly shaped.
 
 @example
 ```ts
 parseExitEvent({ value: [0, null] });
 ```
 */
function parseExitEvent(
  { value, }: { readonly value: unknown; },
): {
  readonly code: number | string;
  readonly signal: string;
} {
  if (!isReadonlyUnknownArray(value,))
    throw new Error('Child-process exit event payload was not an array.',);

  /**
   Raw exit code and signal payloads from Node's `exit` event.
   */
  const [code, signal,] = value;
  if ((code !== null) && ((typeof code) !== 'number'))
    throw new Error('Child-process exit code was neither a number nor null.',);
  if ((signal !== null) && ((typeof signal) !== 'string'))
    throw new Error('Child-process exit signal was neither a string nor null.',);

  return {
    code: code ?? 'null',
    signal: signal ?? 'null',
  };
}

/**
 Waits for a child process to exit successfully.
 
 @param child - Process to observe.
 
 @mutates child through child.kill timeout termination and once exit-listener registration
 
 @example
 ```ts
 await waitForSuccessfulExit({ child });
 ```
 */
export async function waitForSuccessfulExit(
  { child, }: { readonly child: ChildProcess; },
): Promise<void> {
  /**
   Timeout that terminates the child if quit does not complete.
   */
  const timeout = setTimeout(
    function killHungChild(): void {
      child.kill('SIGTERM',);
    },
    shutdownDeadlineMs,
  );

  /**
   Raw exit event payload from child process.
   */
  const exit = await once(
    child,
    'exit',
  );
  clearTimeout(timeout,);

  /**
   Parsed exit status values.
   */
  const {
    code,
    signal,
  } = parseExitEvent({ value: exit, },);

  if (code !== 0)
    throw new Error(`Nested Wayland session exited with ${formatExitStatus({
      code,
      signal,
    },)}`,
    );
}
