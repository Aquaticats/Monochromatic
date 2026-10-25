//region Pools of source texts
// THE PARTS THE SEEDED SAMPLES OF SOURCE TEXTS ARE BUILT FROM (`source-live-sample.test-fixture.ts`): the fragments a
// run of a pair is drawn from, what stands between them and between the runs, and the stages of the generated footnote
// texts.
//
// TEST SUPPORT, NOT PACKAGE SOURCE.

/**
 What a generated footnote text puts before its first line and before each later line.
 */
type Container = {
  /**
   Text before the first line.
   */
  readonly open: string;

  /**
   Text before every other line.
   */
  readonly hang: string;

  /**
   What stands between the lines: a line feed, or a lone carriage return.
   */
  readonly join: string;
};

/**
 Code point of the byte order mark.
 */
const BOM_CODE_POINT = 0xFE_FF;

/**
 Byte order mark, built from its code point so no file holds the invisible character itself.
 */
const BOM = String.fromCodePoint(BOM_CODE_POINT,);

/**
 Depth of a quote chain just under the bound the plain reading keeps.
 */
const QUOTES_UNDER_BOUND = 260;

/**
 Depth of a nesting past the bound the plain reading keeps.
 */
const NESTING_PAST_BOUND = 300;

/**
 Fragments a run is built from.
 */
export const RUN_FRAGMENTS: readonly string[] = [
  // Braces, tags, comments, fences, math, footnotes and the other syntax the grammar reads as code.
  '她说',
  '{',
  '}',
  '{他说}',
  '{x} <y>',
  '<details>',
  '</details>',
  '<Foo />',
  '<b>粗</b>',
  '<https://example.net/a>',
  '<!--',
  '-->',
  '<!-- 注释 -->',
  'import a from "b"',
  'export to PDF',
  '`{`',
  '```',
  '- ```',
  '> ',
  '> '.repeat(QUOTES_UNDER_BOUND,),
  '[',
  '['.repeat(NESTING_PAST_BOUND,),
  '![图](u.png)',
  '[^1]',
  '[^1]: 注',
  '&lt;',
  '&#123;',
  String.raw`\{`,
  String.raw`\<`,
  '$a < b$',
  '$$',
  '<script>',
  '</',
  '<a href="x">',
  '</a>',
  '<>',
  '</>',
  '{/* c */}',
  '<br>',
  '*',
  '_',
  '    {x}',
  '---',
  '# 标题',
  '1. 项',
  'www.example.net/{a}',
  '\r\n',
  BOM,
  // Footnote definitions CommonMark reads as link reference definitions, and the blocks indented under one.
  '[^1]: {',
  '[^1]: <b>',
  '[^1]: <https://example.net/x>',
  '[^1]: a{b',
  '[^1]: a<b',
  '[^1]: 注 ({)',
  '[^1]: x',
  '    ```\n    {\n    ```',
  '    注 {',
  '    ```',
  '    {y}',
  // Setext headings that start with a module word.
  'import data from the old system\n===',
  'export to PDF\n---',
  'import x from y\nabc\n===',
  // Bare addresses ended by a tag, inside link text, with nothing after the protocol, and www.
  'https://example.net/a<b>',
  'www.example.net/a<b>',
  'https://example.net/a_b<c>',
  '[https://example.net/a{ b](u)',
  'https://{x',
  'https://',
  'https://example.net/b{c',
  'https://example.net/a.<b>',
  // An angle autolink inside link text, reference labels with a brace.
  '[<https://example.net/a>](u)',
  '[a {b][]',
  '[a {b]: u',
  '[x][a {b]',
  '[r]: u{b "t {"',
  // Math flows one run opens and the next closes, and a formula on one line.
  '$$\nx < {y}',
  '$$\n{x',
  '$$a < {b}$$',
  String.raw`$a \< b$`,
  // An escapable character outside a fence.
  '{a}',
  '<q>',
  'a_b*c',
  String.raw`\*`,
  // A fence mark indented four spaces after a paragraph line, in containers, across a code span, a tab, a repeated
  // import, a byte order mark line before a setext heading starting import.
  'abc\n    ```',
  '\t```',
  '> a\n>     ```',
  '`x\n    ```\n    y`',
  '    ~~~',
  'import a from "b"\n\n它',
  `${BOM}\nimport q\n===`,
  '[ https://example.net/a{b',
  '<div>\n[https://example.net/a{ b](u)\n</div>',
  '[^1]: 注\n[b]: u{ "t"',
  '`a\n{b` {c',
  // A comment opener beside a definition, a byte order mark before a block, a definition after a block quote holding a
  // footnote definition, a list marker alone after a footnote definition, an autolink in link text, a fence in a nested
  // list item, an unclosed opener twice.
  '<!-- a\n[a]: u',
  `${BOM}- \`\`\``,
  `${BOM}\`\`\`\n{\n\`\`\``,
  '> [^1]: 注\n[b]: u',
  '[^1]: 注\n*\nexport to PDF',
  '[^1]: 注\n2.\nimport a',
  '[<https://example.net/a>](u)',
  '![<https://example.net/a>](u.png)',
  '- a\n  - ```\n    {\n    ```',
  '<!-- a <!-- b',
  '&lt;!--',
  // A lone carriage return as the line ending, and a text mixing the three endings, in the shapes that read lines: a
  // footnote then a definition, a flow that closes and one that never does, a comment before a definition.
  '\r',
  '[^1]: <b>\r[r]: u{b}\r',
  '[^1]: x\r<a href="x">\r[r]: u{b}\r',
  '$\r{a}\r$\r',
  '```\r{a}\r```\r',
  '~~~\r{a}\r~~~\r',
  '$\r{x',
  '- ```\r  {a}\r  ```\r',
  '> $\r> {a}\r> $\r',
  'x\r\r<!--\r[r]: u{b "t {"\r',
  'a\r\nb\rc\n{d}',
  // Footnote labels other than 1: a word, a space, a brace, a less-than sign, a closing brace, and a reference first.
  '[^note]: <b>',
  '[^a b]: x {',
  '[^{a}]: x\n\n[^{a}] {b}',
  '[^<b>]: x\n\n[^<b>] {c}',
  '[^a}]: x {\n\n[^a}] y',
  'x[^{a}] {b}\n\n[^{a}]: y',
  // Tilde fences at the first column, open and closed, with an info string, and a longer backtick fence holding a
  // shorter one.
  '~~~',
  '~~~\n{a}',
  '~~~\n{a}\n~~~',
  '~~~js {x}',
  '````\n```\n{\n````',
  // A footnote that is a tag alone, or a tag alone on the line after it, then a definition line with balanced and
  // unbalanced braces, one after another, after indented code, alone, and in a list item and a block quote.
  '[^1]: <b>\n[r]: u{b}',
  '[^1]: <b>\n[r]: u{b "t {"',
  '[^1]: x\n<a href="x">\n[r]: u{b}',
  '[^1]: x\n<i>\n[r]: u{b}\n[s]: v{c}',
  '[^1]: x\n\n        c\n[r]: u{b}',
  '[r]: u{b}',
  '- [^1]: <b>\n  [r]: u{b}',
  '> [^1]: x\n> <a href="x">\n> [r]: u{b}',
  '- > [^1]: <b>\n  > [r]: u{b}',
  // Nesting 300 deep, in the two forms the plain reading bounds.
  '> '.repeat(NESTING_PAST_BOUND,),
  '- '.repeat(NESTING_PAST_BOUND,),
];

