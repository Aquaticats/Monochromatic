/**
 Guards class one hundred five (CuspariaKLSY9, 2026-09-23): the archive's
 gloss line of a declared name, "“Ling Shui Yu Yu Zi” means fish in clear
 water.", the Chinese silent about it, left the page for the third time on
 the judges' call although the eighty-fifth class's clause stood on every
 sheet. The line is restored after the judges, where the page is in view.
 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { SliceNotOnPageError, } from '@monochromatic-dev/translation-repair-page-pass';
import { restoreNameGlossLines, } from '../../dist/final/node/index.mjs';
import { pair, } from '@monochromatic-dev/translation-repair-slice-reading/ts/corpus-run/archive-slice-pair.test-fixture.ts';

/**
 The archive's gloss line.
 */
const GLOSS = '“Mittens the Cloud” means a cat like a cloud.';

/**
 Archive paragraph: the nickname sentence and its gloss on the next line.
 */
const ARCHIVE = `She got her nickname “Mittens the Cloud” while napping.\n${GLOSS}\n`;

await describe({
  name: `${restoreNameGlossLines.name} (class one hundred five)`,
  children: [
    it({
      name: 'INSERTS the archive\'s gloss line after the shipped line carrying the name, in curly or straight quotes',
      fn: async () => {
        const restored = restoreNameGlossLines({
          slices: [pair({ sliceIndex: 0, target: ARCHIVE, },), pair({ sliceIndex: 1, target: 'The sparrow "Pip" sang.\n"Pip" means a seed.\n', },),],
          replacements: [
            {
              sliceIndex: 0,
              replacementText: 'Her handle “Mittens the Cloud” was coined while she napped.\n\nShe slept on.',
            },
            {
              sliceIndex: 1,
              replacementText: 'The sparrow "Pip" was singing.',
            },
          ],
        },);
        expect(restored.replacements.map(function textOf(row,): string {
          return row.replacementText;
        },),).toEqual([
          `Her handle “Mittens the Cloud” was coined while she napped.\n${GLOSS}\n\nShe slept on.`,
          'The sparrow "Pip" was singing.\n"Pip" means a seed.',
        ],);
        expect(restored.restored.length,).toBe(2,);
        expect(restored.findings,).toEqual([
          `name-gloss-restored (slice 0: "${GLOSS}")`,
          'name-gloss-restored (slice 1: ""Pip" means a seed.")',
        ],);
      },
    },),
    it({
      name: 'REFUSES a replacement for a slice the pairs lack rather than reading the archive as empty text',
      fn: async () => {
        const refusal = caught(function act(): unknown {
          return restoreNameGlossLines({
            slices: [pair({ sliceIndex: 0, target: ARCHIVE, },),],
            replacements: [{ sliceIndex: 7, replacementText: 'Her handle was coined while she napped.', },],
          },);
        },);
        expect(refusal,).toBeInstanceOf(SliceNotOnPageError,);
        /**
         The refusal the page text gives for that slice.
         */
        const expected = new SliceNotOnPageError({ sliceIndex: 7, },);
        expect(String(refusal,),).toBe(String(expected,),);
      },
    },),
    it({
      name: 'READS THE NAME AS A WORD (ledger B23): a text carrying it only inside a longer word gets no gloss, one '
        + 'carrying it as a word later gets the gloss after that line, and an archive using it only inside a longer '
        + 'word glosses no name of its own',
      fn: async () => {
        /**
         Archive paragraph naming the sparrow, with its gloss.
         */
        const sparrow = 'The sparrow "Pip" sang.\n"Pip" means a seed.\n';
        const restored = restoreNameGlossLines({
          slices: [
            pair({ sliceIndex: 0, target: sparrow, },),
            pair({ sliceIndex: 1, target: sparrow, },),
            pair({ sliceIndex: 2, target: 'Pipits sang.\n"Pip" means a seed.\n', },),
          ],
          replacements: [
            { sliceIndex: 0, replacementText: 'The sparrow Pipit sang.', },
            { sliceIndex: 1, replacementText: 'Pipits sang.\nThe sparrow Pip joined them.', },
            { sliceIndex: 2, replacementText: 'The sparrow Pip sang.', },
          ],
        },);
        expect(restored.replacements.map(function textOf(row,): string {
          return row.replacementText;
        },),).toEqual([
          'The sparrow Pipit sang.',
          'Pipits sang.\nThe sparrow Pip joined them.\n"Pip" means a seed.',
          'The sparrow Pip sang.',
        ],);
      },
    },),
    it({
      name: 'LEAVES a text glossing the name its own way (a parenthesis, "means" on the line, the archive\'s line kept), '
        + 'one that never carries the name, and an archive quoting a phrase it never uses',
      fn: async () => {
        const restored = restoreNameGlossLines({
          slices: [
            pair({ sliceIndex: 0, target: ARCHIVE, },),
            pair({ sliceIndex: 1, target: ARCHIVE, },),
            pair({ sliceIndex: 2, target: ARCHIVE, },),
            pair({ sliceIndex: 3, target: ARCHIVE, },),
            pair({ sliceIndex: 4, target: 'She said hello.\n“Hello” means a greeting.\n', },),
          ],
          replacements: [
            { sliceIndex: 0, replacementText: 'Her handle, Mittens the Cloud (a cat like a cloud), was coined while she napped.', },
            { sliceIndex: 1, replacementText: 'Her handle “Mittens the Cloud” means a cat like a cloud and was coined while she napped.', },
            { sliceIndex: 2, replacementText: `She got her nickname “Mittens the Cloud” while napping.\n${GLOSS}`, },
            { sliceIndex: 3, replacementText: 'Her handle was coined while she napped.', },
            { sliceIndex: 4, replacementText: 'She said hi.', },
          ],
        },);
        expect(restored.restored,).toEqual([],);
        expect(restored.findings,).toEqual([],);
      },
    },),

    it({
      name: 'RESTORES NO GLOSS from a line whose quote never closes or whose name is not followed by the '
        + 'gloss verb, where the same archive with a well-formed gloss line restores it',
      fn: async () => {
        /**
         Shipped text carrying the name once, with no gloss of its own.
         */
        const shipped = 'Her handle “Mittens” was coined while she napped.';
        /**
         Archive sentence carrying the name once; the line after it decides.
         */
        const sentence = 'She got her nickname “Mittens” while napping.';
        /**
         Restoration of the archive whose second line is the one given.
         */
        function restoreWith(
          { glossLine, }: { readonly glossLine: string; },
        ): ReturnType<typeof restoreNameGlossLines> {
          return restoreNameGlossLines({
            slices: [pair({ sliceIndex: 0, target: `${sentence}\n${glossLine}\n`, },),],
            replacements: [{ sliceIndex: 0, replacementText: shipped, },],
          },);
        }
        const unclosed = restoreWith({ glossLine: '“Mittens means a cloud.', },);
        expect(unclosed.restored,).toEqual([],);
        expect(unclosed.findings,).toEqual([],);
        const unverbed = restoreWith({ glossLine: '“Mittens” sleeps in the sun.', },);
        expect(unverbed.restored,).toEqual([],);
        expect(unverbed.findings,).toEqual([],);
        const control = restoreWith({ glossLine: '“Mittens” means a cloud.', },);
        expect(control.restored,).toEqual([
          { sliceIndex: 0, replacementText: `${shipped}\n“Mittens” means a cloud.`, },
        ],);
        expect(control.findings,).toEqual(['name-gloss-restored (slice 0: "“Mittens” means a cloud.")',],);
      },
    },),

    it({
      name: 'READS THE NAME AS GLOSSED where a comma stands between it and its parenthesis, and restores the '
        + 'gloss line where no parenthesis follows',
      fn: async () => {
        /**
         Archive carrying a well-formed gloss line of the name.
         */
        const archive = 'Her handle “Mittens” was coined.\n“Mittens” means a cloud.\n';
        const commaThenParenthesis = restoreNameGlossLines({
          slices: [pair({ sliceIndex: 0, target: archive, },),],
          replacements: [{
            sliceIndex: 0,
            replacementText: 'Her handle “Mittens”, (a cloud) was coined.',
          },],
        },);
        expect(commaThenParenthesis.restored,).toEqual([],);
        expect(commaThenParenthesis.findings,).toEqual([],);
        const commaOnly = restoreNameGlossLines({
          slices: [pair({ sliceIndex: 0, target: archive, },),],
          replacements: [{
            sliceIndex: 0,
            replacementText: 'Her handle “Mittens”, a cat, was coined.',
          },],
        },);
        expect(commaOnly.restored,).toEqual([
          { sliceIndex: 0, replacementText: 'Her handle “Mittens”, a cat, was coined.\n“Mittens” means a cloud.', },
        ],);
        expect(commaOnly.findings,).toEqual(['name-gloss-restored (slice 0: "“Mittens” means a cloud.")',],);
      },
    },),
  ],
},);
