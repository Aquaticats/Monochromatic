/**
 Tests that the page the consistency pass reads is the page the run ships, character for character, typography and the
 page-assembly passes included.

 The shipped page is what the publishing builds: the composed artifact with the consistency pass's rows in it, the page-assembly
 guard run over it, `shippableReplacements` of the artifact that carries the guard's record, and `spliceSlices` over those.
 The page the pass reads is `consistencySlicesOf`'s page for the pass's own reading and `pageOfSlices` handed the layout of
 the facts for a trial. They are compared by the text itself, not by a model of either.

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
import { NO_PAGE_ASSEMBLY, } from '@monochromatic-dev/translation-repair-artifact';
import {
  guardPageAssembly,
  shippableReplacements,
  type WouldShipSource,
} from '@monochromatic-dev/translation-repair-page-check';
import {
  type ConsistencyEdit,
  type PageLayout,
  pageOfSlices,
  type PageSlice,
  provenApostrophesInEnglish,
} from '@monochromatic-dev/translation-repair-consistency';
import {
  type ConsistencySliceFacts,
  consistencySlicesOf,
  unprovenOnPage,
} from '../../dist/final/node/index.mjs';
import {
  pickFrom,
  randomOf,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/seeded-draws.test-fixture.ts';
import { projectedCatPage, } from '@monochromatic-dev/translation-repair-run/ts/corpus-run/consistency-page.test-fixture.ts';

/**
 The archive's first section, whose curly quotation marks and apostrophes are the page's convention, and whose spelling is
 Canadian.
 */
const CONVENTION_SECTION = '## One\n\nHe said “hi” and it’s the dogs’ bowl. The colour is red.\n';

/**
 A page of three sections whose archive writes its quotation marks curly: the original and the archive.
 */
const CURLY_PAGE: {
  readonly source: string;
  readonly archive: string;
} = {
  source: '## 一\n\n窗台上有猫。\n\n## 二\n\n小猫打盹了。\n\n## 三\n\n猫在笑。\n',
  archive: [
    CONVENTION_SECTION,
    '## Two\n\nThe cats naps.\n',
    '## Three\n\nThe cat smiles.\n',
  ].join('\n',),
};

/**
 Prepares a page of the original and the archive.

 @param page - original and archive

 @returns Preparation

 @example
 ```ts
 const prepared = preparedOf({ page: CURLY_PAGE, },);
 ```
 */
function preparedOf(
  { page, }: { readonly page: typeof CURLY_PAGE; },
): PreparedDocumentPair {
  return prepareDocumentPair({
    sourceText: page.source,
    targetText: page.archive,
  },);
}

/**
 The page the publishing builds when the lanes agree on the English given and the consistency pass wrote the rows given.

 @param prepared - preparation of the page

 @param english - what each slice ships before the pass, in slice order

 @param rows - what the consistency pass wrote, by slice index

 @returns Text `spliceSlices` writes from what `shippableReplacements` gives for the artifact that carries the guard's record

 @example
 ```ts
 const text = shippedTextOf({ prepared, english, rows: [], },);
 ```
 */
function shippedTextOf(
  {
    prepared,
    english,
    rows,
  }: {
    readonly prepared: PreparedDocumentPair;
    readonly english: readonly string[];
    readonly rows: readonly { readonly sliceIndex: number; readonly text: string; }[];
  },
): string {
  /**
   The artifact as the stages compose it, before the guard.
   */
  const composed: WouldShipSource = {
    comparison: projectedCatPage({
      prepared,
      english,
    },).comparison,
    laneSelection: {
      kind: 'contested',
      slices: [],
    },
    consolidation: {
      kind: 'settled',
      slices: [],
    },
    preparation: { archiveText: prepared.targetText, },
    pageAssembly: NO_PAGE_ASSEMBLY,
    consistency: {
      groups: [],
      slices: rows,
      findings: [],
    },
  };
  /**
   What the guard made of that page.
   */
  const pageAssembly = guardPageAssembly({
    artifact: composed,
    slices: prepared.slices,
    sourceText: prepared.sourceText,
    targetText: prepared.targetText,
    archiveOriginalSpans: prepared.archiveOriginalSpans ?? [],
  },);
  return spliceSlices({
    targetText: prepared.targetText,
    slices: prepared.slices,
    replacements: shippableReplacements({
      artifact: {
        ...composed,
        pageAssembly,
      },
    },),
  },);
}

