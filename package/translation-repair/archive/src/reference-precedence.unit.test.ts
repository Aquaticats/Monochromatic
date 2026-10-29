/**
 Guards ledger S8: the cited-reference rules, the narrative bound and reader
 protection say which of them governs where they meet.

 A characterization or an event a cited reference states was ACCURATE on the
 reference rules and an addition on the narrative bound ("an event, an
 action ... or a characterization the ORIGINAL does not state is an addition"),
 and both sat on the same critic, panel and contest sheets. Neither rule
 limited reader protection, so a method a cited page stated plainly read as
 accurate detail to keep. The dispute note told a judge to weigh an accepted
 addition "against the ORIGINAL alone" beside a reference rule saying the
 opposite, with nothing to say the panel had already weighed the references.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { TRANSLATE_ATTESTED_RULE, } from '@monochromatic-dev/translation-repair-translate-floor';
import {
  CITED_REFERENCE_CANDIDATE_RULE,
  CITED_REFERENCE_RULE,
} from '@monochromatic-dev/translation-repair-slice-reading';
import { archiveDisputeNote, } from '../dist/final/node/index.mjs';

/**
 Wording that settles the reference rule against the narrative bound.
 */
const REFERENCE_IS_A_SOURCE = 'a cited reference is the ORIGINAL\'s own source';

/**
 Wording that keeps reader protection above the references.
 */
const PROTECTION_OUTRANKS = 'Reader protection outranks the references';

/**
 Dispute note over one accepted addition claim on a cat's walk.
 */
const additionDispute = archiveDisputeNote({
  dispute: {
    sliceIndex: 4,
    standIn: 'The cat walked to the river at dawn.',
    standInEligible: true,
    acceptedClaims: ['accuracy/addition major: The translation adds that the walk was at dawn.',],
  },
},);

await describe({
  name: 'reference precedence (ledger S8)',
  children: [
    it({
      name: 'SETTLES the critic and panel reference rule against the narrative bound',
      fn: async () => {
        expect(CITED_REFERENCE_RULE,).toContain(REFERENCE_IS_A_SOURCE,);
      },
    },),
    it({
      name: 'SETTLES the candidate reference rule against the narrative bound',
      fn: async () => {
        expect(CITED_REFERENCE_CANDIDATE_RULE,).toContain(REFERENCE_IS_A_SOURCE,);
      },
    },),
    it({
      name: 'PUTS reader protection above both reference rules, so a method a cited page states stays out',
      fn: async () => {
        expect(CITED_REFERENCE_RULE,).toContain(PROTECTION_OUTRANKS,);
        expect(CITED_REFERENCE_CANDIDATE_RULE,).toContain(PROTECTION_OUTRANKS,);
      },
    },),
    it({
      name: 'PUTS reader protection above the attested details a writer is told to carry',
      fn: async () => {
        expect(TRANSLATE_ATTESTED_RULE,).toContain('Reader protection outranks them',);
      },
    },),
    it({
      name: 'TELLS a judge the panel weighed the references before accepting an addition, so a reference does not '
        + 'reopen it',
      fn: async () => {
        expect(additionDispute,).toContain('against the ORIGINAL alone',);
        expect(additionDispute,).toContain('a cited reference does not reopen it',);
      },
    },),
  ],
},);
