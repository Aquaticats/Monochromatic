/**
 Tests for reading the judged-contest ledger back.

 THE JOIN IS WHAT THESE CHECK. A ballot names a POSITION and the summary
 speaks about MODELS, so every case here is built so a reader that lost the
 `candidates[best - 1]` step would answer differently rather than merely
 answer late.

 THE COMPOSITE CANDIDATE IS DELIBERATE. One candidate is written by two
 models at once, so a ballot for it credits both, and a ballot cast BY either
 of them is a self-vote for both. A reader crediting only the first author
 would pass every single-author case and fail here.

 THE TWO BALLOT FAULTS ARE SEPARATED ON PURPOSE. One judge names nothing and
 one names a fourth candidate on a three-candidate slate. Both are recorded
 rather than corrected, and a reader folding them together would report one
 count of two where the contest had one of each.

 THE DISINTERESTED DENOMINATOR IS CHECKED, not just the numerator. A seat
 every judge could weigh and a seat most judges wrote are different, and only
 the denominator tells them apart.

 Model identifiers come from the catalog. Passages are cat-themed invention,
 so no corpus content appears here.

 @module
 */

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  LedgerShapeError,
  parseLedgerRound,
  summariseLedger,
  type LedgerSummary,
  type ModelWork,
  type ReadBallot,
  type ReadRound,
  workOfModel,
} from '../../dist/final/node/index.mjs';
import {
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

//region Ledger read tests

/**
 Index a ballot carries when its judge named nothing.
 */
const ABSTAINED = 0;

/**
 Seat writing on its own in every fixture.
 */
const SOLO = SEAT_SYNTHETIC_VISION_WITHHELD;

/**
 First author of the jointly written candidate.
 */
const JOINT_ONE = SEAT_SYNTHETIC_TEXT_EVERYWHERE;

/**
 Second author of the jointly written candidate.
 */
const JOINT_TWO = SEAT_SYNTHETIC_VISION_NO_OPENROUTER;

/**
 Seat writing the third candidate, and the one read in full.
 */
const THIRD = SEAT_HYPER_OPENROUTER_VISION_EDITOR;

/**
 Judge with no candidate of its own in any fixture.
 */
const OUTSIDER = 'hf:nvidia/NVIDIA-Nemotron-3-Super-120B-A12B-NVFP4';

/**
 Position of the candidate no slate here holds, named by one judge anyway.
 */
const PAST_THE_END = 4;

/**
 Builds one ballot without restating the fields every case shares.

 @param modelId - judge casting it

 @param best - one-based position named, or zero for an abstention

 @param reason - verbatim stated reason

 @returns Ballot as a ledger file records it

 @example
 ```ts
 const cast = ballot({ modelId: OUTSIDER, best: 1, reason: 'clearest', },);
 ```
 */
function ballot(
  {
    modelId,
    best,
    reason,
  }: {
    readonly modelId: string;
    readonly best: number;
    readonly reason: string;
  },
): ReadBallot {
  return {
    modelId,
    best,
    reason,
  };
}

/**
 One contest: three candidates, one of them written by two seats at once.

 Position 1 wins. `OUTSIDER` and `THIRD` back it, `JOINT_ONE` backs its own
 joint work, `SOLO` abstains, and `JOINT_TWO` names a fourth candidate the
 slate does not have.
 */
const CONTEST: ReadRound = {
  task: 'render this passage',
  at: '2026-08-25T00:00:00.000Z',
  candidates: [
    {
      index: 1,
      producers: [SOLO,],
      rendered: 'The tabby slept on the warm windowsill.',
    },
    {
      index: 2,
      producers: [
        JOINT_ONE,
        JOINT_TWO,
      ],
      rendered: 'A tabby was sleeping upon the sun-warmed sill.',
    },
    {
      index: 3,
      producers: [THIRD,],
      rendered: 'The cat dozed by the window.',
    },
  ],
  ballots: [
    ballot({
      modelId: OUTSIDER,
      best: 1,
      reason: 'keeps the warmth of the sill, which the others drop',
    },),
    ballot({
      modelId: THIRD,
      best: 1,
      reason: 'plainest of the three',
    },),
    ballot({
      modelId: JOINT_ONE,
      best: 2,
      reason: 'reads more naturally',
    },),
    ballot({
      modelId: SOLO,
      best: ABSTAINED,
      reason: 'none of these render the passage',
    },),
    ballot({
      modelId: JOINT_TWO,
      best: PAST_THE_END,
      reason: 'the fourth is best',
    },),
  ],
  selectedIndex: 1,
};

/**
 A second contest where the panel declined outright, so nothing won.
 */
const DECLINED: ReadRound = {
  task: 'render this passage',
  at: '2026-08-25T00:01:00.000Z',
  candidates: [
    {
      index: 1,
      producers: [THIRD,],
      rendered: 'The kitten batted at the string.',
    },
  ],
  ballots: [
    ballot({
      modelId: OUTSIDER,
      best: 1,
      reason: 'the only one offered',
    },),
  ],
  selectedIndex: 'declined',
};

/**
 Looks one seat up in a summary.

 THROWS RATHER THAN RETURNING AN ABSENCE, because every case here names a
 seat that wrote something, so a missing row is the reader losing it.

 @param summary - what `summariseLedger` returned

 @param model - seat wanted

 @returns That seat's counts

 @throws {@link Error} when the summary holds no row for that seat

 @example
 ```ts
 const seat = seatOf({ summary, model: SOLO, },);
 ```
 */
function seatOf(
  {
    summary,
    model,
  }: {
    readonly summary: LedgerSummary;
    readonly model: string;
  },
): ModelWork {
  /**
   Row for that seat, absent when the reader never saw it write anything.
   */
  const found = summary
    .models
    .find(function named(work,): boolean {
      return work.model === model;
    },);

  if (found === undefined)
    throw new Error(`no row for ${model}`,);

  return found;
}

/**
 Renders a contest the way a ledger file holds it, then reads it back.

 WRITES AND RE-READS RATHER THAN CLONING. A clone would hand the parser the
 very objects the fixture built, which is not what a file does: the parser's
 real input has been through a text form, and surviving that round trip is
 exactly what the recorder promises and what this checks.

 @param round - contest to render and read back

 @returns Whatever the text form parsed to, still unchecked

 @example
 ```ts
 const value = asFileWould({ round: CONTEST, },);
 ```
 */
function asFileWould(
  { round, }: { readonly round: ReadRound; },
): unknown {
  /**
   Exactly the bytes the recorder would write.
   */
  const text = JSON.stringify(round,);

  return JSON.parse(text,) as unknown;
}

/**
 Runs the parser over a value that must be refused, and hands back the
 refusal's message.

 @param value - malformed contest

 @param from - file name the refusal should carry

 @returns Message the refusal was raised with

 @throws {@link Error} when the parser accepted a value it should refuse

 @example
 ```ts
 const message = refusalMessageFor({ value, from, },);
 ```
 */
function refusalMessageFor(
  {
    value,
    from,
  }: {
    readonly value: unknown;
    readonly from: string;
  },
): string {
  try {
    parseLedgerRound({
      value,
      from,
    },);
  } catch (refusal) {
    return caughtValueText(refusal,);
  }

  throw new Error(`parser accepted ${from}, which it should have refused`,);
}


await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: summariseLedger.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'CREDITS both authors of a jointly written candidate, so a reader '
            + 'that kept only the first author would under-count the second',
          fn: async () => {
            /**
             One contest read.
             */
            const summary = summariseLedger({ rounds: [CONTEST,], },);

            expect({
              one: seatOf({
                summary,
                model: JOINT_ONE,
              },).candidates,
              two: seatOf({
                summary,
                model: JOINT_TWO,
              },).candidates,
            },)
              .toEqual({
                one: 1,
                two: 1,
              },);
          },
        },),

        it({
          name: 'COUNTS a self-vote apart from the votes that are evidence about a '
            + 'seat, matching what the standing does with the same rounds',
          fn: async () => {
            /**
             One contest read.
             */
            const summary = summariseLedger({ rounds: [CONTEST,], },);

            /**
             First author of the joint candidate, which voted for its own work.
             */
            const seat = seatOf({
              summary,
              model: JOINT_ONE,
            },);

            expect({
              selfVotes: seat.selfVotes,
              votes: seat.votes,
            },)
              .toEqual({
                selfVotes: 1,
                votes: 0,
              },);
          },
        },),

        it({
          name: 'EXCLUDES the authors of a candidate from its own denominator, so '
            + 'a seat two judges wrote is weighed by the three that did not',
          fn: async () => {
            /**
             One contest read.
             */
            const summary = summariseLedger({ rounds: [CONTEST,], },);

            expect({
              joint: seatOf({
                summary,
                model: JOINT_ONE,
              },).ballots,
              solo: seatOf({
                summary,
                model: SOLO,
              },).ballots,
            },)
              .toEqual({
                joint: 3,
                solo: 4,
              },);
          },
        },),

        it({
          name: 'NAMES the winner by the position the contest selected, and gives '
            + 'the losing seats no win',
          fn: async () => {
            /**
             One contest read.
             */
            const summary = summariseLedger({ rounds: [CONTEST,], },);

            expect({
              won: seatOf({
                summary,
                model: SOLO,
              },).wins,
              lost: seatOf({
                summary,
                model: JOINT_ONE,
              },).wins,
            },)
              .toEqual({
                won: 1,
                lost: 0,
              },);
          },
        },),

        it({
          name: 'COUNTS votes for a candidate whose own author abstained, which is '
            + 'a seat winning on other judges alone',
          fn: async () => {
            /**
             One contest read.
             */
            const summary = summariseLedger({ rounds: [CONTEST,], },);

            /**
             Solo seat, whose author named nothing at all.
             */
            const seat = seatOf({
              summary,
              model: SOLO,
            },);

            expect({
              votes: seat.votes,
              selfVotes: seat.selfVotes,
            },)
              .toEqual({
                votes: 2,
                selfVotes: 0,
              },);
          },
        },),

        it({
          name: 'SEPARATES an abstention from a ballot naming a candidate the '
            + 'slate does not have, since only the second is a fault in the judge',
          fn: async () => {
            /**
             One contest read.
             */
            const summary = summariseLedger({ rounds: [CONTEST,], },);

            expect({
              abstentions: summary.abstentions,
              namedMissing: summary.namedMissing,
            },)
              .toEqual({
                abstentions: 1,
                namedMissing: 1,
              },);
          },
        },),

        it({
          name: 'AWARDS no win where the panel declined, while still counting the '
            + 'ballot that backed the candidate it refused',
          fn: async () => {
            /**
             One contest whose panel chose nothing.
             */
            const summary = summariseLedger({ rounds: [DECLINED,], },);

            /**
             Only seat that wrote anything in it.
             */
            const seat = seatOf({
              summary,
              model: THIRD,
            },);

            expect({
              candidates: seat.candidates,
              wins: seat.wins,
              votes: seat.votes,
            },)
              .toEqual({
                candidates: 1,
                wins: 0,
                votes: 1,
              },);
          },
        },),

        it({
          name: 'COUNTS a ballot naming the LAST candidate on a slate as an '
            + 'ordinary vote, not as one naming a candidate that is not there',
          fn: async () => {
            // THE OFF-BY-ONE CASE. A ballot names a one-based position and the
            // slate is a zero-based array, so the join subtracts one. Every ballot
            // naming a middle position resolves under either indexing, and so does
            // a ballot past the end; only a ballot naming the LAST candidate tells
            // the two apart. The declined contest holds exactly one candidate and
            // its only judge names it, which is that case at its smallest.
            expect(summariseLedger({ rounds: [DECLINED,], },).namedMissing,)
              .toEqual(0,);
          },
        },),

        it({
          name: 'ORDERS seats by how much they wrote across every contest read, '
            + 'not by how often they were chosen',
          fn: async () => {
            /**
             Both contests read, so the third seat wrote twice and the rest once.
             */
            const summary = summariseLedger({
              rounds: [
                CONTEST,
                DECLINED,
              ],
            },);

            expect({
              first: summary
                .models
                .at(0,)
                ?.model,
              rounds: summary.rounds,
            },)
              .toEqual({
                first: THIRD,
                rounds: 2,
              },);
          },
        },),

        it({
          name: 'READS a ledger holding no contests as nothing rather than '
            + 'refusing, since a run that judged nothing is an ordinary run',
          fn: async () => {
            /**
             No contests at all.
             */
            const summary = summariseLedger({ rounds: [], },);

            expect(summary,)
              .toEqual({
                models: [],
                rounds: 0,
                abstentions: 0,
                namedMissing: 0,
              },);
          },
        },),
      ],
    },),

    describe({
      name: workOfModel.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'JOINS the stated reason of every disinterested judge to the '
            + 'candidate it named, which is the evidence a roster question needs',
          fn: async () => {
            /**
             Everything the solo seat wrote across one contest.
             */
            const written = workOfModel({
              rounds: [CONTEST,],
              model: SOLO,
            },);

            expect({
              pieces: written.length,
              remarks: written
                .at(0,)
                ?.remarks,
            },)
              .toEqual({
                pieces: 1,
                remarks: [
                  `${OUTSIDER}: keeps the warmth of the sill, which the others drop`,
                  `${THIRD}: plainest of the three`,
                ],
              },);
          },
        },),

        it({
          name: 'OMITS the remark an author made about its own candidate, so a '
            + 'seat cannot supply its own evidence',
          fn: async () => {
            /**
             Everything the joint candidate's first author wrote.
             */
            const written = workOfModel({
              rounds: [CONTEST,],
              model: JOINT_ONE,
            },);

            // Its own ballot is the only one naming position 2, so nothing is left.
            expect(written
              .at(0,)
              ?.remarks,)
              .toEqual([],);
          },
        },),

        it({
          name: 'CARRIES the text the judges actually compared, together with what '
            + 'they were deciding and whether it was chosen',
          fn: async () => {
            /**
             Everything the solo seat wrote.
             */
            const written = workOfModel({
              rounds: [CONTEST,],
              model: SOLO,
            },);

            expect({
              rendered: written
                .at(0,)
                ?.rendered,
              won: written
                .at(0,)
                ?.won,
              task: written
                .at(0,)
                ?.task,
            },)
              .toEqual({
                rendered: 'The tabby slept on the warm windowsill.',
                won: true,
                task: 'render this passage',
              },);
          },
        },),

        it({
          name: 'MARKS a declined contest as won by nobody, rather than reading '
            + 'the refusal as a loss to some other seat',
          fn: async () => {
            /**
             Everything the third seat wrote across both contests.
             */
            const written = workOfModel({
              rounds: [
                CONTEST,
                DECLINED,
              ],
              model: THIRD,
            },);

            expect({
              pieces: written.length,
              declined: written
                .at(1,)
                ?.won,
            },)
              .toEqual({
                pieces: 2,
                declined: false,
              },);
          },
        },),

        it({
          name: 'FINDS nothing for a seat that judged but never wrote, which is a '
            + 'silent answer and not an empty candidate',
          fn: async () => {
            /**
             A judge that never produced a candidate of its own.
             */
            const written = workOfModel({
              rounds: [CONTEST,],
              model: OUTSIDER,
            },);

            expect(written,)
              .toEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: parseLedgerRound.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ACCEPTS a contest holding every field, and reads the slate and '
            + 'the ballots back in the order they were written',
          fn: async () => {
            /**
             A contest round-tripped through the JSON a ledger file holds.
             */
            const round = parseLedgerRound({
              value: asFileWould({ round: CONTEST, },),
              from: 'ledger/000001.json',
            },);

            expect(round,)
              .toEqual(CONTEST,);
          },
        },),

        it({
          name: 'READS a declined outcome as the refusal it is, rather than '
            + 'refusing the file for holding no winning position',
          fn: async () => {
            /**
             The contest whose panel chose nothing.
             */
            const round = parseLedgerRound({
              value: asFileWould({ round: DECLINED, },),
              from: 'ledger/000002.json',
            },);

            expect(round.selectedIndex,)
              .toEqual('declined',);
          },
        },),

        it({
          name: 'REFUSES a file whose ballots are missing, rather than reading it '
            + 'as a contest nobody judged',
          fn: async () => {
            expect(function noBallots(): void {
              parseLedgerRound({
                value: {
                  task: 'render this passage',
                  at: '2026-08-25T00:00:00.000Z',
                  candidates: [],
                  selectedIndex: 'declined',
                },
                from: 'ledger/000003.json',
              },);
            },)
              .toThrow(LedgerShapeError,);
          },
        },),

        it({
          name: 'REFUSES a candidate whose producer list holds something that is '
            + 'not a model id, since the join reads those ids',
          fn: async () => {
            expect(function badProducer(): void {
              parseLedgerRound({
                value: {
                  task: 'render this passage',
                  at: '2026-08-25T00:00:00.000Z',
                  candidates: [
                    {
                      index: 1,
                      producers: [7,],
                      rendered: 'The tabby slept.',
                    },
                  ],
                  ballots: [],
                  selectedIndex: 1,
                },
                from: 'ledger/000004.json',
              },);
            },)
              .toThrow(LedgerShapeError,);
          },
        },),

        it({
          name: 'REFUSES a selected position that is neither a number nor the word '
            + 'a declining panel writes',
          fn: async () => {
            expect(function badOutcome(): void {
              parseLedgerRound({
                value: {
                  task: 'render this passage',
                  at: '2026-08-25T00:00:00.000Z',
                  candidates: [],
                  ballots: [],
                  selectedIndex: 'maybe',
                },
                from: 'ledger/000005.json',
              },);
            },)
              .toThrow(LedgerShapeError,);
          },
        },),

        it({
          name: 'REFUSES a value that is not an object at all, which is what a '
            + 'truncated write leaves behind',
          fn: async () => {
            expect(function notAnObject(): void {
              parseLedgerRound({
                value: null,
                from: 'ledger/000006.json',
              },);
            },)
              .toThrow(LedgerShapeError,);
          },
        },),

        it({
          name: 'NAMES the file and the field in a refusal and quotes neither the '
            + 'passage nor the reason, since a ledger holds corpus wording',
          fn: async () => {
            /**
             A contest whose ballots field is a string, carrying a passage in a
             field beside it that the refusal must not echo.
             */
            const message = refusalMessageFor({
              value: {
                task: 'render this passage',
                at: '2026-08-25T00:00:00.000Z',
                candidates: [
                  {
                    index: 1,
                    producers: [SOLO,],
                    rendered: 'The tabby slept on the warm windowsill.',
                  },
                ],
                selectedIndex: 1,
              },
              from: 'ledger/000007.json',
            },);

            expect({
              namesFile: message.includes('ledger/000007.json',),
              namesField: message.includes('ballots',),
              quotesPassage: message.includes('windowsill',),
            },)
              .toEqual({
                namesFile: true,
                namesField: true,
                quotesPassage: false,
              },);
          },
        },),
      ],
    },),
  ],
},);

//endregion Ledger read tests
