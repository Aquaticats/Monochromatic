/**
 Tests for what would ship at a contested slice without the consolidation.

 WHAT THIS PINS is the decline case, which is the one with consequences. A
 rule that picked a lane on a decline would hand the consolidation's slate a
 candidate no panel chose, and the whole stage would then be deciding between
 a fresh rendering and an arbitrary one.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  contestStandingMayShip,
  standingTextFor,
} from '../dist/final/node/index.mjs';

/**
 What archive carries.
 */
const ARCHIVE = 'The cat sleeps by the window.';

/**
 What the repair lane would ship.
 */
const REPAIR = 'The cat fell asleep by the window.';

/**
 What the translate lane would ship.
 */
const TRANSLATE = 'The cat had fallen asleep beside the window.';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: contestStandingMayShip.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'REFUSES INVALID STANDING even when contest chose its lane',
          fn: async () => {
            expect(contestStandingMayShip({
              choice: 'repair',
              verdict: {
                kind: 'lane-won',
                lane: 'repair',
              },
              standingValid: false,
            },),).toBe(false,);
          },
        },),

        it({
          name: 'ACCEPTS VALID LANE WINNER and keeps unendorsed decline unsafe',
          fn: async () => {
            expect(contestStandingMayShip({
              choice: 'translate',
              verdict: {
                kind: 'lane-won',
                lane: 'translate',
              },
            },),).toBe(true,);
            expect(contestStandingMayShip({
              choice: 'neither',
              verdict: { kind: 'settled-neither', },
            },),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: standingTextFor.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES THE REPAIR LANE when the contest chose it',
          fn: async () => {
            expect(
              standingTextFor({
                choice: 'repair',
                repairText: REPAIR,
                translateText: TRANSLATE,
                incumbentText: ARCHIVE,
              },),
            ).toBe(REPAIR,);
          },
        },),

        it({
          name: 'NAMES THE TRANSLATE LANE when the contest chose it',
          fn: async () => {
            expect(
              standingTextFor({
                choice: 'translate',
                repairText: REPAIR,
                translateText: TRANSLATE,
                incumbentText: ARCHIVE,
              },),
            ).toBe(TRANSLATE,);
          },
        },),

        it({
          name: 'USES ARCHIVE AS COMPARISON BASELINE ON A DECLINE without picking either lane, so third rendering can recover while final guard separately prevents unendorsed fallback',
          fn: async () => {
            expect(
              standingTextFor({
                choice: 'neither',
                repairText: REPAIR,
                translateText: TRANSLATE,
                incumbentText: ARCHIVE,
              },),
            ).toBe(ARCHIVE,);
          },
        },),
      ],
    },),
  ],
},);
