/**
 Guards the one way production code starts a child process: with an `env`
 built by `childEnvironment` (`child-process-environment.ts`), which states
 every variable whose name ends in `_API_KEY`, and every setting of the
 package, as absent. The owner's rule is that no child of production code is
 handed a credential; a child started with no `env`, or with the parent's
 environment whole, inherits every key the run holds, and a child's own log,
 a crash dump, a tool's debug output, or a hook or alias in a repository's
 configuration that git runs can then leak it.
 `test-children-keyless.unit.test.ts` holds the tests' side of the rule.

 WHAT THE SCAN READS, in package source (not tests and fixtures): the
 bindings of every spawning module a file imports (`node:child_process`,
 `nano-spawn`, `execa`, `cross-spawn`, `node:worker_threads`), under whatever
 local name, as a default, named or namespace import, and through
 `promisify` (a declarator whose initializer is `promisify` or `util.promisify`
 of a binding binds a spawning function too). Every call of such a binding,
 and every `new Worker`, must pass an object literal as its options (the
 third argument, or the second for a `Worker`, or the second where that is an
 object literal) holding an `env` property whose value is a call of
 `childEnvironment` or of `corpusGitEnvironment`. A finding names the file,
 the line and which of these fails. A builder other than `childEnvironment`
 must itself call it, which the scan reads in the file that declares it.
 Every other way to load a spawning module (a dynamic `import`, `require`,
 `getBuiltinModule`, a re-export) is a finding as it stands, since a binding
 reached so cannot be followed.

 OUT OF THE SCAN'S REACH: a spawning function reached through an alias
 (`const run = spawn`), a call through a computed non-literal member, a
 module that wraps another language's launcher, and a spawn started inside a
 workspace package this package imports (`@monochromatic-dev/git-executable-core`
 reads files and starts nothing, read by hand). `env` built correctly says
 nothing about the other options.

 THE FIXTURE CASE COMES FIRST, so the package-wide case is read against a
 scan shown able to find each form (ledger M21). Fixtures are cat-themed;
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
  literalText,
  memberName,
  nodesUnder,
  parseSource,
  type SourceText,
  type TreeNode,
  unwrapped,
} from './source-scan.test-fixture.ts';
import { expectNoFindings, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scan-findings.test-fixture.ts';

//region Spawning bindings
// Which local names start a child, read from the file's own imports.

/**
 Modules that start a child process or a thread with its own environment.
 */
const SPAWN_MODULES: ReadonlySet<string> = new Set([
  'child_process',
  'cluster',
  'cross-spawn',
  'execa',
  'nano-spawn',
  'node:child_process',
  'node:cluster',
  'node:worker_threads',
  'worker_threads',
],);

/**
 Names a named import of a spawning module starts a child with.
 */
const SPAWN_NAMES: ReadonlySet<string> = new Set([
  'Worker',
  'exec',
  'execFile',
  'execFileSync',
  'execSync',
  'fork',
  'spawn',
  'spawnSync',
],);

/**
 Functions whose call result is an environment with every credential stated absent.
 */
const ENVIRONMENT_BUILDERS: ReadonlySet<string> = new Set([
  'childEnvironment',
  'corpusGitEnvironment',
],);

/**
 The builder every other builder is made from.
 */
const BASE_BUILDER = 'childEnvironment';

/**
 What a file's imports bind.
 */
type Bindings = {
  /**
   Local names that start a child when called.
   */
  readonly callable: ReadonlySet<string>;

  /**
   Local names of a whole spawning module, whose members start a child.
   */
  readonly namespaces: ReadonlySet<string>;
};

/**
 Reads what a file's static imports of spawning modules bind, and what
 `promisify` makes of them.

 @param program - file's program

 @returns Local names that start a child, and those of whole modules

 @example
 ```ts
 const { callable, } = bindingsOf({ program, },);
 ```
 */
