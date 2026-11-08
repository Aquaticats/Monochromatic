/**
 Which run endings pass for a run's own terms, and what a late failure turns an ending into.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  finalEndReason,
  NO_RUN_FAILURE,
} from '../dist/final/node/index.mjs';

/**
 A failure with words of its own.

 @param problem - what failed

 @returns the failure
 */
function failed(problem: string,): { readonly failed: true; readonly problem: string } {
  return {
    failed: true,
    problem,
  };
}

await describe({
  name: finalEndReason.name,
  children: [
    it({
      name: 'leaves an ending on a run\'s own terms alone when nothing failed late',
      fn: async () => {
        expect(finalEndReason({
          endReason: 'idle',
          stopped: true,
          modelError: NO_RUN_FAILURE,
          feedProblem: NO_RUN_FAILURE,
        },),).toBe('idle',);
        expect(finalEndReason({
          endReason: 'stop-file',
          stopped: true,
          modelError: NO_RUN_FAILURE,
          feedProblem: NO_RUN_FAILURE,
        },),).toBe('stop-file',);
      },
    },),
    it({
      name: 'keeps an ending that already failed, which is more specific than anything found later',
      fn: async () => {
        expect(finalEndReason({
          endReason: 'wall-clock',
          stopped: false,
          modelError: failed('no key',),
          feedProblem: failed('EISDIR',),
        },),).toBe('wall-clock',);
      },
    },),
    it({
      name: 'a Pi whose leaving could not be confirmed never ends clean',
      fn: async () => {
        expect(finalEndReason({
          endReason: 'idle',
          stopped: false,
          modelError: NO_RUN_FAILURE,
          feedProblem: NO_RUN_FAILURE,
        },),).toBe('pi-not-stopped',);
      },
    },),
    it({
      name: 'a model error nothing recovered from never ends clean',
      fn: async () => {
        expect(finalEndReason({
          endReason: 'idle',
          stopped: true,
          modelError: failed('no key',),
          feedProblem: NO_RUN_FAILURE,
        },),).toBe('model-error',);
      },
    },),
    it({
      name: 'a feed that could not be written never ends clean',
      fn: async () => {
        expect(finalEndReason({
          endReason: 'stop-file',
          stopped: true,
          modelError: NO_RUN_FAILURE,
          feedProblem: failed('EISDIR',),
        },),).toBe('driver-failed',);
      },
    },),
  ],
},);
