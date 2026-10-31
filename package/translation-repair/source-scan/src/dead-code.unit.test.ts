/**
 Guards against code nothing reaches (audit area six, ledger B19, B20, B30,
 B31): a top-level declaration private to its file must be named in that
 file beyond its own declaration, and every top-level function, class and
 value must be reached from production. `isAnchored` outlived the change
 that stopped reading it, and `uniqueRosterModelIds` the loop it served, and
 nothing reported either.

 REACH, NOT NAMING (ledger B30). The first form of this guard counted a test,
 or a barrel's re-export list, as naming an export, so 57 functions only
 tests called passed it: wrappers over parts production calls apart, modules
 built beside the live path and superseded, and test support shipped as
 package source. Production's roots are every source file's module-level
 statements that declare nothing, which is where the runner entries call
 their main; a declaration is reached once reached code names it outside a
 type, and its body or initializer is then reached code. Matching is by
 name, so a name two declarations share keeps both reached, and the error
 is only toward calling code reached.

 VALUES TOO (ledger B31). The second form tracked functions alone and ran
 every other top-level statement as a root, so a constant only tests read
 passed, and so did everything its initializer named: a type proof that
 could no longer fail, seat sets only the seat tests read, and the test
 seats themselves. A value named only in type positions (`typeof X`) is
 unreached by design: its initializer runs, but nothing reads what it
 builds.

 THE FIXTURES COME FIRST, so the package-wide case is read against a scan
 shown able to find each kind (ledger M21). Fixtures are cat-themed; the
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
  isTreeNode,
  parseSource,
  type SourceText,
  type TreeNode,
} from './source-scan.test-fixture.ts';
import { expectRecordAsListed, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scan-findings.test-fixture.ts';

/**
 Declarations nothing reaches, by kind, each as `path#name`.
 */
type DeadCode = {
  /**
   Private declarations nothing in their file names.
   */
  readonly private: readonly string[];

  /**
   Declarations production does not reach, the private ones already listed
   left out, and the allowed seams too.
   */
  readonly unreached: readonly string[];

  /**
   Allowed seams production reaches or no file declares, so the allowance
   no longer describes the source.
   */
  readonly staleSeams: readonly string[];
};

/**
 A top-level function, class or value, whether its file exports it inline,
 and the node whose names it reaches when it is reached.
 */
type TopDeclaration = {
  /**
   Its name.
   */
  readonly name: string;

  /**
   Whether it is declared with `export`.
   */
  readonly exported: boolean;

  /**
   Its function or class declaration, its function expression, or its
   value's initializer.
   */
  readonly body: TreeNode;
};

/**
 A top-level declaration and the file declaring it.
 */
type LocatedDeclaration = TopDeclaration & {
  /**
   File declaring it, relative to `src`.
   */
  readonly path: string;
};

/**
 Seams production never calls and the guard allows, each as `path#name`,
 with why no fixture can stand in for it.

 `resetRunSpend` zeroes the process-wide spend meter between the test cases
 one process runs; the meter is module-private state, which no fixture can
 reach without an export (ledger B30, a choice the owner may veto for a meter
 passed in).

 `proseMaskBuilds` counts the prose masks built since the process began, so a test sees how many slices one page pass parsed
 (each slice once, n159, X5); the count is module-private state, which no fixture can reach without an export.

 `renderRunLogLine` renders one line of a record file as the console sink shows it, which the reader's tests hold against
 the real console sink; the reader itself renders a record it has parsed already. `linesOfLog` cuts the text of a run
 folder into the lines a parser takes, which the tests of the reports compare against; the reports read lines directly.
 */
const PACKAGE_SEAMS: ReadonlySet<string> = new Set([
  'glossary/corpus-run/run-log-read.ts#renderRunLogLine',
  'run/corpus-run/report-log-read.ts#linesOfLog',
  'shared-contract/run-spend-meter.ts#resetRunSpend',
  'typography/typography-prose-mask.ts#proseMaskBuilds',
],);

/**
 Syntax that stands in a type position, whose names run nothing.
 */
