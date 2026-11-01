/**
 Guards against reading a passage's blocks by splitting it on blank lines
 (ledger B68, B100). The floor reads blocks off the parse, and a blank-line
 split disagrees with it wherever a fence holds a blank line, a list is
 loose, or a block opens on the line after another with no blank line
 between; seven readers counted, paired or anchored blocks that way and each
 answered differently from the floor.

 WHAT THE SCAN READS, in the package's source (tests and fixtures left out): a
 call to `split`, `indexOf`, `lastIndexOf` or `includes` whose first argument
 is a string holding a blank line (`'\n\n'`), a constant the file declares
 with such a string, or a regular expression matching two line feeds. Joining
 with a blank line writes text and is not read. Out of its reach: a reader
 that walks lines and treats an empty one as a boundary, and a blank line
 built at run time.

 THE FIXTURE CASE COMES FIRST, so the package-wide case is read against a
 scan shown able to find each form (ledger M21). Fixtures are cat-themed; the
 package case reads this package's own source.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { readPackageSource, } from './family-source.test-fixture.ts';
import {
  identifierName,
  isTreeNode,
  memberName,
  nodesUnder,
  parseSource,
  type SourceText,
  type TreeNode,
} from './source-scan.test-fixture.ts';
import { expectNoFindings, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scan-findings.test-fixture.ts';

//region Blank-line splits
// The scan: string searches and splits handed a blank line, keyed by file,
// line and member.

/**
 String members that search or split by their first argument.
 */
const SEARCH_MEMBERS: ReadonlySet<string> = new Set([
  'includes',
  'indexOf',
  'lastIndexOf',
  'split',
],);

/**
 A blank line as a string holds it.
 */
const BLANK_LINE = '\n\n';

/**
 Regular-expression sources that match two line feeds with at most
 whitespace between.
 */
const BLANK_LINE_PATTERNS: readonly string[] = [
  String.raw`\n\n`,
  String.raw`\n\s*\n`,
  String.raw`\n{2`,
  String.raw`\n[ \t]*\n`,
];

/**
 Whether a node is a string literal holding a blank line.

 @param node - node read

 @returns Whether its value is a string with two line feeds in a row

 @example
 ```ts
 const blank = holdsBlankLine({ node: argument, },);
 ```
 */
function holdsBlankLine({ node, }: { readonly node: TreeNode; },): boolean {
  /**
   The literal's value, of whatever kind.
   */
  const { value, } = node;
  return (node.type === 'Literal')
    && ((typeof value) === 'string')
    && value.includes(BLANK_LINE,);
}

/**
 Whether a node is a regular expression matching two line feeds.

 @param node - node read

 @returns Whether its pattern holds one of the blank-line sources

 @example
 ```ts
 const blank = matchesBlankLine({ node: argument, },);
 ```
 */
function matchesBlankLine({ node, }: { readonly node: TreeNode; },): boolean {
  /**
   The literal's regular expression, absent on any other literal.
   */
  const { regex, } = node;
  if ((node.type !== 'Literal') || ((typeof regex) !== 'object') || (regex === null) || (!('pattern' in regex)))
    return false;
  /**
   The expression's source.
   */
  const pattern = String(regex.pattern,);
  return BLANK_LINE_PATTERNS.some(function inPattern(source,): boolean {
    return pattern.includes(source,);
  },);
}

/**
 Names a file declares with a string holding a blank line.

 @param nodes - every node of the file

 @returns Their names

 @example
 ```ts
 const names = blankLineConstants({ nodes, },);
 ```
 */
function blankLineConstants({ nodes, }: { readonly nodes: readonly TreeNode[]; },): ReadonlySet<string> {
  return new Set(nodes
    .filter(function declaresBlankLine(node,): boolean {
      return (node.type === 'VariableDeclarator')
        && isTreeNode(node.init,)
        && holdsBlankLine({ node: node.init, },);
    },)
    .map(function nameOf(node,): string {
      return identifierName({ node: node.id, },);
    },),);
}

