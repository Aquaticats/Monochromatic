/**
 Tests for where a span of an English stands among its blocks, decided without a
 model: the body block a writer is given, and the kind of block (a table, a code
 block, a setext heading) no paragraph rewrite can carry words into.

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
  blockHolding,
  blockStandingOf,
} from '../dist/final/node/index.mjs';

/**
 Where the first stand of a needle in an English stands among its blocks.

 @param english - English to read

 @param needle - words whose first stand is the span

 @returns Standing of the span

 @example
 ```ts
 const standing = standingOfNeedle({ english: 'The cat naps.', needle: 'cat', },);
 ```
 */
function standingOfNeedle({ english, needle, }: { readonly english: string; readonly needle: string; },): unknown {
  return blockStandingOf({
    english,
    start: english.indexOf(needle,),
    end: english.indexOf(needle,) + needle.length,
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: blockHolding.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES THE BODY BLOCK THAT HOLDS A SPAN WHOLE, with its offsets and its text',
          fn: async () => {
            const english = 'The cat naps.\n\nThe fish swims in the bowl.';
            expect([
              blockHolding({ english, start: 4, end: 7, },),
              blockHolding({ english, start: 19, end: 30, },),
              blockHolding({ english, start: 0, end: 13, },),
            ],).toEqual([
              [{ start: 0, end: 13, text: 'The cat naps.', },],
              [{ start: 15, end: 42, text: 'The fish swims in the bowl.', },],
              [{ start: 0, end: 13, text: 'The cat naps.', },],
            ],);
          },
        },),
        it({
          name: 'NAMES THE INNERMOST PARAGRAPH, not the container, for a span inside a list item, a block quote or a nested '
            + 'quote and list, with offsets in the English whole',
          fn: async () => {
            const list = '- The cat naps in the sun.\n- The fish swims.\n';
            const afterIntro = 'Intro here.\n\n- The cat naps in the sun.\n- The fish swims.\n';
            const nested = '> - The cat naps in the sun.\n';
            const quote = '> The cat naps in the sun.\n> It is warm.\n';
            const secondInItem = '- The cat naps.\n\n  The fish swims in the bowl.\n';
            expect([
              blockHolding({ english: list, start: list.indexOf('cat naps',), end: list.indexOf('cat naps',) + 8, },),
              blockHolding({
                english: afterIntro,
                start: afterIntro.indexOf('cat naps',),
                end: afterIntro.indexOf('cat naps',) + 8,
              },),
              blockHolding({ english: nested, start: nested.indexOf('cat naps',), end: nested.indexOf('cat naps',) + 8, },),
              blockHolding({ english: quote, start: quote.indexOf('cat naps',), end: quote.indexOf('cat naps',) + 8, },),
              blockHolding({
                english: secondInItem,
                start: secondInItem.indexOf('fish swims',),
                end: secondInItem.indexOf('fish swims',) + 10,
              },),
            ],).toEqual([
              [{ start: 2, end: 26, text: 'The cat naps in the sun.', },],
              [{ start: 15, end: 39, text: 'The cat naps in the sun.', },],
              [{ start: 4, end: 28, text: 'The cat naps in the sun.', },],
              [{ start: 2, end: 40, text: 'The cat naps in the sun.\n> It is warm.', },],
              [{ start: 19, end: 46, text: 'The fish swims in the bowl.', },],
            ],);
          },
        },),
        it({
          name: 'NAMES THE PARAGRAPH ALONE, WITHOUT THE CLOSING TAG, for a span in the content of a details container, whatever '
            + 'stands before it',
          fn: async () => {
            const details = '<details>\n<summary>Cats</summary>\n\nThe cat naps in the sun.\n\n</details>\n';
            const afterIntro = `Intro here.\n\n${details}`;
            expect([
              blockHolding({ english: details, start: 40, end: 48, },),
              blockHolding({ english: afterIntro, start: 53, end: 61, },),
            ],).toEqual([
              [{ start: 35, end: 59, text: 'The cat naps in the sun.', },],
              [{ start: 48, end: 72, text: 'The cat naps in the sun.', },],
            ],);
          },
        },),
        it({
          name: 'KEEPS THE OFFSETS OF AN ENGLISH THAT STARTS WITH A BYTE ORDER MARK, A FRONT MATTER OR WINDOWS LINE ENDS, naming '
            + 'the same innermost paragraph',
          fn: async () => {
            const bom = `${String.fromCodePoint(0xFE_FF,)}- The cat naps in the sun.\n`;
            const front = '---\ntitle: Mittens\n---\n\n- The cat naps in the sun.\n';
            const windows = '- The cat naps in the sun.\r\n- The fish swims.\r\n';
            expect([
              blockHolding({ english: bom, start: 8, end: 16, },),
              blockHolding({ english: front, start: 29, end: 37, },),
              blockHolding({ english: windows, start: 6, end: 14, },),
            ],).toEqual([
              [{ start: 3, end: 27, text: 'The cat naps in the sun.', },],
              [{ start: 26, end: 50, text: 'The cat naps in the sun.', },],
              [{ start: 2, end: 26, text: 'The cat naps in the sun.', },],
            ],);
          },
        },),
        it({
          name: 'NAMES THE HEADING LINE FOR A SPAN IN AN ATX HEADING, closing marks included, and the paragraph after a table',
          fn: async () => {
            const heading = '## The cat naps\n\nThe fish swims.\n';
            const closed = '## The cat naps ##\n\nThe fish swims.\n';
            const afterTable = '| a | b |\n| - | - |\n| cat | fish |\n\nThe fish swims.\n';
            expect([
              blockHolding({ english: heading, start: 7, end: 15, },),
              blockHolding({ english: closed, start: 7, end: 15, },),
              blockHolding({ english: afterTable, start: afterTable.indexOf('The fish',), end: afterTable.indexOf('The fish',) + 8, },),
            ],).toEqual([
              [{ start: 0, end: 15, text: '## The cat naps', },],
              [{ start: 0, end: 18, text: '## The cat naps ##', },],
              [{ start: 36, end: 51, text: 'The fish swims.', },],
            ],);
          },
        },),
        it({
          name: 'NAMES NO BLOCK FOR A SPAN IN A TABLE, A CODE BLOCK OR A SETEXT HEADING, wherever it stands, since no paragraph '
            + 'a writer could rewrite holds it',
          fn: async () => {
            /**
             The block of the first stand of a needle in an English.
             */
            function blockOfNeedle({ english, needle, }: { readonly english: string; readonly needle: string; },): unknown {
              return blockHolding({
                english,
                start: english.indexOf(needle,),
                end: english.indexOf(needle,) + needle.length,
              },);
            }
            expect([
              blockOfNeedle({ english: '| a | b |\n| - | - |\n| cat naps | x |\n', needle: 'cat naps', },),
              blockOfNeedle({ english: 'a | b\n--|--\ncat naps | x\n', needle: 'cat naps', },),
              blockOfNeedle({ english: '```\nthe cat naps\n```\n', needle: 'cat naps', },),
              blockOfNeedle({ english: '~~~\nthe cat naps\n~~~\n\nAfter.', needle: 'cat naps', },),
              blockOfNeedle({ english: 'The cat naps\n============\n\nAfter.', needle: 'cat naps', },),
              blockOfNeedle({ english: 'The cat\nnaps here\n--------\n\nAfter.', needle: 'naps', },),
              blockOfNeedle({
                english: '- item\n\n  | a | b |\n  | - | - |\n  | cat naps | x |\n\n- Next.\n',
                needle: 'cat naps',
              },),
              blockOfNeedle({ english: '> | a | b |\n> | - | - |\n> | cat naps | x |\n\nAfter.', needle: 'cat naps', },),
              blockOfNeedle({ english: '> ```\n> the cat naps\n> ```\n\nAfter.', needle: 'cat naps', },),
              blockOfNeedle({ english: '- item\n\n  ```\n  the cat naps\n  ```\n\n- Next.\n', needle: 'cat naps', },),
              blockOfNeedle({ english: '> The cat naps\n> =============\n\nAfter.', needle: 'cat naps', },),
            ],).toEqual([[], [], [], [], [], [], [], [], [], [], [],],);
          },
        },),
        it({
          name: 'NAMES NO BLOCK FOR A SPAN IN A FOOTNOTE DEFINITION, and none for a span across two list items either, '
            + 'since no paragraph holds it',
          fn: async () => {
            const footnote = 'The cat naps.[^1]\n\n[^1]: The note says the cat naps.\n';
            const items = '- The cat naps\n- in the sun.\n';
            expect([
              blockHolding({ english: footnote, start: 25, end: 35, },),
              blockHolding({ english: items, start: 11, end: 20, },),
            ],).toEqual([[], [],],);
          },
        },),
        it({
          name: 'NAMES NO BLOCK for a span that crosses two blocks, lies past the English, or lies in the front matter',
          fn: async () => {
            const english = 'The cat naps.\n\nThe fish swims.';
            const withFront = '---\ntitle: Mittens\n---\n\nThe cat naps.';
            expect([
              blockHolding({ english, start: 10, end: 20, },),
              blockHolding({ english, start: 20, end: 90, },),
              blockHolding({ english: withFront, start: 11, end: 18, },),
            ],).toEqual([[], [], [],],);
          },
        },),
      ],
    },),
    describe({
      name: blockStandingOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'NAMES THE KIND OF BLOCK, AND NO BLOCK, FOR A SPAN IN A TABLE, A CODE BLOCK OR A SETEXT HEADING, wherever each '
            + 'stands, and the block with no kind for a paragraph, an ATX heading or a span in no block',
          fn: async () => {
            expect([
              standingOfNeedle({ english: '| a | b |\n| - | - |\n| cat naps | x |\n', needle: 'cat naps', },),
              standingOfNeedle({ english: '~~~\nthe cat naps\n~~~\n\nAfter.', needle: 'cat naps', },),
              standingOfNeedle({ english: 'The cat\nnaps here\n--------\n\nAfter.', needle: 'naps', },),
              standingOfNeedle({ english: '- item\n\n  | a | b |\n  | - | - |\n  | cat naps | x |\n\n- Next.\n', needle: 'cat naps', },),
              standingOfNeedle({ english: '> The cat naps\n> =============\n\nAfter.', needle: 'cat naps', },),
              standingOfNeedle({ english: '> ## The cat naps\n\nAfter.', needle: 'cat naps', },),
              standingOfNeedle({ english: 'The cat naps.\n\nAfter.', needle: 'cat naps', },),
              standingOfNeedle({ english: 'The cat naps.', needle: 'dog', },),
            ],).toEqual([
              { block: [], spliceOnlyIn: ['table',], },
              { block: [], spliceOnlyIn: ['code block',], },
              { block: [], spliceOnlyIn: ['setext heading',], },
              { block: [], spliceOnlyIn: ['table',], },
              { block: [], spliceOnlyIn: ['setext heading',], },
              { block: [{ start: 2, end: 17, text: '## The cat naps', },], spliceOnlyIn: [], },
              { block: [{ start: 0, end: 13, text: 'The cat naps.', },], spliceOnlyIn: [], },
              { block: [], spliceOnlyIn: [], },
            ],);
          },
        },),
        it({
          name: 'NAMES A SETEXT HEADING WHOSE TEXT OPENS WITH A HASH AS A SETEXT HEADING, bare, inside a block quote or a list '
            + 'item, over two text lines and under Windows line ends, since its underline is part of the heading',
          fn: async () => {
            expect([
              standingOfNeedle({ english: '#cats nap\n=========\n\nAfter.', needle: 'nap', },),
              standingOfNeedle({ english: '> #cats nap\n> =========\n\nAfter.', needle: 'nap', },),
              standingOfNeedle({ english: '- #cats nap\n  =========\n', needle: 'nap', },),
              standingOfNeedle({ english: '#cats nap\nhere it is\n=========\n\nAfter.', needle: 'nap', },),
              standingOfNeedle({ english: '#cats nap\r\n=========\r\n\r\nAfter.', needle: 'nap', },),
            ],).toEqual([
              { block: [], spliceOnlyIn: ['setext heading',], },
              { block: [], spliceOnlyIn: ['setext heading',], },
              { block: [], spliceOnlyIn: ['setext heading',], },
              { block: [], spliceOnlyIn: ['setext heading',], },
              { block: [], spliceOnlyIn: ['setext heading',], },
            ],);
          },
        },),
        it({
          name: 'NAMES THE HEADING LINE ALONE FOR AN ATX HEADING, whatever stands in or after the line: closing marks, a later '
            + 'hash, trailing spaces, a paragraph line right after it, a list item, a block quote or Windows line ends; and '
            + 'the paragraph for a hash with no space after it',
          fn: async () => {
            expect([
              standingOfNeedle({ english: '## Cat #nap\n\nAfter.', needle: 'Cat', },),
              standingOfNeedle({ english: '## Cat naps ##\n\nAfter.', needle: 'naps', },),
              standingOfNeedle({ english: '## Cat naps ##   \n\nAfter.', needle: 'naps', },),
              standingOfNeedle({ english: '## Cat naps\nAfter line.\n', needle: 'naps', },),
              standingOfNeedle({ english: '- ## Cat naps\n- Next.\n', needle: 'naps', },),
              standingOfNeedle({ english: '> ## Cat naps\n> After line.\n', needle: 'naps', },),
              standingOfNeedle({ english: '## Cat naps\r\nAfter line.\r\n', needle: 'naps', },),
              standingOfNeedle({ english: '#cats nap\n\nAfter.', needle: 'cats', },),
            ],).toEqual([
              { block: [{ start: 0, end: 11, text: '## Cat #nap', },], spliceOnlyIn: [], },
              { block: [{ start: 0, end: 14, text: '## Cat naps ##', },], spliceOnlyIn: [], },
              { block: [{ start: 0, end: 17, text: '## Cat naps ##   ', },], spliceOnlyIn: [], },
              { block: [{ start: 0, end: 11, text: '## Cat naps', },], spliceOnlyIn: [], },
              { block: [{ start: 2, end: 13, text: '## Cat naps', },], spliceOnlyIn: [], },
              { block: [{ start: 2, end: 13, text: '## Cat naps', },], spliceOnlyIn: [], },
              { block: [{ start: 0, end: 11, text: '## Cat naps', },], spliceOnlyIn: [], },
              { block: [{ start: 0, end: 9, text: '#cats nap', },], spliceOnlyIn: [], },
            ],);
          },
        },),
        it({
          name: 'NAMES THE HEADING LINE ALONE, WITHOUT THE TAGS OF A CONTAINER, for an ATX heading that opens or closes a details '
            + 'container, and a setext heading there as a setext heading',
          fn: async () => {
            expect([
              standingOfNeedle({ english: '<details>\n\n## The cat naps\n\n</details>\n', needle: 'cat naps', },),
              standingOfNeedle({
                english: '<details>\n<summary>Cats</summary>\n\n## The cat naps\n\n</details>\n',
                needle: 'cat naps',
              },),
              standingOfNeedle({ english: '<Note>\n## The cat naps\n</Note>\n', needle: 'cat naps', },),
              standingOfNeedle({ english: '<details>\n\nThe cat naps\n-----------\n\n</details>\n', needle: 'cat naps', },),
              standingOfNeedle({ english: '<details>\n\n#cats nap\n=========\n\n</details>\n', needle: 'nap', },),
            ],).toEqual([
              { block: [{ start: 11, end: 26, text: '## The cat naps', },], spliceOnlyIn: [], },
              { block: [{ start: 35, end: 50, text: '## The cat naps', },], spliceOnlyIn: [], },
              { block: [{ start: 7, end: 22, text: '## The cat naps', },], spliceOnlyIn: [], },
              { block: [], spliceOnlyIn: ['setext heading',], },
              { block: [], spliceOnlyIn: ['setext heading',], },
            ],);
          },
        },),
        it({
          name: 'NAMES THE PARAGRAPH THE SITE RENDERS AS A PARAGRAPH, indented text (MDX turns indented code off), the content '
            + 'of a pre element on lines of its own, a table cell whose content stands apart by blank lines, and a paragraph '
            + 'with an inline element or a fragment',
          fn: async () => {
            expect([
              standingOfNeedle({ english: 'Intro.\n\n    the cat naps\n    here\n\nAfter.', needle: 'cat naps', },),
              standingOfNeedle({ english: '<pre>\nthe cat naps\n</pre>\n\nAfter.', needle: 'cat naps', },),
              standingOfNeedle({
                english: '<table>\n<tr>\n<td>\n\nthe cat naps\n\n</td>\n</tr>\n</table>\n\nAfter.',
                needle: 'cat naps',
              },),
              standingOfNeedle({ english: 'The <b>cat naps</b> here.\n\nAfter.', needle: 'cat naps', },),
              standingOfNeedle({ english: 'The <>cat naps</> here.\n\nAfter.', needle: 'cat naps', },),
            ],).toEqual([
              { block: [{ start: 12, end: 33, text: 'the cat naps\n    here', },], spliceOnlyIn: [], },
              { block: [{ start: 6, end: 18, text: 'the cat naps', },], spliceOnlyIn: [], },
              { block: [{ start: 19, end: 31, text: 'the cat naps', },], spliceOnlyIn: [], },
              { block: [{ start: 0, end: 25, text: 'The <b>cat naps</b> here.', },], spliceOnlyIn: [], },
              { block: [{ start: 0, end: 23, text: 'The <>cat naps</> here.', },], spliceOnlyIn: [], },
            ],);
          },
        },),
        it({
          name: 'NAMES A TABLE FOR A SPAN IN HTML TABLE MARKUP THE SITE RENDERS WITHOUT A PARAGRAPH: a table on one line, '
            + 'a cell on a line of its own, a header cell, a table inside a list item, and a table the strict grammar '
            + 'refuses (in lower case, upper case or with attributes)',
          fn: async () => {
            expect([
              standingOfNeedle({ english: '<table><tr><td>the cat naps</td></tr></table>\n\nAfter.', needle: 'cat naps', },),
              standingOfNeedle({ english: '<table>\n<tr>\n<td>the cat naps</td>\n</tr>\n</table>\n\nAfter.', needle: 'cat naps', },),
              standingOfNeedle({ english: '<table>\n<tr>\n<th>the cat naps</th>\n</tr>\n</table>\n\nAfter.', needle: 'cat naps', },),
              standingOfNeedle({ english: '- <table><tr><td>the cat naps</td></tr></table>\n', needle: 'cat naps', },),
              standingOfNeedle({ english: '<table><tr><td>the cat {naps</td></tr></table>\n\nAfter.', needle: 'cat', },),
              standingOfNeedle({ english: '<TABLE><TR><TD>the cat {naps</TD></TR></TABLE>\n\nAfter.', needle: 'cat', },),
              standingOfNeedle({
                english: '<table class="cats">\n<tr><td>the cat {naps</td></tr>\n</table>\n\nAfter.',
                needle: 'cat',
              },),
            ],).toEqual([
              { block: [], spliceOnlyIn: ['table',], },
              { block: [], spliceOnlyIn: ['table',], },
              { block: [], spliceOnlyIn: ['table',], },
              { block: [], spliceOnlyIn: ['table',], },
              { block: [], spliceOnlyIn: ['table',], },
              { block: [], spliceOnlyIn: ['table',], },
              { block: [], spliceOnlyIn: ['table',], },
            ],);
          },
        },),
        it({
          name: 'READS A TABLE OR PRE ELEMENT WRITTEN IN CAPITALS OR WITH A CAPITAL AS THE ELEMENT IT NAMES, in the strict '
            + 'grammar (where a capitalised name is a JSX component) and in plain markdown alike, bare or inside emphasis '
            + 'or a link, since a page written for the site spells HTML in capitals far more often than it defines a '
            + 'component of that name',
          fn: async () => {
            expect([
              standingOfNeedle({ english: '<TABLE><TR><TD>the cat naps</TD></TR></TABLE>\n\nAfter.', needle: 'cat naps', },),
              standingOfNeedle({ english: '<Table><Row>the cat naps</Row></Table>\n\nAfter.', needle: 'cat naps', },),
              standingOfNeedle({ english: '<Table><tr><td>the cat naps</td></tr></Table>\n\nAfter.', needle: 'cat naps', },),
              standingOfNeedle({ english: 'The <TD>cat naps</TD> here.\n\nAfter.', needle: 'cat naps', },),
              standingOfNeedle({ english: 'The <Td>cat naps</Td> here.\n\nAfter.', needle: 'cat naps', },),
              standingOfNeedle({ english: '<PRE>the cat naps</PRE>\n\nAfter.', needle: 'cat naps', },),
              standingOfNeedle({ english: '<Pre>the cat naps</Pre>\n\nAfter.', needle: 'cat naps', },),
              standingOfNeedle({ english: '**<TD>the cat naps</TD>**\n\nAfter.', needle: 'cat naps', },),
              standingOfNeedle({ english: '[<PRE>the cat naps</PRE>](http://cats.example)\n\nAfter.', needle: 'cat naps', },),
              standingOfNeedle({ english: 'The <TD>cat {naps</TD> here.\n\nAfter.', needle: 'cat', },),
              standingOfNeedle({ english: '**<Pre>the cat {naps</Pre>**\n\nAfter.', needle: 'cat', },),
            ],).toEqual([
              { block: [], spliceOnlyIn: ['table',], },
              { block: [], spliceOnlyIn: ['table',], },
              { block: [], spliceOnlyIn: ['table',], },
              { block: [], spliceOnlyIn: ['table',], },
              { block: [], spliceOnlyIn: ['table',], },
              { block: [], spliceOnlyIn: ['preformatted block',], },
              { block: [], spliceOnlyIn: ['preformatted block',], },
              { block: [], spliceOnlyIn: ['table',], },
              { block: [], spliceOnlyIn: ['preformatted block',], },
              { block: [], spliceOnlyIn: ['table',], },
              { block: [], spliceOnlyIn: ['preformatted block',], },
            ],);
          },
        },),
        it({
          name: 'NAMES THE BLOCK FOR A CAPITALISED ELEMENT THAT IS NOT A TABLE OR PRE ELEMENT, and for cell content that '
            + 'stands apart by blank lines inside capitalised table tags',
          fn: async () => {
            expect([
              standingOfNeedle({ english: '<Cell>the cat naps</Cell>\n\nAfter.', needle: 'cat naps', },),
              standingOfNeedle({ english: '<Tablet>the cat naps</Tablet>\n\nAfter.', needle: 'cat naps', },),
              standingOfNeedle({ english: '<Prefix>the cat naps</Prefix>\n\nAfter.', needle: 'cat naps', },),
              standingOfNeedle({
                english: '<TABLE>\n<TR>\n<TD>\n\nthe cat naps\n\n</TD>\n</TR>\n</TABLE>\n\nAfter.',
                needle: 'cat naps',
              },),
            ],).toEqual([
              { block: [{ start: 0, end: 25, text: '<Cell>the cat naps</Cell>', },], spliceOnlyIn: [], },
              { block: [{ start: 0, end: 29, text: '<Tablet>the cat naps</Tablet>', },], spliceOnlyIn: [], },
              { block: [{ start: 0, end: 29, text: '<Prefix>the cat naps</Prefix>', },], spliceOnlyIn: [], },
              { block: [{ start: 19, end: 31, text: 'the cat naps', },], spliceOnlyIn: [], },
            ],);
          },
        },),
        it({
          name: 'NAMES A PREFORMATTED BLOCK FOR A SPAN IN A PRE ELEMENT THE SITE RENDERS WITHOUT A PARAGRAPH, on one line, over '
            + 'two lines, or in a block the strict grammar refuses; and the block for a div or another element that only '
            + 'shares its first letters',
          fn: async () => {
            expect([
              standingOfNeedle({ english: '<pre>the cat naps</pre>\n\nAfter.', needle: 'cat naps', },),
              standingOfNeedle({ english: '<pre>the cat\nnaps</pre>\n\nAfter.', needle: 'cat', },),
              standingOfNeedle({ english: '<pre>\nthe cat {naps\n</pre>\n\nAfter.', needle: 'cat', },),
              standingOfNeedle({ english: '<div>the cat {naps</div>\n\nAfter.', needle: 'cat', },),
              standingOfNeedle({ english: '<prefix>the cat {naps</prefix>\n\nAfter.', needle: 'cat', },),
            ],).toEqual([
              { block: [], spliceOnlyIn: ['preformatted block',], },
              { block: [], spliceOnlyIn: ['preformatted block',], },
              { block: [], spliceOnlyIn: ['preformatted block',], },
              { block: [{ start: 0, end: 24, text: '<div>the cat {naps</div>', },], spliceOnlyIn: [], },
              { block: [{ start: 0, end: 30, text: '<prefix>the cat {naps</prefix>', },], spliceOnlyIn: [], },
            ],);
          },
        },),
        it({
          name: 'NAMES A TABLE FOR A SPAN IN A TABLE CELL ELEMENT INSIDE EMPHASIS, A LINK OR BOTH, one, two and three levels '
            + 'deep, and when the span lies beside the element',
          fn: async () => {
            expect([
              standingOfNeedle({ english: '**<td>the cat naps</td>**\n\nAfter.', needle: 'cat naps', },),
              standingOfNeedle({ english: '[<td>the cat naps</td>](http://cats.example)\n\nAfter.', needle: 'cat naps', },),
              standingOfNeedle({ english: '[*<td>the cat naps</td>*](http://cats.example)\n\nAfter.', needle: 'cat naps', },),
              standingOfNeedle({
                english: '~~[**<td>the cat naps</td>**](http://cats.example)~~\n\nAfter.',
                needle: 'cat naps',
              },),
              standingOfNeedle({ english: 'The cat naps **<td>here</td>**\n\nAfter.', needle: 'cat naps', },),
            ],).toEqual([
              { block: [], spliceOnlyIn: ['table',], },
              { block: [], spliceOnlyIn: ['table',], },
              { block: [], spliceOnlyIn: ['table',], },
              { block: [], spliceOnlyIn: ['table',], },
              { block: [], spliceOnlyIn: ['table',], },
            ],);
          },
        },),
        it({
          name: 'NAMES A PREFORMATTED BLOCK FOR A SPAN IN A PRE ELEMENT INSIDE EMPHASIS, A LINK OR BOTH, one, two and three '
            + 'levels deep',
          fn: async () => {
            expect([
              standingOfNeedle({ english: '**<pre>the cat naps</pre>**\n\nAfter.', needle: 'cat naps', },),
              standingOfNeedle({ english: '[<pre>the cat naps</pre>](http://cats.example)\n\nAfter.', needle: 'cat naps', },),
              standingOfNeedle({ english: '[*<pre>the cat naps</pre>*](http://cats.example)\n\nAfter.', needle: 'cat naps', },),
              standingOfNeedle({
                english: '~~[**<pre>the cat naps</pre>**](http://cats.example)~~\n\nAfter.',
                needle: 'cat naps',
              },),
            ],).toEqual([
              { block: [], spliceOnlyIn: ['preformatted block',], },
              { block: [], spliceOnlyIn: ['preformatted block',], },
              { block: [], spliceOnlyIn: ['preformatted block',], },
              { block: [], spliceOnlyIn: ['preformatted block',], },
            ],);
          },
        },),
        it({
          name: 'NAMES THE SAME KINDS WHEN THE STRICT GRAMMAR REFUSES THE TEXT AND PLAIN MARKDOWN READS IT: a table cell or '
            + 'a pre element at any depth, also with no emphasis around it',
          fn: async () => {
            expect([
              standingOfNeedle({ english: 'The <td>cat {naps</td> here.\n\nAfter.', needle: 'cat', },),
              standingOfNeedle({ english: '**<td>the cat {naps</td>**\n\nAfter.', needle: 'cat', },),
              standingOfNeedle({ english: '[<td>the cat {naps</td>](http://cats.example)\n\nAfter.', needle: 'cat', },),
              standingOfNeedle({
                english: '~~[**<td>the cat {naps</td>**](http://cats.example)~~\n\nAfter.',
                needle: 'cat',
              },),
              standingOfNeedle({ english: 'The <pre>cat {naps</pre> here.\n\nAfter.', needle: 'cat', },),
              standingOfNeedle({ english: '**<pre>the cat {naps</pre>**\n\nAfter.', needle: 'cat', },),
            ],).toEqual([
              { block: [], spliceOnlyIn: ['table',], },
              { block: [], spliceOnlyIn: ['table',], },
              { block: [], spliceOnlyIn: ['table',], },
              { block: [], spliceOnlyIn: ['table',], },
              { block: [], spliceOnlyIn: ['preformatted block',], },
              { block: [], spliceOnlyIn: ['preformatted block',], },
            ],);
          },
        },),
        it({
          name: 'NAMES THE BLOCK FOR ORDINARY INLINE HTML AT ANY DEPTH, in either reading: bold and span elements, a '
            + 'component, and a table cell tag only written inside inline code',
          fn: async () => {
            expect([
              standingOfNeedle({ english: '**<b>the cat naps</b>**\n\nAfter.', needle: 'cat naps', },),
              standingOfNeedle({ english: '[<span>the cat naps</span>](http://cats.example)\n\nAfter.', needle: 'cat naps', },),
              standingOfNeedle({
                english: '~~[**<b>the cat naps</b>**](http://cats.example)~~\n\nAfter.',
                needle: 'cat naps',
              },),
              standingOfNeedle({ english: '**<b>the cat {naps</b>**\n\nAfter.', needle: 'cat', },),
              standingOfNeedle({
                english: '~~[**<span>the cat {naps</span>**](http://cats.example)~~\n\nAfter.',
                needle: 'cat',
              },),
              standingOfNeedle({ english: '**<Cell>the cat naps</Cell>**\n\nAfter.', needle: 'cat naps', },),
              standingOfNeedle({ english: '**`<td>` the cat naps**\n\nAfter.', needle: 'cat naps', },),
            ],).toEqual([
              { block: [{ start: 0, end: 23, text: '**<b>the cat naps</b>**', },], spliceOnlyIn: [], },
              {
                block: [{ start: 0, end: 48, text: '[<span>the cat naps</span>](http://cats.example)', },],
                spliceOnlyIn: [],
              },
              {
                block: [{ start: 0, end: 50, text: '~~[**<b>the cat naps</b>**](http://cats.example)~~', },],
                spliceOnlyIn: [],
              },
              { block: [{ start: 0, end: 24, text: '**<b>the cat {naps</b>**', },], spliceOnlyIn: [], },
              {
                block: [{ start: 0, end: 57, text: '~~[**<span>the cat {naps</span>**](http://cats.example)~~', },],
                spliceOnlyIn: [],
              },
              { block: [{ start: 0, end: 29, text: '**<Cell>the cat naps</Cell>**', },], spliceOnlyIn: [], },
              { block: [{ start: 0, end: 23, text: '**`<td>` the cat naps**', },], spliceOnlyIn: [], },
            ],);
          },
        },),
        it({
          name: 'NAMES THE KIND OF THE FIRST TABLE OR PRE ELEMENT IN THE ORDER OF THE TEXT when a paragraph holds both bare',
          fn: async () => {
            expect([
              standingOfNeedle({ english: '<td>the cat</td> <pre>naps</pre>\n\nAfter.', needle: 'cat', },),
              standingOfNeedle({ english: '<pre>the cat</pre> <td>naps</td>\n\nAfter.', needle: 'cat', },),
            ],).toEqual([
              { block: [], spliceOnlyIn: ['table',], },
              { block: [], spliceOnlyIn: ['preformatted block',], },
            ],);
          },
        },),
        it({
          name: 'NAMES THE KIND OF THE FIRST TABLE OR PRE ELEMENT IN THE ORDER OF THE TEXT when a paragraph holds both and one '
            + 'or both stand inside a link or emphasis',
          fn: async () => {
            expect([
              standingOfNeedle({ english: '**<td>the cat</td>** and [<pre>naps</pre>](http://cats.example)\n\nAfter.', needle: 'cat', },),
              standingOfNeedle({ english: '[<pre>the cat</pre>](http://cats.example) and **<td>naps</td>**\n\nAfter.', needle: 'cat', },),
              standingOfNeedle({ english: '[<pre>the cat</pre>](http://cats.example) and <td>naps</td>\n\nAfter.', needle: 'cat', },),
            ],).toEqual([
              { block: [], spliceOnlyIn: ['table',], },
              { block: [], spliceOnlyIn: ['preformatted block',], },
              { block: [], spliceOnlyIn: ['preformatted block',], },
            ],);
          },
        },),
        it({
          name: 'CUTS A MULTI-LINE BLOCK QUOTE PARAGRAPH FROM ITS FIRST CHARACTER, so the first line carries no marker and every '
            + 'later line keeps its own',
          fn: async () => {
            expect(blockStandingOf({ english: '> The cat naps\n> in the sun.\n', start: 2, end: 14, },),).toEqual({
              block: [{ start: 2, end: 28, text: 'The cat naps\n> in the sun.', },],
              spliceOnlyIn: [],
            },);
          },
        },),
        it({
          name: 'NAMES NO BLOCK FOR A SPAN THAT CROSSES A PARAGRAPH BOUNDARY INSIDE A BLOCK QUOTE, while a span wholly inside '
            + 'one of its paragraphs keeps that paragraph, so that no two blocks of one English nest',
          fn: async () => {
            const quote = '> The cat naps.\n>\n> The dog barks.\n>\n> The fish swims.';
            expect([
              standingOfNeedle({ english: quote, needle: 'naps.\n>\n> The dog', },),
              standingOfNeedle({ english: quote, needle: 'The fish', },),
              standingOfNeedle({ english: quote, needle: 'The dog', },),
            ],).toEqual([
              { block: [], spliceOnlyIn: [], },
              { block: [{ start: 39, end: 54, text: 'The fish swims.', },], spliceOnlyIn: [], },
              { block: [{ start: 20, end: 34, text: 'The dog barks.', },], spliceOnlyIn: [], },
            ],);
          },
        },),
        it({
          name: 'NAMES NO BLOCK FOR A SPAN THAT CROSSES TWO LIST ITEMS, OR TWO PARAGRAPHS OF ONE LIST ITEM, while a span wholly '
            + 'inside one paragraph keeps that paragraph',
          fn: async () => {
            const items = '- The cat naps.\n- The dog barks.\n- The fish swims.';
            const item = '- The cat naps.\n\n  The dog barks.\n\n  The fish swims.';
            expect([
              standingOfNeedle({ english: items, needle: 'naps.\n- The dog', },),
              standingOfNeedle({ english: items, needle: 'The fish', },),
              standingOfNeedle({ english: item, needle: 'naps.\n\n  The dog', },),
              standingOfNeedle({ english: item, needle: 'The fish', },),
            ],).toEqual([
              { block: [], spliceOnlyIn: [], },
              { block: [{ start: 35, end: 50, text: 'The fish swims.', },], spliceOnlyIn: [], },
              { block: [], spliceOnlyIn: [], },
              { block: [{ start: 37, end: 52, text: 'The fish swims.', },], spliceOnlyIn: [], },
            ],);
          },
        },),
        it({
          name: 'NAMES NO BLOCK FOR A SPAN THAT CROSSES TWO ITEMS OF A LIST INSIDE A BLOCK QUOTE, two containers deep, while a '
            + 'span wholly inside one item keeps its paragraph',
          fn: async () => {
            const nested = '> - The cat naps.\n> - The dog barks.\n> - The fish swims.';
            const nestedParagraphs = '> - The cat naps.\n>\n>   The dog barks.\n>\n>   The fish swims.';
            expect([
              standingOfNeedle({ english: nested, needle: 'naps.\n> - The dog', },),
              standingOfNeedle({ english: nested, needle: 'The fish', },),
              standingOfNeedle({ english: nestedParagraphs, needle: 'naps.\n>\n>   The dog', },),
              standingOfNeedle({ english: nestedParagraphs, needle: 'The fish', },),
            ],).toEqual([
              { block: [], spliceOnlyIn: [], },
              { block: [{ start: 41, end: 56, text: 'The fish swims.', },], spliceOnlyIn: [], },
              { block: [], spliceOnlyIn: [], },
              { block: [{ start: 45, end: 60, text: 'The fish swims.', },], spliceOnlyIn: [], },
            ],);
          },
        },),
        it({
          name: 'NAMES NO BLOCK FOR A SPAN IN AN HTML COMMENT INSIDE A BLOCK QUOTE OR A LIST ITEM, or in a JSX attribute of a '
            + 'container, since it lies in no paragraph, while the paragraph beside it keeps its own block',
          fn: async () => {
            const quote = '> <!-- The dog barks. -->\n>\n> The cat naps.';
            const item = '- The cat naps.\n\n  <!-- The dog barks. -->\n\n  The fish swims.';
            const commentOnly = '- <!-- The dog barks. -->\n\n  The cat naps.';
            const jsx = '<Note title="The dog barks.">\n\nThe cat naps.\n\n</Note>';
            expect([
              standingOfNeedle({ english: quote, needle: 'The dog', },),
              standingOfNeedle({ english: quote, needle: 'The cat', },),
              standingOfNeedle({ english: item, needle: 'The dog', },),
              standingOfNeedle({ english: item, needle: 'The fish', },),
              standingOfNeedle({ english: commentOnly, needle: 'The dog', },),
              standingOfNeedle({ english: jsx, needle: 'The dog', },),
              standingOfNeedle({ english: jsx, needle: 'The cat', },),
            ],).toEqual([
              { block: [], spliceOnlyIn: [], },
              { block: [{ start: 30, end: 43, text: 'The cat naps.', },], spliceOnlyIn: [], },
              { block: [], spliceOnlyIn: [], },
              { block: [{ start: 46, end: 61, text: 'The fish swims.', },], spliceOnlyIn: [], },
              { block: [], spliceOnlyIn: [], },
              { block: [], spliceOnlyIn: [], },
              { block: [{ start: 31, end: 44, text: 'The cat naps.', },], spliceOnlyIn: [], },
            ],);
          },
        },),
        it({
          name: 'KEEPS NO BLOCK FOR A SPAN THAT CROSSES TWO TOP-LEVEL PARAGRAPHS, and the whole top-level block for a span in a '
            + 'block that holds no paragraph or heading, since nothing nests in it',
          fn: async () => {
            const paragraphs = 'The cat naps.\n\nThe dog barks.\n\nThe fish swims.';
            const expression = '{/* The dog barks. */}\n\nThe cat naps.';
            expect([
              standingOfNeedle({ english: paragraphs, needle: 'naps.\n\nThe dog', },),
              standingOfNeedle({ english: paragraphs, needle: 'The fish', },),
              standingOfNeedle({ english: expression, needle: 'The dog', },),
              standingOfNeedle({ english: expression, needle: 'The cat', },),
            ],).toEqual([
              { block: [], spliceOnlyIn: [], },
              { block: [{ start: 31, end: 46, text: 'The fish swims.', },], spliceOnlyIn: [], },
              { block: [{ start: 0, end: 22, text: '{/* The dog barks. */}', },], spliceOnlyIn: [], },
              { block: [{ start: 24, end: 37, text: 'The cat naps.', },], spliceOnlyIn: [], },
            ],);
          },
        },),
      ],
    },),
  ],
},);
