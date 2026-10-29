/**
 Guards against a TSDoc block (`/**`, excluding the trivial `/**\/`) that
 sits apart from the thing it documents (ledger B19 audit area six). A block
 comment documents whatever its own text claims to, not whatever source
 happens to follow it, so a TSDoc left behind by a move, or separated from
 its declaration by a second comment, reads as documentation for the wrong
 thing, or for nothing. Repository rule TD2 requires a TSDoc directly before
 the declaration it documents.

 WHAT THE SCAN READS, in the package's source and its tests alike: every
 TSDoc block, and the first non-whitespace text after it. That text attaches
 the block when it begins a declaration (function, class, variable,
 `using`, `await using`, type alias, interface, enum, or an export of one;
 an import never does), a class member, an object or type-literal property
 or method, an enum member, a union-type member (`| /** doc *\/ 'a'`, on
 either side of its own `|`), or a named function expression returned or
 passed as a bare value. A file-header block carrying the `@module` tag is
 exempt wherever it sits.

 OUT OF THE SCAN'S REACH: nothing else interposed between a TSDoc and its
 declaration is ever legitimate, including a lint-disable comment; the
 repository rule for a suppressed declaration puts the disable comment
 before the TSDoc, never between it and the declaration, so this scan never
 special-cases one.

 THE FIXTURE CASE COMES FIRST, so the package-wide case is read against a
 scan shown able to find each orphan shape and leave each legitimate one
 (ledger M21). Fixtures are cat-themed; the package case reads this
 package's own source.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { readPackageSource, } from './family-source.test-fixture.ts';
import {
  isTreeNode,
  nodesUnder,
  parseSource,
  type SourceText,
  type TreeNode,
} from './source-scan.test-fixture.ts';
import { expectNoFindings, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scan-findings.test-fixture.ts';

//region Tsdoc attachment
// The scan: which AST node kinds a TSDoc block can document, the offset its
// next non-whitespace text starts at, and what sits there when it cannot
// attach.

/**
 Node kinds a TSDoc block can document as a declaration, an export of one
 among them.
 */
const DECLARATION_KINDS: ReadonlySet<string> = new Set([
  'ClassDeclaration',
  'ExportDefaultDeclaration',
  'ExportNamedDeclaration',
  'FunctionDeclaration',
  'TSEnumDeclaration',
  'TSInterfaceDeclaration',
  'TSTypeAliasDeclaration',
  'VariableDeclaration',
],);

/**
 Node kinds a TSDoc block can document as a member: a class member, an
 object or type-literal property or method, an enum member, or a
 union-type member.
 */
const MEMBER_KINDS: ReadonlySet<string> = new Set([
  'MethodDefinition',
  'Property',
  'PropertyDefinition',
  'TSEnumMember',
  'TSLiteralType',
  'TSMethodSignature',
  'TSPropertySignature',
],);

/**
 Node kinds a TSDoc block can document as a bare value: a function
 expression passed where it stands.
 */
const VALUE_KINDS: ReadonlySet<string> = new Set([
  'ArrowFunctionExpression',
  'FunctionExpression',
],);

/**
 Whether a node is the kind of thing a TSDoc block can document.

 @param node - node the comment's next non-whitespace text began

 @returns Whether the comment attaches to it

 @example
 ```ts
 const attached = isDocumentable({ node, },); // true for a FunctionDeclaration
 ```
 */
function isDocumentable({ node, }: { readonly node: TreeNode; },): boolean {
  if (DECLARATION_KINDS.has(node.type,) || MEMBER_KINDS.has(node.type,) || VALUE_KINDS.has(node.type,))
    return true;
  if (node.type !== 'ReturnStatement')
    return false;
  /**
   What the return statement hands back.
   */
  const { argument, } = node;
  return isTreeNode(argument,) && (argument.type === 'FunctionExpression');
}

/**
 Offset of the first non-whitespace character at or after a position, the
 text's own length when there is none.

 @param text - text read

 @param from - offset searched from

 @returns Offset of the first non-whitespace character

 @example
 ```ts
 const offset = firstNonWhitespace({ text, from: comment.end, },);
 ```
 */
function firstNonWhitespace({ text, from, }: { readonly text: string; readonly from: number; },): number {
  /**
   The remainder, with its own leading whitespace trimmed.
   */
  const rest = text.slice(from,);
  return from + (rest.length - rest.trimStart().length);
}

/**
 Line a text offset falls on.

 @param text - text read

 @param offset - offset read

 @returns One-based line number

 @example
 ```ts
 const line = lineOf({ text, offset: comment.start, },); // 1 at the file's first character
 ```
 */
