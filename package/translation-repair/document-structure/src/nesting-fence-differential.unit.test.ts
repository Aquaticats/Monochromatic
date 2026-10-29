/**
 Tests that the nesting scan never skips a stretch the parser would nest,
 against the parser itself, over a generated family of texts: a line that may
 open a fence, in a context that may or may not make it one, with the
 indentation it may carry, closed or not, followed by text the parser would
 nest if it reads it as text.

 THE RULE UNDER TEST. The scan may refuse a text the parser would read, and
 each such case is stated beside the family, but it never passes a text in
 which the parser nests past the bound. The oracle is the grammar's own parser
 run without the scan: for every text the scan passes, the depth of the tree
 it builds stays under a limit and both entries parse the text within a
 stated time.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import remarkGfm from 'remark-gfm';
import remarkMath from 'remark-math';
import remarkMdx from 'remark-mdx';
import remarkParse from 'remark-parse';
import { unified, } from 'unified';

import {
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
} from '@monochromatic-dev/translation-repair-markup';
import {
  MarkdownParseError,
  parseMarkdownBody,
  parseMdxBody,
} from '../dist/final/node/index.mjs';

//region Nesting fence differential tests

/**
 Longest either entry may take to parse a text the scan passes, in
 milliseconds of the process's CPU time. Over three runs of the whole family
 on a machine shared with other builds, the slowest parse of a passed text
 took 6.5, 54.8 and 6.1 milliseconds of real time, the high one while other
 builds ran, so the limit is eighteen times the slowest ever seen; on the CPU
 clock, which a process taken off the CPU does not advance, a parse takes no
 longer than on the real one.
 */
const PARSE_TIME_LIMIT_MS = 1_000;

/**
 One line that may open a fence.
 */
type Opener = {
  readonly name: string;
  readonly line: string;
  readonly closer: string;
  readonly other: string;

  /**
   Whether the parser certainly reads the line as a fence where it begins a
   line outside every container and every raw block.
   */
  readonly certain: boolean;
};

/**
 What stands before the fence line and around it.
 */
type Context = {
  readonly name: string;
  readonly before: readonly string[];
  readonly opening: string;
  readonly inside: string;

  /**
   Grammars under which nothing before the fence line leaves a container, raw
   html, a math block or an expression open, so that the fence line, written
   without indentation, begins a fence the parser certainly reads.
   */
  readonly certainUnder: readonly ('markdown' | 'mdx')[];
};

/**
 The text the parser would nest, and the depth of tree it must stay under
 where the scan passes it.
 */
type Tail = {
  readonly name: string;
  readonly text: string;
  readonly depthLimit: number;
  readonly grammars: readonly ('markdown' | 'mdx')[];
};

/**
 Lines that may open a fence: a backtick fence, a tilde fence, a fence of four
 backticks and a backtick fence whose info string holds a backtick, which is
 no fence. Each names the line that closes it and a line of the other
 character, which closes nothing.
 */
const OPENERS: readonly Opener[] = [
  { name: 'a backtick fence', line: '```ts', closer: '```', other: '~~~', certain: true, },
  { name: 'a tilde fence', line: '~~~ts', closer: '~~~', other: '```', certain: true, },
  { name: 'a four-backtick fence', line: '````ts', closer: '````', other: '~~~~', certain: true, },
  {
    name: 'a backtick fence with a backtick in its info',
    line: '```a`b',
    closer: '```',
    other: '~~~',
    certain: false,
  },
];

/**
 Spaces and tabs the fence line may carry before it.
 */
const INDENTS: readonly string[] = ['', '   ', '    ', '\t',];

/**
 What the fence line may sit in or after.
 */
