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
  endReasonOfPromptWait,
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
          piExit: { code: 0, },
        },),).toBe('idle',);
        expect(finalEndReason({
          endReason: 'stop-file',
          stopped: true,
          modelError: NO_RUN_FAILURE,
          feedProblem: NO_RUN_FAILURE,
          piExit: { code: 0, },
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
          piExit: { code: 1, },
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
          piExit: {},
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
          piExit: { code: 0, },
        },),).toBe('model-error',);
      },
    },),
    it({
      name: 'a Pi that failed after a run ended on its own terms never leaves that run clean',
      fn: async () => {
        expect(finalEndReason({
          endReason: 'idle',
          stopped: true,
          modelError: NO_RUN_FAILURE,
          feedProblem: NO_RUN_FAILURE,
          piExit: { code: 3, },
        },),).toBe('pi-exited',);
      },
    },),
    it({
      name: 'a Pi that a signal ended leaves the run its own reason, because that is how it was asked to go',
      fn: async () => {
        expect(finalEndReason({
          endReason: 'stop-file',
          stopped: true,
          modelError: NO_RUN_FAILURE,
          feedProblem: NO_RUN_FAILURE,
          piExit: { signal: 'SIGTERM', },
        },),).toBe('stop-file',);
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
          piExit: { code: 0, },
        },),).toBe('driver-failed',);
      },
    },),
  ],
},);

await describe({
  name: endReasonOfPromptWait.name,
  children: [
    it({
      name: 'a wait the run\'s deadline ended is the wall clock, whatever Pi was doing',
      fn: async () => {
        expect(endReasonOfPromptWait({
          answer: {
            ending: 'deadline',
            problem: 'no answer before the deadline',
            compactionProblem: 'Pi was still compacting (threshold) after 400 ms',
          },
          passedDeadline: true,
        },),).toBe('wall-clock',);
      },
    },),
    it({
      name: 'a prompt Pi would not take while nothing failed is a rejected prompt',
      fn: async () => {
        expect(endReasonOfPromptWait({
          answer: {
            ending: 'silence',
            problem: 'no answer in 30000 ms',
          },
          passedDeadline: false,
        },),).toBe('prompt-rejected',);
        expect(endReasonOfPromptWait({
          answer: {
            ending: 'answered',
            record: { success: false, },
          },
          passedDeadline: false,
        },),).toBe('prompt-rejected',);
      },
    },),
    it({
      name: 'silence after a compaction Pi could not finish names the compaction, not a prompt Pi refused',
      fn: async () => {
        expect(endReasonOfPromptWait({
          answer: {
            ending: 'silence',
            problem: 'no answer in 30000 ms',
            compactionProblem: 'Auto-compaction failed: the request is larger than the model window',
          },
          passedDeadline: false,
        },),).toBe('compaction-failed',);
      },
    },),
    it({
      name: 'a wait Pi left is a Pi that exited, and a refusal the deadline had already reached is the wall clock',
      fn: async () => {
        expect(endReasonOfPromptWait({
          answer: {
            ending: 'pi-exited',
            problem: 'pi exited',
          },
          passedDeadline: false,
        },),).toBe('pi-exited',);
        expect(endReasonOfPromptWait({
          answer: {
            ending: 'answered',
            record: { success: false, },
          },
          passedDeadline: true,
        },),).toBe('wall-clock',);
      },
    },),
  ],
},);