function bindingsOf({ program, }: { readonly program: TreeNode; },): Bindings {
  /**
   Names that start a child when called.
   */
  const callable = new Set<string>();
  /**
   Names of whole modules.
   */
  const namespaces = new Set<string>();
  /**
   Nodes still to visit.
   */
  const pending: TreeNode[] = [program,];
  /**
   Declarators, read after every import is known.
   */
  const declarators: TreeNode[] = [];
  for (let node = pending.pop(); node !== undefined; node = pending.pop()) {
    pending.push(...childNodes({ node, },),);
    if (node.type === 'VariableDeclarator')
      declarators.push(node,);
    if ((node.type !== 'ImportDeclaration') || (!SPAWN_MODULES.has(literalText({ node: node.source, },),)))
      continue;
    for (const specifier of (node.specifiers as readonly TreeNode[])) {
      /**
       Local name the import binds.
       */
      const local = identifierName({ node: specifier.local, },);
      if (specifier.type === 'ImportNamespaceSpecifier') {
        namespaces.add(local,);
        continue;
      }
      /**
       Name the module exports it under, `default` for a default import.
       */
      const imported = (specifier.type === 'ImportDefaultSpecifier')
        ? 'default'
        : identifierName({ node: specifier.imported, },);
      if ((imported === 'default') || SPAWN_NAMES.has(imported,))
        callable.add(local,);
    }
  }
  for (const declarator of declarators) {
    /**
     Initializer, without wrappers.
     */
    const { inner, } = unwrapped({ node: declarator.init, },);
    if ((!isTreeNode(inner,)) || (inner.type !== 'CallExpression') || (!isTreeNode(inner.callee,)))
      continue;
    /**
     Name `promisify` is called by, bare or through a member.
     */
    const callee = (inner.callee.type === 'MemberExpression')
      ? memberName({ node: inner.callee, },)
      : identifierName({ node: inner.callee, },);
    /**
     Function handed to `promisify`.
     */
    const wrapped = identifierName({ node: (inner.arguments as readonly unknown[])[0], },);
    if ((callee === 'promisify') && callable.has(wrapped,))
      callable.add(identifierName({ node: declarator.id, },),);
  }
  return {
    callable,
    namespaces,
  };
}

//endregion Spawning bindings

//region Starts of a child
// Which calls start a child, and whether the environment each hands is built.

/**
 Line a node starts on.

 @param file - file holding the node

 @param node - node read

 @returns One-based line number

 @example
 ```ts
 const line = lineOf({ file, node, },);
 ```
 */
function lineOf({ file, node, }: { readonly file: SourceText; readonly node: TreeNode; },): number {
  return file.text
    .slice(
    0,
    node.start,
    )
    .split('\n',)
    .length;
}

/**
 Name of the function a call or construction starts a child with, or empty.

 @param node - node read

 @param bindings - what the file's imports bind

 @returns Name as written, empty where the node starts no child

 @example
 ```ts
 const name = spawnerName({ node, bindings, },); // 'cp.execFileSync'
 ```
 */
function spawnerName({ node, bindings, }: { readonly node: TreeNode; readonly bindings: Bindings; },): string {
  if ((node.type !== 'CallExpression') && (node.type !== 'NewExpression'))
    return '';
  /**
   The callee, without wrappers.
   */
  const { inner: callee, } = unwrapped({ node: node.callee, },);
  if (!isTreeNode(callee,))
    return '';
  /**
   Name written bare.
   */
  const bare = identifierName({ node: callee, },);
  if (bindings.callable.has(bare,))
    return bare;
  if (callee.type !== 'MemberExpression')
    return '';
  /**
   Namespace and member of a call such as `cp.fork(...)`.
   */
  const owner = identifierName({ node: callee.object, },);
  const member = memberName({ node: callee, },);
  return (bindings.namespaces.has(owner,) && SPAWN_NAMES.has(member,)) ? `${owner}.${member}` : '';
}

/**
 Why the environment a start of a child hands is not provably built.

 @param node - call or construction that starts a child

 @returns Reason, empty where the options hold an `env` built by a builder

 @example
 ```ts
 const reason = envRefusal({ node, },); // 'passes no env'
 ```
 */
function envRefusal({ node, }: { readonly node: TreeNode; },): string {
  /**
   The arguments.
   */
  const args = node.arguments as readonly unknown[];
  /**
   The options: third argument, second for a `Worker` or where an object.
   */
  const options = (node.type === 'NewExpression')
    ? unwrapped({ node: args[1], },).inner
    : unwrapped({ node: args[2] ?? args[1], },).inner;
  if ((!isTreeNode(options,)) || ((options.type !== 'ObjectExpression') && (args[2] === undefined)))
    return 'passes no options object, so the child inherits the parent\'s whole environment';
  if (options.type !== 'ObjectExpression')
    return 'passes options that are not an object literal, so the scan cannot read their env';
  /**
   The options' properties.
   */
  const properties = options.properties as readonly TreeNode[];
  if (properties.some(function isSpread(property,): boolean {
    return property.type === 'SpreadElement';
  },))
    return 'passes options with a spread, which may carry an env the scan cannot read';
  /**
   The `env` property.
   */
  const env = properties.find(function isEnv(property,): boolean {
    return (property.type === 'Property') && (property.computed !== true)
      && ((identifierName({ node: property.key, },) === 'env') || (literalText({ node: property.key, },) === 'env'));
  },);
  if (env === undefined)
    return 'passes options with no env, so the child inherits the parent\'s whole environment';
  /**
   The `env` value, without wrappers.
   */
  const { inner: value, } = unwrapped({ node: env.value, },);
  if ((!isTreeNode(value,)) || (value.type !== 'CallExpression')
    || (!ENVIRONMENT_BUILDERS.has(identifierName({ node: value.callee, },),)))
    return 'passes an env that is not a call of childEnvironment or corpusGitEnvironment';
  return '';
}

