/**
 Tests for the text of a slice written as literal text for the strict MDX grammar.

 The page carries the original of an untranslated passage as the artifact records it, and writes it through this only
 where the page with it as recorded does not parse. What must change is a brace or a less-than sign that the strict
 grammar reads as syntax; what must not is everything else: Chinese prose, footnote references and definitions,
 links, headings, block quotes, code in any container, math, comments and an escape the original already wrote.

 EVERY CASE OF THE SECOND GROUP COMPARES WHAT A READER SEES: the text of the page's leaves after the note, read by
 the strict grammar, against the text of the source's leaves read as plain Markdown
 (`source-reader-text.test-fixture.ts`). A pinned string says what the encoder wrote; only the comparison says a
 reader sees the source text.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import { parseDocument, } from '@monochromatic-dev/translation-repair-shared-contract';

import {
  encodeAsLiteralMdx,
  readPageGrammar,
} from '../../dist/final/node/index.mjs';
import {
  readerOfPage,
  readerOfSource,
} from './source-reader-text.test-fixture.ts';

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

/**
 Texts a reader must see as the source text once encoded, with what each one stands for in the case name.
 */
const READER_CASES: readonly (readonly [string, string])[] = [
  ['a bare opening brace', '她说 { 然后走了。\n'],
  ['an unclosed brace opening the text', '{ 她说\n'],
  ['an unclosed details tag', '<details>\n\n她说\n'],
  ['a lone closing details tag', '她说\n\n</details>\n'],
  ['an indented line holding an expression and a tag', '她说\n\n  {x} <y>\n'],
  ['an escape the original wrote', `${String.raw`\{ 与 \< 与 \\{ 与 }`}\n`],
  ['a trailing backslash', '行尾的反斜杠\\\n'],
  ['an HTML entity beside a brace', '&lt;猫&gt; 与 { 与 &#123;\n'],
  ['a footnote reference and its definition', '她[^1]说 {\n\n[^1]: 注 {x}，<y>。\n'],
  ['a heading ending in an id', '## 标题 {#id}\n'],
  ['a table cell', '| 甲 | 乙 {x} |\n| --- | --- |\n| 丙 | <丁> |\n'],
  ['a fence in a list item', '- ```\n  {\n  ```\n\n然后 { 走了\n'],
  ['a fence in an ordered list item and a lone closing tag', '1. ```js\n   const a = {};\n   ```\n\n</details>\n'],
  ['a fence in a block quote', '> ```\n> {a}\n> ```\n\n{b}\n'],
  ['a code span across lines', '`a\n{b}` 与 {c}\n'],
  ['inline math holding a less-than sign and a brace', '猫 $a < b {c}$ 走 {\n'],
  ['block math holding a brace', '$$\nx < {y}\n$$\n\n{z}\n'],
  ['an angle autolink', '见 <https://example.net/a> 与 {\n'],
  ['an angle autolink holding characters Markdown reads', '见 <https://example.net/a_b*c(d){e}[f]> 与 {\n'],
  ['an email autolink', '写信到 <tabby@example.net> 与 {\n'],
  ['an angle destination with a space', '[链接](<b c> "标题 {d}") 与 {\n'],
  ['a link whose destination and title hold braces', '[链接 {a}](https://example.net/{b} "标题 {c}") 与 {\n'],
  ['an image whose alternative text holds a brace', '![图 {a}](u.png) 与 {\n'],
  ['a bare address holding a brace', '见 https://example.net/b{c} 与 {d}\n'],
  ['indented code, which MDX does not have', '猫\n\n    {x}\n'],
  ['a component the original wrote', '她说 <Foo bar={1} /> 与 {他说}\n'],
  ['an ES module line that is not valid JavaScript', 'export to PDF 的步骤如下。\n'],
  ['an ES module line in a sentence form', 'import data from the old system\n'],
  ['an ES module line after a blank line', '她说\n\nimport data from the old system\n\n{\n'],
  ['an ES module word later in a paragraph', '她说\nimport data from the old system {\n'],
  ['a comment that closes beside a brace', '<!-- 注释 {a} -->\n\n她说 {b}\n'],
  ['a comment that never closes', '<!-- 注释\n她说\n'],
  ['a footnote definition that is only a brace', '她[^1]说\n\n[^1]: {\n'],
  ['a footnote definition that is only a tag', '她[^1]说\n\n[^1]: <b>\n'],
  ['a footnote definition that is only an address in angle brackets', '她[^1]说\n\n[^1]: <https://example.net/x>\n'],
  ['a footnote definition of one word holding a brace', '她[^1]说\n\n[^1]: a{b\n'],
  ['a footnote definition of one word holding a less-than sign', '她[^1]说\n\n[^1]: a<b\n'],
  ['a footnote definition with a title holding a brace', '她[^1]说\n\n[^1]: 注 ({)\n'],
  ['a footnote definition with a fenced block in its body', '她[^1]说\n\n[^1]: x\n\n    ```\n    {\n    ```\n'],
  [
    'a footnote definition with a paragraph and a fenced block in its body',
    '她[^1]说\n\n[^1]: x\n\n    注 {\n\n    ```\n    {y}\n    ```\n',
  ],
  ['a setext heading starting with import', 'import data from the old system\n===\n'],
  ['a setext heading starting with export', 'export to PDF\n---\n'],
  ['a setext heading of two lines starting with import', 'import x from y\nabc\n===\n'],
  ['a bare address ended by a tag', 'https://example.net/a<b> {\n'],
  ['a bare address ended by a tag after text', '见 https://example.net/a_b<c> 与 {d}\n'],
  ['a bare www address ended by a tag', 'www.example.net/a<b> {\n'],
  ['a bare address inside link text', '[https://example.net/a{ b](u)\n'],
  ['a protocol with a brace right after it', '见 https://{x\n'],
  ['a protocol with nothing after it', '见 https:// {\n'],
  ['a reference label holding a brace, collapsed', '[a {b][]\n\n[a {b]: u\n'],
  ['a reference label holding a brace, full', '[x][a {b]\n\n[a {b]: u\n'],
  ['display math on one line', '$$a < {b}$$ 与 {\n'],
  ['display math that never closes', '$$\n{x\n'],
  ['inline math with an escaped less-than sign', '$a \\< b$ {\n'],
  [
    'a byte order mark line before a setext heading starting with import',
    'x\n\n\uFEFF\nimport data from the old system\n===\n',
  ],
  ['a byte order mark line before an import line in a paragraph', 'x\n\n\uFEFF\nimport data\n'],
  ['a no-break space line before an export line', 'x\n\n\u00A0\nexport to PDF\n---\n'],
  ['a zero-width space line before an import line', 'x\n\n\u200B\nimport data\n'],
  ['two byte order mark lines before an import line', '\uFEFF\n\uFEFF\nimport data\n'],
  ['a definition line right after a footnote definition', '她[^1]说\n\n[^1]: 注\n[b]: u{ "t"\n'],
  ['two definition lines right after a footnote definition', '她[^1]说\n\n[^1]: 注\n[b]: u{\n[c]: v{ "t"\n'],
  [
    'a definition line holding an address right after a footnote definition',
    '她[^1]说\n\n[^1]: 注\n[b]: https://example.net/a{b\n',
  ],
  ['a definition line after a footnote definition and a blank line', '她[^1]说\n\n[^1]: 注\n\n[b]: u{ "t"\n'],
];

await describe({
  name: encodeAsLiteralMdx.name,
  children: [
    describe({
      name: 'what the encoder writes',
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
          name: 'LEAVES THE GREATER-THAN SIGN ALONE, since it is syntax only inside a tag and a tag cannot open once '
            + 'its less-than sign is escaped; escaping it would turn a block quote of the original into a line '
            + 'starting with the sign',
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
          name: 'COPIES CODE AS IT IS, a fenced block and a span closed on its own line, since the grammar reads no '
            + 'syntax there and a backslash written into code would show',
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
          name: 'COPIES CODE THE LINE LEXER MISREAD: a fence inside a list item and inside a block quote, a span '
            + 'across lines, and the closing fence of a list item, which is not an opener',
          fn: async () => {
            expect(encoded({ text: '- ```\n  {\n  ```\n\n然后 { 走了', },),).toBe('- ```\n  {\n  ```\n\n然后 \\{ 走了',);
            expect(encoded({ text: '1. ```js\n   const a = {};\n   ```\n\n</details>', },),)
              .toBe('1. ```js\n   const a = {};\n   ```\n\n\\</details>',);
            expect(encoded({ text: '> ```\n> {a}\n> ```\n\n{b}', },),).toBe('> ```\n> {a}\n> ```\n\n\\{b\\}',);
            expect(encoded({ text: '`a\n{b}` 与 {c}', },),).toBe('`a\n{b}` 与 \\{c\\}',);
          },
        },),

        it({
          name: 'COPIES MATH AS IT IS, inline and as a block, a less-than sign and a brace among it, since the site '
            + 'reads TeX there and TeX writes a literal brace itself',
          fn: async () => {
            expect(encoded({ text: '猫 $a < b {c}$ 与 <d>\n\n$$\nx < {y}\n$$\n\n{e}\n', },),)
              .toBe('猫 $a < b {c}$ 与 \\<d>\n\n$$\nx < {y}\n$$\n\n\\{e\\}\n',);
          },
        },),

        it({
          name: 'ENCODES AFTER A SPAN THAT NEVER CLOSES, which opens no code, and reads a fence that never closes as '
            + 'running to the end of the text',
          fn: async () => {
            expect(encoded({ text: '`{a} 与 {b}\n\n~~~\n{c}', },),).toBe('`\\{a\\} 与 \\{b\\}\n\n~~~\n{c}',);
          },
        },),

        it({
          name: 'READS INDENTED CODE AS TEXT, since MDX has no indented code and reads such a line as a paragraph',
          fn: async () => {
            expect(encoded({ text: '猫\n\n    {x} <y>\n', },),).toBe('猫\n\n    \\{x\\} \\<y>\n',);
          },
        },),

        it({
          name: 'ENCODES THE ALTERNATIVE TEXT OF AN IMAGE and leaves the destination and the title of any link alone, '
            + 'since only the first is read as text',
          fn: async () => {
            expect(encoded({ text: '![图 {a}](u.png)', },),).toBe(String.raw`![图 \{a\}](u.png)`,);
            expect(encoded({ text: '[链接 {a}](https://example.net/{b} "标题 {c}")', },),)
              .toBe(String.raw`[链接 \{a\}](https://example.net/{b} "标题 {c}")`,);
            expect(encoded({ text: '[链接](<b c>) {d}', },),).toBe(String.raw`[链接](<b c>) \{d\}`,);
          },
        },),

        it({
          name: 'LEAVES A BARE ADDRESS ALONE, braces among it, since the strict grammar reads the address to the next '
            + 'space as one link and a backslash written into it would show',
          fn: async () => {
            expect(encoded({ text: '见 https://example.net/b{c} 与 {d}', },),)
              .toBe(String.raw`见 https://example.net/b{c} 与 \{d\}`,);
          },
        },),

        it({
          name: 'REWRITES AN ANGLE AUTOLINK AS A LINK WITH THE SAME TEXT AND TARGET, characters Markdown reads written '
            + 'with a backslash in the text and as a percent escape in the target',
          fn: async () => {
            expect(encoded({ text: '<https://example.net/a> {b}', },),)
              .toBe(String.raw`[https://example.net/a](https://example.net/a) \{b\}`,);
            expect(encoded({ text: '<tabby@example.net>', },),).toBe('[tabby@example.net](mailto:tabby@example.net)',);
            expect(encoded({ text: '<https://example.net/a_b*c(d){e}[f]>', },),).toBe(
              String.raw`[https://example.net/a\_b\*c(d)\{e\}\[f\]](https://example.net/a_b*c%28d%29%7Be%7D[f])`,
            );
          },
        },),

        it({
          name: 'COPIES AN HTML COMMENT AS IT IS, across lines too, since it is masked before the grammar reads the '
            + 'page and an escaped opening would turn the comment into visible text',
          fn: async () => {
            expect(encoded({ text: '<!-- <p> {a} --> {b} <!-- <q>\n{c}\n--> {d}\n', },),)
              .toBe('<!-- <p> {a} --> \\{b\\} <!-- <q>\n{c}\n--> \\{d\\}\n',);
          },
        },),

        it({
          name: 'WRITES THE OPENER OF A COMMENT THAT NEVER CLOSES AS A CHARACTER REFERENCE, since the package masks '
            + 'from any less-than sign and exclamation mark and two dashes to the end of the page, a backslash '
            + 'before it included, and the text after it is read as text',
          fn: async () => {
            expect(encoded({ text: '<!-- 注释\n她说 {a}\n', },),).toBe('&lt;!-- 注释\n她说 \\{a\\}\n',);
            expect(encoded({ text: String.raw`\<!-- 注释`, },),).toBe('&lt;!-- 注释',);
          },
        },),

        it({
          name: 'WRITES THE FIRST LETTER OF AN ES MODULE LINE AS A CHARACTER REFERENCE, a line starting `import ` or '
            + '`export ` at the start of a paragraph, and no other line',
          fn: async () => {
            expect(encoded({ text: 'export to PDF 的步骤如下。', },),).toBe('&#101;xport to PDF 的步骤如下。',);
            expect(encoded({ text: '她说\n\nimport data from the old system\n', },),)
              .toBe('她说\n\n&#105;mport data from the old system\n',);
            // Not at the start of a paragraph, not followed by a space, not in lower case, indented, or in a container.
            expect(encoded({ text: '她说\nimport data\n', },),).toBe('她说\nimport data\n',);
            expect(encoded({ text: 'imported data', },),).toBe('imported data',);
            expect(encoded({ text: 'Import data', },),).toBe('Import data',);
            expect(encoded({ text: ' import data', },),).toBe(' import data',);
            expect(encoded({ text: '> import data', },),).toBe('> import data',);
          },
        },),

        it({
          name: 'WRITES THE FIRST LETTER OF A SETEXT HEADING THAT STARTS WITH AN ES MODULE WORD AS A CHARACTER '
            + 'REFERENCE too, since MDX reads the heading line as the module before it reads the underline',
          fn: async () => {
            expect(encoded({ text: 'import data from the old system\n===\n', },),)
              .toBe('&#105;mport data from the old system\n===\n',);
            expect(encoded({ text: 'export to PDF\n---\n', },),).toBe('&#101;xport to PDF\n---\n',);
            expect(encoded({ text: 'import x from y\nabc\n===\n', },),).toBe('&#105;mport x from y\nabc\n===\n',);
            // An ATX heading starts with its marker, which no module word follows.
            expect(encoded({ text: '# import x\n', },),).toBe('# import x\n',);
          },
        },),

        it({
          name: 'WRITES THE FIRST LETTER OF AN ES MODULE LINE THAT FOLLOWS A LINE OF INVISIBLE CHARACTERS AS A '
            + 'CHARACTER REFERENCE too, since the strict grammar starts the module there and the plain reading '
            + 'continues the paragraph',
          fn: async () => {
            expect(encoded({ text: 'x\n\n\uFEFF\nimport data\n', },),).toBe('x\n\n\uFEFF\n&#105;mport data\n',);
            expect(encoded({ text: 'x\n\n\u00A0\nexport to PDF\n---\n', },),)
              .toBe('x\n\n\u00A0\n&#101;xport to PDF\n---\n',);
            // A line of visible characters before it leaves it a line of the paragraph.
            expect(encoded({ text: 'x\n\n\uFEFFy\nimport data\n', },),).toBe('x\n\n\uFEFFy\nimport data\n',);
          },
        },),

        it({
          name: 'ENCODES THE BODY OF A FOOTNOTE DEFINITION CommonMark READS AS A LINK REFERENCE DEFINITION, since the '
            + 'strict grammar reads the line as the footnote it is and its body as text',
          fn: async () => {
            expect(encoded({ text: '[^1]: {', },),).toBe(String.raw`[^1]: \{`,);
            expect(encoded({ text: '[^1]: <b>', },),).toBe(String.raw`[^1]: \<b>`,);
            expect(encoded({ text: '[^1]: a{b', },),).toBe(String.raw`[^1]: a\{b`,);
            expect(encoded({ text: '[^1]: a<b', },),).toBe(String.raw`[^1]: a\<b`,);
            expect(encoded({ text: '[^1]: 注 ({)', },),).toBe(String.raw`[^1]: 注 (\{)`,);
            expect(encoded({ text: '[^1]: <https://example.net/x>', },),)
              .toBe('[^1]: [https://example.net/x](https://example.net/x)',);
            // A link reference definition that is no footnote keeps its destination and its title.
            expect(encoded({ text: '[r]: u{b "t {"\n', },),).toBe('[r]: u{b "t {"\n',);
          },
        },),

        it({
          name: 'READS A DEFINITION LINE RIGHT AFTER A FOOTNOTE DEFINITION AS A LINE OF THE FOOTNOTE, since the strict '
            + 'grammar continues the paragraph of the footnote with it, where CommonMark reads a second definition',
          fn: async () => {
            expect(encoded({ text: '[^1]: 注\n[b]: u{ "t"\n', },),).toBe(String.raw`[^1]: 注
[b]: u\{ "t"
`,);
            expect(encoded({ text: '[^1]: 注\n[b]: u{\n[c]: v{\n', },),).toBe(String.raw`[^1]: 注
[b]: u\{
[c]: v\{
`,);
            // A blank line ends the footnote, so the definition after it is a definition.
            expect(encoded({ text: '[^1]: 注\n\n[b]: u{ "t"\n', },),).toBe('[^1]: 注\n\n[b]: u{ "t"\n',);
          },
        },),

        it({
          name: 'READS THE BODY OF A FOOTNOTE DEFINITION BY ITS OWN BLOCKS: a paragraph indented under it is text, a '
            + 'fence indented under it is code, which keeps its braces without a backslash',
          fn: async () => {
            expect(encoded({ text: '[^1]: x\n\n    ```\n    {\n    ```\n', },),)
              .toBe('[^1]: x\n\n    ```\n    {\n    ```\n',);
            expect(encoded({ text: '[^1]: x\n\n    注 {\n\n    ```\n    {y}\n    ```\n', },),)
              .toBe('[^1]: x\n\n    注 \\{\n\n    ```\n    {y}\n    ```\n',);
            // Indented code that follows no footnote is still text, as it was.
            expect(encoded({ text: '猫\n\n    {x}\n', },),).toBe('猫\n\n    \\{x\\}\n',);
          },
        },),

        it({
          name: 'REWRITES A BARE ADDRESS THAT A LESS-THAN SIGN ENDS AS A LINK WITH THE SAME TEXT AND TARGET, since a '
            + 'backslash written after it is read as part of the address and an entity is read as part of it too',
          fn: async () => {
            expect(encoded({ text: 'https://example.net/a<b> {', },),)
              .toBe(String.raw`[https://example.net/a](https://example.net/a)\<b> \{`,);
            expect(encoded({ text: 'www.example.net/a<b>', },),)
              .toBe(String.raw`[www.example.net/a](http://www.example.net/a)\<b>`,);
            expect(encoded({ text: '见 https://example.net/a_b<c>', },),)
              .toBe(String.raw`见 [https://example.net/a\_b](https://example.net/a_b)\<c>`,);
          },
        },),

        it({
          name: 'READS A BARE ADDRESS AS GFM DOES: none inside link text, none with a punctuation mark right after the '
            + 'protocol, so a brace there is escaped as any other',
          fn: async () => {
            expect(encoded({ text: '[https://example.net/a{ b](u)', },),)
              .toBe(String.raw`[https://example.net/a\{ b](u)`,);
            expect(encoded({ text: '见 https://{x', },),).toBe(String.raw`见 https://\{x`,);
            expect(encoded({ text: '见 https:// {', },),).toBe(String.raw`见 https:// \{`,);
            expect(encoded({ text: '见 https://example.net/b{c}', },),).toBe('见 https://example.net/b{c}',);
          },
        },),

        it({
          name: 'ENCODES A REFERENCE LABEL THE SAME IN ITS USE AND IN ITS DEFINITION, so the reference still resolves',
          fn: async () => {
            expect(encoded({ text: '[a {b][]\n\n[a {b]: u\n', },),).toBe('[a \\{b][]\n\n[a \\{b]: u\n',);
            expect(encoded({ text: '[x][a {b]\n\n[a {b]: u\n', },),).toBe('[x][a \\{b]\n\n[a \\{b]: u\n',);
          },
        },),

        it({
          name: 'KEEPS THE LINK AROUND AN ANGLE AUTOLINK INSIDE LINK TEXT and writes the angle autolink as text, '
            + 'since a link cannot hold a link and MDX reads the autolink as a tag: the reader sees the outer link '
            + 'and the address as typed (a limit the module comment names)',
          fn: async () => {
            expect(encoded({ text: '[<https://example.net/a>](u) {', },),)
              .toBe(String.raw`[\<https://example.net/a>](u) \{`,);
            expect(readerOfPage({ carried: encoded({ text: '[<https://example.net/a>](u) {', },), },),)
              .toBe('⟦u⟧<https://example.net/a> {',);
          },
        },),

        it({
          name: 'KEEPS THE LINE ENDINGS OF A CARRIAGE-RETURN TEXT, a closing fence among them',
          fn: async () => {
            expect(encoded({ text: '```\r\n{a}\r\n```\r\n{b}\r\n', },),).toBe('```\r\n{a}\r\n```\r\n\\{b\\}\r\n',);
          },
        },),

        it({
          name: 'COUNTS POSITIONS IN UTF-16 UNITS, so a character beyond the first plane before a brace does not shift '
            + 'the escape',
          fn: async () => {
            expect(encoded({ text: '\u{1F408} `{a}` {b}', },),).toBe('\u{1F408} `{a}` \\{b\\}',);
          },
        },),

        it({
          name: 'KEEPS A LEADING BYTE ORDER MARK and encodes what follows it',
          fn: async () => {
            expect(encoded({ text: '\uFEFF她说 {', },),).toBe('\uFEFF她说 \\{',);
          },
        },),

        it({
          name: 'ESCAPES EVERY BRACE AND LESS-THAN SIGN OUTSIDE A COMMENT WHEN THE TEXT IS NESTED PAST WHAT THE READER '
            + 'TAKES, since no structure can be read from it',
          fn: async () => {
            expect(encoded({ text: `${'>'.repeat(300,)} 她说 {`, },),).toBe(`${'>'.repeat(300,)} 她说 \\{`,);
          },
        },),
      ],
    },),

    describe({
      name: 'what a reader of the page sees',
      children: [
        ...READER_CASES.map(function readerCase([name, text,],) {
          return it({
            name: `SHOWS THE SOURCE TEXT FOR ${name.toUpperCase()}: the page parses and its leaves equal the source's`,
            fn: async () => {
              /**
               The run as the encoder writes it.
               */
              const written = encoded({ text, },);
              expect(readPageGrammar({ pageText: `${BEFORE}${written}\n`, },),).toEqual({ kind: 'parses', },);
              expect(readerOfPage({ carried: written, },),).toBe(readerOfSource({ source: text, },),);
            },
          },);
        },),

        it({
          name: 'LEAVES THE ENGLISH AFTER A COMMENT THAT NEVER CLOSES TO THE PACKAGE\'S OWN PARSER: no unterminated '
            + 'comment is found in the page and the paragraph after the run is a node of it',
          fn: async () => {
            /**
             The package's reading of a page whose run opened a comment and whose English follows.
             */
            const { parseFindings, nodes, } = parseDocument({
              text: `${BEFORE}${encoded({ text: '<!-- 注释\n她说\n', },)}\nShe slept on the counter.\n`,
            },);
            expect(parseFindings.map(function kindOf({ kind, },): string {
              return kind;
            },),).toEqual([],);
            expect(nodes.at(-1,)?.text,).toBe('She slept on the counter.',);
          },
        },),

        it({
          name: 'FAILS THE SAME TEXTS AS WRITTEN, the positive control for the cases that parse',
          fn: async () => {
            expect(readPageGrammar({ pageText: `${BEFORE}她说 {\n`, },).kind,).toBe('refused',);
            expect(readPageGrammar({ pageText: `${BEFORE}<details>\n\n她说\n`, },).kind,).toBe('refused',);
            expect(readPageGrammar({ pageText: `${BEFORE}<https://example.net/a>\n`, },).kind,).toBe('refused',);
            expect(readPageGrammar({ pageText: `${BEFORE}export to PDF 的步骤如下。\n`, },).kind,).toBe('refused',);
            expect(readPageGrammar({
              pageText: `${BEFORE}- \`\`\`\n  {\n  \`\`\`\n\n然后 {\n`,
            },).kind,).toBe('refused',);
          },
        },),
      ],
    },),
  ],
},);
