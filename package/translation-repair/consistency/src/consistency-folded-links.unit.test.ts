/**
 Tests for the links of an English as the fold holds them (`consistency-folded-links.ts`, n166 finding Z9 and n168): where the
 last unit of a link that shows nothing stands, where its first unit stands, and the links that show marks and no word.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  foldedLinksOf,
  foldWithOffsets,
  NO_FOLDED_LINKS,
} from '../dist/final/node/index.mjs';

/**
 The links of an English as its fold holds them, as lists of pairs and offsets that print in a test.

 @param english - English, as it was given

 @returns Pairs of last and first unit of the empty links, and the last units of the links of marks, each sorted

 @example
 ```ts
 const { empty, } = linksOf({ english: ' [](u)\'kit', },); // [[6, 1]]
 ```
 */
function linksOf(
  { english, }: { readonly english: string; },
): { readonly empty: readonly (readonly [number, number])[]; readonly marks: readonly number[]; } {
  /**
   What the fold makes of the links of the English.
   */
  const links = foldedLinksOf({
    english,
    folded: foldWithOffsets({ text: english, },),
  },);
  return {
    empty: [...links.emptyStarts.entries(),].toSorted(function byEnd([first,], [second,],): number {
      return first - second;
    },),
    marks: [...links.markOnlyEnds,].toSorted(function byOffset(first, second,): number {
      return first - second;
    },),
  };
}