/**
 Whether a node is a call of the base builder.

 @param node - node read

 @returns Whether it calls `childEnvironment`

 @example
 ```ts
 const calls = callsBaseBuilder({ node, },);
 ```
 */
function callsBaseBuilder({ node, }: { readonly node: TreeNode; }): boolean {
  return (node.type === 'CallExpression') && (identifierName({ node: node.callee, },) === BASE_BUILDER);
}

/**
 Findings of one file: each start of a child whose env is not built, each
 other load of a spawning module, and each builder that is not made from the
 base builder.

 @param file - package source read

 @returns One finding per fault, in source order

 @example
 ```ts
 const found = findingsOfFile({ file, },);
 ```
 */
function findingsOfFile({ file, }: { readonly file: SourceText; },): readonly string[] {
  /**
   The file's program.
   */
  const { program, } = parseSource({ file, },);
  /**
   What its imports bind.
   */
  const bindings = bindingsOf({ program, },);
  /**
   Findings so far, each with the offset that orders it.
   */
  const found: { readonly at: number; readonly text: string; }[] = [];
  /**
   Nodes still to visit.
   */
  const pending: TreeNode[] = [program,];
  for (let node = pending.pop(); node !== undefined; node = pending.pop()) {
    pending.push(...childNodes({ node, },),);
    /**
     Place every finding of this node is named by.
     */
    const place = `${file.path}:${lineOf({ file, node, },)}`;
    /**
     Function this node starts a child with, if it does.
     */
    const spawner = spawnerName({ node, bindings, },);
    if (spawner !== '') {
      /**
       Why its env is not built, empty where it is.
       */
      const refusal = envRefusal({ node, },);
      if (refusal !== '')
        found.push({ at: node.start, text: `${place}: ${(node.type === 'NewExpression') ? 'new ' : ''}${spawner} ${refusal}`, },);
    }
    if (((node.type === 'ExportAllDeclaration') || (node.type === 'ExportNamedDeclaration')
      || (node.type === 'ImportExpression'))
      && SPAWN_MODULES.has(literalText({ node: node.source, },),)) {
      found.push({
        at: node.start,
        text: `${place}: loads ${literalText({ node: node.source, },)} other than by a static import, which the scan cannot follow`,
      },);
    }
    if (node.type === 'CallExpression') {
      /**
       Name called, bare or through a member.
       */
      const called = (isTreeNode(node.callee,) && (node.callee.type === 'MemberExpression'))
        ? memberName({ node: node.callee, },)
        : identifierName({ node: node.callee, },);
      /**
       The module the call loads, where it loads one.
       */
      const loaded = literalText({ node: (node.arguments as readonly unknown[])[0], },);
      if (((called === 'require') || (called === 'getBuiltinModule')) && SPAWN_MODULES.has(loaded,))
        found.push({ at: node.start, text: `${place}: loads ${loaded} other than by a static import, which the scan cannot follow`, },);
    }
    if (((node.type === 'FunctionDeclaration') || (node.type === 'FunctionExpression'))
      && ENVIRONMENT_BUILDERS.has(identifierName({ node: node.id, },),)
      && (identifierName({ node: node.id, },) !== BASE_BUILDER)) {
      /**
       Whether the builder's own body calls the base builder.
       */
      const madeFromBase = childNodes({ node, },)
        .some(function holdsBaseCall(child,): boolean {
          return nodesUnder({ root: child, },).some(function isBaseCall(inner,): boolean {
            return callsBaseBuilder({ node: inner, },);
          },);
        },);
      if (!madeFromBase) {
        found.push({
          at: node.start,
          text: `${place}: ${identifierName({ node: node.id, },)} is a builder that does not call ${BASE_BUILDER}`,
        },);
      }
    }
  }
  return found
    .toSorted(function bySource(left, right,): number {
      return left.at - right.at;
    },)
    .map(function textOf(finding,): string {
      return finding.text;
    },);
}

/**
 Starts of a child in package source that do not hand a built environment.

 @param files - files read; tests and fixtures are skipped

 @returns Findings, by file in input order and by source order within a file

 @example
 ```ts
 const children = childrenWithoutBuiltEnvironment({ files, },);
 ```
 */
function childrenWithoutBuiltEnvironment({ files, }: { readonly files: readonly SourceText[]; },): readonly string[] {
  return files
    .filter(function isSource(file,): boolean {
      return !file.isTest;
    },)
    .flatMap(function findings(file,): readonly string[] {
      return findingsOfFile({ file, },);
    },);
}