/**
 What the pass reads of each slice of a page whose lanes agree on the English given.

 @param prepared - preparation of the page

 @param english - what each slice ships, in slice order

 @returns One entry per slice

 @example
 ```ts
 const facts = factsOf({ prepared, english, },);
 ```
 */
function factsOf(
  {
    prepared,
    english,
  }: {
    readonly prepared: PreparedDocumentPair;
    readonly english: readonly string[];
  },
): readonly ConsistencySliceFacts[] {
  return consistencySlicesOf({
    prepared,
    projected: projectedCatPage({
      prepared,
      english,
    },),
    contestSlices: [],
    consolidateSlices: [],
    archiveDisputes: new Map(),
  },);
}

/**
 The page a trial reads: the slices of the facts that carry English, each with the English given or the one it carries now,
 laid out by the layout of the facts.

 @param facts - what the pass reads of each slice

 @param trial - English a slice is read with in place of the one it carries, by slice index

 @returns Page the proof of an apostrophe reads

 @example
 ```ts
 const page = trialPageOf({ facts, trial: new Map([[1, 'The cats nap.',],],), },);
 ```
 */
function trialPageOf(
  {
    facts,
    trial,
  }: {
    readonly facts: readonly ConsistencySliceFacts[];
    readonly trial: ReadonlyMap<number, string>;
  },
): ReturnType<typeof pageOfSlices> {
  /**
   The layout every slice that carries English shares.
   */
  const layout = nonNullishOrThrow(facts.find(function carriesPage({ layout: held, },): boolean {
    return held !== undefined;
  },)?.layout,);
  return pageOfSlices({
    slices: facts.flatMap(function sliceOf({
      sliceIndex,
      current,
    },): readonly PageSlice[] {
      return (current.kind === 'present')
        ? [
          {
            sliceIndex,
            english: trial.get(sliceIndex,) ?? current.text,
          },
        ]
        : [];
    },),
    layout,
  },);
}

/**
 The positions an English proves as the last character of a possessive and as the first of an elision, sorted, in a form
 that compares.

 @param english - English to read

 @returns Two lists

 @example
 ```ts
 const proven = provenOf({ english: 'The dogs’ bowl.', },);
 ```
 */
function provenOf({ english, }: { readonly english: string; },): string {
  /**
   The proven marks of the English.
   */
  const {
    trailing,
    leading,
  } = provenApostrophesInEnglish({ english, },);
  return JSON.stringify({
    trailing: [...trailing,].toSorted(function ascending(
      left,
      right,
    ): number {
      return left - right;
    },),
    leading: [...leading,].toSorted(function ascending(
      left,
      right,
    ): number {
      return left - right;
    },),
  },);
}

/**
 The wordings of the second slice that make the proof read a different page once typography has curled their quotation marks:
 each proves a different set of marks written straight and written curly (rows of the review's p25).
 */
const PROOF_CHANGING_ROWS: readonly string[] = [
  '".....,"\'s\'',
  's,*tis’"\'tis"""',
];

/**
 The wordings of the second slice whose quotation marks typography curls: the review's p25 rows. The rows that change the
 proof, and `)"'cats'‘`, which changed it until a single mark beside another single mark read free whether straight or curly
 (decision D5 of the independent-oracle round), so its marks prove the same set either way.
 */
const TYPOGRAPHY_ROWS: readonly string[] = [
  ...PROOF_CHANGING_ROWS,
  ')"\'cats\'‘',
];

/**
 The English of the slices when the lanes agree on a wording for the second.

 @param wording - second slice's wording, as the lane wrote it

 @returns English of each slice

 @example
 ```ts
 const english = englishWith({ wording: 'The cats nap.', },);
 ```
 */
function englishWith({ wording, }: { readonly wording: string; },): readonly string[] {
  return [
    CONVENTION_SECTION,
    `## Two\n\n${wording}\n`,
    '## Three\n\nThe cat smiles.\n',
  ];
}

/**
 Draws one generated page and the English the lanes agree on for it, from a seed.

 @param seed - seed

 @returns Preparation, English of each slice, and English a trial reads some slices with

 @example
 ```ts
 const drawn = drawnPageOf({ seed: 7, },);
 ```
 */
