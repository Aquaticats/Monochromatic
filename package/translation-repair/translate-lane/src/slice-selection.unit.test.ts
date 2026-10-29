/**
 Tests for the per-slice selection ledger.

 WHAT THESE PIN is the distinction the ledger exists for: "the judges kept the
 archive" and "the archive was reinstated after the judges replaced it" are
 different facts, and every count the lane reported before this collapsed them.
 Recording who won each slice gives the first; the assembly guard is what makes the second
 possible. A reader holding one number cannot tell them apart, and the
 replacement rate every quality claim rests on is computed from exactly this.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import { selfPreference, } from '@monochromatic-dev/translation-repair-translate-floor';
import {
  buildSliceSelections,
  type TranslateSliceRecord,
} from '../dist/final/node/index.mjs';

/**
 Model standing in for a translator.
 */
const CAT_A = 'hf:cat/Cat-A' as unknown as RosterModelId;

/**
 Builds one settled record with the parts this ledger reads.

 Everything the ledger ignores is filled with whatever satisfies the type:
 what is under test is which fields travel, not what the stage decided.

 @param sliceIndex - slice position

 @param origin - whether the winner was the archive's text or fresh

 @param decision - how the round ended

 @param voteWeight - weight the winner drew

 @returns Record shaped like one the driver settles

 @example
 ```ts
 const record = recordFor({ sliceIndex: 0, origin: 'fresh', decision: 'judged', voteWeight: 2, },);
 ```
 */
function recordFor(
  {
    sliceIndex,
    origin,
    decision,
    voteWeight,
  }: {
    readonly sliceIndex: number;
    readonly origin: string;
    readonly decision: string;
    readonly voteWeight: number;
  },
): TranslateSliceRecord {
  return {
    kind: 'translate-slice',
    schemaVersion: 1,
    sliceIndex,
    outputText: 'The cat sleeps on the windowsill.\n',
    changed: origin === 'fresh',
    disposition: 'stage-result',
    findings: [],
    stageResult: {
      text: 'The cat sleeps on the windowsill.\n',
      origin,
      producer: {
        kind: 'model',
        modelId: CAT_A,
      },
      decision,
      voteWeight,
      tally: {
        judgesAvailable: 3,
        ballots: 3,
        abstentions: 0,
        selfVotes: 0,
      },
      ballots: [],
      heardTranslators: 2,
      candidateCount: 2,
      slate: [],
      selectedIndex: 1,
      shippedIndex: 1,
      perCandidate: [],
      findings: [],
    },
  } as unknown as TranslateSliceRecord;
}

/**
 Second model, so a round can have a stakeholder and a disinterested judge.
 */
const CAT_B = 'hf:cat/Cat-B' as unknown as RosterModelId;

/**
 Builds a record carrying a REAL round: two candidates by two producers, and
 two ballots, one of which is a self-vote.

 Cat-A wrote candidate 1 and named candidate 2; Cat-B wrote candidate 2 and
 named it. So over these two candidates there is one self-vote out of two
 stakeholder ballots, which is what the measurement case reads.

 @param sliceIndex - slice position

 @returns Record whose round is worth measuring

 @example
 ```ts
 const record = recordWithRound({ sliceIndex: 0, },);
 ```
 */
function recordWithRound(
  { sliceIndex, }: { readonly sliceIndex: number; },
): TranslateSliceRecord {
  /**
   Base record, whose round is then replaced with a populated one.
   */
  const base = recordFor({
    sliceIndex,
    origin: 'fresh',
    decision: 'judged',
    voteWeight: 2,
  },);

  return {
    ...base,
    stageResult: {
      ...base.stageResult,
      slate: [
        {
          origin: 'fresh',
          producer: {
            kind: 'model',
            modelId: CAT_A,
          },
        },
        {
          origin: 'fresh',
          producer: {
            kind: 'model',
            modelId: CAT_B,
          },
        },
      ],
      ballots: [
        {
          modelId: CAT_A,
          best: 2,
          reason: 'the second reads more naturally',
          weight: 1,
          selfVote: false,
        },
        {
          modelId: CAT_B,
          best: 2,
          reason: 'mine keeps the tail clause',
          weight: 0.5,
          selfVote: true,
        },
      ],
    },
  } as unknown as TranslateSliceRecord;
}

