/**
 Tests for front matter splitting.
 Fixtures mirror corpus structure only. Cat-themed invention throughout; no
 corpus content appears here.

 @module
 */

import { parse as parseYaml, } from 'yaml';

import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  FrontMatterParseError,
  namesWithoutQuoting,
  requireFrontMatterRefusal,
  splitFrontMatter,
} from '../dist/final/node/index.mjs';

/**
 Well-formed corpus-shaped source with invented metadata and body.
 */
const CORPUS_SHAPED = '---\nname: 小猫-whiskers\ninfo:\n    alias: Whiskers, Mittens\n---\n\n## 简介\n\n猫猫晒太阳。\n';

//region Front matter refusal disclosure

/**
 Word appearing nowhere else in this file, so an assertion of absence cannot
 pass by accident.
 */
const REFUSAL_FIXTURE_WORD = 'Tuftmallow';

/**
 Front matter whose YAML refuses, with the fixture word on the offending line.

 MEASURED: this refuses as `BLOCK_AS_IMPLICIT_KEY` at line 1 column 7, and the
 parser's own message reproduces the line. `rawYamlRefusal`, the control, asserts that,
 because an absence assertion against a probe that cannot show a difference
 proves nothing.
 */
const REFUSING_SOURCE = `---\nname: ${REFUSAL_FIXTURE_WORD}\n  bad: [x\n---\n喵。\n`;

/**
 Reads what the YAML parser says with nothing between it and a reader.

 @param yamlSource - YAML that must refuse

 @returns Parser's own message, code frame included

 @throws {@link Error} where the control fixture parsed, which would leave the
 absence assertions unproven

 @example
 ```ts
 const raw = rawYamlRefusal({ yamlSource: 'a: [x\n', },);
 ```
 */
function rawYamlRefusal({ yamlSource, }: { readonly yamlSource: string; },): string {
  try {
    parseYaml(yamlSource,);
  }
  catch (error) {
    if (Error.isError(error,))
      return error.message;

    throw error;
  }

  throw new Error('the control fixture parsed, so it proves nothing',);
}

/**
 Splits source that must refuse, handing the refusal back to be read.

 @param text - source whose front matter will not parse

 @returns Refusal the split raised

 @throws {@link Error} where the fixture parsed, which would mean it no longer
 exercises anything

 @example
 ```ts
 const refusal = refusalFrom({ text: REFUSING_SOURCE, },);
 ```
 */
