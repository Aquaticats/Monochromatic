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

import { run, } from '../dist/final/node/runner.mjs';
import { blockStatements, } from '../dist/final/node/tunnel-teardown-manual.mjs';
import {
  installTrafficBlock,
  removeTrafficBlock,
  trafficBlockState,
  trafficBlockTable,
} from '../dist/final/node/tunnel-traffic-block.mjs';

/**
 Characters of an nft refusal that the scenario prints.
 */
const DIAGNOSTIC_LIMIT = 120;

/**
 Interface name the block is made for.
 */
const INTERFACE_NAME = 'wgscenario';

/**
 What the block lets through in the comparison of the two ways to install it:
 a mark and an update host of each family.
 */
const CLEARNET = {
  mark: 100,
  hosts: [
    {
      family: '-4',
      address: '203.0.113.7',
    },
    {
      family: '-6',
      address: '2001:db8::7',
    },
  ],
  interfaces: [
    {
      family: '-4',
      name: 'dummy0',
    },
    {
      family: '-6',
      name: 'dummy0',
    },
  ],
} as const;

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
const installed = await trafficBlockState({ interfaceName: INTERFACE_NAME, },);
await removeTrafficBlock({ interfaceName: INTERFACE_NAME, },);
/**
 State after the removal.
 */
const removed = await trafficBlockState({ interfaceName: INTERFACE_NAME, },);
// Removing a block that is not there is not a failure.
await removeTrafficBlock({ interfaceName: INTERFACE_NAME, },);
/**
 Tries to install a block for an interface name and says whether `nft` took the name of its table.

 @param interfaceName - Name to try.

 @returns `installed` when the block was installed,
 which is then removed,
 and `refused` when `nft` did not take the table name.

 @example
 ```ts
 await tryName({ interfaceName: 'a=b' }); // 'refused'
 ```
 */
async function tryName({ interfaceName, }: { readonly interfaceName: string; },): Promise<'installed' | 'refused'> {
  try {
    await installTrafficBlock({
      interfaceName,
      makeDirectory: scratchDirectory,
    },);
  }
  catch (error) {
    process.stderr
      .write(`nft refused the table name of ${interfaceName}: ${String(error,)
        .slice(
          0,
          DIAGNOSTIC_LIMIT,
        )}\n`,);
    return 'refused';
  }
  await removeTrafficBlock({ interfaceName, },);
  return 'installed';
}

/**
 Lists the block table as `nft` prints it.

 @returns The listing.

 @example
 ```ts
 await listing();
 ```
 */
async function listing(): Promise<string> {
  return (await run({
    command: 'nft',
    args: [
      'list',
      'table',
      'inet',
      trafficBlockTable({ interfaceName: INTERFACE_NAME, },),
    ],
  },)).stdout;
}

/**
 Installs the block with what it lets through,
 the way wg-quicker does: a script file read by `nft -f`.
 */
async function installWithExceptions(): Promise<void> {
  await installTrafficBlock({
    interfaceName: INTERFACE_NAME,
    clearnet: CLEARNET,
    makeDirectory: scratchDirectory,
  },);
}

/**
 Installs the block with what it lets through,
 the way the by-hand steps print it: the statements joined into one argument of `nft`.
 */
async function installAsPrinted(): Promise<void> {
  await run({
    command: 'nft',
    args: [blockStatements({
      interfaceName: INTERFACE_NAME,
      clearnet: CLEARNET,
    },),],
  },);
}

await installWithExceptions();
/**
 The table as the script of wg-quicker installs it.
 */
const byScript = await listing();
await removeTrafficBlock({ interfaceName: INTERFACE_NAME, },);
await installAsPrinted();
/**
 The table as the printed one-argument form installs it.
 */
const byPrintedForm = await listing();
/**
 Events while the printed form replaced a block that was in place.
 */
const printedReplacement = await watching({ work: installAsPrinted, },);
/**
 The table after the printed form replaced the block.
 */
const afterPrintedReplacement = await listing();
await removeTrafficBlock({ interfaceName: INTERFACE_NAME, },);
/**
 What `nft` does with the characters that the rule for interface names allows or leaves out of the name of a table.
 */
const tableNames = {
  underscore: await tryName({ interfaceName: 'a_b', },),
  dot: await tryName({ interfaceName: 'a.b', },),
  dash: await tryName({ interfaceName: 'a-b', },),
  equals: await tryName({ interfaceName: 'a=b', },),
  plus: await tryName({ interfaceName: 'a+b', },),
};
process.stdout
  .write(`${JSON.stringify({
    tableNames,
    replacement,
    control,
    installed,
    removed,
    sameTable: byScript === byPrintedForm,
    sameTableAfterReplacement: byScript === afterPrintedReplacement,
    printedReplacement,
  },)}\n`,);
