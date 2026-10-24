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
  liveConstructsOf,
  liveConstructsOfCarried,
} from './source-live-constructs.test-fixture.ts';
import {
  readerOfPage,
  readerOfSource,
  withoutLinkTargets,
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
  ['a comment that never closes and a second opener inside it', '猫 <!-- 一 <!-- 二\n'],
  ['inline math and two openers of a comment that never closes', '$a < b$ <!-- <!--\n'],
  ['an opener of a comment that never closes right before a definition line', '<!--\n[r]: u{b "t {"\n'],
  [
    'an opener of a comment that never closes after a paragraph, right before a definition line',
    'x\n\n<!--\n[r]: u{b "t {"\n',
  ],
  [
    'an opener of a comment that never closes in a list item, right before a definition line',
    '- <!--\n  [r]: u{b "t {"\n',
  ],
  [
    'an opener of a comment that never closes in a block quote, right before a definition line',
    '> <!--\n> [r]: u{b "t {"\n',
  ],
  [
    'an opener of a comment that never closes right before two definition lines',
    '<!--\n[r]: u{b\n[s]: v{c "t"\n',
  ],
  ['an empty bullet line right after a footnote definition, then an export line', '[^1]: 注 ({)\n*\nexport to PDF\n'],
  ['an empty dash line right after a footnote definition, then an export line', '[^1]: 注\n-\nexport to PDF\n'],
  ['an empty plus line right after a footnote definition, then an import line', '[^1]: 注\n+\nimport data\n'],
  [
    'an empty ordered marker line right after a footnote definition, then an export line',
    '[^1]: 注\n2.\nexport to PDF\n',
  ],
  ['an empty ordered marker line with a parenthesis, then an export line', '[^1]: 注\n3)\nexport to PDF\n'],
  ['an empty bullet line with a space after it, then an export line', '[^1]: 注\n* \nexport to PDF\n'],
  ['an empty bullet line indented one space, then an export line', '[^1]: 注\n *\nexport to PDF\n'],
  ['two empty bullet lines right after a footnote definition, then an export line', '[^1]: 注\n*\n*\nexport to PDF\n'],
  [
    'an empty bullet line after a footnote that is a paragraph of its own, then an import line',
    '[^1]: 注 *a*\n*\nimport data\n',
  ],
  [
    'an empty bullet line after a footnote definition and a block indented under it, then an export line',
    '[^1]: 注\n    more\n*\nexport to PDF\n',
  ],
  ['an empty bullet line after a footnote definition, then a setext heading', '[^1]: 注\n*\nexport to PDF\n===\n'],
  ['an empty bullet line after a footnote definition after a paragraph', 'x\n\n[^1]: 注\n*\nimport data\n'],
  ['an empty bullet line in a paragraph that is no footnote, then an export line', 'abc\n*\nexport to PDF\n'],
  ['a bullet with text right after a footnote definition, then an export line', '[^1]: 注\n* a\nexport to PDF\n'],
  [
    'an empty bullet line, a text line, then an export line, after a footnote definition',
    '[^1]: 注\n*\nfoo\nexport to PDF\n',
  ],
  ['a byte order mark before a list item', '\uFEFF- a {\n'],
  ['a byte order mark before a block quote', '\uFEFF> a {\n'],
  ['a byte order mark before a backtick fence', '\uFEFF```\n{\n```\n'],
  ['a byte order mark before a tilde fence', '\uFEFF~~~\n{\n~~~\n'],
  ['a byte order mark before a fence holding a tag', '\uFEFF```\n<b>\n```\n'],
  ['a byte order mark before display math', '\uFEFF$$\n{x\n$$\n'],
  ['a byte order mark before a list item holding a fence', '\uFEFF- ```\n  {\n  ```\n'],
  [
    'a byte order mark and a space before a list item holding a fence with an info string',
    '\uFEFF - ``` [r]: u{b "t {"\n',
  ],
  ['a byte order mark before a link reference definition', '\uFEFF[r]: u{b "t {"\n'],
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
  ['a definition line right after a block quote holding a footnote definition', '> [^1]: 注\n[b]: u{ "t"\n'],
  [
    'a definition line right after a block quote holding a footnote definition after two spaces',
    '>  [^1]: 注\n[b]: u{ "t"\n',
  ],
  ['a definition line in a block quote right after a footnote definition', '> [^1]: 注\n> [b]: u{ "t"\n'],
  ['a definition line right after a list item holding a footnote definition', '- [^1]: 注\n[b]: u{ "t"\n'],
  [
    'a definition line right after a footnote definition indented in a list item',
    '- a\n\n  [^1]: 注\n  [b]: u{ "t"\n',
  ],
  [
    'two definition lines right after a footnote definition in a block quote',
    '> [^1]: 注\n[b]: u{\n[c]: v{ "t"\n',
  ],
  ['a bare address after a bracket nothing closes', '[ https://example.net/b{c\n'],
  ['a bare address after text and a bracket nothing closes', 'x [ y https://example.net/b{c\n'],
  ['a bare address after an image opener nothing closes', 'x ![ https://example.net/b{c\n'],
  ['a bare www address after a bracket nothing closes', 'x [ www.example.net/b{c\n'],
  ['a bare address ended by a tag after a bracket nothing closes', '[a https://example.net/a<b> {\n'],
  ['a bare address after a bracket that holds a link', '[a [b](u) https://example.net/c{d\n'],
  ['a bare address after a bracket that holds a pair of brackets', '[a [b] https://example.net/c{d\n'],
  ['a bare address after a bracket that a later one closes', 'x [y https://example.net/b{c]\n'],
  ['a bare address after a pair of brackets, which is read as an address', 'x [y] https://example.net/b{c\n'],
  ['a bare address in the paragraph after a bracket nothing closes', '[ x\n\nhttps://example.net/b{c\n'],
  ['a code span whose second line starts with an unclosed brace', '`a\n{b` {c\n'],
  ['a code span whose second line starts with a less-than sign', '`a\n<b` {c\n'],
  ['a code span whose second line is indented and starts with a brace', '`a\n  {b` {c\n'],
  ['a code span whose third line starts with a brace', '`a\nb\n{c` {d\n'],
  ['inline math whose second line starts with an unclosed brace', '$a\n{b$ {c\n'],
  ['inline math whose second line starts with a less-than sign', '$a\n<b$ {c\n'],
  ['a code span in a list item whose second line starts with a brace', '- `a\n  {b` {c\n'],
  ['a code span in a block quote whose second line starts with a brace', '> `a\n> {b` {c\n'],
  ['a link title whose second line starts with a brace', '[a](u "t\n{b") {c\n'],
  ['an image title whose second line starts with a less-than sign', '![a](u "t\n<b") {c\n'],
  ['a definition title whose second line starts with a brace', '[r]: u "t\n{b"\n\n{c\n'],
  ['a code span whose second line starts with a closed brace', '`a\n{b}` {c\n'],
  ['a code span whose second line holds a brace after a word', '`a\nb {c` {d\n'],
  ['a fence holding a mark indented four spaces, and a brace after it', '```\n    ```\n{\n```\n'],
  ['a fence that never closes holding a mark indented four spaces', '```\n    ```\n{\n'],
  ['a tilde fence holding a mark indented four spaces', '~~~\n    ~~~\n{\n~~~\n'],
  ['a fence with an info string holding a mark indented four spaces', '```js\n    ```\n{\n```\n'],
  ['a fence in a block quote holding a mark indented four spaces', '> ```\n>     ```\n> {\n> ```\n'],
  ['a fence in a list item holding a mark indented four spaces', '- ```\n      ```\n  {\n  ```\n'],
  ['a fence holding a longer mark indented four spaces', '```\n     ````\n{\n```\n'],
  ['a fence holding a mark indented four spaces, then a paragraph with a brace', '```\n    ```\n```\n\n{x}\n'],
  ['a fence holding a mark of the other kind indented four spaces', '```\n    ~~~\n{\n```\n'],
  ['a fence in a nested list item, closed at the column of its opening', '- a\n  - ```\n    {\n    ```\n\n{x\n'],
  [
    'a fence in a nested list item after a blank line, closed at the column of its opening',
    '- a\n  - b\n\n    ```js\n    code {\n    ```\n\n{\n',
  ],
  [
    'a fence in a nested list item holding a mark indented four spaces more than its opening',
    '- a\n  - ```\n        ```\n    {\n    ```\n\n{x\n',
  ],
  ['a lazy import line after a line of format characters in a list item', '1. 项\n\uFEFF\nimport q\n===\n'],
  ['a lazy export line after a line of format characters in a block quote', '> a\n\uFEFF\nexport to PDF\n'],
  ['a fence mark indented four spaces after a paragraph line', 'abc\n    ```\n'],
  ['a tilde fence mark indented four spaces after a paragraph line', 'abc\n    ~~~\n'],
  ['a fence mark after a tab after a paragraph line', 'abc\n\t```\n'],
  ['a fence mark indented four spaces and an info string holding a brace', 'abc\n    ```js {\n'],
  ['a fence mark indented four spaces in a block quote', '> abc\n>     ```\n'],
  ['a fence mark indented four spaces in a list item', '- a\n      ```\n'],
  ['a fence mark indented four spaces inside a setext heading', 'abc\n    ```\n===\n'],
  ['a code span across a line that is a fence mark indented four spaces', 'a `x\n    ```\n    y` b\n'],
  ['inline math across a line that is a fence mark indented four spaces', '$a\n    ```\n    b$ c\n'],
  ['a link title across a line that is a fence mark indented four spaces', '[a](u "t\n    ```\n    t") b\n'],
  [
    'an indented block after a tag whose first line starts with a fence mark and holds a brace',
    '<details>\n\n    ```js\n    const a = {};\n    ```\n',
  ],
  [
    'an indented block after a line of English with a brace whose first line starts with a fence mark',
    '她说 {\n\n    ```js\n    const a = {};\n    ```\n',
  ],
  ['an indented block of tildes whose first line starts with a fence mark', '<b>\n\n    ~~~\n    {a}\n    ~~~\n'],
  ['a footnote definition with a tag and a fence mark indented four spaces under it', '[^1]: <b> abc\n    ```\n'],
  ['a footnote definition, an indented fence mark and a fence at the first column', '[^1]: x\n    ```\n```\n'],
  ['a footnote definition and a tilde fence mark indented four spaces under it', '[^1]: x\n    ~~~\n'],
  [
    'a footnote definition with a brace, a line of text and a fence mark indented four spaces',
    '[^1]: x {\nabc\n    ```\n',
  ],
  [
    'a footnote definition and a closed fence indented four spaces holding a brace',
    '[^1]: x\n    ```\n    {a}\n    ```\n',
  ],
  [
    'a footnote definition with a tag and an unclosed fence indented four spaces holding a brace',
    '[^1]: <i> abc\n    ```\n    {a}\n',
  ],
  [
    'a footnote definition and a closed fence indented four spaces, then text indented under it',
    '[^1]: x {\n    ```\n    a\n    ```\n    tail {z}\n',
  ],
  [
    'a footnote definition whose first line is a tag alone, a line that is not indented and a closed fence mark',
    '[^1]: <b>\nabc\n    ```\n    ```\n',
  ],
  [
    'a footnote definition whose first line is a tag alone, a line that is not indented and an unclosed fence mark',
    '[^1]: <b>\nabc\n    ```\n',
  ],
  [
    'a footnote definition whose first line is a tag alone, a line that is not indented and a fence holding a brace',
    '[^1]: <b>\nabc\n    ```\n    {a}\n    ```\n',
  ],
  [
    'a footnote definition, an unclosed fence indented four spaces and a line with a tag',
    '[^1]: x\n    ~~~\n<>\n',
  ],
  ['a footnote, a tag alone on a line, then a definition line holding a brace', '[^1]: x\n<a href="x">\n[r]: u{b}\n'],
  ['a footnote, a closing tag alone on a line, then a definition line holding a brace', '[^1]: x\n</a>\n[r]: u{b}\n'],
  [
    'a footnote of an address in angle brackets, a tag alone on a line, then a definition line holding a brace',
    '[^1]: <https://example.net/x>\n<a href="x">\n[r]: u{b}\n',
  ],
  [
    'a footnote, a tag alone on a line, then two definition lines holding braces and a title',
    '[^1]: x\n<a href="x">\n[r]: u{b "t {"\n[s]: v{c}\n',
  ],
  [
    'a footnote after a paragraph, a tag alone on a line, then a definition line holding a brace',
    'x\n[^1]: y\n<a href="x">\n[r]: u{b}\n',
  ],
  ['a footnote that is a tag alone, then a definition line holding a brace', '[^1]: <b>\n[r]: u{b}\n'],
  [
    'a definition holding a brace, then a fence mark indented four spaces and a fence',
    '[r]: u{b}\n    ```\n    {\n    ```\n',
  ],
  [
    'a definition holding a brace, then a fence mark indented five spaces and a fence',
    '[r]: u{b}\n     ```\n    {\n    ```\n',
  ],
  [
    'a definition holding a brace, then a fence mark indented by a tab and a fence',
    '[r]: u{b}\n\t```\n\t{\n\t```\n',
  ],
  [
    'a footnote that is a tag alone, a definition holding a brace, then an indented fence',
    '[^1]: <b>\n[r]: u{b}\n    ```\n    {\n    ```\n',
  ],
  [
    'a footnote, a tag alone, a definition holding a brace, then an indented fence',
    '[^1]: x\n<a href="x">\n[r]: u{b}\n    ```\n    {\n    ```\n',
  ],
  [
    'a definition holding a brace, an indented fence mark, and a brace after a fence at the first column',
    '[r]: u{b}\n    ```\n```\n    tail {z}\n',
  ],
  [
    'a link reference definition with a caret label holding a space, then an indented fence mark and a fence',
    '[^a b]: x\n    ```\n```\n    tail {z}\n',
  ],
  [
    'a link reference definition with a caret label holding a space and a brace, then a fence mark indented by a tab',
    '[^a b]: x {\n\t```\n',
  ],
  [
    'a link reference definition with a caret label holding a space, then a definition line holding a brace',
    '[^a b]: x\n[r]: u{b}\n',
  ],
  ['a code span across a line that starts a footnote definition, in a block quote', '> a `x\n[^1]: <b>` y\n'],
  ['a code span across a footnote definition line holding a brace, in a block quote', '> a `x\n[^1]: {` y\n'],
  ['a code span across a line that starts a footnote definition, in a list item', '- a `x\n[^1]: <b>` y\n'],
  [
    'a code span that opens with three backticks across a line that starts a footnote definition, in a block quote',
    '> a ```\n[^1]: <b>\n\t```\n',
  ],
  ['a code span across a line that starts a footnote definition, at the top', 'a `x\n[^1]: <b>` y\n'],

  [
    'a footnote that is a tag alone, then a definition line holding a brace and a title with a brace',
    '[^1]: <b>\n[r]: u{b "t {"\n',
  ],
  [
    'a footnote that is a tag alone, then two definition lines holding braces',
    '[^1]: <b>\n[r]: u{b}\n[s]: v{c}\n',
  ],
  ['a footnote after a paragraph that is a tag alone, then a definition line', 'x\n\n[^1]: <b>\n[r]: u{b}\n'],
  ['a footnote of indented code, then a definition line holding a brace', '[^1]: x\n\n        c\n[r]: u{b}\n'],
  [
    'an angle autolink holding a pipe in the text of a link in a table cell',
    '| a | b |\n| - | - |\n| [<https://example.net/a|b>](u) | c |\n',
  ],
  [
    'an angle autolink holding a pipe in a table cell',
    '| a | b |\n| - | - |\n| <https://example.net/a|b> | c |\n',
  ],
  [
    'an angle autolink holding a pipe in a table cell in a block quote',
    '> | a | b |\n> | - | - |\n> | <https://example.net/a|b> | c |\n',
  ],
  [
    'an angle autolink without a pipe in a table cell',
    '| a | b |\n| - | - |\n| <https://example.net/a> | c |\n',
  ],
  [
    'a code span across a line of invisible characters, then a line that starts with import',
    '`a\n\u200B\nimport x` b\n',
  ],
  [
    'inline math across a line of invisible characters, then a line that starts with export',
    '$a\n\u200B\nexport x$ b\n',
  ],
  [
    'a code span across a line of a no-break space, then a line that starts with export',
    '`a\n\u00A0\nexport to PDF` b\n',
  ],
  [
    'a code span across a line of a byte order mark, then two lines that start with import',
    '`a\n\uFEFF\nimport x\nimport y` b\n',
  ],
  [
    'a footnote definition, lines of the paragraph and a line of equals signs, then a definition line holding a brace',
    '[^1]: 注 ({)\nimport data from the old system\n===\n[r]: u{b "t {"\n',
  ],
  [
    'a footnote definition the plain reading makes a heading of, then a definition line holding a brace',
    '[^1]: x {\ny\n===\n[r]: u{b "t {"\n',
  ],
  [
    'a footnote definition, a line of two hyphens, then a definition line holding a brace',
    '[^1]: x\ny\n--\n[r]: u{b "t {"\n',
  ],
  [
    'a footnote definition, a line of equals signs indented four spaces, then a definition line holding a brace',
    '[^1]: x\n    ===\n[r]: u{b "t {"\n',
  ],
  [
    'a code span across a footnote definition that starts on a later line of the paragraph',
    'a `b\n[^1]: c {d}\ne` f\n',
  ],
  [
    'inline math across a footnote definition that starts on a later line of the paragraph',
    'a $b\n[^1]: c {d}\ne$ f\n',
  ],
  [
    'a link across a footnote definition that starts on a later line of the paragraph',
    'a [b\n[^1]: c {d}\ne](u) f\n',
  ],
  [
    'an indented fence mark in a paragraph, then a footnote definition holding a brace',
    'abc\n    ```\n[^1]: x {\n    ```\n',
  ],
  [
    'an indented fence mark, a footnote definition, a definition line holding a brace and a fence',
    'abc\n    ```\n[^1]: 注\n[b]: u{ "t"\n    ```\n    {\n    ```\n',
  ],
  [
    'a footnote definition that starts on a later line of a paragraph, an indented fence mark under it',
    'x\n[^1]: 注 {\n    ```\n',
  ],
  [
    'two footnote definitions in one paragraph of the plain reading, a code span across them',
    'a `b\n[^1]: c {d}\n[^2]: e` {f}\n',
  ],
];