function drawnPageOf(
  { seed, }: { readonly seed: number; },
): {
  readonly prepared: PreparedDocumentPair;
  readonly english: readonly string[];
  readonly trial: ReadonlyMap<number, string>;
} {
  /**
   The generator of the seed.
   */
  const rand = randomOf({ seed, },);
  /**
   How many sections the page has.
   */
  const count = 2 + Math.floor(rand() * 3,);
  /**
   The pieces a wording is drawn from: words (Canadian and American spellings among them), quotation marks of both kinds,
   apostrophes, and emphasis.
   */
  const pieces: readonly string[] = [
    'cats',
    'dogs',
    'the',
    'bowl',
    'colour',
    'color',
    'naps',
    ' ',
    ' ',
    ' ',
    '\'',
    '"',
    '’',
    '“',
    '”',
    '‘',
    '*',
    '.',
    ', ',
  ];
  /**
   A wording of some pieces.

   @returns Wording
   */
  function wording(): string {
    /**
     How many pieces it has.
     */
    const size = 4 + Math.floor(rand() * 9,);
    return Array.from(
      { length: size, },
      function piece(): string {
        return pickFrom({
          options: pieces,
          rand,
        },);
      },
    )
      .join('',);
  }
  /**
   The original: a heading and a sentence for each section.
   */
  const source = Array.from(
    { length: count, },
    function sectionOf(
      _slot,
      at,
    ): string {
      return `## ${String(at + 1,)}\n\n窗台上有猫。\n`;
    },
  )
    .join('\n',);
  /**
   The archive: the first section is the convention.
   */
  const archive = Array.from(
    { length: count, },
    function sectionOf(
      _slot,
      at,
    ): string {
      return (at === 0) ? CONVENTION_SECTION : `## Section ${String(at + 1,)}\n\nThe cats naps${wording()}.\n`;
    },
  )
    .join('\n',);
  /**
   The preparation.
   */
  const prepared = preparedOf({
    page: {
      source,
      archive,
    },
  },);
  /**
   What each slice ships: the archive's own text for some, a drawn wording for the others.
   */
  const english = prepared.slices
    .map(function shipsOf(
      slice,
      at,
    ): string {
      return ((at === 0) || (rand() < 0.35))
        ? slice.target
          .text
        : `## Section ${String(at + 1,)}\n\n${wording()}\n`;
    },);
  /**
   What the trial reads some slices with.
   */
  const trial = new Map<number, string>(prepared.slices
    .flatMap(function trialOf(
      slice,
      at,
    ): readonly (readonly [number, string])[] {
      return ((at > 0) && (rand() < 0.6))
        ? [
          [
            slice.target
              .sliceIndex,
            `## Section ${String(at + 1,)}\n\n${wording()}\n`,
          ],
        ]
        : [];
    },),);
  return {
    prepared,
    english,
    trial,
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: consistencySlicesOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: PROOF_CHANGING_ROWS.map(function rowTest(row,) {
        return it({
          name: `READS THE GLYPHS THAT SHIP: the page of the facts is the published page when typography curls ${JSON.stringify(row,)}`,
          fn: async () => {
            const prepared = preparedOf({ page: CURLY_PAGE, },);
            const english = englishWith({ wording: row, },);
            const facts = factsOf({
              prepared,
              english,
            },);
            /**
             The page the publishing builds from the same wording.
             */
            const published = shippedTextOf({
              prepared,
              english,
              rows: [],
            },);
            /**
             The page of the first slice that carries English.
             */
            const page = nonNullishOrThrow(facts[0]?.page,);
            // The control: typography changes which marks this wording proves, so a page that reads the wording as the lane
            // wrote it proves another set than the page that ships.
            expect(provenOf({ english: english.join('\n',), },),).not.toBe(provenOf({ english: published, },),);
            expect(page.text,).toBe(published,);
            expect(provenOf({ english: page.text, },),).toBe(provenOf({ english: published, },),);
          },
        },);
      },),
    },),
    describe({
      name: pageOfSlices.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        ...TYPOGRAPHY_ROWS.map(function rowTest(row,) {
          return it({
            name: `LAYS A TRIAL OUT AS THE PUBLISHING SHIPS IT: a replacement ${JSON.stringify(row,)} that typography curls reads curly`,
            fn: async () => {
              const prepared = preparedOf({ page: CURLY_PAGE, },);
              const english = englishWith({ wording: 'The cats nap.', },);
              const facts = factsOf({
                prepared,
                english,
              },);
              /**
               What the pass writes for the second slice.
               */
              const written = `## Two\n\n${row}\n`;
              /**
               The page the proof reads with the replacement made.
               */
              const page = trialPageOf({
                facts,
                trial: new Map([[1, written,],],),
              },);
              /**
               The page the publishing builds when the pass writes that row.
               */
              const published = shippedTextOf({
                prepared,
                english,
                rows: [
                  {
                    sliceIndex: 1,
                    text: written,
                  },
                ],
              },);
              expect(page.text,).toBe(published,);
              expect(provenOf({ english: page.text, },),).toBe(provenOf({ english: published, },),);
            },
          },);
        },),
        it({
          name: 'READS THE PAGE THE PUBLISHING SHIPS CHARACTER FOR CHARACTER over generated pages: typography and the page-assembly '
            + 'passes included, for the pass\'s own reading and for a trial',
          fn: async () => {
            /**
             The pages that differ from the wording the lanes wrote, which show the generator reaches typography and the passes.
             */
            const rewritten = { count: 0, };
            for (let seed = 1; seed <= 120; seed += 1) {
              const {
                prepared,
                english,
                trial,
              } = drawnPageOf({ seed, },);
              const facts = factsOf({
                prepared,
                english,
              },);
              /**
               The page of the pass's own reading.
               */
              const own = facts.find(function carriesPage({ page, },): boolean {
                return page !== undefined;
              },)?.page;
              /**
               The page the publishing builds from the wording of the lanes.
               */
              const publishedOwn = shippedTextOf({
                prepared,
                english,
                rows: [],
              },);
              expect({ seed, reading: 'the pass\'s own', text: own?.text, },).toEqual({ seed, reading: 'the pass\'s own', text: publishedOwn, },);
              if (publishedOwn !== english.join('\n',))
                rewritten.count += 1;
              /**
               The page the proof reads with the trial's replacements.
               */
              const page = trialPageOf({
                facts,
                trial,
              },);
              /**
               The page the publishing builds when the pass writes the trial's replacements.
               */
              const publishedTrial = shippedTextOf({
                prepared,
                english,
                rows: [...trial.entries(),].map(function rowOf([
                  sliceIndex,
                  text,
                ],): { readonly sliceIndex: number; readonly text: string; } {
                  return {
                    sliceIndex,
                    text,
                  };
                },),
              },);
              expect({ seed, reading: 'the trial', text: page.text, },).toEqual({ seed, reading: 'the trial', text: publishedTrial, },);
            }
            // The control: a good share of the generated pages are rewritten by typography or a pass, or the comparisons of each
            // seed would hold for any reading that ignores them.
            expect(rewritten.count,).toBeGreaterThan(20,);
          },
        },),
      ],
    },),
    describe({
      name: unprovenOnPage.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'FINDS A CARRIED APOSTROPHE WHERE THE PAGE SHIPS IT AFTER A PASS CHANGED THE LENGTH BEFORE IT: a spelling made '
            + 'Canadian in the slice moves the mark by one, and the mark is proven there',
          fn: async () => {
            const prepared = preparedOf({ page: CURLY_PAGE, },);
            const english = englishWith({ wording: 'The cats nap.', },);
            const facts = factsOf({
              prepared,
              english,
            },);
            /**
             What the slice says once the replacement is made: an American spelling before the words that carry the mark.
             */
            const made = '## Two\n\nThe color of the dogs’ bowl.\n';
            /**
             The replacement, whose words end in the apostrophe it carries in.
             */
            const gain: ConsistencyEdit = {
              id: 'a',
              start: made.indexOf('the dogs’',),
              end: made.indexOf('the dogs’',) + 'the family'.length,
              text: 'the dogs’',
            };
            /**
             The page the pass would ship with the replacement made, for the control.
             */
            const published = shippedTextOf({
              prepared,
              english,
              rows: [
                {
                  sliceIndex: 1,
                  text: made,
                },
              ],
            },);
            expect(published,).toContain('The colour of the dogs’ bowl.',);
            expect(unprovenOnPage({
              made: [
                {
                  text: english[0] ?? '',
                  edits: [],
                },
                {
                  text: made,
                  edits: [gain,],
                },
                {
                  text: english[2] ?? '',
                  edits: [],
                },
              ],
              gained: [new Map(), new Map([['a', ['end',] as const,],],), new Map(),],
              layout: nonNullishOrThrow(facts[0]?.layout,),
              sliceIndexes: facts.map(function indexOf({ sliceIndex, },): number {
                return sliceIndex;
              },),
            },).map(function idsOf(edits,): readonly string[] {
              return edits.map(function idOf({ id, },): string {
                return id;
              },);
            },),).toEqual([[], [], [],],);
          },
        },),
        it({
          name: 'LEAVES OUT A CARRIED APOSTROPHE OF A SLICE THE GUARD TOOK BACK: the page ships the archive\'s words there, so '
            + 'the mark is on no page, and the same words the guard keeps are proven',
          fn: async () => {
            const prepared = preparedOf({ page: CURLY_PAGE, },);
            const english = englishWith({ wording: 'The cats nap.', },);
            const facts = factsOf({
              prepared,
              english,
            },);
            /**
             What the guard takes back: a note that no definition answers; and the same words without it.
             */
            const takenBack = '## Two\n\nThe dogs’ bowl[^9] is here.\n';
            const kept = '## Two\n\nThe dogs’ bowl is here.\n';
            /**
             Reads the page with the slice's words made as given.
             */
            function unprovenWith({ text, }: { readonly text: string; },): readonly (readonly string[])[] {
              /**
               The replacement, whose words end in the apostrophe it carries in.
               */
              const gain: ConsistencyEdit = {
                id: 'a',
                start: text.indexOf('The dogs’',),
                end: text.indexOf('The dogs’',) + 'The family'.length,
                text: 'The dogs’',
              };
              return unprovenOnPage({
                made: [
                  {
                    text: english[0] ?? '',
                    edits: [],
                  },
                  {
                    text,
                    edits: [gain,],
                  },
                  {
                    text: english[2] ?? '',
                    edits: [],
                  },
                ],
                gained: [new Map(), new Map([['a', ['end',] as const,],],), new Map(),],
                layout: nonNullishOrThrow(facts[0]?.layout,),
                sliceIndexes: facts.map(function indexOf({ sliceIndex, },): number {
                  return sliceIndex;
                },),
              },).map(function idsOf(edits,): readonly string[] {
                return edits.map(function idOf({ id, },): string {
                  return id;
                },);
              },);
            }
            expect([unprovenWith({ text: kept, },), unprovenWith({ text: takenBack, },),],).toEqual([
              [[], [], [],],
              [[], ['a',], [],],
            ],);
          },
        },),
        it({
          name: 'LEAVES OUT A CARRIED APOSTROPHE OF A SLICE THE PAGE WRITES NOTHING FOR: the page holds no place for the slice\'s '
            + 'words, so the mark is on no page, while the same slice written in is proven',
          fn: async () => {
            const prepared = preparedOf({ page: CURLY_PAGE, },);
            const english = englishWith({ wording: 'The cats nap.', },);
            const facts = factsOf({
              prepared,
              english,
            },);
            const layout = nonNullishOrThrow(facts[0]?.layout,);
            const writer = nonNullishOrThrow(layout.shipped,);
            /**
             The wording the second slice makes, whose words end in the apostrophe the replacement carries in.
             */
            const text = '## Two\n\nThe dogs’ bowl is here.\n';
            /**
             The replacement.
             */
            const gain: ConsistencyEdit = {
              id: 'a',
              start: text.indexOf('The dogs’',),
              end: text.indexOf('The dogs’',) + 'The family'.length,
              text: 'The dogs’',
            };
            /**
             Reads the page with the writer as given.
             */
            function unprovenWith(
              { page, }: { readonly page: PageLayout; },
            ): readonly (readonly string[])[] {
              return unprovenOnPage({
                made: [
                  {
                    text: english[0] ?? '',
                    edits: [],
                  },
                  {
                    text,
                    edits: [gain,],
                  },
                  {
                    text: english[2] ?? '',
                    edits: [],
                  },
                ],
                gained: [new Map(), new Map([['a', ['end',] as const,],],), new Map(),],
                layout: page,
                sliceIndexes: facts.map(function indexOf({ sliceIndex, },): number {
                  return sliceIndex;
                },),
              },).map(function idsOf(edits,): readonly string[] {
                return edits.map(function idOf({ id, },): string {
                  return id;
                },);
              },);
            }
            /**
             The same layout whose writer leaves the second slice out of what it writes.
             */
            const withoutSecond: PageLayout = {
              ...layout,
              shipped: function writesNoSecond({ slices, },): ReturnType<typeof writer> {
                return writer({ slices, },).filter(function isNotSecond({ sliceIndex, },): boolean {
                  return sliceIndex !== 1;
                },);
              },
            };
            expect([unprovenWith({ page: layout, },), unprovenWith({ page: withoutSecond, },),],).toEqual([
              [[], [], [],],
              [[], ['a',], [],],
            ],);
          },
        },),
      ],
    },),
  ],
},);
