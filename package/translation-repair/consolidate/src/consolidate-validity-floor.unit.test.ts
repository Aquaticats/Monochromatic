/**
 Tests for the floor under a consolidation slate.

 WHAT THIS PINS is the case the band pair actually hit: every proposal
 refused by the structural guard, and a consolidation shipping anyway. The
 floor is what makes that impossible, so the case where nothing survives
 matters more here than the case where something does.

 The policy is inherited rather than invented, so these also pin the half
 that is easy to lose in a refactor: a slate with even one survivor is NOT
 refused, however many of its siblings failed. A floor that tripped on any
 invalid proposal would throw away the ensemble.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  floorConsolidateSlate,
  type ProposalValidity,
} from '../dist/final/node/index.mjs';
import { capturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';

/**
 Logger the cases that do not read the log hand to the stage.
 */
const l = tagged({ tag: 'consolidate-floor-test', },);

/**
 Builds one checked proposal.

 @param modelId - voice that wrote it

 @param valid - whether the structural guard passed it

 @returns Proposal shaped as the produce half reports one

 @example
 ```ts
 const checked = checkedAs({ modelId: 'hf:cat/Cat-A', valid: true, },);
 ```
 */
function checkedAs(
  { modelId, valid, }: { readonly modelId: string; readonly valid: boolean; },
): ProposalValidity {
  return {
    modelId,
    validation: valid
      ? {
        kind: 'valid',
        pageGrammar: 'strict',
      }
      : {
        kind: 'invalid',
        findings: ['The page as it stands is 2 blocks and your translation is 1.',],
        unknownDetail: '',
      },
  } as ProposalValidity;
}

await describe({
  name: floorConsolidateSlate.name,
  children: [
    it({
      name: 'REFUSES A SLATE WHERE THE GUARD REJECTED EVERY PROPOSAL, which is the case the band pair '
        + 'hit twice: Zha_Ke#1 finished its repair round with five candidates and zero valid ones, in '
        + 'both runs, and shipped a consolidation at both. The guard had already said all five were '
        + 'structurally not the page they would be written into',
      fn: async () => {
        const floor = floorConsolidateSlate({
          validity: [
            checkedAs({ modelId: 'hf:cat/Cat-A', valid: false, },),
            checkedAs({ modelId: 'hf:cat/Cat-B', valid: false, },),
            checkedAs({ modelId: 'hf:cat/Cat-C', valid: false, },),
          ],
          l,
        },);

        expect(floor.kind,).toBe('incumbent-only',);
        if (floor.kind !== 'incumbent-only')
          throw new Error('incumbent-only by construction',);
        expect(floor.refusedModelIds,).toStrictEqual(['hf:cat/Cat-A', 'hf:cat/Cat-B', 'hf:cat/Cat-C',],);
      },
    },),

    it({
      name: 'KEEPS A SLATE WITH ONE SURVIVOR among any number of refusals, because a floor that '
        + 'tripped on any invalid proposal would throw away the ensemble on the ordinary case: run 8 '
        + 'carried 7 invalid candidates across slices that all shipped normally',
      fn: async () => {
        const floor = floorConsolidateSlate({
          validity: [
            checkedAs({ modelId: 'hf:cat/Cat-A', valid: false, },),
            checkedAs({ modelId: 'hf:cat/Cat-B', valid: true, },),
            checkedAs({ modelId: 'hf:cat/Cat-C', valid: false, },),
          ],
          l,
        },);

        expect(floor.kind,).toBe('proposals',);
        if (floor.kind !== 'proposals')
          throw new Error('proposals by construction',);
        expect(floor.validModelIds,).toStrictEqual(['hf:cat/Cat-B',],);
      },
    },),

    it({
      name: 'NAMES ONLY THE SURVIVORS and in the order they were given, so a caller building the '
        + 'slate from this does not have to re-derive which voices it may carry',
      fn: async () => {
        const floor = floorConsolidateSlate({
          validity: [
            checkedAs({ modelId: 'hf:cat/Cat-A', valid: true, },),
            checkedAs({ modelId: 'hf:cat/Cat-B', valid: false, },),
            checkedAs({ modelId: 'hf:cat/Cat-C', valid: true, },),
          ],
          l,
        },);

        if (floor.kind !== 'proposals')
          throw new Error('proposals by construction',);
        expect(floor.validModelIds,).toStrictEqual(['hf:cat/Cat-A', 'hf:cat/Cat-C',],);
      },
    },),

    it({
      name: 'SAYS IN THE RUN LOG WHICH PROPOSALS IT WITHHELD when some survive (ledger E5): only the '
        + 'all-refused case was logged, so a slate that lost voices here read as a judged slate of '
        + 'fewer voices with nothing naming the guard that removed them',
      fn: async () => {
        /**
         Lines the floor wrote.
         */
        const said: string[] = [];
        floorConsolidateSlate({
          validity: [
            checkedAs({ modelId: 'hf:cat/Cat-A', valid: true, },),
            checkedAs({ modelId: 'hf:cat/Cat-B', valid: false, },),
            checkedAs({ modelId: 'hf:cat/Cat-C', valid: false, },),
          ],
          l: capturingLogger({ messages: said, },),
        },);

        expect(said,).toHaveLength(1,);
        expect(said[0],).toContain('hf:cat/Cat-B, hf:cat/Cat-C',);
        expect(said[0],).not.toContain('hf:cat/Cat-A',);
      },
    },),

    it({
      name: 'SAYS NOTHING when every proposal passed, so a withheld line marks a thinned slate only',
      fn: async () => {
        /**
         Lines the floor wrote.
         */
        const said: string[] = [];
        floorConsolidateSlate({
          validity: [checkedAs({ modelId: 'hf:cat/Cat-A', valid: true, },),],
          l: capturingLogger({ messages: said, },),
        },);

        expect(said,).toStrictEqual([],);
      },
    },),

    it({
      name: 'READS AN EMPTY ROSTER AS INCUMBENT-ONLY WITH NOBODY REFUSED, rather than as an error. A '
        + 'stage that bought no voices and one whose every voice was refused leave the caller the '
        + 'same single option, and the refused list is what tells the two apart afterwards',
      fn: async () => {
        const floor = floorConsolidateSlate({ validity: [], l, },);

        expect(floor.kind,).toBe('incumbent-only',);
        if (floor.kind !== 'incumbent-only')
          throw new Error('incumbent-only by construction',);
        expect(floor.refusedModelIds,).toStrictEqual([],);
      },
    },),
  ],
},);
