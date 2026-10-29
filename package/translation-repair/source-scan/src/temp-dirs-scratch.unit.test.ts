/**
 Guards the one way the package makes a throwaway directory in its own tests
 (ledger B108): through `scratchDir` (`scratch-dir.test-fixture.ts`), bound
 with `await using` where it is made or by every caller of a helper
 returning it, so the directory goes when the case ends, however the case
 ends. A test or fixture that calls `mkdtemp` or `tmpdir()` for itself
 builds a second, unreviewed removal path, which is exactly how the 43
 leaks B108 found were made.

 WHAT THE SCAN READS. Every call to `mkdtemp`, imported by name from
 `node:fs` or `node:fs/promises`, under its own name or a local alias, or
 reached through a namespace or default import of either module
 (`fs.mkdtemp(...)`); and every call to `tmpdir`, imported the same three
 ways from `node:os`. `mkdtempDisposable`, Node's own disposable-returning
 sibling, is a different name and is not read here; a kept site that uses
 it says so. Only test files and test fixtures are read, since this is a
 test-side convention, never package source;
 `scratch-dir.test-fixture.ts` itself is excluded by its exact path, since
 it is the one fixture allowed to call either function directly.

 THE FIXTURE CASE COMES FIRST, so the package-wide case is read against a
 scan shown able to find each form (ledger M21): a bare name, a local
 alias, a namespace member and a default-import member, each left alone
 when the imported name, the module or the file does not match. Fixtures
 are cat-themed; the package case reads this package's own tests.

 KEPT SITES build a path that is deliberately absent and create nothing;
 wrapping one in `scratchDir` would create and later remove a directory for
 no reason, so each stays direct and is named here with why, keyed
 `path: form`. A kept site that stops matching, because the call was
 removed or moved onto `scratchDir` or the repository's `scratchRoot()`
 after all, fails this scan until its `KEPT` entry is removed too.
 `scratchRoot()` (`@monochromatic-dev/test-harness-core`) names the shared
 scratch root and creates nothing, so this scan does not read its calls.

 A SECOND RULE GUARDS THE SETUP WINDOW ITSELF (still ledger B108): every
 call to `scratchDir`, imported by name from the scratch-dir fixture, in a
 test file or fixture other than that fixture, must be the initializer of
 an `await using` declaration. `scratchDir`'s own disposer only exists once
 its call returns, so a bare `const x = await scratchDir(...)` (or a
 `return scratchDir(...)` handing the bare promise on) leaves a window
 between the directory's creation and anything binding its removal; a
 helper whose setup runs more than that one call goes through
 `scratchDirWith` instead, which this rule does not read calls to, since it
 is itself the guarded mechanism.

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
  literalText,
  memberName,
  nodesUnder,
  parentsOf,
  parseSource,
  type SourceText,
  type TreeNode,
  unwrapped,
} from './source-scan.test-fixture.ts';
import {
  expectFindingsAsListed,
  expectNoFindings,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/scan-findings.test-fixture.ts';

//region Temp-dir calls

/**
 Path of the one fixture allowed to call `mkdtemp` or `tmpdir` for itself:
 the single mechanism every other test or fixture goes through instead.
 */
const SCRATCH_DIR_FIXTURE_PATH = 'test-fixture/scratch-dir.test-fixture.ts';

/**
 Modules a bare `mkdtemp` import, or a namespace or default import reaching
 it, comes from.
 */
const MKDTEMP_MODULES: ReadonlySet<string> = new Set(['node:fs', 'node:fs/promises',],);

/**
 Module a bare `tmpdir` import, or a namespace or default import reaching
 it, comes from.
 */
const TMPDIR_MODULE = 'node:os';

/**
 Kept direct calls, each `path: form`, with why `scratchDir` does not
 replace it.
 */
const KEPT: Readonly<Record<string, string>> = {
  'adjudication/corpus-source.unit.test.ts: tmpdir()':
    'builds a clone directory path guaranteed not to exist, for a corpus read against a missing clone; '
      + 'nothing is created, so scratchDir would create and dispose of a directory for no reason',
};

