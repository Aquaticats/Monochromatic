/**
 Guards against text ordered or cased by the runtime's locale (ledger B95).
 `localeCompare` orders by the locale and collation data of whatever machine
 runs it, so a benchmark draw broke its ties one way on one machine and
 another way on the next. The package orders text with `compareCodePoints`
 (`code-points.ts`), the same on every machine, and cases it with the
 locale-free `toLowerCase` and `toUpperCase`.

 WHAT THE SCAN READS, in the package's source and its tests alike: a member
 named `localeCompare`, `toLocaleLowerCase`, `toLocaleUpperCase`,
 `toLocaleString`, `toLocaleDateString` or `toLocaleTimeString`, called or
 handed on, written plainly or computed from a string; and `Intl.Collator`.
 Reads are keyed `path#site: form`, where the site is the nearest enclosing
 named function. Out of the scan's reach: `Intl` reached through
 `globalThis` or another name, and a member named through a variable.

 A SECOND SCAN READS ORDERINGS THAT SKIP THE CODE-POINT ORDER (ledger B122):
 an argument-less `sort` or `toSorted` in package source, which orders by
 UTF-16 unit, so an astral character sorts before one in U+E000 to U+FFFF;
 and a relational operator inside a function handed straight to `sort` or
 `toSorted`, in source and tests alike, which orders text by UTF-16 unit and,
 written as `left < right ? -1 : 1`, never answers zero for two equal keys,
 which the comparator contract requires. Numbers order by subtraction, text by
 `compareCodePoints`, and a list of plain text through `textsInCodePointOrder`
 (`code-points.ts`). A code-unit order kept on purpose is named in
 `CODE_UNIT_ORDER_EXEMPTIONS` with why. Out of this scan's reach: a comparator
 declared apart and handed in by name, and a helper a comparator calls; the
 package hands every comparator inline.

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
  memberName,
  parseSource,
  type SourceText,
  type TreeNode,
  unwrapped,
} from './source-scan.test-fixture.ts';
import { expectNoFindings, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scan-findings.test-fixture.ts';

//region Locale orderings
// The scan itself: which nodes order or case text by the runtime's locale, and
// the walk that keys each by its file and enclosing named function.

/**
 Node kinds that open a function, which names the site of the reads inside.
 */
const FUNCTION_KINDS: ReadonlySet<string> = new Set([
  'ArrowFunctionExpression',
  'FunctionDeclaration',
  'FunctionExpression',
],);

/**
 String and date members whose answer follows the runtime's locale.
 */
const LOCALE_MEMBERS: ReadonlySet<string> = new Set([
  'localeCompare',
  'toLocaleDateString',
  'toLocaleLowerCase',
  'toLocaleString',
  'toLocaleTimeString',
  'toLocaleUpperCase',
],);

/**
 The locale-dependent read a node makes itself, where it makes one.

 @param node - node read

 @returns Read's form as the key writes it, empty where the node makes
 none

 @example
 ```ts
 const read = localeReadOf({ node, },); // 'localeCompare'
 ```
 */
function localeReadOf({ node, }: { readonly node: TreeNode; },): string {
  if (node.type !== 'MemberExpression')
    return '';
  /**
   The member read.
   */
  const member = memberName({ node, },);
  if (LOCALE_MEMBERS.has(member,))
    return member;
  return ((member === 'Collator') && (identifierName({ node: unwrapped({ node: node.object, },).inner, },) === 'Intl'))
    ? 'Intl.Collator'
    : '';
}

/**
 Every locale-dependent read in the files given, keyed `path#site: form`, one
 entry per read.

 @param files - files read, tests among them

 @returns Keys sorted, repeated once per read

 @example
 ```ts
 const reads = localeReads({ files, },);
 ```
 */
