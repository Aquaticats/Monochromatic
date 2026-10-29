/**
 Guards against a test file holding more than one top-level awaited
 describe (ledger B99). module-test's `describe` throws when a child fails,
 and a throw out of a top-level `await` stops the module there: no later
 top-level statement in the file runs, a later `await describe(...)`
 included. A file with two or more top-level awaited describes drops every
 suite after the first that fails, and nothing in the output tells that
 apart from a file whose one suite failed. A file with several suites wraps
 them under one root, `await describe({ name: '', concurrency: 1, children:
 [...] })`, so each runs and each failure is reported.

 WHAT THE SCAN READS, in files whose path ends in `.test.ts`: an
 `AwaitExpression` whose argument is a `describe(...)` call and whose every
 ancestor out to `Program` is an `ExpressionStatement`, a
 `VariableDeclarator`, a `VariableDeclaration` or the `Program`, the shape
 `global-writes-sequenced.unit.test.ts` reads as the suite awaited at the
 file's top. A file passes when it holds at most one.

 OUT OF THE SCAN'S REACH: a suite awaited inside a function the file's top
 calls, and a file that imports `describe` under another name. The words
 `await describe(` inside a string or a comment are no call, and the
 fixture case shows the scan passes them by parsing rather than matching
 text.

 THE FIXTURE CASE COMES FIRST, so the package-wide case is read against a
 scan shown able to tell one top-level suite from two. Fixtures are
 cat-themed; the package case reads this package's own tests.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { readPackageSource, } from './family-source.test-fixture.ts';
import {
  ancestorsOf,
  identifierName,
  isTreeNode,
  parentsOf,
  parseSource,
  type SourceText,
  type TreeNode,
  unwrapped,
} from './source-scan.test-fixture.ts';
import { expectNoFindings, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scan-findings.test-fixture.ts';

//region Top-level describe scan
// The scan: every AwaitExpression wrapping a describe(...) call, kept only
// when every ancestor out to Program is a bare top-level shape, counted per
// test file.

/**
 Node kinds a top-level awaited describe's ancestors must all be, out to
 `Program`.
 */
const TOP_STATEMENT_TYPES: ReadonlySet<string> = new Set([
  'ExpressionStatement',
  'VariableDeclarator',
  'VariableDeclaration',
  'Program',
],);

/**
 Whether a node awaits a `describe(...)` call at its file's top.

 @param node - node read

 @param parents - each node's parent in its file

 @returns Whether it is an await of a describe call held only by top-level
 statement shapes

 @example
 ```ts
 const atTop = awaitsDescribeAtTop({ node, parents, },);
 ```
 */
function awaitsDescribeAtTop(
  {
    node,
    parents,
  }: {
    readonly node: TreeNode;
    readonly parents: ReadonlyMap<TreeNode, TreeNode>;
  },
): boolean {
  if (node.type !== 'AwaitExpression')
    return false;
  /**
   The expression awaited, through any wrappers.
   */
  const argument = unwrapped({ node: node.argument, },).inner;
  if ((!isTreeNode(argument,)) || (argument.type !== 'CallExpression'))
    return false;
  if (identifierName({ node: argument.callee, },) !== 'describe')
    return false;
  return ancestorsOf({ node, parents, },).every(function isTopStatement(ancestor,): boolean {
    return TOP_STATEMENT_TYPES.has(ancestor.type,);
  },);
}

/**
 Paths of every `.test.ts` file holding more than one top-level awaited
 describe.

 @param files - files read; files whose path does not end in `.test.ts` are
 passed over

 @returns Paths sorted

 @example
 ```ts
 const found = multiSuiteTopLevelFiles({ files, },);
 ```
 */
function multiSuiteTopLevelFiles({ files, }: { readonly files: readonly SourceText[]; },): readonly string[] {
  return files
    .filter(function isTestFile({ path, },): boolean {
      return path.endsWith('.test.ts',);
    },)
    .filter(function holdsSeveralSuites(file,): boolean {
      /**
       Each node's parent in this file.
       */
      const parents = parentsOf({ program: parseSource({ file, },).program, },);
      return [...parents.keys(),].filter(function atTop(node,): boolean {
        return awaitsDescribeAtTop({ node, parents, },);
      },).length > 1;
    },)
    .map(function pathOf({ path, },): string {
      return path;
    },)
    .toSorted();
}

//endregion Top-level describe scan

/**
 A source file for the fixture case.

 @param path - file name, whose ending decides whether the scan reads it

 @param text - file text

 @returns Source file

 @example
 ```ts
 const file = fixture({ path: 'cat.test.ts', text: 'export const nap = 1;', },);
 ```
 */
function fixture(
  {
    path,
    text,
  }: {
    readonly path: string;
    readonly text: string;
  },
): SourceText {
  return {
    path,
    text,
    isTest: true,
  };
}

await describe({
  name: 'at most one top-level awaited describe per test file (ledger B99)',
  children: [
    it({
      name: 'FINDS a file with two top-level awaited describes, PASSES a file with one, passes over the '
        + 'words `await describe(` inside a comment or a string, which shows the scan parses rather than '
        + 'matches text, and READS NO FILE whose path does not end in `.test.ts`',
      fn: async () => {
        expect(multiSuiteTopLevelFiles({
          files: [
            fixture({
              path: 'napping-cats.test.ts',
              text: [
                'import { describe, it, } from \'@monochromatic-dev/test-harness-core/ts\';',
                'await describe({ name: \'napping\', children: [',
                '  it({ name: \'curls up\', fn: async () => {}, },),',
                '], },);',
                'await describe({ name: \'stretching\', children: [',
                '  it({ name: \'arches its back\', fn: async () => {}, },),',
                '], },);',
              ].join('\n',),
            },),
            fixture({
              path: 'purring-cats.test.ts',
              text: [
                'import { describe, it, } from \'@monochromatic-dev/test-harness-core/ts\';',
                '// a note that says await describe( to see whether the scan counts it',
                'const NOTE = \'await describe( inside a string, not a call\';',
                'await describe({ name: \'purring\', children: [',
                '  it({ name: \'rumbles\', fn: async () => {}, },),',
                '], },);',
              ].join('\n',),
            },),
            fixture({
              path: 'kibble.ts',
              text: [
                'await describe({ name: \'not a test file\', children: [], },);',
                'await describe({ name: \'still not a test file\', children: [], },);',
              ].join('\n',),
            },),
          ],
        },),).toEqual(['napping-cats.test.ts',],);
      },
    },),
    it({
      name: 'HOLDS NO FILE WITH MORE THAN ONE TOP-LEVEL AWAITED DESCRIBE in this package\'s tests',
      fn: async () => {
        /**
         Every package file, tests among them.
         */
        const files = await readPackageSource();
        expect(files.some(function isTestFile({ path, },): boolean {
          return path.endsWith('.test.ts',);
        },),).toBe(true,);
        expectNoFindings({ findings: multiSuiteTopLevelFiles({ files, },), },);
      },
    },),
  ],
},);
