/**
 Tests the stub-marker strip the archive passes through before preparation.

 THE CASE IS ONE ARCHIVE: a page that is front matter, `(To-Do)`, an HTML
 comment of translator hints and nothing else, which the pipeline published
 with the marker standing over a finished translation. Here the marker goes
 with one blank line, the comment and the front matter stay byte for byte,
 and a marker inside a comment, a code fence, front matter or a sentence is
 left alone.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  isStubMarkerParagraph,
  stripStubMarkersWithOrigins,
  STUB_MARKER_TOKENS,
} from '@monochromatic-dev/translation-repair-page-check';
import { passArchiveText, } from '../../dist/final/node/index.mjs';

/**
 A stub archive opening shaped as the corpus stores one.
 */
const STUB_OPENING = [
  '---',
  'name: Juzi Orange',
  'info:',
  '    alias: Juzi',
  '---',
  '',
  '(To-Do)',
  '',
  '<!-- 翻译提示：',
  '',
  '这篇文章偶尔用猫的口吻。',
  '',
  '-->',
  '<!-- 猫爬架：Cat Tree -->',
  '',
  '## Experience',
  '',
].join('\n',);

/**
 The same opening with the marker and its blank gone.
 */
const STUB_STRIPPED = [
  '---',
  'name: Juzi Orange',
  'info:',
  '    alias: Juzi',
  '---',
  '',
  '<!-- 翻译提示：',
  '',
  '这篇文章偶尔用猫的口吻。',
  '',
  '-->',
  '<!-- 猫爬架：Cat Tree -->',
  '',
  '## Experience',
  '',
].join('\n',);

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: isStubMarkerParagraph.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS a bare or bracketed placeholder token in any case as a marker, and NOTHING ELSE',
          fn: async () => {
            expect(STUB_MARKER_TOKENS.has('to-do',),).toBe(true,);
            for (const paragraph of [
              '(To-Do)',
              'To-Do',
              'TODO',
              '[tbd]',
              '（WIP）',
              '  (todo)  ',
            ]) {
              expect(isStubMarkerParagraph({ paragraph, },),).toBe(true,);
            }
            for (const paragraph of [
              '(To-Do) write the childhood section',
              'the to-do list she kept',
              '()',
              '',
              'Done',
              '((To-Do))',
            ]) {
              expect(isStubMarkerParagraph({ paragraph, },),).toBe(false,);
            }
          },
        },),
        it({
          name: 'READS a placeholder in blockquote markers or a code span as a marker (ledger A1: one archive '
            + 'writes a nested blockquote of inline code saying Under Construction where the original has a heading)',
          fn: async () => {
            for (const paragraph of [
              '>>> `Under Construction`',
              '> Under Construction',
              '> > (WIP)',
              '`TODO`',
            ]) {
              expect(isStubMarkerParagraph({ paragraph, },),).toBe(true,);
            }
            for (const paragraph of [
              '> The tabby\'s house was under construction all spring.',
              '>>> ``',
              '>',
              '``TODO``',
              '> To be continued!',
            ]) {
              expect(isStubMarkerParagraph({ paragraph, },),).toBe(false,);
            }
          },
        },),
      ],
    },),

    describe({
      name: stripStubMarkersWithOrigins.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'REMOVES the stub marker with its following blank line and KEEPS the front matter and the '
            + 'comments byte for byte',
          fn: async () => {
            const { text, stripped, } = stripStubMarkersWithOrigins({ text: STUB_OPENING, },);
            expect(text,).toBe(STUB_STRIPPED,);
            expect(stripped,).toEqual([{
              lineNumber: 7,
              text: '(To-Do)',
            },],);
          },
        },),
        it({
          name: 'LEAVES a marker alone inside a comment, inside a code fence, inside front matter, or inside a '
            + 'sentence, and RETURNS a marker-free page unchanged',
          fn: async () => {
            for (const text of [
              '---\ntitle: (To-Do)\n---\n\nBody.\n',
              'Before.\n\n<!--\n\n(To-Do)\n\n-->\n\nAfter.\n',
              'Before.\n\n```\n\nTODO\n\n```\n\nAfter.\n',
              'Before.\n\n(To-Do) is what she called the list.\n\nAfter.\n',
              'Before.\n\nAfter.\n',
            ]) {
              const { text: kept, stripped, } = stripStubMarkersWithOrigins({ text, },);
              expect(kept,).toBe(text,);
              expect(stripped,).toEqual([],);
            }
          },
        },),
        it({
          name: 'LEAVES A MARKER INSIDE THE FRONT MATTER OF A PAGE OPENING WITH A BYTE ORDER MARK, as it leaves one '
            + 'inside the front matter of the same page without the mark',
          fn: async () => {
            /**
             Front matter whose own paragraph is a placeholder token, then a body.
             */
            const page = '---\nname: Mittens\n\nTODO\n\nnote: x\n---\n\nThe cat naps.\n';
            for (const text of [page, `\uFEFF${page}`,]) {
              expect(stripStubMarkersWithOrigins({ text, },).text,).toBe(text,);
              expect(stripStubMarkersWithOrigins({ text, },).stripped,).toEqual([],);
            }
          },
        },),
        it({
          name: 'REMOVES a marker that ends the document with the blank line above it, so no trailing blank '
            + 'pair is left, and REMOVES a marker that is the whole body',
          fn: async () => {
            expect(stripStubMarkersWithOrigins({ text: 'Body.\n\n(To-Do)\n', },).text,).toBe('Body.\n',);
            expect(stripStubMarkersWithOrigins({ text: '---\nname: X\n---\n\nTBD\n', },).text,).toBe('---\nname: X\n---\n',);
            expect(stripStubMarkersWithOrigins({ text: '(To-Do)', },).text,).toBe('',);
          },
        },),
        it({
          name: 'DROPS the blank line ABOVE a marker that ends the document with no trailing newline, since no '
            + 'following line stands to give up its blank instead',
          fn: async () => {
            expect(stripStubMarkersWithOrigins({ text: 'Body.\n\n(To-Do)', },),).toEqual({
              text: 'Body.',
              stripped: [{
                lineNumber: 3,
                text: '(To-Do)',
              },],
              lines: [{
                text: 'Body.',
                lineNumber: 1,
              },],
            },);
          },
        },),
      ],
    },),

    describe({
      name: passArchiveText.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'FOLDS invisible variants first and STRIPS the marker second, so a marker spelled with a '
            + 'non-breaking hyphen still goes',
          fn: async () => {
            const archive = passArchiveText({
              text: 'Before.\n\n(To‑Do)\n\nnon‑binary after.\n',
              l: tagged({ tag: 'archive-stub-test', },),
            },);
            expect(archive,).toBe('Before.\n\nnon-binary after.\n',);
          },
        },),
        it({
          name: 'KEEPS A LEADING BYTE ORDER MARK the archive opens with, which is no content, and still FOLDS every '
            + 'other mark, so an archive the pass changes in nothing else is written back byte for byte',
          fn: async () => {
            /**
             Page with front matter, a marker spelled with a non-breaking hyphen and one invisible variant.
             */
            const page = '---\nname: Mittens\n---\n\n(To\u2011Do)\n\nThe cat naps, non\u2011binary.\n';
            /**
             The same page after the fold and the strip.
             */
            const expected = '---\nname: Mittens\n---\n\nThe cat naps, non-binary.\n';
            expect(passArchiveText({
              text: page,
              l: tagged({ tag: 'archive-stub-test', },),
            },),).toBe(expected,);
            expect(passArchiveText({
              text: `\uFEFF${page}`,
              l: tagged({ tag: 'archive-stub-test', },),
            },),).toBe(`\uFEFF${expected}`,);
            expect(passArchiveText({
              text: '\uFEFFThe cat naps.\n\uFEFF\nThe cat wakes.\n',
              l: tagged({ tag: 'archive-stub-test', },),
            },),).toBe('\uFEFFThe cat naps.\n\nThe cat wakes.\n',);
          },
        },),
      ],
    },),
  ],
},);