/**
 Name an import specifier brings in, whatever the local alias.

 @param specifier - import specifier read

 @returns Its imported name, empty for a namespace or default specifier

 @example
 ```ts
 const imported = importedNameOf({ specifier, },);
 ```
 */
function importedNameOf({ specifier, }: { readonly specifier: TreeNode; },): string {
  /**
   The imported name when written as an identifier, empty otherwise.
   */
  const asIdentifier = identifierName({ node: specifier.imported, },);
  return (asIdentifier === '')
    ? literalText({ node: specifier.imported, },)
    : asIdentifier;
}

/**
 Import bindings one file brings in from the temp-dir modules: local names
 bound to `mkdtemp` or `tmpdir` by name, and local names bound to a
 namespace or default import of the module each hangs off.
 */
type TempDirBindings = {
  readonly mkdtempNames: ReadonlySet<string>;
  readonly fsObjects: ReadonlySet<string>;
  readonly tmpdirNames: ReadonlySet<string>;
  readonly osObjects: ReadonlySet<string>;
};

/**
 Reads one file's temp-dir import bindings.

 @param program - file's program

 @returns Bindings the file's imports make

 @example
 ```ts
 const bindings = tempDirBindingsOf({ program, },);
 ```
 */
function tempDirBindingsOf({ program, }: { readonly program: TreeNode; },): TempDirBindings {
  /**
   Local names bound to `mkdtemp` by name, direct or aliased.
   */
  const mkdtempNames = new Set<string>();
  /**
   Local names bound to a namespace or default import reaching `mkdtemp`.
   */
  const fsObjects = new Set<string>();
  /**
   Local names bound to `tmpdir` by name, direct or aliased.
   */
  const tmpdirNames = new Set<string>();
  /**
   Local names bound to a namespace or default import reaching `tmpdir`.
   */
  const osObjects = new Set<string>();
  for (const node of nodesUnder({ root: program, },)) {
    if (node.type !== 'ImportDeclaration')
      continue;
    /**
     Module this declaration imports from.
     */
    const source = literalText({ node: node.source, },);
    for (const specifier of (node.specifiers as readonly TreeNode[])) {
      /**
       Local name this specifier binds.
       */
      const local = identifierName({ node: specifier.local, },);
      if (local === '')
        continue;
      /**
       Whether this specifier reaches every export of the module, rather
       than one name.
       */
      const wholeModule = (specifier.type === 'ImportNamespaceSpecifier') || (specifier.type === 'ImportDefaultSpecifier');
      if (MKDTEMP_MODULES.has(source,)) {
        if (wholeModule)
          fsObjects.add(local,);
        else if ((specifier.type === 'ImportSpecifier') && (importedNameOf({ specifier, },) === 'mkdtemp'))
          mkdtempNames.add(local,);
      }
      if (source === TMPDIR_MODULE) {
        if (wholeModule)
          osObjects.add(local,);
        else if ((specifier.type === 'ImportSpecifier') && (importedNameOf({ specifier, },) === 'tmpdir'))
          tmpdirNames.add(local,);
      }
    }
  }
  return {
    mkdtempNames,
    fsObjects,
    tmpdirNames,
    osObjects,
  };
}

/**
 Which temp-dir function a call reaches, given a file's bindings.

 @param node - node read

 @param bindings - file's temp-dir import bindings

 @returns `'mkdtemp'` or `'tmpdir()'`, empty for any other call

 @example
 ```ts
 const form = calleeFormOf({ node, bindings, },);
 ```
 */
function calleeFormOf(
  {
    node,
    bindings,
  }: {
    readonly node: TreeNode;
    readonly bindings: TempDirBindings;
  },
): string {
  if (node.type !== 'CallExpression')
    return '';
  /**
   The callee inside any wrappers.
   */
  const { inner, } = unwrapped({ node: node.callee, },);
  if (!isTreeNode(inner,))
    return '';
  /**
   The callee's bare name, for a direct or aliased import.
   */
  const bare = identifierName({ node: inner, },);
  if (bindings.mkdtempNames.has(bare,))
    return 'mkdtemp';
  if (bindings.tmpdirNames.has(bare,))
    return 'tmpdir()';
  if (inner.type !== 'MemberExpression')
    return '';
  /**
   Object and member names of a namespace or default-import call.
   */
  const [object, member,] = [
    identifierName({ node: inner.object, },),
    memberName({ node: inner, },),
  ];
  if (bindings.fsObjects.has(object,) && (member === 'mkdtemp'))
    return 'mkdtemp';
  return (bindings.osObjects.has(object,) && (member === 'tmpdir')) ? 'tmpdir()' : '';
}