/**
 What a run puts between its fragments.
 */
export const RUN_SEPARATORS: readonly string[] = [
  ' ',
  '\n',
  '\n\n',
  '\r',
  '\r\r',
];

/**
 English the archive holds between the runs.
 */
export const ENGLISH_MIDDLES: readonly string[] = [
  'She slept on the counter.\n',
  '--> She slept.\n',
  '[^1]: A note.\n',
  'import x from "y"\n',
];

/**
 Containers the generated footnote texts stand in.
 */
export const CONTAINERS: readonly Container[] = [
  {
    open: '',
    hang: '',
    join: '\n',
  },
  {
    open: '- ',
    hang: '  ',
    join: '\n',
  },
  {
    open: '> ',
    hang: '> ',
    join: '\n',
  },
  {
    open: '',
    hang: '',
    join: '\r',
  },
];

/**
 Stages of the generated footnote texts: for each stage the lines it may put on the page, one option per element,
 several lines of an option joined by line endings and no line at all written as the empty text.
 */
export const FOOTNOTE_STAGES: readonly (readonly string[])[] = [
  // The footnote definition, with and without a tag beside it.
  [
    '[^1]: x',
    '[^1]: x {',
    '[^1]: <b>',
    '[^1]: <b> abc',
    '[^1]: 注',
    '[^{a}]: x',
    '[^a b]: x',
  ],
  // Paragraph lines of the footnote.
  [
    '',
    'abc',
    'abc {',
  ],
  // A fence mark indented four spaces or more, or a tab, or a tilde fence at the first column.
  [
    '    ```',
    '    ~~~',
    '    ```js',
    '     ```',
    '\t```',
    '~~~',
  ],
  // What stands inside the indented block.
  [
    '',
    '    {a}',
    '    a',
    '    <i>\n    {b}',
  ],
  // What closes it.
  [
    '',
    '    ```',
    '```',
    '    ~~~',
  ],
  // What follows, a definition line among it.
  [
    '',
    '    tail {z}',
    'tail {z}',
    '[r]: u{b}',
  ],
];

//endregion Pools of source texts
