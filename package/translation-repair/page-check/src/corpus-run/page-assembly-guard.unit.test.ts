/**
 Tests the page-level assembly guard over a composed page.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { makeInsertionChunk, } from '@monochromatic-dev/translation-repair-shared-contract';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading';
import {
  guardPageAssembly,
  type WouldShipSource,
} from '../../dist/final/node/index.mjs';

/**
 Archive whose body refers to a note it never defines.
 */
const TARGET = 'The cat naps on the windowsill[^1].\n';

/**
 One content slice plus the anchor where the notes belong.
 */
const SLICES: readonly ChunkPair[] = [
  {
    source: {
      kind: 'content',
      sliceIndex: 0,
      nodes: [],
      startOffset: 0,
      endOffset: 12,
      text: '猫猫在窗台上打盹〔1〕。',
    },
    target: {
      kind: 'content',
      sliceIndex: 0,
      nodes: [],
      startOffset: 0,
      endOffset: TARGET.length,
      text: TARGET,
    },
  },
  {
    source: {
      kind: 'content',
      sliceIndex: 1,
      nodes: [],
      startOffset: 14,
      endOffset: 40,
      text: '〔1〕：那是它最喜欢的位置。\n〔2〕：一只麻雀。',
    },
    target: makeInsertionChunk({ sliceIndex: 1, offset: TARGET.length, },),
  },
];

/**
 Builds a source whose consolidation wrote the notes at the anchor.

 @param notes - what the consolidation wrote at slice 1

 @returns Narrow artifact source read by the publication assembler

 @example
 ```ts
 const artifact = consolidating({ notes: '[^1]: A note.', },);
 ```
 */
function consolidating({ notes, }: { readonly notes: string; },): WouldShipSource {
  return {
    comparison: [
      {
        sliceIndex: 0,
        incumbentKind: 'present',
        incumbentText: TARGET,
        repairText: TARGET,
        translateText: TARGET,
        laneRelation: 'both-kept',
        repairOutcome: { kind: 'decided', acceptedText: TARGET, },
        translateOutcome: { kind: 'decided', acceptedText: TARGET, },
        decisionComparison: { kind: 'comparable', verdict: 'same', },
        repairDelivery: { kind: 'incumbent-retained', },
        translateDelivery: { kind: 'incumbent-retained', },
      },
      {
        sliceIndex: 1,
        incumbentKind: 'absent',
        incumbentText: '',
        repairText: '',
        translateText: '',
        laneRelation: 'both-kept',
        repairOutcome: { kind: 'not-evaluated', },
        translateOutcome: { kind: 'not-evaluated', },
        decisionComparison: { kind: 'comparable', verdict: 'same', },
        repairDelivery: { kind: 'gap-remains', },
        translateDelivery: { kind: 'gap-remains', },
      },
    ],
    consolidation: {
      kind: 'settled',
      slices: [{
        sliceIndex: 1,
        terminal: 'consolidated',
        shipped: { kind: 'consolidated', text: notes, },
        rewrapped: false,
        demoted: false,
        verdicts: [],
        gate: { kind: 'not-asked', },
      },],
    },
    laneSelection: { kind: 'contested', slices: [], },
  } as unknown as WouldShipSource;
}

/**
 Original naming her nickname once.
 */
const GLOSS_SOURCE = '她打盹时给自己起了“云团咪”这个昵称。\n';
/**
 Archive carrying the nickname sentence and, on the next line of the same
 paragraph, the translator's gloss of the name the Chinese is silent about.
 */
const GLOSS_TARGET = 'She got her nickname “Mittens the Cloud” while napping.\n'
  + '“Mittens the Cloud” means a cat like a cloud.\n';
/**
 One content slice over the nickname paragraph.
 */
const GLOSS_SLICES: readonly ChunkPair[] = [{
  source: {
    kind: 'content',
    sliceIndex: 0,
    nodes: [],
    startOffset: 0,
    endOffset: GLOSS_SOURCE.length,
    text: GLOSS_SOURCE,
  },
  target: {
    kind: 'content',
    sliceIndex: 0,
    nodes: [],
    startOffset: 0,
    endOffset: GLOSS_TARGET.length,
    text: GLOSS_TARGET,
  },
},];
/**
 Builds a source whose consolidation shipped the nickname sentence afresh.
 @param text - what the consolidation shipped at slice 0
 @returns Narrow artifact source read by the publication assembler
 @example
 ```ts
 const artifact = shippingGlossless({ text: 'Her handle was coined while she napped.', },);
 ```
 */
