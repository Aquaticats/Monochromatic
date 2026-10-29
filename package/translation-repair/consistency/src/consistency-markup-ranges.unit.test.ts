/**
 Tests for the stretches of images, table rows and inline code content that a parse of an English holds
 (`consistency-markup-ranges.ts`, read through `proseViewOf`).

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { proseViewOf, } from '../dist/final/node/index.mjs';

/**
 The text of each stretch.

 @param english - English the stretches were read from
 @param ranges - stretches
 @returns What each stretch covers, sorted, since the order of the stretches is not promised

 @example
 ```ts
 const texts = coveredBy({ english: 'a ![pic](u) b', ranges: [{ start: 2, end: 11, },], },); // ['![pic](u)']
 ```
 */
function coveredBy(
  { english, ranges, }: { readonly english: string; readonly ranges: readonly { readonly start: number; readonly end: number; }[]; },
): readonly string[] {
  return ranges
    .map(function textOf({ start, end, },): string {
      return english.slice(
        start,
        end,
      );
    },)
    .toSorted();
}

await describe({
  name: 'the markup ranges of a prose view',
  concurrency: 1,
  children: [
    it({
      name: 'COVERS an image and an image reference, each from its bang to its closing bracket or parenthesis',
      fn: async () => {
        /**
         A page with a link-destination image and a reference image.
         */
        const english = 'cats ![pic](u) dogs ![again][r] fish\n\n[r]: https://cats.example/r.png';
        expect(coveredBy({
          english,
          ranges: proseViewOf({ english, },).images,
        },),).toEqual(['![again][r]', '![pic](u)',],);
      },
    },),

    it({
      name: 'COVERS each row of a table from its first pipe to its last, the delimiter row left out',
      fn: async () => {
        /**
         A table of two rows under its header.
         */
        const english = '| cats | dogs |\n| --- | --- |\n| fish | birds |';
        expect(coveredBy({
          english,
          ranges: proseViewOf({ english, },).tableRows,
        },),).toEqual(['| cats | dogs |', '| fish | birds |',],);
      },
    },),

    it({
      name: 'COVERS the content of inline code without the backticks that open and close it, however many',
      fn: async () => {
        /**
         One single-backtick code and one double-backtick code that holds a backtick.
         */
        const english = 'The `kit` and ``a`b`` ate.';
        expect(coveredBy({
          english,
          ranges: proseViewOf({ english, },).codeContents,
        },),).toEqual(['a`b', 'kit',],);
      },
    },),

    it({
      name: 'COVERS a link, a reference and an image whose label shows nothing, from the first mark to the last, however the destination and the title are written',
      fn: async () => {
        /**
         A link whose destination nests round brackets as deep as the grammar allows, thirty-two levels.
         */
        const deepest = `[](${'('.repeat(32,)}a${')'.repeat(32,)})`;
        /**
         English and the stretches of the links with no label it holds, one row for each way of writing the destination or the title.
         */
        const rows: readonly (readonly [string, readonly string[]])[] = [
          [`kit${deepest} kit`, [deepest,],],
          ['a [](u) b', ['[](u)',],],
          ['a [](u "t") b', ['[](u "t")',],],
          ['a []() b', ['[]()',],],
          ['a [](<>) b', ['[](<>)',],],
          ['a [](u\n) b', ['[](u\n)',],],
          ['a ![](u) b', ['![](u)',],],
          ['a [][r] b ![][r] c\n\n[r]: u', ['![][r]', '[][r]',],],
          ['kit[](u(a)) kit', ['[](u(a))',],],
          [String.raw`kit[](u\(a) kit`, [String.raw`[](u\(a)`,],],
          [String.raw`kit[](u\\) kit`, [String.raw`[](u\\)`,],],
          ['kit[](<u(a>) kit', ['[](<u(a>)',],],
          ['kit[](<u>) kit', ['[](<u>)',],],
          ['kit[](u "t(a)") kit', ['[](u "t(a)")',],],
          [String.raw`kit[](u "t\(a") kit`, [String.raw`[](u "t\(a")`,],],
          ['kit[](u \'t(a)\') kit', ['[](u \'t(a)\')',],],
          ['kit[](u)[](v) kit', ['[](u)', '[](v)',],],
          ['kit[![](a)](b) kit', ['![](a)', '[![](a)](b)',],],
          ['kit[`](u) kit', [],],
        ];
        expect(rows.map(function stretchesOf([english,],): readonly string[] {
          return coveredBy({
            english,
            ranges: proseViewOf({ english, },).emptyLinks,
          },);
        },),).toEqual(rows.map(function expectedOf([, expected,],): readonly string[] {
          return expected;
        },),);
      },
    },),

    it({
      name: 'COVERS only the link inside a label that holds a link, and a title that closes early leaves its last bracket out',
      fn: async () => {
        expect(coveredBy({
          english: 'kit[[](a)](b) kit',
          ranges: proseViewOf({ english: 'kit[[](a)](b) kit', },).emptyLinks,
        },),).toEqual(['[](a)',],);
        expect(coveredBy({
          english: 'kit[](u (t(a))) kit',
          ranges: proseViewOf({ english: 'kit[](u (t(a))) kit', },).emptyLinks,
        },),).toEqual(['[](u (t(a))',],);
      },
    },),

    it({
      name: 'COVERS NO STRETCH for text the grammar reads as no link: a destination never closed, a backslash that escapes the last bracket, a newline inside the destination, a reference with no definition',
      fn: async () => {
        /**
         English texts that hold the marks of a link and none.
         */
        const englishes: readonly string[] = [
          'kit[](u(a)\'kit',
          String.raw`kit[](u\)'kit`,
          'kit[](u\nv)\'kit',
          // The grammar allows thirty-two levels of round brackets in a destination (`micromark-factory-destination`), so a thirty-third
          // makes the text no link.
          `kit[](${'('.repeat(33,)}a${')'.repeat(33,)})'kit`,
          // The page grammar refuses a destination that opens with `<` and never closes it, so the English is read as text.
          'kit[](<u)\'kit',
          'kit[](u\'kit',
          'kit[]\'kit',
          'kit[][r]\'kit',
          'kit![][r]\'kit',
        ];
        expect(englishes.map(function stretchesOf(english,): number {
          return proseViewOf({ english, },).emptyLinks.length;
        },),).toEqual(englishes.map(function none(): number {
          return 0;
        },),);
      },
    },),

    it({
      name: 'COVERS a link whose label shows marks or spaces and no letter or digit, unless its last mark closes a bracket or a quotation',
      fn: async () => {
        /**
         English and the stretches of the links with a label of marks it holds.
         */
        const rows: readonly (readonly [string, readonly string[]])[] = [
          ['kit[ ](u) kit', ['[ ](u)',],],
          ['kit[-](u) kit', ['[-](u)',],],
          ['kit[_](u) kit', ['[_](u)',],],
          ['kit[*](u) kit', ['[*](u)',],],
          [String.raw`kit[\[](u) kit`, [String.raw`[\[](u)`,],],
          ['kit[`-`](u) kit', ['[`-`](u)',],],
          ['kit![ ](u) kit', ['![ ](u)',],],
          ['kit[![ ](a)](b) kit', ['![ ](a)', '[![ ](a)](b)',],],
          ['kit[\\\n](u) kit', ['[\\\n](u)',],],
          ['kit[`-`)](u) kit', [],],
          ['kit[`)`-](u) kit', ['[`)`-](u)',],],
          ['kit[*-* )](u) kit', [],],
          [String.raw`kit[\]](u) kit`, [],],
          ['kit[()](u) kit', [],],
          ['kit[”](u) kit', [],],
          ['kit[x](u) kit', [],],
          ['kit[é](u) kit', [],],
          ['kit[7](u) kit', [],],
          ['kit[&#116;](u) kit', [],],
          ['kit[the cat](u) kit', [],],
          ['kit![x](u) kit', [],],
          ['kit[<b/>](u) kit', [],],
          ['kit[](u) kit', [],],
        ];
        expect(rows.map(function stretchesOf([english,],): readonly string[] {
          return coveredBy({
            english,
            ranges: proseViewOf({ english, },).markOnlyLinks,
          },);
        },),).toEqual(rows.map(function expectedOf([, expected,],): readonly string[] {
          return expected;
        },),);
      },
    },),

    it({
      name: 'COVERS a link or an image whose label shows closing brackets and closing curly quotation marks and nothing else (n169)',
      fn: async () => {
        /**
         English and the stretches of the links with a label of closing marks it holds.
         */
        const rows: readonly (readonly [string, readonly string[]])[] = [
          [String.raw`kit[\]](u) kit`, [String.raw`[\]](u)`,],],
          ['kit[)](u) kit', ['[)](u)',],],
          ['kit[”](u) kit', ['[”](u)',],],
          ['kit[))](u) kit', ['[))](u)',],],
          ['kit[”)](u) kit', ['[”)](u)',],],
          ['kit![)](u) kit', ['![)](u)',],],
          ['kit[`)`](u) kit', ['[`)`](u)',],],
          ['kit[()](u) kit', [],],
          ['kit[ )](u) kit', [],],
          ['kit[-)](u) kit', [],],
          ['kit[x)](u) kit', [],],
          ['kit[-](u) kit', [],],
          ['kit[ ](u) kit', [],],
          ['kit[x](u) kit', [],],
          ['kit[](u) kit', [],],
        ];
        expect(rows.map(function stretchesOf([english,],): readonly string[] {
          return coveredBy({
            english,
            ranges: proseViewOf({ english, },).closingLinks,
          },);
        },),).toEqual(rows.map(function expectedOf([, expected,],): readonly string[] {
          return expected;
        },),);
      },
    },),

    it({
      name: 'COVERS the resource of every link and reference from the end of its label, and a whole image, whatever the label shows (n169)',
      fn: async () => {
        /**
         English and the stretches of the resources it holds.
         */
        const rows: readonly (readonly [string, readonly string[]])[] = [
          ['kit[x](u) kit', ['](u)',],],
          ['kit[](u) kit', ['](u)',],],
          ['kit[ ](u) kit', ['](u)',],],
          ['kit[*x*](u "t") kit', ['](u "t")',],],
          ['kit[x][r] kit\n\n[r]: u', ['][r]',],],
          ['kit[][r] kit\n\n[r]: u', ['][r]',],],
          ['kit[x](u\n) kit', ['](u\n)',],],
          ['kit[](u\n) kit', ['](u\n)',],],
          ['kit![x](u) kit', ['![x](u)',],],
          ['kit![](u\n) kit', ['![](u\n)',],],
          ['kit![x][r] kit\n\n[r]: u', ['![x][r]',],],
          ['kit[![a](b)](c) kit', ['![a](b)', '](c)',],],
          ['kit[[](a)](b) kit', ['](a)',],],
          ['kit [x] kit', [],],
          ['kit](u) kit', [],],
        ];
        expect(rows.map(function stretchesOf([english,],): readonly string[] {
          return coveredBy({
            english,
            ranges: proseViewOf({ english, },).linkResources,
          },);
        },),).toEqual(rows.map(function expectedOf([, expected,],): readonly string[] {
          return expected;
        },),);
      },
    },),

    it({
      name: 'COVERS the opening bracket and the resource of a link or a reference whose label shows a word or marks, and of no other '
        + 'link: not one with no label, one of closing marks, an image or an autolink (n173)',
      fn: async () => {
        /**
         English, the text of the opening bracket of each such link and the text of its resource.
         */
        const rows: readonly (readonly [string, readonly string[], readonly string[]])[] = [
          ['kit[x](u) kit', ['[',], ['](u)',],],
          ['kit[ x](u) kit', ['[',], ['](u)',],],
          ['kit[-](u) kit', ['[',], ['](u)',],],
          ['kit[*x*](u "t") kit', ['[',], ['](u "t")',],],
          ['kit[x][r] kit\n\n[r]: u', ['[',], ['][r]',],],
          ['kit[![a](b)](c) kit', ['[',], ['](c)',],],
          ['kit[x](u) [y](v) kit', ['[', '[',], ['](u)', '](v)',],],
          ['kit[](u) kit', [], [],],
          ['kit[)](u) kit', [], [],],
          ['kit![x](u) kit', [], [],],
          ['kit www.example.com kit', [], [],],
          ['kit https://u.example kit', [], [],],
          ['kit <https://u.example> kit', [], [],],
          ['kit [x] kit', [], [],],
        ];
        expect(rows.map(function partsOf([english,],): readonly (readonly string[])[] {
          /**
           The links with a label, in the order the parse listed them.
           */
          const { labelledLinks, } = proseViewOf({ english, },);
          return [
            labelledLinks.map(function openerOf({ opener, },): string {
              return english.slice(
                opener.start,
                opener.end,
              );
            },)
              .toSorted(),
            labelledLinks.map(function resourceOf({ resource, },): string {
              return english.slice(
                resource.start,
                resource.end,
              );
            },)
              .toSorted(),
          ];
        },),).toEqual(rows.map(function expectedOf([, openers, resources,],): readonly (readonly string[])[] {
          return [openers, resources,];
        },),);
      },
    },),

    it({
      name: 'HOLDS no stretch of any kind for an English the page grammar refuses, and for one without the markup',
      fn: async () => {
        /**
         English with none of the three kinds of markup.
         */
        const plain = proseViewOf({ english: 'The cat naps.', },);
        expect(plain.images,).toEqual([],);
        expect(plain.tableRows,).toEqual([],);
        expect(plain.codeContents,).toEqual([],);
        expect(plain.emptyLinks,).toEqual([],);
        expect(plain.markOnlyLinks,).toEqual([],);
        expect(plain.closingLinks,).toEqual([],);
        expect(plain.linkResources,).toEqual([],);
        expect(plain.labelledLinks,).toEqual([],);
        /**
         English the grammar refuses: a container nested past the bound.
         */
        const refused = proseViewOf({ english: `${'> '.repeat(300,)}![pic](u) \`kit\` [](u) [ ](u)`, },);
        expect(refused.refused,).toBe(true,);
        expect(refused.images,).toEqual([],);
        expect(refused.tableRows,).toEqual([],);
        expect(refused.codeContents,).toEqual([],);
        expect(refused.emptyLinks,).toEqual([],);
        expect(refused.markOnlyLinks,).toEqual([],);
        expect(refused.labelledLinks,).toEqual([],);
      },
    },),
  ],
},);
