/**
 Guards the package's checks that a value is an object (ledger B92). Parsed
 JSON and YAML are narrowed to a record by `isJsonRecord` (`json-guard.ts`),
 which refuses arrays; while it admitted them, a reader that probed named
 fields took an array as a stream event naming no type, a resumable cache
 record, answers keyed "0" and "1", or a body with no results. The census
 behind the fix found eighteen checks written inline beside the guard, each
 copying its old test, so a fix to the guard reached none of them.

 WHAT THE SCAN READS, in the package's source (tests and their fixtures
 check mock shapes of their own and are left out): every comparison of a
 `typeof` with the string `object`, by `===` or `!==`. Reads are keyed
 `path#site`, where the site is the nearest enclosing named function. The
 listed exemptions read a value that is not parsed JSON or YAML: a caught
 error, a member a type says cannot exist, a union whose other members are
 text, or the shape a message names.

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
  childNodes,
  identifierName,
  isTreeNode,
  parseSource,
  type SourceText,
  type TreeNode,
  unwrapped,
} from './source-scan.test-fixture.ts';
import { expectNoFindings, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scan-findings.test-fixture.ts';

//region Record checks
// The scan itself: which nodes compare a `typeof` with `object`, and the walk
// that keys each by its file and enclosing named function.

/**
 Node kinds that open a function, which names the site of the reads inside.
 */
const FUNCTION_KINDS: ReadonlySet<string> = new Set([
  'ArrowFunctionExpression',
  'FunctionDeclaration',
  'FunctionExpression',
],);

/**
 Comparison operators a check for an object is written with.
 */
const EQUALITY_OPERATORS: ReadonlySet<string> = new Set([
  '!==',
  '===',
],);

/**
 Checks for an object that read something other than parsed JSON or YAML,
 keyed `path#site`, each with why `isJsonRecord` is not the check there.
 */
const EXEMPT_CHECKS: ReadonlyMap<string, string> = new Map([
  [
    'document-structure/json-guard.ts#isJsonRecord',
    'the guard itself',
  ],
  [
    'corpus-source/corpus-source.ts#classifyCorpusReadFailure',
    'reads a caught error for its stderr',
  ],
  [
    'document-structure/parse-mdx.ts#stopPointOf',
    'reads a caught compiler error for the place it names',
  ],
  [
    'artifact/corpus-run/artifact-two-lane-project.ts#refuseUnknownMember',
    'names the runtime type of a member its type says cannot exist',
  ],
  [
    'artifact-pool/corpus-run/artifact-placement.ts#shapeOf',
    'names the shape of a value for a message, after its own array case',
  ],
  [
    'glossary/corpus-run/directory-listing.ts#carriesFilesystemCode',
    'reads a caught error for its filesystem code',
  ],
  [
    'run/corpus-run/cap-census-read.ts#readCapLog',
    'narrows a spend-line reading whose other members are text',
  ],
],);

/**
 Whether a node is the string `object`.

 @param node - node read

 @returns Whether it is that literal, parentheses aside

 @example
 ```ts
 const literal = isObjectText({ node: comparison.right, },);
 ```
 */
function isObjectText({ node, }: { readonly node: unknown; },): boolean {
  /**
   The node inside any parentheses.
   */
  const { inner, } = unwrapped({ node, },);
  return isTreeNode(inner,) && (inner.type === 'Literal') && (inner.value === 'object');
}

/**
 Whether a node is a `typeof` read.

 @param node - node read

 @returns Whether it is one, parentheses aside

 @example
 ```ts
 const read = isTypeofRead({ node: comparison.left, },);
 ```
 */
function isTypeofRead({ node, }: { readonly node: unknown; },): boolean {
  /**
   The node inside any parentheses.
   */
  const { inner, } = unwrapped({ node, },);
  return isTreeNode(inner,) && (inner.type === 'UnaryExpression') && (inner.operator === 'typeof');
}

