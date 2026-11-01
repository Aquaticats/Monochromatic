/**
 Guards class forty-five (hulicaijia6, 2026-09-17): a heading a lane rewrote
 into another section's heading is restored to the archive's own heading at
 page assembly, so the page keeps as many distinct headings as the original
 and the entry does not die at publish. Cat-themed invention throughout; no
 corpus content appears here.

 @module
 */

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading';
import {
  guardPageAssembly,
  type WouldShipSource,
} from '../../dist/final/node/index.mjs';

/**
 Original with two sections.
 */
const SOURCE_TEXT = '## 小猫\n\n它睡了。\n\n## 大猫\n\n它醒了。\n';

/**
 Archive rendering both sections under distinct headings.
 */
const FIRST = '## Kitten\n\nIt sleeps.';

/**
 Second archive section.
 */
const SECOND = '## Tomcat\n\nIt wakes.';

/**
 Whole archive page.
 */
const TARGET = `${FIRST}\n\n${SECOND}\n`;

/**
 Two content slices, one per section.
 */
const SLICES: readonly ChunkPair[] = [
  {
    source: {
      kind: 'content',
      sliceIndex: 0,
      nodes: [],
      startOffset: 0,
      endOffset: 11,
      text: '## 小猫\n\n它睡了。',
    },
    target: {
      kind: 'content',
      sliceIndex: 0,
      nodes: [],
      startOffset: 0,
      endOffset: FIRST.length,
      text: FIRST,
    },
  },
  {
    source: {
      kind: 'content',
      sliceIndex: 1,
      nodes: [],
      startOffset: 13,
      endOffset: 24,
      text: '## 大猫\n\n它醒了。',
    },
    target: {
      kind: 'content',
      sliceIndex: 1,
      nodes: [],
      startOffset: FIRST.length + 2,
      endOffset: FIRST.length + 2 + SECOND.length,
      text: SECOND,
    },
  },
];

/**
 Builds a source whose consolidation settled the second section on the given
 text and left the first as the archive has it.

 @param second - what ships for the second section

 @returns Narrow artifact source read by the publication assembler

 @example
 ```ts
 const artifact = settledOn({ second: '## Kitten\n\nIt wakes.', },);
 ```
 */
