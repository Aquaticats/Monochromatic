/**
 Tests for which two renderings count as the same wording on a slate.

 WHY THIS FILE EXISTS. The slate's key (`wordingKey`) trims the END of a
 rendering and the trailing spaces of blank lines and blank quote lines,
 reads prose quotes straight (ledger B24), and on a slice the line-structure
 rule does not govern folds a paragraph's soft line breaks (ledger B26), so
 two translators whose text differs only there share one candidate and one
 stake. Trimming the FRONT as well would look like the same tidying and is
 not: leading spaces open a code block, indent a list item, and continue a
 quotation, so two renderings differing there are two different pages.
 Measured on 2026-08-25, trimming both ends failed no test. Trailing spaces on
 a line carrying content are a Markdown hard break, which 65 of the pinned
 corpus's pages use, so those stay.

 READ THROUGH `buildTranslateCandidates` rather than at the key itself,
 because the collapse is only visible as what reaches the ballot: how many
 candidates a judge is offered, and how many stakes each carries. The same
 reading pins that a text showing nothing reaches no ballot at all (ledger
 B40).

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import type { HeardVoice, } from '@monochromatic-dev/translation-repair-stage';
import {
  buildTranslateCandidates,
  foldTranslatorVoices,
  type TranslateReportWire,
  UnfoldedTranslationError,
} from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

//region Fixtures

/**
 Two seated translators, in roster order.
 */
const TRANSLATORS = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
] as const satisfies readonly RosterModelId[];

/**
 Builds one heard reply.

 @param at - roster position that answered

 @param translation - wording it proposed

 @returns Voice shaped as the gather returns one

 @example
 ```ts
 const voice = voiceOf({ at: 0, translation: 'The cat naps.', },);
 ```
 */
function voiceOf(
  {
    at,
    translation,
  }: {
    readonly at: number;
    readonly translation: string;
  },
): HeardVoice<TranslateReportWire> {
  return {
    modelId: TRANSLATORS[at] ?? TRANSLATORS[0],
    value: { translation, },
  };
}

//endregion Fixtures

