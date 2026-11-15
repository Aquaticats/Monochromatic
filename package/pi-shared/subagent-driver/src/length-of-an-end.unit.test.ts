/**
 What each way a run can end says about how long the run lasted: whether that length is the length of the work, and so whether a
 run that ended that way belongs in the sample a running run is judged overdue against.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  END_REASONS_WITHOUT_A_JOB,
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
  ],
},);
