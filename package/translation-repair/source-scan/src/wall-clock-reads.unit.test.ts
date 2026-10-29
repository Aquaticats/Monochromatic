/**
 Guards against durations, deadlines and budgets measured on the wall clock
 (ledger B78). The wall clock is the system's time of day, which a person or
 a time daemon can set: stepped back an hour mid-call, a duration reads
 negative, a hold or a pace waits an hour longer than asked, and a budget
 lasts an hour more; stepped forward, holds end early and a budget runs out
 at once. Elapsed time is read with `monotonicMs` (`monotonic-clock.ts`),
 which Node reads from `CLOCK_MONOTONIC`, the clock libuv's timers already
 run on; the wall clock is read only as a stamp, `new Date().toISOString()`.

 WHAT THE SCAN READS, in the package's source and its tests alike: `Date.now`,
 called, handed on, or read through a computed `'now'`; `now` destructured
 from `Date`; a call of `getTime` or `valueOf`, on anything; a unary plus or a
 `Number` call on `new Date`; `performance.timeOrigin`, the wall clock at the
 process's start; and `Temporal.Now`. Reads are keyed `path#site: form`,
 where the site is the nearest enclosing named function.
 `wall-clock-stub.test-fixture.ts`, which stands in for the wall clock in the
 cases that step it, and `wall-clock-stub.unit.test.ts`, which reads the
 clock to show the stub reaches only its own case, are not read (ledger M100).
 Out of the scan's reach: `Date`,
 `performance` or `Temporal` reached through `globalThis` or another name, a
 date compared with `<` or `>`, which reads it as a number, and a duration
 taken between stamps a process wrote, as the run-timing report takes its
 durations from the logger's stamps (ledger B78).

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

//region Wall clock reads
// The scan itself: which nodes read the wall clock as a number, and the walk
// that keys each read by its file and enclosing named function.

/**
 The two files that name the wall clock on purpose: the stub, which replaces
 `Date.now` for the cases that step it, and its own test, which reads the
 clock in a case beside one that stepped it (ledger M100).
 */
const STUB_PATHS: ReadonlySet<string> = new Set([
  'test-fixture/wall-clock-stub.test-fixture.ts',
  'test-fixture/wall-clock-stub.unit.test.ts',
],);

/**
 Node kinds that open a function, which names the site of the reads inside.
 */
const FUNCTION_KINDS: ReadonlySet<string> = new Set([
  'ArrowFunctionExpression',
  'FunctionDeclaration',
  'FunctionExpression',
],);

/**
 Members whose call reads a date as a number.
 */
const NUMBER_MEMBERS: ReadonlySet<string> = new Set([
  'getTime',
  'valueOf',
],);

/**
 Whether an expression is a global by name, through parentheses and casts.

 @param node - expression read

 @param name - global's name

 @returns Whether the expression is that global

 @example
 ```ts
 const fromDate = isGlobal({ node: member.object, name: 'Date', },);
 ```
 */
function isGlobal(
  {
    node,
    name,
  }: {
    readonly node: unknown;
    readonly name: string;
  },
): boolean {
  return identifierName({ node: unwrapped({ node, },).inner, },) === name;
}

/**
 Whether an expression constructs a date, with or without arguments.

 @param node - expression read

 @returns Whether it is `new Date`

 @example
 ```ts
 const dated = isNewDate({ node: unary.argument, },);
 ```
 */
function isNewDate({ node, }: { readonly node: unknown; },): boolean {
  /**
   The expression inside any parentheses and casts.
   */
  const { inner, } = unwrapped({ node, },);
  return isTreeNode(inner,) && (inner.type === 'NewExpression') && isGlobal({
    node: inner.callee,
    name: 'Date',
  },);
}

/**
 Whether an object pattern takes `now` from what it destructures.

 @param pattern - object pattern read

 @returns Whether one of its properties is keyed `now`

 @example
 ```ts
 const takesNow = destructuresNow({ pattern: declarator.id, },);
 ```
 */
function destructuresNow({ pattern, }: { readonly pattern: TreeNode; },): boolean {
  /**
   The pattern's properties, rest elements among them.
   */
  const properties = pattern.properties as readonly unknown[];
  return properties.some(function isNow(property,): boolean {
    return isTreeNode(property,) && (property.type === 'Property') && (identifierName({ node: property.key, },) === 'now');
  },);
}

/**
 The wall-clock read a node makes itself, where it makes one.

 @param node - node read

 @returns Read's form as the key writes it, empty where the node reads no
 wall clock

 @example
 ```ts
 const read = wallReadOf({ node, },); // 'Date.now'
 ```
 */
