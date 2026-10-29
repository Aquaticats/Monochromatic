/**
 High-level pure-Wayland Electron boundary-test runner.
 
 @example
 ```ts
 await runWaylandElectronBoundaryTest({
   packageRoot: process.cwd(),
   statePathEnvironmentVariable: 'APP_STATE',
   steps: [{ expected: { ready: true } }],
 });
 ```
 */

import { join, } from 'node:path';

import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import {
  defaultAppDir,
  defaultNestedWaylandBinary,
  defaultRepoRoot,
  socketReadyDeadlineMs,
} from './wayland-constants.js';
import {
  assertPathAccessible,
  expectOkControlCommand,
  waitForPath,
} from './wayland-control.js';
import {
  createWaylandFixture,
  spawnNestedWaylandElectron,
  waitForSuccessfulExit,
} from './wayland-process.js';
import {
  waitForObservedState,
  type ExpectedObservedState,
} from './wayland-state.js';

/**
 One expected state checkpoint, optionally preceded by compositor commands.
 
 @example
 ```ts
 const step: WaylandBoundaryStep = { commands: ['key tab', 'key space'], expected: { count: 1 } };
 ```
 */
export type WaylandBoundaryStep = {
  readonly commands?: readonly string[];
  readonly expected: ForeignBorrowed<ExpectedObservedState>;
};

/**
 Options for running an Electron app inside the nested Wayland compositor.
 
 @example
 ```ts
 const options: WaylandElectronBoundaryTestOptions = {
   packageRoot: process.cwd(),
   statePathEnvironmentVariable: 'APP_STATE',
   steps: [{ expected: { ready: true } }],
 };
 ```
 */
export type WaylandElectronBoundaryTestOptions = {
  readonly appDir?: string;
  readonly nestedWaylandBinary?: string;
  readonly packageRoot: string;
  readonly repoRoot?: string;
  readonly screenshotName?: string;
  readonly statePathEnvironmentVariable: string;
  readonly steps: readonly WaylandBoundaryStep[];
};

/**
 Asserts that this host can run the pure-Wayland boundary test.
 
 @example
 ```ts
 assertLinuxWaylandHost();
 ```
 */
function assertLinuxWaylandHost(): void {
  if (process.platform !== 'linux')
    throw new Error('Pure Wayland boundary test only runs on Linux.',);

  if (process.env
    .WAYLAND_DISPLAY
    === undefined)
    throw new Error('Pure Wayland boundary test requires a parent Wayland session.',);

  if (process.env
    .XDG_RUNTIME_DIR
    === undefined)
    throw new Error('Pure Wayland boundary test requires XDG_RUNTIME_DIR.',);
}

/**
 Runs compositor commands before waiting for expected observed state.
 
 @param socketPath - Nested compositor control socket path.
 
 @param statePath - State file written by Electron main process.
 
 @param step - Boundary-test step to execute.
 
 @mutates step - `JSON.stringify` may invoke accessors or proxy traps on expected state when timeout is reported.
 
 @example
 ```ts
 await runBoundaryStep({ socketPath: '/tmp/nws.sock', statePath: '/tmp/state.json', step: { expected: { count: 0 } } });
 ```
 */
async function runBoundaryStep(
  {
    socketPath,
    statePath,
    step,
  }: {
    readonly socketPath: string;
    readonly statePath: string;
    readonly step: WaylandBoundaryStep;
  },
): Promise<void> {
  await Promise.all((step.commands ?? []).map(function sendStepCommand(command,): Promise<void> {
    return expectOkControlCommand({
      command,
      socketPath,
    },);
  },),);
  await waitForObservedState({
    expected: step.expected,
    statePath,
  },);
}

/**
 Captures a screenshot after boundary-test steps complete.
 
 @param root - Boundary-test fixture root directory.
 
 @param screenshotName - Screenshot file basename.
 
 @param socketPath - Nested compositor control socket path.
 
 @example
 ```ts
 await captureBoundaryScreenshot({ root: '/tmp/run', screenshotName: 'after.png', socketPath: '/tmp/nws.sock' });
 ```
 */
