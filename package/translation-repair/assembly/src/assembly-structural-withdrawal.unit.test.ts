/**
 Whole-document proof that one structural withdrawal preserves unrelated translations.
 Fixtures are cat-themed invention.

 @module
 */
import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import { spliceSlices, } from '@monochromatic-dev/translation-repair-page-pass';
import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication';
import {
  type ChunkPair,
  strictRefusalOffset,
} from '@monochromatic-dev/translation-repair-slice-reading';
import {
  advancingStructuralWithdrawal,
  guardFootnoteAssembly,
  singleStructuralWithdrawal,
} from '../dist/final/node/index.mjs';

/** Source and archive component shared before translation. */
const COMPONENT = `<PhotoScroll photos={['\${path}/photos/cat.webp']} />`;
/** Invalid component selected by a lane after its author's unresolved repair. */
const BROKEN_COMPONENT = `<PhotoScroll photos=['\${path}/photos/cat.webp']} />`;
/** Link carried by the source-only passage which an unrelated withdrawal must preserve. */
const LINK = 'https://example.test/cat';
/** Original of three sections, each its own slice. */
const SECTIONS_SOURCE = '## 猫\n\n猫猫在窗台上打盹〔1〕。\n\n## 鸟\n\n窗台上有一只鸟。\n\n## 注\n\n〔1〕：那是它最喜欢的位置。\n';
/** Archive of those sections. */
const SECTIONS_TARGET = '## The cat\n\nThe cat is doing the sleeping on the windowsill[^1].\n\n## The bird\n\n'
  + 'On the windowsill there is being a bird.\n\n## Notes\n\n[^1]: That is its favourite spot.\n';
/** Expression the bird's replacement leaves open, the first break on the page. */
const BIRD_BREAK = '{\'unclosed';
/**
 First section whole and much shorter than its archive text, the other two each breaking the grammar, so
 withdrawing the first lengthens the page before both breaks and moves neither.
 */
const SHIFTING = [
  { sliceIndex: 0, replacementText: '## The cat\n\nIt naps[^1].', },
  { sliceIndex: 1, replacementText: `## The bird\n\nOn the windowsill there sits a bird. ${BIRD_BREAK}`, },
  { sliceIndex: 2, replacementText: '## Notes {\'unclosed\n\n[^1]: That is its favourite spot.', },
];
/**
 First section leaves an element open to the end of the page, the second breaks an expression, the third is
 whole: the parser names the expression, and once it goes the open element stops the grammar at the page's end.
 */
const LEFT_OPEN = [
  { sliceIndex: 0, replacementText: '## The cat\n\n<div>\n\nThe cat naps on the windowsill[^1].', },
  { sliceIndex: 1, replacementText: `## The bird\n\nA bird sits there. ${BIRD_BREAK}`, },
  { sliceIndex: 2, replacementText: '## Notes\n\n[^1]: That is the spot it likes best.', },
];