function settledOn({ second, }: { readonly second: string; },): WouldShipSource {
  return {
    comparison: [
      {
        sliceIndex: 0,
        incumbentKind: 'present',
        incumbentText: FIRST,
        repairText: FIRST,
        translateText: FIRST,
        laneRelation: 'both-kept',
        repairOutcome: { kind: 'decided', acceptedText: FIRST, },
        translateOutcome: { kind: 'decided', acceptedText: FIRST, },
        decisionComparison: { kind: 'comparable', verdict: 'same', },
        repairDelivery: { kind: 'incumbent-retained', },
        translateDelivery: { kind: 'incumbent-retained', },
      },
      {
        sliceIndex: 1,
        incumbentKind: 'present',
        incumbentText: SECOND,
        repairText: second,
        translateText: second,
        laneRelation: 'both-kept',
        repairOutcome: { kind: 'decided', acceptedText: second, },
        translateOutcome: { kind: 'decided', acceptedText: second, },
        decisionComparison: { kind: 'comparable', verdict: 'same', },
        repairDelivery: { kind: 'incumbent-retained', },
        translateDelivery: { kind: 'incumbent-retained', },
      },
    ],
    consolidation: {
      kind: 'settled',
      slices: [{
        sliceIndex: 1,
        terminal: 'consolidated',
        shipped: { kind: 'consolidated', text: second, },
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
 Original sections of a three-section page.
 */
const THREE_SOURCE = [
  '## 一猫\n\n它睡了。',
  '## 二猫\n\n它醒了。',
  '## 三猫\n\n它跑了。',
];

/**
 Archive sections of the same page, each under its own heading.
 */
const THREE_ARCHIVE = [
  '## One Cat\n\nIt sleeps.',
  '## Two Cat\n\nIt wakes.',
  '## Three Cat\n\nIt runs.',
];

/**
 Places each section of a page one blank line after the last.

 @param parts - sections in page order

 @returns Each section's text and offsets in the joined page

 @example
 ```ts
 const sections = placedSections({ parts: THREE_ARCHIVE, },);
 ```
 */
function placedSections({ parts, }: { readonly parts: readonly string[]; },): readonly {
  readonly text: string;
  readonly startOffset: number;
  readonly endOffset: number;
}[] {
  return parts.map(function place(
    text,
    index,
  ) {
    /**
     Characters every earlier section and its blank line take.
     */
    const startOffset = parts
      .slice(0, index,)
      .reduce(function add(
        total,
        earlier,
      ): number {
        return total + earlier.length + '\n\n'.length;
      }, 0,);
    return {
      text,
      startOffset,
      endOffset: startOffset + text.length,
    };
  },);
}

/**
 One content slice per section of the three-section page.
 */
const THREE_SLICES: readonly ChunkPair[] = placedSections({ parts: THREE_SOURCE, },)
  .map(function pair(
    source,
    sliceIndex,
  ): ChunkPair {
    /**
     Archive section this slice spans.
     */
    const target = nonNullishOrThrow(placedSections({ parts: THREE_ARCHIVE, },)[sliceIndex],);
    return {
      source: {
        kind: 'content',
        sliceIndex,
        nodes: [],
        ...source,
      },
      target: {
        kind: 'content',
        sliceIndex,
        nodes: [],
        ...target,
      },
    };
  },);

/**
 Builds a source whose consolidation settled the named slices of the
 three-section page on the given texts and left the rest as the archive has
 them.

 @param shipped - text per slice index the consolidation settled on

 @returns Narrow artifact source read by the publication assembler

 @example
 ```ts
 const artifact = settledEach({ shipped: new Map([[0, 'It sleeps.',],],), },);
 ```
 */
function settledEach({ shipped, }: { readonly shipped: ReadonlyMap<number, string>; },): WouldShipSource {
  return {
    comparison: THREE_ARCHIVE.map(function row(
      incumbent,
      sliceIndex,
    ) {
      /**
       What ships for this slice.
       */
      const text = shipped.get(sliceIndex,) ?? incumbent;
      return {
        sliceIndex,
        incumbentKind: 'present',
        incumbentText: incumbent,
        repairText: text,
        translateText: text,
        laneRelation: 'both-kept',
        repairOutcome: { kind: 'decided', acceptedText: text, },
        translateOutcome: { kind: 'decided', acceptedText: text, },
        decisionComparison: { kind: 'comparable', verdict: 'same', },
        repairDelivery: { kind: 'incumbent-retained', },
        translateDelivery: { kind: 'incumbent-retained', },
      };
    },),
    consolidation: {
      kind: 'settled',
      slices: [...shipped.entries(),].map(function settled([
        sliceIndex,
        text,
      ],) {
        return {
          sliceIndex,
          terminal: 'consolidated',
          shipped: { kind: 'consolidated', text, },
          rewrapped: false,
          demoted: false,
          verdicts: [],
          gate: { kind: 'not-asked', },
        };
      },),
    },
    laneSelection: { kind: 'contested', slices: [], },
  } as unknown as WouldShipSource;
}

await describe({
  name: 'colliding headings restored at page assembly (class forty-five)',
  children: [
    it({
      name: 'RESTORES the archive heading of a section whose rendering repeats another section\'s heading',
      fn: async () => {
        const assembly = guardPageAssembly({
          artifact: settledOn({ second: '## Kitten\n\nIt wakes.', },),
          slices: SLICES,
          sourceText: SOURCE_TEXT,
          targetText: TARGET,
        },);
        expect(assembly.withdrawn,).toEqual([],);
        expect(assembly.trimmed,).toEqual([{
          sliceIndex: 1,
          replacementText: '## Tomcat\n\nIt wakes.',
        },],);
        expect(assembly.findings.join('\n',),).toContain('Kitten',);
        expect(assembly.findings.join('\n',),).toContain('Tomcat',);
      },
    },),
    it({
      name: 'LEAVES distinct headings alone',
      fn: async () => {
        const assembly = guardPageAssembly({
          artifact: settledOn({ second: '## Big Cat\n\nIt wakes.', },),
          slices: SLICES,
          sourceText: SOURCE_TEXT,
          targetText: TARGET,
        },);
        expect(assembly,).toEqual({ trimmed: [], withdrawn: [], findings: [], },);
      },
    },),
    it({
      name: 'SAYS NOTHING when the page carries fewer headings than the original, which another floor owns',
      fn: async () => {
        const assembly = guardPageAssembly({
          artifact: settledOn({ second: 'It wakes.', },),
          slices: SLICES,
          sourceText: SOURCE_TEXT,
          targetText: TARGET,
        },);
        expect(assembly.trimmed,).toEqual([],);
      },
    },),
    it({
      name: 'LEAVES THE PAGE ALONE where the original and the archive carry different numbers of headings, since no '
        + 'position pairs them (T8, twentieth batch)',
      fn: async () => {
        const assembly = guardPageAssembly({
          artifact: settledOn({ second: '## Kitten\n\nIt wakes.', },),
          slices: SLICES,
          sourceText: `${SOURCE_TEXT}\n## 小小猫\n\n它飞了。\n`,
          targetText: TARGET,
        },);
        expect(assembly,).toEqual({ trimmed: [], withdrawn: [], findings: [], },);
      },
    },),
    it({
      name: 'LEAVES THE PAGE ALONE where the original and the archive each head a section no slice covers, so the '
        + 'page\'s headings stop short of theirs and no position pairs them (T8, twentieth batch)',
      fn: async () => {
        const assembly = guardPageAssembly({
          artifact: settledOn({ second: '## Kitten\n\nIt wakes.', },),
          slices: SLICES,
          sourceText: `${SOURCE_TEXT}\n## 小小猫\n\n它飞了。\n`,
          targetText: `${TARGET}\n## Kitty\n\nIt flies.\n`,
        },);
        expect(assembly,).toEqual({ trimmed: [], withdrawn: [], findings: [], },);
      },
    },),
    it({
      name: 'LEAVES A SECTION THE PAGE DOES NOT REPLACE AS THE ARCHIVE HAS IT where one replaced slice dropped its '
        + 'heading and another carries two: the totals still match, but every heading between them sits one '
        + 'position off, and pairing by position once shipped the untouched section as empty text (ledger B83)',
      fn: async () => {
        const assembly = guardPageAssembly({
          artifact: settledEach({
            shipped: new Map([
              [
                0,
                'It sleeps.',
              ],
              [
                2,
                '## Two Cat\n\nIt runs.\n\n## Three Cat\n\nIt runs on.',
              ],
            ],),
          },),
          slices: THREE_SLICES,
          sourceText: `${THREE_SOURCE.join('\n\n',)}\n`,
          targetText: `${THREE_ARCHIVE.join('\n\n',)}\n`,
        },);
        expect(assembly,).toEqual({ trimmed: [], withdrawn: [], findings: [], },);
      },
    },),
  ],
},);
