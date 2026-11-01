/**
 Tests for one seat's rendered standing.

 THE REPORT IS LINES, NOT PRINTS, so these cases read it without capturing
 the console. What they pin: a seat with no rounds says so instead of
 rendering an empty table, a seat with rounds renders its standings and then
 its coverage gaps with the answered-but-unslated state kept apart from the
 silent one, and `slatedAuthors` names every stakeholder of every slate in
 slate order, composites included.

 Fixtures are model ids and ballots, so there is no passage here to invent.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import type { SelectionRound, } from '@monochromatic-dev/translation-repair-translate-floor';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  slatedAuthors,
  sliceStandingLines,
  standingReportLines,
} from '../../dist/final/node/index.mjs';
import {
  SEAT_HYPER_TEXT_BEDROCK,
  SEAT_HYPER_VISION,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

//region Fixtures

/**
 Model whose candidate a judge voted for, so it earns a row.
 */
const WRITER: RosterModelId = SEAT_SYNTHETIC_VISION_WITHHELD;

/**
 Model that cast the ballot and, as a rewriter, was heard proposing nothing.
 */
const JUDGE: RosterModelId = SEAT_SYNTHETIC_VISION_NO_OPENROUTER;

/**
 Model no answer came from.
 */
const IDLE: RosterModelId = SEAT_HYPER_VISION;

/**
 Second contributor of a composite candidate.
 */
const PARTNER: RosterModelId = SEAT_HYPER_TEXT_BEDROCK;

/**
 Seats the run filled, in the order the report should preserve.
 */
const ROSTER: readonly RosterModelId[] = [
  WRITER,
  JUDGE,
  IDLE,
];

/**
 One judged round: a single candidate by the writer, voted for by the judge.
 */
const VOTED_ROUND: SelectionRound = {
  producers: [
    {
      kind: 'model',
      modelId: WRITER,
    },
  ],
  ballots: [
    {
      modelId: JUDGE,
      best: 1,
      reason: 'scripted',
      weight: 1,
      selfVote: false,
    },
  ],
};

/**
 Round whose slate carries a composite candidate beside a plain one, with no
 ballot cast over it.
 */
const COMPOSITE_ROUND: SelectionRound = {
  producers: [
    {
      kind: 'composite',
      contributors: [
        WRITER,
        PARTNER,
      ],
    },
    {
      kind: 'model',
      modelId: JUDGE,
    },
  ],
  ballots: [],
};

/**
 The same slate with the judge's ballot for the composite cast over it.
 */
const JUDGED_COMPOSITE_ROUND: SelectionRound = {
  producers: COMPOSITE_ROUND.producers,
  ballots: [
    {
      modelId: JUDGE,
      best: 1,
      reason: 'scripted',
      weight: 1,
      selfVote: false,
    },
  ],
};

/**
 Round whose slate is one composite candidate every editor proposed, which no
 ballot was cast over, as the run that found the two printers disagreeing had.
 */
const LONE_COMPOSITE_ROUND: SelectionRound = {
  producers: [
    {
      kind: 'composite',
      contributors: [
        WRITER,
        JUDGE,
      ],
    },
  ],
  ballots: [],
};