const TYPE_POSITIONS: ReadonlySet<string> = new Set([
  'TSTypeAnnotation',
  'TSTypeAliasDeclaration',
  'TSInterfaceDeclaration',
  'TSTypeParameterInstantiation',
  'TSTypeParameterDeclaration',
  'TSTypeQuery',
  'TSTypeReference',
],);

/**
 How often each identifier stands in a file's tree.

 @param program - parsed program

 @returns Count per name

 @example
 ```ts
 const counts = identifierCounts({ program, },);
 ```
 */
function identifierCounts({ program, }: { readonly program: TreeNode; },): ReadonlyMap<string, number> {
  /**
   Counts so far.
   */
  const counts = new Map<string, number>();
  /**
   Nodes still to visit.
   */
  const pending: TreeNode[] = [program,];
  while (pending.length > 0) {
    /**
     Node visited now.
     */
    const node = pending.pop() as TreeNode;
    if ((node.type === 'Identifier') && ((typeof node.name) === 'string'))
      counts.set(node.name as string, (counts.get(node.name as string,) ?? 0) + 1,);
    pending.push(...childNodes({ node, },),);
  }
  return counts;
}

/**
 Identifiers a node names where they run, type positions left out.

 @param node - node read

 @returns Names, each once

 @example
 ```ts
 const names = runtimeNames({ node: statement, },);
 ```
 */
function runtimeNames({ node, }: { readonly node: TreeNode; },): ReadonlySet<string> {
  /**
   Names so far.
   */
  const names = new Set<string>();
  /**
   Nodes still to visit.
   */
  const pending: TreeNode[] = [node,];
  while (pending.length > 0) {
    /**
     Node visited now.
     */
    const current = pending.pop() as TreeNode;
    if (!TYPE_POSITIONS.has(current.type,)) {
      if ((current.type === 'Identifier') && ((typeof current.name) === 'string'))
        names.add(current.name as string,);
      pending.push(...childNodes({ node: current, },),);
    }
  }
  return names;
}

/**
 The declaration a top-level statement carries, past an inline `export`.

 @param statement - top-level statement

 @returns Statement, or the declaration its `export` wraps

 @example
 ```ts
 const declaration = declarationOf({ statement, },);
 ```
 */
function declarationOf({ statement, }: { readonly statement: TreeNode; },): TreeNode {
  return ((statement.type === 'ExportNamedDeclaration') && isTreeNode(statement.declaration,))
    ? statement.declaration
    : statement;
}

/**
 Whether a variable declarator binds one plain name, the only form this
 guard tracks; a destructuring declarator stays module-level code.

 @param declarator - variable declarator

 @returns True for `name = ...` and `name`

 @example
 ```ts
 const tracked = bindsOneName({ declarator, },);
 ```
 */
function bindsOneName({ declarator, }: { readonly declarator: TreeNode; },): boolean {
  return isTreeNode(declarator.id,) && (declarator.id.type === 'Identifier');
}

/**
 The bindings one declaration introduces: a function or class declaration,
 or variables bound to a plain name.

 @param declaration - top-level declaration

 @param exported - whether an inline `export` wraps it

 @returns Its bindings, none for any other declaration

 @example
 ```ts
 const bindings = bindingsDeclared({ declaration, exported: false, },);
 ```
 */
function bindingsDeclared(
  {
    declaration,
    exported,
  }: {
    readonly declaration: TreeNode;
    readonly exported: boolean;
  },
): readonly TopDeclaration[] {
  if (((declaration.type === 'FunctionDeclaration') || (declaration.type === 'ClassDeclaration'))
    && isTreeNode(declaration.id,)) {
    return [{
      name: declaration.id.name as string,
      exported,
      body: declaration,
    },];
  }
  if (declaration.type !== 'VariableDeclaration')
    return [];
  return (declaration.declarations as readonly TreeNode[])
    .filter(function tracked(declarator,): boolean {
      return bindsOneName({ declarator, },);
    },)
    .map(function bound(declarator,): TopDeclaration {
      return {
        name: (declarator.id as TreeNode).name as string,
        exported,
        body: isTreeNode(declarator.init,) ? declarator.init : declarator,
      };
    },);
}

