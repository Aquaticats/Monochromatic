/**
 Guard: the holder-scan arm's script takes down a mount a previous run left before it removes the folder that mount
 stands on, which is what lets a second run of the arm in one guest start.

 The arm's own run ends with the checkout's dataset still mounted at that folder and a run killed part-way also leaves
 the clone it attached at the slot inside it, so the script's own `rm` meets a mount point unless the script unmounts
 what stands first.
 */
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { scanCheckout, } from './adoption-fixture.ts';
import { holderScanScript, } from './holder-scan-arm-script-fixture.ts';

/**
 The guest the script is built for, which no line this guard reads depends on.
 */
const GUEST_NAME = 'issue-6615-unit';

/**
 The log the script writes, which no line this guard reads depends on.
 */
const LOG = 'holder-scan-arm.log';

/**
 The line that removes the arm's checkout, which refuses with `Device or resource busy` while anything is mounted on it.
 */
const REMOVE = `rm --recursive --force ${scanCheckout}`;

/**
 The script the arm runs inside the guest.

 @returns the whole script, as the arm hands it to the guest
 */
function script(): string {
  return holderScanScript({
    chosen: {
      name: GUEST_NAME,
      logDir: 'target/holder-scan-script-fixture-logs',
    },
    log: LOG,
  },);
}

/**
 The script's lines, which this guard reads in order.

 @returns one entry per line
 */
function lines(): readonly string[] {
  return script()
    .split('\n',);
}

/**
 Whether one line is the arm's removal of its checkout.

 @param line - one line of the script

 @returns true when the line removes the folder with `rm`
 */
function removesTheCheckout(line: string,): boolean {
  return line === REMOVE;
}

/**
 Whether one line unmounts every mount under the checkout, the slot a previous run's clone stands at included.

 @param line - one line of the script

 @returns true when the line reads the mount table for the checkout and unmounts what it names
 */
function unmountsUnderTheCheckout(line: string,): boolean {
  return line.includes('findmnt',) && line.includes(scanCheckout,);
}

/**
 Whether one line unmounts the checkout itself.

 @param line - one line of the script

 @returns true when the line begins an `umount` that names the checkout
 */
function unmountsTheCheckout(line: string,): boolean {
  return line.startsWith('umount ',) && line.includes(scanCheckout,);
}

await describe({
  name: '',
  children: [
    describe({
      name: holderScanScript.name,
      children: [
        it({
          name: 'removes the folder, which is the line that refuses while a previous run\'s dataset stands there',
          fn: async () => {
            /**
             Where the script removes its checkout.
             */
            const remove = lines()
              .findIndex(removesTheCheckout,);
            expect(remove,).toBeGreaterThanOrEqual(0,);
          },
        },),
        it({
          name: 'unmounts every mount under its checkout before it removes the folder',
          fn: async () => {
            /**
             Where the script takes down every mount under the checkout, and where it removes the folder.
             */
            const underTheCheckout = lines()
              .findIndex(unmountsUnderTheCheckout,);
            /**
             Where the script removes its checkout.
             */
            const remove = lines()
              .findIndex(removesTheCheckout,);
            expect(underTheCheckout,).toBeGreaterThanOrEqual(0,);
            expect(underTheCheckout,).toBeLessThan(remove,);
          },
        },),
        it({
          name: 'unmounts the checkout itself before it removes the folder',
          fn: async () => {
            /**
             Where the script unmounts the checkout itself, and where it removes the folder.
             */
            const theCheckout = lines()
              .findIndex(unmountsTheCheckout,);
            /**
             Where the script removes its checkout.
             */
            const remove = lines()
              .findIndex(removesTheCheckout,);
            expect(theCheckout,).toBeGreaterThanOrEqual(0,);
            expect(theCheckout,).toBeLessThan(remove,);
          },
        },),
      ],
    },),
  ],
},);
