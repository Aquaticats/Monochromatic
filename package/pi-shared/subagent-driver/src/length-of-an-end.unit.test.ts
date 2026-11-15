/**
 What each way a run can end says about how long the run lasted: whether that length is the length of the work, and so whether a
 run that ended that way belongs in the sample a running run is judged overdue against.

 The answers live in `LENGTH_OF_AN_END`, beside the ends they answer for, so an end added without an answer does not compile. These
 controls hold the table to what the rest of the driver derives from it.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  END_REASONS,
  END_REASONS_WITHOUT_A_JOB,
  LENGTH_OF_AN_END,
  lengthOfRunCountsAsWork,
} from '../dist/final/node/index.mjs';

await describe({
  name: 'the length of a run and the end it had',
  children: [
    it({
      name: 'counts the failed recovery of a silent command among the ends whose length is no evidence of a job',
      fn: async () => {
        expect(END_REASONS_WITHOUT_A_JOB,).toContain('silent-command-unanswered',);
      },
    },),
    it({
      name: 'answers for every end the driver can have, so no end is added without saying whether its length is evidence',
      fn: async () => {
        expect(Object.keys(LENGTH_OF_AN_END,).toSorted(),).toEqual([...END_REASONS,].toSorted(),);
      },
    },),
    it({
      name: 'derives the ends without a job from those answers instead of keeping a list of its own',
      fn: async () => {
        expect(END_REASONS_WITHOUT_A_JOB,).toEqual(END_REASONS.filter(function noJob(reason,) {
          return !LENGTH_OF_AN_END[reason].countsAsWork;
        },),);
      },
    },),
    it({
      name: 'leaves out the ends the driver ended first, and counts the ones the job\'s own allowance ended',
      fn: async () => {
        expect(END_REASONS_WITHOUT_A_JOB,).toEqual([
          'prompt-rejected',
          'silent-command-unanswered',
          'compaction-failed',
          'pi-exited-before-prompt',
          'driver-failed',
          'model-error',
        ],);
      },
    },),
    it({
      name: 'states why each end answers what it does, so the table reads as decisions rather than as flags',
      fn: async () => {
        for (const reason of END_REASONS) {
          expect(LENGTH_OF_AN_END[reason].why.trim(),).not.toBe('',);
        }
      },
    },),
  ],
},);

await describe({
  name: lengthOfRunCountsAsWork.name,
  children: [
    it({
      name: 'reads an end a summary names, and keeps counting an end this driver does not know',
      fn: async () => {
        expect(lengthOfRunCountsAsWork('idle',),).toBe(true,);
        expect(lengthOfRunCountsAsWork('silent-command-unanswered',),).toBe(false,);
        // A summary from a newer driver may name an end this one has no answer for, and nothing says it was a failure.
        expect(lengthOfRunCountsAsWork('an-end-from-a-newer-driver',),).toBe(true,);
        expect(lengthOfRunCountsAsWork(undefined,),).toBe(true,);
        expect(lengthOfRunCountsAsWork(7,),).toBe(true,);
      },
    },),
  ],
},);
