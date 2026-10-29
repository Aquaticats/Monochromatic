/**
 Tests for the paired self-preference measurement.

 WHAT THESE PIN is that the instrument can move in BOTH directions and that
 the pairing is real. A measurement that only ever returns zero would agree
 with the half-weight discount correcting nothing, and would agree just as
 readily if the discount were correcting a great deal; the positive control
 here is what separates those.

 Model ids are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import type {
  CandidateProducer,
  SelectionBallot,
} from '@monochromatic-dev/translation-repair-house-rule';
import {
  type SelectionRound,
  selfPreference,
} from '../dist/final/node/index.mjs';

/**
 Models that both write and judge.
 */
const CAT_A = 'hf:cat/Cat-A' as unknown as RosterModelId;

/**
 Second producer-judge.
 */
const CAT_B = 'hf:cat/Cat-B' as unknown as RosterModelId;

/**
 Judge that writes nothing, so its ballots are always disinterested.
 */
const CAT_C = 'hf:cat/Cat-C' as unknown as RosterModelId;

/**
 Builds one ballot.

 @param modelId - judge casting it

 @param best - one-based candidate it names

 @param selfVote - whether that candidate is one this judge helped write.
 Set explicitly rather than derived, matching what selection records: the
 field is a fact about the ballot, kept so a self-preference measurement never
 has to infer it from the weights, which are tuning knobs

 @returns Ballot at full weight, since weight is not what this file measures

 @example
 ```ts
 const ballot = ballotFor({ modelId: CAT_A, best: 1, selfVote: true, },);
 ```
 */
function ballotFor(
  {
    modelId,
    best,
    selfVote = false,
  }: {
    readonly modelId: RosterModelId;
    readonly best: number;
    readonly selfVote?: boolean;
  },
): SelectionBallot {
  return {
    modelId,
    best,
    reason: 'fixture',
    weight: 1,
    selfVote,
  };
}

/**
 Slate of two candidates, one by each producer.
 */
const TWO_PRODUCERS: readonly CandidateProducer[] = [
  {
    kind: 'model',
    modelId: CAT_A,
  },
  {
    kind: 'model',
    modelId: CAT_B,
  },
];