function refusalFrom({ text, }: { readonly text: string; },): Error {
  try {
    splitFrontMatter({ text, },);
  }
  catch (error) {
    if (Error.isError(error,))
      return error;

    throw error;
  }

  throw new Error('the fixture parsed, so it no longer exercises a refusal',);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: splitFrontMatter.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'returns whole input as body when no fence opens the document',
          fn: async () => {
            /** Split of source without front matter. */
            const split = splitFrontMatter({ text: '## 简介\n\n猫猫晒太阳。\n', },);
            expect(split.frontMatter,).toBe(undefined,);
            expect(split.body,).toBe('## 简介\n\n猫猫晒太阳。\n',);
            expect(split.bodyOffset,).toBe(0,);
          },
        },),

        it({
          name: 'splits well-formed front matter with exact raw slice and parsed data',
          fn: async () => {
            /** Split of corpus-shaped source. */
            const split = splitFrontMatter({ text: CORPUS_SHAPED, },);
            expect(split.frontMatter?.raw,).toBe(
              '---\nname: 小猫-whiskers\ninfo:\n    alias: Whiskers, Mittens\n---\n',
            );
            expect(split.frontMatter?.data,).toEqual({
              name: '小猫-whiskers',
              info: { alias: 'Whiskers, Mittens', },
            },);
            expect(split.bodyOffset,).toBe(split.frontMatter?.raw.length,);
            expect(split.body,).toBe('\n## 简介\n\n猫猫晒太阳。\n',);
            // Reassembly invariant: raw plus body reproduces source byte-for-byte.
            expect(`${split.frontMatter?.raw ?? ''}${split.body}`,).toBe(CORPUS_SHAPED,);
          },
        },),

        it({
          name: 'parses empty front matter as null data',
          fn: async () => {
            /** Split of source with empty fence pair. */
            const split = splitFrontMatter({ text: '---\n---\n喵。\n', },);
            expect(split.frontMatter?.data,).toBe(null,);
            expect(split.body,).toBe('喵。\n',);
          },
        },),

        it({
          name: 'treats unterminated fence as body text',
          fn: async () => {
            /** Split of source whose fence never closes. */
            const split = splitFrontMatter({ text: '---\nname: mittens\n\n没有关闭的栅栏\n', },);
            expect(split.frontMatter,).toBe(undefined,);
            expect(split.bodyOffset,).toBe(0,);
          },
        },),

        it({
          name: 'accepts fence closing at end of input',
          fn: async () => {
            /** Split of source that is front matter only. */
            const split = splitFrontMatter({ text: '---\nname: mittens\n---', },);
            expect(split.frontMatter?.data,).toEqual({ name: 'mittens', },);
            expect(split.body,).toBe('',);
          },
        },),

        it({
          name: 'throws FrontMatterParseError on invalid YAML between fences',
          fn: async () => {
            /**
             Value the split of source with malformed YAML threw.
             */
            const refusal = caught(function splitMalformed(): void {
              splitFrontMatter({ text: '---\nname: [unclosed\n---\n喵。\n', },);
            },);
            expect(refusal instanceof FrontMatterParseError,).toBe(true,);
          },
        },),
      ],
    },),

    describe({
      name: 'splitFrontMatter line endings',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'reads front matter written with CRLF, which the fixed `\\n` fences '
            + 'refused outright. One corpus original uses Windows line endings, and '
            + 'the consequence was not a missing field: the whole YAML block parsed '
            + 'as BODY, where `---` became a thematic break and `name: Nori` became a '
            + 'setext heading, so the critics received the metadata as content and '
            + 'the identity context was empty for that entry',
          fn: async () => {
            /**
             Same document in both line endings.
             */
            const lf = splitFrontMatter({ text: '---\nname: Nori\n---\n\nBody.\n', },);

            /**
             Windows-flavoured counterpart.
             */
            const crlf = splitFrontMatter({
              text: '---\r\nname: Nori\r\n---\r\n\r\nBody.\r\n',
            },);

            expect(lf.frontMatter?.data,).toEqual({ name: 'Nori', },);
            expect(crlf.frontMatter?.data,).toEqual({ name: 'Nori', },);
          },
        },),

        it({
          name: 'leaves the body starting after the closing fence in both, so the '
            + 'body offset every later anchor is measured from stays exact',
          fn: async () => {
            /**
             CRLF document whose body is one paragraph.
             */
            const split = splitFrontMatter({
              text: '---\r\nname: Nori\r\n---\r\n\r\nBody.\r\n',
            },);

            expect(split.body.trim(),).toBe('Body.',);
            expect(split.bodyOffset,).toBe(
              '---\r\nname: Nori\r\n---\r\n'.length,
            );
          },
        },),

        it({
          name: 'reads EMPTY front matter closing at end of input under CRLF, the '
            + 'one shape the first CRLF fix still refused. Its guard bounded the '
            + 'closing fence by the opening fence LENGTH minus a single character, '
            + 'which assumes a one-character terminator, so under CRLF the bound '
            + 'sat one past the line break and the document reported no front '
            + 'matter at all. Every other CRLF shape parsed, which is exactly what '
            + 'kept it hidden',
          fn: async () => {
            /**
             Both spellings of a document that is nothing but empty front matter.
             */
            const lf = splitFrontMatter({ text: '---\n---', },);

            /**
             Windows-flavoured counterpart, which used to report no front matter.
             */
            const crlf = splitFrontMatter({ text: '---\r\n---', },);

            expect(lf.frontMatter?.data,).toBe(null,);
            expect(crlf.frontMatter?.data,).toBe(null,);
            expect(crlf.body,).toBe('',);
            expect(crlf.bodyOffset,).toBe('---\r\n---'.length,);
          },
        },),

        it({
          name: 'keeps every other CRLF closing shape reading as it did, since the '
            + 'guard being relaxed is what stops a closing fence being found before '
            + 'the opening one has ended',
          fn: async () => {
            expect(
              splitFrontMatter({ text: '---\r\nname: Nori\r\n---', },).frontMatter?.data,
            ).toEqual({ name: 'Nori', },);
            expect(
              splitFrontMatter({ text: '---\r\n---\r\nBody\r\n', },).frontMatter?.data,
            ).toBe(null,);
            expect(
              splitFrontMatter({ text: '---\r\nname: Nori\r\n', },).frontMatter,
            ).toBe(undefined,);
          },
        },),
      ],
    },),

    describe({
      name: 'splitFrontMatter leading byte order mark',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS THE FRONT MATTER OF A PAGE OPENING WITH A MARK, the fence pair its own raw '
            + 'slice, the mark left outside it and the offsets those of the text as written',
          fn: async () => {
            /**
             Page whose first character is the mark, then its fence pair and a body.
             */
            const text = '\uFEFF---\nname: Mittens\n---\n\nThe cat naps.\n';

            expect(splitFrontMatter({ text, },),).toEqual({
              frontMatter: {
                raw: '---\nname: Mittens\n---\n',
                data: { name: 'Mittens', },
                startOffset: 1,
              },
              body: '\nThe cat naps.\n',
              bodyOffset: 23,
            },);
          },
        },),

        it({
          name: 'READS THE FRONT MATTER OF A PAGE OPENING WITH A MARK under CRLF, and one closing at the end of '
            + 'its input',
          fn: async () => {
            /**
             Windows-flavoured page opening with the mark.
             */
            const crlf = splitFrontMatter({ text: '\uFEFF---\r\nname: Nori\r\n---\r\n\r\nBody.\r\n', },);

            /**
             Page that is the mark and a fence pair, closing at its end.
             */
            const closing = splitFrontMatter({ text: '\uFEFF---\nname: Nori\n---', },);

            expect(crlf.frontMatter,).toEqual({
              raw: '---\r\nname: Nori\r\n---\r\n',
              data: { name: 'Nori', },
              startOffset: 1,
            },);
            expect(crlf.bodyOffset,).toBe(23,);
            expect(closing.frontMatter,).toEqual({
              raw: '---\nname: Nori\n---',
              data: { name: 'Nori', },
              startOffset: 1,
            },);
            expect(closing.body,).toBe('',);
            expect(closing.bodyOffset,).toBe(19,);
          },
        },),

        it({
          name: 'STARTS FRONT MATTER AT ZERO WHERE THE PAGE OPENS WITH THE FENCE, so a page without a mark reads as it did',
          fn: async () => {
            /**
             Page opening with its fence pair and no mark.
             */
            const split = splitFrontMatter({ text: '---\nname: Mittens\n---\n\nThe cat naps.\n', },);

            expect(split.frontMatter?.startOffset,).toBe(0,);
            expect(split.bodyOffset,).toBe(22,);
          },
        },),

        it({
          name: 'LEAVES A MARK THAT DOES NOT OPEN THE PAGE AS CONTENT, so a second mark, a mark after a line '
            + 'break and a mark on the fence line each leave the page without front matter',
          fn: async () => {
            for (
              const text of [
                '\uFEFF\uFEFF---\nname: Mittens\n---\n',
                '\n\uFEFF---\nname: Mittens\n---\n',
                '---\uFEFF\nname: Mittens\n---\n',
              ]
            ) {
              expect(splitFrontMatter({ text, },),).toEqual({
                body: text,
                bodyOffset: 0,
              },);
            }
          },
        },),

        it({
          name: 'LEAVES A PAGE OPENING WITH A MARK AND AN UNTERMINATED FENCE WITHOUT FRONT MATTER, the whole text its body',
          fn: async () => {
            /**
             Page with the mark, an opening fence and no closing one.
             */
            const text = '\uFEFF---\nname: Mittens\n\nThe cat naps.\n';

            expect(splitFrontMatter({ text, },),).toEqual({
              body: text,
              bodyOffset: 0,
            },);
          },
        },),

        it({
          name: 'REPRODUCES THE TEXT FROM THE MARK, THE RAW SLICE AND THE BODY, so every offset a record names '
            + 'indexes the text as written',
          fn: async () => {
            /**
             Page opening with the mark.
             */
            const text = '\uFEFF---\nname: Mittens\n---\n\nThe cat naps.\n';

            /**
             Split of that page.
             */
            const split = splitFrontMatter({ text, },);
            const { frontMatter, } = split;
            if (frontMatter === undefined)
              throw new Error('the page opens with a fence pair, so it has front matter',);

            expect(`${text.slice(0, frontMatter.startOffset,)}${frontMatter.raw}${split.body}`,).toBe(text,);
            expect(text.slice(frontMatter.startOffset, split.bodyOffset,),).toBe(frontMatter.raw,);
          },
        },),
      ],
    },),

    describe({
      name: 'FrontMatterParseError says where, never what',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'CONTROL: the YAML parser itself does quote the line, so absence is provable',
          fn: async () => {
            /**
             What the parser says when nothing stands between it and a reader.
             */
            const raw = rawYamlRefusal({
              yamlSource: `name: ${REFUSAL_FIXTURE_WORD}\n  bad: [x\n`,
            },);

            expect(raw.includes(REFUSAL_FIXTURE_WORD,),).toBe(true,);
          },
        },),
        it({
          name: 'REFUSES to repeat the front matter it could not parse',
          fn: async () => {
            /**
             Refusal as a reader would see it.
             */
            const refusal = refusalFrom({ text: REFUSING_SOURCE, },);

            expect(refusal.message.includes(REFUSAL_FIXTURE_WORD,),).toBe(false,);
          },
        },),
        it({
          name: 'STATES the position and the code the parser assigned',
          fn: async () => {
            /**
             Refusal as a reader would see it.
             */
            const refusal = refusalFrom({ text: REFUSING_SOURCE, },);

            expect(refusal.message.includes('at line 1 column 7',),).toBe(true,);
            expect(refusal.message.includes('BLOCK_AS_IMPLICIT_KEY',),).toBe(true,);
          },
        },),
        it({
          name: 'CARRIES NO cause, which a reporter would render whether asked to or not',
          fn: async () => {
            /**
             Refusal as a reader would see it.
             */
            const refusal = refusalFrom({ text: REFUSING_SOURCE, },);

            expect(refusal.cause,).toBe(undefined,);
          },
        },),
        it({
          name: 'DECLARES its message safe to forward',
          fn: async () => {
            /**
             Refusal as a reader would see it.
             */
            const refusal = refusalFrom({ text: REFUSING_SOURCE, },);

            expect(namesWithoutQuoting(refusal,),).toBe(true,);
          },
        },),
        it({
          name: 'NAMES THE JAVASCRIPT ERROR CLASS WHEN AN ALIAS POINTS AT NO ANCHOR, since the parser raises '
            + 'a ReferenceError the YAML grammar never assigns a position or a fault code to',
          fn: async () => {
            /**
             Refusal for a front matter whose alias names no anchor the document
             ever sets.
             */
            const refusal = refusalFrom({ text: '---\nname: *ghost\n---\n\nBody.\n', },);

            expect(refusal instanceof FrontMatterParseError,).toBe(true,);
            expect(refusal.message.includes('at an unstated position (ReferenceError)',),).toBe(true,);
            expect(refusal.message.includes('ghost',),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: requireFrontMatterRefusal.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'RETURNS the refusal a catch around the splitter holds, the same object',
          fn: async () => {
            /**
             Refusal the splitter raised.
             */
            const refusal = refusalFrom({ text: REFUSING_SOURCE, },);

            expect(refusal instanceof FrontMatterParseError,).toBe(true,);
            expect(requireFrontMatterRefusal({ error: refusal, },),).toBe(refusal,);
          },
        },),
        it({
          name: 'RETHROWS anything else unchanged, an error or not, since an unexpected state must keep propagating',
          fn: async () => {
            /**
             A failure that is not the splitter's refusal.
             */
            const stray = new TypeError('the cat sat on the YAML',);

            expect(caught(function narrowStray(): void {
              requireFrontMatterRefusal({ error: stray, },);
            },),).toBe(stray,);
            expect(caught(function narrowString(): void {
              requireFrontMatterRefusal({ error: 'hairball', },);
            },),).toBe('hairball',);
          },
        },),
      ],
    },),
  ],
},);

//endregion Front matter refusal disclosure
