/**
 Guards the one way a runner reads its command line (ledger B75): each
 runner hands `process.argv` and its own name to `reportingRefusals`, which
 reads the whole line against the runner's declaration in
 `corpus-run/command-lines.ts` before the body starts. Runners that found
 their own flags in `process.argv` read any spelling but the exact token as
 not written, and one that reads the line for itself again would read flags
 its declaration, its usage line and its refusals never name.

 WHAT THE SCAN READS, in the package's source; tests and test fixtures build
 their own lines and are not read. `argv` read off `process`, or off a
 default or namespace import of `node:process`, written as a member, an
 indexed member, an optional member or through a parenthesis or type
 assertion; `argv` destructured from `process`; `argv` imported by name from
 `node:process`; `parseArgs`, which reads `process.argv` when handed no
 arguments, imported by name from `node:util` or read off a default or
 namespace import of it; and any dynamic import of either module. A read of
 `process.argv` is allowed only as the `argv` a `reportingRefusals` call is
 handed; that call must name its own runner as a written string. Out of the
 scan's reach: `process` reached through `globalThis` or handed on whole, and
 a property name built at run time.

 THE FIXTURE CASE COMES FIRST, so the package-wide case is read against a
 scan shown able to find each form (ledger M21). Fixtures are cat-themed; the
 package case reads this package's own source.

 @module
 */

