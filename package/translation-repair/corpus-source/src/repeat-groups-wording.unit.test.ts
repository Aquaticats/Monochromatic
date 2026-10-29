/**
 Tests for the wording reader behind the repeat groups: the stream of a
 page's body paragraphs, one element per code point with its offset in UTF-16
 units and its quotation depth, and the helpers that decide what the stream
 cuts and what it keeps. Fixtures are cat-themed invention only.

 @module
 */

import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { parseDocument, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  blankedAt,
  BREAK,
  cutEndAt,
  cutSetOf,
  hasMarkupOpener,
  hiddenCoverOf,
  hiddenRangesOf,
  wordingEndAt,
  wordingStreamOf,
} from '../dist/final/node/index.mjs';

/**
 Reads the wording stream of a page.

 @param text - full page text

 @returns Characters, offsets and quotation depths of its body paragraphs

 @example
 ```ts
 const stream = streamOf({ text: '猫咪\n', },);
 ```
 */
function streamOf({ text, }: { readonly text: string; },) {
  return wordingStreamOf({
    text,
    document: parseDocument({ text, },),
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: wordingStreamOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'Reads one element per code point and counts each offset in UTF-16 units',
          fn: async () => {
            expect(streamOf({ text: '猫😀咪。\n', },),).toEqual({
              chars: ['猫', '😀', '咪', '。', BREAK,],
              offsets: [0, 1, 3, 4, -1,],
              depths: [0, 0, 0, 0, 0,],
            },);
          },
        },),

        it({
          name: 'Reads a lone surrogate half as one element of its own',
          fn: async () => {
            expect(streamOf({ text: '猫\uD83D咪\n', },),).toEqual({
              chars: ['猫', '\uD83D', '咪', BREAK,],
              offsets: [0, 1, 2, -1,],
              depths: [0, 0, 0, 0,],
            },);
          },
        },),

        it({
          name: 'Ends each paragraph with a break and starts the first without one',
          fn: async () => {
            expect(streamOf({ text: '猫咪\n\n窗台\n', },),).toEqual({
              chars: ['猫', '咪', BREAK, '窗', '台', BREAK,],
              offsets: [0, 1, -1, 4, 5, -1,],
              depths: [0, 0, 0, 0, 0, 0,],
            },);
          },
        },),

        it({
          name: 'Drops spaces, no-break spaces, ideographic spaces and line breaks inside a paragraph without a break',
          fn: async () => {
            expect(streamOf({ text: '猫 咪\u00A0窗\u3000台\n阳\n', },),).toEqual({
              chars: ['猫', '咪', '窗', '台', '阳', BREAK,],
              offsets: [0, 2, 4, 6, 8, -1,],
              depths: [0, 0, 0, 0, 0, 0,],
            },);
          },
        },),

        it({
          name: 'Reads a carriage return before each line feed as whitespace',
          fn: async () => {
            expect(streamOf({ text: '猫咪\r\n窗台\r\n\r\n阳\r\n', },),).toEqual({
              chars: ['猫', '咪', '窗', '台', BREAK, '阳', BREAK,],
              offsets: [0, 1, 4, 5, -1, 10, -1,],
              depths: [0, 0, 0, 0, 0, 0, 0,],
            },);
          },
        },),

        it({
          name: 'Keeps a zero-width space as a wording character, since it is no whitespace',
          fn: async () => {
            expect(streamOf({ text: '猫\u200B咪\n', },),).toEqual({
              chars: ['猫', '\u200B', '咪', BREAK,],
              offsets: [0, 1, 2, -1,],
              depths: [0, 0, 0, 0,],
            },);
          },
        },),

        it({
          name: 'Counts offsets from the start of the page, front matter included',
          fn: async () => {
            /** Page with front matter ahead of its one paragraph. */
            const text = '---\ntitle: 猫\n---\n\n猫咪\n';
            /** Offset of the paragraph. */
            const start = text.indexOf('猫咪',);

            expect(streamOf({ text, },),).toEqual({
              chars: ['猫', '咪', BREAK,],
              offsets: [start, start + 1, -1,],
              depths: [0, 0, 0,],
            },);
          },
        },),

        it({
          name: 'Reads body paragraphs only, leaving out front matter, headings, quotes, lists, code and footnote definitions',
          fn: async () => {
            /** Page with one block of every kind the reader leaves out, around one paragraph. */
            const text = '---\ntitle: 猫\n---\n\n## 窗台\n\n> 咪\n\n- 阳\n\n```\n盹儿\n```\n\n[^1]: 注\n\n猫咪。\n';
            /** Offset of the one paragraph. */
            const start = text.indexOf('猫咪。',);

            expect(streamOf({ text, },),).toEqual({
              chars: ['猫', '咪', '。', BREAK,],
              offsets: [start, start + 1, start + 2, -1,],
              depths: [0, 0, 0, 0,],
            },);
          },
        },),

        it({
          name: 'Adds no element for a page without a paragraph',
          fn: async () => {
            expect(streamOf({ text: '', },),).toEqual({
              chars: [],
              offsets: [],
              depths: [],
            },);
          },
        },),

        it({
          name: 'Adds no break for a paragraph that is cut away whole',
          fn: async () => {
            expect(streamOf({ text: '`code`\n\n猫咪\n', },),).toEqual({
              chars: ['猫', '咪', BREAK,],
              offsets: [8, 9, -1,],
              depths: [0, 0, 0,],
            },);
          },
        },),

        it({
          name: 'Leaves a break where a bare URL is cut',
          fn: async () => {
            expect(streamOf({ text: '猫 https://cat.example.invalid/x 咪\n', },),).toEqual({
              chars: ['猫', BREAK, '咪', BREAK,],
              offsets: [0, -1, 32, -1,],
              depths: [0, 0, 0, 0,],
            },);
          },
        },),

        it({
          name: 'Leaves a break where inline code is cut',
          fn: async () => {
            expect(streamOf({ text: '猫`code`咪\n', },),).toEqual({
              chars: ['猫', BREAK, '咪', BREAK,],
              offsets: [0, -1, 7, -1,],
              depths: [0, 0, 0, 0,],
            },);
          },
        },),

        it({
          name: 'Leaves a break where a footnote reference is cut',
          fn: async () => {
            expect(streamOf({ text: '猫[^1]咪\n\n[^1]: 注\n', },),).toEqual({
              chars: ['猫', BREAK, '咪', BREAK,],
              offsets: [0, -1, 5, -1,],
              depths: [0, 0, 0, 0,],
            },);
          },
        },),

        it({
          name: 'Leaves a break where a link destination is cut, and keeps the bracket and the label before it',
          fn: async () => {
            expect(streamOf({ text: '[猫](./a)咪\n', },),).toEqual({
              chars: ['[', '猫', BREAK, '咪', BREAK,],
              offsets: [0, 1, -1, 8, -1,],
              depths: [0, 0, 0, 0, 0,],
            },);
          },
        },),

        it({
          name: 'Reads past an HTML comment inside a paragraph without a break',
          fn: async () => {
            expect(streamOf({ text: '猫<!-- 注 -->咪\n', },),).toEqual({
              chars: ['猫', '咪', BREAK,],
              offsets: [0, 11, -1,],
              depths: [0, 0, 0,],
            },);
          },
        },),

        it({
          name: 'Gives an HTML comment no say in the test for markup',
          fn: async () => {
            expect(streamOf({ text: '猫<!-- {x} <b> -->咪\n', },),).toEqual({
              chars: ['猫', '咪', BREAK,],
              offsets: [0, 17, -1,],
              depths: [0, 0, 0,],
            },);
          },
        },),

        it({
          name: 'Gives an HTML comment no say in the inline cuts',
          fn: async () => {
            expect(streamOf({ text: '猫<!--https://x-->咪\n', },),).toEqual({
              chars: ['猫', '咪', BREAK,],
              offsets: [0, 17, -1,],
              depths: [0, 0, 0,],
            },);
          },
        },),

        it({
          name: 'Gives inline code no say in the test for markup, so a brace inside it leaves the paragraph read',
          fn: async () => {
            expect(streamOf({ text: '猫`{x}`咪\n', },),).toEqual({
              chars: ['猫', BREAK, '咪', BREAK,],
              offsets: [0, -1, 6, -1,],
              depths: [0, 0, 0, 0,],
            },);
          },
        },),

        it({
          name: 'Gives a bare URL no say in the test for markup, so a brace inside it leaves the paragraph read',
          fn: async () => {
            expect(streamOf({ text: '猫 https://x/{id} 咪\n', },),).toEqual({
              chars: ['猫', BREAK, '咪', BREAK,],
              offsets: [0, -1, 17, -1,],
              depths: [0, 0, 0, 0,],
            },);
          },
        },),

        it({
          name: 'Reads past the tags of a container the parse widened a paragraph over',
          fn: async () => {
            expect(streamOf({ text: '<details>\n\n猫咪\n\n</details>\n\n窗台\n', },),).toEqual({
              chars: ['猫', '咪', BREAK, '窗', '台', BREAK,],
              offsets: [11, 12, -1, 27, 28, -1,],
              depths: [0, 0, 0, 0, 0, 0,],
            },);
          },
        },),

        it({
          name: 'Leaves a paragraph with an inline tag out whole',
          fn: async () => {
            expect(streamOf({ text: '猫<b>咪</b>\n', },),).toEqual({
              chars: [],
              offsets: [],
              depths: [],
            },);
          },
        },),

        it({
          name: 'Leaves a paragraph with a closing tag out whole',
          fn: async () => {
            expect(streamOf({ text: '猫</div>咪\n', },),).toEqual({
              chars: [],
              offsets: [],
              depths: [],
            },);
          },
        },),

        it({
          name: 'Leaves a paragraph with an expression brace out whole',
          fn: async () => {
            expect(streamOf({ text: '猫{x}咪\n', },),).toEqual({
              chars: [],
              offsets: [],
              depths: [],
            },);
          },
        },),

        it({
          name: 'Reads a less-than sign before a digit as wording',
          fn: async () => {
            expect(streamOf({ text: '温度<5度\n', },),).toEqual({
              chars: ['温', '度', '<', '5', '度', BREAK,],
              offsets: [0, 1, 2, 3, 4, -1,],
              depths: [0, 0, 0, 0, 0, 0,],
            },);
          },
        },),

        it({
          name: 'Counts an opening mark as inside its quotation and a closing mark as outside it',
          fn: async () => {
            expect(streamOf({ text: '猫「咪」窗\n', },),).toEqual({
              chars: ['猫', '「', '咪', '」', '窗', BREAK,],
              offsets: [0, 1, 2, 3, 4, -1,],
              depths: [0, 1, 1, 0, 0, 0,],
            },);
          },
        },),

        it({
          name: 'Counts curly double quotation marks as a quotation',
          fn: async () => {
            expect(streamOf({ text: '“猫”咪\n', },),).toEqual({
              chars: ['“', '猫', '”', '咪', BREAK,],
              offsets: [0, 1, 2, 3, -1,],
              depths: [1, 1, 0, 0, 0,],
            },);
          },
        },),

        it({
          name: 'Counts the depth of nested quotations and leaves the outer one open after the inner one closes',
          fn: async () => {
            expect(streamOf({ text: '「猫『咪』窗」台\n', },),).toEqual({
              chars: ['「', '猫', '『', '咪', '』', '窗', '」', '台', BREAK,],
              offsets: [0, 1, 2, 3, 4, 5, 6, 7, -1,],
              depths: [1, 1, 2, 2, 1, 1, 0, 0, 0,],
            },);
          },
        },),

        it({
          name: 'Keeps a closing mark with no opening mark at depth zero and reads a later quotation as quoted',
          fn: async () => {
            expect(streamOf({ text: '猫」咪「窗\n', },),).toEqual({
              chars: ['猫', '」', '咪', '「', '窗', BREAK,],
              offsets: [0, 1, 2, 3, 4, -1,],
              depths: [0, 0, 0, 1, 1, 0,],
            },);
          },
        },),

        it({
          name: 'Keeps an opening mark that never closes open to the end of its paragraph and no further',
          fn: async () => {
            expect(streamOf({ text: '猫「咪\n\n窗台\n', },),).toEqual({
              chars: ['猫', '「', '咪', BREAK, '窗', '台', BREAK,],
              offsets: [0, 1, 2, -1, 5, 6, -1,],
              depths: [0, 1, 1, 0, 0, 0, 0,],
            },);
          },
        },),

        it({
          name: 'Opens and closes a quotation with straight double quotation marks in turn',
          fn: async () => {
            expect(streamOf({ text: '猫"咪"窗"台\n', },),).toEqual({
              chars: ['猫', '"', '咪', '"', '窗', '"', '台', BREAK,],
              offsets: [0, 1, 2, 3, 4, 5, 6, -1,],
              depths: [0, 1, 1, 0, 0, 1, 1, 0,],
            },);
          },
        },),

        it({
          name: 'Counts straight double quotation marks inside a corner-bracket quotation one level deeper',
          fn: async () => {
            expect(streamOf({ text: '「猫"咪"窗」\n', },),).toEqual({
              chars: ['「', '猫', '"', '咪', '"', '窗', '」', BREAK,],
              offsets: [0, 1, 2, 3, 4, 5, 6, -1,],
              depths: [1, 1, 2, 2, 1, 1, 0, 0,],
            },);
          },
        },),

        it({
          name: 'Leaves a straight double quotation mark open to the end of its paragraph when none closes it',
          fn: async () => {
            expect(streamOf({ text: '猫"咪\n\n窗台\n', },),).toEqual({
              chars: ['猫', '"', '咪', BREAK, '窗', '台', BREAK,],
              offsets: [0, 1, 2, -1, 5, 6, -1,],
              depths: [0, 1, 1, 0, 0, 0, 0,],
            },);
          },
        },),
      ],
    },),

    describe({
      name: wordingEndAt.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'Gives the offset one unit past a character inside the basic plane',
          fn: async () => {
            /** Stream of one paragraph, whose first element is a basic-plane character at offset zero. */
            const wording = streamOf({ text: '猫😀咪\n', },);

            expect(wordingEndAt({ wording, index: 0, },),).toBe(1,);
          },
        },),

        it({
          name: 'Gives the offset two units past a character beyond the first plane',
          fn: async () => {
            /** Stream whose second element is a character beyond the first plane at offset one. */
            const wording = streamOf({ text: '猫😀咪\n', },);

            expect(wordingEndAt({ wording, index: 1, },),).toBe(3,);
          },
        },),

        it({
          name: 'Gives the offset one unit past a lone surrogate half',
          fn: async () => {
            /** Stream whose second element is a lone surrogate half at offset one. */
            const wording = streamOf({ text: '猫\uD83D咪\n', },);

            expect(wordingEndAt({ wording, index: 1, },),).toBe(2,);
          },
        },),

        it({
          name: 'Refuses an index past the end of the stream',
          fn: async () => {
            /** Stream of three elements, a break among them. */
            const wording = streamOf({ text: '猫咪\n', },);
            /** What the lookup throws for the index past the break. */
            const refusal = caught(function pastTheEnd(): unknown {
              return wordingEndAt({ wording, index: 3, },);
            },);

            expect(refusal,).toBeInstanceOf(Error,);
            expect(String(refusal,),).toBe('Error: Expected non-nullish value, got undefined',);
          },
        },),
      ],
    },),

    describe({
      name: cutEndAt.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'Ends a bare https URL at the first space',
          fn: async () => {
            expect(cutEndAt({ body: '看 https://cat.example.invalid/a 好', index: 2, },),).toBe(31,);
          },
        },),

        it({
          name: 'Ends a bare http URL at the first space and at the end of the text when no space follows',
          fn: async () => {
            expect(cutEndAt({ body: 'http://cat.example.invalid/a 好', index: 0, },),).toBe(28,);
            expect(cutEndAt({ body: 'http://cat.example.invalid/a', index: 0, },),).toBe(28,);
          },
        },),

        it({
          name: 'Ends a bare URL at a no-break space as it would at a space',
          fn: async () => {
            expect(cutEndAt({ body: 'http://cat\u00A0好', index: 0, },),).toBe(10,);
          },
        },),

        it({
          name: 'Cuts no scheme with nothing after it',
          fn: async () => {
            expect(cutEndAt({ body: 'https:// 好', index: 0, },),).toBe(-1,);
            expect(cutEndAt({ body: 'http://', index: 0, },),).toBe(-1,);
          },
        },),

        it({
          name: 'Cuts no URL from an index that does not begin one',
          fn: async () => {
            expect(cutEndAt({ body: 'a https://cat.example.invalid/a', index: 1, },),).toBe(-1,);
          },
        },),

        it({
          name: 'Ends inline code after its closing backtick',
          fn: async () => {
            expect(cutEndAt({ body: '猫`ab`咪', index: 1, },),).toBe(5,);
          },
        },),

        it({
          name: 'Cuts no inline code whose closing backtick is missing or on the next line',
          fn: async () => {
            expect(cutEndAt({ body: '猫`ab咪', index: 1, },),).toBe(-1,);
            expect(cutEndAt({ body: '猫`ab\n咪`', index: 1, },),).toBe(-1,);
          },
        },),

        it({
          name: 'Ends a footnote reference after its closing bracket',
          fn: async () => {
            expect(cutEndAt({ body: '猫[^ab]咪', index: 1, },),).toBe(6,);
          },
        },),

        it({
          name: 'Cuts no footnote reference whose closing bracket is missing or on the next line',
          fn: async () => {
            expect(cutEndAt({ body: '猫[^ab咪', index: 1, },),).toBe(-1,);
            expect(cutEndAt({ body: '猫[^ab\n咪]', index: 1, },),).toBe(-1,);
          },
        },),

        it({
          name: 'Cuts no opening bracket that is not followed by a caret',
          fn: async () => {
            expect(cutEndAt({ body: '猫[ab]咪', index: 1, },),).toBe(-1,);
          },
        },),

        it({
          name: 'Ends a link destination after its closing parenthesis',
          fn: async () => {
            expect(cutEndAt({ body: '[猫](./a)咪', index: 2, },),).toBe(8,);
          },
        },),

        it({
          name: 'Cuts no link destination whose closing parenthesis is missing or on the next line',
          fn: async () => {
            expect(cutEndAt({ body: '[猫](./a咪', index: 2, },),).toBe(-1,);
            expect(cutEndAt({ body: '[猫](./a\n咪)', index: 2, },),).toBe(-1,);
          },
        },),

        it({
          name: 'Cuts no closing bracket that is not followed by an opening parenthesis',
          fn: async () => {
            expect(cutEndAt({ body: '[猫]咪', index: 2, },),).toBe(-1,);
          },
        },),

        it({
          name: 'Cuts nothing from a plain character',
          fn: async () => {
            expect(cutEndAt({ body: '猫咪', index: 0, },),).toBe(-1,);
          },
        },),
      ],
    },),

    describe({
      name: cutSetOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'Covers every unit of each cut and no unit outside one',
          fn: async () => {
            /** Paragraph with inline code, a footnote reference and a link destination, each between plain characters. */
            const body = '猫`ab`咪[^1]窗[台](./a)阳';

            expect([...cutSetOf({ body, },),],).toEqual([1, 2, 3, 4, 6, 7, 8, 9, 13, 14, 15, 16, 17, 18,],);
          },
        },),

        it({
          name: 'Lets a cut consume the text it holds, so that a cut-like stretch inside it adds nothing',
          fn: async () => {
            /** Inline code that holds a footnote reference. */
            const body = '猫`a[^1]b`咪';

            expect([...cutSetOf({ body, },),],).toEqual([1, 2, 3, 4, 5, 6, 7, 8,],);
          },
        },),

        it({
          name: 'Resumes after a cut that ends the paragraph and covers nothing for plain text',
          fn: async () => {
            expect([...cutSetOf({ body: '猫咪`ab`', },),],).toEqual([2, 3, 4, 5,],);
            expect([...cutSetOf({ body: '猫咪', },),],).toEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: hasMarkupOpener.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'Finds an expression brace anywhere in the text',
          fn: async () => {
            expect(hasMarkupOpener({ body: '猫咪{x}', },),).toBe(true,);
          },
        },),

        it({
          name: 'Finds a tag that opens with a lowercase or an uppercase letter and a closing tag',
          fn: async () => {
            expect(hasMarkupOpener({ body: '猫<b>咪', },),).toBe(true,);
            expect(hasMarkupOpener({ body: '猫<B>咪', },),).toBe(true,);
            expect(hasMarkupOpener({ body: '猫</b>咪', },),).toBe(true,);
          },
        },),

        it({
          name: 'Reads a less-than sign before a digit, a space, a comment bang, a non-Latin letter or the end as wording',
          fn: async () => {
            expect(hasMarkupOpener({ body: '温度<5度', },),).toBe(false,);
            expect(hasMarkupOpener({ body: '猫 < 咪', },),).toBe(false,);
            expect(hasMarkupOpener({ body: '猫<!-- 注 -->咪', },),).toBe(false,);
            expect(hasMarkupOpener({ body: '猫<咪', },),).toBe(false,);
            expect(hasMarkupOpener({ body: '猫咪<', },),).toBe(false,);
          },
        },),

        it({
          name: 'Finds no markup in plain text',
          fn: async () => {
            expect(hasMarkupOpener({ body: '猫咪', },),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: hiddenRangesOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'Lists the range of a terminated HTML comment',
          fn: async () => {
            /** Page whose paragraph holds a comment. */
            const text = '猫<!-- 注 -->咪\n';

            expect(hiddenRangesOf({ document: parseDocument({ text, },), },),).toEqual([
              {
                startOffset: 1,
                endOffset: 11,
              },
            ],);
          },
        },),

        it({
          name: 'Lists the range of an HTML comment that never closes, to the end of the page body',
          fn: async () => {
            /** Page whose second paragraph opens a comment it never closes. */
            const text = '猫咪\n\n窗<!-- 注\n';

            expect(hiddenRangesOf({ document: parseDocument({ text, },), },),).toEqual([
              {
                startOffset: 5,
                endOffset: 12,
              },
            ],);
          },
        },),

        it({
          name: 'Lists both tags of a container the parse dissolved',
          fn: async () => {
            /** Page whose container holds one paragraph. */
            const text = '<details>\n\n猫咪\n\n</details>\n\n窗台\n';

            expect(hiddenRangesOf({ document: parseDocument({ text, },), },),).toEqual([
              {
                startOffset: 0,
                endOffset: 11,
              },
              {
                startOffset: 13,
                endOffset: 25,
              },
            ],);
          },
        },),

        it({
          name: 'Lists nothing for a page with no comment and no container',
          fn: async () => {
            expect(hiddenRangesOf({ document: parseDocument({ text: '猫咪\n', },), },),).toEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: hiddenCoverOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'Covers the indexes of a range that lies inside the paragraph, counted from the paragraph start',
          fn: async () => {
            expect([...hiddenCoverOf({
              ranges: [{ startOffset: 12, endOffset: 15, },],
              startOffset: 10,
              endOffset: 20,
            },),],).toEqual([2, 3, 4,],);
          },
        },),

        it({
          name: 'Covers only the part of a range that lies inside the paragraph when it starts before it or ends after it',
          fn: async () => {
            expect([...hiddenCoverOf({
              ranges: [
                { startOffset: 5, endOffset: 12, },
                { startOffset: 18, endOffset: 30, },
              ],
              startOffset: 10,
              endOffset: 20,
            },),],).toEqual([0, 1, 8, 9,],);
          },
        },),

        it({
          name: 'Covers nothing for a range outside the paragraph or no range at all',
          fn: async () => {
            expect([...hiddenCoverOf({
              ranges: [{ startOffset: 0, endOffset: 10, },],
              startOffset: 10,
              endOffset: 20,
            },),],).toEqual([],);
            expect([...hiddenCoverOf({
              ranges: [],
              startOffset: 10,
              endOffset: 20,
            },),],).toEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: blankedAt.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'Replaces each covered unit with a space and keeps the length',
          fn: async () => {
            expect(blankedAt({ body: '猫咪窗台', covered: new Set([1, 2,],), },),).toBe('猫  台',);
          },
        },),

        it({
          name: 'Blanks the two units of a character beyond the first plane one by one',
          fn: async () => {
            expect(blankedAt({ body: '猫😀咪', covered: new Set([1, 2,],), },),).toBe('猫  咪',);
            expect(blankedAt({ body: '猫😀咪', covered: new Set([2,],), },),).toBe(`猫${'😀'.charAt(0,)} 咪`,);
          },
        },),

        it({
          name: 'Returns the paragraph as it stands when nothing is covered',
          fn: async () => {
            expect(blankedAt({ body: '猫咪', covered: new Set<number>(), },),).toBe('猫咪',);
          },
        },),
      ],
    },),
  ],
},);
