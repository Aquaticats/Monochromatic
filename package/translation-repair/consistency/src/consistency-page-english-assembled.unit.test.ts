/**
 Tests that the page the proof of an apostrophe reads is the page the run publishes, character for character, over
 generated pages: sealed text no slice holds, anchors that insert a slice, spans that keep a blank line at an edge, slices
 that ship no English, front matter at the start, and documents that use `\r\n`.

 The published page is the text `spliceSlices` assembles from the replacements the publishing hands it, and the proof's page
 is `pageOfSlices` handed the layout of the same page and the English of the slices that ship any. They are compared by the
 text itself and by where each slice's words stand, not by a model of either.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type SliceReplacement,
  spliceSlices,
} from '@monochromatic-dev/translation-repair-page-pass';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading';
import {
  pageOfSlices,
  type PageSlice,
} from '../dist/final/node/index.mjs';
import { anchorAt, } from '@monochromatic-dev/translation-repair-slice-reading/ts/insertion-anchor-pair.test-fixture.ts';
import {
  pickFrom,
  randomOf,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/seeded-draws.test-fixture.ts';
import { pairsOver, } from '@monochromatic-dev/translation-repair-slice-reading/ts/span-pairs.test-fixture.ts';

/**
 Where one slice of a generated page stands in its translation and what it ships.
 */
type Placement = {
  /**
   Whether the slice is a span of the translation or an anchor between two places in it.
   */
  readonly kind: 'span' | 'anchor';

  /**
   Where the slice starts in the translation.
   */
  readonly start: number;

  /**
   Where the slice ends in the translation; an anchor's end is its start.
   */
  readonly end: number;

  /**
   The English the slice ships; absent when it ships none.
   */
  readonly english?: string;
};

/**
 A generated page and what the publishing hands the assembler for it.
 */
type Generated = {
  /**
   The translation the slices were cut from.
   */
  readonly targetText: string;

  /**
   The slice pairs in document order.
   */
  readonly chunks: readonly ChunkPair[];

  /**
   The slices that ship English, in page order.
   */
  readonly shipped: readonly PageSlice[];

  /**
   What the slices that ship no English write.
   */
  readonly silent: readonly SliceReplacement[];

  /**
   Which kinds of content the page holds.
   */
  readonly holds: ReadonlySet<string>;
};

/**
 The words a slice may ship, some with a quotation mark and some with an apostrophe.
 */
const WORDS = [
  'Fill the bowl.',
  'He said ‘hi there.',
  'The kittens’ bowl.',
  'Rock ’n’ roll.',
  'Then it napped.',
];

/**
 Generates a page.

 @param seed - what the page is generated from

 @returns Translation, its slices, and what the publishing hands the assembler

 @example
 ```ts
 const { targetText, chunks, shipped, silent, } = pageFor({ seed: 7, },);
 ```
 */
