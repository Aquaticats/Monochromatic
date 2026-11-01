/**
 Tests for which two wordings count as one page (ledger B26).

 WHY THIS FILE EXISTS. Every place that decides whether a proposal changes
 anything reads `sameWording`, so what it folds and what it keeps apart is the
 whole contract: a fold too narrow ships a change the page does not show, and
 one too wide throws away a change a reader would see. Each case that folds
 has a neighbour that must not.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  sameWording,
  wordingKey,
} from '../dist/final/node/index.mjs';

//region Fixtures

/**
 One prose paragraph with a soft line break inside it.
 */
const SOFT = 'The cat naps on the mat\nall afternoon.';

/**
 The same paragraph on one line.
 */
const ONE_LINE = 'The cat naps on the mat all afternoon.';

/**
 Reads a proposal against a standing wording on a prose slice.

 @param proposal - wording proposed

 @param standing - wording already there

 @returns Whether the two publish one page

 @example
 ```ts
 const same = prose({ proposal: 'The cat naps.', standing: 'The cat naps.\n', },);
 ```
 */
function prose(
  {
    proposal,
    standing,
  }: {
    readonly proposal: string;
    readonly standing: string;
  },
): boolean {
  return sameWording({
    proposal,
    standing,
    lineStructured: false,
  },);
}

//endregion Fixtures

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: sameWording.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'FOLDS a paragraph\'s soft line breaks on a prose slice, since the site renders each as a space, '
            + 'in both directions',
          fn: async () => {
            expect(prose({
              proposal: ONE_LINE,
              standing: SOFT,
            },),).toBe(true,);
            expect(prose({
              proposal: SOFT,
              standing: ONE_LINE,
            },),).toBe(true,);
          },
        },),

        it({
          name: 'FOLDS a blockquote the wrap rewrote, which the soft-break fold alone leaves as written',
          fn: async () => {
            expect(prose({
              proposal: '> The cat naps.\n> The dog waits.',
              standing: '> The cat naps. The dog waits.',
            },),).toBe(true,);
          },
        },),

        it({
          name: 'KEEPS APART what renders differently: a hard break, a blank line, an indented opening, and a '
            + 'changed word',
          fn: async () => {
            expect(prose({
              proposal: 'The cat naps on the mat  \nall afternoon.',
              standing: SOFT,
            },),).toBe(false,);
            expect(prose({
              proposal: 'The cat naps on the mat\\\nall afternoon.',
              standing: SOFT,
            },),).toBe(false,);
            expect(prose({
              proposal: 'The cat naps on the mat\n\nall afternoon.',
              standing: SOFT,
            },),).toBe(false,);
            expect(prose({
              proposal: `    ${ONE_LINE}`,
              standing: ONE_LINE,
            },),).toBe(false,);
            expect(prose({
              proposal: 'The cat naps on the rug\nall afternoon.',
              standing: SOFT,
            },),).toBe(false,);
          },
        },),

        it({
          name: 'KEEPS A LINE-STRUCTURED SLICE\'S LINES, which are the producer\'s work there, and still folds its '
            + 'trailing newline and its prose quote style',
          fn: async () => {
            expect(sameWording({
              proposal: ONE_LINE,
              standing: SOFT,
              lineStructured: true,
            },),).toBe(false,);
            expect(sameWording({
              proposal: `${SOFT}\n`,
              standing: SOFT,
              lineStructured: true,
            },),).toBe(true,);
            expect(sameWording({
              proposal: 'The cat didn\'t nap.',
              standing: 'The cat didn’t nap.',
              lineStructured: true,
            },),).toBe(true,);
          },
        },),

        it({
          name: 'KEEPS FRONT MATTER LINES APART, since there a line break is YAML syntax, and neither the wrap nor '
            + 'the fold reads inside the block',
          fn: async () => {
            expect(prose({
              proposal: '---\nname: Mittens\nsays: meow\n---',
              standing: '---\nname: Mittens\n\nsays: meow\n---',
            },),).toBe(false,);
          },
        },),

        it({
          name: 'READS A TEXT WHOSE FRONT MATTER DOES NOT PARSE AS WRITTEN rather than throwing, since the repair '
            + 'turn\'s copy check meets a model\'s reply before any floor: it is one wording only with itself, '
            + 'in all but trailing whitespace',
          fn: async () => {
            /**
             Fenced block whose one line is not a YAML mapping.
             */
            const unparseable = '---\nname: Mittens says: meow\n---';
            expect(prose({
              proposal: unparseable,
              standing: '---\nname: Mittens\nsays: meow\n---',
            },),).toBe(false,);
            expect(prose({
              proposal: `${unparseable}\n`,
              standing: unparseable,
            },),).toBe(true,);
          },
        },),
      ],
    },),

    describe({
      name: wordingKey.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'IS A KEY AND NEVER A TEXT TO SHIP: it folds the soft break the standing wording carries, which '
            + 'a caller shipping it would lose',
          fn: async () => {
            expect(wordingKey({
              text: SOFT,
              lineStructured: false,
            },),).toBe(ONE_LINE,);
            expect(wordingKey({
              text: SOFT,
              lineStructured: true,
            },),).toBe(SOFT,);
          },
        },),
        it({
          name: 'LAYS OUT AS WRITTEN a text nested too deeply to read, whose paragraphs no parser can locate, '
            + 'where plain markdown\'s refusal once ended the comparison',
          fn: async () => {
            /**
             Thousands of nested quotation markers, past what the parser's stack holds.
             */
            const deep = `${'>'.repeat(16_000,)} cat`;

            expect(wordingKey({
              text: `${deep}\n\n`,
              lineStructured: false,
            },),).toBe(deep,);
            expect(sameWording({
              proposal: `${deep}\n`,
              standing: deep,
              lineStructured: false,
            },),).toBe(true,);
            expect(sameWording({
              proposal: `${deep} dog`,
              standing: deep,
              lineStructured: false,
            },),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
