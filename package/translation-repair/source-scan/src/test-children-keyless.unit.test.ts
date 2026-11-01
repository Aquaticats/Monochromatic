/**
 Guards the one way a test starts a child process: through
 `child-environment.test-fixture.ts`, which removes every variable whose name
 ends in `_API_KEY` from the child's environment. The owner's rule is that
 keys come from the repository's task runner and a test's child never
 inherits them; a child started anywhere else takes the parent's environment
 whole, and nothing in the suite says so until a child that spends is
 written. A copy of a keyless runner in each test file kept the rule only
 where its author remembered it.

 WHAT THE SCAN READS, in tests and test fixtures other than the shared
 fixture: an import, re-export, dynamic import, `require` or
 `getBuiltinModule` of `nano-spawn`, `execa`, `cross-spawn`, `node:child_process`
 or `child_process`, and a call of `spawn`, `spawnSync`, `execFile`,
 `execFileSync`, `execSync`, `fork` or `exec`, by a bare name or (but for
 `exec`, which a regular expression has) through a member, whatever the
 import binds it to. A finding names the file and the import or call.
 Package source is not read: its children are production code and the
 owner's call.

 OUT OF THE SCAN'S REACH: a child started through a module the scan does not
 name, a call of one of those functions through a computed non-literal
 member, and a shell started by a wrapper written in another language. The
 scan holds the rule at the import; a reviewer reads the shared fixture.

 THE FIXTURE CASE COMES FIRST, so the package-wide case is read against a
 scan shown able to find each form (ledger M21). Fixtures are cat-themed; the
 package case reads this package's own tests.

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
  literalText,
  memberName,
  parseSource,
  type SourceText,
  type TreeNode,
} from './source-scan.test-fixture.ts';
import { expectNoFindings, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scan-findings.test-fixture.ts';

/**
 Path, relative to `src`, of the one file allowed to start a child.
 */
const SHARED_FIXTURE = 'adjudication/child-environment.test-fixture.ts';

/**
 Modules that start a child process.
 */
const SPAWN_MODULES: ReadonlySet<string> = new Set([
  'child_process',
  'cross-spawn',
  'execa',
  'nano-spawn',
  'node:child_process',
],);

/**
 Functions that start a child, called by a bare name or through a member.
 */
const SPAWN_CALLS: ReadonlySet<string> = new Set([
  'execFile',
  'execFileSync',
  'execSync',
  'fork',
  'spawn',
  'spawnSync',
],);

/**
 Function that starts a child where called by a bare name only, since a
 member of that name is a regular expression's.
 */
const BARE_ONLY_CALL = 'exec';

/**
 What a node says about starting a child, as the scan reports it.

 @param node - node read

 @returns One finding text per import or call of a spawning module or
 function, empty for any other node

 @example
 ```ts
 const found = findingsOf({ node, },); // ['imports nano-spawn'] for import spawn from 'nano-spawn'
 ```
 */
function findingsOf({ node, }: { readonly node: TreeNode; },): readonly string[] {
  if ((node.type === 'ImportDeclaration') || (node.type === 'ExportAllDeclaration')
    || (node.type === 'ExportNamedDeclaration') || (node.type === 'ImportExpression')) {
    /**
     Module the node names.
     */
    const module = literalText({ node: node.source, },);
    return SPAWN_MODULES.has(module,) ? [`imports ${module}`,] : [];
  }
  if (node.type !== 'CallExpression')
    return [];
  /**
   The call's callee and arguments.
   */
  const { callee, } = node;
  const args = node.arguments as readonly TreeNode[];
  if (!isTreeNode(callee,))
    return [];
  /**
   Name the callee is written with, by a bare name or through a member.
   */
  const bare = identifierName({ node: callee, },);
  const member = (callee.type === 'MemberExpression') ? memberName({ node: callee, },) : '';
  if ((bare === 'require') || (member === 'getBuiltinModule')) {
    /**
     Module the call loads.
     */
    const module = literalText({ node: args[0], },);
    return SPAWN_MODULES.has(module,) ? [`imports ${module}`,] : [];
  }
  if ((bare === BARE_ONLY_CALL) || SPAWN_CALLS.has(bare,))
    return [`calls ${bare}`,];
  return SPAWN_CALLS.has(member,) ? [`calls ${member}`,] : [];
}