const CONTEXTS: readonly Context[] = [
  { name: 'at the start of the text', before: [], opening: '', inside: '', certainUnder: ['markdown', 'mdx',], },
  {
    name: 'right after a paragraph line',
    before: ['The cat naps.',],
    opening: '',
    inside: '',
    certainUnder: ['markdown', 'mdx',],
  },
  {
    name: 'after a blank line',
    before: ['The cat naps.', '',],
    opening: '',
    inside: '',
    certainUnder: ['markdown', 'mdx',],
  },
  { name: 'in a quotation', before: [], opening: '> ', inside: '> ', certainUnder: [], },
  { name: 'in a bullet item', before: [], opening: '- ', inside: '  ', certainUnder: [], },
  { name: 'in an ordered item', before: [], opening: '1. ', inside: '   ', certainUnder: [], },
  { name: 'in a footnote definition', before: [], opening: '[^1]: ', inside: '    ', certainUnder: [], },
  { name: 'right after an html line', before: ['<div>',], opening: '', inside: '', certainUnder: [], },
  { name: 'right after an open html comment', before: ['<!--',], opening: '', inside: '', certainUnder: [], },
  { name: 'right after an open pre block', before: ['<pre>',], opening: '', inside: '', certainUnder: [], },
  {
    name: 'right after a line opening a math block',
    before: ['$$',],
    opening: '',
    inside: '',
    certainUnder: ['markdown',],
  },
  {
    name: 'right after a line opening an expression',
    before: ['{',],
    opening: '',
    inside: '',
    certainUnder: ['markdown',],
  },
  { name: 'in a bullet item in a quotation', before: [], opening: '> - ', inside: '>   ', certainUnder: [], },
];

/**
 How the fence is closed before the text that follows, if it is.
 */
const CLOSINGS: readonly string[] = [
  'never closed',
  'closed on its own line',
  'closed by a line indented four columns',
  'closed by a shorter fence',
  'closed by a fence of the other character',
  'closed by a fence with text after it',
];

/**
 Closings after which the scan still reads the fence as open, so the tail is
 inside it. The scan ends a fence at a line of the same character, at least as
 long, with only blanks after it, at any indentation; so a line indented four
 columns ends it early (where the parser does not) and the tail after it is
 counted, the stated over-refusal of a closing line indented four columns.
 */
const CLOSINGS_LEAVING_THE_FENCE_OPEN: ReadonlySet<string> = new Set([
  'never closed',
  'closed by a shorter fence',
  'closed by a fence of the other character',
  'closed by a fence with text after it',
],);

/**
 Text a parser nests: emphasis opened 520 times, under both grammars, whose
 tree is more than half the delimiter bound deep, and 300 open tags under the
 strict one, whose tree is 300 deep.
 */
const TAILS: readonly Tail[] = [
  {
    name: 'emphasis opened 520 times',
    text: `${'*a '.repeat(520,)}cat${'*'.repeat(520,)}`,
    depthLimit: DELIMITER_BOUND / 2,
    grammars: ['markdown', 'mdx',],
  },
  {
    name: 'three hundred open tags',
    text: `${'<div>'.repeat(300,)}cat${'</div>'.repeat(300,)}`,
    depthLimit: NESTING_BOUND,
    grammars: ['mdx',],
  },
];

/**
 Lines of one member of the family: what stands before, the fence line in its
 context, a line of what it holds, how it is closed, and the tail on a line
 outside every container.

 @param opener - line that may open a fence

 @param indent - what stands before it

 @param context - what it sits in or after

 @param closing - how the fence is closed

 @param tail - text that follows

 @returns Text, lines joined by a line feed

 @example
 ```ts
 const text = memberText({ opener: OPENERS[0], indent: '', context: CONTEXTS[0], closing: 'never closed', tail: TAILS[0], },);
 ```
 */
function memberText(
  {
    opener,
    indent,
    context,
    closing,
    tail,
  }: {
    readonly opener: Opener;
    readonly indent: string;
    readonly context: Context;
    readonly closing: string;
    readonly tail: Tail;
  },
): string {
  /**
   The line that closes the fence, or fails to, as the closing says.
   */
  const closingLine = (closing === 'closed by a shorter fence')
    ? opener.closer.slice(1,)
    : (closing === 'closed by a fence of the other character')
      ? opener.other
      : (closing === 'closed by a fence with text after it')
        ? `${opener.closer} cat`
        : (closing === 'closed by a line indented four columns')
          ? `    ${opener.closer}`
          : opener.closer;
  /**
   Lines that close the fence, none for a fence never closed.
   */
  const closers = (closing === 'never closed')
    ? []
    : [`${context.inside}${closingLine}`, '',];
  return [
    ...context.before,
    `${context.opening}${indent}${opener.line}`,
    `${context.inside}code`,
    ...closers,
    tail.text,
  ].join('\n',);
}

/**
 Depth of the tree the grammar's own parser builds, with no scan before it.

 @param body - text to parse

 @param grammar - grammar to parse it under

 @returns Depth of the deepest chain of nodes, absent when the strict grammar
 refuses the text, which leaves no tree to measure

 @throws The caught value when the oracle's own stack is exhausted, or when
 the plain grammar's parser fails at all, since that grammar refuses no text
 and the depth would otherwise read the failure as no tree

 @example
 ```ts
 const depth = rawTreeDepth({ body: 'The cat naps.', grammar: 'markdown', },);
 ```
 */
