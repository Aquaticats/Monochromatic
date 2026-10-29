/**
 Tests for what the repair lane says about a passage the archive never
 translated.

 The outcome is read by position and its finding is what says why, so both
 are pinned: every count zero, every list empty, nothing changed, and the
 finding in its scorecard-stable wording.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  notApplicableFinding,
  notApplicableRepair,
} from '../dist/final/node/index.mjs';

await describe({
  name: notApplicableRepair.name,
  children: [
    it({
      name: 'REFUSES to count an anchor toward the non-translation block, which would block documents for '
        + 'the gaps the translate lane exists to fill',
      fn: async () => {
        const outcome = notApplicableRepair({ sliceIndex: 4, },);

        expect(outcome.nonTranslationStanding,).toBe(false,);
        expect(outcome.nonTranslationContradicted,).toBe(false,);
        expect(outcome.nonTranslationVotes,).toBe(0,);
      },
    },),

    it({
      name: 'states the archive\'s own absence: nothing changed, nobody heard, nothing filed, no author',
      fn: async () => {
        const outcome = notApplicableRepair({ sliceIndex: 4, },);

        expect(outcome.sliceIndex,).toBe(4,);
        expect(outcome.repairedText,).toBe('',);
        expect(outcome.changed,).toBe(false,);
        expect(outcome.heardCritics,).toBe(0,);
        expect(outcome.heardCriticIds,).toEqual([],);
        expect(outcome.issues,).toEqual([],);
        expect(outcome.resolvedIssueIds,).toEqual([],);
        expect(outcome.refined,).toBe(false,);
        expect(outcome.authorship.everyIssue,).toEqual([],);
      },
    },),

    it({
      name: 'carries exactly the not-applicable finding, in the wording the scorecard reads',
      fn: async () => {
        const outcome = notApplicableRepair({ sliceIndex: 4, },);

        expect(outcome.findings,).toEqual([notApplicableFinding({ sliceIndex: 4, },),],);
        expect(notApplicableFinding({ sliceIndex: 4, },),).toBe(
          'repair-not-applicable chunk 4; no translation to repair',
        );
      },
    },),
  ],
},);