await describe({
  name: buildSliceSelections.name,
  children: [
    it({
      name: 'SEPARATES an archive kept by the judges from one reinstated after they replaced it, '
        + 'which is the distinction every count before this collapsed. Both ship the archive text; '
        + 'only `origin` says whether that was chosen or restored',
      fn: async () => {
        /**
         Slice 0: judges kept the archive. Slice 1: judges chose fresh text and
         the assembly guard withdrew it.
         */
        const selections = buildSliceSelections({
          records: [
            recordFor({
              sliceIndex: 0,
              origin: 'incumbent',
              decision: 'judged',
              voteWeight: 2,
            },),
            recordFor({
              sliceIndex: 1,
              origin: 'fresh',
              decision: 'judged',
              voteWeight: 3,
            },),
          ],
          // Neither slice shipped: slice 0 had nothing to ship, slice 1 was
          // taken back.
          changedSliceIndices: [],
        },);
        expect(selections.length,).toBe(2,);
        expect(selections[0]?.origin,).toBe('incumbent',);
        expect(selections[0]?.shipped,).toBe(false,);
        expect(selections[1]?.origin,).toBe('fresh',);
        expect(selections[1]?.shipped,).toBe(false,);
        // The pair reads differently despite both carrying the archive's text,
        // which is the whole claim.
        expect(selections[0]?.origin === selections[1]?.origin,).toBe(false,);
      },
    },),
    it({
      name: 'marks a slice SHIPPED when the document carries it, taken from the assembled bytes '
        + 'rather than re-derived from the record: a record says what the slice chose and the '
        + 'document says what it carries, and those disagree exactly where the guard intervened',
      fn: async () => {
        const selections = buildSliceSelections({
          records: [
            recordFor({
              sliceIndex: 4,
              origin: 'fresh',
              decision: 'judged',
              voteWeight: 3,
            },),
            recordFor({
              sliceIndex: 7,
              origin: 'fresh',
              decision: 'judged',
              voteWeight: 3,
            },),
          ],
          changedSliceIndices: [7,],
        },);
        expect(selections[0]?.shipped,).toBe(false,);
        expect(selections[1]?.shipped,).toBe(true,);
      },
    },),
    it({
      name: 'carries the PRODUCER through, since who wrote the winning text is what a self-'
        + 'preference or per-model rate is computed over and no count preserves it',
      fn: async () => {
        const selections = buildSliceSelections({
          records: [recordFor({
            sliceIndex: 0,
            origin: 'fresh',
            decision: 'judged',
            voteWeight: 2,
          },),],
          changedSliceIndices: [0,],
        },);
        expect(selections[0]?.producer
          .kind,).toBe('model',);
      },
    },),
    it({
      name: 'keeps the DECLINE vocabulary rather than flattening it to a boolean, so a slice the '
        + 'judges could not agree on stays distinguishable from one they agreed to keep',
      fn: async () => {
        const selections = buildSliceSelections({
          records: [
            recordFor({
              sliceIndex: 0,
              origin: 'incumbent',
              decision: 'declined-indecision',
              voteWeight: 0,
            },),
            recordFor({
              sliceIndex: 1,
              origin: 'incumbent',
              decision: 'judged',
              voteWeight: 2,
            },),
          ],
          changedSliceIndices: [],
        },);
        expect(selections[0]?.decision,).toBe('declined-indecision',);
        expect(selections[1]?.decision,).toBe('judged',);
      },
    },),
    it({
      name: 'CARRIES THE WHOLE ROUND, ballots and reasons included, in slate order. The summary '
        + 'says what shipped; only the round says who was asked and what they answered, and the '
        + 'reasons are what found the Kimi-K3 channel marker and the Dethelly relocation',
      fn: async () => {
        const selections = buildSliceSelections({
          records: [recordWithRound({ sliceIndex: 0, },),],
          changedSliceIndices: [0,],
        },);

        /**
         Round as the artifact would carry it.
         */
        const round = selections[0]?.round;
        expect(round?.producers
          .length,).toBe(2,);
        expect(round?.ballots
          .length,).toBe(2,);
        // The judge's stated reason survives, which a tally cannot reconstruct.
        expect(round?.ballots[0]?.reason,).toBe('the second reads more naturally',);
      },
    },),
    it({
      name: 'produces a round `selfPreference` consumes AS IS, which is why it is this shape: the '
        + 'artifact carries what the instrument reads rather than something a reader reshapes, and '
        + 'a reshaping step is where a measurement quietly starts answering a different question',
      fn: async () => {
        const selections = buildSliceSelections({
          records: [recordWithRound({ sliceIndex: 0, },),],
          changedSliceIndices: [0,],
        },);

        /**
         The measurement run straight off the ledger.
         */
        const measured = selfPreference({ rounds: selections.map(function toRound(selection,) {
          return selection.round;
        },), },);
        expect(measured.kind,).toBe('measured',);
        if (measured.kind !== 'measured')
          return;
        // Cat-A produced candidate 1 and named candidate 2; Cat-B produced
        // candidate 2 and named it. One self-vote of two stakeholder ballots.
        expect(measured.opportunities,).toBe(2,);
        expect(measured.ownVotes,).toBe(1,);
      },
    },),
    it({
      name: 'reports one entry per record in the order they arrived, so a reader joining this to '
        + 'the other lane slice by slice does not have to sort first',
      fn: async () => {
        const selections = buildSliceSelections({
          records: [3,
            9,
            11,].map(function toRecord(sliceIndex,) {
              return recordFor({
                sliceIndex,
                origin: 'fresh',
                decision: 'judged',
                voteWeight: 2,
              },);
            },),
          changedSliceIndices: [9,],
        },);
        expect(selections.map(function toIndex(selection,) {
          return selection.sliceIndex;
        },),).toEqual([3,
          9,
          11,],);
      },
    },),
  ],
},);