function pageFor({ seed, }: { readonly seed: number; },): Generated {
  const rand = randomOf({ seed, },);
  /**
   What the page is made of, in order, and the kinds of content it holds, both mutated only inside this function.
   */
  const target = { text: '', };
  const placements: Placement[] = [];
  const holds = new Set<string>();
  /**
   The line ending of the document.
   */
  const eol = (rand() < 0.3) ? '\r\n' : '\n';
  if (eol === '\r\n')
    holds.add('windows',);
  /**
   Whether the first slice that ships English opens with front matter.
   */
  const front = { pending: rand() < 0.4, };
  /**
   How many slices the page has.
   */
  const count = 2 + Math.floor(rand() * 5,);
  for (let at = 0; at < count; at += 1) {
    if (at > 0)
      target.text += eol.repeat(2,);
    if (rand() < 0.25) {
      target.text += `<!-- sealed ‘note ${String(at,)} -->${eol.repeat(2,)}`;
      holds.add('sealed',);
    }
    /**
     The English the slice ships, none when it ships none.
     */
    const ships = rand() < 0.75;
    /**
     Whether the slice is an anchor, between two places in the translation.
     */
    const isAnchor = rand() < 0.3;
    /**
     The words, with front matter before them for the first slice that ships any.
     */
    const words = (front.pending && ships)
      ? `---\ntitle: ‘it’s ${String(at,)}\n---\n\n${pickFrom({ options: WORDS, rand, },)}`
      : pickFrom({ options: WORDS, rand, },);
    if (ships)
      front.pending = false;
    if (isAnchor) {
      /**
       The blank lines and the line ending the English carries at its edges.
       */
      const lead = pickFrom({ options: ['', '\n\n',], rand, },);
      const trail = pickFrom({ options: ['', '\n',], rand, },);
      placements.push({
        kind: 'anchor',
        start: target.text.length,
        end: target.text.length,
        ...(ships ? { english: `${lead}${words}${trail}`, } : {}),
      },);
      holds.add('anchor',);
      continue;
    }
    /**
     The line endings the span keeps at its edges, which the English may carry more or fewer of.
     */
    const keptLead = pickFrom({ options: ['', eol, eol.repeat(2,),], rand, },);
    const keptTrail = pickFrom({ options: ['', eol,], rand, },);
    /**
     The line endings the English carries at its edges.
     */
    const lead = pickFrom({ options: ['', '\n', '\r\n\r\n', '\n\n\n',], rand, },);
    const trail = pickFrom({ options: ['', '\n', '\r\n',], rand, },);
    if ((keptLead !== '') || (keptTrail !== '') || (lead !== '') || (trail !== ''))
      holds.add('edges',);
    if (!ships)
      holds.add('silent',);
    /**
     Where the span starts.
     */
    const start = target.text.length;
    target.text += `${keptLead}Old words ${String(at,)}.${keptTrail}`;
    placements.push({
      kind: 'span',
      start,
      end: target.text.length,
      ...(ships ? { english: `${lead}${words}${trail}`, } : {}),
    },);
  }
  /**
   The translation, once every piece is in.
   */
  const targetText = target.text;
  /**
   The pair of each slice, indexed by its place.
   */
  const chunks = placements.map(function chunkOf(
    placement,
    sliceIndex,
  ): ChunkPair {
    if (placement.kind === 'anchor') {
      return anchorAt({
        sliceIndex,
        offset: placement.start,
      },);
    }
    /**
     The pair over the span.
     */
    const [pair,] = pairsOver({
      targetText,
      spans: [[placement.start, placement.end,],],
      firstIndex: sliceIndex,
    },);
    if (pair === undefined)
      throw new Error('a span gave no pair',);
    return pair;
  },);
  return {
    targetText,
    chunks,
    shipped: placements.flatMap(function shippedBy(
      { english, },
      sliceIndex,
    ): readonly PageSlice[] {
      return (english === undefined) ? [] : [{ sliceIndex, english, },];
    },),
    silent: placements.flatMap(function silentBy(
      { kind, english, },
      sliceIndex,
    ): readonly SliceReplacement[] {
      // What the publishing writes for a slice that ships no English: nothing at an anchor, an empty text over a span.
      return ((english !== undefined) || (kind === 'anchor')) ? [] : [{ sliceIndex, replacementText: '', },];
    },),
    holds,
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: `${pageOfSlices.name} against the page ${spliceSlices.name} assembles`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS THE PAGE THE RUN PUBLISHES, character for character: the text is the assembled text but for the front '
            + 'matter, which is blanked in place, and each slice`s words stand where the page says they start',
          fn: async () => {
            /**
             What differs between the two pages, by seed.
             */
            const violations: string[] = [];
            /**
             The kinds of content the generated pages held, mutated only inside this function.
             */
            const seen = new Set<string>();
            for (let seed = 1; seed <= 400; seed += 1) {
              const {
                targetText,
                chunks,
                shipped,
                silent,
                holds,
              } = pageFor({ seed, },);
              for (const kind of holds)
                seen.add(kind,);
              /**
               The page the publishing assembles.
               */
              const published = spliceSlices({
                targetText,
                slices: chunks,
                replacements: [
                  ...shipped.map(function replacementOf({ sliceIndex, english, },): SliceReplacement {
                    return { sliceIndex, replacementText: english, };
                  },),
                  ...silent,
                ],
              },);
              /**
               The page the proof reads.
               */
              const { text, offsets, } = pageOfSlices({
                slices: shipped,
                layout: { targetText, chunks, silent, },
              },);
              if (text.length !== published.length) {
                violations.push(`seed ${String(seed,)}: the proof's page is ${String(text.length,)} characters and the published page ${String(published.length,)}`,);
                continue;
              }
              /**
               The front matter of the first slice that ships English, which the proof's page blanks in place.
               */
              const firstWords = shipped[0]?.english.trim() ?? '';
              /**
               Whether the first slice opens with a fenced block.
               */
              const hasBlock = firstWords.startsWith('---\n',);
              /**
               How long the block is: up to and including the closing fence, zero where there is none.
               */
              const blockLength = hasBlock ? (firstWords.indexOf('\n---',) + 4) : 0;
              /**
               Where that block stands in the published page; an empty range where the page has no front matter.
               */
              const blockAt = hasBlock
                ? published.indexOf(firstWords.slice(
                  0,
                  blockLength,
                ),)
                : 0;
              /**
               Where the block ends.
               */
              const blockEnd = blockAt + blockLength;
              for (let at = 0; at < published.length; at += 1) {
                if (text.charAt(at,) === published.charAt(at,))
                  continue;
                /**
                 Whether this place is a blanked place of the front matter.
                 */
                const isBlanked = (at >= blockAt) && (at < blockEnd) && [' ', '\r', '\n',].includes(text.charAt(at,),);
                if (!isBlanked) {
                  violations.push(`seed ${String(seed,)}: the pages differ at ${String(at,)}`,);
                  break;
                }
              }
              if (hasBlock)
                seen.add('front matter',);
              for (const { sliceIndex, english, } of shipped) {
                /**
                 The words the slice ships, and how many characters of line endings and blank lines come before them.
                 */
                const core = english.trim();
                const lead = english.length - english.trimStart().length;
                /**
                 Where the proof says the slice's English starts.
                 */
                const offset = offsets.get(sliceIndex,);
                if (offset === undefined) {
                  violations.push(`seed ${String(seed,)}: slice ${String(sliceIndex,)} ships English and has no start`,);
                  continue;
                }
                if (published.slice(
                  offset + lead,
                  offset + lead + core.length,
                ) !== core) {
                  violations.push(`seed ${String(seed,)}: slice ${String(sliceIndex,)} does not stand at ${String(offset + lead,)} in the published page`,);
                }
              }
              if ([...offsets.keys(),].length !== shipped.length)
                violations.push(`seed ${String(seed,)}: the proof gives a start to a slice that ships no English`,);
            }
            expect(violations,).toEqual([],);
            // The generated pages held every kind of content the property is about, so a pass is not an empty one.
            expect(['anchor', 'edges', 'front matter', 'sealed', 'silent', 'windows',].map(function wasHeld(kind,): string {
              return `${kind}: ${String(seen.has(kind,),)}`;
            },),).toEqual([
              'anchor: true',
              'edges: true',
              'front matter: true',
              'sealed: true',
              'silent: true',
              'windows: true',
            ],);
          },
        },),
      ],
    },),
  ],
},);