await describe({
  name: 'structural assembly withdrawal',
  children: [
    it({
      name: 'REFUSES a counterfactual whose restored grammar exposes an unresolved footnote',
      fn: async () => {
        /** Source-only passage gives the second replacement an insertion anchor. */
        const prepared = prepareDocumentPair({
          sourceText: `${COMPONENT}\n\n> 猫咪的记录。`,
          targetText: COMPONENT,
          blockPairings: new Map([[0, [{ source: 0, target: 0, },],],]),
        },);
        /** Removing the malformed component alone would leave an unpaired note. */
        const replacements = prepared.slices.map(function replacement(slice,) {
          return {
            sliceIndex: slice.target.sliceIndex,
            replacementText: slice.source.text.includes('PhotoScroll',) ? BROKEN_COMPONENT : '> The cat rests.[^lost]',
          };
        },);
        expect(singleStructuralWithdrawal({ targetText: COMPONENT, slices: prepared.slices, replacements, },),).toEqual([],);
      },
    },),
    it({
      name: 'WITHDRAWS TWO MALFORMED REPLACEMENTS ONE ROUND AT A TIME by the parser\'s position when no '
        + 'single withdrawal repairs the page, and never gives the whole page up (class fifty-eight)',
      fn: async () => {
        /** Distinct sections force separate slices without a character-budget dial. */
        const targetText = '## Cats\n\nCats rest.\n\n## Birds\n\nBirds sing.';
        /** Identity preparation has no inherited structural defect. */
        const prepared = prepareDocumentPair({ sourceText: targetText, targetText, },);
        /** Every slice independently introduces malformed JSX. */
        const replacements = prepared.slices.map(function malformed(slice,) {
          return { sliceIndex: slice.target.sliceIndex, replacementText: BROKEN_COMPONENT, };
        },);
        expect(replacements.length,).toBeGreaterThan(1,);
        /** No single reversion removes both grammar failures; the advancing step takes the first, the proof the second. */
        const guarded = guardFootnoteAssembly({ targetText, slices: prepared.slices, replacements, },);
        expect(guarded.assembledText,).toBe(targetText,);
        expect(guarded.replacements,).toEqual([],);
        expect(guarded.revertedChunkIndices.length,).toBe(replacements.length,);
        expect(guarded.findings.some(function advancing(finding,): boolean {
          return finding.startsWith('assembly-structure-advancing-withdrawal',);
        },),).toBe(true,);
        expect(guarded.findings.some(function blanket(finding,): boolean {
          return finding.startsWith('assembly-withdrew-every-replacement',);
        },),).toBe(false,);
      },
    },),
    it({
      name: 'PROVES whole-document validity across container halves instead of blaming isolated slices',
      fn: async () => {
        /** Container spans the first two slices; the last has unrelated prose. */
        const targetText = '<details>\n\nCats rest.\n\nBirds sing.\n\n</details>\n\nThe end.';
        /** Actual document boundaries, not standalone parse units. */
        const boundaries = [0, targetText.indexOf('Birds',), targetText.indexOf('The end.',), targetText.length,];
        /** Pairs split inside the container as production container extents permit. */
        const slices: readonly ChunkPair[] = boundaries.slice(0, -1,).map(function pair(startOffset, sliceIndex,) {
          /** End of this slice, known because only nonfinal boundaries are mapped. */
          const endOffset = boundaries[sliceIndex + 1];
          if (endOffset === undefined)
            throw new Error('fixture lost its next boundary',);
          /** Shared chunk before any replacement. */
          const chunk = { sliceIndex, startOffset, endOffset, text: targetText.slice(startOffset, endOffset,), nodes: [], };
          return { source: chunk, target: chunk, };
        },);
        /** Each valid replacement owns only one half of the container. */
        const texts = [
          `<details>\n\nCats [rest](${LINK}).\n\n`,
          'Birds sing softly.\n\n</details>\n\n',
          BROKEN_COMPONENT,
        ];
        /** All replacements are real changes, with grammar broken only at the final one. */
        const replacements = slices.map(function replacement(slice, at,) {
          /** Matching replacement text, checked instead of a non-null assertion. */
          const replacementText = texts[at];
          if (replacementText === undefined)
            throw new Error('fixture lost its replacement',);
          return { sliceIndex: slice.target.sliceIndex, replacementText, };
        },);
        /** Guard must not misclassify the container halves as independently malformed. */
        const guarded = guardFootnoteAssembly({ targetText, slices, replacements, },);
        expect(guarded.replacements,).toEqual(replacements.slice(0, -1,),);
        expect(guarded.revertedChunkIndices,).toEqual([slices.at(-1,)?.target.sliceIndex,],);
        expect(guarded.assembledText,).toContain(LINK,);
        expect(guarded.assembledText,).toContain('Birds sing softly.',);
      },
    },),
    it({
      name: 'KEEPS a translated source-only link when reverting one malformed component repairs the whole page',
      fn: async () => {
        /** Pair only the component; the linked source passage becomes an insertion. */
        const prepared = prepareDocumentPair({
          sourceText: `${COMPONENT}\n\n> 猫咪的[记录](${LINK})。`,
          targetText: COMPONENT,
          blockPairings: new Map([[0, [{ source: 0, target: 0, },],],]),
        },);
        /** Slice containing the component the selected candidate broke. */
        const component = prepared.slices.find(function componentSlice(slice,): boolean {
          return slice.source.text.includes('PhotoScroll',);
        },);
        /** Independently valid translation of a passage absent from the archive. */
        const insertion = prepared.slices.find(function linkedSlice(slice,): boolean {
          return slice.source.text.includes(LINK,);
        },);
        if ((component === undefined) || (insertion === undefined))
          throw new Error('fixture must expose component and insertion slices',);
        /** Exact wording whose loss stopped Mio8 at publication. */
        const translated = `> The cat's [record](${LINK}).`;
        /** Real assembly guard, including whole-page grammar and footnote checks. */
        const guarded = guardFootnoteAssembly({
          targetText: COMPONENT,
          slices: prepared.slices,
          replacements: [
            { sliceIndex: component.target.sliceIndex, replacementText: BROKEN_COMPONENT, },
            { sliceIndex: insertion.target.sliceIndex, replacementText: translated, },
          ],
        },);
        expect(guarded.revertedChunkIndices,).toEqual([component.target.sliceIndex,],);
        expect(guarded.replacements,).toEqual([
          { sliceIndex: insertion.target.sliceIndex, replacementText: translated, },
        ],);
        expect(guarded.assembledText,).toContain(translated,);
        expect(guarded.assembledText,).toContain(COMPONENT,);
        expect(guarded.findings.some(function proved(finding,): boolean {
          return finding.startsWith('assembly-structure-single-withdrawal',);
        },),).toBe(true,);
      },
    },),
    it({
      name: 'READS EVERY REFUSAL ON THE PAGE AS ASSEMBLED, so a withdrawal that only lengthens the text before a '
        + 'break never reads as moving it, and the one that removes the first break is chosen (ledger B85)',
      fn: async () => {
        /** Three sections, three slices. */
        const prepared = prepareDocumentPair({ sourceText: SECTIONS_SOURCE, targetText: SECTIONS_TARGET, },);
        expect(prepared.slices.length,).toBe(SHIFTING.length,);
        /** Page as assembled, both breaks on it. */
        const standing = spliceSlices({ targetText: SECTIONS_TARGET, slices: prepared.slices, replacements: SHIFTING, },);
        /** Where the grammar first stops on it: the bird's open expression. */
        const first = strictRefusalOffset({ text: standing, },);
        /** Same page with the bird's break overwritten at its own length, which leaves the notes break where it stands. */
        const second = strictRefusalOffset({ text: standing.replace(BIRD_BREAK, 'x'.repeat(BIRD_BREAK.length,),), },);
        if ((!first.refused) || (!second.refused))
          throw new Error('both breaks are on the page',);
        expect(second.offset,).toBeGreaterThan(first.offset,);
        /** What the guard would withdraw next. */
        const steps = advancingStructuralWithdrawal({ targetText: SECTIONS_TARGET, slices: prepared.slices, replacements: SHIFTING, },);
        expect(steps.map(function toIndex(step,): number {
          return step.sliceIndex;
        },),).toEqual([1,],);
        expect(steps,).toEqual([{ sliceIndex: 1, from: first.offset, to: second.offset, cleared: false, },],);
      },
    },),
    it({
      name: 'KEEPS A WHOLE REPLACEMENT that only changed the length of the text before two breaks, withdrawing the '
        + 'two broken ones (ledger B85)',
      fn: async () => {
        /** Three sections, three slices. */
        const prepared = prepareDocumentPair({ sourceText: SECTIONS_SOURCE, targetText: SECTIONS_TARGET, },);
        /** Real assembly guard over both breaks and the whole section before them. */
        const guarded = guardFootnoteAssembly({ targetText: SECTIONS_TARGET, slices: prepared.slices, replacements: SHIFTING, },);
        expect(guarded.replacements,).toEqual(SHIFTING.slice(0, 1,),);
        expect(guarded.revertedChunkIndices.toSorted(function ascending(left, right,): number {
          return left - right;
        },),).toEqual([1, 2,],);
        expect(guarded.assembledText,).toContain('It naps[^1].',);
      },
    },),
    it({
      name: 'READS AN ELEMENT LEFT OPEN TO THE END AS STOPPING THERE, so withdrawing the break the parser named first '
        + 'moves the refusal to the page\'s end rather than back to its start (ledger B86)',
      fn: async () => {
        /** Three sections, three slices. */
        const prepared = prepareDocumentPair({ sourceText: SECTIONS_SOURCE, targetText: SECTIONS_TARGET, },);
        /** Page as assembled, both defects on it. */
        const standing = spliceSlices({ targetText: SECTIONS_TARGET, slices: prepared.slices, replacements: LEFT_OPEN, },);
        /** Where the grammar first stops on it: the bird's open expression. */
        const first = strictRefusalOffset({ text: standing, },);
        if (!first.refused)
          throw new Error('the expression is on the page',);
        expect(advancingStructuralWithdrawal({ targetText: SECTIONS_TARGET, slices: prepared.slices, replacements: LEFT_OPEN, },),)
          .toEqual([{ sliceIndex: 1, from: first.offset, to: standing.length, cleared: false, },],);
      },
    },),
    it({
      name: 'KEEPS THE WHOLE SECTION beside an element left open and a broken expression, withdrawing the two '
        + 'one round at a time instead of every replacement (ledger B86)',
      fn: async () => {
        /** Three sections, three slices. */
        const prepared = prepareDocumentPair({ sourceText: SECTIONS_SOURCE, targetText: SECTIONS_TARGET, },);
        /** Real assembly guard over both defects. */
        const guarded = guardFootnoteAssembly({ targetText: SECTIONS_TARGET, slices: prepared.slices, replacements: LEFT_OPEN, },);
        expect(guarded.replacements,).toEqual(LEFT_OPEN.slice(2,),);
        expect(guarded.findings.some(function blanket(finding,): boolean {
          return finding.startsWith('assembly-withdrew-every-replacement',);
        },),).toBe(false,);
      },
    },),
    it({
      name: 'MARKS A WITHDRAWAL AFTER WHICH THE GRAMMAR ACCEPTS THE PAGE as clearing the refusal, its step ending at '
        + 'the page\'s length',
      fn: async () => {
        /** Three sections, three slices. */
        const prepared = prepareDocumentPair({ sourceText: SECTIONS_SOURCE, targetText: SECTIONS_TARGET, },);
        /** A whole section and the bird's break, the only one on the page. */
        const replacements = SHIFTING.slice(0, 2,);
        /** Page as assembled. */
        const standing = spliceSlices({ targetText: SECTIONS_TARGET, slices: prepared.slices, replacements, },);
        /** Where the grammar stops on it. */
        const first = strictRefusalOffset({ text: standing, },);
        if (!first.refused)
          throw new Error('the break is on the page',);
        expect(advancingStructuralWithdrawal({ targetText: SECTIONS_TARGET, slices: prepared.slices, replacements, },),)
          .toEqual([{ sliceIndex: 1, from: first.offset, to: standing.length, cleared: true, },],);
      },
    },),
    it({
      name: 'READS A BREAK BEFORE A WITHDRAWN LAST SECTION AS UNMOVED, when withdrawing it only cuts the page short',
      fn: async () => {
        /** Three sections, three slices. */
        const prepared = prepareDocumentPair({ sourceText: SECTIONS_SOURCE, targetText: SECTIONS_TARGET, },);
        /** The bird's break, and a last section that only adds a closing line to its archive text. */
        const replacements = [
          ...SHIFTING.slice(1, 2,),
          { sliceIndex: 2, replacementText: '## Notes\n\n[^1]: That is its favourite spot.\n\nThe end.', },
        ];
        /** Page as assembled. */
        const standing = spliceSlices({ targetText: SECTIONS_TARGET, slices: prepared.slices, replacements, },);
        /** Where the grammar stops on it. */
        const first = strictRefusalOffset({ text: standing, },);
        if (!first.refused)
          throw new Error('the break is on the page',);
        expect(advancingStructuralWithdrawal({ targetText: SECTIONS_TARGET, slices: prepared.slices, replacements, },),)
          .toEqual([{ sliceIndex: 1, from: first.offset, to: standing.length, cleared: true, },],);
      },
    },),
    it({
      name: 'READS A BREAK AFTER A SECTION THAT ONLY GAINED A SENTENCE AS UNMOVED: the withdrawn page is the '
        + 'assembled one less that sentence, every other character shared',
      fn: async () => {
        /** Three sections, three slices. */
        const prepared = prepareDocumentPair({ sourceText: SECTIONS_SOURCE, targetText: SECTIONS_TARGET, },);
        /** The opening section's archive text with a sentence added, and the bird's break after it. */
        const replacements = [
          {
            sliceIndex: 0,
            replacementText: '## The cat\n\nThe cat is doing the sleeping on the windowsill[^1]. She dreams of fish.',
          },
          ...SHIFTING.slice(1, 2,),
        ];
        /** Page as assembled. */
        const standing = spliceSlices({ targetText: SECTIONS_TARGET, slices: prepared.slices, replacements, },);
        /** Where the grammar stops on it. */
        const first = strictRefusalOffset({ text: standing, },);
        if (!first.refused)
          throw new Error('the break is on the page',);
        expect(advancingStructuralWithdrawal({ targetText: SECTIONS_TARGET, slices: prepared.slices, replacements, },),)
          .toEqual([{ sliceIndex: 1, from: first.offset, to: standing.length, cleared: true, },],);
      },
    },),
    it({
      name: 'PLACES A REFUSAL IN THE WITHDRAWN SLICE\'S OWN ARCHIVE TEXT where that slice begins, so restoring an '
        + 'archive section that breaks earlier than the page does is never chosen',
      fn: async () => {
        /** Archive whose bird heading leaves an expression open. */
        const brokenArchive = SECTIONS_TARGET.replace('## The bird\n', '## The bird {\'archived\n',);
        expect(brokenArchive,).not.toBe(SECTIONS_TARGET,);
        /** Three sections, three slices. */
        const prepared = prepareDocumentPair({ sourceText: SECTIONS_SOURCE, targetText: brokenArchive, },);
        /** A bird section that mends the archive's heading, and a notes heading that breaks. */
        const replacements = [
          { sliceIndex: 1, replacementText: '## The bird\n\nA bird sits on the windowsill.', },
          { sliceIndex: 2, replacementText: '## Notes {\'unclosed\n\n[^1]: That is its favourite spot.', },
        ];
        /** Page as assembled. */
        const standing = spliceSlices({ targetText: brokenArchive, slices: prepared.slices, replacements, },);
        /** Where the grammar stops on it: the notes heading. */
        const first = strictRefusalOffset({ text: standing, },);
        if (!first.refused)
          throw new Error('the notes break is on the page',);
        expect(advancingStructuralWithdrawal({ targetText: brokenArchive, slices: prepared.slices, replacements, },),)
          .toEqual([{ sliceIndex: 2, from: first.offset, to: standing.length, cleared: true, },],);
      },
    },),
    it({
      name: 'CHOOSES NOTHING when no single withdrawal moves the first refusal: two expressions each left open to the '
        + 'end of the page stop the grammar there whichever one is withdrawn',
      fn: async () => {
        /** Three sections, three slices. */
        const prepared = prepareDocumentPair({ sourceText: SECTIONS_SOURCE, targetText: SECTIONS_TARGET, },);
        /** Two sections each opening an expression at the start of a line, which the grammar refuses at the end. */
        const replacements = [
          { sliceIndex: 1, replacementText: '## The bird\n\n{\'one', },
          { sliceIndex: 2, replacementText: '## Notes\n\n{\'two', },
        ];
        /** Page as assembled. */
        const standing = spliceSlices({ targetText: SECTIONS_TARGET, slices: prepared.slices, replacements, },);
        expect(strictRefusalOffset({ text: standing, },),).toEqual({ refused: true, offset: standing.length, },);
        expect(advancingStructuralWithdrawal({ targetText: SECTIONS_TARGET, slices: prepared.slices, replacements, },),)
          .toEqual([],);
      },
    },),
    it({
      name: 'RANKS TWO WITHDRAWALS THAT EACH MOVE THE REFUSAL by whether the grammar then accepts the page, then by '
        + 'how far the refusal moves, keeping the earlier slice where both reach the same place alike',
      fn: async () => {
        /** Three sections, three slices. */
        const prepared = prepareDocumentPair({ sourceText: SECTIONS_SOURCE, targetText: SECTIONS_TARGET, },);
        /** Sets where two withdrawals each move the refusal, with the slice and clearing the ranking picks. */
        const ranked = [
          {
            // Neither clears; withdrawing the bird moves the refusal to the page's end, the cat less far.
            replacements: [
              { sliceIndex: 0, replacementText: '## The cat\n\n{\n\nThe cat naps[^1].', },
              { sliceIndex: 1, replacementText: '## The bird\n\n{\n\nA bird sits there. }\n\n}', },
            ],
            sliceIndex: 1,
            cleared: false,
          },
          {
            // Both clear and reach the page's end, so the earlier slice stands.
            replacements: [
              { sliceIndex: 0, replacementText: '## The cat\n\n```\n\nThe cat naps[^1]. {', },
              { sliceIndex: 1, replacementText: '## The bird\n\n```\n\nA bird sits there. {\n\n}', },
            ],
            sliceIndex: 0,
            cleared: true,
          },
          {
            // Both reach the page's end, and only withdrawing the notes clears.
            replacements: [
              { sliceIndex: 0, replacementText: '## The cat\n\n}\n\nThe cat naps[^1].\n\n```', },
              { sliceIndex: 1, replacementText: '## The bird\n\n{/*\n\nA bird sits there.\n\n<Cat', },
              { sliceIndex: 2, replacementText: '## Notes\n\n```\n\n[^1]: That is its favourite spot. {\'open', },
            ],
            sliceIndex: 2,
            cleared: true,
          },
          {
            // Both reach the page's end, and only withdrawing the cat clears.
            replacements: [
              { sliceIndex: 0, replacementText: '## The cat\n\n{\n\nThe cat naps[^1]. {\n\n{/*', },
              { sliceIndex: 1, replacementText: '## The bird\n\n}\n\nA bird sits there.', },
            ],
            sliceIndex: 0,
            cleared: true,
          },
        ];
        expect(ranked.map(function chosen({ replacements, },) {
          return advancingStructuralWithdrawal({ targetText: SECTIONS_TARGET, slices: prepared.slices, replacements, },);
        },),).toEqual(ranked.map(function expected({ replacements, sliceIndex, cleared, },) {
          /** Page as assembled. */
          const standing = spliceSlices({ targetText: SECTIONS_TARGET, slices: prepared.slices, replacements, },);
          /** Where the grammar stops on it. */
          const first = strictRefusalOffset({ text: standing, },);
          if (!first.refused)
            throw new Error('a break is on the page',);
          return [{ sliceIndex, from: first.offset, to: standing.length, cleared, },];
        },),);
      },
    },),
  ],
},);
