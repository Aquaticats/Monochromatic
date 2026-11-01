/**
 Tests for naming who a standing table leaves out.

 THE WHOLE POINT IS THE SPLIT. `producerStandings` carries a row only for a
 model somebody voted on, and three very different things put a seated model
 outside that set: a provider that refused it, a slate where every peer
 proposed the same wording so nothing was ever voted on, and a rewriter that
 answered every ask and never proposed a candidate. These cases pin that the
 three stay apart, that a seat which cannot say who answered says so instead
 of calling the unknown silent, and that a table describing a roster the run
 never seated is refused rather than reported.

 Fixtures are model ids and counts, so there is no passage here to invent. The
 ids come from the catalog, since the roster is what this file is about.

 @module
 */

import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  coverageGapLines,
  type ProducerStanding,
  readStandingCoverage,
  type SeatAnswers,
  UnseatedStandingError,
} from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_TEXT_BEDROCK,
  SEAT_HYPER_VISION,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 Model whose candidates drew ballots in every case that needs one.
 */
const JUDGED: RosterModelId = SEAT_SYNTHETIC_VISION_WITHHELD;

/**
 Model that writes without ever being voted on.
 */
const UNVOTED: RosterModelId = SEAT_SYNTHETIC_VISION_NO_OPENROUTER;

/**
 Model whose provider is out of budget, so it writes nothing.
 */
const ABSENT: RosterModelId = SEAT_HYPER_VISION;

/**
 Second silent model, so ordering can be checked on more than one.
 */
const ALSO_ABSENT: RosterModelId = SEAT_HYPER_TEXT_BEDROCK;

/**
 Model the roster dropped on 2026-08-24, which no run seats today.

 CAST ON PURPOSE, and this is the only cast here. `RosterModelId` is a closed
 union of models a run may seat, so a departed id cannot be spelled inside it,
 and refusing one is exactly what the refusal cases check. A test that could
 not name a departed model could not check the refusal at all.
 `judge-fidelity.unit.test.ts` reaches for the same cast for the same reason.
 */
const DEPARTED = 'hf:zai-org/GLM-4.7-Flash' as unknown as RosterModelId;

/**
 Seats every case starts from, in the order a report should preserve.
 */
const ROSTER: readonly RosterModelId[] = [
  JUDGED,
  UNVOTED,
  ABSENT,
  ALSO_ABSENT,
];

/**
 A seat that does not carry its answers out, as the editor seat does not.
 */
const UNRECORDED: SeatAnswers = { kind: 'unrecorded', };

/**
 Builds one standing row, since only its model id decides coverage.

 @param modelId - model the row is about

 @returns Row shaped as the tally produces them

 @example
 ```ts
 const standings = [standingOf({ modelId: JUDGED, },),];
 ```
 */
function standingOf(
  { modelId, }: { readonly modelId: RosterModelId; },
): ProducerStanding {
  return {
    modelId,
    candidates: 3,
    disinterestedBallots: 9,
    disinterestedVotes: 4,
  };
}

/**
 Builds a recorded answer list, so cases read as what was heard.

 @param modelIds - models heard with a usable answer

 @returns Recorded seat answers

 @example
 ```ts
 const answered = heardFrom([JUDGED, ABSENT,],);
 ```
 */