function shippingGlossless(
  { text = 'Her handle “Mittens the Cloud” was coined while she napped.', }: { readonly text?: string; } = {},
): WouldShipSource {
  return {
    comparison: [{
      sliceIndex: 0,
      incumbentKind: 'present',
      incumbentText: GLOSS_TARGET,
      repairText: GLOSS_TARGET,
      translateText: text,
      laneRelation: 'both-kept',
      repairOutcome: { kind: 'decided', acceptedText: GLOSS_TARGET, },
      translateOutcome: { kind: 'decided', acceptedText: text, },
      decisionComparison: { kind: 'comparable', verdict: 'differ', },
      repairDelivery: { kind: 'incumbent-retained', },
      translateDelivery: { kind: 'delivered', },
    },],
    consolidation: {
      kind: 'settled',
      slices: [{
        sliceIndex: 0,
        terminal: 'consolidated',
        shipped: { kind: 'consolidated', text, },
        rewrapped: false,
        demoted: false,
        verdicts: [],
        gate: { kind: 'not-asked', },
      },],
    },
    laneSelection: { kind: 'contested', slices: [], },
  } as unknown as WouldShipSource;
}
/**
 Archive carrying one paragraph and none of the disclosure block after it.
 */
const HALVES_TARGET = 'The cat naps on the windowsill.\n\n';

/**
 Opening half of the block the archive never carried.
 */
const OPEN_HALF_SOURCE = '<details style="margin-top: 0.5rem;">\n<summary>猫的故事</summary>';

/**
 Closing half of that block.
 */
const CLOSE_HALF_SOURCE = '猫在夜里回家了。\n\n</details>';

/**
 Rendering of the closing half.
 */
const CLOSE_HALF_TEXT = 'The cat came home at night.\n\n</details>\n';

/**
 One content slice and the two halves of the block, both anchored after it.
 */
const HALVES_SLICES: readonly ChunkPair[] = [
  {
    source: {
      kind: 'content',
      sliceIndex: 0,
      nodes: [],
      startOffset: 0,
      endOffset: 9,
      text: '猫猫在窗台上打盹。',
    },
    target: {
      kind: 'content',
      sliceIndex: 0,
      nodes: [],
      startOffset: 0,
      endOffset: HALVES_TARGET.length,
      text: HALVES_TARGET,
    },
  },
  {
    source: {
      kind: 'content',
      sliceIndex: 1,
      nodes: [],
      startOffset: 11,
      endOffset: 11 + OPEN_HALF_SOURCE.length,
      text: OPEN_HALF_SOURCE,
    },
    target: makeInsertionChunk({ sliceIndex: 1, offset: HALVES_TARGET.length, },),
  },
  {
    source: {
      kind: 'content',
      sliceIndex: 2,
      nodes: [],
      startOffset: 13 + OPEN_HALF_SOURCE.length,
      endOffset: 13 + OPEN_HALF_SOURCE.length + CLOSE_HALF_SOURCE.length,
      text: CLOSE_HALF_SOURCE,
    },
    target: makeInsertionChunk({ sliceIndex: 2, offset: HALVES_TARGET.length, },),
  },
];

/**
 Builds a source whose translate lane filled the closing half alone.

 @returns Narrow artifact source read by the publication assembler

 @example
 ```ts
 const artifact = closingHalfAlone();
 ```
 */