/**
 Children the tests and fixtures start outside the shared fixture, as
 `path: finding`.

 @param files - files read; package source is skipped

 @returns Findings, sorted and without repeats

 @example
 ```ts
 const children = childrenStartedApart({ files, },);
 ```
 */
function childrenStartedApart({ files, }: { readonly files: readonly SourceText[]; },): readonly string[] {
  /**
   Findings so far.
   */
  const found = new Set<string>();
  for (const file of files) {
    if ((!file.isTest) || (file.path === SHARED_FIXTURE))
      continue;
    /**
     Nodes still to visit.
     */
    const pending: TreeNode[] = [parseSource({ file, },).program,];
    while (pending.length > 0) {
      /**
       Node visited now.
       */
      const node = pending.pop() as TreeNode;
      pending.push(...childNodes({ node, },),);
      for (const finding of findingsOf({ node, },))
        found.add(`${file.path}: ${finding}`,);
    }
  }
  return [...found,].toSorted();
}

await describe({
  name: 'children started by tests keep no provider key',
  children: [
    it({
      name: 'FINDS a file that imports a spawning module under any form or name, or calls a spawning function, '
        + 'and leaves the shared fixture, a file that goes through it, a regular expression\'s exec and package source',
      fn: async () => {
        expect(childrenStartedApart({
          files: [
            {
              path: 'cat.unit.test.ts',
              text: [
                'import spawn from \'nano-spawn\';',
                'export const nap = async () => await spawn(\'git\', [\'status\',],);',
              ].join('\n',),
              isTest: true,
            },
            {
              path: 'kitten.test-fixture.ts',
              text: [
                'import { spawnSync as launch, } from \'node:child_process\';',
                'export const purr = () => launch(\'git\', [],);',
              ].join('\n',),
              isTest: true,
            },
            {
              path: 'tabby.unit.test.ts',
              text: [
                'import * as cp from \'child_process\';',
                'export const knead = () => cp.execFileSync(\'git\', [],);',
                'export const groom = async () => await import(\'node:child_process\');',
                'export const yawn = () => require(\'execa\');',
                'export const pounce = () => process.getBuiltinModule(\'node:child_process\');',
                'export { fork, } from \'node:child_process\';',
              ].join('\n',),
              isTest: true,
            },
            {
              path: 'calico.unit.test.ts',
              text: 'export const stretch = () => exec(\'git status\',);',
              isTest: true,
            },
            {
              path: 'ginger.unit.test.ts',
              text: [
                'import { spawnKeyless, } from \'./child-environment.test-fixture.ts\';',
                'export const sleep = async () => await spawnKeyless({ file: \'git\', args: [], },);',
                'export const find = (text: string) => /cat/u.exec(text,);',
              ].join('\n',),
              isTest: true,
            },
            {
              path: 'adjudication/child-environment.test-fixture.ts',
              text: 'import spawn from \'nano-spawn\'; export const run = () => spawn(\'git\', [],);',
              isTest: true,
            },
            {
              path: 'cat.ts',
              text: 'import spawn from \'nano-spawn\'; export const run = () => spawn(\'git\', [],);',
              isTest: false,
            },
          ],
        },),).toEqual([
          'calico.unit.test.ts: calls exec',
          'cat.unit.test.ts: calls spawn',
          'cat.unit.test.ts: imports nano-spawn',
          'kitten.test-fixture.ts: imports node:child_process',
          'tabby.unit.test.ts: calls execFileSync',
          'tabby.unit.test.ts: imports child_process',
          'tabby.unit.test.ts: imports execa',
          'tabby.unit.test.ts: imports node:child_process',
        ],);
      },
    },),
    it({
      name: 'FINDS NO CHILD STARTED ACROSS THE PACKAGE\'S TESTS AND FIXTURES outside the shared fixture that '
        + 'removes every provider key from the child\'s environment',
      fn: async () => {
        /**
         Every package file, source among them to be skipped.
         */
        const files = await readPackageSource();
        expectNoFindings({ findings: childrenStartedApart({ files, },), },);
        expect(files.some(function isSharedFixture(file,): boolean {
          return file.path === SHARED_FIXTURE;
        },),).toBe(true,);
      },
    },),
  ],
},);
