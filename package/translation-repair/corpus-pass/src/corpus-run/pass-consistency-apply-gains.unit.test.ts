/**
 Tests for deciding, on the page with every replacement of the pass made, which
 replacements bring an apostrophe into a place that held none and keep it one:
 the others are left out, the place keeps its words, and the choice is repeated
 until it no longer changes. The page is every slice of it, so a quotation mark in
 another slice, or one that a replacement of another slice carries in, counts.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  applyConsistencyEdits,
  type ConsistencyEdit,
  emphasisDelimitersOf,
  markReadingsInEnglish,
  type PageLayout,
  provenApostrophesInEnglish,
} from '@monochromatic-dev/translation-repair-consistency';
import {
  editedStartOf,
  gainedBy,
  madeIn,
  unprovenOnPage,
  withoutUnprovenGains,
} from '../../dist/final/node/index.mjs';
import { pairsOver, } from '@monochromatic-dev/translation-repair-slice-reading/ts/span-pairs.test-fixture.ts';

/**
 The names of some edits.

 @param edits - edits
 @returns Their names, in order
 @example
 ```ts
 const names = idsOf(edits,); // ['a',]
 ```
 */
function idsOf(edits: readonly ConsistencyEdit[],): readonly string[] {
  return edits.map(function nameOf({ id, },): string {
    return id;
  },);
}

/**
 The sides a slice's code splices gain an edge apostrophe on, for one splice.

 @param id - name of the splice
 @param side - side of its words it brings the mark in on
 @returns Sides by name
 @example
 ```ts
 const gained = gainedOn({ id: 'a', side: 'end', },);
 ```
 */
function gainedOn(
  {
    id,
    side,
  }: {
    readonly id: string;
    readonly side: 'end' | 'start';
  },
): ReadonlyMap<string, readonly ('end' | 'start')[]> {
  return new Map<string, readonly ('end' | 'start')[]>([[id, [side,],],],);
}

/**
 One slice of a page for a case: its English, the replacements addressed to it and the names of those that are code
 splices.
 */
type SliceCase = {
  /**
   The English of the slice before any edit.
   */
  readonly english: string;

  /**
   The replacements in priority order.
   */
  readonly edits: readonly ConsistencyEdit[];

  /**
   The names of the replacements that are code splices, the only ones whose carried apostrophe is checked.
   */
  readonly splices: readonly string[];
};

/**
 The names of the edits kept and of the edits left out in each slice of a page, for a case to compare.

 @param slices - slices of the page, in page order
 @param between - text the page carries around the slices, blank line between each two when absent
 @returns Two lists of names for each slice
 @example
 ```ts
 const seen = namesOver({ slices: [{ english: 'Fill the family bowl.', edits: [], splices: [], },], },);
 ```
 */
function namesOver(
  { slices, between, }: { readonly slices: readonly SliceCase[]; readonly between?: readonly string[]; },
): readonly { readonly kept: readonly string[]; readonly dropped: readonly string[]; }[] {
  return withoutUnprovenGains({
    slices: slices.map(function sliceOf(
      { english, edits, splices, },
      sliceIndex,
    ) {
      return {
        sliceIndex,
        english,
        edits,
        splices: new Set(splices,),
      };
    },),
    ...(between === undefined) ? {} : { between, },
  },)
    .map(function namesIn({ kept, dropped, },) {
      return {
        kept: kept.map(function nameOf({ id, },): string {
          return id;
        },),
        dropped: dropped.map(function nameOf({ id, },): string {
          return id;
        },),
      };
    },);
}

/**
 The names of the edits kept and of the edits left out of a page of one slice.

 @param english - English before any edit
 @param edits - edits, in priority order
 @param splices - names of the edits that are code splices, only ones whose carried apostrophe is checked
 @returns Two lists of names
 @example
 ```ts
 const seen = namesOf({ english: 'Fill the family bowl.', edits: [], splices: [], },);
 ```
 */
