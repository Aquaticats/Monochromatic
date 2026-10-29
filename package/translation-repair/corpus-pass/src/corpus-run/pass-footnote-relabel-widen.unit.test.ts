/**
 Tests for the relabel reading widening to the paired slices where the
 definitions the roster paired do not close.

 CLASS NINETY-FIVE (hulicaijia14, 2026-09-23): the roster paired 14 of 15
 blocks and left one original definition unpaired, so the map read off the
 definitions alone landed on an archive label while one original label stood
 unaccounted for, the archive's labels stood, the lanes wrote the original's
 labels into a page keyed by the archive's, and the assembly trimmed the
 notes as orphans. Cat-themed invention; no corpus content appears here.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication';
import { relabelArchiveFootnotes, } from '../../dist/final/node/index.mjs';

//region Fixtures

/**
 Original: two notes, the cat's spot first, the sparrow second.
 */
const SOURCE_TEXT = '## 生活\n\n猫猫[^1]睡在窗台上。\n\n鸟儿[^2]在外面唱歌。\n\n'
  + '[^1]: 它最喜欢的位置。\n\n[^2]: 一只麻雀。\n';

/**
 Archive: a translator's own note takes `[^1]`, so the original's two notes
 sit at `[^2]` and `[^3]`, and the translator's marker rides the second
 paragraph beside the sparrow's.
 */
const TARGET_TEXT = '## Life\n\nThe cat[^2] slept on the sill.\n\nThe bird[^3] sang outside.[^1]\n\n'
  + '[^1]: Translator\'s note: the sill faces east.\n\n[^2]: Her favourite spot.\n\n[^3]: A sparrow.\n';

/**
 Slice budget at which each paragraph is its own slice, so the body
 paragraphs pair one to one.
 */
const SLICE_BUDGET = 40;

/**
 Test logger.
 */
const l = tagged({ tag: 'footnote-relabel-widen-test', },);

//endregion Fixtures

await describe({
  name: `${relabelArchiveFootnotes.name} (class ninety-five)`,
  children: [
    it({
      name: 'READS THE PAIRED SLICES BESIDE THE DEFINITIONS where the roster paired one definition short and the '
        + 'map read off the definitions alone does not close (hulicaijia14: 14 of 15 blocks paired, the archive '
        + 'labels stood and the lanes wrote the original\'s)',
      fn: async () => {
        /**
         Preparation over the fixture pair, whose first paragraph pairs the
         cat's `[^2]` with the original's `[^1]`.
         */
        const prepared = prepareDocumentPair({
          sourceText: SOURCE_TEXT,
          targetText: TARGET_TEXT,
          sliceCharBudget: SLICE_BUDGET,
        },);
        /**
         The roster paired the sparrow's definitions alone.
         */
        const result = relabelArchiveFootnotes({
          entryId: 'invented-widen',
          slices: prepared.slices,
          definitionPairs: [ {
            sourceLabel: '2',
            targetLabel: '3',
          }, ],
          sourceText: SOURCE_TEXT,
          archiveText: TARGET_TEXT,
          l,
        },);
        expect(result.withheld,).toBeUndefined();
        expect(result.changed,).toBe(true,);
        expect(result.archiveText,).toContain('The cat[^1] slept',);
        expect(result.archiveText,).toContain('The bird[^2] sang',);
        expect(result.archiveText,).toContain('[^1]: Her favourite spot.',);
        expect(result.archiveText,).toContain('[^2]: A sparrow.',);
        expect(result.archiveText,).not.toContain('[^3]',);
        expect(result.findings.some(function widened(finding,): boolean {
          return finding.includes('the paired slices',);
        },),).toBe(true,);
      },
    },),

    it({
      name: 'LEAVES THE ARCHIVE STANDING where the definitions and the slices together still do not close',
      fn: async () => {
        /**
         An archive whose second paragraph carries the original's note under
         one label and the first paragraph two notes, so no slice pairs the
         cat's note and the sparrow's definition alone cannot place it.
         */
        const archiveText = '## Life\n\nThe cat[^2] slept on the sill.[^1]\n\nThe bird[^3] sang outside.[^4]\n\n'
          + '[^1]: Translator\'s note.\n\n[^2]: Her favourite spot.\n\n[^3]: A sparrow.\n\n[^4]: Another note.\n';
        /**
         Preparation over the pair.
         */
        const prepared = prepareDocumentPair({
          sourceText: SOURCE_TEXT,
          targetText: archiveText,
          sliceCharBudget: SLICE_BUDGET,
        },);
        /**
         The roster paired the sparrow's definitions alone.
         */
        const result = relabelArchiveFootnotes({
          entryId: 'invented-widen-open',
          slices: prepared.slices,
          definitionPairs: [ {
            sourceLabel: '2',
            targetLabel: '3',
          }, ],
          sourceText: SOURCE_TEXT,
          archiveText,
          l,
        },);
        expect(result.withheld,).toBe('correspondence',);
        expect(result.changed,).toBe(false,);
        expect(result.archiveText,).toBe(archiveText,);
      },
    },),
  ],
},);
