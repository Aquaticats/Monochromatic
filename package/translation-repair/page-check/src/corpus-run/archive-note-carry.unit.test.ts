/**
 Guards the page against losing a note between translators by the luck of a
 vote (XingZ60 slice 6, 2026-10-19). The archive's English ended a sentence
 with an HTML comment asking the other translators whether anyone understood
 the Chinese before it. The page carried that comment only because the
 winning wording happened to keep it: the repair lane keeps comments as markup
 and the translate lane does not, so a page lost or kept such a note by which
 lane won. A reader never sees an HTML comment, but the translators who open
 the page next do, and a note that opens a sealed span (`hakureico`'s) steers
 the pass that reads the page after this one.

 Whatever wording wins, the page carries each note the archive's slice holds
 and the shipped text lacks. Fixtures are cat-themed invention.

 @module
 */

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
 Note the translators left each other beside the sentence it is about.
 */
const NOTE = '<!-- 前面这句猫话懂什么意思，先这么翻译，大家看一下 -->';

/**
 Note an English-speaking translator left, which is not a Han note.
 */
const ENGLISH_NOTE = '<!-- check the cat\'s name -->';

/**
 Original passage.
 */
const SOURCE = '猫睡在窗台上。\n';

/**
 One content slice over a passage whose archive ends in the given comments.

 @param archive - archive's text for the slice

 @returns Preparation's one pair

 @example
 ```ts
 const slices = slicesOver({ archive: 'The cat slept.', },);
 ```
 */
function slicesOver({ archive, }: { readonly archive: string; },): readonly ChunkPair[] {
  return [{
    source: {
      kind: 'content',
      sliceIndex: 0,
      nodes: [],
      startOffset: 0,
      endOffset: SOURCE.length,
      text: SOURCE,
    },
    target: {
      kind: 'content',
      sliceIndex: 0,
      nodes: [],
      startOffset: 0,
      endOffset: archive.length,
      text: archive,
    },
  },];
}

/**
 Builds a source whose consolidation shipped the given wording at slice 0.

 @param archive - archive's text for the slice

 @param text - what the consolidation shipped there

 @returns Narrow artifact source read by the publication assembler

 @example
 ```ts
 const artifact = shipping({ archive, text: 'The cat napped.', },);
 ```
 */
function shipping({ archive, text, }: { readonly archive: string; readonly text: string; },): WouldShipSource {
  return {
    comparison: [{
      sliceIndex: 0,
      incumbentKind: 'present',
      incumbentText: archive,
      repairText: archive,
      translateText: text,
      laneRelation: 'both-differ',
      repairOutcome: { kind: 'decided', acceptedText: archive, },
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
 Runs the page assembly over one slice and reads the text it carries.

 @param archive - archive's text for the slice

 @param text - what the consolidation shipped there

 @returns Slice's rewritten texts (none when the passes left it as it was)
 and the findings

 @example
 ```ts
 const { carried, findings, } = pageCarrying({ archive, text, },);
 ```
 */
function pageCarrying(
  { archive, text, }: { readonly archive: string; readonly text: string; },
): { readonly carried: readonly string[]; readonly findings: readonly string[]; } {
  const assembly = guardPageAssembly({
    artifact: shipping({ archive, text, },),
    slices: slicesOver({ archive, },),
    sourceText: SOURCE,
    targetText: archive,
  },);
  return {
    carried: assembly.trimmed
      .filter(function atSlice(row,): boolean {
        return row.sliceIndex === 0;
      },)
      .map(function textOf(row,): string {
        return row.replacementText;
      },),
    findings: assembly.findings,
  };
}

await describe({
  name: 'the page carries the archive\'s notes between translators whichever wording wins',
  children: [
    it({
      name: 'CARRIES a Han note the shipped wording lacks, on a line of its own after the wording',
      fn: async () => {
        const { carried, findings, } = pageCarrying({
          archive: `The cat slept on the sill. ${NOTE}`,
          text: 'The cat napped on the sill.',
        },);
        expect(carried,).toEqual([`The cat napped on the sill.\n\n${NOTE}`,],);
        expect(findings,).toEqual([`archive-note-carried (slice 0: ${JSON.stringify(NOTE,)})`,],);
      },
    },),
    it({
      name: 'DOES NOT repeat a note the shipped wording already carries, and LEAVES a note in English alone',
      fn: async () => {
        const kept = pageCarrying({
          archive: `The cat slept on the sill. ${NOTE}`,
          text: `The cat napped on the sill. ${NOTE}`,
        },);
        expect(kept.carried,).toEqual([],);
        const english = pageCarrying({
          archive: `The cat slept on the sill. ${ENGLISH_NOTE}`,
          text: 'The cat napped on the sill.',
        },);
        expect(english.carried,).toEqual([],);
      },
    },),
    it({
      name: 'CARRIES every note the wording lacks in the archive\'s order, and keeps the whitespace that ends the text',
      fn: async () => {
        const second = '<!-- 猫窝：Cat Bed -->';
        const { carried, } = pageCarrying({
          archive: `The cat slept. ${NOTE}\n\n${second}\n`,
          text: 'The cat napped.\n',
        },);
        expect(carried,).toEqual([`The cat napped.\n\n${NOTE}\n\n${second}\n`,],);
      },
    },),
    it({
      name: 'CARRIES NOTHING onto a slice the page removes, and NOTHING into a span sealed as the English original',
      fn: async () => {
        const archive = `The cat slept. ${NOTE}`;
        expect(pageCarrying({ archive, text: '', },).carried,).toEqual([],);
        const sealed = guardPageAssembly({
          artifact: shipping({ archive, text: 'The cat napped.', },),
          slices: slicesOver({ archive, },),
          sourceText: SOURCE,
          targetText: archive,
          archiveOriginalSpans: [{ startOffset: 0, endOffset: archive.length, note: 'seal', },],
        },);
        expect(sealed.trimmed.map(function textOf(row,): string {
          return row.replacementText;
        },),).not.toContain(`The cat napped.\n\n${NOTE}`,);
      },
    },),
  ],
},);