/**
 Texts the strict grammar reads as plain paragraphs and a reader of plain Markdown sees as raw HTML or code, whose
 characters the page must show as they stand: the page parses and its text, links aside, equals the source's.
 */
const LITERAL_CASES: readonly (readonly [string, string])[] = [
  [
    'a raw HTML block holding a link whose text is an address with a brace',
    '<div>\n[https://example.net/a{ b](u)\n</div>\n',
  ],
  ['a raw HTML block on one line holding that link', '<div>[https://example.net/a{ b](u)\n'],
  ['indented code holding that link', '猫\n\n    [https://example.net/a{ b](u)\n'],
  ['indented code holding emphasis and a bare address with a brace', '猫\n\n    *粗* https://example.net/a{b\n'],
  ['a raw HTML block in a block quote holding that link', '> <div>\n> [https://example.net/a{ b](u)\n'],
  ['indented code in a list item holding that link', '- 猫\n\n      [https://example.net/a{ b](u)\n'],
  ['a raw HTML block holding backslash escapes', '<div>\\{ \\\\ {x}\n'],
  ['a raw HTML block holding entities', '<p>&lt; &amp; &#123; {x}</p>\n'],
  ['indented code holding block marks', '猫\n\n    - a {\n    # b {\n    > c {\n    1. d {\n'],
  ['indented code holding a setext underline and a table', '猫\n\n    a {\n    ===\n    | b | c |\n    | - | - |\n'],
  ['a raw HTML block holding emphasis and a code span', '<div>\n*a* _b_ `{c}`\n</div>\n'],
  [
    'a raw HTML block holding a link whose text is an address with a brace, after a bracket nothing closes',
    '[ https://example.net/a{b\n<div>\n[https://example.net/a{ b](u)\n</div>\n',
  ],
  [
    'a raw HTML block holding a bare address with a brace, after a bracket nothing closes',
    '[ x\n<div>\nhttps://example.net/a{b\n</div>\n',
  ],
  [
    'a raw HTML block holding a bare address with a brace, after a blank line',
    '[ x\n\n<div>\nhttps://example.net/a{b\n</div>\n',
  ],
  ['a raw HTML block holding an import line after a blank line', '<script>\n\nimport data from the old system\n===\n'],
  ['a raw HTML block holding an export line after a blank line', '<script>\n\nexport to PDF\n'],
  ['a raw HTML block holding an import line after a line of format characters', '<details>\n\uFEFF\nimport q\n===\n'],
  ['a raw HTML block holding an import line after a blank line, then more text', '<script>\n\nimport a\n\nb {\n'],
  ['an angle autolink alone in the text of a link', '[<https://example.net/a>](u)\n'],
  [
    'a footnote definition, an unclosed fence indented by a tab and a line with a link',
    '[^1]: {\n\t```\n[<https://example.net/a>](u)\n',
  ],
  ['an angle autolink among words in the text of a link, then a brace', '她[说 <https://example.net/a> 好](u) {\n'],
  ['an angle autolink holding two dollar signs in the text of a link', '[<https://example.net/a$b$c>](u)\n'],
  ['an angle autolink holding two dollar signs at the top', '<https://example.net/a$b$c>\n'],
  [
    'an angle autolink holding a dollar sign in the text of a link, then a dollar sign later in the paragraph',
    '[<https://example.net/a$b>](u) cost $5 total\n',
  ],
  ['an angle autolink in the text of a link with a title', '[<https://example.net/a>](u "t")\n'],
  ['an angle autolink in the text of a reference link', '[<https://example.net/a>][r]\n\n[r]: u\n'],
  ['an email autolink in the text of a link', '[<tabby@example.net>](u)\n'],
  [
    'an angle autolink holding characters Markdown reads, in the text of a link',
    '[<https://example.net/a_b*c(d){e}[f]>](u)\n',
  ],
  ['an angle autolink holding an entity, in the text of a link', '[<https://example.net/a&amp;b>](u)\n'],
  ['an angle autolink in the text of a link in a block quote', '> [<https://example.net/a>](u) {\n'],
  [
    'a footnote definition inside a paragraph, then a bullet alone and an export line',
    'x\n[^1]: 注\n*\nexport to PDF\n',
  ],
  [
    'a footnote definition inside a paragraph, then an ordered marker alone and an import line',
    'x\n[^1]: 注\n2.\nimport a\n',
  ],
  [
    'a footnote definition after a line of code, then an ordered marker and an import line',
    '`{`\n[^1]: 注\n2.\nimport a\n[\n',
  ],
  [
    'a footnote definition indented three spaces inside a paragraph, then a marker and an import line',
    'x\n   [^1]: 注\n2.\nimport a\n',
  ],
  [
    'a footnote definition inside a paragraph, a line of text, a marker and an export line',
    'x\n[^1]: 注\nfoo\n*\nexport to PDF\n',
  ],
  [
    'two footnote definitions inside a paragraph, then a dash alone and an import line',
    'x\n[^1]: 注\n[^2]: 注\n-\nimport a\n',
  ],
  [
    'a footnote definition inside a setext heading, then a marker and an export line',
    'x\n[^1]: 注\n*\nexport to PDF\n===\n',
  ],
  [
    'a line of invisible characters, then a bullet alone and a setext heading starting import',
    '\uFEFF\n*\nimport x from y\nabc\n===\n',
  ],
  ['a line of invisible characters, then a bullet alone and an export line', 'x\n\uFEFF\n*\nexport to PDF\n'],
  [
    'a block quote with an indented fence mark that opens a code span across lines, and a brace',
    '> a\n>     ```\n`x {\n    ```\n    y`\n',
  ],
  [
    'an indented fence mark that opens a code span across lines, below a paragraph line',
    'a\n    ```\n`x {\n    ```\n    y`\n',
  ],
  [
    'a block quote with an indented fence mark that opens a code span, and a link with a brace',
    '> a\n>     ```\n[https://example.net/a{ b](u)\n`x\n    ```\n    y`\n',
  ],
  [
    'a block quote with an indented fence mark that opens a code span, and an import line inside it',
    '> a\n>     ```\nimport data from the old system\n\uFEFF```\n{\n```\n',
  ],
  [
    'a block quote with an indented fence mark that opens a code span closed by a mark after a byte order mark',
    '> a\n>     ```\nbcd\n\uFEFF```\n{\n```\n',
  ],
  [
    'a footnote definition with a brace inside a paragraph, then a marker and an export line',
    'x\n[^1]: 注 {\n*\nexport to PDF\n',
  ],
  [
    'a list item, a line of invisible characters, a module line, a setext underline and a second module line',
    '1. 项\n\uFEFF\nimport q\n===\nexport to PDF\n',
  ],
  ['the same in a block quote', '> a\n\uFEFF\nimport q\n===\nexport to PDF\n'],
  ['the same in a list item of two lines', '- a\n  b\n\uFEFF\nimport q\n===\nexport to PDF\n'],
  ['the same after a no-break space line', '- a\n\u00A0\nimport q\n===\nexport to PDF\n'],
  ['the same with an import line as the second module line', '- a\n\uFEFF\nimport q\n===\nimport r\n'],
  ['the same with a heading of two lines', '- a\n\uFEFF\nimport q\nmore\n===\nexport to PDF\n'],
];