/**
 Direct `mkdtemp` and `tmpdir()` calls in the test files and fixtures
 given, as `path: form`.

 @param files - files read; package source and `scratch-dir.test-fixture.ts`
 are skipped

 @returns Findings, sorted and without repeats

 @example
 ```ts
 const calls = tempDirCalls({ files, },);
 ```
 */
function tempDirCalls({ files, }: { readonly files: readonly SourceText[]; },): readonly string[] {
  /**
   Findings so far.
   */
  const found = new Set<string>();
  for (const file of files) {
    if ((!file.isTest) || (file.path === SCRATCH_DIR_FIXTURE_PATH))
      continue;
    /**
     This file's program and its temp-dir import bindings.
     */
    const { program, } = parseSource({ file, },);
    const bindings = tempDirBindingsOf({ program, },);
    for (const node of nodesUnder({ root: program, },)) {
      /**
       Form this node calls, empty for anything else.
       */
      const form = calleeFormOf({ node, bindings, },);
      if (form !== '')
        found.add(`${file.path}: ${form}`,);
    }
  }
  return [...found,].toSorted();
}

//endregion Temp-dir calls

//region ScratchDir binding

/**
 Suffix of the module path a bare `scratchDir` import must come from.
 */
const SCRATCH_DIR_MODULE_SUFFIX = 'scratch-dir.test-fixture.ts';

/**
 Node kinds a call's result passes through unchanged, for finding where
 that result actually lands.
 */
const VALUE_THROUGH_KINDS: ReadonlySet<string> = new Set([
  'AwaitExpression',
  'ChainExpression',
  'ParenthesizedExpression',
  'TSAsExpression',
  'TSNonNullExpression',
  'TSSatisfiesExpression',
  'TSTypeAssertion',
],);

/**
 Local names a file binds to `scratchDir` by name, direct or aliased, from
 the scratch-dir fixture.

 @param program - file's program

 @returns Local names bound to `scratchDir`

 @example
 ```ts
 const names = scratchDirNamesOf({ program, },);
 ```
 */
function scratchDirNamesOf({ program, }: { readonly program: TreeNode; },): ReadonlySet<string> {
  /**
   Names found so far.
   */
  const names = new Set<string>();
  for (const node of nodesUnder({ root: program, },)) {
    if (node.type !== 'ImportDeclaration')
      continue;
    /**
     Module this declaration imports from.
     */
    const source = literalText({ node: node.source, },);
    if (!source.endsWith(SCRATCH_DIR_MODULE_SUFFIX,))
      continue;
    for (const specifier of (node.specifiers as readonly TreeNode[])) {
      if ((specifier.type !== 'ImportSpecifier') || (importedNameOf({ specifier, },) !== 'scratchDir'))
        continue;
      /**
       Local name this specifier binds.
       */
      const local = identifierName({ node: specifier.local, },);
      if (local !== '')
        names.add(local,);
    }
  }
  return names;
}

/**
 The node holding a call's result, walking past `await` and the wrappers
 that pass a value through unchanged.

 @param node - call read

 @param parents - each node's parent

 @returns Nearest ancestor that is not itself passed through further

 @example
 ```ts
 const holder = valueHolderOf({ node: call, parents, },);
 ```
 */
function valueHolderOf(
  {
    node,
    parents,
  }: {
    readonly node: TreeNode;
    readonly parents: ReadonlyMap<TreeNode, TreeNode>;
  },
): TreeNode {
  /**
   Node reached so far, climbing past transparent wrappers.
   */
  let at = node;
  for (let parent = parents.get(at,); parent !== undefined; parent = parents.get(at,)) {
    if (!VALUE_THROUGH_KINDS.has(parent.type,))
      break;
    at = parent;
  }
  return at;
}

