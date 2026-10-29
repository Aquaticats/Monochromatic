/**
 Tests the guard that admits a translator's reply as a heard voice.

 A REPLY THAT SHOWS NOTHING IS NOT A REPLY, and the guard is where it is
 refused, so the roster asks that model again and the loss is counted as one.
 It asked `trim()` until ledger B40, which keeps every invisible character
 that is not whitespace: a reply of one zero-width space passed, the intake
 fold then removed it, and an empty candidate reached the judges. Nothing in
 the package tested this guard directly before then.

 Every invisible fixture is spelled as an escape.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { isTranslateReportWire, } from '../dist/final/node/index.mjs';

await describe({
  name: isTranslateReportWire.name,
  children: [
    it({
      name: 'REFUSES a reply whose translation shows nothing: empty, whitespace, or only invisible characters '
        + 'trim() keeps, folded or not (ledger B40)',
      fn: async () => {
        expect([
          '',
          ' \n\t ',
          '\u{3000}',
          '\u{200B}',
          '\u{2060}\u{00AD}',
          '\u{3164}',
          ' \u{E0001} ',
        ].map(function admitted(translation,): boolean {
          return isTranslateReportWire({ translation, },);
        },),).toEqual([
          false,
          false,
          false,
          false,
          false,
          false,
          false,
        ],);
      },
    },),
    it({
      name: 'REFUSES a reply that is not a record or whose translation is not a string',
      fn: async () => {
        expect(isTranslateReportWire('The cat naps.',),).toBe(false,);
        expect(isTranslateReportWire(null,),).toBe(false,);
        expect(isTranslateReportWire([{ translation: 'The cat naps.', },],),).toBe(false,);
        expect(isTranslateReportWire({ translation: 2, },),).toBe(false,);
        expect(isTranslateReportWire({},),).toBe(false,);
      },
    },),
    it({
      name: 'ADMITS a reply with wording, and one whose wording carries an invisible character among visible '
        + 'ones, which the intake fold tidies rather than refuses',
      fn: async () => {
        expect(isTranslateReportWire({ translation: 'The cat naps.', },),).toBe(true,);
        expect(isTranslateReportWire({ translation: 'The cat\u{200B} naps.', },),).toBe(true,);
      },
    },),
  ],
},);