await describe({
  name: buildTranslateCandidates.name,
  children: [
    it({
      name: 'COLLAPSES two renderings that differ only after their last character, since a trailing '
        + 'newline is not a different translation and offering it twice would split one wording`s stake '
        + 'across two candidates',
      fn: async () => {
        const set = buildTranslateCandidates({
          voices: [
            voiceOf({
              at: 0,
              translation: 'The cat sleeps on the windowsill.',
            },),
            voiceOf({
              at: 1,
              translation: 'The cat sleeps on the windowsill.\n\n',
            },),
          ],
          translatorModelIds: [...TRANSLATORS,],
          incumbentText: '',
          lineStructured: false,
        },);
        expect(set.candidates,).toHaveLength(1,);
        expect(set.collapsed,).toBe(1,);
      },
    },),
    it({
      name: 'KEEPS two renderings that differ only BEFORE their first character apart, because leading '
        + 'spaces open a code block, indent a list item and continue a quotation. A key trimming both '
        + 'ends would merge two different pages and hand a judge one of them',
      fn: async () => {
        const set = buildTranslateCandidates({
          voices: [
            voiceOf({
              at: 0,
              translation: 'The cat sleeps on the windowsill.',
            },),
            voiceOf({
              at: 1,
              translation: '    The cat sleeps on the windowsill.',
            },),
          ],
          translatorModelIds: [...TRANSLATORS,],
          incumbentText: '',
          lineStructured: false,
        },);
        expect(set.candidates,).toHaveLength(2,);
        expect(set.collapsed,).toBe(0,);
        // The indented one is still on the ballot with its own bytes, which is
        // what a judge needs to see to reject it.
        expect(set.candidates
          .map(function toRendering(candidate,): string {
            return candidate.rendered;
          },),).toContain('    The cat sleeps on the windowsill.',);
      },
    },),

    it({
      name: 'COLLAPSES a rendering onto the incumbent when it differs only by a trailing space on a '
        + 'blank quote line (the Toka_ls slice 1 replacement of 2026-09-02, two bytes judged five to two), '
        + 'and KEEPS one apart that differs by a hard break on a content line',
      fn: async () => {
        /**
         Archive text with a clean blank quote line.
         */
        const incumbent = '> The cat sleeps.\n>\n> The cat wakes.';
        const set = buildTranslateCandidates({
          voices: [
            voiceOf({
              at: 0,
              translation: '> The cat sleeps.\n> \n> The cat wakes.',
            },),
            voiceOf({
              at: 1,
              translation: '> The cat sleeps.  \n>\n> The cat wakes.',
            },),
          ],
          translatorModelIds: [...TRANSLATORS,],
          incumbentText: incumbent,
          lineStructured: false,
        },);
        /**
         Renderings on the ballot.
         */
        const rendered = set.candidates
          .map(function toRendering(candidate,): string {
            return candidate.rendered;
          },);
        // The blank-quote variant is the incumbent's own bytes; the hard-break
        // variant is a different page and stays on the ballot as itself.
        expect(rendered,).toContain(incumbent,);
        expect(rendered,).toContain('> The cat sleeps.  \n>\n> The cat wakes.',);
        expect(set.candidates,).toHaveLength(2,);
        expect(set.findings,).toContain(`translate-matched-incumbent (${TRANSLATORS[0] ?? ''})`,);
      },
    },),

    it({
      name: 'COLLAPSES a rendering onto the incumbent when it differs only in the style of its prose apostrophes '
        + 'and quotation marks, which the typography restoration makes one after the ballot (ledger B24), '
        + 'and KEEPS one apart whose quotes differ inside a code span, where a quote is content',
      fn: async () => {
        /**
         Archive text with curly quotes in its prose.
         */
        const incumbent = 'The cat didn’t say “meow” today.';
        const prose = buildTranslateCandidates({
          voices: [
            voiceOf({
              at: 0,
              translation: 'The cat didn\'t say "meow" today.',
            },),
            voiceOf({
              at: 1,
              translation: 'The cat didn’t say "meow" today.',
            },),
          ],
          translatorModelIds: [...TRANSLATORS,],
          incumbentText: incumbent,
          lineStructured: false,
        },);
        expect(prose.candidates
          .map(function toRendering(candidate,): string {
            return candidate.rendered;
          },),).toEqual([incumbent,],);
        expect(prose.collapsed,).toBe(2,);
        /**
         Two renderings apart only inside a code span.
         */
        const code = buildTranslateCandidates({
          voices: [
            voiceOf({
              at: 0,
              translation: 'Type `say "meow"` to wake the cat.',
            },),
            voiceOf({
              at: 1,
              translation: 'Type `say “meow”` to wake the cat.',
            },),
          ],
          translatorModelIds: [...TRANSLATORS,],
          incumbentText: '',
          lineStructured: false,
        },);
        expect(code.candidates,).toHaveLength(2,);
      },
    },),

    it({
      name: 'COLLAPSES a rendering onto the incumbent when it differs only in where a paragraph\'s soft line '
        + 'breaks fall, which the site renders as spaces (ledger B26), and KEEPS apart one that adds a hard '
        + 'break, one that splits the paragraph and one that changes a word, each a different page',
      fn: async () => {
        /**
         Archive text with a soft line break inside its one paragraph.
         */
        const incumbent = 'The cat naps on the mat\nall afternoon.';
        /**
         Slate offered one rendering at a time beside the incumbent.

         @param translation - rendering a translator proposed

         @returns Renderings the ballot carries

         @example
         ```ts
         const rendered = renderedBeside({ translation: 'The cat naps.', },);
         ```
         */
        function renderedBeside({ translation, }: { readonly translation: string; },): readonly string[] {
          return buildTranslateCandidates({
            voices: [voiceOf({
              at: 0,
              translation,
            },),],
            translatorModelIds: [...TRANSLATORS,],
            incumbentText: incumbent,
            lineStructured: false,
          },)
            .candidates
            .map(function toRendering(candidate,): string {
              return candidate.rendered;
            },);
        }
        expect(renderedBeside({ translation: 'The cat naps on the mat all afternoon.', },),).toEqual([incumbent,],);
        expect(renderedBeside({ translation: 'The cat naps on the mat  \nall afternoon.', },),).toHaveLength(2,);
        expect(renderedBeside({ translation: 'The cat naps on the mat\n\nall afternoon.', },),).toHaveLength(2,);
        expect(renderedBeside({ translation: 'The cat naps on the rug\nall afternoon.', },),).toHaveLength(2,);
      },
    },),

    it({
      name: 'COLLAPSES a rendering the intake fold made plain into the plain one that says the same, '
        + 'and REFUSES a voice that skipped the fold, since every check before the slate then read '
        + 'bytes that will not ship (ledger B112)',
      fn: async () => {
        /**
         Both replies as the lane holds them after its intake fold.
         */
        const intake = foldTranslatorVoices({
          voices: [
            voiceOf({
              at: 0,
              translation: 'A part\u2011time shop cat.',
            },),
            voiceOf({
              at: 1,
              translation: 'A part-time shop cat.',
            },),
          ],
        },);
        const set = buildTranslateCandidates({
          voices: intake.voices,
          translatorModelIds: TRANSLATORS,
          incumbentText: '',
          lineStructured: false,
        },);

        expect(set.candidates,).toHaveLength(1,);
        expect(set.candidates[0]?.rendered,).toBe('A part-time shop cat.',);
        expect(set.findings,).toStrictEqual([],);

        /**
         What building a slate from the unfolded reply threw.
         */
        const refusal = caught(function buildUnfolded(): unknown {
          return buildTranslateCandidates({
            voices: [
              voiceOf({
                at: 1,
                translation: 'A part-time shop cat.',
              },),
              voiceOf({
                at: 0,
                translation: 'A part\u2011time shop cat.',
              },),
            ],
            translatorModelIds: TRANSLATORS,
            incumbentText: '',
            lineStructured: false,
          },);
        },);
        expect(refusal,).toBeInstanceOf(UnfoldedTranslationError,);
        expect((refusal instanceof UnfoldedTranslationError) ? refusal.modelIds : [],).toStrictEqual([
          TRANSLATORS[0],
        ],);
      },
    },),

    it({
      name: 'OFFERS NO CANDIDATE for a text that shows nothing (ledger B40): a reply of a zero-width space, '
        + 'which the intake fold empties, one of a Hangul filler, which no fold touches, and an incumbent '
        + 'or lane text of invisible characters alone, naming each blank reply with its author',
      fn: async () => {
        const set = buildTranslateCandidates({
          voices: foldTranslatorVoices({
            voices: [
              voiceOf({
                at: 0,
                translation: '\u{200B}',
              },),
              voiceOf({
                at: 1,
                translation: '\u{3164}',
              },),
            ],
          },).voices,
          translatorModelIds: TRANSLATORS,
          incumbentText: '\u{2060}',
          lineStructured: false,
          laneTexts: [{
            lane: 'repair',
            text: '\u{00AD}',
          },],
        },);

        expect(set.candidates,).toStrictEqual([],);
        expect(set.findings,).toStrictEqual([
          `translate-blank (${TRANSLATORS[0] ?? ''})`,
          `translate-blank (${TRANSLATORS[1] ?? ''})`,
        ],);
      },
    },),
  ],
},);