/**
 Texts whose definition line the plain reading takes for a definition and the strict grammar for the lazy line of a
 paragraph, which a raw HTML block or indented code above it is to the strict grammar: the page parses and shows the
 characters of the line, which no reader of plain Markdown sees as text. Each case has what the page shows.
 */
const SHOWN_CASES: readonly (readonly [string, string, string])[] = [
  [
    'a definition line right after a list item holding a raw HTML block',
    '* </details> [^1]: 注\n[b]: u{ "t"\n',
    '</details> [^1]: 注 [b]: u{ "t"',
  ],
  [
    'a definition line right after a block quote holding a raw HTML block',
    '> <div>\n[b]: u{ "t"\n',
    '<div> [b]: u{ "t"',
  ],
  [
    'a definition line right after indented code',
    '猫\n\n    code {\n[b]: u{ "t"\n',
    '猫 code { [b]: u{ "t"',
  ],
  [
    'a definition line right after indented code, then a definition after a blank line',
    '猫\n\n    code {\n[b]: u{ "t"\n\n[c]: v{ "s"\n',
    '猫 code { [b]: u{ "t"',
  ],
  [
    'an opener of a comment that never closes, then a blank line and a definition line holding a brace',
    '<!-- open\n\n[a]: u{b "t"\n',
    '<!-- open',
  ],
  [
    'an opener of a comment that never closes, then a blank line and a definition line without a brace',
    '<!-- open\n\n[a]: u "t"\n',
    '<!-- open',
  ],
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
              .toBe('1. ```js\n   const a = {};\n   ```\n\n\\</details\\>',);
            expect(encoded({ text: '> ```\n> {a}\n> ```\n\n{b}', },),).toBe('> ```\n> {a}\n> ```\n\n\\{b\\}',);
            // A span across lines whose later line starts with a brace is joined to one line, which the strict
            // grammar would otherwise cut at.
            expect(encoded({ text: '`a\n{b}` 与 {c}', },),).toBe('`a {b}` 与 \\{c\\}',);
            expect(encoded({ text: '`a\nb {c}` 与 {d}', },),).toBe('`a\nb {c}` 与 \\{d\\}',);
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
            expect(encoded({ text: '`{a} 与 {b}\n\n~~~\n{c}', },),).toBe('`\\{a\\} 与 \\{b\\}\n\n~~~\n{c}\n~~~',);
          },
        },),

        it({
          name: 'READS INDENTED CODE AS TEXT, since MDX has no indented code and reads such a line as a paragraph',
          fn: async () => {
            expect(encoded({ text: '猫\n\n    {x} <y>\n', },),).toBe('猫\n\n    \\{x\\} \\<y\\>\n',);
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
          name: 'READS THE LINE AFTER AN OPENER OF A COMMENT THAT NEVER CLOSES AS A LINE OF ITS PARAGRAPH, since the '
            + 'opener is written as text and a definition cannot interrupt a paragraph, so a brace in that line is '
            + 'escaped as any other',
          fn: async () => {
            expect(encoded({ text: '<!--\n[r]: u{b "t {"\n', },),).toBe('&lt;!--\n[r]: u\\{b "t \\{"\n',);
            expect(encoded({ text: '- <!--\n  [r]: u{b\n', },),).toBe('- &lt;!--\n  [r]: u\\{b\n',);
          },
        },),

        it({
          name: 'WRITES EVERY OPENER INSIDE A COMMENT THAT NEVER CLOSES AS A CHARACTER REFERENCE TOO, since the '
            + 'package\'s mask opens a comment of its own at each one that stays, and the text after the run would '
            + 'be hidden from it',
          fn: async () => {
            expect(encoded({ text: '猫 <!-- 一 <!-- 二\n', },),).toBe('猫 &lt;!-- 一 &lt;!-- 二\n',);
            expect(encoded({ text: '<!-- 一\n<!-- 二\n<!-- 三 {a}\n', },),)
              .toBe('&lt;!-- 一\n&lt;!-- 二\n&lt;!-- 三 \\{a\\}\n',);
            // A backslash before a later opener goes with it, as it does before the first.
            expect(encoded({ text: String.raw`猫 <!-- 一 \<!-- 二`, },),).toBe('猫 &lt;!-- 一 &lt;!-- 二',);
            // A comment that closes keeps its opener, and the opener of one that does not is written whole.
            expect(encoded({ text: '猫 <!-- 一 --> 猫 <!-- 二 <!-- 三\n', },),)
              .toBe('猫 <!-- 一 --> 猫 &lt;!-- 二 &lt;!-- 三\n',);
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
            expect(encoded({ text: '[^1]: <b>', },),).toBe(String.raw`[^1]: \<b\>`,);
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
          name: 'JOINS THE LINES OF A CODE SPAN, INLINE MATH OR A TITLE WHOSE LATER LINE STARTS WITH A BRACE OR A '
            + 'LESS-THAN SIGN, since the strict grammar starts an expression or a tag there and ends the paragraph, '
            + 'where a line ending in a code span reads as a space and no backslash may be written into the span',
          fn: async () => {
            expect(encoded({ text: '`a\n{b` {c', },),).toBe('`a {b` \\{c',);
            expect(encoded({ text: '`a\n  <b` {c', },),).toBe('`a <b` \\{c',);
            expect(encoded({ text: '$a\n{b$ {c', },),).toBe(String.raw`$a {b$ \{c`,);
            expect(encoded({ text: '- `a\n  {b` {c', },),).toBe('- `a {b` \\{c',);
            expect(encoded({ text: '> `a\n> {b` {c', },),).toBe('> `a {b` \\{c',);
            expect(encoded({ text: '[a](u "t\n{b") {c', },),).toBe(String.raw`[a](u "t {b") \{c`,);
            // A closed brace joins too, since a line holding only it is an expression of its own to the strict grammar.
            expect(encoded({ text: '`a\n{b}` {c', },),).toBe('`a {b}` \\{c',);
            // A later line that does not start with a brace or a less-than sign keeps its line ending.
            expect(encoded({ text: '`a\nb {c` {d', },),).toBe('`a\nb {c` \\{d',);
          },
        },),

        it({
          name: 'READS NO BARE ADDRESS AFTER A BRACKET NOTHING CLOSES, since GFM leaves an address in the text after '
            + 'an open label, so a brace in it is escaped as any other and a pair of brackets before it changes '
            + 'nothing',
          fn: async () => {
            expect(encoded({ text: '[ https://example.net/b{c', },),).toBe(String.raw`[ https://example.net/b\{c`,);
            expect(encoded({ text: 'x ![ www.example.net/b{c', },),).toBe(String.raw`x ![ www.example.net/b\{c`,);
            expect(encoded({ text: '[a [b](u) https://example.net/c{d', },),)
              .toBe(String.raw`[a [b](u) https://example.net/c\{d`,);
            // An address a tag ends is a link of its own there too, since only the transform reads it and it runs on.
            expect(encoded({ text: '[a https://example.net/a<b> {', },),)
              .toBe(String.raw`[a [https://example.net/a](https://example.net/a)\<b> \{`,);
            // A pair of brackets, a new paragraph and an escaped bracket leave the address one.
            expect(encoded({ text: 'x [y] https://example.net/b{c', },),).toBe('x [y] https://example.net/b{c',);
            expect(encoded({ text: '[ x\n\nhttps://example.net/b{c', },),).toBe(String.raw`[ x

https://example.net/b{c`,);
            expect(encoded({ text: String.raw`\[ https://example.net/b{c`, },),)
              .toBe(String.raw`\[ https://example.net/b{c`,);
          },
        },),

        it({
          name: 'ESCAPES EVERY CHARACTER OF A RAW HTML BLOCK OR INDENTED CODE THAT MARKDOWN READS AS SYNTAX, since the '
            + 'strict grammar reads such lines as a paragraph and the reader must see them as they stand: the '
            + 'backslash itself, a block mark and a link among them',
          fn: async () => {
            expect(encoded({ text: '<div>\n[a](u) *b* {c}\n</div>\n', },),)
              .toBe(String.raw`\<div\>
\[a\](u\) \*b\* \{c\}
\</div\>
`,);
            expect(encoded({ text: '猫\n\n    - a {\n    # b\n', },),).toBe(String.raw`猫

    \- a \{
    \# b
`,);
            expect(encoded({ text: '<div>\\{ &lt;</div>\n', },),).toBe(String.raw`\<div\>\\\{ \&lt;\</div\>
`,);
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
          name: 'WRITES THE FIRST LETTER OF A MODULE LINE THAT FOLLOWS A LINE OF A LIST MARKER ALONE RIGHT AFTER A '
            + 'FOOTNOTE DEFINITION, since the strict grammar starts a list there, which it does not where a '
            + 'paragraph is open, and the line after the marker starts a module',
          fn: async () => {
            expect(encoded({ text: '[^1]: 注\n*\nexport to PDF\n', },),).toBe('[^1]: 注\n*\n&#101;xport to PDF\n',);
            expect(encoded({ text: '[^1]: 注 *a*\n*\nimport data\n', },),)
              .toBe('[^1]: 注 *a*\n*\n&#105;mport data\n',);
            // Where a paragraph is open the marker is a line of it, and so is the line after it.
            expect(encoded({ text: 'abc\n*\nexport to PDF\n', },),).toBe('abc\n*\nexport to PDF\n',);
            expect(encoded({ text: '[^1]: 注\n*\nfoo\nexport to PDF\n', },),)
              .toBe('[^1]: 注\n*\nfoo\nexport to PDF\n',);
          },
        },),

        it({
          name: 'READS A DEFINITION LINE RIGHT AFTER A FOOTNOTE DEFINITION IN A BLOCK QUOTE OR A LIST ITEM AS A LINE '
            + 'OF THE FOOTNOTE too, since the strict grammar continues the paragraph of the footnote there as well',
          fn: async () => {
            expect(encoded({ text: '> [^1]: 注\n[b]: u{ "t"\n', },),).toBe('> [^1]: 注\n[b]: u\\{ "t"\n',);
            expect(encoded({ text: '- [^1]: 注\n[b]: u{ "t"\n', },),).toBe('- [^1]: 注\n[b]: u\\{ "t"\n',);
            expect(encoded({ text: '- a\n\n  [^1]: 注\n  [b]: u{ "t"\n', },),)
              .toBe('- a\n\n  [^1]: 注\n  [b]: u\\{ "t"\n',);
            // A blank line ends the footnote in the container, so the definition after it is a definition.
            expect(encoded({ text: '> [^1]: 注\n>\n> [b]: u{ "t"\n', },),).toBe('> [^1]: 注\n>\n> [b]: u{ "t"\n',);
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
          name: 'KEEPS THE LINK AROUND AN ANGLE AUTOLINK INSIDE LINK TEXT and writes the address as text without '
            + 'the angle brackets, since a link cannot hold a link and MDX reads the autolink as a tag: the reader '
            + 'sees the outer link and the address, as the source shows them',
          fn: async () => {
            expect(encoded({ text: '[<https://example.net/a>](u) {', },),)
              .toBe(String.raw`[https://example.net/a](u) \{`,);
            expect(readerOfPage({ carried: encoded({ text: '[<https://example.net/a>](u) {', },), },),)
              .toBe('⟦u⟧https://example.net/a {',);
          },
        },),

        it({
          name: 'WRITES AN ANGLE AUTOLINK INSIDE THE ALTERNATIVE TEXT OF AN IMAGE WITH A BACKSLASH LIKE ANY TEXT, '
            + 'so the page shows its brackets (a limit the module comment names)',
          fn: async () => {
            expect(encoded({ text: '![<https://example.net/a>](u.png) {', },),)
              .toBe(String.raw`![\<https://example.net/a>](u.png) \{`,);
            expect(readerOfPage({ carried: encoded({ text: '![<https://example.net/a>](u.png) {', },), },),)
              .toBe('<https://example.net/a> {',);
          },
        },),

        it({
          name: 'PARSES AND HOLDS NO EXPRESSION FOR A FOOTNOTE DEFINITION INSIDE A CONTAINER, A TAG ALONE AND A '
            + 'DEFINITION LINE WITH A BRACE (a reading difference the follow-ups name): the definition line shows as '
            + 'text on the page, where the source reads it as a definition',
          fn: async () => {
            for (
              const text of [
                '> a\n> [^1]: x\n> <a href="x">\n> [r]: u{b}\n',
                '> a\n>     ```\n> [^1]: x\n> <a href="x">\n> [r]: u{b}\n',
                '- a\n  [^1]: x\n  <a href="x">\n  [r]: u{b}\n',
              ]
            ) {
              /**
               What the encoder writes for the text.
               */
              const written = encoded({ text, },);
              expect(readPageGrammar({ pageText: `${BEFORE}${written}\n`, },),).toEqual({ kind: 'parses', },);
              expect(liveConstructsOfCarried({ carried: written, },),).toEqual([],);
            }
          },
        },),

        it({
          name: 'WRITES A REFERENCE WHOSE LABEL HOLDS A BRACE OR A LESS-THAN SIGN WITH A BACKSLASH AND LEAVES THE '
            + 'LABEL OF ITS DEFINITION AS IT STANDS (a limit the module comment names): the page parses and holds no '
            + 'expression, and the reference no longer names the definition, so it shows as text',
          fn: async () => {
            for (
              const [text, written,] of [
                ['猫[^{a}]\n\n[^{a}]: 注\n', '猫[^\\{a\\}]\n\n[^{a}]: 注\n',],
                ['猫[^a}]\n\n[^a}]: 注\n', '猫[^a\\}]\n\n[^a}]: 注\n',],
                ['猫[^<b>]\n\n[^<b>]: 注\n', '猫[^\\<b>]\n\n[^<b>]: 注\n',],
              ] as const
            ) {
              expect(encoded({ text, },),).toBe(written,);
              expect(readPageGrammar({ pageText: `${BEFORE}${written}\n`, },),).toEqual({ kind: 'parses', },);
            }
          },
        },),

        it({
          name: 'WRITES THE EXPORT LINE AFTER A MARKER ALONE THAT FOLLOWS A FOOTNOTE DEFINITION INDENTED FOUR SPACES '
            + 'INSIDE A PARAGRAPH, which MDX reads as a definition where a reader of CommonMark with GFM, whose '
            + 'indented code the strict grammar has not, reads a line of the paragraph',
          fn: async () => {
            /**
             The page the run is written to, which parses.
             */
            const written = encoded({ text: 'x\n    [^1]: 注\n*\nexport to PDF\n', },);
            expect(written,).toBe('x\n    [^1]: 注\n*\n&#101;xport to PDF\n',);
            expect(readPageGrammar({ pageText: `${BEFORE}${written}\n`, },),).toEqual({ kind: 'parses', },);
          },
        },),

        it({
          name: 'READS THE LINES INDENTED UNDER A FOOTNOTE DEFINITION AT THE TOP AS THE BLOCKS OF THE FOOTNOTE, so a '
            + 'fence mark among them is left as it stands and the braces inside the fence are not escaped',
          fn: async () => {
            expect(encoded({ text: '[^1]: x {\nabc\n    ```\n    {a}\n', },),)
              .toBe('[^1]: x \\{\nabc\n    ```\n    {a}\n',);
            expect(encoded({ text: '[^1]: x\n    ~~~\n    {a}\n    ~~~\n    tail {z}\n', },),)
              .toBe('[^1]: x\n    ~~~\n    {a}\n    ~~~\n    tail \\{z\\}\n',);
          },
        },),

        it({
          name: 'ENDS THE FOOTNOTE AT THE LINE AFTER AN UNCLOSED FENCE INDENTED UNDER IT, so the lines that follow are '
            + 'written as the text they are and the page parses',
          fn: async () => {
            for (
              const [source, written,] of [
                ['[^1]: x\n    ~~~\n<>\n', '[^1]: x\n    ~~~\n\\<>\n',],
                ['[^1]: x\n    ```\n</\n', '[^1]: x\n    ```\n\\</\n',],
                ['[^1]: 注 ({)\n    ```\n[x][a {b]\n', '[^1]: 注 (\\{)\n    ```\n[x][a \\{b]\n',],
                ['[^1]: x\n    ```\n{b}\n', '[^1]: x\n    ```\n\\{b\\}\n',],
              ] as const
            ) {
              expect(encoded({ text: source, },),).toBe(written,);
              expect(readPageGrammar({ pageText: `${BEFORE}${written}\n`, },),).toEqual({ kind: 'parses', },);
            }
          },
        },),

        it({
          name: 'WRITES A BACKSLASH BEFORE THE MARKS OF A FENCE INDENTED UNDER A FOOTNOTE DEFINITION IN A LIST ITEM OR '
            + 'A BLOCK QUOTE, and the page shows the marks (a limit the module comment names): the lines the nested '
            + 'reading takes carry the prefix of the container, which it does not strip',
          fn: async () => {
            expect(encoded({ text: '- [^1]: x {\n\n      ```\n      {a}\n      ```\n', },),)
              .toBe('- [^1]: x \\{\n\n      \\`\\`\\`\n      \\{a\\}\n      \\`\\`\\`\n',);
            expect(encoded({ text: '> [^1]: x {\n>\n>     ```\n>     {a}\n>     ```\n', },),)
              .toBe('> [^1]: x \\{\n>\n>     \\`\\`\\`\n>     \\{a\\}\n>     \\`\\`\\`\n',);
          },
        },),

        it({
          name: 'WRITES ONE LINK ACROSS THE CELLS OF A TABLE FOR A BARE ADDRESS THAT A PIPE ENDS AND FOR AN ANGLE '
            + 'AUTOLINK HOLDING A PIPE WRITTEN WITH A BACKSLASH (limits the module comment names): the table cuts '
            + 'the source at the pipe and the page holds one link',
          fn: async () => {
            for (
              const [text, written,] of [
                [
                  '| a | b |\n| - | - |\n| https://example.net/a|b<c> | d |\n',
                  '| a | b |\n| - | - |\n| [https://example.net/a|b](https://example.net/a|b)\\<c> | d |\n',
                ],
                [
                  '| a | b |\n| - | - |\n| <https://example.net/a\\|b> | c |\n',
                  '| a | b |\n| - | - |\n| [https://example.net/a\\\\|b](https://example.net/a%5C|b) | c |\n',
                ],
              ] as const
            ) {
              expect(encoded({ text, },),).toBe(written,);
              expect(readerOfPage({ carried: written, },) === readerOfSource({ source: text, },),).toBe(false,);
            }
          },
        },),

        it({
          name: 'SEPARATES THE DEFINITION LINE AFTER A TAG ALONE ON A LINE FROM THE FOOTNOTE WITH A BLANK LINE, since '
            + 'the original reads the tag as a block that ends the footnote and the page, which writes the tag as '
            + 'text, would otherwise go on with the footnote\'s paragraph and hold the braces of the definition as an '
            + 'expression',
          fn: async () => {
            /**
             A footnote, a tag alone on the next line, then a line the plain parse reads as a definition.
             */
            const text = '[^1]: x\n<a href="x">\n[r]: u{b}\n';
            /**
             What the encoder writes for it: a blank line ends the footnote's paragraph before the definition line.
             */
            const written = '[^1]: x\n\\<a href="x">\n\n[r]: u{b}\n';
            expect(encoded({ text, },),).toBe(written,);
            expect(liveConstructsOfCarried({ carried: written, },),).toEqual([],);
            expect(readerOfPage({ carried: written, },),).toBe(readerOfSource({ source: text, },),);
          },
        },),

        it({
          name: 'CLOSES A FENCE OR DISPLAY MATH THE TEXT LEAVES OPEN AT ITS END, so the lines the page carries after '
            + 'the text are no part of it: a fence mark in the next run would close it and expose the rest of that run',
          fn: async () => {
            for (
              const [text, written,] of [
                ['$$\n{x\n', '$$\n{x\n$$',],
                ['$$\nx < {y}', '$$\nx < {y}\n$$',],
                ['```\n{a}\n', '```\n{a}\n```',],
                ['~~~~\n{a}', '~~~~\n{a}\n~~~~',],
                ['```\r\n{a}', '```\r\n{a}\r\n```',],
                ['\uFEFF```\n{\n```\n', '\uFEFF```\n\\{\n```\n```',],
              ] as const
            ) {
              expect(encoded({ text, },),).toBe(written,);
            }
          },
        },),

        it({
          name: 'LEAVES A TEXT THAT CLOSES ITS FENCES, AND A FENCE IN A CONTAINER, AS IT IS: only a fence or display '
            + 'math that is open at the end of the text, at its top, gets a closing line',
          fn: async () => {
            for (
              const [text, written,] of [
                ['```\n{a}\n```\n', '```\n{a}\n```\n',],
                ['$$\nx\n$$', '$$\nx\n$$',],
                ['```\n{a}\n```\n\n{b}', '```\n{a}\n```\n\n\\{b\\}',],
                ['- ```\n  {a}\n', '- ```\n  {a}\n',],
                ['> ```\n> {a}\n', '> ```\n> {a}\n',],
                ['[^1]: x\n    ```\n    {a}\n', '[^1]: x\n    ```\n    {a}\n',],
              ] as const
            ) {
              expect(encoded({ text, },),).toBe(written,);
            }
          },
        },),

        it({
          name: 'LEAVES NO CODE ON A PAGE WHOSE TWO RUNS EACH LEAVE A FLOW OPEN, where the second run\'s opening line '
            + 'closed the first run\'s flow and exposed the braces after it',
          fn: async () => {
            /**
             Two runs that each end inside display math, with English between them.
             */
            const first = encoded({ text: '$$\n{x\n', },);
            const second = encoded({ text: '$$\nx < {y}\n', },);
            expect(liveConstructsOf({
              pageText: `${BEFORE}${first}\n\nShe slept on the counter.\n\n${second}\n`,
            },),).toEqual([],);
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
          name: 'READS A BYTE ORDER MARK THAT OPENS THE TEXT AS A CHARACTER OF ITS FIRST LINE, since the page carries '
            + 'the text after a note and the mark then stands in the middle of the page, where it starts no block '
            + 'and no module',
          fn: async () => {
            expect(encoded({ text: '\uFEFF```\n{\n```\n', },),).toBe('\uFEFF```\n\\{\n```\n```',);
            expect(encoded({ text: '\uFEFF- ```\n  {\n  ```\n', },),).toBe('\uFEFF- ```\n  \\{\n  ```\n```',);
            expect(encoded({ text: '\uFEFFimport a {\n', },),).toBe('\uFEFFimport a \\{\n',);
            expect(encoded({ text: '\uFEFF[r]: u{b "t {"\n', },),).toBe('\uFEFF[r]: u\\{b "t \\{"\n',);
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
              expect(liveConstructsOfCarried({ carried: written, },),).toEqual([],);
              expect(readerOfPage({ carried: written, },),).toBe(readerOfSource({ source: text, },),);
            },
          },);
        },),

        ...LITERAL_CASES.map(function literalCase([name, text,],) {
          return it({
            name: `SHOWS THE CHARACTERS OF THE SOURCE FOR ${name.toUpperCase()}: the page parses and its text, `
              + 'links aside, equals the source\'s',
            fn: async () => {
              /**
               The run as the encoder writes it.
               */
              const written = encoded({ text, },);
              expect(readPageGrammar({ pageText: `${BEFORE}${written}\n`, },),).toEqual({ kind: 'parses', },);
              expect(liveConstructsOfCarried({ carried: written, },),).toEqual([],);
              expect(withoutLinkTargets({ account: readerOfPage({ carried: written, },), },),)
                .toBe(withoutLinkTargets({ account: readerOfSource({ source: text, },), },),);
            },
          },);
        },),

        ...SHOWN_CASES.map(function shownCase([name, source, shown,],) {
          return it({
            name: `SHOWS THE CHARACTERS OF THE LINE FOR ${name.toUpperCase()}: the page parses and shows ${shown}`,
            fn: async () => {
              /**
               The run as the encoder writes it.
               */
              const written = encoded({ text: source, },);
              expect(readPageGrammar({ pageText: `${BEFORE}${written}\n`, },),).toEqual({ kind: 'parses', },);
              expect(liveConstructsOfCarried({ carried: written, },),).toEqual([],);
              expect(readerOfPage({ carried: written, },).startsWith(shown,),).toBe(true,);
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
          name: 'LEAVES THE ENGLISH AFTER A RUN HOLDING TWO OPENERS OF A COMMENT THAT NEVER CLOSES TO THE PACKAGE\'S '
            + 'OWN PARSER: no unterminated comment is found in the page and the paragraph after the run is a node of '
            + 'it',
          fn: async () => {
            for (const text of ['猫 <!-- 一 <!-- 二\n', '$a < b$ <!-- <!--\n', '[^1]: 注 <!-- <!--\n',]) {
              /**
               The package's reading of a page whose run holds two openers and whose English follows.
               */
              const { parseFindings, nodes, } = parseDocument({
                text: `${BEFORE}${encoded({ text, },)}\nShe slept on the counter.\n`,
              },);
              expect(parseFindings.map(function kindOf({ kind, },): string {
                return kind;
              },),).toEqual([],);
              expect(nodes.at(-1,)?.text,).toBe('She slept on the counter.',);
            }
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
