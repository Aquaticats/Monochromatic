/**
 Tests for the non-translation vote floor once it is the two answering judges a decision needs and no more.

 THE OWNER'S RULINGS OF 2026-10-25, quoted where the rule is stated
 (`doc/design-commitments-votes-repairs-and-run-offs.md`): "floor of 3 - don't set a floor.", "Two ballots decide
 (Recommended)" and "Every weight floor". A claim or contest is decided with two or more answering judges, and one
 judge alone never decides. The block used to need three wire votes, so two critics who both called a slice a
 non-translation could not stop it (XingZ6021 slices 68 and 69, which drew two such votes). These cases are written with
 the literal counts, not the exported constant, so they fail against a floor of three.

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  assessNonTranslationEvidence,
  nonTranslationVotesStand,
  screenNonTranslationVotes,
} from '../dist/final/node/index.mjs';

await describe({
  name: 'non-translation votes at two answering judges',
  children: [
    it({
      name: 'STANDS at two votes where deterministic evidence has not contradicted them, so two critics block a slice',
      fn: async () => {
        expect(nonTranslationVotesStand({ votes: 2, contradicted: false, },),).toBe(true,);
      },
    },),

    it({
      name: 'DOES NOT STAND at one vote, the positive control: one judge alone never decides',
      fn: async () => {
        expect(nonTranslationVotesStand({ votes: 1, contradicted: false, },),).toBe(false,);
        expect(nonTranslationVotesStand({ votes: 0, contradicted: false, },),).toBe(false,);
      },
    },),

    it({
      name: 'DOES NOT STAND at two votes once the evidence contradicted them',
      fn: async () => {
        expect(nonTranslationVotesStand({ votes: 2, contradicted: true, },),).toBe(false,);
      },
    },),

    it({
      name: 'LEAVES two votes standing when too little content critique anchors the target to contradict them, and '
        + 'contradicts them with eight such claims',
      fn: async () => {
        /**
         One content-critique claim anchored in the target, repeated to the count asked.
         */
        const claims = (count: number,) =>
          Array.from({ length: count, },).map(function toClaim(_entry, index,) {
            return {
              category: 'accuracy/mistranslation' as const,
              severity: 'major' as const,
              summary: `Purring nuance ${String(index,)} is rendered as hissing.`,
              spans: [
                {
                  side: 'target' as const,
                  nodeId: 'block/0',
                  nodeHash: 'hash/whisker',
                  startOffset: 0,
                  endOffset: 4,
                  quotedText: 'purr',
                },
              ],
            };
          },);
        expect(assessNonTranslationEvidence({ votes: 2, claims: claims(7,), },).contradicted,).toBe(false,);
        expect(assessNonTranslationEvidence({ votes: 2, claims: claims(8,), },).contradicted,).toBe(true,);
        expect(assessNonTranslationEvidence({ votes: 1, claims: claims(8,), },).contradicted,).toBe(false,);
        expect(screenNonTranslationVotes({ votes: 2, claims: claims(8,), },).contradicted,).toBe(true,);
      },
    },),
  ],
},);