function rawTreeDepth(
  {
    body,
    grammar,
  }: {
    readonly body: string;
    readonly grammar: 'markdown' | 'mdx';
  },
): readonly number[] {
  /**
   Parser for the grammar, with the extensions the entries use.
   */
  const processor = (grammar === 'mdx')
    ? unified()
      .use(remarkParse,)
      .use(remarkMdx,)
      .use(remarkGfm,)
      .use(remarkMath,)
    : unified()
      .use(remarkParse,)
      .use(remarkGfm,);
  try {
    /**
     Nodes still to visit, each with the depth it sits at.
     */
    const pending: [{ readonly children?: readonly unknown[]; }, number][] = [[processor.parse(body,), 1,],];
    /**
     Depth of every node visited.
     */
    const depths: number[] = [];
    for (let next = pending.pop(); next !== undefined; next = pending.pop()) {
      const [node, depth,] = next;
      depths.push(depth,);
      for (const child of node.children ?? []) {
        if (((typeof child) === 'object') && (child !== null))
          pending.push([child, depth + 1,],);
      }
    }
    return [Math.max(...depths,),];
  }
  catch (error) {
    // Only the strict grammar refuses a text, which leaves no tree to
    // measure; a failure under the plain grammar, and a stack exhausted in the
    // oracle under either, is no tree of depth zero.
    if ((grammar !== 'mdx') || (!Error.isError(error,)) || isStackOverflow(error,))
      throw error;
    return [];
  }
}

/**
 How long one entry takes to parse a text, and whether it refused.

 @param run - entry applied to the text

 @returns Milliseconds taken and the refusal the entry raised, if any

 @example
 ```ts
 const timed = timedParse({ run: function parse(): unknown { return parseMarkdownBody({ body, },); }, },);
 ```
 */
function timedParse({ run, }: { readonly run: () => unknown; },): {
  readonly milliseconds: number;
  readonly refusals: readonly unknown[];
} {
  // ON THE PROCESS'S CPU CLOCK, not the real one: a parse is synchronous
  // work, and the CPU time it took is what grows with the text, while a
  // loaded machine that takes the process off the CPU mid-parse stretches the
  // real time without bound (`mistake-prevention.md`, "Tests on the real
  // clock"). `process.cpuUsage` counts microseconds spent on the CPU, user and
  // system together.
  /**
   CPU reading before the parse.
   */
  const started = process.cpuUsage();
  try {
    run();
    return { milliseconds: cpuMillisecondsSince({ started, },), refusals: [], };
  }
  catch (error) {
    return { milliseconds: cpuMillisecondsSince({ started, },), refusals: [error,], };
  }
}

/**
 Milliseconds of CPU the process has spent since a reading.

 @param started - reading taken before the work measured

 @returns User and system time together, in milliseconds

 @example
 ```ts
 const spent = cpuMillisecondsSince({ started: process.cpuUsage(), },);
 ```
 */
function cpuMillisecondsSince({ started, }: { readonly started: NodeJS.CpuUsage; },): number {
  /**
   CPU spent since the reading, in microseconds.
   */
  const spent = process.cpuUsage(started,);
  return (spent.user + spent.system) / 1_000;
}

/**
 What the family came to for one tail and grammar.
 */
type Tally = {
  readonly members: number;

  /**
   Members the scan passed, by name, in the family's order.
   */
  readonly passed: readonly string[];

  /**
   Members whose tail the scan's stated rule skips, by name, in the family's
   order: a certain opener, written without indentation, in a context that
   leaves nothing open under the grammar, with a closing after which the scan
   still reads the fence as open.
   */
  readonly skippedByRule: readonly string[];
  readonly deeperThanLimit: readonly string[];
  readonly tooSlow: readonly string[];
  readonly markdownRefusals: number;
};

/**
 Reads every member of the family against one tail under one grammar.

 @param tail - text that follows the fence line

 @param grammar - grammar the scan and the oracle read under

 @returns How many members there are, which the scan passed and which its
 stated rule skips, which passed members the parser nests past the limit or
 parses slowly, and how many the plain entry refused for its nesting

 @example
 ```ts
 const tally = tallyFamily({ tail: TAILS[0], grammar: 'markdown', },);
 ```
 */