/**
 Every search or split one production file hands a blank line, keyed
 `path:line member`.

 @param file - file read

 @returns Its keys, in source order

 @example
 ```ts
 const found = blankLineSplitsIn({ file, },);
 ```
 */
function blankLineSplitsIn({ file, }: { readonly file: SourceText; },): readonly string[] {
  /**
   Every node of the file.
   */
  const nodes = nodesUnder({ root: parseSource({ file, },).program, },);
  /**
   Constants the file declares with a blank line.
   */
  const constants = blankLineConstants({ nodes, },);
  return nodes
    .flatMap(function searchOf(node,): readonly {
      readonly start: number;
      readonly member: string;
    }[] {
      if ((node.type !== 'CallExpression') || (!isTreeNode(node.callee,)) || (node.callee.type !== 'MemberExpression'))
        return [];
      /**
       The member the call names.
       */
      const member = memberName({ node: node.callee, },);
      /**
       What the call is passed.
       */
      const args: readonly unknown[] = Array.isArray(node.arguments,) ? node.arguments : [];
      /**
       The call's first argument.
       */
      const [argument,] = args;
      return (SEARCH_MEMBERS.has(member,) && isTreeNode(argument,)
        && (holdsBlankLine({ node: argument, },)
          || matchesBlankLine({ node: argument, },)
          || constants.has(identifierName({ node: argument, },),)))
        ? [{
          start: node.start,
          member,
        },]
        : [];
    },)
    .toSorted(function bySource(
      left,
      right,
    ): number {
      return left.start - right.start;
    },)
    .map(function keyOf({
      start,
      member,
    },): string {
      /**
       Line the call starts on.
       */
      const line = file.text
        .slice(
        0,
        start,
        )
        .split('\n',)
        .length;
      return `${file.path}:${String(line,)} ${member}`;
    },);
}

/**
 Every blank-line search or split in the package's source files.

 @param files - files read; tests and fixtures are passed over

 @returns Keys in file order

 @example
 ```ts
 const found = blankLineSplits({ files, },);
 ```
 */
function blankLineSplits({ files, }: { readonly files: readonly SourceText[]; },): readonly string[] {
  return files
    .filter(function isSource({ isTest, },): boolean {
      return !isTest;
    },)
    .flatMap(function splitsIn(file,): readonly string[] {
      return blankLineSplitsIn({ file, },);
    },);
}

//endregion Blank-line splits

await describe({
  name: 'reading blocks by splitting on blank lines (ledger B68, B100)',
  children: [
    it({
      name: 'FINDS a split on a blank-line string, an indexOf on a constant holding one, a split on a '
        + 'pattern matching two line feeds and an includes of a string holding one; PASSES a join with a '
        + 'blank line, a split on one line feed, and a test file',
      fn: async () => {
        expect(blankLineSplits({
          files: [
            {
              path: 'naps.ts',
              isTest: false,
              text: [
                String.raw`const NAP_BREAK = '\n\n';`,
                String.raw`export const naps = (text: string) => text.split('\n\n',);`,
                'export const first = (text: string) => text.indexOf(NAP_BREAK,);',
                String.raw`export const loose = (text: string) => text.split(/\n\s*\n/u,);`,
                String.raw`export const joined = (naps: string[]) => naps.join('\n\n',);`,
                String.raw`export const lines = (text: string) => text.split('\n',);`,
                String.raw`export const purrs = (text: string) => text.includes('purr\n\npurr',);`,
              ].join('\n',),
            },
            {
              path: 'naps.unit.test.ts',
              isTest: true,
              text: String.raw`export const naps = (text: string) => text.split('\n\n',);`,
            },
          ],
        },),).toStrictEqual([
          'naps.ts:2 split',
          'naps.ts:3 indexOf',
          'naps.ts:4 split',
          'naps.ts:7 includes',
        ],);
      },
    },),
    it({
      name: 'FINDS NO BLANK-LINE SPLIT in this package\'s source',
      fn: async () => {
        /**
         Every package file.
         */
        const files = await readPackageSource();
        expect(files.length,).toBeGreaterThan(0,);
        expectNoFindings({ findings: blankLineSplits({ files, },), },);
      },
    },),
  ],
},);