//endregion Starts of a child

await describe({
  name: 'children started by production code are handed no credential',
  children: [
    it({
      name: 'FINDS each form of start of a child that hands no built environment, and each other load of '
        + 'a spawning module, and passes a start with a built environment, a regular expression\'s exec and tests',
      fn: async () => {
        expect(childrenWithoutBuiltEnvironment({
          files: [
            {
              path: 'cat.ts',
              text: [
                'import spawn from \'nano-spawn\';',
                'export const nap = async () => await spawn(\'git\', [\'status\',],);',
                'export const purr = async () => await spawn(\'git\', [\'status\',], { cwd: \'.\', },);',
                'export const knead = async () => await spawn(\'git\', [], { env: process.env, },);',
                'export const groom = async () => await spawn(\'git\', [], { ...options, },);',
                'export const stretch = async () => await spawn(\'git\', [], options,);',
                'export const clean = async () => await spawn(\'git\', [], { env: childEnvironment({ parent: process.env, },), },);',
              ].join('\n',),
              isTest: false,
            },
            {
              path: 'kitten.ts',
              text: [
                'import { execFile as launch, } from \'node:child_process\';',
                'import { promisify, } from \'node:util\';',
                'const launched = promisify(launch,);',
                'export const pounce = async () => await launched(\'git\', [],);',
                'export const yawn = () => launch(\'git\', [], { env: corpusGitEnvironment(), }, () => {},);',
              ].join('\n',),
              isTest: false,
            },
            {
              path: 'tabby.ts',
              text: [
                'import * as cp from \'child_process\';',
                'import { Worker, } from \'node:worker_threads\';',
                'export const sleep = () => cp.execFileSync(\'git\', [],);',
                'export const find = () => cp.fork(\'cat.js\', [], { env: {}, },);',
                'export const hunt = () => new Worker(\'cat.js\',);',
                'export const rest = () => new Worker(\'cat.js\', { env: childEnvironment({ parent: process.env, },), },);',
                'export const groomed = async () => await import(\'node:child_process\');',
                'export const stalk = () => require(\'execa\');',
                'export const prowl = () => process.getBuiltinModule(\'node:child_process\');',
                'export { fork, } from \'node:child_process\';',
              ].join('\n',),
              isTest: false,
            },
            {
              path: 'calico.ts',
              text: [
                'export const roll = (text: string) => /cat/u.exec(text,);',
                'export const own = () => exec(\'git status\',);',
                'export function corpusGitEnvironment() { return {}; }',
              ].join('\n',),
              isTest: false,
            },
            {
              path: 'ginger.unit.test.ts',
              text: 'import spawn from \'nano-spawn\'; export const run = () => spawn(\'git\', [],);',
              isTest: true,
            },
          ],
        },),).toEqual([
          'cat.ts:2: spawn passes no options object, so the child inherits the parent\'s whole environment',
          'cat.ts:3: spawn passes options with no env, so the child inherits the parent\'s whole environment',
          'cat.ts:4: spawn passes an env that is not a call of childEnvironment or corpusGitEnvironment',
          'cat.ts:5: spawn passes options with a spread, which may carry an env the scan cannot read',
          'cat.ts:6: spawn passes options that are not an object literal, so the scan cannot read their env',
          'kitten.ts:4: launched passes no options object, so the child inherits the parent\'s whole environment',
          'tabby.ts:3: cp.execFileSync passes no options object, so the child inherits the parent\'s whole environment',
          'tabby.ts:4: cp.fork passes an env that is not a call of childEnvironment or corpusGitEnvironment',
          'tabby.ts:5: new Worker passes no options object, so the child inherits the parent\'s whole environment',
          'tabby.ts:7: loads node:child_process other than by a static import, which the scan cannot follow',
          'tabby.ts:8: loads execa other than by a static import, which the scan cannot follow',
          'tabby.ts:9: loads node:child_process other than by a static import, which the scan cannot follow',
          'tabby.ts:10: loads node:child_process other than by a static import, which the scan cannot follow',
          'calico.ts:3: corpusGitEnvironment is a builder that does not call childEnvironment',
        ],);
      },
    },),
    it({
      name: 'FINDS NO START OF A CHILD ACROSS THE PACKAGE\'S SOURCE that hands the parent\'s environment, '
        + 'or none built by childEnvironment',
      fn: async () => {
        /**
         Every package file, tests and fixtures among them to be skipped.
         */
        const files = await readPackageSource();
        expectNoFindings({ findings: childrenWithoutBuiltEnvironment({ files, },), },);
        expect(files.some(function isHelper(file,): boolean {
          return file.path === 'typography/child-process-environment.ts';
        },),).toBe(true,);
      },
    },),
  ],
},);