function closingHalfAlone(): WouldShipSource {
  return {
    comparison: [
      {
        sliceIndex: 0,
        incumbentKind: 'present',
        incumbentText: HALVES_TARGET,
        repairText: HALVES_TARGET,
        translateText: HALVES_TARGET,
        laneRelation: 'both-kept',
        repairOutcome: { kind: 'decided', acceptedText: HALVES_TARGET, },
        translateOutcome: { kind: 'decided', acceptedText: HALVES_TARGET, },
        decisionComparison: { kind: 'comparable', verdict: 'same', },
        repairDelivery: { kind: 'incumbent-retained', },
        translateDelivery: { kind: 'incumbent-retained', },
      },
      {
        sliceIndex: 1,
        incumbentKind: 'absent',
        incumbentText: '',
        repairText: '',
        translateText: '',
        laneRelation: 'both-kept',
        repairOutcome: { kind: 'not-evaluated', },
        translateOutcome: { kind: 'not-evaluated', },
        decisionComparison: { kind: 'comparable', verdict: 'same', },
        repairDelivery: { kind: 'gap-remains', },
        translateDelivery: { kind: 'gap-remains', },
      },
      {
        sliceIndex: 2,
        incumbentKind: 'absent',
        incumbentText: '',
        repairText: '',
        translateText: CLOSE_HALF_TEXT,
        laneRelation: 'both-kept',
        repairOutcome: { kind: 'not-evaluated', },
        translateOutcome: { kind: 'decided', acceptedText: CLOSE_HALF_TEXT, },
        decisionComparison: { kind: 'comparable', verdict: 'same', },
        repairDelivery: { kind: 'gap-remains', },
        translateDelivery: { kind: 'gap-remains', },
      },
    ],
    consolidation: {
      kind: 'settled',
      slices: [{
        sliceIndex: 2,
        terminal: 'consolidated',
        shipped: { kind: 'consolidated', text: CLOSE_HALF_TEXT, },
        rewrapped: false,
        demoted: false,
        verdicts: [],
        gate: { kind: 'not-asked', },
      },],
    },
    laneSelection: { kind: 'contested', slices: [], },
  } as unknown as WouldShipSource;
}

/**
 Archive paragraph carrying a day-first date and a reference to a note
 defined in the next slice.
 */
const DATED_PARAGRAPH = 'The cat napped on 29 April 2024 by the window[^1].\n\n';

/**
 Archive definition of that note.
 */
const DATED_NOTE = '[^1]: Its favourite spot.\n';

/**
 Original paragraph beside that archive paragraph.
 */
const DATED_SOURCE_PARAGRAPH = '猫猫在2024年4月29日在窗边打盹〔1〕。\n\n';

/**
 Original definition beside that archive definition.
 */
const DATED_SOURCE_NOTE = '〔1〕：它最喜欢的位置。\n';

/**
 The dated paragraph and the note, one content slice each.
 */
const DATED_SLICES: readonly ChunkPair[] = [
  {
    source: {
      kind: 'content',
      sliceIndex: 0,
      nodes: [],
      startOffset: 0,
      endOffset: DATED_SOURCE_PARAGRAPH.length,
      text: DATED_SOURCE_PARAGRAPH,
    },
    target: {
      kind: 'content',
      sliceIndex: 0,
      nodes: [],
      startOffset: 0,
      endOffset: DATED_PARAGRAPH.length,
      text: DATED_PARAGRAPH,
    },
  },
  {
    source: {
      kind: 'content',
      sliceIndex: 1,
      nodes: [],
      startOffset: DATED_SOURCE_PARAGRAPH.length,
      endOffset: DATED_SOURCE_PARAGRAPH.length + DATED_SOURCE_NOTE.length,
      text: DATED_SOURCE_NOTE,
    },
    target: {
      kind: 'content',
      sliceIndex: 1,
      nodes: [],
      startOffset: DATED_PARAGRAPH.length,
      endOffset: DATED_PARAGRAPH.length + DATED_NOTE.length,
      text: DATED_NOTE,
    },
  },
];

/**
 Builds a source whose consolidation rewrote the dated paragraph without its
 note reference, which the footnote guard takes back.

 @returns Narrow artifact source read by the publication assembler

 @example
 ```ts
 const artifact = droppingTheReference();
 ```
 */
function droppingTheReference(): WouldShipSource {
  /**
   Lane wording that lost the reference.
   */
  const unreferenced = 'The cat napped by the window on the 29th of April.\n\n';
  return {
    comparison: [
      {
        sliceIndex: 0,
        incumbentKind: 'present',
        incumbentText: DATED_PARAGRAPH,
        repairText: DATED_PARAGRAPH,
        translateText: unreferenced,
        laneRelation: 'translate-only',
        repairOutcome: { kind: 'decided', acceptedText: DATED_PARAGRAPH, },
        translateOutcome: { kind: 'decided', acceptedText: unreferenced, },
        decisionComparison: { kind: 'comparable', verdict: 'different', },
        repairDelivery: { kind: 'incumbent-retained', },
        translateDelivery: { kind: 'replacement-shipped', },
      },
      {
        sliceIndex: 1,
        incumbentKind: 'present',
        incumbentText: DATED_NOTE,
        repairText: DATED_NOTE,
        translateText: DATED_NOTE,
        laneRelation: 'both-kept',
        repairOutcome: { kind: 'decided', acceptedText: DATED_NOTE, },
        translateOutcome: { kind: 'decided', acceptedText: DATED_NOTE, },
        decisionComparison: { kind: 'comparable', verdict: 'same', },
        repairDelivery: { kind: 'incumbent-retained', },
        translateDelivery: { kind: 'incumbent-retained', },
      },
    ],
    consolidation: {
      kind: 'settled',
      slices: [{
        sliceIndex: 0,
        terminal: 'consolidated',
        shipped: { kind: 'consolidated', text: unreferenced, },
        rewrapped: false,
        demoted: false,
        verdicts: [],
        gate: { kind: 'not-asked', },
      },],
    },
    laneSelection: { kind: 'contested', slices: [], },
  } as unknown as WouldShipSource;
}