function lineOf({ text, offset, }: { readonly text: string; readonly offset: number; },): number {
  return text.slice(0, offset,).split('\n',).length;
}

/**
 What sits where an orphan TSDoc's declaration should begin, named for the
 failure list.

 @param file - file the comment sits in

 @param comment - orphan TSDoc's own range

 @param offset - first non-whitespace offset after the comment

 @param comments - every comment in the file, this one included

 @param nodes - every node in the file, the program's own node excluded

 @returns Phrase naming what the comment sits before

 @example
 ```ts
 const shape = orphanShape({ file, comment, offset, comments, nodes, },); // 'before a statement'
 ```
 */
function orphanShape(
  {
    file,
    comment,
    offset,
    comments,
    nodes,
  }: {
    readonly file: SourceText;
    readonly comment: { readonly start: number; readonly end: number; };
    readonly offset: number;
    readonly comments: readonly { readonly start: number; readonly end: number; }[];
    readonly nodes: readonly TreeNode[];
  },
): string {
  /**
   Another comment sitting where the declaration should start, if any.
   */
  const interposed = comments.find(function sitsBetween(candidate,): boolean {
    return (candidate !== comment) && (candidate.start >= comment.end) && (candidate.start <= offset);
  },);
  if (interposed !== undefined) {
    /**
     The interposed comment's own text.
     */
    const interposedText = file.text.slice(interposed.start, interposed.end,);
    if (interposedText.startsWith('/**',) && (interposedText !== '/**/'))
      return 'before a second TSDoc comment';
    if (interposedText.includes('disable-next-line',))
      return 'before a disable-next-line comment';
    if (interposedText.startsWith('//',))
      return 'before a // comment';
    return 'before a lint-disable comment';
  }
  /**
   Widest node starting exactly at the offset: the statement itself over any
   node nested inside it.
   */
  const widest = nodes
    .filter(function startsHere(node,): boolean { return node.start === offset; },)
    .toSorted(function widestFirst(left, right,): number { return (right.end - right.start) - (left.end - left.start); },)
    .at(0,);
  if (widest === undefined)
    return 'before end of file';
  if (widest.type === 'ImportDeclaration')
    return 'before an import';
  if (widest.type.endsWith('Statement',))
    return 'before a statement';
  return `before ${widest.type}`;
}

/**
 Every TSDoc block in the files given whose next non-whitespace text does
 not begin the thing it documents, a file header's `@module` block always
 excepted.

 @param files - files read, tests among them

 @returns `path:line: shape` entries, sorted

 @example
 ```ts
 const orphans = orphanTsdocEntries({ files, },);
 ```
 */
function orphanTsdocEntries({ files, }: { readonly files: readonly SourceText[]; },): readonly string[] {
  /**
   Entries found so far.
   */
  const found: string[] = [];
  for (const file of files) {
    /**
     Program and comments this file parsed to.
     */
    const { program, comments, } = parseSource({ file, },);
    /**
     Every node under the program, the program's own node excluded since its
     `start` can coincide with its first body statement's.
     */
    const nodes = nodesUnder({ root: program, },)
      .filter(function notProgram(node,): boolean { return node.type !== 'Program'; },);
    for (const comment of comments) {
      /**
       The comment's own text.
       */
      const text = file.text.slice(comment.start, comment.end,);
      if ((!text.startsWith('/**',)) || (text === '/**/') || text.includes('@module',))
        continue;
      /**
       Offset of the first non-whitespace character after the comment.
       */
      const offset = firstNonWhitespace({ text: file.text, from: comment.end, },);
      /**
       Whether the offset falls inside a union type's span: a member after
       the first carries no node start of its own (only the union as a
       whole starts at its first `|`), so it is found by containment.
       */
      const insideUnion = nodes.some(function unionContains(node,): boolean {
        return (node.type === 'TSUnionType') && (node.start <= offset) && (offset < node.end);
      },);
      /**
       Whether any node starting exactly at the offset is documentable.
       */
      const attached = insideUnion || nodes.some(function matchesHere(node,): boolean {
        return (node.start === offset) && isDocumentable({ node, },);
      },);
      if (attached)
        continue;
      found.push(`${file.path}:${lineOf({ text: file.text, offset: comment.start, },)}: ${orphanShape({
        file,
        comment,
        offset,
        comments,
        nodes,
      },)}`,);
    }
  }
  return found.toSorted();
}

//endregion Tsdoc attachment