async function captureBoundaryScreenshot(
  {
    root,
    screenshotName,
    socketPath,
  }: {
    readonly root: string;
    readonly screenshotName: string;
    readonly socketPath: string;
  },
): Promise<void> {
  await expectOkControlCommand({
    socketPath,
    command: `screenshot ${join(
      root,
      screenshotName,
    )}`,
  },);
}

/**
 Runs a staged Electron app through this repo's pure-Wayland boundary harness.
 
 @param appDir - Optional staged app directory, defaulting to `dist/app`.
 
 @param nestedWaylandBinary - Optional nested compositor binary path.
 
 @param packageRoot - Package directory used as task working directory.
 
 @param repoRoot - Optional repository root, defaulting from package root.
 
 @param screenshotName - Optional screenshot basename captured after steps.
 
 @param statePathEnvironmentVariable - Environment variable receiving test state path.
 
 @param steps - State checkpoints and optional control commands.
 
 @mutates steps - `JSON.stringify` may invoke accessors or proxy traps on foreign expected state.
 
 @example
 ```ts
 await runWaylandElectronBoundaryTest({ packageRoot: process.cwd(), statePathEnvironmentVariable: 'APP_STATE', steps: [{ expected: { count: 0 } }] });
 ```
 */
export async function runWaylandElectronBoundaryTest(
  {
    appDir,
    nestedWaylandBinary,
    packageRoot,
    repoRoot,
    screenshotName = 'after-boundary-test.png',
    statePathEnvironmentVariable,
    steps,
  }: WaylandElectronBoundaryTestOptions,
): Promise<void> {
  assertLinuxWaylandHost();

  /**
   Repository root used to locate the nested compositor binary.
   */
  const resolvedRepoRoot = repoRoot ?? defaultRepoRoot({ packageRoot, },);

  /**
   Staged app directory loaded by Electron.
   */
  const resolvedAppDir = appDir ?? defaultAppDir({ packageRoot, },);

  /**
   Nested compositor binary path.
   */
  const resolvedNestedWaylandBinary = nestedWaylandBinary ?? defaultNestedWaylandBinary({
    repoRoot: resolvedRepoRoot,
  },);

  await assertPathAccessible({
    path: resolvedAppDir,
    label: 'staged Electron app',
  },);
  await assertPathAccessible({
    path: resolvedNestedWaylandBinary,
    label: 'nested Wayland compositor binary',
  },);

  /**
   Temp fixture shared by nested compositor and control client.
   */
  await using fixture = await createWaylandFixture();

  /**
   Nested compositor process hosting Electron.
   */
  const child = spawnNestedWaylandElectron({
    appDir: resolvedAppDir,
    fixture,
    nestedWaylandBinary: resolvedNestedWaylandBinary,
    statePathEnvironmentVariable,
  },);

  try {
    await waitForPath({
      path: fixture.socketPath,
      deadlineMs: socketReadyDeadlineMs,
    },);
    await expectOkControlCommand({
      socketPath: fixture.socketPath,
      command: 'ping',
    },);

    for (const step of steps)
      // oxlint-disable-next-line eslint/no-await-in-loop -- steps intentionally observe state after each command group.
      await runBoundaryStep({
        socketPath: fixture.socketPath,
        statePath: fixture.statePath,
        step,
      },);

    await captureBoundaryScreenshot({
      root: fixture.root,
      screenshotName,
      socketPath: fixture.socketPath,
    },);
    await expectOkControlCommand({
      socketPath: fixture.socketPath,
      command: 'quit',
    },);
  }
  catch (error: unknown) {
    child.kill('SIGTERM',);
    throw error;
  }

  await waitForSuccessfulExit({ child, },);
}