function wallReadOf({ node, }: { readonly node: TreeNode; },): string {
  if (node.type === 'MemberExpression') {
    /**
     The member read.
     */
    const member = memberName({ node, },);
    if ((member === 'now') && isGlobal({
      node: node.object,
      name: 'Date',
    },))
      return 'Date.now';
    if ((member === 'timeOrigin') && isGlobal({
      node: node.object,
      name: 'performance',
    },))
      return 'performance.timeOrigin';
    return ((member === 'Now') && isGlobal({
      node: node.object,
      name: 'Temporal',
    },))
      ? 'Temporal.Now'
      : '';
  }
  if (node.type === 'CallExpression') {
    /**
     The callee inside any parentheses and casts.
     */
    const { inner: callee, } = unwrapped({ node: node.callee, },);
    if (isTreeNode(callee,) && (callee.type === 'MemberExpression') && NUMBER_MEMBERS.has(memberName({ node: callee, },),))
      return `${memberName({ node: callee, },)}()`;
    /**
     The arguments the call passes.
     */
    const args = node.arguments as readonly unknown[];
    return (isGlobal({
      node: node.callee,
      name: 'Number',
    },) && isNewDate({ node: args[0], },))
      ? 'Number(new Date)'
      : '';
  }
  if ((node.type === 'UnaryExpression') && (node.operator === '+'))
    return isNewDate({ node: node.argument, },) ? '+new Date' : '';
  /**
   What a declaration or an assignment destructures, and from what.
   */
  const [pattern, source,] = (node.type === 'VariableDeclarator')
    ? [
      node.id,
      node.init,
    ]
    : (node.type === 'AssignmentExpression')
      ? [
        node.left,
        node.right,
      ]
      : [
        undefined,
        undefined,
      ];
  return (isTreeNode(pattern,) && (pattern.type === 'ObjectPattern') && isGlobal({
    node: source,
    name: 'Date',
  },) && destructuresNow({ pattern, },))
    ? 'now destructured from Date'
    : '';
}

/**
 Every read of the wall clock as a number in the files given, keyed
 `path#site: form`, one entry per read.

 @param files - files read, tests among them; the wall-clock stub and its
 test are skipped

 @returns Keys sorted, repeated once per read

 @example
 ```ts
 const reads = wallClockReads({ files, },);
 ```
 */
function wallClockReads({ files, }: { readonly files: readonly SourceText[]; },): readonly string[] {
  /**
   Keys found so far.
   */
  const found: string[] = [];
  for (const file of files) {
    if (STUB_PATHS.has(file.path,))
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
       Enclosing named function for the node's children.
       */
      const here = (FUNCTION_KINDS.has(node.type,) && isTreeNode(node.id,)) ? identifierName({ node: node.id, },) : site;
      /**
       The read the node makes, where it makes one.
       */
      const read = wallReadOf({ node, },);
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

//endregion Wall clock reads

await describe({
  name: 'wall clock reads (ledger B78)',
  children: [
    it({
      name: 'FINDS Date.now called, handed on, computed or destructured, a date read as a number by getTime, '
        + 'valueOf, unary plus or Number, performance.timeOrigin and Temporal.Now, in source and tests alike, '
        + 'and leaves stamps, performance.now, other objects\' now, and the wall-clock stub and its test',
      fn: async () => {
        expect(wallClockReads({
          files: [
            {
              path: 'litter.ts',
              text: [
                'export function napLength(start: number,): number {',
                '  return Date.now() - start;',
                '}',
                'export const pace = { now: Date.now, };',
                'export const kitten = Date[\'now\']();',
                'const { now, } = Date;',
                'let wake: () => number;',
                '({ now: wake, } = Date);',
                'export function purr(): number {',
                '  return new Date().getTime() + (new Date()).valueOf() + +new Date() + Number(new Date(),);',
                '}',
                'export const origin = performance.timeOrigin + now() + wake();',
                'export const instant = Temporal.Now.instant();',
                'export const shadow = (Date as DateConstructor).now();',
                'export const stamp = new Date().toISOString();',
                'export const steady = performance.now();',
                'export const sleepy = { now: 1, }.now + Number(\'2\',) + +\'3\';',
              ].join('\n',),
              isTest: false,
            },
            {
              path: 'catnip.unit.test.ts',
              text: 'export const began = Date.now();',
              isTest: true,
            },
            ...[...STUB_PATHS,].map(function exempt(path,): SourceText {
              return {
                path,
                text: 'Date.now = () => new Date().getTime() - Date.now();',
                isTest: true,
              };
            },),
          ],
        },),).toEqual([
          'catnip.unit.test.ts#<module>: Date.now',
          'litter.ts#<module>: Date.now',
          'litter.ts#<module>: Date.now',
          'litter.ts#<module>: Date.now',
          'litter.ts#<module>: Temporal.Now',
          'litter.ts#<module>: now destructured from Date',
          'litter.ts#<module>: now destructured from Date',
          'litter.ts#<module>: performance.timeOrigin',
          'litter.ts#napLength: Date.now',
          'litter.ts#purr: +new Date',
          'litter.ts#purr: Number(new Date)',
          'litter.ts#purr: getTime()',
          'litter.ts#purr: valueOf()',
        ],);
      },
    },),
    it({
      name: 'READS NO WALL CLOCK AS A NUMBER in the package\'s source or tests',
      fn: async () => {
        /**
         Every package file, tests among them.
         */
        const files = await readPackageSource();
        expect(files.some(function isSource(file,): boolean {
          return !file.isTest;
        },),).toBe(true,);
        expectNoFindings({ findings: wallClockReads({ files, },), },);
      },
    },),
  ],
},);