import { basename, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  readPackageSource,
  runnerEntriesOf,
} from './family-source.test-fixture.ts';
import {
  identifierName,
  isTreeNode,
  literalText,
  nodesUnder,
  parseSource,
  type SourceText,
  type TreeNode,
  unwrapped,
} from './source-scan.test-fixture.ts';
import { expectRecordAsListed, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scan-findings.test-fixture.ts';

/**
 Specifiers of the module `process` is.
 */
const PROCESS_MODULES: ReadonlySet<string> = new Set(['node:process', 'process',],);

/**
 Specifiers of the module `parseArgs` comes from.
 */
const UTIL_MODULES: ReadonlySet<string> = new Set(['node:util', 'util',],);

/**
 The call every runner hands its line to.
 */
const HAND_OFF = 'reportingRefusals';

/**
 Reads allowed outside a runner's hand-off, each as `path: what`, with why.
 */
const ALLOWED: Readonly<Record<string, string>> = {
  'run/corpus-run/command-line.ts: imports parseArgs from node:util':
    'the one reader of the whole line, which hands parseArgs the arguments after the script and reads its '
      + 'tokens against the declaration',
};

/**
 Folder the runner entries sit in, under a family package's `src`.
 */
const RUNNER_FOLDER = '/corpus-run/';

/**
 Name of the property a member expression or object property names, as
 written: an identifier, or a string in brackets or quotes.

 @param node - member expression or property

 @param key - field holding the name: `property` for a member, `key` for a
 property

 @returns Name, empty when it is built at run time

 @example
 ```ts
 const name = propertyName({ node: member, key: 'property', },); // 'argv' for process['argv']
 ```
 */
function propertyName(
  {
    node,
    key,
  }: {
    readonly node: TreeNode;
    readonly key: 'property' | 'key';
  },
): string {
  /**
   The name node.
   */
  const named = node[key];
  if (node.computed === true)
    return literalText({ node: named, },);
  return identifierName({ node: named, },) || literalText({ node: named, },);
}

/**
 Local names a file binds to a whole module, by default or namespace import.

 @param nodes - file's nodes

 @param modules - specifiers of the module

 @returns Local names

 @example
 ```ts
 const names = wholeModuleNames({ nodes, modules: PROCESS_MODULES, },); // ['proc'] for import proc from 'node:process'
 ```
 */
function wholeModuleNames(
  {
    nodes,
    modules,
  }: {
    readonly nodes: readonly TreeNode[];
    readonly modules: ReadonlySet<string>;
  },
): ReadonlySet<string> {
  return new Set(
    nodes
      .filter(function importsModule(node,): boolean {
        return (node.type === 'ImportDeclaration') && modules.has(literalText({ node: node.source, },),);
      },)
      .flatMap(function wholeNames(declaration,): readonly string[] {
        return (declaration.specifiers as readonly TreeNode[])
          .filter(function isWhole(specifier,): boolean {
            return (specifier.type === 'ImportDefaultSpecifier') || (specifier.type === 'ImportNamespaceSpecifier');
          },)
          .map(function localName(specifier,): string {
            return identifierName({ node: specifier.local, },);
          },);
      },),
  );
}

/**
 What one file's source reads of the command line outside a runner's
 hand-off, and whether it hands its line on as a runner does.

 @param file - file read

 @returns Each read as `path: what`, and whether a hand-off names this file's
 runner and hands it `process.argv`

 @example
 ```ts
 const { reads, handsOff, } = commandLineReadsOf({ file, },);
 ```
 */
function commandLineReadsOf({ file, }: { readonly file: SourceText; },): {
  readonly reads: readonly string[];
  readonly handsOff: boolean;
} {
  /**
   Every node in the file.
   */
  const nodes = nodesUnder({ root: parseSource({ file, },).program, },);

  /**
   Names `process` goes by here: the global and any whole import of it.
   */
  const processNames = new Set([
    'process',
    ...wholeModuleNames({
      nodes,
      modules: PROCESS_MODULES,
    },),
  ],);

  /**
   Names a whole import of the util module goes by here.
   */
  const utilNames = wholeModuleNames({
    nodes,
    modules: UTIL_MODULES,
  },);

  /**
   Whether a member expression reads `argv` off `process`.

   @param node - node read

   @returns True for `process.argv` in any of its spellings

   @example
   ```ts
   const reads = readsArgv({ node, },);
   ```
   */
  function readsArgv({ node, }: { readonly node: unknown; },): boolean {
    /**
     The node without its wrappers.
     */
    const member = unwrapped({ node, },).inner;
    return isTreeNode(member,)
      && (member.type === 'MemberExpression')
      && processNames.has(identifierName({ node: unwrapped({ node: member.object, },).inner, },),)
      && (propertyName({
        node: member,
        key: 'property',
      },) === 'argv');
  }

  /**
   Runner this file is, by its name, when it sits among the runners.
   */
  const runner = `/${file.path}`.includes(RUNNER_FOLDER,)
    ? basename(
      file.path,
      '.ts',
    )
    : '';

  /**
   Each hand-off call: the `argv` it is handed and the runner it names.
   */
  const handOffs = nodes
    .filter(function isHandOff(node,): boolean {
      return (node.type === 'CallExpression') && (identifierName({ node: node.callee, },) === HAND_OFF);
    },)
    .map(function handedOff(node,): {
      readonly argv: unknown;
      readonly what: string;
    } {
      /**
       The object the call is handed, absent when it is handed none.
       */
      const [options,] = node.arguments as readonly unknown[];

      /**
       Its properties, by name as written.
       */
      const properties = new Map(
        (isTreeNode(options,) && (options.type === 'ObjectExpression')
          ? options.properties as readonly TreeNode[]
          : [])
          .map(function named(property,): [string, TreeNode,] {
            return [
              propertyName({
                node: property,
                key: 'key',
              },),
              property,
            ];
          },),
      );
      return {
        argv: properties.get('argv',)?.value,
        what: literalText({ node: properties.get('what',)?.value, },),
      };
    },);

  /**
   Start offsets of the `process.argv` reads a hand-off is given.
   */
  const handedOn = new Set(
    handOffs
      .filter(function handsArgv({ argv, },): boolean {
        return readsArgv({ node: argv, },);
      },)
      .map(function startOf({ argv, },): number {
        return (unwrapped({ node: argv, },).inner as TreeNode).start;
      },),
  );

  /**
   What the file reads, as `path: what`, starting with what its hand-offs
   are handed wrongly.
   */
  const reads: string[] = handOffs.flatMap(function handedWrongly({
    argv,
    what,
  },): readonly string[] {
    /**
     Whether the call names a runner other than this file's, or none.
     */
    const namesAnother = (what === '') || (what !== runner);
    return [
      ...(readsArgv({ node: argv, },) ? [] : [`${file.path}: hands ${HAND_OFF} an argv other than process.argv`,]),
      ...(namesAnother ? [`${file.path}: hands ${HAND_OFF} what ${JSON.stringify(what,)}`,] : []),
    ];
  },);

  for (const node of nodes) {
    if ((node.type === 'MemberExpression') && readsArgv({ node, },) && (!handedOn.has(node.start,)))
      reads.push(`${file.path}: reads argv off process`,);
    if (
      (node.type === 'MemberExpression')
        && utilNames.has(identifierName({ node: unwrapped({ node: node.object, },).inner, },),)
        && (propertyName({
          node,
          key: 'property',
        },) === 'parseArgs')
    )
      reads.push(`${file.path}: reads parseArgs off node:util`,);
    if (
      (node.type === 'VariableDeclarator')
        && isTreeNode(node.id,)
        && (node.id.type === 'ObjectPattern')
        && processNames.has(identifierName({ node: unwrapped({ node: node.init, },).inner, },),)
        && (node.id.properties as readonly TreeNode[]).some(function namesArgv(property,): boolean {
          return (property.type === 'Property') && (propertyName({
            node: property,
            key: 'key',
          },) === 'argv');
        },)
    )
      reads.push(`${file.path}: destructures argv from process`,);
    if (node.type === 'ImportExpression') {
      /**
       Module the dynamic import names, empty when built at run time.
       */
      const from = literalText({ node: node.source, },);
      if (PROCESS_MODULES.has(from,) || UTIL_MODULES.has(from,) || (from === ''))
        reads.push(`${file.path}: imports ${JSON.stringify(from,)} dynamically`,);
    }
    if (node.type === 'ImportDeclaration') {
      /**
       Module the import names.
       */
      const from = literalText({ node: node.source, },);
      for (const specifier of node.specifiers as readonly TreeNode[]) {
        /**
         Name imported, whatever the local alias.
         */
        const imported = identifierName({ node: specifier.imported, },) || literalText({ node: specifier.imported, },);
        if (PROCESS_MODULES.has(from,) && (imported === 'argv'))
          reads.push(`${file.path}: imports argv from node:process`,);
        if (UTIL_MODULES.has(from,) && (imported === 'parseArgs'))
          reads.push(`${file.path}: imports parseArgs from node:util`,);
      }
    }
  }
  return {
    reads,
    handsOff: (runner !== '') && handOffs.some(function isOwn({
      argv,
      what,
    },): boolean {
      return readsArgv({ node: argv, },) && (what === runner);
    },),
  };
}

/**
 What the package's source reads of the command line outside the runners'
 hand-offs, and which files hand their line on.

 @param files - files read; tests and fixtures are skipped

 @returns Reads as `path: what`, sorted with repeats kept, and the files that
 hand their line on, sorted

 @example
 ```ts
 const { reads, handOffs, } = commandLineReads({ files, },);
 ```
 */
function commandLineReads({ files, }: { readonly files: readonly SourceText[]; },): {
  readonly reads: readonly string[];
  readonly handOffs: readonly string[];
} {
  /**
   Each source file's reads and hand-off.
   */
  const scanned = files
    .filter(function isSource({ isTest, },): boolean {
      return !isTest;
    },)
    .map(function scanOne(file,): {
      readonly path: string;
      readonly reads: readonly string[];
      readonly handsOff: boolean;
    } {
      return {
        path: file.path,
        ...commandLineReadsOf({ file, },),
      };
    },);
  return {
    reads: scanned
      .flatMap(function readsOf({ reads, },): readonly string[] {
        return reads;
      },)
      .toSorted(),
    handOffs: scanned
      .filter(function handsOff({ handsOff: hands, },): boolean {
        return hands;
      },)
      .map(function pathOf({ path, },): string {
        return path;
      },)
      .toSorted(),
  };
}

await describe({
  name: 'command-line reads (ledger B75)',
  children: [
    it({
      name: 'FINDS argv read off process in every spelling, destructured or imported, parseArgs imported or read '
        + 'off the module, dynamic imports, and hand-offs naming another runner or another argv, and leaves '
        + 'a runner\'s own hand-off, other names and tests',
      fn: async () => {
        expect(commandLineReads({
          files: [
            {
              path: 'corpus-run/nap-report.ts',
              text: 'await reportingRefusals({ what: \'nap-report\', argv: process.argv, run: async () => {}, },);',
              isTest: false,
            },
            {
              path: 'corpus-run/purr-report.ts',
              text: [
                'await reportingRefusals({ what: \'nap-report\', argv: process.argv, run: async () => {}, },);',
                'await reportingRefusals({ what: \'purr-report\', argv: [\'node\', \'purr.mjs\',], run: '
                  + 'async () => {}, },);',
              ].join('\n',),
              isTest: false,
            },
            {
              path: 'whiskers.ts',
              text: [
                'import { parseArgs as split, promisify, } from \'node:util\';',
                'import { argv as typed, env, } from \'node:process\';',
                'import proc from \'node:process\';',
                'import * as utils from \'util\';',
                'const { argv, } = process;',
                'const { env: home, } = process;',
                'export const naps = [',
                '  process.argv[2], process[\'argv\'], process?.argv, (process as NodeJS.Process).argv,',
                '  process!.argv, proc.argv, utils.parseArgs, process.env, typed, argv, split, promisify, env,',
                '  home, await import(\'node:process\'), await import(\'node:fs\'),',
                '];',
              ].join('\n',),
              isTest: false,
            },
            {
              path: 'cat.unit.test.ts',
              text: 'import { parseArgs, } from \'node:util\'; export const nap = [process.argv, parseArgs,];',
              isTest: true,
            },
          ],
        },),).toEqual({
          reads: [
            'corpus-run/purr-report.ts: hands reportingRefusals an argv other than process.argv',
            'corpus-run/purr-report.ts: hands reportingRefusals what "nap-report"',
            'whiskers.ts: destructures argv from process',
            'whiskers.ts: imports "node:process" dynamically',
            'whiskers.ts: imports argv from node:process',
            'whiskers.ts: imports parseArgs from node:util',
            'whiskers.ts: reads argv off process',
            'whiskers.ts: reads argv off process',
            'whiskers.ts: reads argv off process',
            'whiskers.ts: reads argv off process',
            'whiskers.ts: reads argv off process',
            'whiskers.ts: reads argv off process',
            'whiskers.ts: reads parseArgs off node:util',
          ],
          handOffs: ['corpus-run/nap-report.ts',],
        },);
      },
    },),
    it({
      name: 'READS THE COMMAND LINE only in each runner\'s own hand-off and the one reader, and every runner '
        + 'the build makes hands its line on',
      fn: async () => {
        /**
         Every file of the family.
         */
        const files = await readPackageSource();
        /**
         Runner entry files, as the scan names paths.
         */
        const runners = runnerEntriesOf({ files, },)
          .map(function scanPath({ path, },): string {
            return path;
          },)
          .toSorted();
        expectRecordAsListed({
          found: commandLineReads({ files, },),
          listed: {
            reads: Object.keys(ALLOWED,).toSorted(),
            handOffs: runners,
          },
        },);
      },
    },),
  ],
},);
