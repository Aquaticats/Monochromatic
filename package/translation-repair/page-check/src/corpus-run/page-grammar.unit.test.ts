/**
 Tests for the page-level floor that refuses a page the MDX grammar cannot
 parse.

 WHY THIS FLOOR EXISTS. On 2026-09-06 the yulianNyanner page shipped a
 component line whose JSX string literal had been curled into typographic
 quotes at the would-ship reading, after every slice floor had passed the
 straight-quoted slice. A page the site cannot compile is not a page, and
 nothing between the would-ship reading and the disk read it as a document.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  assertPageParses,
  readPageGrammar,
  UnparseablePageError,
} from '../../dist/final/node/index.mjs';

/**
 Left double quotation mark.
 */
const OPEN = '\u{201C}';

/**
 Right double quotation mark.
 */
const CLOSE = '\u{201D}';

/**
 Page whose blockquoted component line carries straight quotes, as the
 archives write it.
 */
const PARSING_PAGE = `---\nname: Cat\n---\n\n## Cat\n\n> Pick up a mouse, if you have one.\n>\n`
  + `> <PhotoScroll photos={["\${path}/photos/photo3.webp"]} />\n\nIt sleeps.\n`;

/**
 Same page with the literal curled, the shape of 2026-09-06.
 */
const CURLED_PAGE = PARSING_PAGE.replace(
  `{["\${path}/photos/photo3.webp"]}`,
  `{[${OPEN}\${path}/photos/photo3.webp${CLOSE}]}`,
);

await describe({
  name: 'page grammar',
  children: [
    describe({
      name: assertPageParses.name,
      children: [
        it({
          name: 'REFUSES a page whose JSX string literal carries curly quotes, naming the entry and '
            + 'the refusal site and quoting nothing (2026-09-06)',
          fn: async () => {
            /**
             What the floor threw on the curled page.
             */
            const refusal = caught(function publishCurled(): void {
              assertPageParses({
                entryId: 'Cat',
                pageText: CURLED_PAGE,
              },);
            },);

            expect(refusal,).toBeInstanceOf(UnparseablePageError,);
            expect((refusal as Error).message,).toContain('Cat',);
            expect((refusal as Error).message,).not.toContain('PhotoScroll',);
            expect((refusal as Error).message,).not.toContain('photo3',);
            expect((refusal as Error).message,).not.toContain('It sleeps',);
          },
        },),

        it({
          name: 'PASSES the same page with the literal in straight quotes, which is the control',
          fn: async () => {
            expect(function publishParsing(): void {
              assertPageParses({
                entryId: 'Cat',
                pageText: PARSING_PAGE,
              },);
            },).not.toThrow();
          },
        },),

        it({
          name: 'PASSES a page whose curly quotes sit in prose and inside an HTML comment, since the '
            + 'grammar reads neither as code',
          fn: async () => {
            expect(function publishProse(): void {
              assertPageParses({
                entryId: 'Cat',
                pageText: `## Cat\n\n<!-- ${OPEN}a note${CLOSE} -->\n\nShe said ${OPEN}hello${CLOSE}.\n`,
              },);
            },).not.toThrow();
          },
        },),

        it({
          name: 'REFUSES an unclosed component tag too, since the floor is the grammar and not one '
            + 'quote rule',
          fn: async () => {
            expect(function publishUnclosed(): void {
              assertPageParses({
                entryId: 'Cat',
                pageText: '## Cat\n\n<PhotoScroll photos={[]}\n\nIt sleeps.\n',
              },);
            },).toThrow(UnparseablePageError,);
          },
        },),
      ],
    },),

    describe({
      name: readPageGrammar.name,
      children: [
        it({
          name: 'READS A PAGE THAT PARSES as parsing, and names no refusal',
          fn: async () => {
            expect(readPageGrammar({ pageText: PARSING_PAGE, },),).toEqual({ kind: 'parses', },);
          },
        },),

        it({
          name: 'READS A PAGE THE STRICT GRAMMAR FELL BACK ON as refused, with the refusal site and no page text',
          fn: async () => {
            /**
             What the grammar said of the curled page.
             */
            const reading = readPageGrammar({ pageText: CURLED_PAGE, },);
            expect(reading.kind,).toBe('refused',);
            expect(JSON.stringify(reading,),).not.toContain('photo3',);
          },
        },),

        it({
          name: 'READS A TEXT NESTED PAST THE BOUND OF THE MARKDOWN FALLBACK as refused too, which parseDocument '
            + 'raises as an error of its own and not as a downgrade',
          fn: async () => {
            /**
             What the grammar said of three hundred open brackets.
             */
            const reading = readPageGrammar({ pageText: `${'['.repeat(300,)}\n`, },);
            expect(reading.kind,).toBe('refused',);
          },
        },),

        it({
          name: 'READS A FRONT MATTER FENCE WHOSE YAML DOES NOT PARSE as refused, the other error parseDocument raises',
          fn: async () => {
            /**
             What the grammar said of a fence holding an unclosed flow sequence.
             */
            const reading = readPageGrammar({ pageText: '---\na: [\n---\n\n她说\n', },);
            expect(reading.kind,).toBe('refused',);
          },
        },),

        it({
          name: 'LETS A FAULT IN THE CODE PROPAGATE, so only the three refusals answer "refused"',
          fn: async () => {
            expect(function readNonText(): void {
              readPageGrammar({ pageText: 42 as unknown as string, },);
            },).toThrow();
          },
        },),
      ],
    },),
  ],
},);