await describe({
  name: 'tsdoc attachment (ledger B19)',
  children: [
    it({
      name: 'FINDS a TSDoc before a second TSDoc, before a // comment, before a statement, before a '
        + 'disable-next-line comment and before an import, and leaves a declaration, an export of one, a class '
        + 'member, an object or type-literal property or method, an enum member, a union member on either side of '
        + 'its own `|`, a `using` or `await using` declaration, a returned named function expression and an '
        + '`@module` header wherever it sits',
      fn: async () => {
        /**
         Fixture file text, as a cat-themed module would write it.
         */
        const litter = [
          '/**',
          ' Module header for the litter file.',
          '',
          ' @module',
          ' */',
          '',
          'import { nap, } from \'./nap.ts\';',
          '',
          '/**',
          ' Orphan: before a second TSDoc comment.',
          ' */',
          '/**',
          ' The real doc for the cat.',
          ' */',
          'export const cat = 1;',
          '',
          '/**',
          ' Orphan: before a // comment.',
          ' */',
          '// a plain prose comment',
          'export const kitten = 2;',
          '',
          '/**',
          ' Orphan: before a statement.',
          ' */',
          'if (cat === 1) {',
          '  void kitten;',
          '}',
          '',
          '/**',
          ' Orphan: before a disable-next-line comment.',
          ' */',
          '/* oxlint-disable-next-line no-regex -- old unfixed shape */',
          'export const tabby = /tabby/u;',
          '',
          '/**',
          ' Legit: function declaration.',
          ' */',
          'export function purr() {',
          '  return 1;',
          '}',
          '',
          '/**',
          ' Legit: class declaration.',
          ' */',
          'export class Cat {',
          '  /**',
          '   Legit: class field.',
          '   */',
          '  whiskers = 1;',
          '',
          '  /**',
          '   Legit: class method.',
          '   */',
          '  meow() { return \'meow\'; }',
          '}',
          '',
          '/**',
          ' Legit: object property and method.',
          ' */',
          'export const bowl = {',
          '  /**',
          '   Legit: property.',
          '   */',
          '  food: \'fish\',',
          '  /**',
          '   Legit: method.',
          '   */',
          '  fill() { return true; },',
          '};',
          '',
          '/**',
          ' Legit: interface member.',
          ' */',
          'export interface Toy {',
          '  /**',
          '   Legit: interface property.',
          '   */',
          '  squeaks: boolean;',
          '}',
          '',
          '/**',
          ' Legit: type literal member.',
          ' */',
          'export type Bed = {',
          '  /**',
          '   Legit: type-literal property.',
          '   */',
          '  soft: boolean;',
          '};',
          '',
          '/**',
          ' Legit: enum, enum member.',
          ' */',
          'export enum Mood {',
          '  /**',
          '   Legit: enum member.',
          '   */',
          '  Napping,',
          '  Hunting,',
          '}',
          '',
          'export type Toy2 =',
          '  | /** Legit: union member before the first. */ \'ball\'',
          '  /**',
          '   Legit: union member between pipes.',
          '   */',
          '  | \'mouse\';',
          '',
          '/**',
          ' Legit: using declaration.',
          ' */',
          'using box = openBox();',
          '',
          '/**',
          ' Legit: await using declaration.',
          ' */',
          'await using carrier = openCarrier();',
          '',
          'export function wrapsNamedFunction() {',
          '  /**',
          '   Legit: returned named function expression.',
          '   */',
          '  return function named() { return 1; };',
          '}',
          '',
          '/**',
          ' Legit mid-file module note, exempt by its own @module tag.',
          '',
          ' @module',
          ' */',
          '',
          '/**',
          ' Orphan: before an import, which this rule never documents.',
          ' */',
          'import { collar, } from \'./collar.ts\';',
          'void collar;',
        ].join('\n',);
        expect(orphanTsdocEntries({
          files: [
            {
              path: 'litter.ts',
              text: litter,
              isTest: false,
            },
          ],
        },),).toEqual([
          'litter.ts:133: before an import',
          'litter.ts:17: before a // comment',
          'litter.ts:23: before a statement',
          'litter.ts:30: before a disable-next-line comment',
          'litter.ts:9: before a second TSDoc comment',
        ],);
      },
    },),
    it({
      name: 'ATTACHES EVERY TSDOC in this package\'s own source and tests to the thing it documents',
      fn: async () => {
        /**
         Every package file, tests among them.
         */
        const files = await readPackageSource();
        expect(files.some(function isSource(file,): boolean {
          return !file.isTest;
        },),).toBe(true,);
        expectNoFindings({ findings: orphanTsdocEntries({ files, },), },);
      },
    },),
  ],
},);