/**
 The functions, classes and values a file declares at its top level.

 @param program - parsed program

 @returns Its top-level declarations

 @example
 ```ts
 const declarations = topDeclarations({ program, },);
 ```
 */
function topDeclarations({ program, }: { readonly program: TreeNode; },): readonly TopDeclaration[] {
  return (program.body as readonly TreeNode[]).flatMap(function declared(statement,): readonly TopDeclaration[] {
    return bindingsDeclared({
      declaration: declarationOf({ statement, },),
      exported: (statement.type === 'ExportNamedDeclaration') && isTreeNode(statement.declaration,),
    },);
  },);
}

/**
 A file's module-level code: every top-level statement but imports, export
 lists and the declarations it tracks, which run their bodies only when
 named, plus any destructuring declarator, whose initializer runs as it
 stands.

 @param program - parsed program

 @returns Code that runs when the file loads

 @example
 ```ts
 const roots = moduleLevelCode({ program, },);
 ```
 */
function moduleLevelCode({ program, }: { readonly program: TreeNode; },): readonly TreeNode[] {
  return (program.body as readonly TreeNode[]).flatMap(function runsOnLoad(statement,): readonly TreeNode[] {
    if ((statement.type === 'ImportDeclaration') || (statement.type === 'ExportAllDeclaration'))
      return [];
    if ((statement.type === 'ExportNamedDeclaration') && (!isTreeNode(statement.declaration,)))
      return [];
    /**
     The declaration the statement carries, past an inline `export`.
     */
    const declaration = declarationOf({ statement, },);
    if (declaration.type === 'VariableDeclaration') {
      return (declaration.declarations as readonly TreeNode[]).filter(function untracked(declarator,): boolean {
        return !bindsOneName({ declarator, },);
      },);
    }
    return (bindingsDeclared({
      declaration,
      exported: false,
    },)
      .length === 0)
      ? [statement,]
      : [];
  },);
}

/**
 Names a file exports by an export list (`export { name, };`).

 @param program - parsed program

 @returns Local names listed for export

 @example
 ```ts
 const listed = listedExports({ program, },);
 ```
 */
function listedExports({ program, }: { readonly program: TreeNode; },): ReadonlySet<string> {
  return new Set((program.body as readonly TreeNode[])
    .filter(function isExportList(statement,): boolean {
      return (statement.type === 'ExportNamedDeclaration') && (!isTreeNode(statement.declaration,));
    },)
    .flatMap(function names(statement,): readonly string[] {
      return (statement.specifiers as readonly TreeNode[]).map(function local(specifier,): string {
        /**
         Local binding the specifier exports.
         */
        const bound = specifier.local as TreeNode;
        return (bound.name ?? bound.value) as string;
      },);
    },),);
}

/**
 Private declarations nothing in their file names beyond their declaration.

 @param path - file path, for the location

 @param program - parsed program

 @returns Each as `path#name`

 @example
 ```ts
 const unnamed = unnamedPrivates({ path: 'cat.ts', program, },);
 ```
 */
function unnamedPrivates(
  {
    path,
    program,
  }: {
    readonly path: string;
    readonly program: TreeNode;
  },
): readonly string[] {
  /**
   Identifier counts in the file.
   */
  const counts = identifierCounts({ program, },);
  /**
   Names its export lists carry.
   */
  const listed = listedExports({ program, },);
  return topDeclarations({ program, },)
    .filter(function unnamedPrivate({ name, exported, },): boolean {
      return (!exported) && (!listed.has(name,)) && ((counts.get(name,) ?? 0) <= 1);
    },)
    .map(function located({ name, },): string {
      return `${path}#${name}`;
    },);
}

/**
 Declarations production's module-level code reaches, by walking the names
 reached code carries.

 @param declarations - every top-level declaration in the source files

 @param roots - every source file's module-level code

 @returns Declarations reached

 @example
 ```ts
 const reached = reachedDeclarations({ declarations, roots, },);
 ```
 */
