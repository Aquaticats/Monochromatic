/**
 Guards against a caught error dropped on the floor (ledger B29): every catch
 clause in the package's source binds what it caught, never discards it with
 `void`, and logs it, rethrows, or passes it on, by returning, recording or
 handing on an expression that names it. `openRouterChunksOf` wrote
 `void error` to satisfy the rule that a binding be used and returned
 nothing, the one clause of 142 that dropped its error.

 WHAT COUNTS AS PASSING IT ON. Any use of the binding other than `void`
 inside the clause, outside functions nested in it: the scan does not follow
 where the value goes, so a clause that names the error only to test its
 class and then returns a constant passes, and review has to catch that.

 THE FIXTURE CASE COMES FIRST, so the package-wide case is read against a
 scan shown able to find each kind (ledger M21). Fixtures are cat-themed;
 the package case reads this package's own source.

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
} from './source-scan.test-fixture.ts';
import { expectNoFindings, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scan-findings.test-fixture.ts';

/**
 Method names a logger call goes through.
 */
const LOG_METHODS: ReadonlySet<string> = new Set(['debug', 'error', 'info', 'trace', 'warn',],);

/**
 Node kinds that open a function, whose body a clause's scan does not enter.
 */
const FUNCTION_KINDS: ReadonlySet<string> = new Set(['ArrowFunctionExpression', 'FunctionDeclaration', 'FunctionExpression',],);

/**
 What a catch clause does with the error it caught.
 */
type ClauseReading = {
  /**
   Whether it throws.
   */
  readonly rethrows: boolean;

  /**
   Whether it calls a logger method.
   */
  readonly logs: boolean;

  /**
   Whether it names the binding other than under `void`.
   */
  readonly passesOn: boolean;

  /**
   Whether it discards the binding with `void`.
   */
  readonly discards: boolean;
};

/**
 The child of a node that only spells a name, where an identifier spelled like
 the binding is no use of it: the property after the dot of a member access and
 the key of an object literal's entry (a shorthand entry's key is also its
 value, so it stays).

 @param node - node whose children are about to be visited

 @returns Name-only child, or nothing when the node has none

 @example
 ```ts
 const name = namePositionOf({ node: memberAccess, },); // the `error` of `console.error`
 ```
 */
function namePositionOf({ node, }: { readonly node: TreeNode; },): unknown {
  if ((node.type === 'MemberExpression') && (node.computed !== true))
    return node.property;

  if ((node.type === 'Property') && (node.computed !== true) && (node.shorthand !== true))
    return node.key;

  return undefined;
}

/**
 Reads one catch clause's body, not entering functions nested in it.

 @param body - clause's block

 @param binding - name the clause binds its error to

 @returns What the clause does with the error

 @example
 ```ts
 const reading = readClause({ body: clause.body, binding: 'error', },);
 ```
 */
function readClause(
  {
    body,
    binding,
  }: {
    readonly body: TreeNode;
    readonly binding: string;
  },
): ClauseReading {
  /**
   Findings so far.
   */
  const reading = {
    rethrows: false,
    logs: false,
    passesOn: false,
    discards: false,
  };
  /**
   Nodes still to visit, each with whether it sits directly under `void`.
   */
  const pending: { readonly node: TreeNode; readonly underVoid: boolean; }[] = [{ node: body, underVoid: false, },];
  while (pending.length > 0) {
    /**
     Node visited now.
     */
    const { node, underVoid, } = pending.pop() as (typeof pending)[number];
    if (FUNCTION_KINDS.has(node.type,))
      continue;
    if (node.type === 'ThrowStatement')
      reading.rethrows = true;
    if ((node.type === 'CallExpression') && isTreeNode(node.callee,) && (node.callee.type === 'MemberExpression')
      && isTreeNode(node.callee.property,) && LOG_METHODS.has(identifierName({ node: node.callee.property, },),))
      reading.logs = true;
    if (identifierName({ node, },) === binding) {
      if (underVoid)
        reading.discards = true;
      else
        reading.passesOn = true;
    }
    /**
     Whether the node's own operand is discarded.
     */
    const voids = (node.type === 'UnaryExpression') && (node.operator === 'void');
    /**
     Child that only spells a name, which is no use of the binding.
     */
    const spellsName = namePositionOf({ node, },);
    pending.push(...childNodes({ node, },)
      .filter(function isUse(child,): boolean {
        return child !== spellsName;
      },)
      .map(function withContext(child,) {
        return {
          node: child,
          underVoid: voids,
        };
      },),);
  }
  return reading;
}