/**
 Whether a `scratchDir` call is bound as the initializer of an `await using`
 declaration.

 @param call - call read

 @param parents - each node's parent

 @returns Whether the call's result is awaited straight into an `await
 using` binding

 @example
 ```ts
 const bound = boundWithAwaitUsing({ call, parents, },);
 ```
 */
function boundWithAwaitUsing(
  {
    call,
    parents,
  }: {
    readonly call: TreeNode;
    readonly parents: ReadonlyMap<TreeNode, TreeNode>;
  },
): boolean {
  /**
   Where the call's result actually lands.
   */
  const holder = valueHolderOf({ node: call, parents, },);
  /**
   The declarator the holder sits in, if any.
   */
  const declarator = parents.get(holder,);
  if ((declarator === undefined) || (declarator.type !== 'VariableDeclarator') || (declarator.init !== holder))
    return false;
  /**
   The declaration the declarator sits in.
   */
  const declaration = parents.get(declarator,);
  return (declaration !== undefined) && (declaration.type === 'VariableDeclaration') && (declaration.kind === 'await using');
}

/**
 `scratchDir` calls in the test files and fixtures given that are not bound
 as the initializer of an `await using` declaration, as `path: scratchDir
 not bound with await using`.

 @param files - files read; package source and `scratch-dir.test-fixture.ts`
 are skipped

 @returns Findings, sorted and without repeats

 @example
 ```ts
 const unbound = unboundScratchDirCalls({ files, },);
 ```
 */
function unboundScratchDirCalls({ files, }: { readonly files: readonly SourceText[]; },): readonly string[] {
  /**
   Findings so far.
   */
  const found = new Set<string>();
  for (const file of files) {
    if ((!file.isTest) || (file.path === SCRATCH_DIR_FIXTURE_PATH))
      continue;
    /**
     This file's program, its parents and its `scratchDir` local names.
     */
    const { program, } = parseSource({ file, },);
    const parents = parentsOf({ program, },);
    const names = scratchDirNamesOf({ program, },);
    if (names.size === 0)
      continue;
    for (const node of nodesUnder({ root: program, },)) {
      if (node.type !== 'CallExpression')
        continue;
      /**
       The callee inside any wrappers.
       */
      const { inner, } = unwrapped({ node: node.callee, },);
      /**
       The callee's bare name, for a direct or aliased import.
       */
      const bare = identifierName({ node: inner, },);
      if ((!names.has(bare,)) || boundWithAwaitUsing({ call: node, parents, },))
        continue;
      found.add(`${file.path}: scratchDir not bound with await using`,);
    }
  }
  return [...found,].toSorted();
}

//endregion ScratchDir binding