function reachedDeclarations(
  {
    declarations,
    roots,
  }: {
    readonly declarations: readonly LocatedDeclaration[];
    readonly roots: readonly TreeNode[];
  },
): ReadonlySet<LocatedDeclaration> {
  /**
   Declarations under each name; two files may declare the same one.
   */
  const byName = Map.groupBy(
    declarations,
    function nameOf({ name, },): string {
      return name;
    },
  );
  /**
   Declarations reached so far.
   */
  const reached = new Set<LocatedDeclaration>();
  /**
   Reached code whose names are still to read.
   */
  const pending: TreeNode[] = [...roots,];
  while (pending.length > 0) {
    /**
     Code read now.
     */
    const code = pending.pop() as TreeNode;
    /**
     Declarations its names reach for the first time.
     */
    const fresh = [...runtimeNames({ node: code, },),]
      .flatMap(function named(name,): readonly LocatedDeclaration[] {
        return byName.get(name,) ?? [];
      },)
      .filter(function unseen(located,): boolean {
        return !reached.has(located,);
      },);
    for (const located of fresh) {
      reached.add(located,);
      pending.push(located.body,);
    }
  }
  return reached;
}

/**
 Every declaration nothing reaches across a set of files; only source files
 are scanned and only source files reach, so a test naming a declaration
 never makes it reached.

 @param files - package's files

 @param seams - declarations allowed to go unreached, each as `path#name`

 @returns Dead code by kind, and allowances that no longer hold

 @example
 ```ts
 const dead = deadCode({ files, seams: PACKAGE_SEAMS, },);
 ```
 */
function deadCode(
  {
    files,
    seams,
  }: {
    readonly files: readonly SourceText[];
    readonly seams: ReadonlySet<string>;
  },
): DeadCode {
  /**
   Every source file, parsed once.
   */
  const parsed = files
    .filter(function isSource(file,): boolean {
      return !file.isTest;
    },)
    .map(function parse(file,): {
      readonly path: string;
      readonly program: TreeNode;
    } {
      return {
        path: file.path,
        program: parseSource({ file, },).program,
      };
    },);
  /**
   Private declarations their file never names.
   */
  const unnamed = parsed
    .flatMap(unnamedPrivates,)
    .toSorted();
  /**
   Every top-level declaration, located.
   */
  const declarations = parsed.flatMap(function located({ path, program, },): readonly LocatedDeclaration[] {
    return topDeclarations({ program, },).map(function withPath(top,): LocatedDeclaration {
      return {
        ...top,
        path,
      };
    },);
  },);
  /**
   Declarations production reaches.
   */
  const reached = reachedDeclarations({
    declarations,
    roots: parsed.flatMap(function rootsOf({ program, },): readonly TreeNode[] {
      return moduleLevelCode({ program, },);
    },),
  },);
  /**
   Every declaration production does not reach, as `path#name`.
   */
  const notReached = declarations
    .filter(function unreachedHere(located,): boolean {
      return !reached.has(located,);
    },)
    .map(function toKey({ path, name, },): string {
      return `${path}#${name}`;
    },);
  return {
    private: unnamed,
    unreached: notReached
      .filter(function unlisted(key,): boolean {
        return (!unnamed.includes(key,)) && (!seams.has(key,));
      },)
      .toSorted(),
    staleSeams: [...seams,]
      .filter(function heldNoLonger(key,): boolean {
        return !notReached.includes(key,);
      },)
      .toSorted(),
  };
}

/**
 The cat fixtures: one source file holding a declaration of every kind, a
 barrel re-exporting some, a bowl whose one module-level statement reads a
 value built from others, and a test reading some.
 */