/**
 Whether a node compares a `typeof` read with the string `object`.

 @param node - node read

 @returns Whether it is such a comparison, either way round

 @example
 ```ts
 const check = checksForObject({ node, },);
 ```
 */
function checksForObject({ node, }: { readonly node: TreeNode; },): boolean {
  if ((node.type !== 'BinaryExpression') || (!EQUALITY_OPERATORS.has(String(node.operator,),)))
    return false;
  return (isTypeofRead({ node: node.left, },) && isObjectText({ node: node.right, },))
    || (isObjectText({ node: node.left, },) && isTypeofRead({ node: node.right, },));
}

/**
 Every check for an object in the package source given, keyed `path#site`,
 one entry per check.

 @param files - files read; tests among them are passed over

 @returns Keys sorted, repeated once per check

 @example
 ```ts
 const checks = recordChecks({ files, },);
 ```
 */
function recordChecks({ files, }: { readonly files: readonly SourceText[]; },): readonly string[] {
  /**
   Keys found so far.
   */
  const found: string[] = [];
  for (const file of files) {
    if (file.isTest)
      continue;
    /**
     Nodes still to visit, each with its enclosing named function.
     */
    const pending: {
      readonly node: TreeNode;
      readonly site: string;
    }[] = [{
      node: parseSource({ file, },).program,
      site: '<module>',
    },];
    for (let next = pending.pop(); next !== undefined; next = pending.pop()) {
      /**
       Node visited now, and the site it sits in.
       */
      const {
        node,
        site,
      } = next;
      /**
       Enclosing named function for the node and its children.
       */
      const here = (FUNCTION_KINDS.has(node.type,) && isTreeNode(node.id,))
        ? identifierName({ node: node.id, },)
        : site;
      if (checksForObject({ node, },))
        found.push(`${file.path}#${here}`,);
      pending.push(...childNodes({ node, },).map(function withSite(child,) {
        return {
          node: child,
          site: here,
        };
      },),);
    }
  }
  return found.toSorted();
}

//endregion Record checks

await describe({
  name: 'record checks (ledger B92)',
  children: [
    it({
      name: 'FINDS a typeof compared with object by === or !==, parenthesized or bare, either way round, at module '
        + 'level or in a named function, and leaves other typeof comparisons, other object text and tests',
      fn: async () => {
        expect(recordChecks({
          files: [
            {
              path: 'litter.ts',
              text: [
                'export function isBasket(value: unknown,): boolean {',
                '  return ((typeof value) === \'object\') && (value !== null);',
                '}',
                'export function notBasket(value: unknown,): boolean {',
                '  return (\'object\' !== typeof value);',
                '}',
                'export const loose = typeof globalThis === \'object\';',
                'export const named = (typeof 7) === \'number\';',
                'export const said = \'object\' === \'object\';',
              ].join('\n',),
              isTest: false,
            },
            {
              path: 'catnip.unit.test.ts',
              text: 'export const mock = (typeof {}) === \'object\';',
              isTest: true,
            },
          ],
        },),).toEqual([
          'litter.ts#<module>',
          'litter.ts#isBasket',
          'litter.ts#notBasket',
        ],);
      },
    },),
    it({
      name: 'CHECKS PARSED JSON AND YAML FOR AN OBJECT ONLY THROUGH isJsonRecord in the package\'s source, outside '
        + 'the named exemptions, and every exemption still names a check',
      fn: async () => {
        /**
         Every package file, tests among them.
         */
        const files = await readPackageSource();
        expect(files.some(function isSource(file,): boolean {
          return !file.isTest;
        },),).toBe(true,);
        /**
         Every check the scan finds.
         */
        const checks = recordChecks({ files, },);
        expectNoFindings({
          findings: checks.filter(function unexempt(check,): boolean {
            return !EXEMPT_CHECKS.has(check,);
          },),
        },);
        expectNoFindings({
          findings: [...EXEMPT_CHECKS.keys(),].filter(function stale(exempt,): boolean {
            return !checks.includes(exempt,);
          },),
        },);
      },
    },),
  ],
},);
