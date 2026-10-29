/**
 A title reference the original writes as a link (`《[title](url)》`), found
 on the page by the link's destination. Cat-themed invention throughout; no
 corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { unifyTitleReferences, } from '../../dist/final/node/index.mjs';
import {
  pair,
  textsOf,
} from './title-reference.test-fixture.ts';

/**
 Heading the references point at.
 */
const HEADING = pair({
  sliceIndex: 0,
  source: '### 窗边猫',
  target: '',
},);

/**
 Rendering of that heading.
 */
const HEADING_ROW = {
  sliceIndex: 0,
  replacementText: '### Cat by the Window',
} as const;

/**
 Finding for a reference the pass found no span for.
 */
const UNPLACED = 'title-reference-unplaced (slice 1: 「窗边猫」 rendered by the heading of slice 0 as "Cat by the Window", '
  + 'no linked, glossed, bracketed or quoted span found)';

await describe({
  name: `${unifyTitleReferences.name} over linked references`,
  children: [
    it({
      name: 'READS NO LINK TEXT ACROSS A LINE (ledger B57): a page link that lost its opening bracket below a footnote '
        + 'line leaves the footnote whole',
      fn: async () => {
        /**
         Page text whose link lost its opening bracket.
         */
        const page = '[^1]: A cat.\n\n——From The Window Cat](https://example.test/cat)';
        /**
         Pass over the heading and that page.
         */
        const unified = unifyTitleReferences({
          slices: [
            HEADING,
            pair({
              sliceIndex: 1,
              source: '[^1]: 一只猫。\n\n——出自《[窗边猫](https://example.test/cat)》',
              target: '',
            },),
          ],
          replacements: [
            HEADING_ROW,
            {
              sliceIndex: 1,
              replacementText: page,
            },
          ],
        },);
        expect(textsOf({ rows: unified.replacements, },),).toEqual([
          HEADING_ROW.replacementText,
          page,
        ],);
        expect(unified.findings,).toEqual([UNPLACED,],);
      },
    },),
    it({
      name: 'READS NO LINK TEXT PAST ANOTHER LINK\'S BRACKET (ledger B57): a page link that lost its opening bracket '
        + 'after an earlier link on its line leaves that link whole',
      fn: async () => {
        /**
         Page text whose second link lost its opening bracket.
         */
        const page = 'See [cat](https://example.test/a) and The Window Cat](https://example.test/cat)';
        /**
         Pass over the heading and that page.
         */
        const unified = unifyTitleReferences({
          slices: [
            HEADING,
            pair({
              sliceIndex: 1,
              source: '见[猫](https://example.test/a)与《[窗边猫](https://example.test/cat)》',
              target: '',
            },),
          ],
          replacements: [
            HEADING_ROW,
            {
              sliceIndex: 1,
              replacementText: page,
            },
          ],
        },);
        expect(textsOf({ rows: unified.replacements, },),).toEqual([
          HEADING_ROW.replacementText,
          page,
        ],);
        expect(unified.findings,).toEqual([UNPLACED,],);
      },
    },),
    it({
      name: 'READS NO LINK where no opening bracket stands before the destination on the page',
      fn: async () => {
        /**
         Page text carrying the destination with no bracket before it.
         */
        const page = '——From The Window Cat](https://example.test/cat)';
        /**
         Pass over the heading and that page.
         */
        const unified = unifyTitleReferences({
          slices: [
            HEADING,
            pair({
              sliceIndex: 1,
              source: '——出自《[窗边猫](https://example.test/cat)》',
              target: '',
            },),
          ],
          replacements: [
            HEADING_ROW,
            {
              sliceIndex: 1,
              replacementText: page,
            },
          ],
        },);
        expect(textsOf({ rows: unified.replacements, },),).toEqual([
          HEADING_ROW.replacementText,
          page,
        ],);
        expect(unified.findings,).toEqual([UNPLACED,],);
      },
    },),
    it({
      name: 'READS THE PAGE LINK AT THE TITLE LINK\'S PLACE (ledger B59): where the original links the destination '
        + 'twice, "here" first and the title second, the page\'s second link takes the heading\'s rendering',
      fn: async () => {
        /**
         Pass over a slice linking the destination from a word and from the title.
         */
        const unified = unifyTitleReferences({
          slices: [
            HEADING,
            pair({
              sliceIndex: 1,
              source: '详见[这里](https://example.test/cat)。——出自《[窗边猫](https://example.test/cat)》',
              target: '',
            },),
          ],
          replacements: [
            HEADING_ROW,
            {
              sliceIndex: 1,
              replacementText: 'See [here](https://example.test/cat). ——From [The Window Cat](https://example.test/cat)',
            },
          ],
        },);
        expect(textsOf({ rows: unified.replacements, },),).toEqual([
          HEADING_ROW.replacementText,
          'See [here](https://example.test/cat). ——From [Cat by the Window](https://example.test/cat)',
        ],);
      },
    },),
    it({
      name: 'STANDS ASIDE WHERE THE PAGE LINKS THE DESTINATION MORE OFTEN THAN THE ORIGINAL (ledger B59), since '
        + 'which link carries the title cannot be read',
      fn: async () => {
        /**
         Page text linking the destination twice where the original links it once.
         */
        const page = '——From [The Window Cat](https://example.test/cat) ([source](https://example.test/cat))';
        /**
         Pass over the heading and that page.
         */
        const unified = unifyTitleReferences({
          slices: [
            HEADING,
            pair({
              sliceIndex: 1,
              source: '——出自《[窗边猫](https://example.test/cat)》',
              target: '',
            },),
          ],
          replacements: [
            HEADING_ROW,
            {
              sliceIndex: 1,
              replacementText: page,
            },
          ],
        },);
        expect(textsOf({ rows: unified.replacements, },),).toEqual([
          HEADING_ROW.replacementText,
          page,
        ],);
        expect(unified.findings,).toEqual([
          'title-reference-ambiguous (slice 1: 「窗边猫」 rendered by the heading of slice 0 as "Cat by the Window", '
            + 'but the slice offers more than one span to read)',
        ],);
      },
    },),
    it({
      name: 'READS EVERY LINK OF THE TITLE: where the original links the title twice to one destination, '
        + 'both of the page\'s links at those places take the heading\'s rendering',
      fn: async () => {
        /**
         Pass over a slice linking the title twice to one destination.
         */
        const unified = unifyTitleReferences({
          slices: [
            HEADING,
            pair({
              sliceIndex: 1,
              source: '——出自《[窗边猫](https://example.test/cat)》，又见《[窗边猫](https://example.test/cat)》',
              target: '',
            },),
          ],
          replacements: [
            HEADING_ROW,
            {
              sliceIndex: 1,
              replacementText: '——From [The Window Cat](https://example.test/cat), again [The Window Cat](https://example.test/cat)',
            },
          ],
        },);
        expect(textsOf({ rows: unified.replacements, },),).toEqual([
          HEADING_ROW.replacementText,
          '——From [Cat by the Window](https://example.test/cat), again [Cat by the Window](https://example.test/cat)',
        ],);
      },
    },),
    it({
      name: 'READS EVERY LINK OF THE TITLE: where the original links the title to two destinations, '
        + 'the page\'s link to each takes the heading\'s rendering',
      fn: async () => {
        /**
         Pass over a slice linking the title to two destinations.
         */
        const unified = unifyTitleReferences({
          slices: [
            HEADING,
            pair({
              sliceIndex: 1,
              source: '《[窗边猫](https://example.test/a)》与《[窗边猫](https://example.test/b)》',
              target: '',
            },),
          ],
          replacements: [
            HEADING_ROW,
            {
              sliceIndex: 1,
              replacementText: '[The Window Cat](https://example.test/a) and [The Window Cat](https://example.test/b)',
            },
          ],
        },);
        expect(textsOf({ rows: unified.replacements, },),).toEqual([
          HEADING_ROW.replacementText,
          '[Cat by the Window](https://example.test/a) and [Cat by the Window](https://example.test/b)',
        ],);
      },
    },),
    it({
      name: 'STANDS ASIDE WHERE ONE OF THE TITLE\'S TWO DESTINATIONS IS LINKED A DIFFERENT NUMBER OF TIMES on the page, '
        + 'since which of its links carries the title cannot be read',
      fn: async () => {
        /**
         Page text linking the second destination twice where the original links it once.
         */
        const page = '[The Window Cat](https://example.test/a) and [The Window Cat](https://example.test/b) '
          + '([source](https://example.test/b))';
        /**
         Pass over the heading and that page.
         */
        const unified = unifyTitleReferences({
          slices: [
            HEADING,
            pair({
              sliceIndex: 1,
              source: '《[窗边猫](https://example.test/a)》与《[窗边猫](https://example.test/b)》',
              target: '',
            },),
          ],
          replacements: [
            HEADING_ROW,
            {
              sliceIndex: 1,
              replacementText: page,
            },
          ],
        },);
        expect(textsOf({ rows: unified.replacements, },),).toEqual([
          HEADING_ROW.replacementText,
          page,
        ],);
        expect(unified.findings,).toEqual([
          'title-reference-ambiguous (slice 1: 「窗边猫」 rendered by the heading of slice 0 as "Cat by the Window", '
            + 'but the slice offers more than one span to read)',
        ],);
      },
    },),
    it({
      name: 'READS THE LINK THE PAGE KEPT WHERE IT DROPPED ONE OF THE TITLE\'S TWO DESTINATIONS',
      fn: async () => {
        /**
         Pass over a page that kept the link to the first destination only.
         */
        const unified = unifyTitleReferences({
          slices: [
            HEADING,
            pair({
              sliceIndex: 1,
              source: '《[窗边猫](https://example.test/a)》与《[窗边猫](https://example.test/b)》',
              target: '',
            },),
          ],
          replacements: [
            HEADING_ROW,
            {
              sliceIndex: 1,
              replacementText: '[The Window Cat](https://example.test/a) and the same cat',
            },
          ],
        },);
        expect(textsOf({ rows: unified.replacements, },),).toEqual([
          HEADING_ROW.replacementText,
          '[Cat by the Window](https://example.test/a) and the same cat',
        ],);
      },
    },),
    it({
      name: 'READS THE QUOTES WHERE THE ORIGINAL\'S LINK NEVER CLOSES, since no destination can be read from it',
      fn: async () => {
        /**
         Pass over an original whose link lost its closing parenthesis.
         */
        const unified = unifyTitleReferences({
          slices: [
            HEADING,
            pair({
              sliceIndex: 1,
              source: '——出自《[窗边猫](https://example.test/cat》',
              target: '',
            },),
          ],
          replacements: [
            HEADING_ROW,
            {
              sliceIndex: 1,
              replacementText: '——From \u{201C}The Window Cat\u{201D}',
            },
          ],
        },);
        expect(textsOf({ rows: unified.replacements, },)[1],).toBe('——From \u{201C}Cat by the Window\u{201D}',);
      },
    },),
    it({
      name: 'READS THE QUOTES WHERE THE PAGE LINKS THE DESTINATION NOWHERE',
      fn: async () => {
        /**
         Pass over a page that quotes the title instead of linking it.
         */
        const unified = unifyTitleReferences({
          slices: [
            HEADING,
            pair({
              sliceIndex: 1,
              source: '——出自《[窗边猫](https://example.test/cat)》',
              target: '',
            },),
          ],
          replacements: [
            HEADING_ROW,
            {
              sliceIndex: 1,
              replacementText: '——From the page \u{201C}The Window Cat\u{201D}',
            },
          ],
        },);
        expect(textsOf({ rows: unified.replacements, },)[1],).toBe('——From the page \u{201C}Cat by the Window\u{201D}',);
      },
    },),
  ],
},);
