import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  coverageControlHolds,
  withoutSpans,
} from '../../dist/final/node/index.mjs';
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

//region Coverage control cut
// What the absence control actually deletes.
//
// The control asks the coverage roster about a passage, deletes the spans it
// anchored on, and asks again. Everything the control concludes rests on the
// deletion being real: a cut that returned the document unchanged would let a
// wire that cannot see absence pass, and a cut that returned blank on
// everything would report a working wire as unmeasurable.
//
// Fixtures are invented cat prose, never corpus text.

await describe({
  name: withoutSpans.name,
  children: [
    it({
      name: 'REMOVES ONE ANCHORED SPAN and leaves the rest of the document standing, which is the '
        + 'ordinary case the control is built on',
      fn: async function removesOneSpan() {
        expect(
          withoutSpans({
            text: 'The tabby slept. The grey cat waited.',
            spans: ['The tabby slept. ',],
          },),
        ).toBe('The grey cat waited.',);
      },
    },),

    it({
      name: 'REMOVES EVERY OCCURRENCE of a span rather than the first, because a rendering that '
        + 'survives anywhere in the document is a rendering the roster can still anchor on',
      fn: async function removesEveryOccurrence() {
        expect(
          withoutSpans({
            text: 'a cat, a dog, a cat',
            spans: ['a cat',],
          },),
        ).toBe(', a dog, ',);
      },
    },),

    it({
      name: 'CUTS THE LONGEST SPAN FIRST so a span nested inside another is gone by its own turn '
        + 'rather than by accident, which keeps the result independent of roster answer order',
      fn: async function longestSpanGoesFirst() {
        // `waited` sits inside `The grey cat waited`, so a shortest-first cut
        // would leave `The grey cat ` behind and the two orders would disagree.
        expect(
          withoutSpans({
            text: 'The tabby slept. The grey cat waited.',
            spans: ['waited', 'The grey cat waited',],
          },),
        ).toBe('The tabby slept. .',);
      },
    },),

    it({
      name: 'REFUSES A DOCUMENT NO SPAN APPEARS IN by returning blank, since re-asking about text '
        + 'nothing was removed from would measure the provider rather than the wire',
      fn: async function absentSpanIsRefused() {
        expect(
          withoutSpans({
            text: 'The tabby slept.',
            spans: ['a sentence this document does not hold',],
          },),
        ).toBe('',);
      },
    },),

    it({
      name: 'REFUSES AN EMPTY SPAN LIST rather than reporting the document as damaged, which is '
        + 'the shape a roster that anchored nothing would hand it',
      fn: async function noSpansIsRefused() {
        expect(
          withoutSpans({
            text: 'The tabby slept.',
            spans: [],
          },),
        ).toBe('',);
      },
    },),

    it({
      name: 'REFUSES A BLANK SPAN instead of splitting the document on it, because a blank quote '
        + 'removes nothing and must not be counted as a rendering that was deleted',
      fn: async function blankSpanIsRefused() {
        expect(
          withoutSpans({
            text: 'The tabby slept.',
            spans: ['',],
          },),
        ).toBe('',);
      },
    },),

    it({
      name: 'REFUSES A SPAN THAT SHOWS A READER NOTHING as it refuses an empty one, cutting none of the spaces, '
        + 'zero-width spaces or Hangul fillers the document holds',
      fn: async function invisibleSpanIsRefused() {
        expect([
          ' ',
          '\u{200B}',
          '\u{3164}',
        ].map(function cut(span,): string {
          return withoutSpans({
            text: 'The tabby\u{200B} slept\u{3164}.',
            spans: [span,],
          },);
        },),).toEqual([
          '',
          '',
          '',
        ],);
      },
    },),

    it({
      name: 'REFUSES A CASE WHOSE CUT LEAVES ONLY WHAT SHOWS A READER NOTHING as a cut that left nothing, asking '
        + 'the roster no second round about a page of one line break',
      fn: async function blankRemainderIsRefused() {
        /**
         Sentence every scripted judge quotes, the translation's whole wording.
         */
        const quoted = 'Whiskers counts the birds outside.';
        expect(await coverageControlHolds({
          client: coverageControlClient({ quote: quoted, },),
          cases: coverageControlCasesAt({
            where: ['slice-0',],
            sourcePassage: '白胡子数着外面的鸟。',
            translationText: `${quoted}\n`,
          },),
          modelIds: [
            SEAT_HYPER_OPENROUTER_VISION_EDITOR,
            SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
            SEAT_SYNTHETIC_VISION_WITHHELD,
          ],
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          exchangeTimeoutMs: HANG_STOP_MS,
          l: tagged({ tag: 'coverage-control-test', },),
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },),).toEqual({
          held: false,
          sawAbsenceOnTarget: 0,
          sawAbsenceOnDecoy: 0,
          decoysTaken: 0,
          rows: [],
          refusals: [{
            where: 'slice-0',
            reason: 'cut-left-nothing',
            verdict: 'carried',
            absent: 0,
            offeredSpans: 3,
          },],
        },);
      },
    },),
  ],
},);

//endregion Coverage control cut
