/**
 Scenario run inside a disposable user and network namespace by `tunnel-traffic-block.integration.test.ts`.
 It exercises the real `nft` on the traffic block and prints what `nft monitor` saw while a block that is
 already in place was replaced,
 and while the old sequence (delete, then install) ran as the positive control.
 It changes nothing outside the namespace it runs in.

 @module
 */

import { spawn, } from 'node:child_process';
import {
  mkdtemp,
  rm,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { setTimeout as wait, } from 'node:timers/promises';

import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import {
  installTrafficBlock,
  removeTrafficBlock,
  trafficBlockExists,
} from '../dist/final/node/tunnel-traffic-block.mjs';

/**
 Interface name the block is made for.
 */
const INTERFACE_NAME = 'wgscenario';

/**
 Time `nft monitor` is given to start listening and to print the events of a change.
 */
const MONITOR_SETTLE_MS = 400;

/**
 Makes a scratch directory in the test scratch root,
 removed when the scope exits,
 because the real one may belong to another user inside the namespace.

 @returns A disposable directory.

 @example
 ```ts
 await using dir = await scratchDirectory();
 ```
 */
async function scratchDirectory(): ReturnType<NonNullable<Parameters<typeof installTrafficBlock>[0]['makeDirectory']>> {
  /**
   Path of the new directory.
   */
  const path = await mkdtemp(join(
    scratchRoot(),
    'wg-quicker-scenario-',
  ),);
  return {
    path,
    [Symbol.asyncDispose]: async function removeScratchDirectory(): Promise<void> {
      await rm(
        path,
        {
          recursive: true,
          force: true,
        },
      );
    },
  };
}

/**
 Installs the block in the namespace.

 @example
 ```ts
 await install();
 ```
 */
async function install(): Promise<void> {
  await installTrafficBlock({
    interfaceName: INTERFACE_NAME,
    makeDirectory: scratchDirectory,
  },);
}

/**
 Runs `work` while `nft monitor` listens and returns the lines it printed.

 @param work - The change to watch.

 @returns The monitor's output during the change.

 @example
 ```ts
 await watching({ work: install });
 ```
 */
async function watching({ work, }: { readonly work: () => Promise<void>; },): Promise<string> {
  /**
   The monitor process.
   */
  const monitor = spawn(
    'nft',
    ['monitor',],
    { stdio: [
      'ignore',
      'pipe',
      'inherit',
    ], },
  );
  /**
   Output captured so far.
   */
  const captured = { text: '', };
  /**
   Keeps what the monitor prints.

   @param chunk - A piece of its output.
   */
  function collect(chunk: string,): void {
    captured.text += chunk;
  }
  monitor.stdout
    .setEncoding('utf8',)
    .on(
    'data',
    collect,
    );
  await wait(MONITOR_SETTLE_MS,);
  await work();
  await wait(MONITOR_SETTLE_MS,);
  monitor.kill('SIGTERM',);
  return captured.text;
}

await install();
/**
 Events while the block already in place was replaced.
 */
const replacement = await watching({ work: install, },);
/**
 The old sequence:
 delete the table, then install.
 */
async function deleteThenInstall(): Promise<void> {
  await removeTrafficBlock({ interfaceName: INTERFACE_NAME, },);
  await install();
}
/**
 Events while the old sequence ran.
 */
const control = await watching({ work: deleteThenInstall, },);
/**
 State with the block in place.
 */
const installed = await trafficBlockExists({ interfaceName: INTERFACE_NAME, },);
await removeTrafficBlock({ interfaceName: INTERFACE_NAME, },);
/**
 State after the removal.
 */
const removed = await trafficBlockExists({ interfaceName: INTERFACE_NAME, },);
// Removing a block that is not there is not a failure.
await removeTrafficBlock({ interfaceName: INTERFACE_NAME, },);
process.stdout
  .write(`${JSON.stringify({
    replacement,
    control,
    installed,
    removed,
  },)}\n`,);