await describe({
  name: 'test temp directories (ledger B108)',
  children: [
    describe({
      name: 'direct mkdtemp and tmpdir calls in tests (ledger B108)',
      children: [
        it({
          name: 'FINDS a bare import, a local alias, a namespace member and a default-import member of each '
            + 'function, and leaves `mkdtempDisposable`, a `tmpdir` from another module, a non-test file and the '
            + 'scratchDir fixture itself',
          fn: async () => {
            expect(tempDirCalls({
              files: [
                {
                  path: 'cat.unit.test.ts',
                  text: [
                    'import {',
                    '  mkdtemp,',
                    '  mkdtemp as napDig,',
                    '  mkdtempDisposable,',
                    '  readFile,',
                    '} from \'node:fs/promises\';',
                    'import { tmpdir as denTemp, } from \'node:os\';',
                    'export const nap = async () => {',
                    '  await mkdtemp(\'whiskers-\');',
                    '  await napDig(\'whiskers-\');',
                    '  await mkdtempDisposable(\'whiskers-\');',
                    '  await readFile(\'bowl.json\');',
                    '  denTemp();',
                    '};',
                  ].join('\n',),
                  isTest: true,
                },
                {
                  path: 'kitten.unit.test.ts',
                  text: [
                    'import * as fs from \'node:fs/promises\';',
                    'import os from \'node:os\';',
                    'export const purr = async () => {',
                    '  await fs.mkdtemp(\'tabby-\');',
                    '  await fs.readFile(\'saucer.json\');',
                    '  os.tmpdir();',
                    '};',
                  ].join('\n',),
                  isTest: true,
                },
                {
                  path: 'siamese.unit.test.ts',
                  text: [
                    'import { tmpdir, } from \'./not-os.ts\';',
                    'export const nap = () => tmpdir();',
                  ].join('\n',),
                  isTest: true,
                },
                {
                  path: 'calico.ts',
                  text: [
                    'import { mkdtemp, } from \'node:fs/promises\';',
                    'export const nap = () => mkdtemp(\'calico-\');',
                  ].join('\n',),
                  isTest: false,
                },
                {
                  path: SCRATCH_DIR_FIXTURE_PATH,
                  text: [
                    'import { mkdtemp, } from \'node:fs/promises\';',
                    'import { tmpdir, } from \'node:os\';',
                    'export const scratchDir = async () => mkdtemp(tmpdir(),);',
                  ].join('\n',),
                  isTest: true,
                },
              ],
            },),).toEqual([
              'cat.unit.test.ts: mkdtemp',
              'cat.unit.test.ts: tmpdir()',
              'kitten.unit.test.ts: mkdtemp',
              'kitten.unit.test.ts: tmpdir()',
            ],);
          },
        },),
        it({
          name: 'FINDS NO DIRECT MKDTEMP OR TMPDIR CALL across the package\'s tests but through scratchDir, the '
            + 'scratchDir fixture itself, and the kept sites `KEPT` names',
          fn: async () => {
            expectFindingsAsListed({
              findings: tempDirCalls({ files: await readPackageSource(), },),
              listed: Object.keys(KEPT,).toSorted(),
            },);
          },
        },),
      ],
    },),
    describe({
      name: 'scratchDir calls bound with await using (ledger B108)',
      children: [
        it({
          name: 'FINDS a bare binding and a returned call, and leaves an await-using binding and a scratchDirWith '
            + 'call',
          fn: async () => {
            expect(unboundScratchDirCalls({
              files: [
                {
                  path: 'whiskers-bare.unit.test.ts',
                  text: [
                    'import { scratchDir, } from \'./scratch-dir.test-fixture.ts\';',
                    'export const nap = async () => {',
                    '  const dir = await scratchDir({ prefix: \'whiskers-bare-\', },);',
                    '  return dir.path;',
                    '};',
                  ].join('\n',),
                  isTest: true,
                },
                {
                  path: 'whiskers-returned.unit.test.ts',
                  text: [
                    'import { scratchDir, } from \'./scratch-dir.test-fixture.ts\';',
                    'export const nap = () => scratchDir({ prefix: \'whiskers-returned-\', },);',
                  ].join('\n',),
                  isTest: true,
                },
                {
                  path: 'whiskers-proper.unit.test.ts',
                  text: [
                    'import { scratchDir, } from \'./scratch-dir.test-fixture.ts\';',
                    'export const nap = async () => {',
                    '  await using dir = await scratchDir({ prefix: \'whiskers-proper-\', },);',
                    '  return dir.path;',
                    '};',
                  ].join('\n',),
                  isTest: true,
                },
                {
                  path: 'whiskers-with.unit.test.ts',
                  text: [
                    'import { scratchDir, scratchDirWith, } from \'./scratch-dir.test-fixture.ts\';',
                    'export const nap = async () => scratchDirWith({',
                    '  prefix: \'whiskers-with-\',',
                    '  setup: async function nothing() { return {}; },',
                    '},);',
                    'export const also = async () => {',
                    '  await using dir = await scratchDir({ prefix: \'whiskers-with-also-\', },);',
                    '  return dir.path;',
                    '};',
                  ].join('\n',),
                  isTest: true,
                },
              ],
            },),).toEqual([
              'whiskers-bare.unit.test.ts: scratchDir not bound with await using',
              'whiskers-returned.unit.test.ts: scratchDir not bound with await using',
            ],);
          },
        },),
        it({
          name: 'FINDS NO SCRATCHDIR CALL across the package\'s tests left unbound by await using',
          fn: async () => {
            expectNoFindings({ findings: unboundScratchDirCalls({ files: await readPackageSource(), },), },);
          },
        },),
      ],
    },),
  ],
},);