function tallyFamily(
  {
    tail,
    grammar,
  }: {
    readonly tail: Tail;
    readonly grammar: 'markdown' | 'mdx';
  },
): Tally {
  /**
   Every member of the family, named, with its text.
   */
  const family = OPENERS.flatMap(function byOpener(opener,) {
    return INDENTS.flatMap(function byIndent(indent,) {
      return CONTEXTS.flatMap(function byContext(context,) {
        return CLOSINGS.map(function byClosing(closing,) {
          return {
            name: `${opener.name} ${JSON.stringify(indent,)} ${context.name}, ${closing}`,
            body: memberText({
              opener,
              indent,
              context,
              closing,
              tail,
            },),
            skippedByRule: opener.certain
              && (indent === '')
              && context.certainUnder.includes(grammar,)
              && CLOSINGS_LEAVING_THE_FENCE_OPEN.has(closing,),
          };
        },);
      },);
    },);
  },);
  /**
   Members the scan passes.
   */
  const passed = family.filter(function isPassed(member,): boolean {
    return firstNestingExcess({ body: member.body, grammar, },).kind === 'within';
  },);
  /**
   What each passed member's tree and both entries came to.
   */
  const measured = passed.map(function measure(member,) {
    const [depth = 0,] = rawTreeDepth({ body: member.body, grammar, },);
    return {
      name: member.name,
      depth,
      timed: [
        timedParse({ run: function plain(): unknown { return parseMarkdownBody({ body: member.body, },); }, },),
        timedParse({ run: function strict(): unknown { return parseMdxBody({ body: member.body, },); }, },),
      ],
    };
  },);
  /**
   Every parse of every passed member.
   */
  const parses = measured.flatMap(function timedOf(member,) {
    return member.timed;
  },);
  return {
    members: family.length,
    passed: passed.map(function nameOf(member,): string {
      return member.name;
    },),
    skippedByRule: family
      .filter(function isSkipped(member,): boolean {
        return member.skippedByRule;
      },)
      .map(function nameOf(member,): string {
        return member.name;
      },),
    deeperThanLimit: measured
      .filter(function isDeep(member,): boolean {
        return member.depth > tail.depthLimit;
      },)
      .map(function nameOf(member,): string {
        return member.name;
      },),
    tooSlow: measured
      .filter(function isSlow(member,): boolean {
        return member.timed.some(function overLimit(entry,): boolean {
          return entry.milliseconds > PARSE_TIME_LIMIT_MS;
        },);
      },)
      .map(function nameOf(member,): string {
        return member.name;
      },),
    markdownRefusals: parses
      .flatMap(function refusalsOf(entry,): readonly unknown[] {
        return entry.refusals;
      },)
      .filter(function isPlainRefusal(refusal,): boolean {
        return refusal instanceof MarkdownParseError;
      },)
      .length,
  };
}

/**
 One shape of text that may or may not hold a fenced block, with what the
 scan must say of the nesting that follows it under each grammar.
 */
type Shape = {
  /**
   The shape in words, for the case name.
   */
  readonly phrase: string;

  /**
   The text of the shape with a tail after it.
   */
  readonly text: (tail: string,) => string;

  /**
   Whether the parser nests the tail, so the scan must refuse it, or reads it
   as code, so the scan may skip it where it is certain and refuses it
   otherwise, which is the stated over-refusal.
   */
  readonly follows: 'nests' | 'code';

  /**
   What the scan says under the plain grammar.
   */
  readonly markdown: 'within' | 'beyond';

  /**
   What the scan says under the strict grammar.
   */
  readonly mdx: 'within' | 'beyond';
};

/**
 Shapes the review named and the others the grammar gives. The parser nests
 the tail of every shape marked so under the plain grammar (read off the tree
 of the grammar's own parser on the base build, where the scan passed each),
 and reads the tail of the others as code. Three of the others are refused
 all the same, which is the scan's stated over-refusal: a fence indented one
 to three columns, a closing line indented four columns, and a fence line in
 a raw html comment or a pre block.
 */
