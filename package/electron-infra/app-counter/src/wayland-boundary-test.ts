/**
 End-user boundary test for the Electron counter under pure Wayland.
 
 The test hosts Electron inside this repo's nested Wayland compositor,
 launches the app with `DISPLAY` unset, activates the rendered button through
 compositor keyboard input, and waits for the main process to mirror renderer
 state into a JSON file.
 
 @example
 ```ts
 await runElectronCounterWaylandBoundaryTest();
 ```
 */

import { runWaylandElectronBoundaryTest, } from '@monochromatic-dev/electron-infra-core/ts/wayland-test';

/**
 Environment variable consumed by the Electron app's main process.
 
 @example
 ```ts
 console.log(statePathEnvironmentVariable);
 ```
 */
const statePathEnvironmentVariable = 'MONOCHROMATIC_ELECTRON_COUNTER_STATE_PATH';

/**
 Runs the complete pure-Wayland interaction test.
 
 @example
 ```ts
 await runElectronCounterWaylandBoundaryTest();
 ```
 */
async function runElectronCounterWaylandBoundaryTest(): Promise<void> {
  await runWaylandElectronBoundaryTest({
    packageRoot: process.cwd(),
    statePathEnvironmentVariable,
    screenshotName: 'after-click.png',
    steps: [
      { expected: { count: 0, }, },
      {
        commands: [
          'key tab',
          'key space',
        ],
        expected: { count: 1, },
      },
    ],
  },);
}

await runElectronCounterWaylandBoundaryTest();