/**
 Catch clauses in the package's source that drop what they caught, as
 `path#site: reason`.

 @param files - files read; tests and fixtures are skipped

 @returns Findings, sorted

 @example
 ```ts
 const dropped = droppedErrors({ files, },);
 ```
 */
function droppedErrors({ files, }: { readonly files: readonly SourceText[]; },): readonly string[] {
  /**
   Findings so far.
   */
  const found: string[] = [];
  for (const file of files) {
    if (file.isTest)
      continue;
    /**
     Nodes still to visit, each with its enclosing function's name.
     */
    const pending: { readonly node: TreeNode; readonly site: string; }[] = [{
      node: parseSource({ file, },).program,
      site: '<module>',
    },];
    while (pending.length > 0) {
      /**
       Node visited now.
       */
      const { node, site, } = pending.pop() as (typeof pending)[number];
      /**
       Enclosing function's name for the node's children.
       */
      const here = (FUNCTION_KINDS.has(node.type,) && isTreeNode(node.id,)) ? identifierName({ node: node.id, },) : site;
      pending.push(...childNodes({ node, },).map(function withSite(child,) {
        return {
          node: child,
          site: here,
        };
      },),);
      if ((node.type !== 'CatchClause') || (!isTreeNode(node.body,)))
        continue;
      /**
       Name the clause binds, empty for none.
       */
      const binding = isTreeNode(node.param,) ? identifierName({ node: node.param, },) : '';
      if (binding === '') {
        found.push(`${file.path}#${here}: binds no error`,);
        continue;
      }
      /**
       What the clause does with it.
       */
      const reading = readClause({
        body: node.body,
        binding,
      },);
      if (reading.discards)
        found.push(`${file.path}#${here}: discards ${binding} with void`,);
      else if (!(reading.rethrows || reading.logs || reading.passesOn))
        found.push(`${file.path}#${here}: neither logs, rethrows nor passes on ${binding}`,);
    }
  }
  return found.toSorted();
}

await describe({
  name: 'caught errors kept (ledger B29)',
  children: [
    it({
      name: 'FINDS a clause binding nothing, one discarding its error with void, one that drops it, and one that only '
        + 'names a property or a key spelled like it, and leaves clauses that log, rethrow or return it, and tests',
      fn: async () => {
        expect(droppedErrors({
          files: [
            {
              path: 'cat.ts',
              text: [
                'export function nap(): number { try { return 1; } catch { return 0; } }',
                'export function purr(): number { try { return 1; } catch (error) { void error; return 0; } }',
                'export function groom(): number { try { return 1; } catch (error) { return 0; } }',
                'export function knead(l: { warn(text: string): void }): number { try { return 1; } '
                  + 'catch (error) { l.warn(\'no knead\'); return 0; } }',
                'export function pounce(): number { try { return 1; } catch (error) { throw new Error(\'missed\'); } }',
                'export function stretch(): string { try { return \'\'; } catch (error) { return String(error); } }',
                'export function sulk(state: { error: number }): number { try { return 1; } '
                  + 'catch (error) { return { error: 0 }.error + state.error; } }',
                'export function yawn(): string { try { return \'\'; } catch (error) { const later = () => error; return \'\'; } }',
              ].join('\n',),
              isTest: false,
            },
            {
              path: 'cat.unit.test.ts',
              text: 'try { JSON.parse(\'{\'); } catch (error) { void error; }',
              isTest: true,
            },
          ],
        },),).toEqual([
          'cat.ts#groom: neither logs, rethrows nor passes on error',
          'cat.ts#nap: binds no error',
          'cat.ts#purr: discards error with void',
          'cat.ts#sulk: neither logs, rethrows nor passes on error',
          'cat.ts#yawn: neither logs, rethrows nor passes on error',
        ],);
      },
    },),
    it({
      name: 'FINDS NO CAUGHT ERROR DROPPED across the package',
      fn: async () => {
        /**
         Every package file, tests among them to be skipped.
         */
        const files = await readPackageSource();
        expect(files.some(function isSource(file,): boolean {
          return !file.isTest;
        },),).toBe(true,);
        expectNoFindings({ findings: droppedErrors({ files, },), },);
      },
    },),
  ],
},);
