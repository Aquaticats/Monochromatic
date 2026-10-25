/**
 Tests for what a checker roster may look like: when a checker is allowed to
 have written the text it grades, and how small the roster may get.

 TWO REFUSALS BEHAVE DIFFERENTLY AND THE DIFFERENCE IS THE POINT. Overlap is a
 question about evidence quality, and `tallyResolutionChecks` can answer it by
 halving a self-vote per issue, so the owner's ruling of 2026-08-23 sent it to
 measurement behind a switch. A repeated id and a roster too small to decide
 are not that kind of question: the first makes the quorum count disagree with
 the ballot count, and the second makes disagreement return nothing at all.
 Neither is rescued by any weighting, so neither honours the switch.

 Model ids come from the catalog because `RosterModelId` is a closed union.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  assertCheckerIndependence,
  assertCheckerQuorumReachable,
  CheckerIndependenceError,
  CheckerQuorumError,
} from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_VISION,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 Writers as production seats them: three models that edit and refine.
 */
const WRITERS: readonly RosterModelId[] = [
  SEAT_SYNTHETIC_VISION_WITHHELD,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_HYPER_VISION,
];

/**
 Checkers as production seats them, disjoint from every writer.
 */
const DISJOINT_CHECKERS: readonly RosterModelId[] = [
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
];

/**
 Whole roster checking, which is the arm the measurement exists to price.
 */
const EVERY_MODEL: readonly RosterModelId[] = [
  ...WRITERS,
  ...DISJOINT_CHECKERS,
];

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: assertCheckerIndependence.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'REFUSES a checker that also writes when nothing permits it, which is the default and '
            + 'stays the default: an unset switch must not read as permission',
          fn: async () => {
            expect(function checksOwnWork() {
              assertCheckerIndependence({
                editorModelIds: WRITERS,
                checkerModelIds: EVERY_MODEL,
              },);
            },).toThrow(CheckerIndependenceError,);

            expect(function permissionOff() {
              assertCheckerIndependence({
                editorModelIds: WRITERS,
                checkerModelIds: EVERY_MODEL,
                selfCertificationPermitted: false,
              },);
            },).toThrow(CheckerIndependenceError,);
          },
        },),

        it({
          name: 'ACCEPTS the whole roster checking when self-certification is permitted, since the '
            + 'weight of a checker on text it wrote is chosen per issue rather than the seat being '
            + 'refused outright',
          fn: async () => {
            assertCheckerIndependence({
              editorModelIds: WRITERS,
              refinerModelIds: WRITERS,
              checkerModelIds: EVERY_MODEL,
              selfCertificationPermitted: true,
            },);
          },
        },),

        it({
          name: 'REFUSES a REPEATED checker id even with self-certification permitted, because a repeat '
            + 'is a different fault: it meets quorum on fewer independent voices than the roster size '
            + 'promises, and no per-issue weighting can undo that',
          fn: async () => {
            expect(function repeatsAVoice() {
              assertCheckerIndependence({
                editorModelIds: WRITERS,
                checkerModelIds: [
                  ...DISJOINT_CHECKERS,
                  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
                ],
                selfCertificationPermitted: true,
              },);
            },).toThrow(CheckerIndependenceError,);
          },
        },),

        it({
          name: 'REFUSES a REFINER among the checkers as readily as an editor, because the recheck that '
            + 'follows a refinement asks whether the accepted issues survived it',
          fn: async () => {
            expect(function refinerChecks() {
              assertCheckerIndependence({
                editorModelIds: [SEAT_SYNTHETIC_VISION_WITHHELD,],
                refinerModelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
                checkerModelIds: [
                  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
                  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
                  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
                ],
              },);
            },).toThrow(CheckerIndependenceError,);
          },
        },),

        it({
          name: 'ACCEPTS production’s disjoint rosters unchanged, so the switch landing changes '
            + 'nothing about the arm that ships today',
          fn: async () => {
            assertCheckerIndependence({
              editorModelIds: WRITERS,
              refinerModelIds: WRITERS,
              checkerModelIds: DISJOINT_CHECKERS,
            },);
          },
        },),
      ],
    },),

    describe({
      name: assertCheckerQuorumReachable.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ACCEPTS three checkers, which is where a two-to-one reading still resolves',
          fn: async () => {
            assertCheckerQuorumReachable({ checkerModelIds: DISJOINT_CHECKERS, },);
            assertCheckerQuorumReachable({ checkerModelIds: EVERY_MODEL, },);
          },
        },),

        it({
          name: 'ACCEPTS two checkers (the owner\'s answer of 2026-10-25, "Allow two checkers"): two answering '
            + 'checkers are two ballots, which decide a unanimous two-nil, and a split one-one resolves '
            + 'nothing rather than anything',
          fn: async () => {
            assertCheckerQuorumReachable({ checkerModelIds: [
              SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
              SEAT_SYNTHETIC_TEXT_EVERYWHERE,
            ], },);
          },
        },),

        it({
          name: 'REFUSES one checker and an empty roster, since one judge alone never decides, so the floor '
            + 'covers the case the empty-role guard already caught as well as the ones it never did',
          fn: async () => {
            expect(function onlyOne() {
              assertCheckerQuorumReachable({ checkerModelIds: [SEAT_SYNTHETIC_VISION_NO_OPENROUTER,], },);
            },).toThrow(CheckerQuorumError,);
            expect(function nobody() {
              assertCheckerQuorumReachable({ checkerModelIds: [], },);
            },).toThrow(CheckerQuorumError,);
          },
        },),

        it({
          name: 'NAMES the floor of two in the refusal, so a reader of a refused roster is told the number that '
            + 'holds now',
          fn: async () => {
            expect(function refusesARosterOfOne() {
              assertCheckerQuorumReachable({ checkerModelIds: [SEAT_SYNTHETIC_VISION_NO_OPENROUTER,], },);
            },).toThrow('against a floor of 2',);
          },
        },),

        it({
          name: 'REFUSES a roster of one that is ALSO permitted to self-certify, since the floor and '
            + 'the switch answer different questions and permission to overlap is not permission to '
            + 'shrink below a decidable panel',
          fn: async () => {
            /** One writer checking its own work, which passes independence under the switch. */
            const oneWriter: readonly RosterModelId[] = [SEAT_SYNTHETIC_VISION_WITHHELD,];
            assertCheckerIndependence({
              editorModelIds: WRITERS,
              checkerModelIds: oneWriter,
              selfCertificationPermitted: true,
            },);
            expect(function stillTooFew() {
              assertCheckerQuorumReachable({ checkerModelIds: oneWriter, },);
            },).toThrow(CheckerQuorumError,);
          },
        },),

        it({
          name: 'ACCEPTS a roster of two that is ALSO permitted to self-certify, the discount halving both '
            + 'sides of the tally so two writers still resolve a unanimous two-nil',
          fn: async () => {
            /** Two writers checking their own work, which passes independence under the switch. */
            const twoWriters: readonly RosterModelId[] = [
              SEAT_SYNTHETIC_VISION_WITHHELD,
              SEAT_HYPER_OPENROUTER_VISION_EDITOR,
            ];
            assertCheckerIndependence({
              editorModelIds: WRITERS,
              checkerModelIds: twoWriters,
              selfCertificationPermitted: true,
            },);
            assertCheckerQuorumReachable({ checkerModelIds: twoWriters, },);
          },
        },),
      ],
    },),
  ],
},);
