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
 The words of the line that unmounts the checkout itself.
 */
const UNMOUNT_THE_CHECKOUT = `umount ${scanCheckout}`;

/**
 The words of the line that unmounts every mount under the checkout, the slot a previous run's clone stands at
 included.
 */
const UNMOUNT_UNDER_THE_CHECKOUT = `findmnt --list --noheadings --output TARGET | grep --fixed-strings ${scanCheckout}`;

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
 Where one piece of the script stands among its text.

 @param piece - the text whose first offset is wanted

 @returns the offset of that text, or -1 when the script does not hold it
 */
function at({ piece, }: { readonly piece: string; },): number {
  return script()
    .indexOf(piece,);
}

await describe({
  name: '',
  children: [
    describe({
      name: 'the holder-scan arm\'s script',
      children: [
        it({
          name: 'unmounts every mount under its checkout before it removes the folder',
          fn: async () => {
            expect(at({ piece: UNMOUNT_UNDER_THE_CHECKOUT, },),).toBeGreaterThan(-1,);
            expect(at({ piece: UNMOUNT_UNDER_THE_CHECKOUT, },),).toBeLessThan(at({ piece: REMOVE, },),);
          },
        },),
        it({
          name: 'unmounts the checkout itself before it removes the folder',
          fn: async () => {
            expect(at({ piece: UNMOUNT_THE_CHECKOUT, },),).toBeGreaterThan(-1,);
            expect(at({ piece: UNMOUNT_THE_CHECKOUT, },),).toBeLessThan(at({ piece: REMOVE, },),);
          },
        },),
        it({
          name: 'removes the folder, which is the line that refuses while a previous run\'s dataset stands there',
          fn: async () => {
            expect(at({ piece: REMOVE, },),).toBeGreaterThan(-1,);
          },
        },),
      ],
    },),
  ],
},);
