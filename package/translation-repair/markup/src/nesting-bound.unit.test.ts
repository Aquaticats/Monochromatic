/**
 Tests for the nesting bound: the linear pass that refuses a body nested
 deeply enough to exhaust a parser's stack, or to cost it minutes, before any
 parser reads it.

 Every measure has a case at the bound and a case one past it, since the
 bound is what the case pins: a count that drifts by one in either direction
 either refuses a text the grammar reads or admits one it cannot.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  DELIMITER_BOUND,
  firstNestingExcess,
  isStackOverflow,
  NESTING_BOUND,
} from '../dist/final/node/index.mjs';

//region Nesting bound tests

/**
 What reading a body found within the bound.
 */
const WITHIN = { kind: 'within', };

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: firstNestingExcess.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS the bounds as 256 levels and 1,024 emphasis delimiters',
          fn: async () => {
            expect(NESTING_BOUND,).toBe(256,);
            expect(DELIMITER_BOUND,).toBe(1_024,);
          },
        },),
        it({
          name: 'READS ordinary prose, a quotation and a list as within the bound under both grammars',
          fn: async () => {
            /**
             Text no reader would find nested.
             */
            const body = 'The cat naps.\n\n> The cat said **mew** and [left](https://cat.example/a).\n\n- one\n- two\n';

            expect(firstNestingExcess({ body, grammar: 'mdx', },),).toEqual(WITHIN,);
            expect(firstNestingExcess({ body, grammar: 'markdown', },),).toEqual(WITHIN,);
          },
        },),
        it({
          name: 'READS 256 quotation marks as within and the 257th as the first container marker past the bound',
          fn: async () => {
            expect(firstNestingExcess({ body: `${'>'.repeat(256,)} cat`, grammar: 'markdown', },),).toEqual(WITHIN,);
            expect(firstNestingExcess({ body: `${'>'.repeat(257,)} cat`, grammar: 'markdown', },),).toEqual({
              kind: 'beyond',
              measure: 'container markers',
              bound: 256,
              line: 1,
              column: 257,
            },);
            expect(firstNestingExcess({ body: `${'> '.repeat(257,)}cat`, grammar: 'mdx', },),).toEqual({
              kind: 'beyond',
              measure: 'container markers',
              bound: 256,
              line: 1,
              column: 513,
            },);
          },
        },),
        it({
          name: 'COUNTS bullet markers, ordered markers and footnote labels as container markers on the line they open',
          fn: async () => {
            expect(firstNestingExcess({ body: `cat\n${'- '.repeat(257,)}cat`, grammar: 'markdown', },),).toEqual({
              kind: 'beyond',
              measure: 'container markers',
              bound: 256,
              line: 2,
              column: 513,
            },);
            expect(firstNestingExcess({ body: `${'1. '.repeat(257,)}cat`, grammar: 'markdown', },),).toEqual({
              kind: 'beyond',
              measure: 'container markers',
              bound: 256,
              line: 1,
              column: 769,
            },);
            expect(firstNestingExcess({ body: `${'[^1]: '.repeat(257,)}cat`, grammar: 'markdown', },),).toEqual({
              kind: 'beyond',
              measure: 'container markers',
              bound: 256,
              line: 1,
              column: 1_537,
            },);
            expect(firstNestingExcess({ body: `${'- '.repeat(256,)}cat`, grammar: 'markdown', },),).toEqual(WITHIN,);
          },
        },),
        it({
          name: 'COUNTS a marker only at the start of a line, so a greater-than sign inside a sentence nests nothing',
          fn: async () => {
            expect(firstNestingExcess({
              body: `The cat is ${'> '.repeat(300,)}bigger.\n\n2024. A year.\n\n-not a list`,
              grammar: 'markdown',
            },),).toEqual(WITHIN,);
          },
        },),
        it({
          name: 'READS 512 columns of indentation as within and 514 as an indentation past the bound',
          fn: async () => {
            expect(firstNestingExcess({ body: `${' '.repeat(512,)}cat`, grammar: 'markdown', },),).toEqual(WITHIN,);
            expect(firstNestingExcess({ body: `cat\n${' '.repeat(514,)}cat`, grammar: 'markdown', },),).toEqual({
              kind: 'beyond',
              measure: 'indentation',
              bound: 256,
              line: 2,
              column: 515,
            },);
            expect(firstNestingExcess({ body: `${'\t'.repeat(129,)}cat`, grammar: 'markdown', },),).toEqual({
              kind: 'beyond',
              measure: 'indentation',
              bound: 256,
              line: 1,
              column: 130,
            },);
          },
        },),
        it({
          name: 'IGNORES a line of nothing but blanks, however wide',
          fn: async () => {
            expect(firstNestingExcess({ body: `cat\n${' '.repeat(2_000,)}\ncat`, grammar: 'markdown', },),).toEqual(WITHIN,);
          },
        },),
        it({
          name: 'READS 256 open brackets as within and the 257th as the first bracket past the bound',
          fn: async () => {
            expect(firstNestingExcess({ body: `${'['.repeat(256,)}cat`, grammar: 'markdown', },),).toEqual(WITHIN,);
            expect(firstNestingExcess({ body: `${'!['.repeat(257,)}cat`, grammar: 'markdown', },),).toEqual({
              kind: 'beyond',
              measure: 'brackets',
              bound: 256,
              line: 1,
              column: 514,
            },);
          },
        },),
        it({
          name: 'CLOSES a bracket on its closer, so a long run of links nests nothing, and ends the count at a blank line',
          fn: async () => {
            expect(firstNestingExcess({ body: '[a](u) '.repeat(1_000,), grammar: 'markdown', },),).toEqual(WITHIN,);
            expect(firstNestingExcess({ body: `${`${'['.repeat(200,)}\n\n`.repeat(3,)}cat`, grammar: 'markdown', },),).toEqual(WITHIN,);
          },
        },),
        it({
          name: 'SKIPS an escaped bracket, which opens nothing',
          fn: async () => {
            expect(firstNestingExcess({ body: String.raw`\[`.repeat(300,), grammar: 'markdown', },),).toEqual(WITHIN,);
          },
        },),
        it({
          name: 'READS 1,024 emphasis delimiters of a block as within and the 1,025th as the first past the bound',
          fn: async () => {
            expect(firstNestingExcess({ body: '*a '.repeat(1_024,), grammar: 'markdown', },),).toEqual(WITHIN,);
            expect(firstNestingExcess({ body: '*a '.repeat(1_025,), grammar: 'markdown', },),).toEqual({
              kind: 'beyond',
              measure: 'emphasis delimiters',
              bound: 1_024,
              line: 1,
              column: 3_073,
            },);
            expect(firstNestingExcess({ body: `${'_a '.repeat(600,)}\n${'~~a '.repeat(600,)}`, grammar: 'markdown', },),).toEqual({
              kind: 'beyond',
              measure: 'emphasis delimiters',
              bound: 1_024,
              line: 2,
              column: 849,
            },);
          },
        },),
        it({
          name: 'SKIPS the delimiters of a thematic break and starts a new count at each block',
          fn: async () => {
            expect(firstNestingExcess({ body: `${'*'.repeat(3_000,)}\n\ncat`, grammar: 'markdown', },),).toEqual(WITHIN,);
            expect(firstNestingExcess({ body: `${'*a '.repeat(600,)}\n\n${'*a '.repeat(600,)}`, grammar: 'markdown', },),).toEqual(WITHIN,);
          },
        },),
        it({
          name: 'COUNTS the delimiters of a line that mixes stars and hyphens, which is no thematic break and which '
            + 'costs the parser seconds at 24,000 pairs',
          fn: async () => {
            expect(firstNestingExcess({ body: '*-'.repeat(1_100,), grammar: 'markdown', },),).toEqual({
              kind: 'beyond',
              measure: 'emphasis delimiters',
              bound: 1_024,
              line: 1,
              column: 2_049,
            },);
            expect(firstNestingExcess({ body: '*-'.repeat(1_100,), grammar: 'mdx', },).kind,).toBe('beyond',);
          },
        },),
        it({
          name: 'READS nothing inside a fenced block, where the parser nests nothing, and resumes after its closing fence',
          fn: async () => {
            /**
             A fence holding more brackets and delimiters than the bounds allow.
             */
            const fenced = `\`\`\`\n${'['.repeat(300,)}\n${'*a '.repeat(2_000,)}\n\`\`\`\n`;

            expect(firstNestingExcess({ body: fenced, grammar: 'markdown', },),).toEqual(WITHIN,);
            expect(firstNestingExcess({ body: `${fenced}${'['.repeat(257,)}`, grammar: 'markdown', },),).toEqual({
              kind: 'beyond',
              measure: 'brackets',
              bound: 256,
              line: 5,
              column: 257,
            },);
            expect(firstNestingExcess({ body: `~~~\n${'['.repeat(300,)}\n\`\`\`\n${'['.repeat(300,)}`, grammar: 'markdown', },),).toEqual(WITHIN,);
            expect(firstNestingExcess({ body: `~~~\n${'['.repeat(300,)}\n~~~\n${'['.repeat(300,)}`, grammar: 'markdown', },),).toEqual({
              kind: 'beyond',
              measure: 'brackets',
              bound: 256,
              line: 4,
              column: 257,
            },);
          },
        },),
        it({
          name: 'STILL COUNTS the container markers and the indentation of a line inside a fence',
          fn: async () => {
            expect(firstNestingExcess({ body: `\`\`\`\n${'>'.repeat(257,)} cat`, grammar: 'markdown', },),).toEqual({
              kind: 'beyond',
              measure: 'container markers',
              bound: 256,
              line: 2,
              column: 257,
            },);
          },
        },),
        it({
          name: 'READS 256 open tags as within and the 257th as the first open tag past the bound, under the strict grammar only',
          fn: async () => {
            expect(firstNestingExcess({ body: `${'<div>'.repeat(256,)}cat`, grammar: 'mdx', },),).toEqual(WITHIN,);
            expect(firstNestingExcess({ body: `${'<div>'.repeat(257,)}cat`, grammar: 'mdx', },),).toEqual({
              kind: 'beyond',
              measure: 'open tags',
              bound: 256,
              line: 1,
              column: 1_285,
            },);
            expect(firstNestingExcess({ body: `${'<div>'.repeat(257,)}cat`, grammar: 'markdown', },),).toEqual(WITHIN,);
          },
        },),
        it({
          name: 'CLOSES a tag on its closing tag and counts neither a self-closing tag nor a void element as open',
          fn: async () => {
            expect(firstNestingExcess({ body: '<b>cat</b>'.repeat(1_000,), grammar: 'mdx', },),).toEqual(WITHIN,);
            expect(firstNestingExcess({ body: '<Cat name="mew" />'.repeat(1_000,), grammar: 'mdx', },),).toEqual(WITHIN,);
            expect(firstNestingExcess({ body: 'cat<br>'.repeat(1_000,), grammar: 'mdx', },),).toEqual(WITHIN,);
          },
        },),
        it({
          name: 'READS 256 open braces as within and the 257th as the first brace past the bound, under the strict grammar only',
          fn: async () => {
            expect(firstNestingExcess({ body: `${'{'.repeat(256,)}1`, grammar: 'mdx', },),).toEqual(WITHIN,);
            expect(firstNestingExcess({ body: `cat ${'{'.repeat(257,)}1`, grammar: 'mdx', },),).toEqual({
              kind: 'beyond',
              measure: 'open braces',
              bound: 256,
              line: 1,
              column: 261,
            },);
            expect(firstNestingExcess({ body: `cat ${'{'.repeat(257,)}1`, grammar: 'markdown', },),).toEqual(WITHIN,);
            expect(firstNestingExcess({ body: '{a}'.repeat(1_000,), grammar: 'mdx', },),).toEqual(WITHIN,);
          },
        },),
        it({
          name: 'NAMES the column from the first character the parser reads, a leading byte order mark not counted',
          fn: async () => {
            expect(firstNestingExcess({ body: `\uFEFF${'>'.repeat(257,)} cat`, grammar: 'markdown', },),).toEqual({
              kind: 'beyond',
              measure: 'container markers',
              bound: 256,
              line: 1,
              column: 257,
            },);
          },
        },),
        it({
          name: 'NAMES the earliest of two measures that pass the bound, in reading order',
          fn: async () => {
            expect(firstNestingExcess({
              body: `${'['.repeat(257,)}\n${'>'.repeat(257,)} cat`,
              grammar: 'markdown',
            },),).toEqual({
              kind: 'beyond',
              measure: 'brackets',
              bound: 256,
              line: 1,
              column: 257,
            },);
          },
        },),
        it({
          name: 'FINISHES on a long text whose openers never close, which a rescan per opener would not',
          fn: async () => {
            expect(firstNestingExcess({ body: '<a '.repeat(200_000,), grammar: 'mdx', },),).toEqual(WITHIN,);
            expect(firstNestingExcess({ body: '1'.repeat(200_000,), grammar: 'markdown', },),).toEqual(WITHIN,);
          },
        },),
      ],
    },),

    describe({
      name: isStackOverflow.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS the RangeError the engine raises on a runaway recursion as a stack overflow',
          fn: async () => {
            /**
             What the engine throws when a function calls itself without end.
             */
            const exhaustion = caught(function recurseForever(): void {
              (function again(): void {
                again();
              })();
            },);

            /**
             What an engine that ends the message with a full stop raises.
             */
            const stopped = new RangeError('Maximum call stack size exceeded.',);

            expect(isStackOverflow(exhaustion,),).toBe(true,);
            expect(isStackOverflow(stopped,),).toBe(true,);
          },
        },),
        it({
          name: 'READS any other RangeError, any other class with the same words and a bare string as no stack overflow',
          fn: async () => {
            /**
             A range failure of another kind.
             */
            const other = new RangeError('Invalid array length',);
            /**
             A failure of another class with the engine's words.
             */
            const wrongClass = new Error('Maximum call stack size exceeded',);

            expect(isStackOverflow(other,),).toBe(false,);
            expect(isStackOverflow(wrongClass,),).toBe(false,);
            expect(isStackOverflow('Maximum call stack size exceeded',),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);

//endregion Nesting bound tests
