/**
 Tests for the text of a slice written as literal text for the strict MDX grammar.

 The page carries the original of an untranslated passage as the artifact records it, and writes it through this only
 where the page with it as recorded does not parse. What must change is a brace or a less-than sign that is syntax;
 what must not is everything else: Chinese prose, footnote references and definitions, links, headings, block quotes,
 code, comments and an escape the original already wrote.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  encodeAsLiteralMdx,
  readPageGrammar,
} from '../../dist/final/node/index.mjs';

/**
 Page text standing before the encoded text in the parse cases, the way the note stands before an original.
 */
const BEFORE = '## Description\n\n> This passage has not been translated yet. The original text follows.\n\n';

/**
 Encodes a text.

 @param text - text to encode

 @returns Encoded text

 @example
 ```ts
 const literal = encoded({ text: '{', },);
 ```
 */
function encoded({ text, }: { readonly text: string; },): string {
  return encodeAsLiteralMdx({ text, },);
}

await describe({
  name: encodeAsLiteralMdx.name,
  children: [
    it({
      name: 'WRITES A BACKSLASH BEFORE EACH BRACE AND EACH LESS-THAN SIGN, which the grammar reads as an '
        + 'expression and a tag, and nothing else',
      fn: async () => {
        expect(encoded({ text: '她说 {x} <y> {', },),).toBe(String.raw`她说 \{x\} \<y> \{`,);
        expect(encoded({ text: '}', },),).toBe(String.raw`\}`,);
      },
    },),

    it({
      name: 'RETURNS A TEXT WITH NO SYNTAX UNCHANGED, Chinese prose, footnote references and definitions, links, '
        + 'headings and a block quote among it',
      fn: async () => {
        /**
         Text built to hold every construct that must survive.
         */
        const plain = '## 后记\n\n> 引文 [她的主页](https://example.net/a?b=1&c=2)[^1]。\n\n[^1]: 注，“引号”与 *强调*。\n';
        expect(encoded({ text: plain, },),).toBe(plain,);
      },
    },),

    it({
      name: 'LEAVES THE GREATER-THAN SIGN ALONE, since it is syntax only inside a tag and a tag cannot open once its '
        + 'less-than sign is escaped; escaping it would turn a block quote of the original into a line starting '
        + 'with the sign',
      fn: async () => {
        expect(encoded({ text: '> 引文 <b>粗</b>\n', },),).toBe('> 引文 \\<b>粗\\</b>\n',);
      },
    },),

    it({
      name: 'STEPS OVER AN ESCAPE THE ORIGINAL WROTE as the pair it is, so an escaped brace is not doubled into a '
        + 'backslash and an open brace, and an escaped backslash does not escape what follows it',
      fn: async () => {
        expect(encoded({ text: String.raw`\{转义\} \< 与 \\{x}`, },),).toBe(String.raw`\{转义\} \< 与 \\\{x\}`,);
        expect(encoded({ text: '行尾的反斜杠\\', },),).toBe('行尾的反斜杠\\',);
      },
    },),

    it({
      name: 'COPIES CODE AS IT IS, a fenced block and a span closed on its own line, since the grammar reads no syntax '
        + 'there and a backslash written into code would show',
      fn: async () => {
        /**
         Text with braces in a span, in a tilde fence, in a backtick fence closed by a longer run, and outside all
         three.
         */
        const text = '`{a}` 与 ``b ` {c}`` {d}\n\n~~~\n{e} <f>\n~~~\n\n```ts\n{g}\n`````\n\n{h}\n';
        expect(encoded({ text, },),).toBe(
          '`{a}` 与 ``b ` {c}`` \\{d\\}\n\n~~~\n{e} <f>\n~~~\n\n```ts\n{g}\n`````\n\n\\{h\\}\n',
        );
      },
    },),

    it({
      name: 'ENCODES AFTER A SPAN THAT NEVER CLOSES and after a fence line holding a backtick, which open no code, '
        + 'and reads a fence that never closes as running to the end of the text',
      fn: async () => {
        expect(encoded({ text: '`{a}\n``` `{b}\n~~~\n{c}', },),).toBe('`\\{a\\}\n``` `\\{b\\}\n~~~\n{c}',);
      },
    },),

    it({
      name: 'COPIES AN HTML COMMENT AS IT IS, across lines too, since it is masked before the grammar reads the page '
        + 'and an escaped opening would turn the comment into visible text',
      fn: async () => {
        expect(encoded({ text: '<!-- <p> {a} --> {b} <!-- <q>\n{c}\n--> {d}\n<!-- {e}', },),)
          .toBe('<!-- <p> {a} --> \\{b\\} <!-- <q>\n{c}\n--> \\{d\\}\n<!-- {e}',);
      },
    },),

    it({
      name: 'KEEPS THE LINE ENDINGS OF A CARRIAGE-RETURN TEXT, a closing fence among them',
      fn: async () => {
        expect(encoded({ text: '```\r\n{a}\r\n```\r\n{b}\r\n', },),).toBe('```\r\n{a}\r\n```\r\n\\{b\\}\r\n',);
      },
    },),

    describe({
      name: 'a page carrying the encoded text',
      children: [
        ...[
          '她说 {',
          '{ 她说',
          '<details>\n\n她说\n',
          '她说\n\n</details>',
          '她说\n\n  {x} <y>',
          '<https://example.net/a>',
          `${'<b>'.repeat(300,)}她说`,
          `${'{'.repeat(300,)}${'}'.repeat(300,)}`,
          String.raw`\{ 与 \\{ 与 }`,
        ].map(function parseCase(text,) {
          return it({
            name: `PARSES UNDER THE STRICT GRAMMAR when the original is ${JSON.stringify(text.slice(0, 40,),)}`,
            fn: async () => {
              expect(readPageGrammar({ pageText: `${BEFORE}${encoded({ text, },)}\n`, },),)
                .toEqual({ kind: 'parses', },);
            },
          },);
        },),

        it({
          name: 'FAILS THE SAME TEXTS AS WRITTEN, the positive control for the cases that parse',
          fn: async () => {
            expect(readPageGrammar({ pageText: `${BEFORE}她说 {\n`, },).kind,).toBe('refused',);
            expect(readPageGrammar({ pageText: `${BEFORE}<details>\n\n她说\n`, },).kind,).toBe('refused',);
          },
        },),
      ],
    },),
  ],
},);
