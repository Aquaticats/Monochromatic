/**
 Tests for the footnote guard's rule that a slice is withdrawn only when the withdrawal provably clears the finding on
 the page as it will be written, that a missing reference is blamed on the slice that lost it, and that where neither
 holds nothing is withdrawn and the orphan is reported (lead's adoption of 2026-10-29, open to the owner's veto).

 XingZ6022 (2026-10-29): slice 44 kept an archive text that had dropped the reference `[^3]`, the translate lane's
 text for slice 115 (an anchor the archive never translated, whose original holds definition 3) carried the
 definition, and the guard withdrew slice 115 for the orphan. The withdrawal could not clear it: an untranslated
 anchor ships its original under a note, and that original carries definition 3, so a good translation was lost and
 the orphan stayed.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { makeInsertionChunk, } from '@monochromatic-dev/translation-repair-shared-contract';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading';
import { guardFootnoteAssembly, } from '../dist/final/node/index.mjs';

/**
 Archive text of the first slice, which kept its reference.
 */
const WITH_REFERENCE = 'The cat naps on the windowsill[^3].\n';

/**
 Archive text of the first slice, which lost its reference.
 */
const WITHOUT_REFERENCE = 'The cat naps on the windowsill.\n';

/**
 Definition the translate lane wrote for the anchor.
 */
const DEFINITION = '[^3]: That is its favourite spot.\n';

/**
 Builds the two slices: a content slice the archive translated and the anchor where the notes belong.

 @param archiveText - the first slice's archive text, which is also the whole document
 @param anchorSource - original of the anchor, whose definition the page ships when no wording fills it

 @returns Slices in document order

 @example
 ```ts
 const slices = slicesOf({ archiveText: WITHOUT_REFERENCE, anchorSource: '[^3]: 那是它最喜欢的位置。', },);
 ```
 */
function slicesOf(
  {
    archiveText,
    anchorSource,
  }: {
    readonly archiveText: string;
    readonly anchorSource: string;
  },
): readonly ChunkPair[] {
  return [
    {
      source: {
        kind: 'content',
        sliceIndex: 0,
        nodes: [],
        startOffset: 0,
        endOffset: 12,
        text: '猫猫在窗台上打盹[^3]。',
      },
      target: {
        kind: 'content',
        sliceIndex: 0,
        nodes: [],
        startOffset: 0,
        endOffset: archiveText.length,
        text: archiveText,
      },
    },
    {
      source: {
        kind: 'content',
        sliceIndex: 1,
        nodes: [],
        startOffset: 14,
        endOffset: 14 + anchorSource.length,
        text: anchorSource,
      },
      target: makeInsertionChunk({
        sliceIndex: 1,
        offset: archiveText.length,
      },),
    },
  ];
}

await describe({
  name: guardFootnoteAssembly.name,
  children: [
    it({
      name: 'WITHDRAWS NOTHING AND REPORTS THE ORPHAN where the definition sits in an anchor whose original carries '
        + 'it, since the page ships that original when the translation is withdrawn and the orphan stays (XingZ6022 '
        + 'slice 115, over the reference slice 44\'s archive had dropped)',
      fn: async () => {
        const guarded = guardFootnoteAssembly({
          targetText: WITHOUT_REFERENCE,
          slices: slicesOf({
            archiveText: WITHOUT_REFERENCE,
            anchorSource: '[^3]: 那是它最喜欢的位置。',
          },),
          replacements: [{
            sliceIndex: 1,
            replacementText: DEFINITION,
          },],
        },);
        expect(guarded.revertedChunkIndices,).toEqual([],);
        expect(guarded.replacements,).toEqual([{
          sliceIndex: 1,
          replacementText: DEFINITION,
        },],);
        expect(guarded.assembledText,).toContain(DEFINITION.trim(),);
        expect(guarded.findings.some(function reportsIt(finding,): boolean {
          return finding.startsWith('assembly-footnote-reported orphan-definition gfm 3 (round 1)',);
        },),).toBe(true,);
      },
    },),
    it({
      name: 'STILL WITHDRAWS the definition where the anchor\'s original carries none, since the withdrawal then '
        + 'clears the orphan on the page as it will be written',
      fn: async () => {
        const guarded = guardFootnoteAssembly({
          targetText: WITHOUT_REFERENCE,
          slices: slicesOf({
            archiveText: WITHOUT_REFERENCE,
            anchorSource: '一只麻雀。',
          },),
          replacements: [{
            sliceIndex: 1,
            replacementText: DEFINITION,
          },],
        },);
        expect(guarded.revertedChunkIndices,).toEqual([1,],);
        expect(guarded.replacements,).toEqual([],);
        expect(guarded.findings.some(function revertsIt(finding,): boolean {
          return finding.startsWith('assembly-footnote-reverted orphan-definition gfm 3',);
        },),).toBe(true,);
      },
    },),
    it({
      name: 'BLAMES THE SLICE THAT LOST THE REFERENCE and keeps the definition beside it, since withdrawing that '
        + 'slice clears the orphan and the definition\'s anchor withdrawn would ship the same definition in the '
        + 'original',
      fn: async () => {
        const guarded = guardFootnoteAssembly({
          targetText: WITH_REFERENCE,
          slices: slicesOf({
            archiveText: WITH_REFERENCE,
            anchorSource: '[^3]: 那是它最喜欢的位置。',
          },),
          replacements: [
            {
              sliceIndex: 0,
              replacementText: WITHOUT_REFERENCE,
            },
            {
              sliceIndex: 1,
              replacementText: DEFINITION,
            },
          ],
        },);
        expect(guarded.revertedChunkIndices,).toEqual([0,],);
        expect(guarded.replacements,).toEqual([{
          sliceIndex: 1,
          replacementText: DEFINITION,
        },],);
        expect(guarded.assembledText,).toContain('windowsill[^3].',);
        expect(guarded.assembledText,).toContain(DEFINITION.trim(),);
      },
    },),
  ],
},);