await describe({
  name: guardPageAssembly.name,
  children: [
    it({
      name: 'WITHHOLDS A CONTAINER HALF WHOSE PARTNER SHIPS NOTHING on the composed page, naming the partner '
        + '(class fifty-seven, XingZ607)',
      fn: async () => {
        const assembly = guardPageAssembly({
          artifact: closingHalfAlone(),
          slices: HALVES_SLICES,
          sourceText: `猫猫在窗台上打盹。\n\n${OPEN_HALF_SOURCE}\n\n${CLOSE_HALF_SOURCE}\n`,
          targetText: HALVES_TARGET,
        },);
        expect(assembly.withdrawn,).toEqual([2,],);
        expect(assembly.trimmed,).toEqual([],);
        expect(assembly.findings,).toEqual([
          'assembly-container-half-withheld (slice 2 beside slice 1: one container\'s halves ship together, and '
            + 'slice 1 ships nothing)',
        ],);
      },
    },),
    it({
      name: 'TRIMS the orphan out of the consolidation\'s two notes one line apart and records the text the '
        + 'page carries (the twenty-second hakureico pass of 2026-09-09 shipped the consolidation\'s fresh '
        + 'rendering of both notes and the page guard refused the page)',
      fn: async () => {
        const assembly = guardPageAssembly({
          artifact: consolidating({ notes: '[^1]: That is its favourite spot.\n[^2]: A sparrow.', },),
          slices: SLICES,
          sourceText: '猫猫在窗台上打盹〔1〕。\n\n〔1〕：那是它最喜欢的位置。\n〔2〕：一只麻雀。\n',
          targetText: TARGET,
        },);
        expect(assembly.withdrawn,).toEqual([],);
        expect(assembly.trimmed,).toEqual([{
          sliceIndex: 1,
          replacementText: '[^1]: That is its favourite spot.',
        },],);
        expect(assembly.findings,).toEqual([
          'assembly-footnote-trimmed orphan-definition gfm 2 (slice 1)',
        ],);
      },
    },),
    it({
      name: 'RECORDS A SLICE ONCE WHERE A PASS REWROTE IT AND THE FOOTNOTE GUARD TRIMMED IT, the guard\'s row '
        + 'carrying the rewrite',
      fn: async () => {
        const assembly = guardPageAssembly({
          artifact: consolidating({ notes: '[^1]: That is its favourite spot since 4 May.\n[^2]: A sparrow.', },),
          slices: SLICES,
          sourceText: '猫猫在窗台上打盹〔1〕。\n\n〔1〕：那是它最喜欢的位置。\n〔2〕：一只麻雀。\n',
          targetText: TARGET,
        },);
        expect(assembly.withdrawn,).toEqual([],);
        expect(assembly.trimmed,).toEqual([{
          sliceIndex: 1,
          replacementText: '[^1]: That is its favourite spot since May 4.',
        },],);
        expect(assembly.findings,).toEqual([
          'canadian-form-rewritten (slice 1: "4 May" to "May 4")',
          'assembly-footnote-trimmed orphan-definition gfm 2 (slice 1)',
        ],);
      },
    },),
    it({
      name: 'RESTORES THE ARCHIVE\'S GLOSS LINE OF A NAME the shipped text carries without any gloss '
        + '(class one hundred five, CuspariaKLSY9, 2026-09-23: "“Ling Shui Yu Yu Zi” means fish in clear '
        + 'water" dropped for the third time on the judges\' call)',
      fn: async () => {
        const assembly = guardPageAssembly({
          artifact: shippingGlossless(),
          slices: GLOSS_SLICES,
          sourceText: GLOSS_SOURCE,
          targetText: GLOSS_TARGET,
        },);
        expect(assembly.withdrawn,).toEqual([],);
        expect(assembly.trimmed,).toEqual([{
          sliceIndex: 0,
          replacementText: 'Her handle “Mittens the Cloud” was coined while she napped.\n'
            + '“Mittens the Cloud” means a cat like a cloud.',
        },],);
        expect(assembly.findings,).toEqual([
          'name-gloss-restored (slice 0: "“Mittens the Cloud” means a cat like a cloud.")',
        ],);
      },
    },),
    it({
      name: 'LEAVES a shipped text that glosses the name in its own way, and one that never carries the name',
      fn: async () => {
        /**
         The bench's parenthetical gloss, the house rule's form.
         */
        const parenthetical = guardPageAssembly({
          artifact: shippingGlossless({ text: 'Her handle, Mittens the Cloud (a cat like a cloud), was coined while she napped.', },),
          slices: GLOSS_SLICES,
          sourceText: GLOSS_SOURCE,
          targetText: GLOSS_TARGET,
        },);
        expect(parenthetical.trimmed,).toEqual([],);
        expect(parenthetical.findings,).toEqual([],);
        /**
         A rendering that never names her.
         */
        const nameless = guardPageAssembly({
          artifact: shippingGlossless({ text: 'Her handle was coined while she napped.', },),
          slices: GLOSS_SLICES,
          sourceText: GLOSS_SOURCE,
          targetText: GLOSS_TARGET,
        },);
        expect(nameless.trimmed,).toEqual([],);
        expect(nameless.findings,).toEqual([],);
      },
    },),
    it({
      name: 'records nothing on a page whose footnote graph is whole',
      fn: async () => {
        const assembly = guardPageAssembly({
          artifact: consolidating({ notes: '[^1]: That is its favourite spot.', },),
          slices: SLICES,
          sourceText: '猫猫在窗台上打盹〔1〕。\n\n〔1〕：那是它最喜欢的位置。\n〔2〕：一只麻雀。\n',
          targetText: TARGET,
        },);
        expect(assembly,).toEqual({ trimmed: [], withdrawn: [], findings: [], },);
      },
    },),
    it({
      name: 'FOLDS Windows line endings a lane wording carries, so an LF page never ships a carriage return (ledger A3: '
        + 'three lines of one page ended in a carriage return a model wrote)',
      fn: async () => {
        const assembly = guardPageAssembly({
          artifact: shippingGlossless({ text: 'Her handle “Mittens the Cloud” was coined\r\nwhile she napped.', },),
          slices: GLOSS_SLICES,
          sourceText: GLOSS_SOURCE,
          targetText: GLOSS_TARGET,
        },);
        /**
         What the page carries at the slice.
         */
        const carried = assembly.trimmed.find(function atSlice(row,): boolean {
          return row.sliceIndex === 0;
        },);
        expect(carried?.replacementText.includes('\r',),).toBe(false,);
        expect(carried?.replacementText,).toContain('while she napped.',);
        expect(assembly.findings.some(function namesFold(finding,): boolean {
          return finding.includes('Windows line ending',);
        },),).toBe(true,);
      },
    },),
    it({
      name: 'RUNS THE PAGE PASSES OVER THE ARCHIVE TEXT OF A SLICE IT WITHDRAWS, as over any untouched slice, '
        + 'so the page carries its date month first (ledger K5: a withdrawn slice shipped the archive text '
        + 'the passes never read)',
      fn: async () => {
        const assembly = guardPageAssembly({
          artifact: droppingTheReference(),
          slices: DATED_SLICES,
          sourceText: `${DATED_SOURCE_PARAGRAPH}${DATED_SOURCE_NOTE}`,
          targetText: `${DATED_PARAGRAPH}${DATED_NOTE}`,
        },);
        // The lane's row is still recorded as taken back.
        expect(assembly.withdrawn,).toEqual([0,],);
        expect(assembly.trimmed
          .filter(function atSliceZero(row,): boolean {
            return row.sliceIndex === 0;
          },)
          .map(function carried(row,): string {
            return row.replacementText;
          },),).toEqual(['The cat napped on April 29, 2024, by the window[^1].\n\n',],);
      },
    },),
  ],
},);