const SHAPES: readonly Shape[] = [
  {
    phrase: 'a backtick fence at the start of the text, never closed',
    text: (tail,) => `\`\`\`js\ncode\n${tail}\n`,
    follows: 'code',
    markdown: 'within',
    mdx: 'within',
  },
  {
    phrase: 'a backtick fence whose info string holds a backtick, which is no fence',
    text: (tail,) => `\`\`\`a\`b\n${tail}\n`,
    follows: 'nests',
    markdown: 'beyond',
    mdx: 'beyond',
  },
  {
    phrase: 'a backtick fence whose meta holds a backtick, which is no fence',
    text: (tail,) => `\`\`\`a b\`c\n${tail}\n`,
    follows: 'nests',
    markdown: 'beyond',
    mdx: 'beyond',
  },
  {
    phrase: 'a tilde fence whose info string holds a backtick, which is a fence',
    text: (tail,) => `~~~a\`b\n${tail}\n`,
    follows: 'code',
    markdown: 'within',
    mdx: 'within',
  },
  {
    phrase: 'a fence indented four columns, which is indented code',
    text: (tail,) => `    \`\`\`\n${tail}\n`,
    follows: 'nests',
    markdown: 'beyond',
    mdx: 'beyond',
  },
  {
    phrase: 'a fence indented five columns',
    text: (tail,) => `     \`\`\`\n${tail}\n`,
    follows: 'nests',
    markdown: 'beyond',
    mdx: 'beyond',
  },
  {
    phrase: 'a fence indented by a tab',
    text: (tail,) => `\t\`\`\`\n${tail}\n`,
    follows: 'nests',
    markdown: 'beyond',
    mdx: 'beyond',
  },
  {
    phrase: 'a fence indented three columns, which is a fence',
    text: (tail,) => `   \`\`\`\n${tail}\n`,
    follows: 'code',
    markdown: 'beyond',
    mdx: 'beyond',
  },
  {
    phrase: 'a fence in a quotation that the next line leaves',
    text: (tail,) => `> \`\`\`\n> code\n${tail}\n`,
    follows: 'nests',
    markdown: 'beyond',
    mdx: 'beyond',
  },
  {
    phrase: 'a fence in a quotation whose next line continues it lazily',
    text: (tail,) => `> \`\`\`\ncode\n${tail}\n`,
    follows: 'nests',
    markdown: 'beyond',
    mdx: 'beyond',
  },
  {
    phrase: 'a fence in a bullet item that the next line leaves',
    text: (tail,) => `- \`\`\`\n  code\n${tail}\n`,
    follows: 'nests',
    markdown: 'beyond',
    mdx: 'beyond',
  },
  {
    phrase: 'a fence in an ordered item that the next line leaves',
    text: (tail,) => `1. \`\`\`\n   code\n${tail}\n`,
    follows: 'nests',
    markdown: 'beyond',
    mdx: 'beyond',
  },
  {
    phrase: 'a fence indented two columns inside a bullet item, the next line unindented',
    text: (tail,) => `- a\n  \`\`\`\n  code\n${tail}\n`,
    follows: 'nests',
    markdown: 'beyond',
    mdx: 'beyond',
  },
  {
    phrase: 'a fence line right after a paragraph line, which interrupts the paragraph',
    text: (tail,) => `The cat naps.\n\`\`\`\n${tail}\n`,
    follows: 'code',
    markdown: 'within',
    mdx: 'within',
  },
  {
    phrase: 'a tilde fence with a line of backticks in it',
    text: (tail,) => `~~~\n\`\`\`\n${tail}\n`,
    follows: 'code',
    markdown: 'within',
    mdx: 'within',
  },
  {
    phrase: 'a backtick fence with a line of tildes in it',
    text: (tail,) => `\`\`\`\n~~~\n${tail}\n`,
    follows: 'code',
    markdown: 'within',
    mdx: 'within',
  },
  {
    phrase: 'a fence of four backticks with a line of three in it',
    text: (tail,) => `\`\`\`\`\ncode\n\`\`\`\n${tail}\n`,
    follows: 'code',
    markdown: 'within',
    mdx: 'within',
  },
  {
    phrase: 'a fence with a closing line that has text after the fence',
    text: (tail,) => `\`\`\`\ncode\n\`\`\` cat\n${tail}\n`,
    follows: 'code',
    markdown: 'within',
    mdx: 'within',
  },
  {
    phrase: 'a fence with a closing line indented four columns, which does not close it',
    text: (tail,) => `\`\`\`\ncode\n    \`\`\`\n${tail}\n`,
    follows: 'code',
    markdown: 'beyond',
    mdx: 'beyond',
  },
  {
    phrase: 'a fence closed on its own line, a blank line and the tail',
    text: (tail,) => `\`\`\`\ncode\n\`\`\`\n\n${tail}\n`,
    follows: 'nests',
    markdown: 'beyond',
    mdx: 'beyond',
  },
  {
    phrase: 'a fence line in an html block, which ends at a blank line',
    text: (tail,) => `<div>\n\`\`\`\n\n${tail}\n`,
    follows: 'nests',
    markdown: 'beyond',
    mdx: 'beyond',
  },
  {
    phrase: 'a fence line in an html comment, which ends at its marker',
    text: (tail,) => `<!--\n\`\`\`\n\n${tail}\n-->\n`,
    follows: 'code',
    markdown: 'beyond',
    mdx: 'beyond',
  },
  {
    phrase: 'a fence line in a pre block, which ends at its closing tag',
    text: (tail,) => `<pre>\n\`\`\`\n\n${tail}\n</pre>\n`,
    follows: 'code',
    markdown: 'beyond',
    mdx: 'beyond',
  },
  {
    phrase: 'a fence line in a math block',
    text: (tail,) => `$$\n\`\`\`\n$$\n\n${tail}\n`,
    follows: 'nests',
    markdown: 'within',
    mdx: 'beyond',
  },
  {
    phrase: 'a fence line in a multi-line expression',
    text: (tail,) => `{\n\`\`\`\n}\n\n${tail}\n`,
    follows: 'nests',
    markdown: 'within',
    mdx: 'beyond',
  },
  {
    phrase: 'a fence line in a footnote definition',
    text: (tail,) => `[^1]: \`\`\`\n    code\n${tail}\n`,
    follows: 'nests',
    markdown: 'beyond',
    mdx: 'beyond',
  },
  {
    phrase: 'a fence line in an indented code block',
    text: (tail,) => `    code\n    \`\`\`\n\n${tail}\n`,
    follows: 'nests',
    markdown: 'beyond',
    mdx: 'beyond',
  },
];

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: firstNestingExcess.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: TAILS.flatMap(function casesOf(tail,) {
        return tail.grammars.map(function caseOf(grammar,) {
          return it({
            name: `PASSES no text of the generated family in which the parser nests ${tail.name} past its limit, `
              + `parses every text it passes in time, and passes exactly the texts its stated rule skips, under the `
              + `${grammar} grammar`,
            fn: async () => {
              const tally = tallyFamily({ tail, grammar, },);

              expect(tally.members,).toBe(OPENERS.length * INDENTS.length * CONTEXTS.length * CLOSINGS.length,);
              expect(tally.deeperThanLimit,).toEqual([],);
              expect(tally.tooSlow,).toEqual([],);
              expect(tally.markdownRefusals,).toBe(0,);
              expect(tally.passed,).toEqual(tally.skippedByRule,);
            },
          },);
        },);
      },),
    },),

    describe({
      name: 'firstNestingExcess over named shapes',
      concurrency: DEFAULT_CONCURRENCY,
      children: SHAPES.map(function caseOf(shape,) {
        /**
         What the scan does under the plain grammar, in the case name's words.
         */
        const plainVerb = (shape.markdown === 'beyond') ? 'REFUSES' : 'SKIPS';
        /**
         What the scan does under the strict grammar, in the case name's words.
         */
        const strictVerb = (shape.mdx === 'beyond') ? 'refuses' : 'skips';
        /**
         What the parser does with the tail, in the case name's words.
         */
        const parserDoes = (shape.follows === 'nests') ? 'nests it' : 'reads it as code';
        return it({
          name: `${plainVerb} what follows ${shape.phrase} under the plain grammar, and ${strictVerb} it under the `
            + `strict one, where the parser ${parserDoes}`,
          fn: async () => {
            /**
             Emphasis opened 520 times after the shape.
             */
            const [emphasis, strictTags,] = TAILS;
            expect(firstNestingExcess({
              body: shape.text(emphasis?.text ?? '',),
              grammar: 'markdown',
            },).kind,).toBe(shape.markdown,);
            expect(firstNestingExcess({
              body: shape.text(emphasis?.text ?? '',),
              grammar: 'mdx',
            },).kind,).toBe(shape.mdx,);
            expect(firstNestingExcess({
              body: shape.text(strictTags?.text ?? '',),
              grammar: 'mdx',
            },).kind,).toBe(shape.mdx,);
          },
        },);
      },),
    },),
  ],
},);

//endregion Nesting fence differential tests