await describe({
  name: foldedLinksOf.name,
  concurrency: 1,
  children: [
    it({
      name: 'MAPS the last unit of an empty link to its first unit, for a link, a link with a title, a reference and an image',
      fn: async () => {
        expect([
          linksOf({ english: ' [](u)\'kit', },).empty,
          linksOf({ english: ' [](u "t")\'kit', },).empty,
          linksOf({ english: ' []()\'kit', },).empty,
          linksOf({ english: ' ![](u)\'kit', },).empty,
          linksOf({ english: ' [][r]\'kit\n\n[r]: u', },).empty,
        ],).toEqual([
          [[5, 1,],],
          [[9, 1,],],
          [[4, 1,],],
          [[6, 1,],],
          [[5, 1,],],
        ],);
      },
    },),

    it({
      name: 'MAPS IN THE OFFSETS OF THE FOLD, not of the English, where a run of white space folds to one unit',
      fn: async () => {
        expect(linksOf({ english: 'cat   [](u)\'kit', },).empty,).toEqual([[8, 4,],],);
      },
    },),

    it({
      name: 'MAPS EVERY LINK of a run of empty links, each to its own first unit',
      fn: async () => {
        expect(linksOf({ english: 'kit[](u)![](v)[]()\'kit', },).empty,).toEqual([[7, 3,], [13, 8,], [17, 14,],],);
      },
    },),

    it({
      name: 'LISTS THE LAST UNIT of a link that shows marks or spaces and no letter or digit, and of none that shows a word, a closing bracket '
        + 'or nothing',
      fn: async () => {
        expect([
          linksOf({ english: ' [ ](u)\'kit', },).marks,
          linksOf({ english: ' [-](u)\'kit', },).marks,
          linksOf({ english: ' [_](u "t")\'kit', },).marks,
          linksOf({ english: ' ![ ](u)\'kit', },).marks,
          linksOf({ english: ' [x](u)\'kit', },).marks,
          linksOf({ english: String.raw` [\]](u)'kit`, },).marks,
          linksOf({ english: ' [](u)\'kit', },).marks,
        ],).toEqual([[6,], [6,], [10,], [7,], [], [], [],],);
      },
    },),

    it({
      name: 'READS NO LINK where no closing square bracket stands in the English, since a link has one, and where the page grammar '
        + 'refuses the English (n169)',
      fn: async () => {
        expect([
          linksOf({ english: ' kit \'kit', },),
          linksOf({ english: ' (u) \u{201D}kit', },),
          linksOf({ english: ' [](<u)\'kit', },),
        ],).toEqual([
          { empty: [], marks: [], },
          { empty: [], marks: [], },
          { empty: [], marks: [], },
        ],);
        expect(NO_FOLDED_LINKS.emptyStarts.size,).toBe(0,);
        expect(NO_FOLDED_LINKS.emptyEnds.size,).toBe(0,);
        expect(NO_FOLDED_LINKS.closingSpans.size,).toBe(0,);
        expect(NO_FOLDED_LINKS.markOnlyEnds.size,).toBe(0,);
      },
    },),

    it({
      name: 'READS THE LINKS WHEREVER A CLOSING SQUARE BRACKET STANDS, not only before an apostrophe, since a joint of any kind may follow '
        + 'or precede an empty link (n169)',
      fn: async () => {
        expect([
          linksOf({ english: ' [](u) kit', },),
          linksOf({ english: ' [](u)\u{201D}kit', },),
          linksOf({ english: 'kit[](u)-kit', },),
          linksOf({ english: '1[](u).5', },),
        ],).toEqual([
          { empty: [[5, 1,],], marks: [], },
          { empty: [[5, 1,],], marks: [], },
          { empty: [[7, 3,],], marks: [], },
          { empty: [[5, 1,],], marks: [], },
        ],);
      },
    },),

    it({
      name: 'MAPS THE FIRST UNIT OF AN EMPTY LINK to the offset after its last unit, for a link, a reference, an image and a run of them (n169)',
      fn: async () => {
        /**
         The first unit and the offset after the last unit of each empty link of an English, sorted by first unit.
         */
        function endsOf({ english, }: { readonly english: string; },): readonly (readonly [number, number])[] {
          return [
            ...foldedLinksOf({
              english,
              folded: foldWithOffsets({ text: english, },),
            },).emptyEnds.entries(),
          ].toSorted(function byFirst([first,], [second,],): number {
            return first - second;
          },);
        }
        expect([
          endsOf({ english: ' [](u)\'kit', },),
          endsOf({ english: ' [](u "t")\'kit', },),
          endsOf({ english: ' ![](u)\'kit', },),
          endsOf({ english: ' [][r]\'kit\n\n[r]: u', },),
          endsOf({ english: 'kit[](u)![](v)[]()\'kit', },),
          endsOf({ english: 'cat   [](u)\'kit', },),
          endsOf({ english: ' [x](u)\'kit', },),
        ],).toEqual([
          [[1, 6,],],
          [[1, 10,],],
          [[1, 7,],],
          [[1, 6,],],
          [[3, 8,], [8, 14,], [14, 18,],],
          [[4, 9,],],
          [],
        ],);
      },
    },),

    it({
      name: 'MAPS THE FIRST UNIT OF A LINK that shows closing marks and nothing else to the offset after its last unit, and no other link (n169)',
      fn: async () => {
        /**
         The first unit and the offset after the last unit of each link of closing marks of an English, sorted by first unit.
         */
        function spansOf({ english, }: { readonly english: string; },): readonly (readonly [number, number])[] {
          return [
            ...foldedLinksOf({
              english,
              folded: foldWithOffsets({ text: english, },),
            },).closingSpans.entries(),
          ].toSorted(function byFirst([first,], [second,],): number {
            return first - second;
          },);
        }
        expect([
          spansOf({ english: String.raw` [\]](u)'kit`, },),
          spansOf({ english: ' [\u{201D}](u)\'kit', },),
          spansOf({ english: ' [))](u)\'kit', },),
          spansOf({ english: ' [()](u)\'kit', },),
          spansOf({ english: ' [ ](u)\'kit', },),
          spansOf({ english: ' [x](u)\'kit', },),
          spansOf({ english: ' [](u)\'kit', },),
        ],).toEqual([
          [[1, 8,],],
          [[1, 7,],],
          [[1, 8,],],
          [],
          [],
          [],
          [],
        ],);
      },
    },),

    it({
      name: 'READS A REFERENCE WITH AN EMPTY LABEL AS A LINK where the definitions of the page hold its label, and as text where they do not (n169)',
      fn: async () => {
        /**
         The English, which holds the reference and no definition.
         */
        const english = ' [][r]\'kit';
        /**
         The English in the fold.
         */
        const folded = foldWithOffsets({ text: english, },);
        expect([
          [...foldedLinksOf({ english, folded, definitions: '[r]: u', },).emptyStarts.entries(),],
          [...foldedLinksOf({ english, folded, definitions: '[x]: u', },).emptyStarts.entries(),],
          [...foldedLinksOf({ english, folded, },).emptyStarts.entries(),],
        ],).toEqual([[[5, 1,],], [], [],],);
      },
    },),

    it({
      name: 'LEAVES OUT A LINK whose first or last character the fold holds no unit for, or whose first unit stands after its last, and '
        + 'keeps the others',
      fn: async () => {
        /**
         The English: two empty links and a link of marks, each before an apostrophe.
         */
        const english = ' [](u)\'a [](v)\'b [-](w)\'c';
        /**
         The English in the fold, which holds a unit for every character of it.
         */
        const whole = foldWithOffsets({ text: english, },);
        /**
         The links of the English as the fold holds them when the offsets of the units are changed.

         @param changes - for each unit to change, the offset of the English it stands for in place of its own

         @returns Pairs of last and first unit of the empty links, and the last units of the links of marks
         */
        function linksWith(
          { changes, }: { readonly changes: ReadonlyMap<number, number>; },
        ): { readonly empty: readonly (readonly [number, number])[]; readonly marks: readonly number[]; } {
          /**
           What the fold makes of the links.
           */
          const links = foldedLinksOf({
            english,
            folded: {
              ...whole,
              starts: whole.starts.map(function changed(
                start,
                unit,
              ): number {
                return changes.get(unit,) ?? start;
              },),
            },
          },);
          return {
            empty: [...links.emptyStarts.entries(),].toSorted(function byEnd([first,], [second,],): number {
              return first - second;
            },),
            marks: [...links.markOnlyEnds,],
          };
        }
        expect(linksWith({ changes: new Map(), },),).toEqual({ empty: [[5, 1,], [13, 9,],], marks: [22,], },);
        expect(linksWith({ changes: new Map([[1, 99,],],), },),).toEqual({ empty: [[13, 9,],], marks: [22,], },);
        expect(linksWith({ changes: new Map([[13, 99,],],), },),).toEqual({ empty: [[5, 1,],], marks: [22,], },);
        expect(linksWith({ changes: new Map([[22, 99,],],), },),).toEqual({ empty: [[5, 1,], [13, 9,],], marks: [], },);
        expect(linksWith({ changes: new Map([[1, 99,], [6, 1,],],), },),).toEqual({ empty: [[13, 9,],], marks: [22,], },);
      },
    },),

    it({
      name: 'LEAVES OUT A LINK OF CLOSING MARKS AND A STRETCH OF A LINK WITH A LABEL whose first or last character the fold holds no '
        + 'unit for, or whose first unit stands after its last, and keeps the others (n169, n173)',
      fn: async () => {
        /**
         The English: a link of closing marks and a link with a label, each before an apostrophe.
         */
        const english = ' [)](u)\'a [kit](v)\'b';
        /**
         The English in the fold, which holds a unit for every character of it.
         */
        const whole = foldWithOffsets({ text: english, },);
        /**
         The closing links and the stretches that show nothing as the fold holds them when the offsets of the units are changed.

         @param changes - for each unit to change, the offset of the English it stands for in place of its own

         @returns Closing links by first unit, and the hidden stretches by last unit and by first unit
         */
        function linksWith({ changes, }: { readonly changes: ReadonlyMap<number, number>; },): {
          readonly closing: readonly (readonly [number, number])[];
          readonly starts: readonly (readonly [number, number])[];
          readonly ends: readonly (readonly [number, number])[];
        } {
          /**
           What the fold makes of the links.
           */
          const links = foldedLinksOf({
            english,
            folded: {
              ...whole,
              starts: whole.starts.map(function changed(
                start,
                unit,
              ): number {
                return changes.get(unit,) ?? start;
              },),
            },
          },);
          return {
            closing: [...links.closingSpans.entries(),],
            starts: [...links.hiddenStarts.entries(),].toSorted(function byLast([first,], [second,],): number {
              return first - second;
            },),
            ends: [...links.hiddenEnds.entries(),].toSorted(function byFirst([first,], [second,],): number {
              return first - second;
            },),
          };
        }
        // The opening bracket stands at offset 10 and the resource `](v)` from 14 to 17; the closing link at 1 to 6.
        expect(linksWith({ changes: new Map(), },),).toEqual({
          closing: [[1, 7,],],
          starts: [[10, 10,], [17, 14,],],
          ends: [[10, 11,], [14, 18,],],
        },);
        // A closing link whose first unit stands after its last.
        expect(linksWith({ changes: new Map([[1, 6,], [6, 1,],],), },),).toEqual({
          closing: [],
          starts: [[10, 10,], [17, 14,],],
          ends: [[10, 11,], [14, 18,],],
        },);
        // A resource whose first unit stands after its last.
        expect(linksWith({ changes: new Map([[14, 17,], [17, 14,],],), },),).toEqual({
          closing: [[1, 7,],],
          starts: [[10, 10,],],
          ends: [[10, 11,],],
        },);
        // A resource the fold holds no unit for at its last character, and one it holds none for at its first.
        expect(linksWith({ changes: new Map([[17, 99,],],), },),).toEqual({
          closing: [[1, 7,],],
          starts: [[10, 10,],],
          ends: [[10, 11,],],
        },);
        expect(linksWith({ changes: new Map([[14, 99,],],), },),).toEqual({
          closing: [[1, 7,],],
          starts: [[10, 10,],],
          ends: [[10, 11,],],
        },);
      },
    },),

    it({
      name: 'MAPS THE OPENING BRACKET AND THE RESOURCE OF A LINK WITH A LABEL as two stretches that show a reader nothing, by their last '
        + 'unit and by their first, for a link, a link with a title, a reference and a link of marks, and none for a link with no label, '
        + 'a link of closing marks, an image or an autolink (n173)',
      fn: async () => {
        /**
         The stretches that show nothing in a link with a label, as pairs sorted by their key: by last unit, then by first unit.
         */
        function hiddenOf(
          { english, definitions = '', }: { readonly english: string; readonly definitions?: string; },
        ): { readonly starts: readonly (readonly [number, number])[]; readonly ends: readonly (readonly [number, number])[]; } {
          /**
           What the fold makes of the links of the English.
           */
          const links = foldedLinksOf({
            english,
            folded: foldWithOffsets({ text: english, },),
            definitions,
          },);
          return {
            starts: [...links.hiddenStarts.entries(),].toSorted(function byKey([first,], [second,],): number {
              return first - second;
            },),
            ends: [...links.hiddenEnds.entries(),].toSorted(function byKey([first,], [second,],): number {
              return first - second;
            },),
          };
        }
        expect([
          hiddenOf({ english: 'kit[kit](u)', },),
          hiddenOf({ english: 'kit[kit](u "t")', },),
          hiddenOf({ english: 'kit[kit][r]\n\n[r]: u', },),
          hiddenOf({ english: 'kit[-](u)', },),
          hiddenOf({ english: 'kit[kit][r]', definitions: '[r]: u', },),
        ],).toEqual([
          { starts: [[3, 3,], [10, 7,],], ends: [[3, 4,], [7, 11,],], },
          { starts: [[3, 3,], [14, 7,],], ends: [[3, 4,], [7, 15,],], },
          { starts: [[3, 3,], [10, 7,],], ends: [[3, 4,], [7, 11,],], },
          { starts: [[3, 3,], [8, 5,],], ends: [[3, 4,], [5, 9,],], },
          { starts: [[3, 3,], [10, 7,],], ends: [[3, 4,], [7, 11,],], },
        ],);
        expect([
          hiddenOf({ english: 'kit[](u)', },),
          hiddenOf({ english: 'kit[)](u)', },),
          hiddenOf({ english: 'kit![kit](u)', },),
          hiddenOf({ english: 'kit www.example.com', },),
          hiddenOf({ english: 'kit[kit][r]', },),
          hiddenOf({ english: 'kit[kit][r]', definitions: '[x]: u', },),
        ],).toEqual([
          { starts: [], ends: [], },
          { starts: [], ends: [], },
          { starts: [], ends: [], },
          { starts: [], ends: [], },
          { starts: [], ends: [], },
          { starts: [], ends: [], },
        ],);
      },
    },),

    it({
      name: 'READS EVERY LINK OF A LONG TEXT, a thousand of them, each to its own first unit',
      fn: async () => {
        /**
         A thousand links with empty labels, one apostrophe word after each.
         */
        const english = '[](u)\'kit '.repeat(1_000,);
        /**
         The empty links of the English as the fold holds them.
         */
        const { empty, } = linksOf({ english, },);
        expect(empty.length,).toBe(1_000,);
        expect(
          empty.every(function spansFiveUnits([end, start,],): boolean {
            return (end - start) === 4;
          },),
        ).toBe(true,);
      },
    },),
  ],
},);
