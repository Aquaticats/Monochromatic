/**
 Tests for the live window check: whether the neighbouring original reached
 the judges of the first wide arm.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  checkWindowReached,
  WINDOW_UNCHECKED,
  WindowEvidenceError,
} from '../../dist/final/node/index.mjs';
import { capturingLoggerPair, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { armRow, } from './window-trial-probe-rows.test-fixture.ts';

/**
 Sheet a judge of the wide arm received.
 */
const WIDE_SHEET = 'the passage\nSURROUNDING ORIGINAL\n猫猫在睡觉。';

/**
 What the refusal says when one sheet of the two judges' wide arm carried the
 window, which is half of what the arm owed.
 */
const WINDOW_NOT_SEEN = 'WindowEvidenceError: 1 of the judge sheets carried SURROUNDING ORIGINAL where 2 should have. The wide arm '
  + 'is the only thing this trial varies, so if it did not carry the window then all three arms saw the same evidence and '
  + 'every row bought after this point would report a false null. Nothing has been spent beyond the first slice.';

/**
 Sheet a judge of a narrow arm received.
 */
const NARROW_SHEET = 'the passage';

await describe({
  name: checkWindowReached.name,
  children: [
    it({
      name: 'WAITS for a wide arm: a slice that bought only narrow arms leaves the check unpassed and says nothing',
      fn: async () => {
        const { logger, lines, } = capturingLoggerPair();

        expect(checkWindowReached({
          check: WINDOW_UNCHECKED,
          rows: [
            armRow({ arm: 'narrow-a', shipped: false, },),
            armRow({ arm: 'narrow-b', shipped: false, },),
          ],
          sheets: [NARROW_SHEET,],
          judges: 2,
          l: logger,
        },),).toEqual({
          wideArms: 0,
          passed: false,
        },);
        expect(lines,).toEqual([],);
      },
    },),
    it({
      name: 'PASSES on the first wide arm when every judge saw the window, and says so',
      fn: async () => {
        const { logger, lines, } = capturingLoggerPair();

        expect(checkWindowReached({
          check: WINDOW_UNCHECKED,
          rows: [armRow({ arm: 'wide', shipped: false, },),],
          sheets: [
            NARROW_SHEET,
            WIDE_SHEET,
            WIDE_SHEET,
          ],
          judges: 2,
          l: logger,
        },),).toEqual({
          wideArms: 1,
          passed: true,
        },);
        expect(lines,).toEqual(['the window reached every judge of the first wide arm',],);
      },
    },),
    it({
      name: 'COUNTS the wide arms already bought toward what the judges should have seen',
      fn: async () => {
        const { logger, } = capturingLoggerPair();

        expect(checkWindowReached({
          check: {
            wideArms: 1,
            passed: false,
          },
          rows: [armRow({ arm: 'wide', shipped: true, },),],
          sheets: [
            WIDE_SHEET,
            WIDE_SHEET,
            WIDE_SHEET,
            WIDE_SHEET,
          ],
          judges: 2,
          l: logger,
        },),).toEqual({
          wideArms: 2,
          passed: true,
        },);
      },
    },),
    it({
      name: 'REFUSES a run in which a judge of the first wide arm never saw the window, naming both counts',
      fn: async () => {
        const { logger, lines, } = capturingLoggerPair();

        /**
         What the check threw.
         */
        const refusal = caught(function act(): unknown {
          return checkWindowReached({
            check: WINDOW_UNCHECKED,
            rows: [armRow({ arm: 'wide', shipped: false, },),],
            sheets: [
              NARROW_SHEET,
              WIDE_SHEET,
            ],
            judges: 2,
            l: logger,
          },);
        },);

        expect(refusal,).toBeInstanceOf(WindowEvidenceError,);
        expect(String(refusal,),).toBe(WINDOW_NOT_SEEN,);
        expect(lines,).toEqual([],);
      },
    },),
    it({
      name: 'RETURNS the state it was given once the check has passed, reading nothing further',
      fn: async () => {
        const { logger, lines, } = capturingLoggerPair();
        /**
         State after a pass.
         */
        const passed = {
          wideArms: 1,
          passed: true,
        };

        expect(checkWindowReached({
          check: passed,
          rows: [armRow({ arm: 'wide', shipped: false, },),],
          sheets: [],
          judges: 2,
          l: logger,
        },),).toEqual(passed,);
        expect(lines,).toEqual([],);
      },
    },),
  ],
},);
