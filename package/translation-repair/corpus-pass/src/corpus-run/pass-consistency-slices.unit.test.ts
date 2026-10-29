/**
 Tests for what the consistency pass reads of each slice: the original, the
 wording the slice would ship at this point, and the facts the floors need.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import { spliceSlices, } from '@monochromatic-dev/translation-repair-page-pass';
import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication';
import type { PreparedDocumentPair, } from '@monochromatic-dev/translation-repair-corpus-source';
import type { ArchiveDispute, } from '@monochromatic-dev/translation-repair-archive';
import {
  type ConsistencySliceFacts,
  consistencySlicesOf,
} from '../../dist/final/node/index.mjs';
import {
  archiveEnglishOf,
  preparedCatPage,
  projectedCatPage,
} from '@monochromatic-dev/translation-repair-run/ts/corpus-run/consistency-page.test-fixture.ts';

/**
 Reads the facts of the cat page with the fields a case replaces.

 @param prepared - preparation of the page, the cat page by default

 @param english - what each slice ships, the archive by default

 @param archiveDisputes - slices whose archive rendering was disputed

 @returns One entry per slice

 @example
 ```ts
 const facts = factsOf({},);
 ```
 */
function factsOf(
  {
    prepared = preparedCatPage(),
    english = archiveEnglishOf({ prepared, },),
    archiveDisputes = new Map<number, ArchiveDispute>(),
  }: {
    readonly prepared?: PreparedDocumentPair;
    readonly english?: readonly string[];
    readonly archiveDisputes?: ReadonlyMap<number, ArchiveDispute>;
  },
): readonly ConsistencySliceFacts[] {
  return consistencySlicesOf({
    prepared,
    projected: projectedCatPage({ prepared, english, },),
    contestSlices: [],
    consolidateSlices: [],
    archiveDisputes,
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: consistencySlicesOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS EACH SLICE\'S ORIGINAL, ITS OFFSET AND BLOCKS, and the wording it would ship: the archive\'s '
            + 'where the archive stands',
          fn: async () => {
            const prepared = preparedCatPage();
            const facts = factsOf({ prepared, },);
            const first = nonNullishOrThrow(facts[0],);
            expect({
              count: facts.length,
              sliceIndex: first.sliceIndex,
              sourceIndex: first.sourceIndex,
              sourceText: first.sourceText,
              sourceStart: first.sourceStart,
              nodes: first.nodes.length,
              current: first.current,
              lineStructured: first.lineStructured,
              disputedWordings: first.disputedWordings,
            },).toEqual({
              count: 4,
              sliceIndex: 0,
              sourceIndex: 0,
              sourceText: '## 一\n\n窗台上的阳光让小猫睡着了。所以这只猫一定是胖胖的小宝贝。',
              sourceStart: 0,
              nodes: 2,
              current: {
                kind: 'present',
                text: '## One\n\nThe sunny window put the kitten to sleep. So this cat must be a chubby little treasure.',
              },
              lineStructured: false,
              disputedWordings: [],
            },);
          },
        },),
        it({
          name: 'READS THE WORDING THE LANES SHIPPED where the archive does not stand',
          fn: async () => {
            const prepared = preparedCatPage();
            const english = [...archiveEnglishOf({ prepared, },),];
            english[1] = '## Two\n\nThe fish smell woke the kitten.';
            expect(nonNullishOrThrow(factsOf({ prepared, english, },)[1],).current,).toEqual({
              kind: 'present',
              text: '## Two\n\nThe fish smell woke the kitten.',
            },);
          },
        },),
        it({
          name: 'READS A SLICE NOTHING SHIPS AT AS CARRYING NO ENGLISH, never as an empty wording',
          fn: async () => {
            const prepared = preparedCatPage();
            const english = [...archiveEnglishOf({ prepared, },),];
            english[2] = '';
            expect(nonNullishOrThrow(factsOf({ prepared, english, },)[2],).current,).toEqual({ kind: 'absent', },);
          },
        },),
        it({
          name: 'CARRIES THE PAGE AS IT WILL BE PUBLISHED BESIDE EVERY SLICE THAT HAS ENGLISH, and where the slice starts in '
            + 'it: the archive\'s text with each slice\'s span holding what the slice ships, a span of a slice that ships '
            + 'nothing holding nothing, none for a slice that carries none',
          fn: async () => {
            const prepared = preparedCatPage();
            const english = [...archiveEnglishOf({ prepared, },),];
            english[2] = '';
            const facts = factsOf({ prepared, english, },);
            /**
             The span of the slice that ships nothing.
             */
            const gone = nonNullishOrThrow(prepared.slices[2],).target;
            /**
             What the page carries: the archive less the span of the slice that ships nothing.
             */
            const text = prepared.targetText.slice(0, gone.startOffset,) + prepared.targetText.slice(gone.endOffset,);
            expect(facts.map(function pageOf({ page, },): unknown {
              return (page === undefined) ? 'none' : { sameText: page.text === text, offset: page.offset, };
            },),).toEqual([
              { sameText: true, offset: 0, },
              { sameText: true, offset: nonNullishOrThrow(prepared.slices[1],).target.startOffset, },
              'none',
              {
                sameText: true,
                offset: nonNullishOrThrow(prepared.slices[3],).target.startOffset - (gone.endOffset - gone.startOffset),
              },
            ],);
          },
        },),
        it({
          name: 'CARRIES WHAT THE PAGE IS ASSEMBLED FROM BESIDE EVERY SLICE THAT HAS ENGLISH, the same layout for all: the '
            + 'translation, the pairs, and the empty text that goes over the span of a slice that ships nothing, so the page is '
            + 'what the code that publishes it assembles from the wording of the slices',
          fn: async () => {
            const prepared = preparedCatPage();
            const english = [...archiveEnglishOf({ prepared, },),];
            english[2] = '';
            const facts = factsOf({ prepared, english, },);
            /**
             The layout of the slices that carry English.
             */
            const layouts = facts.flatMap(function layoutOf({ layout, },) {
              return (layout === undefined) ? [] : [layout,];
            },);
            expect(layouts.length,).toBe(3,);
            expect(layouts.every(function isTheFirst(layout,): boolean {
              return layout === layouts[0];
            },),).toBe(true,);
            expect(facts.map(function holdsLayout({ layout, },): boolean {
              return layout !== undefined;
            },),).toEqual([true, true, false, true,],);
            expect(layouts[0]?.targetText,).toBe(prepared.targetText,);
            expect(layouts[0]?.chunks,).toEqual(prepared.slices,);
            expect(layouts[0]?.silent,).toEqual([{ sliceIndex: nonNullishOrThrow(prepared.slices[2],).target.sliceIndex, replacementText: '', },],);
          },
        },),
        it({
          name: 'CARRIES A PAGE THAT IS THE ONE THE PUBLISHING ASSEMBLES, character for character: the text spliceSlices writes '
            + 'from the wording of the slices that ship English and the empty text over the others',
          fn: async () => {
            const prepared = preparedCatPage();
            const english = [...archiveEnglishOf({ prepared, },),];
            english[1] = '## Two\n\nThe fish smell woke the kitten, who stretched and yawned.';
            english[2] = '';
            const facts = factsOf({ prepared, english, },);
            const layout = nonNullishOrThrow(facts[0]?.layout,);
            /**
             The page the publishing assembles from the same wording.
             */
            const published = spliceSlices({
              targetText: layout.targetText,
              slices: layout.chunks,
              replacements: [
                ...facts.flatMap(function wordingOf({ sliceIndex, current, },) {
                  return (current.kind === 'present') ? [{ sliceIndex, replacementText: current.text, },] : [];
                },),
                ...layout.silent,
              ],
            },);
            expect(nonNullishOrThrow(facts[0]?.page,).text,).toBe(published,);
          },
        },),
        it({
          name: 'PUTS THE WORDING A SLICE SHIPS INTO THE PAGE WHERE IT DIFFERS FROM THE ARCHIVE\'S, and moves the slices after it '
            + 'by the difference, so each slice stands in the page at its offset',
          fn: async () => {
            const prepared = preparedCatPage();
            const english = [...archiveEnglishOf({ prepared, },),];
            english[1] = '## Two\n\nThe fish smell woke the kitten, who stretched and yawned.';
            const facts = factsOf({ prepared, english, },);
            expect(facts.map(function holdsItsEnglish({ page, current, },): boolean {
              return (page !== undefined)
                && (current.kind === 'present')
                && (page.text.startsWith(current.text, page.offset,));
            },),).toEqual([true, true, true, true,],);
          },
        },),
        it({
          name: 'CARRIES THE TEXT NO SLICE HOLDS IN THE PAGE: a sealed span the translators\' note calls the English original '
            + 'stands between two slices, and the page the slices read holds it, so a quotation mark in it can pair',
          fn: async () => {
            /**
             The preparation of a page whose second section holds a note and a sealed quotation that opens and never closes.
             */
            const prepared = prepareDocumentPair({
              sourceText: '## 节1\n\n窗台上的阳光让小猫睡着了。\n\n## 节2\n\n别的话。\n\n## 节3\n\n窗台上的阳光让小猫睡着了。\n',
              targetText: '## Part 1\n\nThen the dogs now.\n\n## Part 2\n\n<!-- 这段话以下全部，原文都是英文，中文是反向翻译的 -->\n\n'
                + 'She said ‘fill the bowls with water.\n\n## Part 3\n\nThen feed the kittens’ now.\n',
              sealArchiveOriginal: true,
            },);
            const facts = factsOf({ prepared, },);
            expect(facts.map(function sealedIn({ page, },): unknown {
              return (page === undefined)
                ? 'none'
                : { whole: page.text === prepared.targetText, sealed: page.text.includes('She said ‘fill the bowls with water.',), };
            },),).toEqual(facts.map(function everyPageSealed(): unknown {
              return { whole: true, sealed: true, };
            },),);
            expect(facts.map(function startsAt({ page, },): unknown {
              return page?.offset;
            },),).toEqual(prepared.slices.map(function sliceStart({ target, },): number {
              return target.startOffset;
            },),);
          },
        },),
        it({
          name: 'CARRIES THE QUOTATION-LIKE REFERENCES OF THE ENGLISH OF EVERY SLICE BESIDE EVERY SLICE, each once in the order of the '
            + 'slices, whether the slice holds the reference or not and whether it stands in the page or ships no English',
          fn: async () => {
            const prepared = preparedCatPage();
            const english = [...archiveEnglishOf({ prepared, },),];
            english[1] = `${english[1] ?? ''}\nA &#39;word is here and a &#39;second one.\n`;
            english[2] = '';
            english[3] = `${english[3] ?? ''}\nA &rsquo;word and a &#39;third.\n`;
            expect(factsOf({ prepared, english, },).map(function referencesOf({ wholePageReferences, },): unknown {
              return wholePageReferences;
            },),).toEqual([
              ['&#39;', '&rsquo;',],
              ['&#39;', '&rsquo;',],
              ['&#39;', '&rsquo;',],
              ['&#39;', '&rsquo;',],
            ],);
            expect(factsOf({ prepared, },).map(function referencesOf({ wholePageReferences, },): unknown {
              return wholePageReferences;
            },),).toEqual([[], [], [], [],],);
          },
        },),
        it({
          name: 'CARRIES THE LINE-STRUCTURE FLAG OF A SLICE the preparation marks',
          fn: async () => {
            const whole = preparedCatPage();
            const prepared = { ...whole, lineStructuredSliceIndices: new Set([1,],), };
            const facts = factsOf({ prepared, english: archiveEnglishOf({ prepared: whole, },), },);
            expect(facts.map(function flagOf(entry,): boolean {
              return entry.lineStructured;
            },),).toEqual([false, true, false, false,],);
          },
        },),
        it({
          name: 'CARRIES THE WORDINGS A DISPUTED SLICE REFUSES, the archive\'s own and the stand-in that may not stand',
          fn: async () => {
            const prepared = preparedCatPage();
            const dispute: ArchiveDispute = {
              sliceIndex: 1,
              standIn: '## Two\n\nThe fish smell woke the kitten.',
              acceptedClaims: [],
              standInEligible: false,
              standInRefusal: 'unresolved',
            };
            const facts = factsOf({ prepared, archiveDisputes: new Map([[1, dispute,],],), },);
            expect(nonNullishOrThrow(facts[1],).disputedWordings.map(function textOf(wording,): string {
              return wording.text;
            },),).toEqual([
              archiveEnglishOf({ prepared, },)[1],
              '## Two\n\nThe fish smell woke the kitten.',
            ],);
            expect(nonNullishOrThrow(facts[0],).disputedWordings,).toEqual([],);
          },
        },),
        it({
          name: 'REFUSES A SLICE THE COMPARISON HAS NO ROW FOR, by name, since the comparison is derived from rows of '
            + 'that preparation',
          fn: async () => {
            const prepared = preparedCatPage();
            const projected = projectedCatPage({ prepared, english: archiveEnglishOf({ prepared, },), });
            let refusal = '';
            try {
              consistencySlicesOf({
                prepared,
                projected: { ...projected, comparison: projected.comparison.slice(0, 3,), },
                contestSlices: [],
                consolidateSlices: [],
                archiveDisputes: new Map(),
              },);
            }
            catch (error) {
              refusal = String(error,);
            }
            expect(refusal,).toBe(
              'ConsistencyInvariantError: unreachable: slice 3 of the preparation has no comparison row, '
                + 'and the comparison is derived from rows of that preparation',
            );
          },
        },),
      ],
    },),
  ],
},);