//endregion Fixtures

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: standingReportLines.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'renders a heading and the no-rounds note when nothing was judged',
          fn: async () => {
            /**
             Report for a seat that judged nothing on either slice.
             */
            const lines = standingReportLines({
              seat: 'EDITOR',
              roster: ROSTER,
              perSlice: [
                [],
                [],
              ],
              produced: [],
              answered: { kind: 'unrecorded', },
            },);

            expect(lines,).toHaveLength(2,);
            expect(lines[0],).toBe('\nEDITOR standing over 0 judged rounds, from 0 of 2 slices',);
            expect(lines[1],).toContain('NO ROUNDS',);
          },
        },),

        it({
          name: 'COUNTS A ROUND WHOSE LONE COMPOSITE CANDIDATE DREW NO BALLOT as no judged round, the heading '
            + 'counting it apart, and says no ballot was cast rather than that the seat had no round',
          fn: async () => {
            expect(standingReportLines({
              seat: 'EDITOR',
              roster: ROSTER,
              perSlice: [[LONE_COMPOSITE_ROUND,],],
              produced: [
                WRITER,
                JUDGE,
              ],
              answered: { kind: 'unrecorded', },
            },),).toEqual([
              '\nEDITOR standing over 0 judged rounds, from 0 of 1 slice; 1 round drew no ballot',
              '  NO JUDGED ROUNDS. The seat has no standing, since no ballot was cast on any round it produced: a '
                + 'slate of one candidate, which is what every producer proposing the same wording leaves, needs no '
                + 'vote, and a panel whose every judge abstained or failed casts none. Draw more slices.',
            ],);
          },
        },),

        it({
          name: 'COUNTS ONLY THE JUDGED ROUNDS in the table and the slice lines when a seat\'s rounds are judged on '
            + 'one slice and drew no ballot on another, the heading counting the round no ballot was cast on '
            + 'apart, and naming the writer the table leaves out',
          fn: async () => {
            expect(standingReportLines({
              seat: 'EDITOR',
              roster: [
                WRITER,
                JUDGE,
              ],
              perSlice: [
                [VOTED_ROUND,],
                [LONE_COMPOSITE_ROUND,],
              ],
              produced: [
                WRITER,
                JUDGE,
              ],
              answered: { kind: 'unrecorded', },
            },),).toEqual([
              '\nEDITOR standing over 1 judged round, from 1 of 2 slices; 1 round drew no ballot',
              `  ${WRITER}: 100.0% (1 of 1 disinterested ballot, over 1 candidate)`,
              `  WROTE AND WAS NEVER VOTED ON: ${JUDGE}. Their text reached a slate and no disinterested ballot `
                + 'was cast over it, which is what a slice where every producer proposed the same wording does: it '
                + 'ships unjudged. The table says nothing about them either way, and more slices are what would.',
              `  slice 1: 1 judged round; ${WRITER} 1/1 over 1`,
            ],);
          },
        },),

        it({
          name: 'renders the standings, then the answered-but-unslated seat, then the silent seat, '
            + 'each on its own line',
          fn: async () => {
            /**
             Report for one voted round on the first of two slices, at a seat
             that heard the writer and the judge and never the idle model.
             */
            const lines = standingReportLines({
              seat: 'REFINER',
              roster: ROSTER,
              perSlice: [
                [VOTED_ROUND,],
                [],
              ],
              produced: [WRITER,],
              answered: {
                kind: 'recorded',
                modelIds: [
                  WRITER,
                  JUDGE,
                ],
              },
            },);

            expect(lines[0],).toBe('\nREFINER standing over 1 judged round, from 1 of 2 slices',);
            expect(lines[1],).toContain(WRITER,);
            expect(lines[2],).toContain('ANSWERED AND WAS NEVER SLATED',);
            expect(lines[2],).toContain(JUDGE,);
            expect(lines[2],).not.toContain(IDLE,);
            expect(lines[3],).toContain('ANSWERED NOTHING USABLE',);
            expect(lines[3],).toContain(IDLE,);
            expect(lines[3],).not.toContain(JUDGE,);
            expect(lines.at(-1,),).toBe(`  slice 1: 1 judged round; ${WRITER} 1/1 over 1`,);
            expect(lines.slice(1, -1,).some(function isSliceLine(line,): boolean {
              return line.startsWith('  slice ',);
            },),).toBe(false,);
          },
        },),

        it({
          name: 'ends with one counts line per slice that bought a judged round, in sample order, '
            + 'crediting every author of a composite and skipping slices that bought nothing or only a round no '
            + 'ballot was cast over',
          fn: async () => {
            /**
             Per-slice lines for a voted slice, an empty slice, a composite
             slice nobody voted on and the same slate voted on.
             */
            const lines = sliceStandingLines({
              perSlice: [
                [VOTED_ROUND,],
                [],
                [COMPOSITE_ROUND,],
                [JUDGED_COMPOSITE_ROUND,],
              ],
            },);

            expect(lines,).toStrictEqual([
              `  slice 1: 1 judged round; ${WRITER} 1/1 over 1`,
              `  slice 4: 1 judged round; ${WRITER} 1/1 over 1; ${PARTNER} 1/1 over 1; ${JUDGE} 0/0 over 1`,
            ],);
          },
        },),

        it({
          name: 'indents every line after the heading, so the report reads as one block under it',
          fn: async () => {
            /**
             Report with a standing line and a coverage line to check the
             indentation of.
             */
            const lines = standingReportLines({
              seat: 'EDITOR',
              roster: ROSTER,
              perSlice: [[VOTED_ROUND,],],
              produced: [WRITER,],
              answered: { kind: 'unrecorded', },
            },);

            for (const line of lines.slice(1,)) {
              expect(line.startsWith('  ',),).toBe(true,);
            }
          },
        },),
      ],
    },),

    describe({
      name: slatedAuthors.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'names every stakeholder of every slate in slate order, composites flattened',
          fn: async () => {
            /**
             Authors across two slices, the second carrying a composite.
             */
            const authors = slatedAuthors({
              perSlice: [
                [VOTED_ROUND,],
                [COMPOSITE_ROUND,],
              ],
            },);

            expect(authors,).toStrictEqual([
              WRITER,
              WRITER,
              PARTNER,
              JUDGE,
            ],);
          },
        },),

        it({
          name: 'names nobody for slices that bought no round',
          fn: async () => {
            expect(slatedAuthors({
              perSlice: [
                [],
                [],
              ],
            },),).toStrictEqual([],);
          },
        },),
      ],
    },),
  ],
},);