const CAT_FILES: readonly SourceText[] = [
  {
    path: 'cat.ts',
    text: [
      'function nap(): number { return 1; }',
      'function purr(): number { return 2; }',
      'export function loaf(): number { return purr(); }',
      'export function stretch(): number { return 3; }',
      'export const groom = (): number => 4;',
      'const knead = (): number => 5;',
      'export { knead, };',
      'export function yawn(): number { return 6; }',
      'function hiss(): number { return 7; }',
      'export function pounce(): number { return hiss(); }',
      'export function shape(): number { return 8; }',
      'export type Shape = ReturnType<typeof shape>;',
      'export const PERCH = 9 as unknown as ReturnType<typeof shape>;',
      'export function tally(): number { return 10; }',
      'export const TOTAL = tally();',
      'export function reset(): void {}',
      'const WHISKERS = 11;',
      'export const TAIL = 12;',
      'export const LITTER = [13,];',
      'export const LITTER_SIZE = LITTER.length;',
      'export const PAWS = [4,] as const;',
      'export type Paw = typeof PAWS[number];',
      'export class Tabby {}',
      'export class Calico {}',
      'const naps = new Map<string, number>();',
      'export function doze(): number { naps.set(\'cat\', 1,); return naps.size; }',
    ].join('\n',),
    isTest: false,
  },
  {
    path: 'barrel.ts',
    text: 'export { yawn, pounce, reset, TAIL, } from \'./cat.ts\';',
    isTest: false,
  },
  {
    path: 'bowl.ts',
    text: [
      'import { loaf, knead, TOTAL, PERCH, Calico, doze, } from \'./cat.ts\';',
      'export const meal = loaf() + knead() + TOTAL + PERCH + doze();',
      'console.log(meal, new Calico(),);',
    ].join('\n',),
    isTest: false,
  },
  {
    path: 'cat.unit.test.ts',
    text: 'import { groom, reset, TAIL, } from \'./cat.ts\';\ngroom();\nreset();\nconsole.log(TAIL,);',
    isTest: true,
  },
];

await describe({
  name: 'code nothing reaches',
  children: [
    it({
      name: 'FINDS a private function or value its file never names, and every declaration production does '
        + 'not reach: a function nothing names, one only a test calls, ones only a barrel re-exports and '
        + 'the private one only they call, one named only in types, a value only a test reads, a value only '
        + 'an unreached value reads and that one, a value named only in types, and a class nothing '
        + 'constructs; KEEPS what a module-level statement reaches, directly or through a value, a '
        + 'function or a module-level map a reached function reads, and the allowed seam',
      fn: async () => {
        expect(deadCode({
          files: CAT_FILES,
          seams: new Set(['cat.ts#reset',],),
        },),).toEqual({
          private: ['cat.ts#WHISKERS', 'cat.ts#nap',],
          unreached: [
            'cat.ts#LITTER',
            'cat.ts#LITTER_SIZE',
            'cat.ts#PAWS',
            'cat.ts#TAIL',
            'cat.ts#Tabby',
            'cat.ts#groom',
            'cat.ts#hiss',
            'cat.ts#pounce',
            'cat.ts#shape',
            'cat.ts#stretch',
            'cat.ts#yawn',
          ],
          staleSeams: [],
        },);
      },
    },),
    it({
      name: 'NAMES an allowed seam production reaches, or no file declares, as stale',
      fn: async () => {
        expect(deadCode({
          files: CAT_FILES,
          seams: new Set([
            'cat.ts#reset',
            'cat.ts#tally',
            'cat.ts#scratch',
          ],),
        },).staleSeams,).toEqual(['cat.ts#scratch', 'cat.ts#tally',],);
      },
    },),
    it({
      name: 'FINDS NO CODE NOTHING REACHES across the package\'s source, and no stale seam',
      fn: async () => {
        /**
         Every package file, tests among them, which never reach.
         */
        const files = await readPackageSource();
        expect(files.some(function isSource(file,): boolean {
          return !file.isTest;
        },),).toBe(true,);
        expectRecordAsListed({
          found: deadCode({
            files,
            seams: PACKAGE_SEAMS,
          },),
          listed: {
            private: [],
            unreached: [],
            staleSeams: [],
          },
        },);
      },
    },),
  ],
},);