await describe({
  name: selfPreference.name,
  children: [
    it({
      name: 'POSITIVE CONTROL: reports a LARGE excess when every producer names its own candidate '
        + 'and no disinterested judge agrees, which is the shape the half-weight discount exists '
        + 'to correct. Without this case a zero could mean either no favouritism or a broken probe',
      fn: async () => {
        /**
         Both producers back themselves; the neutral judge backs neither.
         */
        const rounds: readonly SelectionRound[] = [{
          producers: TWO_PRODUCERS,
          ballots: [
            ballotFor({ modelId: CAT_A, best: 1, selfVote: true, },),
            ballotFor({ modelId: CAT_B, best: 2, selfVote: true, },),
            ballotFor({ modelId: CAT_C, best: 0, },),
          ],
        },];

        const measured = selfPreference({ rounds, },);
        expect(measured.kind,).toBe('measured',);
        if (measured.kind !== 'measured')
          return;
        expect(measured.ownRate,).toBe(1,);
        expect(measured.disinterestedRate,).toBe(0,);
        expect(measured.excess,).toBe(1,);

        // CROSS-CHECK against the flag selection records on each ballot. The
        // count here is derived from the SLATE, by asking which models hold a
        // stake in the named candidate; `selfVote` is recorded by selection at
        // the time the ballot is read. Two independent derivations of one fact,
        // and a disagreement means one of them is wrong.
        expect(measured.ownVotes,).toBe(rounds
          .flatMap(function toBallots(round,) {
            return round.ballots;
          },)
          .filter(function wasSelfVote(ballot,) {
            return ballot.selfVote;
          },)
          .length,);
      },
    },),
    it({
      name: 'reports ZERO excess when a producer backs its own candidate and every disinterested '
        + 'judge backs it too, since agreeing with everyone about a text is not preferring it for '
        + 'being yours: this is the case a bare self-vote COUNT would misread as favouritism',
      fn: async () => {
        /**
         One candidate everyone names, produced by Cat-A.
         */
        const rounds: readonly SelectionRound[] = [{
          producers: [{
            kind: 'model',
            modelId: CAT_A,
          },],
          ballots: [
            ballotFor({ modelId: CAT_A, best: 1, },),
            ballotFor({ modelId: CAT_B, best: 1, },),
            ballotFor({ modelId: CAT_C, best: 1, },),
          ],
        },];

        const measured = selfPreference({ rounds, },);
        expect(measured.kind,).toBe('measured',);
        if (measured.kind !== 'measured')
          return;
        expect(measured.ownVotes,).toBe(1,);
        expect(measured.excess,).toBe(0,);
      },
    },),
    it({
      name: 'reports a NEGATIVE excess when a producer declines its own work that others back, '
        + 'so the instrument is not one-sided: a roster harder on itself than on its neighbours '
        + 'would be evidence AGAINST discounting, and has to be readable',
      fn: async () => {
        const rounds: readonly SelectionRound[] = [{
          producers: [{
            kind: 'model',
            modelId: CAT_A,
          },],
          ballots: [
            ballotFor({ modelId: CAT_A, best: 0, },),
            ballotFor({ modelId: CAT_B, best: 1, },),
            ballotFor({ modelId: CAT_C, best: 1, },),
          ],
        },];

        const measured = selfPreference({ rounds, },);
        expect(measured.kind,).toBe('measured',);
        if (measured.kind !== 'measured')
          return;
        expect(measured.excess,).toBe(-1,);
      },
    },),
    it({
      name: 'counts every model collapsed into a COMPOSITE as a stakeholder, so a candidate two '
        + 'models assembled is not read as disinterested for either of them',
      fn: async () => {
        const rounds: readonly SelectionRound[] = [{
          producers: [{
            kind: 'composite',
            contributors: [CAT_A,
              CAT_B,],
          },],
          ballots: [
            ballotFor({ modelId: CAT_A, best: 1, },),
            ballotFor({ modelId: CAT_B, best: 1, },),
            ballotFor({ modelId: CAT_C, best: 0, },),
          ],
        },];

        const measured = selfPreference({ rounds, },);
        expect(measured.kind,).toBe('measured',);
        if (measured.kind !== 'measured')
          return;
        expect(measured.opportunities,).toBe(2,);
        expect(measured.excess,).toBe(1,);
      },
    },),
    it({
      name: 'treats a model COLLAPSED INTO THE INCUMBENT as a stakeholder in it, which is what '
        + '`matched` records: a model that independently wrote the archive\'s wording has a stake '
        + 'in that candidate winning even though it did not write the archive',
      fn: async () => {
        const rounds: readonly SelectionRound[] = [{
          producers: [{
            kind: 'incumbent',
            matched: [CAT_A,],
          },],
          ballots: [
            ballotFor({ modelId: CAT_A, best: 1, },),
            ballotFor({ modelId: CAT_C, best: 0, },),
          ],
        },];

        const measured = selfPreference({ rounds, },);
        expect(measured.kind,).toBe('measured',);
        if (measured.kind !== 'measured')
          return;
        expect(measured.opportunities,).toBe(1,);
        expect(measured.ownVotes,).toBe(1,);
      },
    },),
    it({
      name: 'EXCLUDES candidates nobody had a stake in from BOTH sides, so the baseline is taken '
        + 'over the same texts the own rate is: counting a bystander candidate\'s ballots into the '
        + 'baseline would dilute the comparison with texts the question is not about',
      fn: async () => {
        /**
         Cat-A's candidate, plus an unowned one nobody produced.
         */
        const rounds: readonly SelectionRound[] = [{
          producers: [
            {
              kind: 'model',
              modelId: CAT_A,
            },
            {
              kind: 'incumbent',
              matched: [],
            },
          ],
          ballots: [
            ballotFor({ modelId: CAT_A, best: 1, },),
            ballotFor({ modelId: CAT_C, best: 2, },),
          ],
        },];

        const measured = selfPreference({ rounds, },);
        expect(measured.kind,).toBe('measured',);
        if (measured.kind !== 'measured')
          return;
        // Only candidate 1 contributes. Cat-C cast one ballot over it and named
        // something else, so the baseline is 0 of 1 rather than 1 of 2.
        expect(measured.otherBallots,).toBe(1,);
        expect(measured.otherVotes,).toBe(0,);
      },
    },),
    it({
      name: 'reports `no-stakeholder-ballots` rather than a rate when no producer voted at all, '
        + 'since the question was never put and a zero would say it was put and answered',
      fn: async () => {
        const rounds: readonly SelectionRound[] = [{
          producers: [{
            kind: 'model',
            modelId: CAT_A,
          },],
          ballots: [ballotFor({ modelId: CAT_C, best: 1, },),],
        },];

        expect(selfPreference({ rounds, },).kind,).toBe('no-stakeholder-ballots',);
      },
    },),
    it({
      name: 'reports `no-disinterested-ballots` when every judge held a stake, which is a roster '
        + 'shape rather than missing data and is exactly what a roster of all producers would hit',
      fn: async () => {
        const rounds: readonly SelectionRound[] = [{
          producers: [{
            kind: 'composite',
            contributors: [CAT_A,
              CAT_B,],
          },],
          ballots: [
            ballotFor({ modelId: CAT_A, best: 1, },),
            ballotFor({ modelId: CAT_B, best: 1, },),
          ],
        },];

        expect(selfPreference({ rounds, },).kind,).toBe('no-disinterested-ballots',);
      },
    },),
    it({
      name: 'pools ACROSS ROUNDS rather than averaging per-round rates, so a slice with one '
        + 'stakeholder ballot does not weigh as much as a slice with six',
      fn: async () => {
        /**
         One round where the producer backs itself, one where it does not.
         */
        const rounds: readonly SelectionRound[] = [
          {
            producers: [{
              kind: 'model',
              modelId: CAT_A,
            },],
            ballots: [
              ballotFor({ modelId: CAT_A, best: 1, },),
              ballotFor({ modelId: CAT_C, best: 0, },),
            ],
          },
          {
            producers: [{
              kind: 'composite',
              contributors: [CAT_A,
                CAT_B,],
            },],
            ballots: [
              ballotFor({ modelId: CAT_A, best: 0, },),
              ballotFor({ modelId: CAT_B, best: 0, },),
              ballotFor({ modelId: CAT_C, best: 0, },),
            ],
          },
        ];

        const measured = selfPreference({ rounds, },);
        expect(measured.kind,).toBe('measured',);
        if (measured.kind !== 'measured')
          return;
        // Three stakeholder ballots pooled, one of them a self-vote.
        expect(measured.opportunities,).toBe(3,);
        expect(measured.ownVotes,).toBe(1,);
      },
    },),
  ],
},);