function localeReads({ files, }: { readonly files: readonly SourceText[]; },): readonly string[] {
  /**
   Keys found so far.
   */
  const found: string[] = [];
  for (const file of files) {
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
       Enclosing named function for the node's children.
       */
      const here = (FUNCTION_KINDS.has(node.type,) && isTreeNode(node.id,)) ? identifierName({ node: node.id, },) : site;
      /**
       The read the node makes, where it makes one.
       */
      const read = localeReadOf({ node, },);
      if (read !== '')
        found.push(`${file.path}#${here}: ${read}`,);
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

//endregion Locale orderings

//region Code-unit orderings
// The second scan (ledger B122): orderings that skip `compareCodePoints` by
// ordering on UTF-16 units, and comparators that cannot answer zero.

/**
 Array members that order their receiver.
 */
const SORT_MEMBERS: ReadonlySet<string> = new Set([
  'sort',
  'toSorted',
],);

/**
 Binary operators that order two values.
 */
const RELATIONAL_OPERATORS: ReadonlySet<string> = new Set([
  '<',
  '<=',
  '>',
  '>=',
],);

/**
 Orderings that are code-unit order on purpose, each with why.

 `canonicalPromptValue` serializes a record's keys as canonical JSON
 (RFC 8785) does, which sorts keys by UTF-16 unit, and its digest names a
 durable payload record another host reads; code-point order would be
 another canonical form.
 */
const CODE_UNIT_ORDER_EXEMPTIONS: ReadonlySet<string> = new Set([
  'stage/prompt-uniqueness-client.ts#canonicalPromptValue: bare sort',
],);

/**
 The sort call a node is, with the arguments it passes, where it is one.

 @param node - node read

 @returns Call's arguments, or no record where the node calls no sort

 @example
 ```ts
 const sort = sortCallOf({ node, },); // { arguments: [] }
 ```
 */
function sortCallOf({ node, }: { readonly node: TreeNode; },): readonly { readonly arguments: readonly unknown[]; }[] {
  if ((node.type !== 'CallExpression') || (!isTreeNode(node.callee,)))
    return [];
  /**
   The callee, past any parentheses or type wrappers.
   */
  const { inner: callee, } = unwrapped({ node: node.callee, },);
  if ((!isTreeNode(callee,)) || (callee.type !== 'MemberExpression') || (!SORT_MEMBERS.has(memberName({ node: callee, },),)))
    return [];
  return [{ arguments: Array.isArray(node.arguments,) ? node.arguments : [], },];
}

/**
 Every ordering in the files given that skips the package's code-point order,
 keyed `path#site: form`, one entry per ordering: an argument-less `sort` or
 `toSorted` in package source, which orders by UTF-16 unit, and a relational
 operator inside a function handed straight to `sort` or `toSorted`, in
 source and tests alike, which orders by UTF-16 unit on text and, written to
 answer -1 or 1, never answers zero for two equal keys.

 TESTS MAY SORT WITHOUT A COMPARATOR: every such sort in the package's tests
 orders text-typed values to compare two sides, which no order changes
 (measured with a type-aware census, ledger B122).

 @param files - files read, tests among them

 @returns Keys sorted, repeated once per ordering

 @example
 ```ts
 const orderings = codeUnitOrderings({ files, },);
 ```
 */
function codeUnitOrderings({ files, }: { readonly files: readonly SourceText[]; },): readonly string[] {
  /**
   Keys found so far.
   */
  const found: string[] = [];
  for (const file of files) {
    /**
     The functions handed straight to a sort as its order, past any
     parentheses or type wrappers, gathered as their calls are visited, which
     is always before the functions themselves.
     */
    const comparators = new Set<unknown>();
    /**
     Nodes still to visit, each with its enclosing named function and whether
     it sits inside a comparator.
     */
    const pending: {
      readonly node: TreeNode;
      readonly site: string;
      readonly inComparator: boolean;
    }[] = [{
      node: parseSource({ file, },).program,
      site: '<module>',
      inComparator: false,
    },];
    for (let next = pending.pop(); next !== undefined; next = pending.pop()) {
      /**
       Node visited now and its site.
       */
      const {
        node,
        site,
      } = next;
      /**
       Whether the node is a comparator or sits inside one.
       */
      const inComparator = next.inComparator || (comparators.has(node,) && FUNCTION_KINDS.has(node.type,));
      /**
       Enclosing named function for the node's children.
       */
      const here = (FUNCTION_KINDS.has(node.type,) && isTreeNode(node.id,)) ? identifierName({ node: node.id, },) : site;
      /**
       The sort the node calls, where it calls one.
       */
      const [sort,] = sortCallOf({ node, },);
      if (sort !== undefined) {
        if ((sort.arguments.length === 0) && (!file.isTest))
          found.push(`${file.path}#${here}: bare sort`,);
        comparators.add(unwrapped({ node: sort.arguments[0], },).inner,);
      }
      if (inComparator && (node.type === 'BinaryExpression') && RELATIONAL_OPERATORS.has(String(node.operator,),))
        found.push(`${file.path}#${here}: ${String(node.operator,)} in a comparator`,);
      pending.push(...childNodes({ node, },).map(function withSite(child,) {
        return {
          node: child,
          site: here,
          inComparator,
        };
      },),);
    }
  }
  return found.toSorted();
}

//endregion Code-unit orderings

await describe({
  name: 'locale and code-unit orderings (ledger B95, B122)',
  children: [
    it({
      name: 'FINDS localeCompare called, handed on or computed, the toLocale casing and formatting members, and '
        + 'Intl.Collator, in source and tests alike, and leaves toLowerCase, toUpperCase, toString and other '
        + 'objects\' Collator',
      fn: async () => {
        expect(localeReads({
          files: [
            {
              path: 'litter.ts',
              text: [
                'export function byName(left: string, right: string,): number {',
                '  return left.localeCompare(right,);',
                '}',
                'export const order = String.prototype.localeCompare;',
                'export const kitten = \'Tabby\'[\'localeCompare\'](\'mooncat\',);',
                'export const loud = \'purr\'.toLocaleUpperCase() + \'MEOW\'.toLocaleLowerCase();',
                'export const shown = (7).toLocaleString() + new Date().toLocaleDateString()'
                  + ' + new Date().toLocaleTimeString();',
                'export const collator = new Intl.Collator(\'en\',);',
                'export const plain = \'Purr\'.toLowerCase() + \'purr\'.toUpperCase() + (7).toString();',
                'export const other = { Collator: 1, }.Collator;',
              ].join('\n',),
              isTest: false,
            },
            {
              path: 'catnip.unit.test.ts',
              text: 'export const sorted = [\'b\', \'a\',].toSorted((left, right,) => left.localeCompare(right,),);',
              isTest: true,
            },
          ],
        },),).toEqual([
          'catnip.unit.test.ts#<module>: localeCompare',
          'litter.ts#<module>: Intl.Collator',
          'litter.ts#<module>: localeCompare',
          'litter.ts#<module>: localeCompare',
          'litter.ts#<module>: toLocaleDateString',
          'litter.ts#<module>: toLocaleLowerCase',
          'litter.ts#<module>: toLocaleString',
          'litter.ts#<module>: toLocaleTimeString',
          'litter.ts#<module>: toLocaleUpperCase',
          'litter.ts#byName: localeCompare',
        ],);
      },
    },),
    it({
      name: 'ORDERS AND CASES NO TEXT BY THE RUNTIME\'S LOCALE in the package\'s source or tests',
      fn: async () => {
        /**
         Every package file, tests among them.
         */
        const files = await readPackageSource();
        expect(files.some(function isSource(file,): boolean {
          return !file.isTest;
        },),).toBe(true,);
        expectNoFindings({ findings: localeReads({ files, },), },);
      },
    },),
    it({
      name: 'FINDS an argument-less sort or toSorted in source, and a relational operator in a comparator handed '
        + 'straight to a sort, named, wrapped or an arrow, in source and tests alike, and leaves a test\'s '
        + 'argument-less sort, a comparator through compareCodePoints or by subtraction, a comparator handed in by '
        + 'name, and a relational operator outside any comparator (ledger B122)',
      fn: async () => {
        expect(codeUnitOrderings({
          files: [
            {
              path: 'litter.ts',
              text: [
                'export const names = [\'Tabby\', \'Mooncat\',].toSorted();',
                'export const kept = [\'b\', \'a\',].sort();',
                'export const purred = [\'b\', \'a\',].toSorted(function byPurr(left, right,) {',
                '  return left < right ? -1 : 1;',
                '},);',
                'export const mewed = [\'b\', \'a\',].toSorted((function byMew(left: string, right: string,): number {',
                '  return left >= right ? 1 : -1;',
                '}),);',
                'export const fine = [\'b\', \'a\',].toSorted(function byCode(left, right,) {',
                '  return compareCodePoints({ left, right, },);',
                '},);',
                'export const counted = [3, 1,].toSorted(function byCount(left, right,) {',
                '  return left - right;',
                '},);',
                'export const handed = [\'b\', \'a\',].toSorted(byPurr,);',
                'export const outside = 1 < 2;',
              ].join('\n',),
              isTest: false,
            },
            {
              path: 'catnip.unit.test.ts',
              text: [
                'export const sorted = [\'b\', \'a\',].toSorted();',
                'export const ordered = [\'b\', \'a\',].toSorted((left, right,) => (left > right ? 1 : -1),);',
              ].join('\n',),
              isTest: true,
            },
          ],
        },),).toEqual([
          'catnip.unit.test.ts#<module>: > in a comparator',
          'litter.ts#<module>: bare sort',
          'litter.ts#<module>: bare sort',
          'litter.ts#byMew: >= in a comparator',
          'litter.ts#byPurr: < in a comparator',
        ],);
      },
    },),
    it({
      name: 'ORDERS NO TEXT BY UTF-16 UNIT AND HANDS NO SORT A COMPARATOR THAT CANNOT ANSWER ZERO in the package\'s '
        + 'source, or a comparator of that kind in its tests, outside the named exemptions, and every exemption '
        + 'still names one',
      fn: async () => {
        /**
         Every package file, tests among them.
         */
        const files = await readPackageSource();
        expect(files.some(function isSource(file,): boolean {
          return !file.isTest;
        },),).toBe(true,);
        /**
         Every ordering the scan finds, exemptions included.
         */
        const found = codeUnitOrderings({ files, },);
        expectNoFindings({
          findings: found.filter(function unexempt(key,): boolean {
            return !CODE_UNIT_ORDER_EXEMPTIONS.has(key,);
          },),
        },);
        expectNoFindings({
          findings: [...CODE_UNIT_ORDER_EXEMPTIONS,].filter(function gone(key,): boolean {
            return !found.includes(key,);
          },),
        },);
      },
    },),
  ],
},);
