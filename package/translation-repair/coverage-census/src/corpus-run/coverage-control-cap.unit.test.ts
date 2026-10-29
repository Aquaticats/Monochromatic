/**
 Tests that the coverage control STOPS AFTER ITS THIRD USABLE CASE.

 WHAT THE CONTROL BUYS. Each case it takes costs three rounds of the whole
 roster: the passage as it stands, the passage with its rendering cut out, and
 an equally large cut taken somewhere else as a decoy. Three cases is the
 declared size of the control, and the cap is what keeps a caller handing it a
 long list from paying for the whole list.

 WHAT WAS MEASURED. On 2026-08-25, the cap failed no test in this package.
 Nothing throws when it is wrong and nothing looks broken; the control simply
 spends more than it said it would, and the majority it reports is taken over
 a different denominator than the one its own rules describe.

 COUNTED ON ROWS, NOT ON CASES, which is the part worth pinning. A case the
 wire already declines to call covered is reported as a refusal and does NOT
 count against the cap, because it never had its rendering damaged and so
 showed nothing about whether damage is noticed. Five cases go in here and all
 five are damageable, so the rows are the cap exactly.

 NO NETWORK. One scripted answer serves every round: full coverage, quoting a
 sentence that really is in the translation, which is what makes the standing
 verdict `carried` and its evidence locatable.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { coverageControlHolds, } from '../../dist/final/node/index.mjs';
import {
  coverageControlCasesAt,
  coverageControlClient,
} from './coverage-control-cases.test-fixture.ts';
import {
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Logger for the control under test.
 */
const l = tagged({ tag: 'coverage-control-cap-test', },);

//region Fixtures

/**
 Sentence every scripted judge quotes, verbatim from `TRANSLATION`, so
 the standing verdict carries and its evidence can be located and cut.
 */
const QUOTED = 'Whiskers counts the birds outside.';

/**
 Translation the cases ask about, long enough on both sides of the quoted
 sentence that an equally large decoy cut has somewhere to go.
 */
const TRANSLATION = `The kitten dozes on the windowsill in the afternoon sun. ${QUOTED} `
  + 'The tabby sleeps by the radiator until evening.';

/**
 Original passage the cases ask about.
 */
const SOURCE_PASSAGE = '白胡子数着外面的鸟。';

/**
 Roster asked at every round.
 */
const MODEL_IDS = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
] as const;

/**
 Names of the five cases handed in, in the order they are offered.
 */
const WHERE = [
  'slice-0',
  'slice-1',
  'slice-2',
  'slice-3',
  'slice-4',
] as const;

/**
 Five identical damageable cases, distinguished only by where they sit.
 */
const CASES = coverageControlCasesAt({ where: WHERE, sourcePassage: SOURCE_PASSAGE, translationText: TRANSLATION, },);

/**
 Client answering every coverage round with full coverage and a real quote.
 */
const CLIENT = coverageControlClient({ quote: QUOTED, },);

//endregion Fixtures

await describe({
  name: coverageControlHolds.name,
  children: [
    it({
      name: 'TAKES THREE CASES AND STOPS, since each one it takes costs three rounds of the whole '
        + 'roster and the majority it reports is taken over the rows it actually damaged',
      fn: async () => {
        /**
         What the control made of five damageable cases.
         */
        const control = await coverageControlHolds({
          client: CLIENT,
          cases: CASES,
          modelIds: [...MODEL_IDS,],
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          exchangeTimeoutMs: HANG_STOP_MS,
          l,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);

        expect(control.rows
          .map(function toWhere(row,): string {
            return row.where;
          },),).toStrictEqual([
            'slice-0',
            'slice-1',
            'slice-2',
          ],);

        // Every case here is damageable, so nothing was set aside as a refusal
        // and the rows ARE the cap rather than whatever survived a filter.
        expect(control.refusals,).toStrictEqual([],);
      },
    },),
  ],
},);
