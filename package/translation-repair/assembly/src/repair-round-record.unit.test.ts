/**
 Tests for the record the repair lane keeps of each judged round.

 WHY THE DECLINE CASE IS THE POINT. A round that chose nothing carries the
 same ballots a round that chose something does, and it is the shape this lane
 produces whenever a panel cannot agree. A recorder that kept only winners
 would drop exactly the rounds where the judges were divided, which is where
 the reasoning is worth reading.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure';
import {
  CHUNK_SCOPE_ENVELOPE,
  describeJudgedRound,
  describeRepairSlate,
} from '../dist/final/node/index.mjs';
import {
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 Replacement that keeps the alias the front matter declared.
 */
const KEEPS_ALIAS = 'Mittens the Cat naps on the sill.';

/**
 Same sentence with that alias taken out.
 */
const DROPS_ALIAS = 'The cat naps on the sill.';

/**
 Candidate at position one, which keeps the alias.
 */
const KEEPER = {
  producer: {
    kind: 'model' as const,
    modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  },
  value: { note: 'keeps', },
  rendered: KEEPS_ALIAS,
};

/**
 Candidate at position two, which takes it out.
 */
const DROPPER = {
  producer: {
    kind: 'model' as const,
    modelId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  },
  value: { note: 'drops', },
  rendered: DROPS_ALIAS,
};

/**
 Candidates in the order judges were shown them.
 */
const CANDIDATES = [
  KEEPER,
  DROPPER,
];

/**
 One ballot naming the shorter wording, with the reason judges actually give.
 */
const BALLOT_FOR_SHORTER = {
  modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  best: 2,
  reason: 'the alias has no basis in the original',
  weight: 1,
  selfVote: false,
};

/**
 What a round counted when one judge answered.
 */
const ONE_BALLOT_TALLY = {
  judgesAvailable: 1,
  ballots: 1,
  abstentions: 0,
  selfVotes: 0,
};

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: describeRepairSlate.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name:
            'numbers positions one-based and hashes the RENDERED text, which is what the sheet displayed '
              + 'rather than any internal value: an envelope candidate is a patch operation, and a ballot '
              + 'naming position 2 can only be joined back to words through what was actually shown',
          fn: async () => {
            expect(describeRepairSlate({ candidates: CANDIDATES, },),).toEqual([
              {
                index: 1,
                rendered: KEEPS_ALIAS,
                hash: hashContent({ content: KEEPS_ALIAS, },),
                producer: KEEPER.producer,
              },
              {
                index: 2,
                rendered: DROPS_ALIAS,
                hash: hashContent({ content: DROPS_ALIAS, },),
                producer: DROPPER.producer,
              },
            ],);
          },
        },),
      ],
    },),

    describe({
      name: describeJudgedRound.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name:
            'keeps every ballot of a round that CHOSE something, beside the position it chose and the '
              + 'weight that position drew, so a reader can tell a narrow win from a unanimous one',
          fn: async () => {
            expect(describeJudgedRound({
              stage: 'envelope',
              envelopeId: 'envelope/nap',
              candidates: CANDIDATES,
              outcome: {
                kind: 'selected',
                value: DROPPER.value,
                producer: DROPPER.producer,
                voteWeight: 1,
                selectedIndex: 2,
                tally: ONE_BALLOT_TALLY,
                findings: [],
                ballots: [BALLOT_FOR_SHORTER,],
                perCandidate: [],
              },
            },),).toEqual({
              kind: 'selected',
              stage: 'envelope',
              envelopeId: 'envelope/nap',
              slate: describeRepairSlate({ candidates: CANDIDATES, },),
              ballots: [BALLOT_FOR_SHORTER,],
              tally: ONE_BALLOT_TALLY,
              perCandidate: [],
              selectedIndex: 2,
              voteWeight: 1,
            },);
          },
        },),
        it({
          name:
            'keeps every ballot of a round that chose NOTHING, beside why and which kind of refusal it '
              + 'was. This is the half a winner-only recorder would lose, and the probe behind the '
              + 'declared-name guard produced exactly this shape on its third round',
          fn: async () => {
            /**
             Round where the one judge named the shorter wording and the panel
             still could not reach the two ballots a winner needs.
             */
            const round = describeJudgedRound({
              stage: 'refine',
              envelopeId: CHUNK_SCOPE_ENVELOPE,
              candidates: CANDIDATES,
              outcome: {
                kind: 'declined',
                reason: 'leader drew 1 of a required 2',
                disposition: 'indecision',
                tally: ONE_BALLOT_TALLY,
                findings: [],
                ballots: [BALLOT_FOR_SHORTER,],
                perCandidate: [],
              },
            },);
            expect(round,).toEqual({
              kind: 'declined',
              stage: 'refine',
              envelopeId: CHUNK_SCOPE_ENVELOPE,
              slate: describeRepairSlate({ candidates: CANDIDATES, },),
              ballots: [BALLOT_FOR_SHORTER,],
              tally: ONE_BALLOT_TALLY,
              perCandidate: [],
              reason: 'leader drew 1 of a required 2',
              disposition: 'indecision',
            },);

            // The reason a judge gave is the evidence this whole record exists for,
            // so it is asserted separately rather than only inside the whole-object
            // `toEqual`: a later change that summarised reasons instead of
            // keeping them would still satisfy a shape check.
            expect(round.ballots.at(0,)?.reason,).toBe('the alias has no basis in the original',);
          },
        },),
        it({
          name:
            'names the whole chunk rather than an envelope when the round decided the whole chunk, using '
              + 'one sentinel both chunk-scope stages share, so a reader never has to tell an unstamped '
              + 'envelope from a round that had none',
          fn: async () => {
            expect(describeJudgedRound({
              stage: 'chunk-patch',
              envelopeId: CHUNK_SCOPE_ENVELOPE,
              candidates: CANDIDATES,
              outcome: {
                kind: 'declined',
                reason: 'every judge declined',
                disposition: 'rejection',
                tally: ONE_BALLOT_TALLY,
                findings: [],
                ballots: [],
                perCandidate: [],
              },
            },).envelopeId,).toBe('chunk',);
          },
        },),
      ],
    },),
  ],
},);