function namesOf(
  {
    english,
    edits,
    splices,
  }: {
    readonly english: string;
    readonly edits: readonly ConsistencyEdit[];
    readonly splices: readonly string[];
  },
): { readonly kept: readonly string[]; readonly dropped: readonly string[]; } {
  return namesOver({
    slices: [
      {
        english,
        edits,
        splices,
      },
    ],
  },)[0] ?? {
    kept: [],
    dropped: [],
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: withoutUnprovenGains.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'KEEPS A SPLICE THAT BRINGS AN APOSTROPHE WHERE THE PAGE HOLDS NO PARTNER FOR IT, and one whose words carry '
            + 'none, in every order',
          fn: async () => {
            expect([
              namesOf({
                english: 'Fill the family bowl.',
                edits: [{ id: 'a', start: 5, end: 15, text: 'the parents’', },],
                splices: ['a',],
              },),
              namesOf({
                english: 'Say round the family now.',
                edits: [{ id: 'a', start: 4, end: 20, text: '’round the family', },],
                splices: ['a',],
              },),
              namesOf({
                english: '‘Fill the family bowl.',
                edits: [{ id: 'a', start: 6, end: 16, text: 'the parent', },],
                splices: ['a',],
              },),
              namesOf({ english: 'Fill the family bowl.', edits: [], splices: [], },),
            ],).toEqual([
              { kept: ['a',], dropped: [], },
              { kept: ['a',], dropped: [], },
              { kept: ['a',], dropped: [], },
              { kept: [], dropped: [], },
            ],);
          },
        },),
        it({
          name: 'LEAVES OUT A SPLICE THAT BRINGS A SINGLE MARK TO AN EDGE WHATEVER STANDS INSIDE IT, the oracle\'s pages: the words '
            + '`_cats_ʼ` hold no real emphasis (the modifier letter apostrophe is a letter to the parse), so the mark reads free on '
            + 'the page, no reader proves it, and the splice is left out; the words that replace a leading mark alone are a gain at the end',
          fn: async () => {
            /**
             The pages the oracle failed, each with the span that holds its leading mark or its word and the words that go in.
             */
            const pages = [
              ['He said 5 ʼcats 🐱*猫 now.', 10, 18,],
              ['He said ʼthe cats猫 now.', 8, 17,],
              ['He said 5 ʼcats…s now.', 10, 15,],
              ['He said “cats…ʼ5 now.', 9, 13,],
              ['He said 5 ʼcats…」s now.', 10, 15,],
              // Two of the oracle's two-edit pages, the edit of the second place alone.
              ['He said 「He said "hi" there now. He said 5 ʼcats 🐱」5 now.', 43, 51,],
              ['He said 猫(rock\'n\'roll"猫 now. He said ʼ5 cats」猫 now.', 37, 44,],
            ] as const;
            expect(pages.map(function namesOfPage([english, start, end,],) {
              return namesOf({
                english,
                edits: [{ id: 'a', start, end, text: '_cats_ʼ', },],
                splices: ['a',],
              },);
            },),).toEqual(pages.map(function leftOut() {
              return { kept: [], dropped: ['a',], };
            },),);
            expect(gainedBy({
              slice: {
                sliceIndex: 0,
                english: 'He said 5 ʼcats…s now.',
                edits: [{ id: 'a', start: 10, end: 15, text: '_cats_ʼ', },],
                splices: new Set(['a',],),
              },
            },),).toEqual(gainedOn({ id: 'a', side: 'end', },),);
          },
        },),
        it({
          name: 'KEEPS BOTH SPLICES OF A PARAGRAPH WHOSE TWO MARKS FOLLOW REAL CLOSING STRONG EMPHASIS BESIDE A LITERAL `*`, the '
            + 'oracle\'s two-edit pages: by the package\'s own parse `***cats**` holds a literal asterisk and a strong pair whose '
            + 'closing delimiters end right before the mark, a Latin letter stands before that run and what follows the mark parts '
            + 'it, so each mark reads trailing, no mark can pair with it, and each is a proven apostrophe',
          fn: async () => {
            /**
             The pages, each with the two splices that carry `**cats**` and a mark.
             */
            const pages = [
              ['He said 5 *So she was kind)猫 now. He said 猫goin) now.', [11, 26,], [43, 47,], '’',],
              ['He said 猫*cats…" now. He said 猫Come (here)) now.', [10, 14,], [31, 42,], '’',],
              ['He said *So she was kind」s now. He said 猫cats* now.', [9, 24,], [41, 45,], '\'',],
              ['He said 猫*cats 🐱"s now. He said 猫5 cats*s now.', [10, 17,], [34, 40,], '’',],
            ] as const;
            /**
             What the check keeps and drops on each page, with the page the kept splices make read by the proof.
             */
            const seen = pages.map(function readPage([english, first, second, mark,],) {
              const edits = [
                { id: 'e0', start: first[0], end: first[1], text: `**cats**${mark}`, },
                { id: 'e1', start: second[0], end: second[1], text: `**cats**${mark}`, },
              ];
              const [verdict,] = withoutUnprovenGains({
                slices: [{ sliceIndex: 0, english, edits, splices: new Set(['e0', 'e1',],), },],
              },);
              const made = applyConsistencyEdits({ english, edits: verdict?.kept ?? [], },).text;
              return {
                kept: verdict?.kept.map(function nameOf({ id, },): string {
                  return id;
                },),
                dropped: verdict?.dropped.length,
                kinds: [...markReadingsInEnglish({ english: made, },),].map(function shown([, kind,],): string {
                  return kind;
                },),
                proven: provenApostrophesInEnglish({ english: made, },).trailing.size,
              };
            },);
            expect(seen,).toEqual(pages.map(function bothKept() {
              return { kept: ['e0', 'e1',], dropped: 0, kinds: ['trailing', 'trailing',], proven: 2, };
            },),);
            // The first page, read by the parse: the literal asterisk at 10 is no delimiter, the strong pairs are at 11-12 and 17-18
            // and at 37-38 and 43-44, and the mark right after each closing pair is the one proven.
            expect([...emphasisDelimitersOf({ english: 'He said 5 ***cats**’)猫 now. He said 猫**cats**’) now.', },).offsets,]
              .toSorted(function ascending(left, right,): number {
                return left - right;
              },),).toEqual([11, 12, 17, 18, 37, 38, 43, 44,],);
          },
        },),
        it({
          name: 'KEEPS A SPLICE WHOSE REAL EMPHASIS CLOSES BEFORE THE MARK, so the mark is the edge apostrophe of the word before the '
            + 'markup and the page proves it, as it did before a mark counted whatever stood inside it',
          fn: async () => {
            expect(namesOf({
              english: 'He said 5 ʼcats…s now.',
              edits: [{ id: 'a', start: 10, end: 15, text: '*cats*ʼ', },],
              splices: ['a',],
            },),).toEqual({ kept: ['a',], dropped: [], },);
          },
        },),
        it({
          name: 'LEAVES OUT A SPLICE WHOSE CARRIED APOSTROPHE A PARTNER ON THE PAGE COULD PAIR WITH, an opener before a trailing '
            + 'mark, a closer after a leading one, in another paragraph or inside a block quote, or one of the other '
            + 'single quotation characters',
          fn: async () => {
            expect([
              namesOf({
                english: '‘Fill the family bowl.',
                edits: [{ id: 'a', start: 6, end: 16, text: 'the parents’', },],
                splices: ['a',],
              },),
              namesOf({
                english: '‘Fill the bowls.\n\nThen feed the family.',
                edits: [{ id: 'a', start: 18, end: 38, text: 'Then feed the parents’', },],
                splices: ['a',],
              },),
              namesOf({
                english: 'Say round the family now, said ‘Mio’.',
                edits: [{ id: 'a', start: 4, end: 20, text: '’round the family', },],
                splices: ['a',],
              },),
              namesOf({
                english: 'She said ‚fill the family bowl.',
                edits: [{ id: 'a', start: 15, end: 25, text: 'the parents’', },],
                splices: ['a',],
              },),
            ],).toEqual([
              { kept: [], dropped: ['a',], },
              { kept: [], dropped: ['a',], },
              { kept: [], dropped: ['a',], },
              { kept: [], dropped: ['a',], },
            ],);
          },
        },),
        it({
          name: 'LEAVES OUT BOTH SPLICES WHEN TWO GAINS ARE EACH PROVEN ALONE AND NEITHER TOGETHER, the review\'s paragraph: a '
            + 'leading gain and a trailing gain are one another\'s partner',
          fn: async () => {
            /**
             The paragraph and the two gains.
             */
            const english = 'Say round the family and fill the kittens bowl.';
            const leading = { id: 'a', start: 4, end: 20, text: '’round the family', };
            const trailing = { id: 'b', start: 30, end: 41, text: 'the kittens’', };
            expect([
              namesOf({ english, edits: [leading,], splices: ['a',], },),
              namesOf({ english, edits: [trailing,], splices: ['b',], },),
              namesOf({ english, edits: [leading, trailing,], splices: ['a', 'b',], },),
            ],).toEqual([
              { kept: ['a',], dropped: [], },
              { kept: ['b',], dropped: [], },
              { kept: [], dropped: ['a', 'b',], },
            ],);
          },
        },),
        it({
          name: 'DOES NOT CHECK A MARK THE RECEIVING WORDS ALREADY HELD, the replaced apostrophe stays where it was, and '
            + 'does not check an edit that is no splice, the block a writer rewrote',
          fn: async () => {
            expect([
              namesOf({
                english: '‘Fill the folks’ bowl.',
                edits: [{ id: 'a', start: 6, end: 16, text: 'the parents’', },],
                splices: ['a',],
              },),
              namesOf({
                english: '‘Fill the family bowl.',
                edits: [{ id: 'w', start: 6, end: 16, text: 'the parents’', },],
                splices: [],
              },),
            ],).toEqual([
              { kept: ['a',], dropped: [], },
              { kept: ['w',], dropped: [], },
            ],);
          },
        },),
        it({
          name: 'DECIDES AGAIN WHEN AN EDIT LEFT OUT LETS AN EDIT IT OVERLAPPED BE MADE, until the choice no longer changes',
          fn: async () => {
            /**
             A gain that overlaps a later edit of the same words, which the earlier edit wins unless it is left out.
             */
            const english = '‘Fill the family bowl.';
            expect(namesOf({
              english,
              edits: [
                { id: 'a', start: 6, end: 16, text: 'the parents’', },
                { id: 'b', start: 6, end: 16, text: 'the folks', },
              ],
              splices: ['a', 'b',],
            },),).toEqual({ kept: ['b',], dropped: ['a',], },);
          },
        },),
        it({
          name: 'CHECKS THE EDIT THAT TAKES THE PLACE OF ONE LEFT OUT, round after round: the edit it overlapped is made once the '
            + 'first is gone, and its own carried apostrophe is read on the page that then stands',
          fn: async () => {
            /**
             Two gains over the same words, the first of which wins while it stands; the page holds an opener for both.
             */
            const english = '‘Fill the family bowl.';
            expect(namesOf({
              english,
              edits: [
                { id: 'a', start: 6, end: 16, text: 'the parents’', },
                { id: 'b', start: 6, end: 16, text: 'the kittens’', },
              ],
              splices: ['a', 'b',],
            },),).toEqual({ kept: [], dropped: ['a', 'b',], },);
          },
        },),
        it({
          name: 'READS THE PAGE AS PROSE: the strings of a component are no partner, and an English the grammar cannot read is '
            + 'read as text where every mark could be one',
          fn: async () => {
            expect([
              namesOf({
                english: '<PhotoScroll photos={[\'a.webp\']} />\n\nFill the family bowl.',
                edits: [{ id: 'a', start: 42, end: 52, text: 'the parents’', },],
                splices: ['a',],
              },),
              namesOf({
                english: 'Say \'{ Fill the family bowl.',
                edits: [{ id: 'a', start: 12, end: 22, text: 'the parents’', },],
                splices: ['a',],
              },),
            ],).toEqual([
              { kept: ['a',], dropped: [], },
              { kept: [], dropped: ['a',], },
            ],);
          },
        },),
        it({
          name: 'READS THE WHOLE PAGE, EVERY SLICE OF IT: an opener in an earlier slice or a closer in a later one is a partner '
            + 'for the apostrophe a splice carries in, a slice with no partner beside it keeps the splice, and a slice with '
            + 'no edit is read as it stands',
          fn: async () => {
            /**
             The slice that holds the gain, a trailing mark after its words.
             */
            const gaining: SliceCase = {
              english: 'Then the family naps.',
              edits: [{ id: 'a', start: 5, end: 15, text: 'the parents’', },],
              splices: ['a',],
            };
            /**
             The slice that holds a leading gain.
             */
            const leading: SliceCase = {
              english: 'Say round the family now.',
              edits: [{ id: 'b', start: 4, end: 20, text: '’round the family', },],
              splices: ['b',],
            };
            /**
             A slice with an opener, one with a closer, and one with neither.
             */
            const opener: SliceCase = { english: '## One\n\nShe said ‘hello.', edits: [], splices: [], };
            const closer: SliceCase = { english: '## Three\n\nShe said hello’.', edits: [], splices: [], };
            const neither: SliceCase = { english: '## Two\n\nShe said hello.', edits: [], splices: [], };
            expect([
              namesOver({ slices: [opener, gaining,], },)[1],
              namesOver({ slices: [neither, gaining,], },)[1],
              namesOver({ slices: [gaining, opener,], },)[0],
              namesOver({ slices: [leading, closer,], },)[0],
              namesOver({ slices: [closer, leading,], },)[1],
            ],).toEqual([
              { kept: [], dropped: ['a',], },
              { kept: ['a',], dropped: [], },
              { kept: ['a',], dropped: [], },
              { kept: [], dropped: ['b',], },
              { kept: ['b',], dropped: [], },
            ],);
          },
        },),
        it({
          name: 'READS THE PAGE WITH EVERY EDIT OF THE PASS MADE, in every slice: a replacement that carries an opener into one '
            + 'slice leaves out the gain of a later slice, and two gains in two slices, each proven alone and neither '
            + 'together, are both left out',
          fn: async () => {
            /**
             The slice whose replacement carries an opener in.
             */
            const opening: SliceCase = {
              english: 'Fill the cats bowl.',
              edits: [{ id: 'o', start: 5, end: 13, text: '‘the cats', },],
              splices: [],
            };
            /**
             The slice with the trailing gain.
             */
            const trailing: SliceCase = {
              english: 'Then the family naps.',
              edits: [{ id: 'a', start: 5, end: 15, text: 'the kittens’', },],
              splices: ['a',],
            };
            /**
             The slice with the leading gain.
             */
            const leading: SliceCase = {
              english: 'Say round the family now.',
              edits: [{ id: 'b', start: 4, end: 20, text: '’round the family', },],
              splices: ['b',],
            };
            expect([
              namesOver({ slices: [opening, trailing,], },),
              namesOver({ slices: [leading,], },),
              namesOver({ slices: [trailing,], },),
              namesOver({ slices: [leading, trailing,], },),
            ],).toEqual([
              [{ kept: ['o',], dropped: [], }, { kept: [], dropped: ['a',], },],
              [{ kept: ['b',], dropped: [], },],
              [{ kept: ['a',], dropped: [], },],
              [{ kept: [], dropped: ['b',], }, { kept: [], dropped: ['a',], },],
            ],);
          },
        },),
        it({
          name: 'READS THE TEXT THE PAGE CARRIES BETWEEN THE SLICES, the text no slice holds: an opener in a sealed span between '
            + 'two slices is a partner for the apostrophe a splice carries in, and the same page with a blank line there is '
            + 'not',
          fn: async () => {
            /**
             The slice that holds the gain, a trailing mark after its words.
             */
            const gaining: SliceCase = {
              english: 'Then the family naps.',
              edits: [{ id: 'a', start: 5, end: 15, text: 'the parents’', },],
              splices: ['a',],
            };
            /**
             The slice before it, with no edit and no quotation mark.
             */
            const before: SliceCase = { english: '## One\n\nFill the bowls.', edits: [], splices: [], };
            /**
             What each page carries between its two slices and after the last.
             */
            const sealed = ['', '\n\n<!-- note -->\n\nShe said ‘fill it.\n\n', '\n',];
            const blank = ['', '\n\n', '\n',];
            expect([sealed, blank,].map(function over(between,): unknown {
              return namesOver({ slices: [before, gaining,], between, },);
            },),).toEqual([
              [{ kept: [], dropped: [], }, { kept: [], dropped: ['a',], },],
              [{ kept: [], dropped: [], }, { kept: ['a',], dropped: [], },],
            ],);
          },
        },),
        it({
          name: 'REFUSES TEXT BETWEEN THE SLICES THAT DOES NOT COUNT ONE MORE THAN THE SLICES, since the page built on it would '
            + 'misplace a slice',
          fn: async () => {
            const refusal = caught(function readsTooFew(): unknown {
              return withoutUnprovenGains({
                slices: [{ sliceIndex: 0, english: 'Then the family naps.', edits: [{ id: 'a', start: 5, end: 15, text: 'the parents’', },], splices: new Set(['a',],), },],
                between: [''],
              },);
            },);
            expect(String(refusal,),).toBe(
              'ConsistencyInvariantError: unreachable: the text around the slices of the page holds 1 piece for 1 slice, '
                + 'and it must hold one more piece than there are slices: the text before the first, between each two and after '
                + 'the last',
            );
          },
        },),
      ],
    },),
    describe({
      name: editedStartOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'GIVES WHERE A REPLACEMENT\'S WORDS START IN THE ENGLISH THE REPLACEMENTS MADE, moved by how much longer or shorter '
            + 'each replacement before it made the text, and not by those that stand after it',
          fn: async () => {
            /**
             A replacement that adds three characters at the start, one that removes two further on, and one after both.
             */
            const longer: ConsistencyEdit = { id: 'a', start: 0, end: 2, text: 'abcde', };
            const shorter: ConsistencyEdit = { id: 'b', start: 4, end: 8, text: 'xy', };
            const last: ConsistencyEdit = { id: 'c', start: 10, end: 12, text: 'z', };
            const edits = [shorter, last, longer,];
            expect([longer, shorter, last,].map(function startOf(edit,): number {
              return editedStartOf({ edits, edit, },);
            },),).toEqual([0, 7, 11,],);
          },
        },),
      ],
    },),
    describe({
      name: gainedBy.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES THE SIDES EACH CODE SPLICE GAINS AN EDGE APOSTROPHE ON, and none for a replacement that is no splice, '
            + 'nor for a side the words it replaces already held one',
          fn: async () => {
            const trailing: ConsistencyEdit = { id: 'a', start: 5, end: 15, text: 'the parents’', };
            const leading: ConsistencyEdit = { id: 'b', start: 20, end: 30, text: '’the family', };
            const held: ConsistencyEdit = { id: 'c', start: 40, end: 46, text: 'pups’', };
            const plain: ConsistencyEdit = { id: 'd', start: 50, end: 54, text: 'bowl', };
            /**
             The English the replacements address: the words at 40 to 46 already end in an apostrophe.
             */
            const english = `${' '.repeat(40,)}the p’${' '.repeat(10,)}`;
            expect([...gainedBy({
              slice: { sliceIndex: 0, english, edits: [trailing, leading, held, plain,], splices: new Set(['a', 'b', 'c',],), },
            },).entries(),],).toEqual([['a', ['end',],], ['b', ['start',],], ['c', [],],],);
          },
        },),
      ],
    },),
    describe({
      name: madeIn.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'MAKES THE REPLACEMENTS THAT STILL STAND AND NAMES THE ONES THAT WERE MADE, leaving the English as it is when none '
            + 'remain',
          fn: async () => {
            const edit: ConsistencyEdit = { id: 'a', start: 5, end: 15, text: 'the parents’', };
            const slice = { sliceIndex: 0, english: 'Fill the family bowl.', edits: [edit,], splices: new Set(['a',],), };
            expect([
              madeIn({ slice, remaining: [edit,], },),
              madeIn({ slice, remaining: [], },),
            ],).toEqual([
              { text: 'Fill the parents’ bowl.', edits: [edit,], },
              { text: 'Fill the family bowl.', edits: [], },
            ],);
          },
        },),
      ],
    },),
    describe({
      name: unprovenOnPage.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES THE GAINS OF ONE ROUND THAT THE PAGE DOES NOT PROVE, in each slice, in one reading of the page, and '
            + 'leaves the choice to the caller: a trailing mark after an opener, a leading mark before a closer, and the '
            + 'same marks with no partner on the page',
          fn: async () => {
            /**
             A leading gain before its slice's words, and a trailing gain after the words of the slice that follows.
             */
            const leading: ConsistencyEdit = { id: 'b', start: 4, end: 20, text: '’round the family', };
            const trailing: ConsistencyEdit = { id: 'a', start: 5, end: 15, text: 'the parents’', };
            /**
             What each gain brings an apostrophe in on.
             */
            const gains = [gainedOn({ id: 'b', side: 'start', },), gainedOn({ id: 'a', side: 'end', },),];
            /**
             The slices as the replacements made them: both gains, and each alone with the other's words as they were.
             */
            const both = [
              { text: 'Say ’round the family now.', edits: [leading,], },
              { text: 'Then the parents’ naps.', edits: [trailing,], },
            ];
            const leadingAlone = [both[0] ?? { text: '', edits: [], }, { text: 'Then the family naps.', edits: [], },];
            const trailingAlone = [{ text: 'Say round the family now.', edits: [], }, both[1] ?? { text: '', edits: [], },];
            expect([
              unprovenOnPage({ made: both, gained: gains, },).map(idsOf,),
              unprovenOnPage({ made: leadingAlone, gained: gains, },).map(idsOf,),
              unprovenOnPage({ made: trailingAlone, gained: gains, },).map(idsOf,),
            ],).toEqual([
              [['b',], ['a',],],
              [[], [],],
              [[], [],],
            ],);
          },
        },),
        it({
          name: 'READS THE TEXT BETWEEN THE SLICES WHEN IT IS GIVEN, and the slices joined by a blank line when it is not, and '
            + 'names no edit when no edit gains an apostrophe',
          fn: async () => {
            /**
             A trailing gain after its slice's words.
             */
            const trailing: ConsistencyEdit = { id: 'a', start: 5, end: 15, text: 'the parents’', };
            /**
             The slices as the replacements made them.
             */
            const made = [{ text: 'Then the parents’ naps.', edits: [trailing,], },];
            /**
             The side the gain brings its apostrophe in on.
             */
            const gains = [gainedOn({ id: 'a', side: 'end', },),];
            expect([
              unprovenOnPage({ made, gained: gains, between: ['She said ‘hello.\n\n', '',], },).map(idsOf,),
              unprovenOnPage({ made, gained: gains, between: ['', '',], },).map(idsOf,),
              unprovenOnPage({ made, gained: [new Map(),], between: ['She said ‘hello.\n\n', '',], },).map(idsOf,),
            ],).toEqual([[['a',],], [[],], [[],],],);
          },
        },),
        it({
          name: 'ASSEMBLES THE PAGE FROM THE LAYOUT WHEN IT IS GIVEN, with the code that publishes it and the slices named by the '
            + 'index the artifact gives them, so a sealed note between two slices and the span of a slice that ships nothing are '
            + 'the published page\'s, and the text between the slices is not read',
          fn: async () => {
            /**
             What a page is assembled from: three slices, the middle one shipping nothing, and a sealed note between the first
             two that holds the given mark.
             */
            function layoutWith({ mark, }: { readonly mark: string; },): PageLayout {
              const targetText = `One.\n\nShe said ${mark}hello.\n\nGone.\n\nThen old.`;
              const goneAt = targetText.indexOf('Gone.',);
              const thenAt = targetText.indexOf('Then old.',);
              return {
                targetText,
                chunks: pairsOver({ targetText, spans: [[0, 4,], [goneAt, goneAt + 5,], [thenAt, targetText.length,],], },),
                silent: [{ sliceIndex: 1, replacementText: '', },],
              };
            }
            /**
             A trailing gain after its slice's words, in the slice the artifact names two.
             */
            const trailing: ConsistencyEdit = { id: 'a', start: 5, end: 15, text: 'the parents’', };
            const made = [{ text: 'One.', edits: [], }, { text: 'Then the parents’ naps.', edits: [trailing,], },];
            const gained = [new Map<string, readonly ('end' | 'start')[]>(), gainedOn({ id: 'a', side: 'end', },),];
            expect([
              unprovenOnPage({ made, gained, layout: layoutWith({ mark: '‘', },), sliceIndexes: [0, 2,], between: ['', '', '',], },)
                .map(idsOf,),
              unprovenOnPage({ made, gained, layout: layoutWith({ mark: '', },), sliceIndexes: [0, 2,], },).map(idsOf,),
            ],).toEqual([[[], ['a',],], [[], [],],],);
          },
        },),
      ],
    },),
  ],
},);