function heardFrom(modelIds: readonly RosterModelId[],): SeatAnswers {
  return {
    kind: 'recorded',
    modelIds,
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: readStandingCoverage.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'splits a seated roster into judged, unvoted and silent when answers are unrecorded',
          fn: async () => {
            /**
             One model of each kind, so all the reachable groups are non-empty.
             */
            const coverage = readStandingCoverage({
              roster: ROSTER,
              standings: [standingOf({ modelId: JUDGED, },),],
              produced: [
                JUDGED,
                UNVOTED,
              ],
              answered: UNRECORDED,
            },);

            expect(coverage.judged,).toStrictEqual([JUDGED,],);
            expect(coverage.wroteUnjudged,).toStrictEqual([UNVOTED,],);
            expect(coverage.answeredUnslated,).toStrictEqual([],);
            expect(coverage.neverWrote,).toStrictEqual([
              ABSENT,
              ALSO_ABSENT,
            ],);
            expect(coverage.answersRecorded,).toBe(false,);
          },
        },),

        it({
          name: 'REPORTS a seat that answered and never reached a slate as answered-but-unslated, '
            + 'never as silent, once the seat records who answered',
          fn: async () => {
            /**
             Coverage where one absent-from-the-table model was heard on every
             ask and the other never answered, which is the live case: a SEAT
             line at 31 of 31 beside a table calling the seat silent.
             */
            const coverage = readStandingCoverage({
              roster: ROSTER,
              standings: [standingOf({ modelId: JUDGED, },),],
              produced: [JUDGED,],
              answered: heardFrom([
                JUDGED,
                ABSENT,
                ABSENT,
              ],),
            },);

            expect(coverage.answeredUnslated,).toStrictEqual([ABSENT,],);
            // The unvoted model neither wrote nor answered in this fixture, so it is silent beside the other.
            expect(coverage.neverWrote,).toStrictEqual([
              UNVOTED,
              ALSO_ABSENT,
            ],);
            expect(coverage.answersRecorded,).toBe(true,);
          },
        },),

        it({
          name: 'keeps an answering model that also produced a candidate out of the unslated group',
          fn: async () => {
            /**
             Coverage where the unvoted writer was also heard, which must count
             it once, as a writer, since writing implies answering.
             */
            const coverage = readStandingCoverage({
              roster: ROSTER,
              standings: [standingOf({ modelId: JUDGED, },),],
              produced: [UNVOTED,],
              answered: heardFrom([
                UNVOTED,
                JUDGED,
              ],),
            },);

            expect(coverage.wroteUnjudged,).toStrictEqual([UNVOTED,],);
            expect(coverage.answeredUnslated,).toStrictEqual([],);
          },
        },),

        it({
          name: 'keeps roster order rather than the order evidence arrived in',
          fn: async () => {
            /**
             Silent models, listed in the order the seats were filled even though
             nothing about them was produced in that order.
             */
            const { neverWrote, } = readStandingCoverage({
              roster: ROSTER,
              standings: [standingOf({ modelId: JUDGED, },),],
              produced: [UNVOTED,],
              answered: UNRECORDED,
            },);

            expect(neverWrote,).toStrictEqual([
              ABSENT,
              ALSO_ABSENT,
            ],);
          },
        },),

        it({
          name: 'counts a model that wrote AND was judged once, as judged',
          fn: async () => {
            /**
             Coverage where the same model appears in every input, which is the
             ordinary case: everything judged was also written and answered.
             */
            const coverage = readStandingCoverage({
              roster: ROSTER,
              standings: [standingOf({ modelId: JUDGED, },),],
              produced: [
                JUDGED,
                JUDGED,
              ],
              answered: heardFrom([JUDGED,],),
            },);

            expect(coverage.judged,).toStrictEqual([JUDGED,],);
            expect(coverage.wroteUnjudged,).toStrictEqual([],);
            expect(coverage.answeredUnslated,).toStrictEqual([],);
          },
        },),

        it({
          name: 'ACCEPTS a run where every seated model was judged',
          fn: async () => {
            /**
             Coverage with nothing missing, so every silent group is empty.
             */
            const coverage = readStandingCoverage({
              roster: [JUDGED,],
              standings: [standingOf({ modelId: JUDGED, },),],
              produced: [JUDGED,],
              answered: heardFrom([JUDGED,],),
            },);

            expect(coverage.wroteUnjudged,).toStrictEqual([],);
            expect(coverage.answeredUnslated,).toStrictEqual([],);
            expect(coverage.neverWrote,).toStrictEqual([],);
          },
        },),

        it({
          name: 'REFUSES a standing naming a model the run never seated',
          fn: async () => {
            /**
             What the reader threw, held so the class and the id it names can be
             asserted apart.
             */
            const refusal = caught(function readsAnotherRoster() {
              readStandingCoverage({
                roster: ROSTER,
                standings: [standingOf({ modelId: DEPARTED, },),],
                produced: [],
                answered: UNRECORDED,
              },);
            },);

            expect(refusal,).toBeInstanceOf(UnseatedStandingError,);
            expect((refusal as Error).message,).toContain(DEPARTED,);
          },
        },),

        it({
          name: 'REFUSES a produced list naming a model the run never seated',
          fn: async () => {
            /**
             Same refusal reached through the second input, since a slate naming
             an unseated model is the same contradiction as a table doing it.
             */
            const refusal = caught(function readsAnotherSlate() {
              readStandingCoverage({
                roster: ROSTER,
                standings: [],
                produced: [DEPARTED,],
                answered: UNRECORDED,
              },);
            },);

            expect(refusal,).toBeInstanceOf(UnseatedStandingError,);
            expect((refusal as Error).message,).toContain(DEPARTED,);
          },
        },),

        it({
          name: 'REFUSES an answer list naming a model the run never seated',
          fn: async () => {
            /**
             Same refusal reached through the third input, since a seat hearing
             from an unseated model is the same contradiction again.
             */
            const refusal = caught(function readsAnotherSeat() {
              readStandingCoverage({
                roster: ROSTER,
                standings: [],
                produced: [],
                answered: heardFrom([DEPARTED,],),
              },);
            },);

            expect(refusal,).toBeInstanceOf(UnseatedStandingError,);
            expect((refusal as Error).message,).toContain(DEPARTED,);
          },
        },),
      ],
    },),

    describe({
      name: coverageGapLines.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'says nothing when every seated model is on the table',
          fn: async () => {
            /**
             Lines for a run with no gap at all, which must be none rather than a
             note claiming completeness.
             */
            const lines = coverageGapLines({
              coverage: readStandingCoverage({
                roster: [JUDGED,],
                standings: [standingOf({ modelId: JUDGED, },),],
                produced: [JUDGED,],
                answered: heardFrom([JUDGED,],),
              },),
            },);

            expect(lines,).toStrictEqual([],);
          },
        },),

        it({
          name: 'names the unvoted writers apart from the silent models',
          fn: async () => {
            /**
             Lines for a run with both kinds of gap, which must be two distinct
             lines rather than one absence.
             */
            const lines = coverageGapLines({
              coverage: readStandingCoverage({
                roster: ROSTER,
                standings: [standingOf({ modelId: JUDGED, },),],
                produced: [
                  JUDGED,
                  UNVOTED,
                ],
                answered: UNRECORDED,
              },),
            },);

            expect(lines,).toHaveLength(2,);
            expect(lines[0],).toContain(UNVOTED,);
            expect(lines[0],).not.toContain(ABSENT,);
            expect(lines[1],).toContain(ABSENT,);
            expect(lines[1],).toContain(ALSO_ABSENT,);
            expect(lines[1],).not.toContain(UNVOTED,);
          },
        },),

        it({
          name: 'names an answered-but-unslated seat on its own line, which never tells the reader '
            + 'to re-run it and never calls it silent',
          fn: async () => {
            /**
             Lines for the live case: one seat heard on every ask with nothing
             of its reaching a slate, one seat never heard.
             */
            const lines = coverageGapLines({
              coverage: readStandingCoverage({
                roster: ROSTER,
                standings: [standingOf({ modelId: JUDGED, },),],
                produced: [
                  JUDGED,
                  UNVOTED,
                ],
                answered: heardFrom([
                  JUDGED,
                  UNVOTED,
                  ABSENT,
                ],),
              },),
            },);

            expect(lines,).toHaveLength(3,);
            expect(lines[1],).toContain('ANSWERED AND WAS NEVER SLATED',);
            expect(lines[1],).toContain(ABSENT,);
            expect(lines[1],).not.toContain(ALSO_ABSENT,);
            expect(lines[1],).not.toContain('Re-run',);
            expect(lines[2],).toContain('ANSWERED NOTHING USABLE',);
            expect(lines[2],).toContain(ALSO_ABSENT,);
            expect(lines[2],).not.toContain(ABSENT,);
          },
        },),

        it({
          name: 'carries both denominators over all four groups, so a narrowed table cannot read '
            + 'as a full one',
          fn: async () => {
            /**
             Line about the silent model, which has to count the unslated seat
             among the seats filled even though it is reported on another line.
             */
            const lines = coverageGapLines({
              coverage: readStandingCoverage({
                roster: ROSTER,
                standings: [standingOf({ modelId: JUDGED, },),],
                produced: [JUDGED,],
                answered: heardFrom([
                  JUDGED,
                  UNVOTED,
                ],),
              },),
            },);

            expect(lines,).toHaveLength(2,);
            expect(lines[1],).toContain('covers 1 of 4 seats',);
          },
        },),

        it({
          name: 'says a seat does not record who answered instead of calling the absent silent, '
            + 'and still points at the SEAT lines, since this line cannot tell a budget refusal '
            + 'from a timeout from an answer dropped before judging and the seat report can '
            + '(calibrate-1)',
          fn: async () => {
            /**
             Line about the models off the table at a seat that carries no
             answer list out, which has to say so and say where the counts are.
             */
            const lines = coverageGapLines({
              coverage: readStandingCoverage({
                roster: ROSTER,
                standings: [standingOf({ modelId: JUDGED, },),],
                produced: [JUDGED,],
                answered: UNRECORDED,
              },),
            },);

            expect(lines,).toHaveLength(1,);
            expect(lines[0],).toContain('NO CANDIDATE OF THEIRS REACHED ANY SLATE',);
            expect(lines[0],).toContain('does not record who answered',);
            expect(lines[0],).not.toContain('ANSWERED NOTHING USABLE',);
            expect(lines[0],).toContain('the SEAT lines at the end of this command',);
          },
        },),

        it({
          name: 'POINTS AT THE SEAT LINES from the recorded silent-seat sentence too',
          fn: async () => {
            /**
             Line about a model no usable answer came from, at a seat that does
             record answers, which has to say where the per-seat counts are.
             */
            const lines = coverageGapLines({
              coverage: readStandingCoverage({
                roster: ROSTER,
                standings: [standingOf({ modelId: JUDGED, },),],
                produced: [JUDGED,],
                answered: heardFrom([JUDGED,],),
              },),
            },);

            expect(lines,).toHaveLength(1,);
            expect(lines[0],).toContain('ANSWERED NOTHING USABLE',);
            expect(lines[0],).toContain('the SEAT lines at the end of this command',);
          },
        },),
      ],
    },),
  ],
},);
